use crate::{output::Shell, terminal::TerminalText};
use std::{
    fmt::{Display, Write as _},
    io::Write,
};
use turnkey_proofs::tvc::{
    AcceptanceEvidence, AcceptanceFacts, Accepted, ClaimEvidence, ProofCheck, Rejected,
};
use uuid::Uuid;

const DIVIDER: &str = "________________________________________________________________\n";

pub(crate) struct ResumedActivity<'a> {
    pub operation_id: Uuid,
    pub activity_id: &'a str,
}

/// Presents the recorded verification result and returns it for finalization.
pub(crate) fn present<W: Write, E: Write, T>(
    shell: &mut Shell<W, E>,
    acceptance: Result<Accepted<T>, Rejected>,
    verbose: bool,
    resumed: Option<ResumedActivity<'_>>,
) -> Result<T, Rejected> {
    if verbose && !shell.message_format().is_json() {
        let report = match &acceptance {
            Ok(accepted) => render(accepted.facts(), None, resumed),
            Err(rejected) => render(rejected.facts(), Some(rejected), resumed),
        };
        let _ = shell.human().err_line(report);
        if acceptance.is_ok() {
            let _ = shell.human().err_line(DIVIDER);
        }
    } else if acceptance.is_ok() {
        present_passed_status(shell);
    }
    acceptance.map(Accepted::into_value)
}

pub(crate) fn present_skipped<W: Write, E: Write>(shell: &mut Shell<W, E>, verbose: bool) {
    let mut human = shell.human();
    let _ = human.warn("Proof verification: SKIPPED");
    if verbose {
        let _ = human.err_line("Signer and AWS Nitro proofs were not checked. The result was not verified against the original request.");
        let _ = human.err_line(DIVIDER);
    }
}

fn present_passed_status<W: Write, E: Write>(shell: &mut Shell<W, E>) {
    let _ = shell.human().status("Proof verification", "passed");
}

fn row(s: &mut String, name: &str, value: impl Display) {
    let _ = writeln!(s, "  {name}: {}", TerminalText(value).single_line());
}

/// Successful acceptance covers every required check, including each app
/// signature when the verifier reuses boot evidence.
fn render(
    facts: &AcceptanceFacts,
    rejection: Option<&Rejected>,
    resumed: Option<ResumedActivity<'_>>,
) -> String {
    let title = match facts.acceptance() {
        AcceptanceEvidence::Operator { .. } => "HOSTED OPERATOR VERIFICATION",
        AcceptanceEvidence::Quorum { .. } => "QUORUM KEY VERIFICATION",
    };
    let mut report = format!(
        "{title} · {}\n",
        TerminalText(facts.environment()).single_line()
    );
    if let Some(resumed) = resumed {
        row(&mut report, "Local operation ID", resumed.operation_id);
        row(&mut report, "Activity queried", resumed.activity_id);
    }
    row(
        &mut report,
        "Pinned Signer quorum key",
        hex::encode(facts.signer_quorum_key().as_ref()),
    );
    let _ = writeln!(report);

    if let Some(rejection) = rejection {
        render_failure(&mut report, rejection);
        return report;
    }

    let _ = writeln!(
        report,
        r#"PASS  The pinned Signer endorsed the boot attestation.
PASS  The attestation signature and certificate chain verified to the AWS Nitro root at issuance.
PASS  The manifest, enclave measurements, and ephemeral key matched the attested values."#
    );
    match facts.acceptance() {
        AcceptanceEvidence::Operator {
            organization,
            wallet,
            base_path,
            ..
        } => {
            let _ = writeln!(
                report,
                r#"PASS  Attested enclave keys signed the encryption and signing claims.
PASS  Both claims matched the original saved request.
PASS  The organization and derivation paths matched the request.
PASS  The returned wallet and distinct role keys matched the signed claims."#
            );
            row(&mut report, "Organization", organization);
            row(&mut report, "Derivation base path", base_path);
            if let Some(wallet) = wallet {
                row(&mut report, "Matched requested wallet", wallet);
            }
        }
        AcceptanceEvidence::Quorum {
            organization,
            threshold,
            recipients,
            ..
        } => {
            let _ = writeln!(
                report,
                r#"PASS  An attested enclave key signed the quorum-key claim.
PASS  The claim matched the original saved request.
PASS  The organization, threshold, and recipient order matched the request.
PASS  The returned quorum key ID and public key matched the signed claim."#
            );
            row(&mut report, "Organization", organization);
            row(
                &mut report,
                "Threshold",
                format_args!("{threshold} of {}", recipients.len()),
            );
        }
    }
    if let Some(fingerprint) = facts.matched_fingerprint() {
        row(
            &mut report,
            "Matched request fingerprint",
            fingerprint.to_hex(),
        );
    }
    let routing_label = match facts.acceptance() {
        AcceptanceEvidence::Operator { .. } => {
            "The operator ID below is an unsigned routing label."
        }
        AcceptanceEvidence::Quorum { .. } => "The share IDs below are unsigned routing labels.",
    };
    let _ = writeln!(report, "\n{routing_label}");
    report
}

