//! Deploy debug-logs command.

use anyhow::{Context, bail};
use chrono::{DateTime, SecondsFormat, Utc};
use clap::Args as ClapArgs;
use std::collections::HashSet;
use std::time::Duration;
use turnkey_client::TurnkeyP256ApiKey;
use turnkey_client::generated::external::data::v1::{LogLine, Timestamp};
use turnkey_client::generated::{
    EnclaveDebugLogEntry, GetEnclaveDebugLogsRequest, GetEnclaveDebugLogsResponse,
};

const FOLLOW_POLL_INTERVAL: Duration = Duration::from_secs(2);
const FOLLOW_SINCE_SECONDS: i64 = 5;

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

    /// Limit history to the last N lines per replica.
    #[arg(long, env = "TVC_DEBUG_LOGS_TAIL_LINES", allow_hyphen_values = true)]
    pub tail_lines: Option<i32>,

    /// Return logs newer than this many seconds ago.
    #[arg(long, env = "TVC_DEBUG_LOGS_SINCE_SECONDS", allow_hyphen_values = true)]
    pub since_seconds: Option<i64>,

    /// Include the platform timestamp prepended by the Kubernetes log stream.
    #[arg(long, env = "TVC_DEBUG_LOGS_INCLUDE_PLATFORM_TIMESTAMP")]
    pub include_platform_timestamp: bool,
}

/// Run the `deploy debug-logs` command.
pub async fn run(args: Args) -> anyhow::Result<()> {
    let tail_lines = tail_lines_or_default(args.tail_lines)?;
    let since_seconds = since_seconds_or_default(args.since_seconds)?;
    let auth = crate::client::build_client().await?;

    let request = DebugLogQueryRequest {
        organization_id: auth.org_id.clone(),
        deployment_id: args.deploy_id,
        follow: args.follow,
        tail_lines,
        since_seconds,
        include_platform_timestamp: args.include_platform_timestamp,
    };

    query_debug_logs(&auth.client, &request).await
}

#[derive(Clone, Debug)]
struct DebugLogQueryRequest {
    organization_id: String,
    deployment_id: String,
    follow: bool,
    tail_lines: i32,
    since_seconds: i64,
    include_platform_timestamp: bool,
}

impl DebugLogQueryRequest {
    fn to_api_request(&self) -> GetEnclaveDebugLogsRequest {
        GetEnclaveDebugLogsRequest {
            organization_id: self.organization_id.clone(),
            deployment_id: self.deployment_id.clone(),
            tail_lines: self.tail_lines,
            since_seconds: self.since_seconds,
        }
    }

    fn followup_request(&self) -> Self {
        Self {
            tail_lines: 0,
            since_seconds: FOLLOW_SINCE_SECONDS,
            ..self.clone()
        }
    }
}

async fn query_debug_logs(
    client: &turnkey_client::TurnkeyClient<TurnkeyP256ApiKey>,
    request: &DebugLogQueryRequest,
) -> anyhow::Result<()> {
    let mut printed_lines = HashSet::new();
    let mut current_request = request.clone();

    let response = fetch_debug_logs(client, &current_request).await?;
    if request.follow {
        eprintln!("Connected; polling for debug logs...");
    }
    print_debug_log_response(
        &response,
        &mut printed_lines,
        request.include_platform_timestamp,
    );

    if !request.follow {
        return Ok(());
    }

    current_request = request.followup_request();

    loop {
        tokio::time::sleep(FOLLOW_POLL_INTERVAL).await;
        let response = fetch_debug_logs(client, &current_request).await?;
        print_debug_log_response(
            &response,
            &mut printed_lines,
            request.include_platform_timestamp,
        );
    }
}

async fn fetch_debug_logs(
    client: &turnkey_client::TurnkeyClient<TurnkeyP256ApiKey>,
    request: &DebugLogQueryRequest,
) -> anyhow::Result<GetEnclaveDebugLogsResponse> {
    client
        .get_enclave_debug_logs(request.to_api_request())
        .await
        .context("failed to fetch debug logs")
}

fn tail_lines_or_default(tail_lines: Option<i32>) -> anyhow::Result<i32> {
    let Some(tail_lines) = tail_lines else {
        return Ok(0);
    };

    if tail_lines < 0 {
        bail!("--tail-lines must be greater than or equal to 0");
    }

    Ok(tail_lines)
}

