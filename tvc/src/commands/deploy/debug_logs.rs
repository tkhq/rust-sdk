//! Deploy debug-logs command.

use crate::commands::app_status::TimestampPayload;
use crate::outcome::Outcome;
use crate::output::{Ctx, Message, StdCtx};
use crate::shell_eprintln;
use anyhow::Context;
use chrono::{DateTime, SecondsFormat, Utc};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::collections::{HashSet, VecDeque};
use std::fmt::{self, Display, Formatter};
use std::io::Write;
use std::time::Duration;
use turnkey_client::TurnkeyP256ApiKey;
use turnkey_client::generated::external::data::v1::{LogLine, Timestamp};
use turnkey_client::generated::{
    GetTvcDeploymentDebugLogsRequest, GetTvcDeploymentDebugLogsResponse,
};
use uuid::Uuid;

const DEFAULT_POLL_INTERVAL_SECONDS: i64 = 2;
const POLL_OVERLAP_SECONDS: i64 = 2;

fn poll_interval_seconds_parser() -> clap::builder::RangedI64ValueParser<i64> {
    clap::value_parser!(i64).range(1..=i64::MAX - POLL_OVERLAP_SECONDS)
}

fn non_negative_i32_parser() -> clap::builder::RangedI64ValueParser<i32> {
    clap::value_parser!(i32).range(0..)
}

fn non_negative_i64_parser() -> clap::builder::RangedI64ValueParser<i64> {
    clap::value_parser!(i64).range(0..)
}

fn positive_usize_parser() -> clap::builder::RangedU64ValueParser<usize> {
    clap::builder::RangedU64ValueParser::<usize>::new().range(1..)
}

pub(crate) const LONG_ABOUT: &str = r#"
Fetch debug logs for a deployment.

Debug logs are only available when debug mode has been enabled at both the app
and deployment levels. First, the app must opt in to allowing debug-mode
deployments with `--dangerous-enable-debug-mode-deployments`. That app-level
opt-in only permits debug deployments; it does not make every deployment
debuggable.

The specific deployment must also be created in debug mode with
`--dangerous-deploy-debug-mode`. Existing non-debug deployments cannot expose
debug logs retroactively. Create a new debug-mode deployment, then pass that
deployment ID to this command.

Use `--poll` to keep polling for new log lines after the initial log window.
Poll mode polls every 2 seconds by default and requests a 2-second overlap
on each subsequent poll to avoid missing late-arriving log lines.
By default, output omits the platform timestamp that Kubernetes prepends to each
log line; pass `--include-platform-timestamp` to show it.
The CLI dedupes timestamped lines with identical content across poll overlap windows
by default; pass `--disable-dedupe` to print every returned line.
See `tvc app create --help`, `tvc deploy create --help`, and `tvc --help`
for more info."#;

/// Fetch debug logs for a deployment.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = LONG_ABOUT)]
pub struct Args {
    /// ID of the deployment.
    #[arg(short = 'd', long, env = "TVC_DEPLOY_ID")]
    pub deploy_id: Uuid,

    /// Keep polling for new lines.
    #[arg(long, env = "TVC_DEBUG_LOGS_POLL")]
    pub poll: bool,

    /// Seconds to wait between polls.
    #[arg(
        long,
        env = "TVC_DEBUG_LOGS_POLL_INTERVAL_SECONDS",
        default_value_t = DEFAULT_POLL_INTERVAL_SECONDS,
        value_parser = poll_interval_seconds_parser(),
        allow_hyphen_values = true
    )]
    pub poll_interval_seconds: i64,

    /// Limit raw pod log history requested per replica. Filtered output may contain (many) fewer lines.
    #[arg(
        long,
        env = "TVC_DEBUG_LOGS_TAIL_LINES",
        default_value_t = 0,
        value_parser = non_negative_i32_parser(),
        allow_hyphen_values = true
    )]
    pub tail_lines: i32,

    /// Return logs newer than this many seconds ago.
    #[arg(
        long,
        env = "TVC_DEBUG_LOGS_SINCE_SECONDS",
        default_value_t = 0,
        value_parser = non_negative_i64_parser(),
        allow_hyphen_values = true
    )]
    pub since_seconds: i64,

    /// Include the platform timestamp prepended by the Kubernetes log stream.
    #[arg(long, env = "TVC_DEBUG_LOGS_INCLUDE_PLATFORM_TIMESTAMP")]
    pub include_platform_timestamp: bool,

    /// Disable client-side dedupe across poll overlap windows.
    #[arg(long, env = "TVC_DEBUG_LOGS_DISABLE_DEDUPE")]
    pub disable_dedupe: bool,

    /// Number of recently printed timestamped lines retained for poll-mode dedupe. This should
    /// only be increased if high-volume logs still show duplicates across poll overlap windows.
    #[arg(
        long,
        env = "TVC_DEBUG_LOGS_RECENT_LINE_CAPACITY",
        default_value_t = 1000,
        value_parser = positive_usize_parser()
    )]
    pub recent_line_capacity: usize,
}