fn render_failure(report: &mut String, rejection: &Rejected) {
    let stage = rejection.stage();
    // Use the verifier's stage and authenticated claim facts. The command's
    // existing error still carries the original source and recovery guidance.
    let _ = writeln!(report, "FAIL  {stage}");
    let facts = rejection.facts();
    match stage.check {
        ProofCheck::CommonSubmission => {
            let requirement = match facts.acceptance() {
                AcceptanceEvidence::Operator { .. } => {
                    "one saved request fingerprint in both signed claims"
                }
                AcceptanceEvidence::Quorum { .. } => {
                    "a saved request fingerprint in the signed claim"
                }
            };
            let _ = writeln!(report, "  Verification could not find {requirement}.");
        }
        ProofCheck::ClaimSetAndIntent => {
            for claim in facts.envelopes().iter().filter_map(|proof| proof.claim()) {
                match (facts.acceptance(), claim) {
                    (
                        AcceptanceEvidence::Operator {
                            organization,
                            wallet,
                            base_path,
                            ..
                        },
                        ClaimEvidence::Operator {
                            organization: signed_org,
                            wallet: signed_wallet,
                            path,
                            ..
                        },
                    ) => {
                        difference(report, "organization", organization, signed_org);
                        if let Some(wallet) = wallet {
                            difference(report, "wallet", wallet, signed_wallet);
                        }
                        let paths = [format!("{base_path}/0"), format!("{base_path}/1")];
                        if !paths.contains(path) {
                            row(report, "Requested role paths", paths.join(", "));
                            row(report, "Signed role path", path);
                        }
                    }
                    (
                        AcceptanceEvidence::Quorum {
                            organization,
                            threshold,
                            recipients,
                            ..
                        },
                        ClaimEvidence::Quorum {
                            organization: signed_org,
                            threshold: signed_threshold,
                            recipients: signed_recipients,
                            ..
                        },
                    ) => {
                        difference(report, "organization", organization, signed_org);
                        if threshold != signed_threshold {
                            row(report, "Requested threshold", threshold);
                            row(report, "Signed threshold", signed_threshold);
                        }
                        if recipients.len() != signed_recipients.len() {
                            row(report, "Requested recipient count", recipients.len());
                            row(report, "Signed recipient count", signed_recipients.len());
                        }
                        if let Some(index) = recipients
                            .iter()
                            .zip(signed_recipients)
                            .position(|(expected, signed)| expected != signed)
                        {
                            row(
                                report,
                                &format!("Requested recipient [{index}]"),
                                recipients[index],
                            );
                            row(
                                report,
                                &format!("Signed recipient [{index}]"),
                                signed_recipients[index],
                            );
                        }
                    }
                    _ => (),
                }
            }
        }
        ProofCheck::ResultBinding => {
            for claim in facts.envelopes().iter().filter_map(|proof| proof.claim()) {
                match (facts.acceptance(), claim) {
                    (
                        AcceptanceEvidence::Operator {
                            base_path,
                            returned: Some(returned),
                            ..
                        },
                        ClaimEvidence::Operator {
                            wallet,
                            path,
                            public_key,
                            ..
                        },
                    ) => {
                        difference(report, "wallet", wallet, &returned.wallet);
                        let (role, key) = if *path == format!("{base_path}/0") {
                            ("encryption public key", &returned.encrypt)
                        } else {
                            ("signing public key", &returned.sign)
                        };
                        difference(report, role, &public_key.to_string(), key);
                    }
                    (
                        AcceptanceEvidence::Quorum {
                            returned: Some(returned),
                            ..
                        },
                        ClaimEvidence::Quorum { id, public_key, .. },
                    ) => {
                        difference(report, "quorum key ID", id, &returned.id);
                        difference(
                            report,
                            "quorum public key",
                            &hex::encode(public_key.as_ref()),
                            &returned.public_key,
                        );
                    }
                    _ => (),
                }
            }
        }
        _ => (),
    }
    let _ = writeln!(report, "  Verification stopped at this check.");
}

