//! Signed requests and resumable activity operations for shared CLI identities.
use std::fmt::{self, Display, Formatter};
use std::io::Read;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use clap::{Args, Subcommand};
use reqwest::{Client, Url, redirect::Policy};
use serde::Serialize;
use serde_json::{Value, json};
use turnkey_api_key_stamper::{Stamp, TurnkeyP256ApiKey};
use turnkey_client::generated::{
    GetActivitiesRequest, GetActivityRequest,
    external::{
        activity::v1::{ApproveActivityRequest, RejectActivityRequest},
        options::v1::Pagination,
    },
    immutable::activity::v1::{ApproveActivityIntent, RejectActivityIntent},
};

#[derive(Debug, Args)]
pub struct RequestArgs {
    #[arg(long, value_parser = request_path)]
    path: String,
    #[arg(
        long,
        required_unless_present = "body_file",
        conflicts_with = "body_file"
    )]
    body: Option<String>,
    /// Read exact UTF-8 request bytes from a file, or - for stdin.
    #[arg(long, required_unless_present = "body", conflicts_with = "body")]
    body_file: Option<PathBuf>,
    /// Produce a stamp without submitting the request.
    #[arg(long)]
    stamp_only: bool,
}

#[derive(Debug, Subcommand)]
pub enum ActivityCommand {
    List {
        #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u32).range(1..))]
        limit: u32,
        /// API after cursor (activity ID); pagination is explicitly caller-driven.
        #[arg(long)]
        cursor: Option<String>,
    },
    Get {
        id: String,
    },
    Approve {
        id: String,
    },
    Reject {
        id: String,
    },
    Wait {
        id: String,
        #[arg(long, default_value_t = 60, value_parser = clap::value_parser!(u64).range(1..))]
        timeout: u64,
    },
}

/// A terminal command record. A nonzero exit reflects operation failure, not an
/// inspected resource's state; successfully inspecting a rejected activity works.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationOutput {
    schema_version: u8,
    reason: &'static str,
    command: &'static str,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Box<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    activity: Option<Box<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    http_status: Option<u16>,
}

impl OperationOutput {
    pub fn failed(&self) -> bool {
        self.code.is_some()
    }
    pub fn result(command: &'static str, data: Value) -> Self {
        let activity = data
            .get("activity")
            .filter(|v| v.is_object())
            .map(|v| json!({"id":v.get("id"),"status":v.get("status")}));
        let status = activity
            .as_ref()
            .map(activity_status)
            .unwrap_or("completed");
        Self {
            schema_version: 1,
            reason: "command_result",
            command,
            status,
            data: Some(Box::new(data)),
            activity: activity.map(Box::new),
            code: None,
            message: None,
            http_status: None,
        }
    }
    pub fn error(command: &'static str, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            schema_version: 1,
            reason: "command_error",
            command,
            status: if code == "submission_unknown" {
                "unknown"
            } else {
                "failed"
            },
            data: None,
            activity: None,
            code: Some(code),
            message: Some(message.into()),
            http_status: None,
        }
    }
    fn fail(mut self, code: &'static str, message: &str) -> Self {
        self.reason = "command_error";
        self.code = Some(code);
        self.message = Some(message.to_owned());
        self
    }
}
impl Display for OperationOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self).map_err(|_| fmt::Error)?
        )
    }
}

