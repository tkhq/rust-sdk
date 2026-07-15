//! Policy init command - provisions org policies for TVC resources.

use crate::client::build_client;
use crate::prompts;
use anyhow::{Context, Result, anyhow, bail};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::fmt;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::TurnkeyClientError;
use turnkey_client::generated::{
    CreatePoliciesIntent, CreatePolicyIntentV3, CreateUserTagIntent, immutable::common::v1::Effect,
};

const CREATED_TAG_ID_PLACEHOLDER: &str = "<created tag ID from create_user_tag>";

/// Provision TVC org policies for designated users.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = LONG_ABOUT)]
pub struct Args {
    /// Comma-separated TVC policy resources to allow.
    #[arg(
        long,
        value_name = "LIST",
        value_delimiter = ',',
        default_value = "TVC_APP,TVC_DEPLOYMENT,TVC_OPERATOR,TVC_QUORUM_KEY"
    )]
    resources: Vec<TvcResource>,

    /// Number of tagged/designated approvals required.
    #[arg(long, default_value_t = 1)]
    threshold: u32,

    /// Create a user tag with this name and attach --user-ids before creating the policy.
    #[arg(long, value_name = "NAME")]
    tag_name: Option<String>,

    /// Reuse an existing user tag UUID instead of creating one.
    #[arg(long, value_name = "UUID")]
    tag_id: Option<String>,

    /// Comma-separated user UUIDs for tag creation or tagless policy consensus.
    #[arg(long, value_name = "UUID,...", value_delimiter = ',')]
    user_ids: Vec<String>,

    /// Policy name. Defaults to tvc-operators-<resources>-threshold-<N>.
    #[arg(long, value_name = "NAME")]
    policy_name: Option<String>,

    /// Optional policy notes.
    #[arg(long)]
    notes: Option<String>,

    /// Print the generated policy and planned API calls without submitting anything.
    #[arg(long)]
    dry_run: bool,
}

pub(crate) const LONG_ABOUT: &str = r#"
Provision an organization policy that allows designated users to perform TVC
actions without root-quorum consensus.

The generated policy always includes both a TVC resource condition and a
consensus expression. This command never creates a policy that targets every org
user.

Modes:
  Tag mode with a new tag:
    tvc policy init --tag-name "TVC Operators" --user-ids <UUID,...>

  Tag mode with an existing tag:
    tvc policy init --tag-id <TAG_UUID>

  Tagless mode by user ID:
    tvc policy init --user-ids <UUID,...>

If your org root quorum is greater than one, creating the tag or policy may
return pending consensus. In that case, root-quorum members must approve the
printed activity ID before provisioning can continue."#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TvcResource {
    App,
    Deployment,
    Operator,
    QuorumKey,
}

impl TvcResource {
    pub fn all() -> Vec<Self> {
        vec![Self::App, Self::Deployment, Self::Operator, Self::QuorumKey]
    }

    fn as_policy_value(self) -> &'static str {
        match self {
            Self::App => "TVC_APP",
            Self::Deployment => "TVC_DEPLOYMENT",
            Self::Operator => "TVC_OPERATOR",
            Self::QuorumKey => "TVC_QUORUM_KEY",
        }
    }

    fn default_name_part(self) -> &'static str {
        match self {
            Self::App => "tvc-app",
            Self::Deployment => "tvc-deployment",
            Self::Operator => "tvc-operator",
            Self::QuorumKey => "tvc-quorum-key",
        }
    }
}

impl FromStr for TvcResource {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.trim() {
            "TVC_APP" => Ok(Self::App),
            "TVC_DEPLOYMENT" => Ok(Self::Deployment),
            "TVC_OPERATOR" => Ok(Self::Operator),
            "TVC_QUORUM_KEY" => Ok(Self::QuorumKey),
            other => Err(format!("invalid TVC policy resource: {other}")),
        }
    }
}

