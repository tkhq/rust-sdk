//! Deploy debug-logs command.

use anyhow::{Context, bail};
use chrono::{DateTime, SecondsFormat, Utc};
use clap::Args as ClapArgs;
use futures_util::StreamExt;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use turnkey_client::generated::external::data::v1::{LogEventType, LogLine, Timestamp};
use turnkey_client::generated::{GetEnclaveDebugLogsRequest, GetEnclaveDebugLogsResponse};
use turnkey_client::{TurnkeyClientError, TurnkeyP256ApiKey};

const GATEWAY_DEADLINE_EXCEEDED_CODE: i32 = 4;
const GATEWAY_DEADLINE_EXCEEDED_MESSAGE: &str = "context deadline exceeded";
const GATEWAY_RST_STREAM_CANCEL_MARKERS: [&str; 2] = ["RST_STREAM", "CANCEL"];
const EXPECTED_STREAM_WINDOW: Duration = Duration::from_secs(10);

pub(crate) const LONG_ABOUT: &str = r#"
Stream debug logs for a deployment.

Debug logs are only available when debug mode has been enabled at both the app
and deployment levels. First, the app must opt in to allowing debug-mode
deployments with `--dangerous-enable-debug-mode-deployments`. That app-level
opt-in only permits debug deployments; it does not make every deployment
debuggable.

The specific deployment must also be created in debug mode with
`--dangerous-deploy-debug-mode`. Existing non-debug deployments cannot expose
debug logs retroactively. Create a new debug-mode deployment, then pass that
deployment ID to this command.

Use `--follow` to keep listening for new log lines after the initial log
buffer. See `tvc app create --help`, `tvc deploy create --help`, and
`tvc --help` for more info."#;

/// Stream debug logs for a deployment.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = LONG_ABOUT)]
pub struct Args {
    /// ID of the deployment.
    #[arg(short = 'd', long, env = "TVC_DEPLOY_ID")]
    pub deploy_id: String,

    /// Keep streaming new lines until the request timeout.
    #[arg(long, env = "TVC_DEBUG_LOGS_FOLLOW")]
    pub follow: bool,

    /// Limit initial history to the last N lines per replica. Omit for the full log buffer.
    #[arg(long, env = "TVC_DEBUG_LOGS_TAIL_LINES", allow_hyphen_values = true)]
    pub tail_lines: Option<i32>,
}

/// Run the `deploy debug-logs` command.
pub async fn run(args: Args) -> anyhow::Result<()> {
    let tail_lines = tail_lines_or_default(args.tail_lines)?;
    let auth = crate::client::build_client().await?;

    let request = DebugLogStreamRequest {
        organization_id: auth.org_id.clone(),
        deployment_id: args.deploy_id,
        follow: args.follow,
        tail_lines,
    };

    stream_debug_logs(&auth.client, &request).await
}

#[derive(Clone, Debug)]
struct DebugLogStreamRequest {
    organization_id: String,
    deployment_id: String,
    follow: bool,
    tail_lines: i32,
}

impl DebugLogStreamRequest {
    fn to_api_request(&self) -> GetEnclaveDebugLogsRequest {
        GetEnclaveDebugLogsRequest {
            organization_id: self.organization_id.clone(),
            deployment_id: self.deployment_id.clone(),
            follow: self.follow,
            tail_lines: self.tail_lines,
        }
    }
}

async fn stream_debug_logs(
    client: &turnkey_client::TurnkeyClient<TurnkeyP256ApiKey>,
    request: &DebugLogStreamRequest,
) -> anyhow::Result<()> {
    let retry_policy = StreamRetryPolicy::new(request.follow);
    let mut printed_lines = HashSet::new();
    let mut connected = false;

    loop {
        let outcome = drain_stream_window(client, request, &mut printed_lines, connected).await?;
        connected = true;

        if retry_policy.should_retry(&outcome) {
            continue;
        }

        if let Some(err) = outcome.error {
            return Err(err).context("debug log stream failed");
        }

        return Ok(());
    }
}

