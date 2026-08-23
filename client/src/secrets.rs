//! High-level helpers for importing, listing, and exporting secrets.

use std::collections::BTreeMap;

use serde::Serialize;
use sha2::{Digest, Sha256};
use turnkey_api_key_stamper::Stamp;
use turnkey_enclave_encrypt::{ExportClient, ImportClient, QuorumPublicKey};

use crate::generated::external::activity::v1::{
    Activity, ExportSecretsRequest, InitImportSecretsRequest,
};
use crate::generated::external::options::v1::Pagination;
use crate::generated::immutable::activity::v1::{
    ActivityStatus, ActivityType, ExportSecretParams, ExportSecretsIntent, ExportSecretsResult,
    ImportSecretParams, ImportSecretsIntent, InitImportSecretsIntent, InitImportSecretsResult,
    result,
};
use crate::generated::immutable::models::v1::{KeyValue, TransportEncryptionSuite};
use crate::generated::services::coordinator::public::v1::{
    ActivityResponse, GetActivitiesRequest, GetActivityRequest, ListSecretsRequest, SecretMetadata,
};
use crate::{TurnkeyClient, TurnkeyClientError};

const EXPORT_SECRETS_PATH: &str = "/public/v1/submit/export_secrets";
const INIT_IMPORT_SECRETS_PATH: &str = "/public/v1/submit/init_import_secrets";

/// Input for importing one UTF-8 secret.
#[derive(Debug, Clone)]
pub struct ImportSecretInput {
    /// Organization receiving the secret.
    pub organization_id: String,
    /// Caller-provided activity timestamp in Unix milliseconds.
    pub timestamp_ms: u128,
    /// Plaintext UTF-8 secret material.
    pub plaintext: String,
    /// Optional display name.
    pub name: Option<String>,
    /// Deterministically ordered policy-visible properties.
    pub static_properties: BTreeMap<String, String>,
}

/// Input for exporting one secret.
#[derive(Debug, Clone)]
pub struct ExportSecretInput {
    /// Organization that owns the secret.
    pub organization_id: String,
    /// Caller-provided activity timestamp in Unix milliseconds.
    pub timestamp_ms: u128,
    /// Secret identifier.
    pub secret_id: String,
}

/// Input for preparing a canonical export proposal.
#[derive(Debug, Clone)]
pub struct CreateExportSecretsProposalInput {
    /// Organization that owns the secrets.
    pub organization_id: String,
    /// Caller-provided activity timestamp in Unix milliseconds.
    pub timestamp_ms: u128,
    /// Non-empty secret identifiers to export.
    pub secret_ids: Vec<String>,
}

/// Validated, shareable canonical export request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSecretsProposal {
    canonical_body: String,
    fingerprint: String,
}

impl ExportSecretsProposal {
    /// Reconstruct and validate a proposal received from another participant.
    pub fn from_canonical_body(body: impl Into<String>) -> Result<Self, TurnkeyClientError> {
        let canonical_body = body.into();
        let request = validate_proposal_body(canonical_body.as_bytes())?;
        validate_request(&request)?;
        let fingerprint = fingerprint(canonical_body.as_bytes());
        Ok(Self {
            canonical_body,
            fingerprint,
        })
    }

    /// Exact JSON bytes that participants must stamp and submit.
    #[must_use]
    pub fn canonical_body(&self) -> &str {
        &self.canonical_body
    }

    /// SHA-256 fingerprint of the exact canonical body.
    #[must_use]
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    fn validated_request(&self) -> Result<ExportSecretsRequest, TurnkeyClientError> {
        let request = validate_proposal_body(self.canonical_body.as_bytes())?;
        validate_request(&request)?;
        let actual = fingerprint(self.canonical_body.as_bytes());
        if actual != self.fingerprint {
            return Err(TurnkeyClientError::SecretFingerprintMismatch {
                expected: self.fingerprint.clone(),
                actual,
            });
        }
        Ok(request)
    }
}

/// A proposal paired with its one-shot, zeroizing recipient key.
pub struct PreparedExportSecrets {
    proposal: ExportSecretsProposal,
    recipient: ExportClient,
}