/// Run the `deploy debug-logs` command.
pub async fn run(ctx: &mut StdCtx, args: Args) -> anyhow::Result<Outcome> {
    let auth = crate::client::build_client().await?;

    let request = DebugLogQueryRequest {
        organization_id: auth.org_id,
        deployment_id: args.deploy_id.to_string(),
        poll: args.poll,
        poll_interval_seconds: args.poll_interval_seconds,
        tail_lines: args.tail_lines,
        since_seconds: args.since_seconds,
        include_platform_timestamp: args.include_platform_timestamp,
        disable_dedupe: args.disable_dedupe,
        recent_line_capacity: args.recent_line_capacity,
    };

    query_debug_logs(ctx, &auth.client, request).await
}

#[derive(Clone, Debug)]
struct DebugLogQueryRequest {
    organization_id: String,
    deployment_id: String,
    poll: bool,
    poll_interval_seconds: i64,
    tail_lines: i32,
    since_seconds: i64,
    include_platform_timestamp: bool,
    disable_dedupe: bool,
    recent_line_capacity: usize,
}

impl From<DebugLogQueryRequest> for GetTvcDeploymentDebugLogsRequest {
    fn from(req: DebugLogQueryRequest) -> Self {
        Self {
            organization_id: req.organization_id,
            deployment_id: req.deployment_id,
            tail_lines: req.tail_lines,
            since_seconds: req.since_seconds,
        }
    }
}

impl DebugLogQueryRequest {
    fn into_poll_request(self) -> Self {
        Self {
            tail_lines: 0,
            since_seconds: self.poll_interval_seconds + POLL_OVERLAP_SECONDS,
            ..self
        }
    }
}

async fn query_debug_logs(
    ctx: &mut StdCtx,
    client: &turnkey_client::TurnkeyClient<TurnkeyP256ApiKey>,
    request: DebugLogQueryRequest,
) -> anyhow::Result<Outcome> {
    let deployment_id = request.deployment_id.clone();
    let mut log_printer = DebugLogPrinter::new(
        request.include_platform_timestamp,
        request.recent_line_capacity,
        request.disable_dedupe,
    );

    let (current_request, poll_request) = if request.poll {
        (request.clone(), Some(request.into_poll_request()))
    } else {
        (request, None)
    };

    let response = fetch_debug_logs(client, current_request).await?;
    let line_count = log_printer.print_response(ctx, &response)?;

    // In poll mode the loop below runs until killed (or errors), so the
    // terminal outcome is only reachable in non-poll mode.
    let Some(poll_request) = poll_request else {
        return Ok(Outcome::DeployDebugLogs(DebugLogsFetched {
            deployment_id,
            line_count,
        }));
    };

    shell_eprintln!(ctx, "Connected; polling for debug logs...")?;

    let poll_interval = Duration::from_secs(poll_request.poll_interval_seconds as u64);
    loop {
        tokio::time::sleep(poll_interval).await;
        let response = fetch_debug_logs(client, poll_request.clone()).await?;
        log_printer.print_response(ctx, &response)?;
    }
}

/// One streamed log line (NDJSON in JSON mode; the formatted line in human
/// mode). Emitted inline by `deploy debug-logs`, not part of `Outcome`.
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DebugLogLine {
    replica: String,
    content: String,
    ts: Option<TimestampPayload>,
    /// Human-rendering knob only; not part of the JSON payload.
    #[serde(skip)]
    include_platform_timestamp: bool,
}

impl Message for DebugLogLine {
    fn reason(&self) -> &'static str {
        "debug-log-line"
    }

    fn human_message(&self) -> String {
        // Rebuild the API line and defer to `format_log_line` so human output
        // stays byte-identical to the pre-outcome rendering.
        let line = LogLine {
            content: self.content.clone(),
            ts: self.ts.as_ref().map(|ts| Timestamp {
                seconds: ts.seconds.clone(),
                nanos: ts.nanos.clone(),
            }),
        };
        format_log_line(&self.replica, &line, self.include_platform_timestamp)
    }
}