fn since_seconds_or_default(since_seconds: Option<i64>) -> anyhow::Result<i64> {
    let Some(since_seconds) = since_seconds else {
        return Ok(0);
    };

    if since_seconds < 0 {
        bail!("--since-seconds must be greater than or equal to 0");
    }

    Ok(since_seconds)
}

#[derive(Debug, Eq, Hash, PartialEq)]
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
            replica: replica.to_string(),
            content: line.content.clone(),
            seconds: ts.seconds.clone(),
            nanos: ts.nanos.clone(),
        })
    }
}

fn print_debug_log_response(
    response: &GetEnclaveDebugLogsResponse,
    printed_lines: &mut HashSet<LogLineKey>,
    include_platform_timestamp: bool,
) {
    for entry in &response.entries {
        print_debug_log_entry(entry, printed_lines, include_platform_timestamp);
    }
}

fn print_debug_log_entry(
    entry: &EnclaveDebugLogEntry,
    printed_lines: &mut HashSet<LogLineKey>,
    include_platform_timestamp: bool,
) {
    let Some(line) = entry.line.as_ref() else {
        return;
    };

    if let Some(key) = LogLineKey::new(&entry.replica, line) {
        if !printed_lines.insert(key) {
            return;
        }
    }

    println!(
        "{}",
        format_log_line(&entry.replica, line, include_platform_timestamp)
    );
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

    fn entry(replica: &str, line: Option<LogLine>) -> EnclaveDebugLogEntry {
        EnclaveDebugLogEntry {
            replica: replica.to_string(),
            line,
        }
    }

    #[test]
    fn tail_lines_defaults_to_zero_when_omitted() {
        assert_eq!(tail_lines_or_default(None).unwrap(), 0);
    }

    #[test]
    fn tail_lines_accepts_zero_and_positive_values() {
        assert_eq!(tail_lines_or_default(Some(0)).unwrap(), 0);
        assert_eq!(tail_lines_or_default(Some(25)).unwrap(), 25);
    }

    #[test]
    fn tail_lines_rejects_negative_values() {
        let err = tail_lines_or_default(Some(-1)).unwrap_err();
        assert!(err.to_string().contains("--tail-lines"));
    }

    #[test]
    fn since_seconds_defaults_to_zero_when_omitted() {
        assert_eq!(since_seconds_or_default(None).unwrap(), 0);
    }

    #[test]
    fn since_seconds_accepts_zero_and_positive_values() {
        assert_eq!(since_seconds_or_default(Some(0)).unwrap(), 0);
        assert_eq!(since_seconds_or_default(Some(25)).unwrap(), 25);
    }

    #[test]
    fn since_seconds_rejects_negative_values() {
        let err = since_seconds_or_default(Some(-1)).unwrap_err();
        assert!(err.to_string().contains("--since-seconds"));
    }

    #[test]
    fn request_maps_to_unary_api_request() {
        let request = DebugLogQueryRequest {
            organization_id: "org".to_string(),
            deployment_id: "deployment".to_string(),
            follow: true,
            tail_lines: 10,
            since_seconds: 30,
            include_platform_timestamp: true,
        };

        let api_request = request.to_api_request();

        assert_eq!(api_request.organization_id, "org");
        assert_eq!(api_request.deployment_id, "deployment");
        assert_eq!(api_request.tail_lines, 10);
        assert_eq!(api_request.since_seconds, 30);
    }

    #[test]
    fn followup_request_uses_overlapping_since_window() {
        let request = DebugLogQueryRequest {
            organization_id: "org".to_string(),
            deployment_id: "deployment".to_string(),
            follow: true,
            tail_lines: 100,
            since_seconds: 0,
            include_platform_timestamp: false,
        };

        let followup = request.followup_request();

        assert_eq!(followup.tail_lines, 0);
        assert_eq!(followup.since_seconds, FOLLOW_SINCE_SECONDS);
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
    fn print_response_skips_empty_entries_and_dedupes_timestamped_lines() {
        let response = GetEnclaveDebugLogsResponse {
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
        let mut printed = HashSet::new();

        print_debug_log_response(&response, &mut printed, false);

        assert_eq!(printed.len(), 1);
    }
}