impl PreparedExportSecrets {
    /// Shareable canonical proposal.
    #[must_use]
    pub fn proposal(&self) -> &ExportSecretsProposal {
        &self.proposal
    }
}

/// Result of submitting an export proposal.
#[derive(Debug, Clone)]
pub struct ExportSecretsSubmission {
    /// Activity identifier returned by this exact submission.
    pub activity_id: String,
    /// Server-reported request fingerprint.
    pub fingerprint: String,
    /// Current activity status.
    pub status: ActivityStatus,
    /// Payloads returned when the activity completed immediately.
    pub secret_payloads: Vec<String>,
}

impl<S: Stamp> TurnkeyClient<S> {
    /// Imports one UTF-8 secret and returns its identifier.
    pub async fn import_secret(
        &self,
        input: ImportSecretInput,
        signer_quorum_public_key: &QuorumPublicKey,
    ) -> Result<String, TurnkeyClientError> {
        if input.organization_id.is_empty() {
            return Err(TurnkeyClientError::MalformedSecretsProposal(
                "organization ID must not be empty".to_string(),
            ));
        }
        let init_request = InitImportSecretsRequest {
            r#type: "ACTIVITY_TYPE_INIT_IMPORT_SECRETS".to_string(),
            timestamp_ms: input.timestamp_ms.to_string(),
            organization_id: input.organization_id.clone(),
            parameters: Some(InitImportSecretsIntent {
                encryption_suite: TransportEncryptionSuite::EnclaveEncryptV1,
                num_secrets: 1,
            }),
        };
        let activity = self
            .process_activity(&init_request, INIT_IMPORT_SECRETS_PATH.to_string())
            .await?;
        let targets = init_import_result(activity)?.enclave_target_messages;
        let target = exactly_one(targets, "init import target")?;
        let encrypted = ImportClient::new(signer_quorum_public_key).encrypt_secret_with_bundle(
            &input.plaintext,
            target,
            &input.organization_id,
        )?;
        let static_properties = input
            .static_properties
            .into_iter()
            .map(|(key, value)| KeyValue { key, value })
            .collect();
        let imported = self
            .import_secrets(
                input.organization_id,
                input.timestamp_ms,
                ImportSecretsIntent {
                    secrets: vec![ImportSecretParams {
                        name: input.name,
                        secret_payload: encrypted.secret_payload,
                        target_public_key: encrypted.target_public_key,
                        encryption_suite: TransportEncryptionSuite::EnclaveEncryptV1,
                        static_properties,
                    }],
                },
            )
            .await?;
        exactly_one(imported.result.secret_ids, "imported secret ID")
    }

    /// Lists secret metadata for the supplied query.
    pub async fn get_secrets(
        &self,
        request: ListSecretsRequest,
    ) -> Result<Vec<SecretMetadata>, TurnkeyClientError> {
        Ok(self.list_secrets(request).await?.secrets)
    }

    /// Creates a canonical proposal and retains its one-shot recipient key.
    pub fn create_export_secrets_proposal(
        &self,
        input: CreateExportSecretsProposalInput,
        signer_quorum_public_key: &QuorumPublicKey,
    ) -> Result<PreparedExportSecrets, TurnkeyClientError> {
        if input.organization_id.is_empty() || input.secret_ids.iter().any(String::is_empty) {
            return Err(TurnkeyClientError::MalformedSecretsProposal(
                "organization and secret IDs must not be empty".to_string(),
            ));
        }
        if input.secret_ids.is_empty() {
            return Err(TurnkeyClientError::MalformedSecretsProposal(
                "at least one secret is required".to_string(),
            ));
        }
        let recipient = ExportClient::new(signer_quorum_public_key);
        let target_public_key = recipient.target_public_key()?;
        let request = ExportSecretsRequest {
            r#type: "ACTIVITY_TYPE_EXPORT_SECRETS".to_string(),
            timestamp_ms: input.timestamp_ms.to_string(),
            organization_id: input.organization_id,
            parameters: Some(ExportSecretsIntent {
                secrets: input
                    .secret_ids
                    .into_iter()
                    .map(|secret_id| ExportSecretParams {
                        secret_id,
                        target_public_key: target_public_key.clone(),
                        encryption_suite: TransportEncryptionSuite::EnclaveEncryptV1,
                    })
                    .collect(),
            }),
        };
        let canonical_body = serde_json::to_string(&request)?;
        let proposal = ExportSecretsProposal::from_canonical_body(canonical_body)?;
        Ok(PreparedExportSecrets {
            proposal,
            recipient,
        })
    }