/// Terminal outcome for non-poll `deploy debug-logs` runs. Machine-only: the
/// log lines themselves stream as `debug-log-line` messages, so human mode
/// prints nothing extra for this outcome.
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugLogsFetched {
    deployment_id: String,
    /// Number of log lines actually printed (after dedupe).
    line_count: usize,
}

impl Display for DebugLogsFetched {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        // Machine-only terminal outcome; no human rendering.
        Ok(())
    }
}

async fn fetch_debug_logs(
    client: &turnkey_client::TurnkeyClient<TurnkeyP256ApiKey>,
    request: DebugLogQueryRequest,
) -> anyhow::Result<GetTvcDeploymentDebugLogsResponse> {
    client
        .get_tvc_deployment_debug_logs(request.into())
        .await
        .context("failed to fetch debug logs")
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct LogLineKey {
    replica: String,
    content: String,
    seconds: String,
    nanos: String,
}

impl LogLineKey {
    fn new(replica: &str, ts: Timestamp, content: String) -> Self {
        Self {
            replica: replica.into(),
            content,
            seconds: ts.seconds,
            nanos: ts.nanos,
        }
    }
}

#[derive(Debug)]
struct RecentLogLineDeduper {
    seen: HashSet<LogLineKey>,
    order: VecDeque<LogLineKey>,
}

impl RecentLogLineDeduper {
    fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "recent log line capacity must be positive");

        Self {
            seen: HashSet::with_capacity(capacity),
            order: VecDeque::with_capacity(capacity),
        }
    }

    fn insert(&mut self, key: LogLineKey) -> bool {
        if self.seen.contains(&key) {
            return false;
        }

        if self.order.len() == self.order.capacity() {
            let oldest = self
                .order
                .pop_front()
                .expect("full recent log line cache should contain an oldest key");
            self.seen.remove(&oldest);
        }

        self.seen.insert(key.clone());
        self.order.push_back(key);

        true
    }

    /// Records timestamped lines and returns whether they are new; lines missing timestamps always pass.
    ///
    /// NOTE: Mutating filter
    fn record_if_new(&mut self, replica: &str, line: &LogLine) -> bool {
        let LogLine { content, ts } = line;
        let Some(ts) = ts else {
            return true;
        };

        let key = LogLineKey::new(replica, ts.clone(), content.clone());

        self.insert(key)
    }
}

#[derive(Debug)]
struct DebugLogPrinter {
    deduper: Option<RecentLogLineDeduper>,
    include_platform_timestamp: bool,
}

impl DebugLogPrinter {
    fn new(
        include_platform_timestamp: bool,
        recent_line_capacity: usize,
        disable_dedupe: bool,
    ) -> Self {
        let deduper = if disable_dedupe {
            None
        } else {
            Some(RecentLogLineDeduper::new(recent_line_capacity))
        };

        Self {
            deduper,
            include_platform_timestamp,
        }
    }

    fn should_print_line(&mut self, replica: &str, line: &LogLine) -> bool {
        match self.deduper.as_mut() {
            Some(deduper) => deduper.record_if_new(replica, line),
            None => true,
        }
    }

    /// Emit new log lines and return how many were printed.
    fn print_response<W: Write, W2: Write>(
        &mut self,
        ctx: &mut Ctx<W, W2>,
        response: &GetTvcDeploymentDebugLogsResponse,
    ) -> anyhow::Result<usize> {
        let mut printed = 0;
        for entry in &response.entries {
            let Some(line) = entry.line.as_ref() else {
                continue;
            };
            let replica = entry.replica_label.as_str();

            if self.should_print_line(replica, line) {
                ctx.shell().emit(&DebugLogLine {
                    replica: replica.to_string(),
                    content: line.content.clone(),
                    ts: line.ts.clone().map(Into::into),
                    include_platform_timestamp: self.include_platform_timestamp,
                })?;
                printed += 1;
            }
        }
        Ok(printed)
    }
}

fn format_log_line(replica: &str, line: &LogLine, include_platform_timestamp: bool) -> String {
    if !include_platform_timestamp {
        return format!("{replica} {}", line.content);
    }

    match line.ts.as_ref().and_then(format_timestamp) {
        Some(ts) => format!("{ts} {replica} {}", line.content),
        None => format!("{replica} {}", line.content),
    }
}

