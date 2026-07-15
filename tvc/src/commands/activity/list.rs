//! Activity list command - list activities for the org, pending consensus by default.

use crate::client::build_client;
use anyhow::{Context, Result, anyhow};
use clap::{Args as ClapArgs, ValueEnum};
use turnkey_client::generated::external::data::v1::Timestamp;
use turnkey_client::generated::{ActivityStatus, ActivityType, GetActivitiesRequest};

/// List activities for the organization. Defaults to activities pending consensus.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Filter by activity status (repeatable). Defaults to consensus-needed.
    #[arg(long = "status", value_enum, value_name = "STATUS")]
    pub statuses: Vec<StatusFilter>,

    /// Filter by activity type, e.g. ACTIVITY_TYPE_CREATE_TVC_DEPLOYMENT (repeatable).
    #[arg(long = "activity-type", value_name = "TYPE")]
    pub activity_types: Vec<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum StatusFilter {
    Created,
    Pending,
    ConsensusNeeded,
    Completed,
    Failed,
    Rejected,
}

impl From<StatusFilter> for ActivityStatus {
    fn from(filter: StatusFilter) -> Self {
        match filter {
            StatusFilter::Created => ActivityStatus::Created,
            StatusFilter::Pending => ActivityStatus::Pending,
            StatusFilter::ConsensusNeeded => ActivityStatus::ConsensusNeeded,
            StatusFilter::Completed => ActivityStatus::Completed,
            StatusFilter::Failed => ActivityStatus::Failed,
            StatusFilter::Rejected => ActivityStatus::Rejected,
        }
    }
}

fn parse_activity_types(raw: &[String]) -> Result<Vec<ActivityType>> {
    raw.iter()
        .map(|name| {
            ActivityType::from_str_name(name).ok_or_else(|| {
                anyhow!(
                    "unknown activity type: {name} (expected an ACTIVITY_TYPE_* name, \
                     e.g. ACTIVITY_TYPE_CREATE_TVC_DEPLOYMENT)"
                )
            })
        })
        .collect()
}

fn format_created_at(created_at: Option<&Timestamp>) -> String {
    created_at
        .and_then(|ts| ts.seconds.parse::<i64>().ok())
        .and_then(|seconds| chrono::DateTime::from_timestamp(seconds, 0))
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| "-".to_string())
}

pub async fn run(args: Args) -> Result<()> {
    let filter_by_type = parse_activity_types(&args.activity_types)?;

    let filter_by_status: Vec<ActivityStatus> = if args.statuses.is_empty() {
        vec![ActivityStatus::ConsensusNeeded]
    } else {
        args.statuses.into_iter().map(Into::into).collect()
    };

    let auth = build_client().await?;

    let response = auth
        .client
        .get_activities(GetActivitiesRequest {
            organization_id: auth.org_id.clone(),
            filter_by_status,
            pagination_options: None,
            filter_by_type,
        })
        .await
        .context("failed to list activities")?;

    if response.activities.is_empty() {
        println!("No matching activities found.");
        return Ok(());
    }

    for activity in &response.activities {
        println!("Activity ID: {}", activity.id);
        println!("  Type: {}", activity.r#type.as_str_name());
        println!("  Status: {}", activity.status.as_str_name());
        println!("  Fingerprint: {}", activity.fingerprint);
        println!(
            "  Created: {}",
            format_created_at(activity.created_at.as_ref())
        );
        println!();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_activity_types_accepts_known_names() {
        let parsed = parse_activity_types(&[
            "ACTIVITY_TYPE_CREATE_TVC_DEPLOYMENT".to_string(),
            "ACTIVITY_TYPE_CREATE_TVC_MANIFEST_APPROVALS".to_string(),
        ])
        .unwrap();
        assert_eq!(
            parsed,
            vec![
                ActivityType::CreateTvcDeployment,
                ActivityType::CreateTvcManifestApprovals
            ]
        );
    }

    #[test]
    fn parse_activity_types_rejects_unknown_names() {
        let error = parse_activity_types(&["NOT_A_REAL_ACTIVITY_TYPE".to_string()]).unwrap_err();
        assert!(error.to_string().contains("NOT_A_REAL_ACTIVITY_TYPE"));
    }

    #[test]
    fn format_created_at_renders_rfc3339() {
        let formatted = format_created_at(Some(&Timestamp {
            seconds: "1752570000".to_string(),
            nanos: "0".to_string(),
        }));
        assert!(formatted.starts_with("2025-07-15"), "{formatted}");
    }

    #[test]
    fn format_created_at_falls_back_for_missing_timestamp() {
        assert_eq!(format_created_at(None), "-");
    }
}