fn difference(report: &mut String, field: &str, expected: &str, observed: &str) {
    if expected != observed {
        row(report, &format!("Expected {field}"), expected);
        row(report, &format!("Observed {field}"), observed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::{ErrorMessage, TestShell};
    use serde_json::json;
    use std::convert::Infallible;
    use turnkey_client::{TurnkeyClient, TurnkeyP256ApiKey, generated::CreateTvcOperatorIntent};
    use turnkey_proofs::tvc::{OperatorExpectation, TvcProvenance, VerifiedOperatorKeys};
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

    #[test]
    fn success_status_is_human_only() {
        let mut human = TestShell::with_human_formatter();
        present_passed_status(&mut human);
        assert_eq!(human.into_stderr(), b"Proof verification: passed\n");

        let mut json = TestShell::with_json_formatter();
        present_passed_status(&mut json);
        assert!(json.into_stderr().is_empty());
    }

    fn completed<T>(outcome: turnkey_client::SubmissionOutcome<T>) -> T {
        match outcome {
            turnkey_client::SubmissionOutcome::Completed(value) => value,
            turnkey_client::SubmissionOutcome::Pending { .. } => {
                panic!("expected completed proof fixture")
            }
        }
    }

    async fn operator_acceptance(
        app_proofs: serde_json::Value,
    ) -> Result<Accepted<VerifiedOperatorKeys>, Rejected> {
        let organization_id = uuid::uuid!("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa");
        let server = MockServer::start().await;
        Mock::given(method("POST")).respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "activity": {
                "id": "activity-1", "organizationId": organization_id, "type": "ACTIVITY_TYPE_CREATE_TVC_OPERATOR",
                "status": "ACTIVITY_STATUS_COMPLETED", "fingerprint": "untrusted-fingerprint",
                "appProofs": app_proofs, "result": { "createTvcOperatorResult": {
                    "walletId": "wallet-1", "operatorId": "unsigned\n\u{1b}[2Jlabel",
                    "encryptPublicKey": "public-output-key", "signPublicKey": "other-public-output-key"
                }}
            }
        }))).mount(&server).await;
        let client = TurnkeyClient::builder()
            .api_key(TurnkeyP256ApiKey::from_bytes([6; 32], None).unwrap())
            .base_url(server.uri())
            .build()
            .unwrap()
            .with_app_proofs();
        let receipt = client
            .create_tvc_operator_with_receipt(
                organization_id,
                100,
                CreateTvcOperatorIntent {
                    wallet_name: Some("wallet".into()),
                    wallet_id: None,
                    path: "m/0".into(),
                    operator_name: "private-request-sentinel".into(),
                },
                async |_| Ok::<_, Infallible>(()),
            )
            .await
            .map(completed)
            .unwrap()
            .into_parts()
            .1;
        let expected =
            OperatorExpectation::parse(organization_id.to_string(), None, "m/0".into()).unwrap();
        TvcProvenance::try_from((client, "https://api.turnkey.com"))
            .unwrap()
            .accept_operator(expected, receipt)
            .await
    }

    #[tokio::test]
    async fn human_failure_shows_the_failed_stage_and_escapes_resume_context() {
        let result = operator_acceptance(json!([])).await;
        let result = result.unwrap_err();
        let expected = format!(
            r#"HOSTED OPERATOR VERIFICATION · production
  Local operation ID: {}
  Activity queried: [warning: escaped control characters] activity-1\n\u{{1b}}[2Jlabel
  Pinned Signer quorum key: {}

FAIL  claim set and caller intent
  Verification stopped at this check.
"#,
            Uuid::nil(),
            hex::encode(result.facts().signer_quorum_key().as_ref())
        );
        let mut shell = TestShell::with_human_formatter();
        let result = present::<_, _, VerifiedOperatorKeys>(
            &mut shell,
            Err(result),
            true,
            Some(ResumedActivity {
                operation_id: Uuid::nil(),
                activity_id: "activity-1\n\u{1b}[2Jlabel",
            }),
        );
        assert!(result.is_err());
        assert_eq!(
            String::from_utf8(shell.into_stderr()).unwrap().trim_end(),
            expected.trim_end()
        );
    }

    #[tokio::test]
    async fn failure_names_the_envelope_that_stopped_verification() {
        let policy = json!({
            "scheme": "SIGNATURE_SCHEME_UNSPECIFIED", "publicKey": "ump-key",
            "proofPayload": r#"{"type":"APP_PROOF_TYPE_POLICY_OUTCOME"}"#, "signature": ""
        });
        let invalid = json!({
            "scheme": "SIGNATURE_SCHEME_UNSPECIFIED", "publicKey": "invalid-key",
            "proofPayload": "invalid JSON", "signature": ""
        });
        let result = operator_acceptance(json!([policy, invalid, policy]))
            .await
            .unwrap_err();
        let output = render(result.facts(), Some(&result), None);
        assert!(output.contains("FAIL  envelope routing (proof envelope 1)"));
        assert_eq!(output.matches("proof envelope").count(), 1);
        assert!(!output.contains("PASS"));
    }

    #[tokio::test]
    async fn verbose_json_is_quiet_and_keeps_the_original_error_classification() {
        let result = operator_acceptance(json!([])).await;
        let mut shell = TestShell::with_json_formatter();
        let error = anyhow::Error::from(present(&mut shell, result, true, None).unwrap_err());
        assert!(shell.into_stderr().is_empty());

        let mut shell = TestShell::with_json_formatter();
        shell.emit(&ErrorMessage::from_error(&error)).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&shell.into_stdout()).unwrap();
        assert_eq!(json["code"], "proof_verification_failed");
    }

    #[derive(Default)]
    struct BrokenWriter;
    impl Write for BrokenWriter {
        fn write(&mut self, _bytes: &[u8]) -> std::io::Result<usize> {
            Err(std::io::ErrorKind::BrokenPipe.into())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn diagnostic_write_failure_preserves_the_original_rejection() {
        let result = operator_acceptance(json!([])).await;
        let stage = result.as_ref().unwrap_err().stage();
        let mut shell: Shell<Vec<u8>, BrokenWriter> = Shell::default();
        let result = present(&mut shell, result, true, None);
        assert_eq!(result.as_ref().unwrap_err().stage(), stage);
        assert_eq!(
            crate::errors::classify(&anyhow::Error::from(result.unwrap_err())).code,
            crate::errors::ErrorCode::ProofVerificationFailed
        );
    }

    #[test]
    fn public_values_are_not_shortened() {
        let key = "04".to_owned() + &"ab".repeat(64);
        let mut text = String::new();
        row(&mut text, "Public key", &key);
        assert_eq!(text, format!("  Public key: {key}\n"));
    }
}