fn format_timestamp(timestamp: &Timestamp) -> Option<String> {
    let seconds = timestamp.seconds.parse::<i64>().ok()?;
    let nanos = timestamp.nanos.parse::<u32>().ok()?;

    DateTime::<Utc>::from_timestamp(seconds, nanos)
        .map(|dt| dt.to_rfc3339_opts(SecondsFormat::Nanos, true))
}

#[cfg(test)]
mod tests {
    use super::*;
    use turnkey_client::generated::TvcDeploymentDebugLogEntry;

    fn log_line(content: &str, ts: Option<Timestamp>) -> LogLine {
        LogLine {
            content: content.to_string(),
            ts,
        }
    }

    fn timestamp(seconds: &str, nanos: &str) -> Timestamp {
        Timestamp {
            seconds: seconds.to_string(),
            nanos: nanos.to_string(),
        }
    }

    fn entry(replica: &str, line: Option<LogLine>) -> TvcDeploymentDebugLogEntry {
        TvcDeploymentDebugLogEntry {
            replica_label: replica.to_string(),
            line,
        }
    }

    fn log_line_key(replica: &str, content: &str, seconds: &str, nanos: &str) -> LogLineKey {
        LogLineKey::new(replica, timestamp(seconds, nanos), content.to_string())
    }

    #[test]
    fn request_maps_to_unary_api_request() {
        let request = DebugLogQueryRequest {
            organization_id: "org".to_string(),
            deployment_id: "deployment".to_string(),
            poll: true,
            poll_interval_seconds: DEFAULT_POLL_INTERVAL_SECONDS,
            tail_lines: 10,
            since_seconds: 30,
            include_platform_timestamp: true,
            disable_dedupe: false,
            recent_line_capacity: 1000,
        };

        let api_request: GetTvcDeploymentDebugLogsRequest = request.into();

        assert_eq!(api_request.organization_id, "org");
        assert_eq!(api_request.deployment_id, "deployment");
        assert_eq!(api_request.tail_lines, 10);
        assert_eq!(api_request.since_seconds, 30);
    }

    #[test]
    fn into_poll_request_uses_overlapping_since_window() {
        let request = DebugLogQueryRequest {
            organization_id: "org".to_string(),
            deployment_id: "deployment".to_string(),
            poll: true,
            poll_interval_seconds: 7,
            tail_lines: 100,
            since_seconds: 0,
            include_platform_timestamp: false,
            disable_dedupe: false,
            recent_line_capacity: 1000,
        };

        let poll_request = request.clone().into_poll_request();

        assert_eq!(poll_request.tail_lines, 0);
        assert_eq!(poll_request.since_seconds, 9);
        assert_eq!(poll_request.organization_id, request.organization_id);
        assert_eq!(poll_request.deployment_id, request.deployment_id);
    }

    #[test]
    fn log_line_key_matches_duplicate_lines() {
        assert_eq!(
            log_line_key("replica 1/3", "hello", "1710000000", "123456789"),
            log_line_key("replica 1/3", "hello", "1710000000", "123456789")
        );
    }

    #[test]
    fn record_if_new_treats_missing_timestamp_as_new() {
        let line = log_line("hello", None);
        let mut deduper = RecentLogLineDeduper::new(1000);

        assert!(deduper.record_if_new("replica 1/3", &line));
        assert!(deduper.record_if_new("replica 1/3", &line));
        assert_eq!(deduper.seen.len(), 0);
    }

    #[test]
    fn format_log_line_omits_platform_timestamp_by_default() {
        let line = log_line("hello", Some(timestamp("1710000000", "123456789")));

        assert_eq!(
            format_log_line("replica 1/3", &line, false),
            "replica 1/3 hello"
        );
    }

    #[test]
    fn format_log_line_includes_platform_timestamp_when_requested() {
        let line = log_line("hello", Some(timestamp("1710000000", "123456789")));

        assert_eq!(
            format_log_line("replica 1/3", &line, true),
            "2024-03-09T16:00:00.123456789Z replica 1/3 hello"
        );
    }

    #[test]
    fn format_log_line_omits_timestamp_when_missing() {
        let line = log_line("hello", None);

        assert_eq!(
            format_log_line("replica 1/3", &line, true),
            "replica 1/3 hello"
        );
    }

    #[test]
    fn format_log_line_omits_invalid_timestamp() {
        let line = log_line("hello", Some(timestamp("bad", "123")));

        assert_eq!(
            format_log_line("replica 1/3", &line, true),
            "replica 1/3 hello"
        );
    }