    /// Submits the exact canonical proposal bytes once, without polling.
    pub async fn submit_export_secrets(
        &self,
        proposal: &ExportSecretsProposal,
    ) -> Result<ExportSecretsSubmission, TurnkeyClientError> {
        let request = proposal.validated_request()?;
        let response: ActivityResponse = self
            .process_serialized_request(
                proposal.canonical_body.as_bytes(),
                EXPORT_SECRETS_PATH.to_string(),
            )
            .await?;
        let activity = response
            .activity
            .ok_or(TurnkeyClientError::MissingActivity)?;
        validate_export_activity(&activity, &request.organization_id, proposal.fingerprint())?;
        submission_from_activity(activity)
    }

    /// Waits for a known submission by its activity ID and decrypts all payloads.
    pub async fn await_exported_secrets(
        &self,
        mut prepared: PreparedExportSecrets,
        submission: ExportSecretsSubmission,
    ) -> Result<Vec<String>, TurnkeyClientError> {
        let request = prepared.proposal.validated_request()?;
        if submission.fingerprint != prepared.proposal.fingerprint {
            return Err(TurnkeyClientError::SecretFingerprintMismatch {
                expected: prepared.proposal.fingerprint.clone(),
                actual: submission.fingerprint,
            });
        }
        let expected = request
            .parameters
            .as_ref()
            .map_or(0, |parameters| parameters.secrets.len());
        let payloads = match submission.status {
            ActivityStatus::Completed => submission.secret_payloads,
            ActivityStatus::ConsensusNeeded | ActivityStatus::AuthenticatorsNeeded => {
                return Err(TurnkeyClientError::ActivityRequiresApproval(
                    submission.activity_id,
                ));
            }
            ActivityStatus::Failed => return Err(TurnkeyClientError::ActivityFailed(None)),
            ActivityStatus::Pending | ActivityStatus::Created => {
                self.poll_export_activity(
                    &request.organization_id,
                    &submission.activity_id,
                    &prepared.proposal.fingerprint,
                )
                .await?
            }
            status => {
                return Err(TurnkeyClientError::UnexpectedActivityStatus(
                    status.as_str_name().to_string(),
                ));
            }
        };
        validate_payload_count(expected, payloads.len())?;
        prepared
            .recipient
            .decrypt_secret_payloads(&payloads, &request.organization_id)
            .map_err(|error| match error {
                turnkey_enclave_encrypt::errors::EnclaveEncryptError::InvalidUtf8Bytes(message) => {
                    TurnkeyClientError::InvalidSecretUtf8(message)
                }
                other => TurnkeyClientError::EnclaveEncrypt(other),
            })
    }

    /// Finds a previously submitted proposal by fingerprint, then awaits it by activity ID.
    ///
    /// This is an explicit fallback for participants that did not perform the submission.
    pub async fn await_exported_secrets_by_fingerprint(
        &self,
        prepared: PreparedExportSecrets,
    ) -> Result<Vec<String>, TurnkeyClientError> {
        let request = prepared.proposal.validated_request()?;
        let mut before = String::new();
        loop {
            let response = self
                .get_activities(GetActivitiesRequest {
                    organization_id: request.organization_id.clone(),
                    filter_by_status: Vec::new(),
                    pagination_options: Some(Pagination {
                        limit: "100".to_string(),
                        before: before.clone(),
                        after: String::new(),
                    }),
                    filter_by_type: vec![ActivityType::ExportSecrets],
                })
                .await?;
            if let Some(activity) = response
                .activities
                .iter()
                .find(|activity| activity.fingerprint == prepared.proposal.fingerprint)
            {
                let submission = submission_from_activity(activity.clone())?;
                return self.await_exported_secrets(prepared, submission).await;
            }
            if response.activities.len() < 100 {
                break;
            }
            let next = response
                .activities
                .last()
                .map(|activity| activity.id.clone())
                .ok_or_else(|| {
                    TurnkeyClientError::MissingSecretResult(
                        "activity pagination cursor".to_string(),
                    )
                })?;
            if next == before {
                break;
            }
            before = next;
        }
        Err(TurnkeyClientError::MissingSecretResult(format!(
            "no export activity found for {}",
            prepared.proposal.fingerprint
        )))
    }