impl fmt::Display for TvcResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_policy_value())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicySubject {
    Tag { tag_id: String },
    UserIds(Vec<String>),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GeneratedPolicy {
    pub policy_name: String,
    pub effect: &'static str,
    pub condition: String,
    pub consensus: String,
    pub notes: String,
}

enum Mode {
    NewTag {
        tag_name: String,
        user_ids: Vec<String>,
    },
    ExistingTag {
        tag_id: String,
    },
    UserIds(Vec<String>),
}

pub async fn run(args: Args, is_non_interactive: bool) -> Result<()> {
    let mode = resolve_mode(&args)?;
    let subject = match &mode {
        Mode::NewTag { .. } => PolicySubject::Tag {
            tag_id: CREATED_TAG_ID_PLACEHOLDER.to_string(),
        },
        Mode::ExistingTag { tag_id } => PolicySubject::Tag {
            tag_id: tag_id.clone(),
        },
        Mode::UserIds(user_ids) => PolicySubject::UserIds(user_ids.clone()),
    };
    let preview = build_policy(
        args.policy_name.clone(),
        &args.resources,
        args.threshold,
        subject,
        args.notes.clone(),
    )?;

    print_plan(&preview, &mode, args.dry_run)?;
    if args.dry_run {
        return Ok(());
    }

    if !is_non_interactive && prompts::stdin_can_prompt() {
        prompts::confirm_or_bail("Create this TVC policy?", "policy provisioning")?;
    }

    let auth = build_client().await?;

    let tag_id = match mode {
        Mode::NewTag { tag_name, user_ids } => {
            println!("Creating user tag '{tag_name}'...");
            let result = auth
                .client
                .create_user_tag(
                    auth.org_id.clone(),
                    timestamp_ms()?,
                    CreateUserTagIntent {
                        user_tag_name: tag_name,
                        user_ids,
                    },
                )
                .await;
            match result {
                Ok(result) => {
                    println!("Created user tag: {}", result.result.user_tag_id);
                    result.result.user_tag_id
                }
                Err(TurnkeyClientError::ActivityRequiresApproval(activity_id)) => {
                    exit_pending_consensus("create_user_tag", &activity_id);
                }
                Err(err) => return Err(anyhow!(err).context("failed to create user tag")),
            }
        }
        Mode::ExistingTag { tag_id } => tag_id,
        Mode::UserIds(_) => CREATED_TAG_ID_PLACEHOLDER.to_string(),
    };

    let subject = match &tag_id[..] {
        CREATED_TAG_ID_PLACEHOLDER => match resolve_mode(&args)? {
            Mode::UserIds(user_ids) => PolicySubject::UserIds(user_ids),
            _ => unreachable!("placeholder tag ID is only used for dry-run and tagless mode"),
        },
        _ => PolicySubject::Tag { tag_id },
    };
    let policy = build_policy(
        args.policy_name,
        &args.resources,
        args.threshold,
        subject,
        args.notes,
    )?;

    println!("Creating TVC policy '{}'...", policy.policy_name);
    let result = auth
        .client
        .create_policies(
            auth.org_id,
            timestamp_ms()?,
            CreatePoliciesIntent {
                policies: vec![policy.clone().into_create_policy_intent()],
            },
        )
        .await;

    match result {
        Ok(result) => {
            println!("TVC policy created successfully.");
            println!("Policy IDs: {}", result.result.policy_ids.join(", "));
            Ok(())
        }
        Err(TurnkeyClientError::ActivityRequiresApproval(activity_id)) => {
            exit_pending_consensus("create_policies", &activity_id);
        }
        Err(err) => Err(anyhow!(err).context("failed to create policies")),
    }
}

pub fn build_policy(
    policy_name: Option<String>,
    resources: &[TvcResource],
    threshold: u32,
    subject: PolicySubject,
    notes: Option<String>,
) -> Result<GeneratedPolicy> {
    if resources.is_empty() {
        bail!("at least one TVC policy resource is required");
    }
    if threshold == 0 {
        bail!("--threshold must be at least 1");
    }

    let consensus = build_consensus(threshold, subject)?;
    if consensus.trim().is_empty() {
        bail!("internal error: generated policy consensus cannot be empty");
    }

    Ok(GeneratedPolicy {
        policy_name: policy_name.unwrap_or_else(|| default_policy_name(resources, threshold)),
        effect: "EFFECT_ALLOW",
        condition: build_condition(resources),
        consensus,
        notes: notes.unwrap_or_default(),
    })
}

impl GeneratedPolicy {
    fn into_create_policy_intent(self) -> CreatePolicyIntentV3 {
        CreatePolicyIntentV3 {
            policy_name: self.policy_name,
            effect: Effect::Allow,
            condition: Some(self.condition),
            consensus: Some(self.consensus),
            notes: self.notes,
        }
    }
}

fn resolve_mode(args: &Args) -> Result<Mode> {
    let user_ids = normalize_user_ids(&args.user_ids)?;
    let has_user_ids = !user_ids.is_empty();
    let has_tag_name = args.tag_name.as_ref().is_some_and(|v| !v.trim().is_empty());
    let has_tag_id = args.tag_id.as_ref().is_some_and(|v| !v.trim().is_empty());

    match (has_tag_id, has_tag_name, has_user_ids) {
        (true, false, false) => Ok(Mode::ExistingTag {
            tag_id: args.tag_id.clone().unwrap(),
        }),
        (false, true, true) => Ok(Mode::NewTag {
            tag_name: args.tag_name.clone().unwrap(),
            user_ids,
        }),
        (false, false, true) => Ok(Mode::UserIds(user_ids)),
        _ => bail!(
            "provide --tag-id, or provide --user-ids with --tag-name for tag mode, or --user-ids alone for tagless mode"
        ),
    }
}

fn normalize_user_ids(user_ids: &[String]) -> Result<Vec<String>> {
    let mut normalized = Vec::new();
    for user_id in user_ids {
        let trimmed = user_id.trim();
        if trimmed.is_empty() {
            bail!("--user-ids cannot contain empty values");
        }
        normalized.push(trimmed.to_string());
    }
    Ok(normalized)
}

fn build_condition(resources: &[TvcResource]) -> String {
    if resources.len() == 1 {
        return format!("activity.resource == '{}'", resources[0].as_policy_value());
    }

    format!(
        "activity.resource in [{}]",
        resources
            .iter()
            .map(|resource| format!("'{}'", resource.as_policy_value()))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn build_consensus(threshold: u32, subject: PolicySubject) -> Result<String> {
    match subject {
        PolicySubject::Tag { tag_id } => {
            if tag_id.trim().is_empty() {
                bail!("tag ID cannot be empty");
            }
            if threshold == 1 {
                Ok(format!(
                    "approvers.any(user, user.tags.contains('{tag_id}'))"
                ))
            } else {
                Ok(format!(
                    "approvers.filter(user, user.tags.contains('{tag_id}')).count() >= {threshold}"
                ))
            }
        }
        PolicySubject::UserIds(user_ids) => {
            let user_ids = normalize_user_ids(&user_ids)?;
            if user_ids.is_empty() {
                bail!("at least one --user-ids value is required for tagless mode");
            }
            let list = format_string_list(&user_ids);
            if threshold == 1 {
                if user_ids.len() == 1 {
                    Ok(format!("approvers.any(user, user.id == '{}')", user_ids[0]))
                } else {
                    Ok(format!("approvers.any(user, user.id in {list})"))
                }
            } else {
                Ok(format!(
                    "approvers.filter(user, user.id in {list}).count() >= {threshold}"
                ))
            }
        }
    }
}

fn format_string_list(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| format!("'{value}'"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn default_policy_name(resources: &[TvcResource], threshold: u32) -> String {
    let resource_part = if resources == TvcResource::all().as_slice() {
        "all-resources".to_string()
    } else {
        resources
            .iter()
            .map(|resource| resource.default_name_part())
            .collect::<Vec<_>>()
            .join("-")
    };

    format!("tvc-operators-{resource_part}-threshold-{threshold}")
}

fn print_plan(policy: &GeneratedPolicy, mode: &Mode, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Dry run: no API calls will be submitted.");
    }
    println!("Generated TVC policy:");
    println!("{}", serde_json::to_string_pretty(policy)?);
    println!();
    println!("Planned API calls:");
    match mode {
        Mode::NewTag { tag_name, user_ids } => {
            println!(
                "1. create_user_tag: name='{tag_name}', user_ids=[{}]",
                user_ids.join(", ")
            );
            println!("2. create_policies: 1 EFFECT_ALLOW policy for TVC resources");
        }
        Mode::ExistingTag { tag_id } => {
            println!("1. create_policies: 1 EFFECT_ALLOW policy using tag '{tag_id}'");
        }
        Mode::UserIds(user_ids) => {
            println!(
                "1. create_policies: 1 EFFECT_ALLOW policy using user_ids=[{}]",
                user_ids.join(", ")
            );
        }
    }
    println!();
    Ok(())
}

fn timestamp_ms() -> Result<u128> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before unix epoch")?
        .as_millis())
}

fn exit_pending_consensus(activity_name: &str, activity_id: &str) -> ! {
    eprintln!("Provisioning activity is pending root-quorum consensus.");
    eprintln!("Activity: {activity_name}");
    eprintln!("Activity ID: {activity_id}");
    eprintln!(
        "Ask the required root-quorum members to approve this activity, then rerun `tvc policy init` if the remaining provisioning step has not yet run."
    );
    std::process::exit(2);
}