fn activity_status(activity: &Value) -> &'static str {
    match activity.get("status").and_then(Value::as_str) {
        Some("ACTIVITY_STATUS_COMPLETED") => "completed",
        Some("ACTIVITY_STATUS_REJECTED") => "rejected",
        Some("ACTIVITY_STATUS_FAILED") => "failed",
        Some(
            "ACTIVITY_STATUS_CREATED"
            | "ACTIVITY_STATUS_PENDING"
            | "ACTIVITY_STATUS_CONSENSUS_NEEDED"
            | "ACTIVITY_STATUS_AUTHENTICATORS_NEEDED",
        ) => "pending",
        _ => "unknown",
    }
}
fn request_path(value: &str) -> Result<String, String> {
    if !value.starts_with("/public/v1/")
        || value.contains(['?', '#', '\\', '%'])
        || value.split('/').any(|s| s == ".." || s == ".")
    {
        return Err(
            "path must be an absolute /public/v1/ API path without query, fragment, or traversal"
                .into(),
        );
    }
    Ok(value.into())
}
fn url(base: &str, path: &str) -> Result<Url, &'static str> {
    let base = Url::parse(base).map_err(|_| "invalid API base URL")?;
    if !matches!(base.scheme(), "https" | "http")
        || !base.username().is_empty()
        || base.password().is_some()
        || base.query().is_some()
        || base.fragment().is_some()
    {
        return Err("API base URL must be HTTP(S) without credentials, query, or fragment");
    }
    base.join(path).map_err(|_| "invalid request URL")
}
fn client() -> Result<Client, reqwest::Error> {
    Client::builder()
        .redirect(Policy::none())
        .timeout(Duration::from_secs(30))
        .build()
}
async fn post(
    command: &'static str,
    http: &Client,
    endpoint: Url,
    body: String,
    stamper: &TurnkeyP256ApiKey,
    mutation: bool,
) -> Result<Value, OperationOutput> {
    let stamp = stamper
        .stamp(body.as_bytes())
        .map_err(|_| OperationOutput::error(command, "invalid_input", "could not stamp request"))?;
    let response = http
        .post(endpoint)
        .header(stamp.name, stamp.value)
        .header("Content-Type", "application/json")
        .header("X-TVC-CLIENT-VERSION", env!("CARGO_PKG_VERSION"))
        .body(body)
        .send()
        .await
        .map_err(|_| {
            OperationOutput::error(
                command,
                if mutation {
                    "submission_unknown"
                } else {
                    "network_error"
                },
                if mutation {
                    "request outcome is unknown; inspect activities before resubmitting"
                } else {
                    "API request failed"
                },
            )
        })?;
    let status = response.status();
    if !status.is_success() {
        let code = match status.as_u16() {
            401 | 403 => "unauthorized",
            404 => "not_found",
            _ => "api_error",
        };
        let mut output = OperationOutput::error(
            command,
            code,
            format!("API returned HTTP {}", status.as_u16()),
        );
        output.http_status = Some(status.as_u16());
        return Err(output);
    }
    response.json().await.map_err(|_| {
        OperationOutput::error(
            command,
            if mutation {
                "submission_unknown"
            } else {
                "api_error"
            },
            "API returned an unreadable response",
        )
    })
}
fn encode<T: Serialize>(command: &'static str, value: &T) -> Result<String, OperationOutput> {
    serde_json::to_string(value)
        .map_err(|_| OperationOutput::error(command, "invalid_input", "could not encode request"))
}
fn validate_body(body: &str, org: &str) -> Result<(), &'static str> {
    let value: Value = serde_json::from_str(body).map_err(|_| "body must be valid JSON")?;
    if !value.is_object() {
        return Err("body must be a JSON object");
    }
    if value.get("organizationId").and_then(Value::as_str) != Some(org) {
        return Err("body organizationId must match selected organization");
    }
    Ok(())
}

/// A raw request whose local body has been read and parsed before authentication.
pub struct PreparedRequest {
    path: String,
    body: String,
    stamp_only: bool,
}

impl RequestArgs {
    pub fn prepare(self) -> Result<PreparedRequest, OperationOutput> {
        let command = "request";
        let body = match (self.body, self.body_file) {
            (Some(body), None) => Ok(body),
            (None, Some(path)) if path.as_os_str() == "-" => {
                let mut body = String::new();
                std::io::stdin().read_to_string(&mut body).map(|_| body)
            }
            (None, Some(path)) => std::fs::read_to_string(path),
            _ => {
                return Err(OperationOutput::error(
                    command,
                    "invalid_input",
                    "provide exactly one request body source",
                ));
            }
        };
        let body = match body {
            Ok(body) => body,
            Err(_) => {
                return Err(OperationOutput::error(
                    command,
                    "invalid_input",
                    "could not read UTF-8 request body",
                ));
            }
        };
        let value: Value = serde_json::from_str(&body).map_err(|_| {
            OperationOutput::error("request", "invalid_input", "body must be valid JSON")
        })?;
        if !value.is_object() {
            return Err(OperationOutput::error(
                "request",
                "invalid_input",
                "body must be a JSON object",
            ));
        }
        Ok(PreparedRequest {
            path: self.path,
            body,
            stamp_only: self.stamp_only,
        })
    }
}