    /// Exports and decrypts one secret using the submission's returned activity ID.
    pub async fn export_secret(
        &self,
        input: ExportSecretInput,
        signer_quorum_public_key: &QuorumPublicKey,
    ) -> Result<String, TurnkeyClientError> {
        let prepared = self.create_export_secrets_proposal(
            CreateExportSecretsProposalInput {
                organization_id: input.organization_id,
                timestamp_ms: input.timestamp_ms,
                secret_ids: vec![input.secret_id],
            },
            signer_quorum_public_key,
        )?;
        let submission = self.submit_export_secrets(prepared.proposal()).await?;
        let secrets = self.await_exported_secrets(prepared, submission).await?;
        exactly_one(secrets, "exported secret")
    }

    async fn poll_export_activity(
        &self,
        organization_id: &str,
        activity_id: &str,
        expected_fingerprint: &str,
    ) -> Result<Vec<String>, TurnkeyClientError> {
        for retry_count in 0..=self.retry_config.max_retries {
            let response = self
                .get_activity(GetActivityRequest {
                    organization_id: organization_id.to_string(),
                    activity_id: activity_id.to_string(),
                })
                .await?;
            let activity = response
                .activity
                .ok_or(TurnkeyClientError::MissingActivity)?;
            validate_export_activity(&activity, organization_id, expected_fingerprint)?;
            match activity.status {
                ActivityStatus::Completed => return export_result(activity),
                ActivityStatus::ConsensusNeeded | ActivityStatus::AuthenticatorsNeeded => {
                    return Err(TurnkeyClientError::ActivityRequiresApproval(activity.id));
                }
                ActivityStatus::Failed => {
                    return Err(TurnkeyClientError::ActivityFailed(activity.failure));
                }
                ActivityStatus::Pending | ActivityStatus::Created => {
                    if retry_count == self.retry_config.max_retries {
                        return Err(TurnkeyClientError::ExceededRetries(retry_count));
                    }
                    tokio::time::sleep(self.retry_config.compute_delay(retry_count + 1)).await;
                }
                status => {
                    return Err(TurnkeyClientError::UnexpectedActivityStatus(
                        status.as_str_name().to_string(),
                    ));
                }
            }
        }
        Err(TurnkeyClientError::ExceededRetries(
            self.retry_config.max_retries,
        ))
    }
}

fn validate_proposal_body(body: &[u8]) -> Result<ExportSecretsRequest, TurnkeyClientError> {
    let request: ExportSecretsRequest = serde_json::from_slice(body).map_err(|error| {
        TurnkeyClientError::MalformedSecretsProposal(format!("invalid JSON: {error}"))
    })?;
    let canonical = serde_json::to_vec(&request)?;
    if canonical != body {
        return Err(TurnkeyClientError::MalformedSecretsProposal(
            "request body is not in canonical serialized form".to_string(),
        ));
    }
    Ok(request)
}

fn validate_request(request: &ExportSecretsRequest) -> Result<(), TurnkeyClientError> {
    if request.r#type != "ACTIVITY_TYPE_EXPORT_SECRETS" {
        return Err(TurnkeyClientError::MalformedSecretsProposal(
            "unexpected request type".to_string(),
        ));
    }
    if request.organization_id.is_empty() {
        return Err(TurnkeyClientError::MalformedSecretsProposal(
            "organization ID must not be empty".to_string(),
        ));
    }
    let secrets = request
        .parameters
        .as_ref()
        .ok_or_else(|| {
            TurnkeyClientError::MalformedSecretsProposal("missing parameters".to_string())
        })?
        .secrets
        .as_slice();
    if secrets.is_empty() || secrets.iter().any(|secret| secret.secret_id.is_empty()) {
        return Err(TurnkeyClientError::MalformedSecretsProposal(
            "secret IDs must not be empty".to_string(),
        ));
    }
    Ok(())
}

fn fingerprint(body: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(body)))
}

