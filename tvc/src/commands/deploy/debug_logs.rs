//! Deploy debug-logs command.

use anyhow::{Context, bail};
use chrono::{DateTime, SecondsFormat, Utc};
use clap::Args as ClapArgs;
use futures_util::StreamExt;
use turnkey_client::generated::external::data::v1::{LogEventType, LogLine, Timestamp};
use turnkey_client::generated::{GetEnclaveDebugLogsRequest, GetEnclaveDebugLogsResponse};

/// Stream debug logs for a deployment.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
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

    let request = GetEnclaveDebugLogsRequest {
        organization_id: auth.org_id.clone(),
        deployment_id: args.deploy_id,
        follow: args.follow,
        tail_lines,
    };

    let mut stream = auth
        .client
        .get_enclave_debug_logs(request)
        .await
        .context("failed to start debug log stream")?;

    while let Some(response) = stream.next().await {
        print_debug_log_response(&response.context("debug log stream failed")?);
    }

    Ok(())
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

fn print_debug_log_response(response: &GetEnclaveDebugLogsResponse) {
    if response.event == LogEventType::PodTerminated {
        eprintln!("{}", format_pod_terminated(&response.pod_name));
        return;
    }

    for line in &response.lines {
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