pub async fn run_request(
    args: RequestArgs,
    org_id: &str,
    api_base_url: &str,
    stamper: &TurnkeyP256ApiKey,
) -> OperationOutput {
    match args.prepare() {
        Ok(prepared) => prepared.run(org_id, api_base_url, stamper).await,
        Err(error) => error,
    }
}

impl PreparedRequest {
    pub async fn run(
        self,
        org_id: &str,
        api_base_url: &str,
        stamper: &TurnkeyP256ApiKey,
    ) -> OperationOutput {
        let command = "request";
        let body = self.body;
        if let Err(message) = validate_body(&body, org_id) {
            return OperationOutput::error(command, "invalid_input", message);
        }
        let endpoint = match url(api_base_url, &self.path) {
            Ok(url) => url,
            Err(message) => return OperationOutput::error(command, "invalid_input", message),
        };
        if self.stamp_only {
            return match stamper.stamp(body.as_bytes()) {
                Ok(stamp) => OperationOutput::result(
                    command,
                    json!({"url":endpoint.as_str(),"method":"POST","header":{"name":stamp.name,"value":stamp.value},"body":body}),
                ),
                Err(_) => {
                    OperationOutput::error(command, "invalid_input", "could not stamp request")
                }
            };
        }
        let http = match client() {
            Ok(http) => http,
            Err(_) => {
                return OperationOutput::error(
                    command,
                    "command_error",
                    "could not initialize HTTP client",
                );
            }
        };
        match post(
            command,
            &http,
            endpoint,
            body,
            stamper,
            !self.path.starts_with("/public/v1/query/"),
        )
        .await
        {
            Ok(value) if self.path.starts_with("/public/v1/query/") => {
                OperationOutput::result(command, value)
            }
            Ok(value) => submission_result(command, value),
            Err(error) => error,
        }
    }
}

fn submission_result(command: &'static str, data: Value) -> OperationOutput {
    if data
        .pointer("/activity/id")
        .and_then(Value::as_str)
        .is_none_or(str::is_empty)
    {
        return OperationOutput::error(
            command,
            "submission_unknown",
            "response omitted activity identity; inspect activities before resubmitting",
        );
    }
    terminal_submission(OperationOutput::result(command, data))
}

fn terminal_submission(output: OperationOutput) -> OperationOutput {
    match output.status {
        "rejected" => output.fail("api_error", "activity rejected"),
        "failed" => output.fail("api_error", "activity failed"),
        "unknown" => output.fail(
            "submission_unknown",
            "unknown activity status; inspect activity before resubmitting",
        ),
        _ => output,
    }
}

async fn query_activity(
    command: &'static str,
    http: &Client,
    base: &str,
    org: &str,
    id: &str,
    stamper: &TurnkeyP256ApiKey,
) -> Result<Value, OperationOutput> {
    let body = encode(
        command,
        &GetActivityRequest {
            organization_id: org.into(),
            activity_id: id.into(),
        },
    )?;
    let endpoint = url(base, "/public/v1/query/get_activity")
        .map_err(|message| OperationOutput::error(command, "invalid_input", message))?;
    let value = post(command, http, endpoint, body, stamper, false).await?;
    if value.pointer("/activity/id").and_then(Value::as_str) != Some(id) {
        return Err(OperationOutput::error(
            command,
            "api_error",
            "response omitted or mismatched requested activity",
        ));
    }
    Ok(value)
}

