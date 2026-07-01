//! Deploy debug-logs command.

use anyhow::Context;
use chrono::{DateTime, SecondsFormat, Utc};
use clap::Args as ClapArgs;
use std::collections::{HashSet, VecDeque};
use std::time::Duration;
use turnkey_client::TurnkeyP256ApiKey;
use turnkey_client::generated::external::data::v1::{LogLine, Timestamp};
use turnkey_client::generated::{
    GetTvcDeploymentDebugLogsRequest, GetTvcDeploymentDebugLogsResponse,
};

const DEFAULT_FOLLOW_POLL_INTERVAL_SECONDS: i64 = 2;
const FOLLOW_OVERLAP_SECONDS: i64 = 2;
const RECENT_LOG_LINE_CAPACITY: usize = 1000;

fn follow_poll_interval_seconds_parser() -> clap::builder::RangedI64ValueParser<i64> {
    clap::value_parser!(i64).range(1..=i64::MAX - FOLLOW_OVERLAP_SECONDS)
}

fn non_negative_i32_parser() -> clap::builder::RangedI64ValueParser<i32> {
    clap::value_parser!(i32).range(0..)
}

fn non_negative_i64_parser() -> clap::builder::RangedI64ValueParser<i64> {
    clap::value_parser!(i64).range(0..)
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

Use `--follow` to keep polling for new log lines after the initial log window.
Follow mode polls every 2 seconds by default and requests a 2-second overlap
on each follow-up poll to avoid missing late-arriving log lines.
By default, output omits the platform timestamp that Kubernetes prepends to each
log line; pass `--include-platform-timestamp` to show it.
See `tvc app create --help`, `tvc deploy create --help`, and `tvc --help`
for more info."#;

/// Fetch debug logs for a deployment.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = LONG_ABOUT)]
pub struct Args {
    /// ID of the deployment.
    #[arg(short = 'd', long, env = "TVC_DEPLOY_ID")]
    pub deploy_id: String,

    /// Keep polling for new lines.
    #[arg(long, env = "TVC_DEBUG_LOGS_FOLLOW")]
    pub follow: bool,

    /// Seconds to wait between follow-mode polls.
    #[arg(
        long,
        env = "TVC_DEBUG_LOGS_FOLLOW_POLL_INTERVAL_SECONDS",
        default_value_t = DEFAULT_FOLLOW_POLL_INTERVAL_SECONDS,
        value_parser = follow_poll_interval_seconds_parser(),
        allow_hyphen_values = true
    )]
    pub follow_poll_interval_seconds: i64,

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
}

/// Run the `deploy debug-logs` command.
pub async fn run(args: Args) -> anyhow::Result<()> {
    let auth = crate::client::build_client().await?;

    let request = DebugLogQueryRequest {
        organization_id: auth.org_id,
        deployment_id: args.deploy_id,
        follow: args.follow,
        follow_poll_interval_seconds: args.follow_poll_interval_seconds,
        tail_lines: args.tail_lines,
        since_seconds: args.since_seconds,
        include_platform_timestamp: args.include_platform_timestamp,
    };

    query_debug_logs(&auth.client, request).await
}

#[derive(Clone, Debug)]
struct DebugLogQueryRequest {
    organization_id: String,
    deployment_id: String,
    follow: bool,
    follow_poll_interval_seconds: i64,
    tail_lines: i32,
    since_seconds: i64,
    include_platform_timestamp: bool,
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
    fn into_followup_request(self) -> Self {
        Self {
            tail_lines: 0,
            since_seconds: self.follow_poll_interval_seconds + FOLLOW_OVERLAP_SECONDS,
            ..self
        }
    }
}