fn init_import_result(activity: Activity) -> Result<InitImportSecretsResult, TurnkeyClientError> {
    match activity
        .result
        .ok_or_else(|| {
            TurnkeyClientError::MissingSecretResult("init import activity result".to_string())
        })?
        .inner
        .ok_or_else(|| {
            TurnkeyClientError::MissingSecretResult("init import inner activity result".to_string())
        })? {
        result::Inner::InitImportSecretsResult(value) => Ok(value),
        other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
            serde_json::to_string(&other)?,
        )),
    }
}

fn export_result(activity: Activity) -> Result<Vec<String>, TurnkeyClientError> {
    match activity
        .result
        .ok_or_else(|| {
            TurnkeyClientError::MissingSecretResult("export activity result".to_string())
        })?
        .inner
        .ok_or_else(|| {
            TurnkeyClientError::MissingSecretResult("export inner activity result".to_string())
        })? {
        result::Inner::ExportSecretsResult(ExportSecretsResult { secret_payloads }) => {
            Ok(secret_payloads)
        }
        other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
            serde_json::to_string(&other)?,
        )),
    }
}

fn submission_from_activity(
    activity: Activity,
) -> Result<ExportSecretsSubmission, TurnkeyClientError> {
    let secret_payloads = if activity.status == ActivityStatus::Completed {
        export_result(activity.clone())?
    } else {
        Vec::new()
    };
    Ok(ExportSecretsSubmission {
        activity_id: activity.id,
        fingerprint: activity.fingerprint,
        status: activity.status,
        secret_payloads,
    })
}

fn validate_export_activity(
    activity: &Activity,
    expected_organization_id: &str,
    expected_fingerprint: &str,
) -> Result<(), TurnkeyClientError> {
    if activity.r#type != ActivityType::ExportSecrets {
        return Err(TurnkeyClientError::MalformedSecretsProposal(
            "response has unexpected activity type".to_string(),
        ));
    }
    if activity.organization_id != expected_organization_id {
        return Err(TurnkeyClientError::MalformedSecretsProposal(
            "response organization does not match proposal".to_string(),
        ));
    }
    if activity.fingerprint != expected_fingerprint {
        return Err(TurnkeyClientError::SecretFingerprintMismatch {
            expected: expected_fingerprint.to_string(),
            actual: activity.fingerprint.clone(),
        });
    }
    Ok(())
}

fn exactly_one<T>(mut values: Vec<T>, label: &str) -> Result<T, TurnkeyClientError> {
    if values.len() != 1 {
        return Err(TurnkeyClientError::MissingSecretResult(format!(
            "expected one {label}, received {}",
            values.len()
        )));
    }
    Ok(values.remove(0))
}