pub async fn run_activity(
    args: ActivityCommand,
    org_id: &str,
    api_base_url: &str,
    stamper: &TurnkeyP256ApiKey,
) -> OperationOutput {
    let command = match &args {
        ActivityCommand::List { .. } => "activity.list",
        ActivityCommand::Get { .. } => "activity.get",
        ActivityCommand::Approve { .. } => "activity.approve",
        ActivityCommand::Reject { .. } => "activity.reject",
        ActivityCommand::Wait { .. } => "activity.wait",
    };
    let http = match client() {
        Ok(http) => http,
        Err(_) => {
            return OperationOutput::error(
                command,
                "command_error",
                "could not initialize HTTP client",
            );
        }
    };
    match activity_inner(command, args, org_id, api_base_url, stamper, &http).await {
        Ok(output) | Err(output) => output,
    }
}
async fn activity_inner(
    command: &'static str,
    args: ActivityCommand,
    org: &str,
    base: &str,
    stamper: &TurnkeyP256ApiKey,
    http: &Client,
) -> Result<OperationOutput, OperationOutput> {
    if let ActivityCommand::List { limit, cursor } = args {
        let request = GetActivitiesRequest {
            organization_id: org.into(),
            filter_by_status: vec![],
            filter_by_type: vec![],
            pagination_options: Some(Pagination {
                limit: limit.to_string(),
                before: String::new(),
                after: cursor.unwrap_or_default(),
            }),
        };
        let endpoint = url(base, "/public/v1/query/list_activities")
            .map_err(|message| OperationOutput::error(command, "invalid_input", message))?;
        let response = post(
            command,
            http,
            endpoint,
            encode(command, &request)?,
            stamper,
            false,
        )
        .await?;
        let items = response
            .get("activities")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                OperationOutput::error(command, "api_error", "response omitted activities")
            })?;
        // The API exposes no hasNextPage. Return a continuation candidate rather
        // than claim a full page proves more results exist.
        let next = if items.len() == limit as usize {
            items.last().and_then(|item| item.get("id")).cloned()
        } else {
            None
        };
        return Ok(OperationOutput::result(
            command,
            json!({"items":items,"nextCursor":next}),
        ));
    }
    let (id, timeout, vote) = match args {
        ActivityCommand::Get { id } => (id, None, None),
        ActivityCommand::Wait { id, timeout } => (id, Some(timeout), None),
        ActivityCommand::Approve { id } => (id, None, Some(true)),
        ActivityCommand::Reject { id } => (id, None, Some(false)),
        ActivityCommand::List { .. } => {
            return Err(OperationOutput::error(
                command,
                "invalid_input",
                "invalid activity operation",
            ));
        }
    };
    if let Some(seconds) = timeout {
        let mut last = None;
        let result = tokio::time::timeout(Duration::from_secs(seconds), async {
            loop {
                let value = query_activity(command, http, base, org, &id, stamper).await?;
                let output = OperationOutput::result(command, value);
                if output.status != "pending" {
                    return Ok::<_, OperationOutput>(terminal_submission(output));
                }
                last = output.activity;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        })
        .await;
        return match result {
            Ok(Ok(output)) => Ok(output),
            Ok(Err(mut output)) => {
                output.activity =
                    Some(last.unwrap_or_else(|| Box::new(json!({"id": id, "status": null}))));
                Err(output)
            }
            Err(_) => {
                let mut output = OperationOutput::error(
                    command,
                    "wait_timeout",
                    "wait timed out; resume with activity wait and the same ID",
                );
                output.status = "pending";
                output.activity =
                    Some(last.unwrap_or_else(|| Box::new(json!({"id":id,"status":null}))));
                Ok(output)
            }
        };
    }
    let value = query_activity(command, http, base, org, &id, stamper).await?;
    let Some(approve) = vote else {
        return Ok(OperationOutput::result(command, value));
    };
    let target = Box::new(json!({"id": id, "status": value.pointer("/activity/status")}));
    let submitted = async {
        let fingerprint = value
            .pointer("/activity/fingerprint")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                OperationOutput::error(command, "api_error", "activity omitted fingerprint")
            })?
            .to_owned();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| {
                OperationOutput::error(command, "command_error", "system clock precedes Unix epoch")
            })?
            .as_millis()
            .to_string();
        let (path, body) = if approve {
            (
                "/public/v1/submit/approve_activity",
                encode(
                    command,
                    &ApproveActivityRequest {
                        r#type: "ACTIVITY_TYPE_APPROVE_ACTIVITY".into(),
                        timestamp_ms: timestamp,
                        organization_id: org.into(),
                        parameters: Some(ApproveActivityIntent { fingerprint }),
                        generate_app_proofs: None,
                    },
                )?,
            )
        } else {
            (
                "/public/v1/submit/reject_activity",
                encode(
                    command,
                    &RejectActivityRequest {
                        r#type: "ACTIVITY_TYPE_REJECT_ACTIVITY".into(),
                        timestamp_ms: timestamp,
                        organization_id: org.into(),
                        parameters: Some(RejectActivityIntent { fingerprint }),
                        generate_app_proofs: None,
                    },
                )?,
            )
        };
        let endpoint = url(base, path)
            .map_err(|message| OperationOutput::error(command, "invalid_input", message))?;
        let response = post(command, http, endpoint, body, stamper, true).await?;
        if response
            .pointer("/activity/id")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            return Err(OperationOutput::error(
                command,
                "submission_unknown",
                "response omitted activity; inspect activity before resubmitting",
            ));
        }
        let output = OperationOutput::result(command, response);
        // A rejected target is the successful outcome of an explicit rejection.
        Ok::<_, OperationOutput>(if !approve && output.status == "rejected" {
            output
        } else {
            terminal_submission(output)
        })
    }
    .await;
    match submitted {
        Ok(mut output) => {
            if output.failed() {
                output.activity = Some(target);
            }
            Ok(output)
        }
        Err(mut output) => {
            // This is the last observed target state, not proof of the vote's
            // outcome. Preserve submission_unknown and give callers an ID to inspect.
            output.activity = Some(target);
            Err(output)
        }
    }
}