async fn drain_stream_window(
    client: &turnkey_client::TurnkeyClient<TurnkeyP256ApiKey>,
    request: &DebugLogStreamRequest,
    printed_lines: &mut HashSet<LogLineKey>,
    reconnected: bool,
) -> anyhow::Result<StreamWindowOutcome> {
    let started_at = Instant::now();
    let mut stream = client
        .get_enclave_debug_logs(request.to_api_request())
        .await
        .context("failed to start debug log stream")?;

    if reconnected {
        eprintln!("Reconnected; listening for debug logs...");
    } else {
        eprintln!("Connected; listening for debug logs...");
    }

    while let Some(response) = stream.next().await {
        match response {
            Ok(response) => print_debug_log_response(&response, printed_lines),
            Err(err) => {
                return Ok(StreamWindowOutcome {
                    elapsed: started_at.elapsed(),
                    error: Some(err),
                });
            }
        }
    }

    Ok(StreamWindowOutcome {
        elapsed: started_at.elapsed(),
        error: None,
    })
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

#[derive(Debug)]
struct StreamWindowOutcome {
    elapsed: Duration,
    error: Option<TurnkeyClientError>,
}

struct StreamRetryPolicy {
    follow: bool,
}

impl StreamRetryPolicy {
    fn new(follow: bool) -> Self {
        Self { follow }
    }

    fn should_retry(&self, outcome: &StreamWindowOutcome) -> bool {
        if !self.follow || outcome.elapsed < EXPECTED_STREAM_WINDOW {
            return false;
        }

        match outcome.error.as_ref() {
            Some(err) => is_expected_gateway_stream_end(err),
            None => true,
        }
    }
}

fn is_expected_gateway_stream_end(err: &TurnkeyClientError) -> bool {
    let TurnkeyClientError::StreamError { code, message } = err else {
        return false;
    };

    if *code != GATEWAY_DEADLINE_EXCEEDED_CODE {
        return false;
    }

    message == GATEWAY_DEADLINE_EXCEEDED_MESSAGE
        || GATEWAY_RST_STREAM_CANCEL_MARKERS
            .iter()
            .all(|marker| message.contains(marker))
}

#[derive(Debug, Eq, Hash, PartialEq)]
struct LogLineKey {
    pod_name: String,
    content: String,
    seconds: String,
    nanos: String,
}

impl LogLineKey {
    fn new(pod_name: &str, line: &LogLine) -> Option<Self> {
        let ts = line.ts.as_ref()?;

        Some(Self {
            pod_name: pod_name.to_string(),
            content: line.content.clone(),
            seconds: ts.seconds.clone(),
            nanos: ts.nanos.clone(),
        })
    }
}

fn print_debug_log_response(
    response: &GetEnclaveDebugLogsResponse,
    printed_lines: &mut HashSet<LogLineKey>,
) {
    if response.event == LogEventType::PodTerminated {
        eprintln!("{}", format_pod_terminated(&response.pod_name));
        return;
    }

    for line in &response.lines {
        if let Some(key) = LogLineKey::new(&response.pod_name, line) {
            if !printed_lines.insert(key) {
                continue;
            }
        }

        println!("{}", format_log_line(&response.pod_name, line));
    }
}

fn format_log_line(pod_name: &str, line: &LogLine) -> String {
    match line.ts.as_ref().and_then(format_timestamp) {
        Some(ts) => format!("{ts} {pod_name} {}", line.content),
        None => format!("{pod_name} {}", line.content),
    }
}

fn format_pod_terminated(pod_name: &str) -> String {
    format!("{pod_name} stream terminated")
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

    fn stream_error(code: i32, message: &str) -> TurnkeyClientError {
        TurnkeyClientError::StreamError {
            code,
            message: message.to_string(),
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
    fn retry_policy_retries_follow_deadline_after_expected_window() {
        let policy = StreamRetryPolicy::new(true);
        let outcome = StreamWindowOutcome {
            elapsed: EXPECTED_STREAM_WINDOW,
            error: Some(stream_error(4, GATEWAY_DEADLINE_EXCEEDED_MESSAGE)),
        };

        assert!(policy.should_retry(&outcome));
    }

    #[test]
    fn retry_policy_retries_follow_rst_stream_cancel_after_expected_window() {
        let policy = StreamRetryPolicy::new(true);
        let outcome = StreamWindowOutcome {
            elapsed: EXPECTED_STREAM_WINDOW,
            error: Some(stream_error(
                4,
                "stream terminated by RST_STREAM with error code: CANCEL",
            )),
        };

        assert!(policy.should_retry(&outcome));
    }

    #[test]
    fn retry_policy_retries_follow_clean_end_after_expected_window() {
        let policy = StreamRetryPolicy::new(true);
        let outcome = StreamWindowOutcome {
            elapsed: EXPECTED_STREAM_WINDOW,
            error: None,
        };

        assert!(policy.should_retry(&outcome));
    }

    #[test]
    fn retry_policy_does_not_retry_immediate_deadline_error() {
        let policy = StreamRetryPolicy::new(true);
        let outcome = StreamWindowOutcome {
            elapsed: Duration::from_secs(1),
            error: Some(stream_error(4, GATEWAY_DEADLINE_EXCEEDED_MESSAGE)),
        };

        assert!(!policy.should_retry(&outcome));
    }

    #[test]
    fn retry_policy_does_not_retry_immediate_rst_stream_cancel() {
        let policy = StreamRetryPolicy::new(true);
        let outcome = StreamWindowOutcome {
            elapsed: Duration::from_secs(1),
            error: Some(stream_error(
                4,
                "stream terminated by RST_STREAM with error code: CANCEL",
            )),
        };

        assert!(!policy.should_retry(&outcome));
    }

    #[test]
    fn retry_policy_does_not_retry_non_follow_streams() {
        let policy = StreamRetryPolicy::new(false);
        let outcome = StreamWindowOutcome {
            elapsed: EXPECTED_STREAM_WINDOW,
            error: Some(stream_error(4, GATEWAY_DEADLINE_EXCEEDED_MESSAGE)),
        };

        assert!(!policy.should_retry(&outcome));
    }

    #[test]
    fn retry_policy_does_not_retry_other_stream_errors() {
        let policy = StreamRetryPolicy::new(true);
        let outcome = StreamWindowOutcome {
            elapsed: EXPECTED_STREAM_WINDOW,
            error: Some(stream_error(7, "denied")),
        };

        assert!(!policy.should_retry(&outcome));
    }

    #[test]
    fn retry_policy_does_not_retry_unexpected_deadline_error() {
        let policy = StreamRetryPolicy::new(true);
        let outcome = StreamWindowOutcome {
            elapsed: EXPECTED_STREAM_WINDOW,
            error: Some(stream_error(4, "unexpected deadline exceeded")),
        };

        assert!(!policy.should_retry(&outcome));
    }

    #[test]
    fn log_line_key_matches_duplicate_lines() {
        let line = log_line("hello", Some(timestamp("1710000000", "123456789")));

        assert_eq!(
            LogLineKey::new("pod-a", &line),
            LogLineKey::new("pod-a", &line)
        );
    }

    #[test]
    fn log_line_key_requires_timestamp() {
        let line = log_line("hello", None);

        assert_eq!(LogLineKey::new("pod-a", &line), None);
    }

    #[test]
    fn format_log_line_includes_timestamp_when_present() {
        let line = log_line("hello", Some(timestamp("1710000000", "123456789")));

        assert_eq!(
            format_log_line("pod-a", &line),
            "2024-03-09T16:00:00.123456789Z pod-a hello"
        );
    }

    #[test]
    fn format_log_line_omits_timestamp_when_missing() {
        let line = log_line("hello", None);

        assert_eq!(format_log_line("pod-a", &line), "pod-a hello");
    }

    #[test]
    fn format_log_line_omits_invalid_timestamp() {
        let line = log_line("hello", Some(timestamp("bad", "123")));

        assert_eq!(format_log_line("pod-a", &line), "pod-a hello");
    }

    #[test]
    fn format_pod_terminated_mentions_pod() {
        assert_eq!(format_pod_terminated("pod-a"), "pod-a stream terminated");
    }
}