    #[test]
    fn recent_log_line_deduper_treats_non_timestamped_lines_as_new() {
        let line = log_line("hello", None);
        let mut deduper = RecentLogLineDeduper::new(1000);

        assert!(deduper.record_if_new("replica 1/2", &line));
        assert!(deduper.record_if_new("replica 1/2", &line));
        assert_eq!(deduper.seen.len(), 0);
    }

    #[test]
    fn recent_log_line_deduper_dedupes_timestamped_lines() {
        let line = log_line("hello", Some(timestamp("1710000000", "1")));
        let mut deduper = RecentLogLineDeduper::new(1000);

        assert!(deduper.record_if_new("replica 1/2", &line));
        assert!(!deduper.record_if_new("replica 1/2", &line));
        assert_eq!(deduper.seen.len(), 1);
    }

    #[test]
    #[should_panic(expected = "recent log line capacity must be positive")]
    fn recent_log_line_deduper_requires_positive_capacity() {
        RecentLogLineDeduper::new(0);
    }

    #[test]
    fn debug_log_printer_skips_empty_entries_and_dedupes_timestamped_lines() {
        let response = GetTvcDeploymentDebugLogsResponse {
            entries: vec![
                entry(
                    "replica 1/2",
                    Some(log_line("hello", Some(timestamp("1710000000", "1")))),
                ),
                entry(
                    "replica 1/2",
                    Some(log_line("hello", Some(timestamp("1710000000", "1")))),
                ),
                entry("replica 2/2", None),
            ],
        };
        let mut printer = DebugLogPrinter::new(false, 1000, false);
        let mut ctx = Ctx::new(crate::output::EmptyShell::default(), false);

        printer.print_response(&mut ctx, &response).unwrap();

        assert_eq!(printer.deduper.as_ref().unwrap().seen.len(), 1);
    }

    #[test]
    fn debug_log_printer_prints_duplicate_timestamped_lines_when_dedupe_is_disabled() {
        let line = log_line("hello", Some(timestamp("1710000000", "1")));
        let mut printer = DebugLogPrinter::new(false, 1000, true);

        assert!(printer.should_print_line("replica 1/2", &line));
        assert!(printer.should_print_line("replica 1/2", &line));
        assert!(printer.deduper.is_none());
    }

    #[test]
    fn recent_log_line_deduper_evicts_oldest_entries() {
        let first = log_line_key("replica 1/2", "first", "1710000000", "1");
        let second = log_line_key("replica 1/2", "second", "1710000000", "2");
        let third = log_line_key("replica 1/2", "third", "1710000000", "3");
        let mut deduper = RecentLogLineDeduper::new(2);

        assert!(deduper.insert(first.clone()));
        assert!(deduper.insert(second.clone()));
        assert!(!deduper.insert(first.clone()));
        assert!(deduper.insert(third));

        assert_eq!(deduper.seen.len(), 2);
        assert!(deduper.insert(first));
        assert_eq!(deduper.seen.len(), 2);
    }

    #[test]
    fn recent_log_line_deduper_does_not_grow_past_initial_capacity() {
        let first = log_line_key("replica 1/2", "first", "1710000000", "1");
        let second = log_line_key("replica 1/2", "second", "1710000000", "2");
        let third = log_line_key("replica 1/2", "third", "1710000000", "3");
        let mut deduper = RecentLogLineDeduper::new(2);
        let initial_capacity = deduper.order.capacity();

        assert!(deduper.insert(first));
        assert!(deduper.insert(second));
        assert!(deduper.insert(third));

        assert_eq!(deduper.seen.len(), 2);
        assert_eq!(deduper.order.capacity(), initial_capacity);
    }

    fn sample_debug_log_line() -> DebugLogLine {
        DebugLogLine {
            replica: "replica 1/2".to_string(),
            content: "hello".to_string(),
            ts: Some(TimestampPayload {
                seconds: "1710000000".to_string(),
                nanos: "123456789".to_string(),
            }),
            include_platform_timestamp: false,
        }
    }

    #[test]
    fn debug_log_line_human_message_matches_previous_rendering() {
        let mut line = sample_debug_log_line();
        assert_eq!(line.human_message(), "replica 1/2 hello");

        line.include_platform_timestamp = true;
        assert_eq!(
            line.human_message(),
            "2024-03-09T16:00:00.123456789Z replica 1/2 hello"
        );
    }

    #[test]
    fn debug_logs_fetched_is_machine_only_in_human_mode() {
        assert!(DebugLogsFetched::default().to_string().is_empty());
    }
}