/// Convert a decoded SDK activity without requiring its result to be complete.
pub fn from_activity(
    command: &'static str,
    activity: turnkey_client::generated::external::activity::v1::Activity,
) -> OperationOutput {
    match serde_json::to_value(activity) {
        Ok(value) => submission_result(command, json!({"activity": value})),
        Err(_) => {
            OperationOutput::error(command, "command_error", "could not encode activity output")
        }
    }
}

/// Submit one typed activity envelope without implicit retries or result unwrapping.
pub async fn submit<T: Serialize>(
    command: &'static str,
    path: &str,
    request: &T,
    api_base_url: &str,
    stamper: &TurnkeyP256ApiKey,
) -> OperationOutput {
    let body = match encode(command, request) {
        Ok(body) => body,
        Err(error) => return error,
    };
    let endpoint = match url(api_base_url, path) {
        Ok(endpoint) => endpoint,
        Err(message) => return OperationOutput::error(command, "invalid_input", message),
    };
    let http = match client() {
        Ok(http) => http,
        Err(_) => {
            return OperationOutput::error(
                command,
                "command_error",
                "could not initialize HTTP client",
            );
        }
    };
    match post(command, &http, endpoint, body, stamper, true).await {
        Ok(value) => submission_result(command, value),
        Err(error) => error,
    }
}