fn validate_payload_count(expected: usize, actual: usize) -> Result<(), TurnkeyClientError> {
    if expected != actual {
        return Err(TurnkeyClientError::SecretPayloadCountMismatch { expected, actual });
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::time::Duration;

    use p256::ecdsa::SigningKey;
    use sha2::{Digest, Sha256};
    use turnkey_enclave_encrypt::{P256Public, server::EnclaveEncryptServer};
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;
    use crate::{RetryConfig, TurnkeyP256ApiKey};

    fn test_quorum_public_key() -> QuorumPublicKey {
        QuorumPublicKey::from_string(concat!(
            "04",
            "aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899",
            "99887766554433221100ffeeddccbbaa99887766554433221100ffeeddccbbaa",
            "04",
            "8ee67fa8ae8e5fac0e343c84fa0921ecb3a31a67aee2e6a0880a09072eaaf2ae",
            "ffa43f73fa021fa4d0b550072ba1f9011ff7cf917e4bf2708670e5ac57a81c78"
        ))
        .unwrap()
    }

    fn test_quorum_private_key() -> SigningKey {
        SigningKey::from_slice(
            &hex::decode("28ebf311b27f34cdf078489584d336423e09c522342f5b067dea36823c2cc5ed")
                .unwrap(),
        )
        .unwrap()
    }

    async fn setup() -> (TurnkeyClient<TurnkeyP256ApiKey>, MockServer) {
        let server = MockServer::start().await;
        let client = TurnkeyClient::builder()
            .api_key(TurnkeyP256ApiKey::generate())
            .base_url(server.uri())
            .retry_config(RetryConfig {
                initial_delay: Duration::from_millis(1),
                multiplier: 1.0,
                max_delay: Duration::from_millis(1),
                max_retries: 1,
            })
            .build()
            .unwrap();
        (client, server)
    }

    fn prepared(client: &TurnkeyClient<TurnkeyP256ApiKey>) -> PreparedExportSecrets {
        client
            .create_export_secrets_proposal(
                CreateExportSecretsProposalInput {
                    organization_id: "org-id".to_string(),
                    timestamp_ms: 123,
                    secret_ids: vec!["secret-id".to_string()],
                },
                &test_quorum_public_key(),
            )
            .unwrap()
    }

    fn activity_json(
        status: &str,
        fingerprint: &str,
        result: serde_json::Value,
    ) -> serde_json::Value {
        serde_json::json!({
            "activity": activity_value("activity-id", status, fingerprint, result)
        })
    }

    fn activity_value(
        id: &str,
        status: &str,
        fingerprint: &str,
        result: serde_json::Value,
    ) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "organizationId": "org-id",
            "status": status,
            "type": "ACTIVITY_TYPE_EXPORT_SECRETS",
            "result": result,
            "votes": [],
            "appProofs": [],
            "fingerprint": fingerprint,
            "canApprove": false,
            "canReject": false
        })
    }

    #[test]
    fn proposal_is_canonical_and_fingerprinted() {
        let client = TurnkeyClient::builder()
            .api_key(TurnkeyP256ApiKey::generate())
            .build()
            .unwrap();
        let prepared = prepared(&client);
        let body = prepared.proposal().canonical_body();
        assert_eq!(
            prepared.proposal().fingerprint(),
            format!("sha256:{}", hex::encode(Sha256::digest(body.as_bytes())))
        );
        let reparsed = ExportSecretsProposal::from_canonical_body(body).unwrap();
        assert_eq!(reparsed.canonical_body(), body);
        assert_eq!(reparsed.fingerprint(), prepared.proposal().fingerprint());
    }

    #[test]
    fn malformed_proposals_are_rejected() {
        assert!(matches!(
            ExportSecretsProposal::from_canonical_body("{}"),
            Err(TurnkeyClientError::SerdeJsonFailure(_))
                | Err(TurnkeyClientError::MalformedSecretsProposal(_))
        ));
        let wrong_type = r#"{"type":"ACTIVITY_TYPE_EXPORT_WALLET","timestampMs":"1","organizationId":"org","parameters":{"secrets":[{"secretId":"id","targetPublicKey":"04","encryptionSuite":"TRANSPORT_ENCRYPTION_SUITE_ENCLAVE_ENCRYPT_V1"}]}}"#;
        assert!(matches!(
            ExportSecretsProposal::from_canonical_body(wrong_type),
            Err(TurnkeyClientError::MalformedSecretsProposal(_))
        ));
    }

    #[tokio::test]
    async fn submit_preserves_exact_body_and_returns_consensus_activity_id() {
        let (client, server) = setup().await;
        let prepared = prepared(&client);
        let fingerprint = prepared.proposal().fingerprint().to_string();
        Mock::given(method("POST"))
            .and(path(EXPORT_SECRETS_PATH))
            .respond_with(ResponseTemplate::new(200).set_body_json(activity_json(
                "ACTIVITY_STATUS_CONSENSUS_NEEDED",
                &fingerprint,
                serde_json::Value::Null,
            )))
            .expect(1)
            .mount(&server)
            .await;

        let submission = client
            .submit_export_secrets(prepared.proposal())
            .await
            .unwrap();
        let requests = server.received_requests().await.unwrap();
        assert_eq!(
            requests[0].body,
            prepared.proposal().canonical_body().as_bytes()
        );
        assert!(matches!(
            client.await_exported_secrets(prepared, submission).await,
            Err(TurnkeyClientError::ActivityRequiresApproval(id)) if id == "activity-id"
        ));
    }

    #[tokio::test]
    async fn multiple_participants_submit_identical_proposal_bytes() {
        let (client, server) = setup().await;
        let prepared = prepared(&client);
        let fingerprint = prepared.proposal().fingerprint().to_string();
        Mock::given(method("POST"))
            .and(path(EXPORT_SECRETS_PATH))
            .respond_with(ResponseTemplate::new(200).set_body_json(activity_json(
                "ACTIVITY_STATUS_CONSENSUS_NEEDED",
                &fingerprint,
                serde_json::Value::Null,
            )))
            .expect(2)
            .mount(&server)
            .await;
        client
            .submit_export_secrets(prepared.proposal())
            .await
            .unwrap();
        client
            .submit_export_secrets(prepared.proposal())
            .await
            .unwrap();
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests[0].body, requests[1].body);
        assert_eq!(
            requests[0].body,
            prepared.proposal().canonical_body().as_bytes()
        );
    }

    #[tokio::test]
    async fn pending_submission_polls_returned_activity_id_and_rejects_partial_payloads() {
        let (client, server) = setup().await;
        let prepared = prepared(&client);
        let fingerprint = prepared.proposal().fingerprint().to_string();
        Mock::given(method("POST"))
            .and(path("/public/v1/query/get_activity"))
            .respond_with(ResponseTemplate::new(200).set_body_json(activity_json(
                "ACTIVITY_STATUS_COMPLETED",
                &fingerprint,
                serde_json::json!({"exportSecretsResult":{"secretPayloads":[]}}),
            )))
            .expect(1)
            .mount(&server)
            .await;
        let error = client
            .await_exported_secrets(
                prepared,
                ExportSecretsSubmission {
                    activity_id: "activity-id".to_string(),
                    fingerprint,
                    status: ActivityStatus::Pending,
                    secret_payloads: Vec::new(),
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            TurnkeyClientError::SecretPayloadCountMismatch {
                expected: 1,
                actual: 0
            }
        ));
    }

    #[tokio::test]
    async fn export_secret_decrypts_immediate_completion() {
        let (client, server) = setup().await;
        Mock::given(method("POST"))
            .and(path(EXPORT_SECRETS_PATH))
            .respond_with(|request: &wiremock::Request| {
                let export_request: ExportSecretsRequest =
                    serde_json::from_slice(&request.body).unwrap();
                let target: P256Public =
                    hex::decode(&export_request.parameters.unwrap().secrets[0].target_public_key)
                        .unwrap()
                        .try_into()
                        .unwrap();
                let enclave = EnclaveEncryptServer::from_enclave_auth_key(
                    test_quorum_private_key(),
                    "org-id".to_string(),
                    None,
                );
                let payload =
                    serde_json::to_string(&enclave.encrypt(&target, b"immediate secret").unwrap())
                        .unwrap();
                let fingerprint = format!(
                    "sha256:{}",
                    hex::encode(Sha256::digest(request.body.as_slice()))
                );
                ResponseTemplate::new(200).set_body_json(activity_json(
                    "ACTIVITY_STATUS_COMPLETED",
                    &fingerprint,
                    serde_json::json!({
                        "exportSecretsResult": {"secretPayloads": [payload]}
                    }),
                ))
            })
            .expect(1)
            .mount(&server)
            .await;
        let secret = client
            .export_secret(
                ExportSecretInput {
                    organization_id: "org-id".to_string(),
                    timestamp_ms: 123,
                    secret_id: "secret-id".to_string(),
                },
                &test_quorum_public_key(),
            )
            .await
            .unwrap();
        assert_eq!(secret, "immediate secret");
    }

    #[tokio::test]
    async fn submit_rejects_response_fingerprint_mismatch() {
        let (client, server) = setup().await;
        let prepared = prepared(&client);
        Mock::given(method("POST"))
            .and(path(EXPORT_SECRETS_PATH))
            .respond_with(ResponseTemplate::new(200).set_body_json(activity_json(
                "ACTIVITY_STATUS_CONSENSUS_NEEDED",
                "sha256:wrong",
                serde_json::Value::Null,
            )))
            .mount(&server)
            .await;
        assert!(matches!(
            client.submit_export_secrets(prepared.proposal()).await,
            Err(TurnkeyClientError::SecretFingerprintMismatch { .. })
        ));
    }

    #[tokio::test]
    async fn fingerprint_fallback_paginates_for_non_submitter() {
        let (client, server) = setup().await;
        let prepared = prepared(&client);
        let fingerprint = prepared.proposal().fingerprint().to_string();
        let first_page = (0..100)
            .map(|index| {
                activity_value(
                    &format!("id-{index}"),
                    "ACTIVITY_STATUS_CONSENSUS_NEEDED",
                    "sha256:other",
                    serde_json::Value::Null,
                )
            })
            .collect::<Vec<_>>();
        Mock::given(method("POST"))
            .and(path("/public/v1/query/list_activities"))
            .and(body_partial_json(serde_json::json!({
                "paginationOptions": {"before": ""}
            })))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"activities": first_page})),
            )
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/public/v1/query/list_activities"))
            .and(body_partial_json(serde_json::json!({
                "paginationOptions": {"before": "id-99"}
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "activities": [activity_value(
                    "matching-activity",
                    "ACTIVITY_STATUS_CONSENSUS_NEEDED",
                    &fingerprint,
                    serde_json::Value::Null
                )]
            })))
            .expect(1)
            .mount(&server)
            .await;
        assert!(matches!(
            client
                .await_exported_secrets_by_fingerprint(prepared)
                .await,
            Err(TurnkeyClientError::ActivityRequiresApproval(id)) if id == "matching-activity"
        ));
    }

    #[tokio::test]
    async fn get_secrets_returns_generated_metadata() {
        let (client, server) = setup().await;
        Mock::given(method("POST"))
            .and(path("/public/v1/query/list_secrets"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "secrets": [{
                    "secretId": "secret-id",
                    "name": "database",
                    "staticProperties": [],
                    "createdAtUnixMs": "123"
                }]
            })))
            .mount(&server)
            .await;
        let secrets = client
            .get_secrets(ListSecretsRequest {
                organization_id: "org-id".to_string(),
                pagination_options: None,
            })
            .await
            .unwrap();
        assert_eq!(secrets.len(), 1);
        assert_eq!(secrets[0].secret_id, "secret-id");
    }

    #[tokio::test]
    async fn import_secret_runs_init_encrypt_and_import_sequence() {
        let (client, server) = setup().await;
        let enclave = EnclaveEncryptServer::from_enclave_auth_key(
            test_quorum_private_key(),
            "org-id".to_string(),
            None,
        );
        let target = serde_json::to_string(&enclave.publish_secret_target().unwrap()).unwrap();
        Mock::given(method("POST"))
            .and(path(INIT_IMPORT_SECRETS_PATH))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(activity_json_for_type(
                    "ACTIVITY_TYPE_INIT_IMPORT_SECRETS",
                    serde_json::json!({
                        "initImportSecretsResult": {"enclaveTargetMessages": [target]}
                    }),
                )),
            )
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/public/v1/submit/import_secrets"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(activity_json_for_type(
                    "ACTIVITY_TYPE_IMPORT_SECRETS",
                    serde_json::json!({"importSecretsResult": {"secretIds": ["secret-id"]}}),
                )),
            )
            .expect(1)
            .mount(&server)
            .await;

        let secret_id = client
            .import_secret(
                ImportSecretInput {
                    organization_id: "org-id".to_string(),
                    timestamp_ms: 123,
                    plaintext: "secret value".to_string(),
                    name: Some("database".to_string()),
                    static_properties: BTreeMap::from([
                        ("environment".to_string(), "production".to_string()),
                        ("service".to_string(), "api".to_string()),
                    ]),
                },
                &test_quorum_public_key(),
            )
            .await
            .unwrap();
        assert_eq!(secret_id, "secret-id");

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 2);
        let import_body: serde_json::Value = serde_json::from_slice(&requests[1].body).unwrap();
        let properties = &import_body["parameters"]["secrets"][0]["staticProperties"];
        assert_eq!(properties[0]["key"], "environment");
        assert_eq!(properties[1]["key"], "service");
    }

    fn activity_json_for_type(activity_type: &str, result: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "activity": {
                "id": "activity-id",
                "organizationId": "org-id",
                "status": "ACTIVITY_STATUS_COMPLETED",
                "type": activity_type,
                "result": result,
                "votes": [],
                "appProofs": [],
                "fingerprint": "sha256:test",
                "canApprove": false,
                "canReject": false
            }
        })
    }
}