async fn query_debug_logs(
    client: &turnkey_client::TurnkeyClient<TurnkeyP256ApiKey>,
    request: DebugLogQueryRequest,
) -> anyhow::Result<()> {
    let mut log_printer =
        DebugLogPrinter::new(request.include_platform_timestamp, RECENT_LOG_LINE_CAPACITY);

    let (current_request, followup_request) = if request.follow {
        (request.clone(), Some(request.into_followup_request()))
    } else {
        (request, None)
    };

    let response = fetch_debug_logs(client, current_request).await?;
    log_printer.print_response(&response);

    let Some(followup_request) = followup_request else {
        return Ok(());
    };

    eprintln!("Connected; polling for debug logs...");

    let follow_poll_interval =
        Duration::from_secs(followup_request.follow_poll_interval_seconds as u64);
    loop {
        tokio::time::sleep(follow_poll_interval).await;
        let response = fetch_debug_logs(client, followup_request.clone()).await?;
        log_printer.print_response(&response);
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
    fn new(replica: &str, line: &LogLine) -> Option<Self> {
        let ts = line.ts.as_ref()?;

        Some(Self {
            replica: replica.into(),
            content: line.content.clone(),
            seconds: ts.seconds.clone(),
            nanos: ts.nanos.clone(),
        })
    }
}

#[derive(Debug)]
struct RecentLogLines {
    seen: HashSet<LogLineKey>,
    order: VecDeque<LogLineKey>,
    capacity: usize,
}

impl RecentLogLines {
    fn new(capacity: usize) -> Self {
        Self {
            seen: HashSet::new(),
            order: VecDeque::new(),
            capacity,
        }
    }

    fn insert(&mut self, key: LogLineKey) -> bool {
        if self.capacity == 0 {
            return true;
        }

        if !self.seen.insert(key.clone()) {
            return false;
        }

        self.order.push_back(key);

        while self.order.len() > self.capacity {
            if let Some(oldest) = self.order.pop_front() {
                self.seen.remove(&oldest);
            }
        }

        true
    }

    /// Records timestamped lines and returns whether they are new; lines missing timestamps always pass.
    ///
    /// NOTE: Mutating filter
    fn record_if_new(&mut self, replica: &str, line: &LogLine) -> bool {
        match LogLineKey::new(replica, line) {
            Some(key) => self.insert(key),
            None => true,
        }
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.seen.len()
    }
}

#[derive(Debug)]
struct DebugLogPrinter {
    recent_lines: RecentLogLines,
    include_platform_timestamp: bool,
}

impl DebugLogPrinter {
    fn new(include_platform_timestamp: bool, recent_line_capacity: usize) -> Self {
        Self {
            recent_lines: RecentLogLines::new(recent_line_capacity),
            include_platform_timestamp,
        }
    }

    fn print_response(&mut self, response: &GetTvcDeploymentDebugLogsResponse) {
        let recent_lines = &mut self.recent_lines;
        let include_platform_timestamp = self.include_platform_timestamp;

        // `record_if_new` updates the `recent_lines` cache while filtering duplicates.
        response
            .entries
            .iter()
            .filter_map(|entry| {
                entry
                    .line
                    .as_ref()
                    .map(|line| (entry.replica_label.as_str(), line))
            })
            .filter(|(replica, line)| recent_lines.record_if_new(replica, line))
            .for_each(|(replica, line)| {
                println!(
                    "{}",
                    format_log_line(replica, line, include_platform_timestamp)
                );
            });
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

    #[test]
    fn request_maps_to_unary_api_request() {
        let request = DebugLogQueryRequest {
            organization_id: "org".to_string(),
            deployment_id: "deployment".to_string(),
            follow: true,
            follow_poll_interval_seconds: DEFAULT_FOLLOW_POLL_INTERVAL_SECONDS,
            tail_lines: 10,
            since_seconds: 30,
            include_platform_timestamp: true,
        };

        let api_request: GetTvcDeploymentDebugLogsRequest = request.into();

        assert_eq!(api_request.organization_id, "org");
        assert_eq!(api_request.deployment_id, "deployment");
        assert_eq!(api_request.tail_lines, 10);
        assert_eq!(api_request.since_seconds, 30);
    }

    #[test]
    fn into_followup_request_uses_overlapping_since_window() {
        let request = DebugLogQueryRequest {
            organization_id: "org".to_string(),
            deployment_id: "deployment".to_string(),
            follow: true,
            follow_poll_interval_seconds: 7,
            tail_lines: 100,
            since_seconds: 0,
            include_platform_timestamp: false,
        };

        let followup = request.clone().into_followup_request();

        assert_eq!(followup.tail_lines, 0);
        assert_eq!(followup.since_seconds, 9);
        assert_eq!(followup.organization_id, request.organization_id);
        assert_eq!(followup.deployment_id, request.deployment_id);
    }

    #[test]
    fn log_line_key_matches_duplicate_lines() {
        let line = log_line("hello", Some(timestamp("1710000000", "123456789")));

        assert_eq!(
            LogLineKey::new("replica 1/3", &line),
            LogLineKey::new("replica 1/3", &line)
        );
    }

    #[test]
    fn log_line_key_requires_timestamp() {
        let line = log_line("hello", None);

        assert_eq!(LogLineKey::new("replica 1/3", &line), None);
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
    fn recent_log_lines_treats_untimestamped_lines_as_new() {
        let line = log_line("hello", None);
        let mut recent = RecentLogLines::new(RECENT_LOG_LINE_CAPACITY);

        assert!(recent.record_if_new("replica 1/2", &line));
        assert!(recent.record_if_new("replica 1/2", &line));
        assert_eq!(recent.len(), 0);
    }

    #[test]
    fn recent_log_lines_dedupes_timestamped_lines() {
        let line = log_line("hello", Some(timestamp("1710000000", "1")));
        let mut recent = RecentLogLines::new(RECENT_LOG_LINE_CAPACITY);

        assert!(recent.record_if_new("replica 1/2", &line));
        assert!(!recent.record_if_new("replica 1/2", &line));
        assert_eq!(recent.len(), 1);
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
        let mut printer = DebugLogPrinter::new(false, RECENT_LOG_LINE_CAPACITY);

        printer.print_response(&response);

        assert_eq!(printer.recent_lines.len(), 1);
    }

    #[test]
    fn recent_log_lines_evicts_oldest_entries() {
        let first = LogLineKey::new(
            "replica 1/2",
            &log_line("first", Some(timestamp("1710000000", "1"))),
        )
        .unwrap();
        let second = LogLineKey::new(
            "replica 1/2",
            &log_line("second", Some(timestamp("1710000000", "2"))),
        )
        .unwrap();
        let third = LogLineKey::new(
            "replica 1/2",
            &log_line("third", Some(timestamp("1710000000", "3"))),
        )
        .unwrap();
        let mut recent = RecentLogLines::new(2);

        assert!(recent.insert(first.clone()));
        assert!(recent.insert(second.clone()));
        assert!(!recent.insert(first.clone()));
        assert!(recent.insert(third));

        assert_eq!(recent.len(), 2);
        assert!(recent.insert(first));
        assert_eq!(recent.len(), 2);
    }
}