/// Query with the same bounded transport and without activity completion checks.
pub async fn query<T: Serialize>(
    command: &'static str,
    path: &str,
    request: &T,
    api_base_url: &str,
    stamper: &TurnkeyP256ApiKey,
) -> OperationOutput {
    let body = match encode(command, request) {
        Ok(body) => body,
        Err(error) => return error,
    };
    let endpoint = match url(api_base_url, path) {
        Ok(endpoint) => endpoint,
        Err(message) => return OperationOutput::error(command, "invalid_input", message),
    };
    let http = match client() {
        Ok(http) => http,
        Err(_) => {
            return OperationOutput::error(
                command,
                "command_error",
                "could not initialize HTTP client",
            );
        }
    };
    match post(command, &http, endpoint, body, stamper, false).await {
        Ok(value) => OperationOutput::result(command, value),
        Err(error) => error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_string, header_exists, method, path},
    };
    #[derive(Parser)]
    struct RequestCli {
        #[command(flatten)]
        request: RequestArgs,
    }
    #[derive(Parser)]
    struct ActivityCli {
        #[command(subcommand)]
        activity: ActivityCommand,
    }
    fn activity(id: &str, status: &str) -> Value {
        json!({"activity":{"id":id,"status":status,"fingerprint":"sha256:example"}})
    }
    #[test]
    fn decoded_sdk_activities_require_identity_for_completed_and_pending_results() {
        for status in [
            "ACTIVITY_STATUS_COMPLETED",
            "ACTIVITY_STATUS_CONSENSUS_NEEDED",
        ] {
            let decoded = serde_json::from_value(json!({
                "id": "", "organizationId": "org", "status": status,
                "type": "ACTIVITY_TYPE_APPROVE_ACTIVITY", "fingerprint": "sha256:example"
            }))
            .unwrap();
            let output = from_activity("test", decoded);
            assert_eq!(output.code, Some("submission_unknown"));
            assert_eq!(output.status, "unknown");
            assert!(output.failed());
        }
    }
    #[test]
    fn result_serialization_preserves_the_machine_contract() {
        let data = json!({"activity":{"id":"a","status":"ACTIVITY_STATUS_COMPLETED"}});
        let output = OperationOutput::result("test", data);
        assert_eq!(
            serde_json::to_value(output).unwrap(),
            json!({
                "schemaVersion":1,"reason":"command_result","command":"test","status":"completed",
                "data":{"activity":{"id":"a","status":"ACTIVITY_STATUS_COMPLETED"}},
                "activity":{"id":"a","status":"ACTIVITY_STATUS_COMPLETED"}
            })
        );
    }
    #[test]
    fn submission_requires_recoverable_activity_identity() {
        let output = submission_result(
            "test",
            json!({"activity":{"status":"ACTIVITY_STATUS_COMPLETED"}}),
        );
        assert_eq!(output.code, Some("submission_unknown"));
        assert_eq!(output.status, "unknown");
    }
    #[test]
    fn parser_enforces_body_source_and_safe_path() {
        assert!(RequestCli::try_parse_from(["tk", "--path", "/public/v1/query/whoami"]).is_err());
        assert!(
            RequestCli::try_parse_from([
                "tk",
                "--path",
                "/public/v1/query/whoami",
                "--body",
                "{}",
                "--body-file",
                "-"
            ])
            .is_err()
        );
        assert!(
            RequestCli::try_parse_from(["tk", "--path", "https://other.test", "--body", "{}"])
                .is_err()
        );
        assert!(ActivityCli::try_parse_from(["tk", "wait", "a", "--timeout", "0"]).is_err());
    }
    #[tokio::test]
    async fn raw_request_preserves_signed_bytes() {
        let server = MockServer::start().await;
        let body = "{\n  \"organizationId\": \"org\"\n}\n";
        let key = TurnkeyP256ApiKey::generate();
        let stamp = key.stamp(body.as_bytes()).unwrap();
        Mock::given(method("POST"))
            .and(path("/public/v1/query/whoami"))
            .and(body_string(body))
            .and(wiremock::matchers::header(
                stamp.name.as_str(),
                stamp.value.as_str(),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"userId":"user"})))
            .expect(1)
            .mount(&server)
            .await;
        let output = run_request(
            RequestArgs {
                path: "/public/v1/query/whoami".into(),
                body: Some(body.into()),
                body_file: None,
                stamp_only: false,
            },
            "org",
            &server.uri(),
            &key,
        )
        .await;
        assert!(!output.failed());
        assert_eq!(output.data.as_deref(), Some(&json!({"userId":"user"})));
        server.verify().await;
    }
    #[tokio::test]
    async fn stamp_only_and_mismatch_make_no_requests() {
        let server = MockServer::start().await;
        let key = TurnkeyP256ApiKey::generate();
        let args = |org: &str| RequestArgs {
            path: "/public/v1/query/whoami".into(),
            body: Some(json!({"organizationId":org}).to_string()),
            body_file: None,
            stamp_only: true,
        };
        assert!(
            !run_request(args("org"), "org", &server.uri(), &key)
                .await
                .failed()
        );
        assert_eq!(
            run_request(args("other"), "org", &server.uri(), &key)
                .await
                .code,
            Some("invalid_input")
        );
        assert!(server.received_requests().await.unwrap().is_empty());
    }
    #[tokio::test]
    async fn redirect_is_not_followed() {
        let server = MockServer::start().await;
        Mock::given(path("/public/v1/submit/test"))
            .respond_with(ResponseTemplate::new(307).insert_header("Location", "/leaked"))
            .expect(1)
            .mount(&server)
            .await;
        let key = TurnkeyP256ApiKey::generate();
        let output = submit(
            "test",
            "/public/v1/submit/test",
            &json!({}),
            &server.uri(),
            &key,
        )
        .await;
        assert_eq!(output.http_status, Some(307));
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }
    #[tokio::test]
    async fn pending_submission_retains_activity_without_retry() {
        let server = MockServer::start().await;
        Mock::given(path("/public/v1/submit/test"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(activity("a", "ACTIVITY_STATUS_CONSENSUS_NEEDED")),
            )
            .expect(1)
            .mount(&server)
            .await;
        let output = submit(
            "test",
            "/public/v1/submit/test",
            &json!({}),
            &server.uri(),
            &TurnkeyP256ApiKey::generate(),
        )
        .await;
        assert_eq!(output.status, "pending");
        assert_eq!(
            output.activity.as_deref(),
            Some(&json!({"id":"a","status":"ACTIVITY_STATUS_CONSENSUS_NEEDED"}))
        );
        assert!(!output.failed());
        server.verify().await;
    }
    #[tokio::test]
    async fn wait_timeout_retains_identity_and_get_rejection_is_success() {
        let server = MockServer::start().await;
        let key = TurnkeyP256ApiKey::generate();
        Mock::given(path("/public/v1/query/get_activity"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(activity("a", "ACTIVITY_STATUS_CONSENSUS_NEEDED")),
            )
            .mount(&server)
            .await;
        let output = run_activity(
            ActivityCommand::Wait {
                id: "a".into(),
                timeout: 1,
            },
            "org",
            &server.uri(),
            &key,
        )
        .await;
        assert_eq!(output.code, Some("wait_timeout"));
        assert_eq!(output.activity.unwrap()["id"], "a");
        server.reset().await;
        Mock::given(path("/public/v1/query/get_activity"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(activity("a", "ACTIVITY_STATUS_REJECTED")),
            )
            .mount(&server)
            .await;
        let output = run_activity(
            ActivityCommand::Get { id: "a".into() },
            "org",
            &server.uri(),
            &key,
        )
        .await;
        assert!(!output.failed());
        assert_eq!(output.status, "rejected");
        let waited = run_activity(
            ActivityCommand::Wait {
                id: "a".into(),
                timeout: 1,
            },
            "org",
            &server.uri(),
            &key,
        )
        .await;
        assert!(waited.failed());
    }
    #[tokio::test]
    async fn reject_success_and_malformed_submission_are_distinct() {
        let server = MockServer::start().await;
        let key = TurnkeyP256ApiKey::generate();
        Mock::given(path("/public/v1/query/get_activity"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(activity("a", "ACTIVITY_STATUS_CONSENSUS_NEEDED")),
            )
            .mount(&server)
            .await;
        Mock::given(path("/public/v1/submit/reject_activity"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(activity("a", "ACTIVITY_STATUS_REJECTED")),
            )
            .expect(1)
            .mount(&server)
            .await;
        let output = run_activity(
            ActivityCommand::Reject { id: "a".into() },
            "org",
            &server.uri(),
            &key,
        )
        .await;
        assert!(!output.failed());
        assert_eq!(output.status, "rejected");
        Mock::given(path("/public/v1/submit/test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
            .expect(1)
            .mount(&server)
            .await;
        assert_eq!(
            submit(
                "test",
                "/public/v1/submit/test",
                &json!({}),
                &server.uri(),
                &key
            )
            .await
            .code,
            Some("submission_unknown")
        );
        server.verify().await;
    }
    #[tokio::test]
    async fn list_preserves_api_cursor_and_does_not_fetch_extra_pages() {
        let server = MockServer::start().await;
        Mock::given(path("/public/v1/query/list_activities"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"activities":[{"id":"b"}]})),
            )
            .expect(1)
            .mount(&server)
            .await;
        let output = run_activity(
            ActivityCommand::List {
                limit: 1,
                cursor: Some("a".into()),
            },
            "org",
            &server.uri(),
            &TurnkeyP256ApiKey::generate(),
        )
        .await;
        assert_eq!(
            output.data.as_deref(),
            Some(&json!({"items":[{"id":"b"}],"nextCursor":"b"}))
        );
        let requests = server.received_requests().await.unwrap();
        let request: GetActivitiesRequest = serde_json::from_slice(&requests[0].body).unwrap();
        assert_eq!(
            request.pagination_options,
            Some(Pagination {
                limit: "1".into(),
                before: String::new(),
                after: "a".into()
            })
        );
        server.verify().await;
    }
    #[tokio::test]
    async fn mutation_timeout_is_unknown_and_does_not_leak_body() {
        let server = MockServer::start().await;
        Mock::given(path("/public/v1/submit/test"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(Duration::from_millis(100))
                    .set_body_json(json!({})),
            )
            .expect(1)
            .mount(&server)
            .await;
        let http = Client::builder()
            .timeout(Duration::from_millis(10))
            .build()
            .unwrap();
        let output = post(
            "test",
            &http,
            url(&server.uri(), "/public/v1/submit/test").unwrap(),
            "secret-marker".into(),
            &TurnkeyP256ApiKey::generate(),
            true,
        )
        .await
        .unwrap_err();
        assert_eq!(output.code, Some("submission_unknown"));
        assert!(
            !serde_json::to_string(&output)
                .unwrap()
                .contains("secret-marker")
        );
        server.verify().await;
    }
    #[tokio::test]
    async fn vote_submission_failures_retain_last_observed_target() {
        for approve in [true, false] {
            for timeout in [true, false] {
                let server = MockServer::start().await;
                let key = TurnkeyP256ApiKey::generate();
                Mock::given(path("/public/v1/query/get_activity"))
                    .respond_with(
                        ResponseTemplate::new(200)
                            .set_body_json(activity("target", "ACTIVITY_STATUS_CONSENSUS_NEEDED")),
                    )
                    .expect(1)
                    .mount(&server)
                    .await;
                let vote_path = if approve {
                    "/public/v1/submit/approve_activity"
                } else {
                    "/public/v1/submit/reject_activity"
                };
                let response = ResponseTemplate::new(200).set_body_json(json!({}));
                Mock::given(path(vote_path))
                    .respond_with(if timeout {
                        response.set_delay(Duration::from_millis(500))
                    } else {
                        response
                    })
                    .expect(1)
                    .mount(&server)
                    .await;
                let http = Client::builder()
                    .timeout(Duration::from_millis(100))
                    .build()
                    .unwrap();
                let args = if approve {
                    ActivityCommand::Approve {
                        id: "target".into(),
                    }
                } else {
                    ActivityCommand::Reject {
                        id: "target".into(),
                    }
                };
                let command = if approve {
                    "activity.approve"
                } else {
                    "activity.reject"
                };
                let output = activity_inner(command, args, "org", &server.uri(), &key, &http)
                    .await
                    .unwrap_err();
                assert_eq!(output.code, Some("submission_unknown"));
                assert_eq!(output.status, "unknown");
                assert_eq!(
                    output.activity.as_deref(),
                    Some(&json!({"id":"target", "status":"ACTIVITY_STATUS_CONSENSUS_NEEDED"}))
                );
                server.verify().await;
            }
        }
    }
    #[tokio::test]
    async fn approve_fetches_fingerprint_then_submits_once() {
        let server = MockServer::start().await;
        Mock::given(path("/public/v1/query/get_activity"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(activity("a", "ACTIVITY_STATUS_CONSENSUS_NEEDED")),
            )
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(path("/public/v1/submit/approve_activity"))
            .and(header_exists("X-Stamp"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(activity("vote", "ACTIVITY_STATUS_COMPLETED")),
            )
            .expect(1)
            .mount(&server)
            .await;
        let output = run_activity(
            ActivityCommand::Approve { id: "a".into() },
            "org",
            &server.uri(),
            &TurnkeyP256ApiKey::generate(),
        )
        .await;
        assert!(!output.failed());
        let requests = server.received_requests().await.unwrap();
        let body: ApproveActivityRequest = serde_json::from_slice(&requests[1].body).unwrap();
        assert_eq!(body.parameters.unwrap().fingerprint, "sha256:example");
        assert_eq!(body.organization_id, "org");
        server.verify().await;
    }
}
