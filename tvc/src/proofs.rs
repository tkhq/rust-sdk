use crate::{output::Shell, terminal::TerminalText};
use chrono::{DateTime, SecondsFormat, Utc};
use qos_nsm::nitro::LIVE_MANIFEST_COMMITMENT_PCR_INDEX;
use std::{fmt::Write as _, io::Write};
use turnkey_proofs::tvc::{
    AcceptanceEvidence, AttestationEvidence, BootEvidence, Check, CheckState, ClaimEvidence,
    ProofCheck, ProofReport, TvcProvenanceError,
};

pub(crate) fn present<W: Write, E: Write, T>(
    shell: &mut Shell<W, E>,
    result: &std::result::Result<T, TvcProvenanceError>,
    report: &ProofReport,
    verbose: bool,
) {
    if verbose {
        let _ = shell.human().err_line(render(report));
    }
    if result.is_ok() {
        let _ = shell.human().status("Proof verification", "passed");
        if verbose {
            let _ = shell
                .human()
                .err_line("________________________________________________________________\n");
        }
    }
}

fn section(s: &mut String, title: &str) {
    let _ = writeln!(s, "{title}");
    let _ = writeln!(s, "─────────────────────────────────────");
}
fn row(s: &mut String, name: &str, value: impl std::fmt::Display) {
    let _ = writeln!(s, "  {name}: {}", TerminalText(value).single_line());
}
fn checks(s: &mut String, checks: &[Check]) {
    for check in checks {
        let label = match check.kind {
            ProofCheck::ActivityShape => "Activity type, status and result shape",
            ProofCheck::EnvelopeCount => "Envelope count bound",
            ProofCheck::EnvelopeRouting => "Envelope size and routing",
            ProofCheck::EphemeralKeyFormat => "Composite ephemeral key format",
            ProofCheck::BootProofLookup => "Boot proof lookup",
            ProofCheck::DocumentEndorsement => "Pinned Signer document signature",
            ProofCheck::AwsAttestationAtIssuance => "AWS syntax, chain and COSE at issuance time",
            ProofCheck::BootBinding => {
                "Manifest approvals, commitments, EK equality and live PCR binding"
            }
            ProofCheck::ApplicationSignature => "Ephemeral application signature",
            ProofCheck::ClaimDecode => "Strict signed claim and request-context decoding",
            ProofCheck::ClaimSetAndIntent => "Required claim set and caller intent",
            ProofCheck::CommonSubmission => {
                "One common fingerprint from the saved creation request"
            }
            ProofCheck::OutputKeyRoles => "Output key rules",
            ProofCheck::ResultBinding => "Signed claims equal returned result",
            ProofCheck::ShareRoutingShape => "Unsigned share routing label count and shape",
        };
        let state = match check.state {
            CheckState::Pass => "PASS",
            CheckState::Fail => "FAIL",
            CheckState::NotRun => "NOT RUN",
        };
        let _ = writeln!(s, "  [{state}] {label}");
    }
}

fn successful_boot<'a>(
    checks: &[Check],
    attestation: Option<&'a AttestationEvidence>,
    boot: Option<&'a BootEvidence>,
) -> Option<(&'a AttestationEvidence, &'a BootEvidence)> {
    [
        ProofCheck::DocumentEndorsement,
        ProofCheck::AwsAttestationAtIssuance,
        ProofCheck::BootBinding,
    ]
    .iter()
    .all(|kind| {
        checks
            .iter()
            .any(|check| check.kind == *kind && check.state == CheckState::Pass)
    })
    .then_some(())?;
    Some((attestation?, boot?))
}

fn render_boot_evidence<'a>(
    s: &mut String,
    checks: &[Check],
    attestation: Option<&AttestationEvidence>,
    boot: Option<&BootEvidence>,
    mut prior_boots: impl Iterator<Item = (usize, (&'a AttestationEvidence, &'a BootEvidence))>,
) {
    if let Some(current) = successful_boot(checks, attestation, boot)
        && let Some((first, _)) = prior_boots.find(|(_, prior)| *prior == current)
    {
        row(
            s,
            "Verified boot evidence",
            format!("Identical to PROOF ENVELOPE {first}"),
        );
        return;
    }
    if let Some(attestation) = attestation {
        let utc = i64::try_from(attestation.issued_at_ms)
            .ok()
            .and_then(DateTime::<Utc>::from_timestamp_millis)
            .map(|time| time.to_rfc3339_opts(SecondsFormat::Millis, true))
            .unwrap_or_else(|| "UTC time out of range".into());
        row(
            s,
            "AWS-authenticated attestation timestamp",
            format!("{utc} (Unix ms: {})", attestation.issued_at_ms),
        );
        let utc = i64::try_from(attestation.issued_at_ms / 1000)
            .ok()
            .and_then(|seconds| DateTime::<Utc>::from_timestamp(seconds, 0))
            .map(|time| time.to_rfc3339_opts(SecondsFormat::Secs, true))
            .unwrap_or_else(|| "UTC time out of range".into());
        row(
            s,
            "Certificate validation time",
            format!("{utc} (Unix seconds: {})", attestation.issued_at_ms / 1000),
        );
        row(
            s,
            "AWS-authenticated ephemeral public key",
            attestation.public_key.as_deref().unwrap_or("absent"),
        );
        row(
            s,
            "AWS-authenticated manifest commitment",
            attestation
                .manifest_commitment
                .as_deref()
                .unwrap_or("absent"),
        );
        for (index, pcr) in &attestation.pcrs {
            if *index < 4 || *index == usize::from(LIVE_MANIFEST_COMMITMENT_PCR_INDEX) {
                row(s, &format!("AWS-authenticated PCR{index}"), pcr);
            }
        }
    } else {
        row(s, "AWS-authenticated attestation", "unavailable");
    }
    if let Some(boot) = boot {
        row(s, "Supplied manifest hash", &boot.manifest_hash);
        row(
            s,
            "Supplied envelope manifest hash",
            &boot.envelope_manifest_hash,
        );
        row(
            s,
            "Supplied manifest namespace quorum key",
            if boot.manifest_quorum_key.is_empty() {
                "absent (permitted by policy)"
            } else {
                &boot.manifest_quorum_key
            },
        );
        row(s, "Supplied boot ephemeral public key", &boot.ephemeral_key);
        for (i, pcr) in boot.pcrs.iter().enumerate() {
            row(s, &format!("Supplied manifest PCR{i}"), pcr);
        }
    } else {
        row(s, "Manifest operands", "unavailable");
    }
}

fn render(report: &ProofReport) -> String {
    let mut s = String::new();
    section(&mut s, "PROOF VERIFICATION POLICY");
    row(&mut s, "Selected API environment", report.environment());
    row(
        &mut s,
        "Independently selected Signer quorum key",
        report.signer_quorum_key(),
    );
    row(
        &mut s,
        "AWS trust anchor",
        "Bundled AWS Nitro root certificate",
    );
    row(
        &mut s,
        "Signer endorsement message",
        format!(
            "{} || raw attestation document",
            String::from_utf8_lossy(turnkey_proofs::tvc::ENDORSEMENT_PREFIX).replace('\0', r"\0")
        ),
    );
    let _ = writeln!(s);
    section(&mut s, "REQUEST AND RESULT BINDING");
    checks(&mut s, report.checks());
    match report.acceptance() {
        AcceptanceEvidence::Operator {
            organization,
            wallet,
            base_path,
            returned,
        } => {
            row(&mut s, "Expected organization", organization);
            row(
                &mut s,
                "Expected wallet",
                wallet
                    .as_deref()
                    .unwrap_or("new wallet; both signed roles must agree"),
            );
            row(&mut s, "Expected base path (roles /0 and /1)", base_path);
            if let Some(returned) = returned {
                row(&mut s, "Returned wallet", &returned.wallet);
                row(&mut s, "Returned encryption public key", &returned.encrypt);
                row(&mut s, "Returned signing public key", &returned.sign);
                row(
                    &mut s,
                    "Unsigned operator routing label",
                    &returned.operator_id,
                );
            } else {
                row(&mut s, "Returned values", "unavailable");
            }
        }
        AcceptanceEvidence::Quorum {
            organization,
            threshold,
            recipients,
            returned,
        } => {
            row(&mut s, "Expected organization", organization);
            row(&mut s, "Expected threshold", threshold);
            for (i, key) in recipients.iter().enumerate() {
                row(&mut s, &format!("Expected recipient [{i}]"), key);
            }
            if let Some(returned) = returned {
                row(&mut s, "Returned quorum key ID", &returned.id);
                row(&mut s, "Returned quorum public key", &returned.public_key);
                for (i, id) in returned.share_ids.iter().enumerate() {
                    row(&mut s, &format!("Unsigned share routing label [{i}]"), id);
                }
            } else {
                row(&mut s, "Returned values", "unavailable");
            }
        }
    }
    for digest in report.sent_fingerprints() {
        row(&mut s, "Saved creation request fingerprint", digest);
    }
    row(
        &mut s,
        "Common matched fingerprint",
        report.matched_fingerprint().unwrap_or("unavailable"),
    );
    let _ = writeln!(s);
    for (i, envelope) in report.envelopes().iter().enumerate() {
        if envelope.policy_excluded() {
            let _ = writeln!(
                s,
                "PROOF ENVELOPE {i}: Policy outcome excluded; cryptographic checks intentionally NOT RUN\n"
            );
            continue;
        }
        section(&mut s, &format!("PROOF ENVELOPE {i}"));
        checks(&mut s, envelope.checks());
        row(
            &mut s,
            "Supplied app composite ephemeral public key",
            envelope.ephemeral_key().unwrap_or("unavailable"),
        );
        render_boot_evidence(
            &mut s,
            envelope.checks(),
            envelope.attestation(),
            envelope.boot(),
            report.envelopes()[..i]
                .iter()
                .enumerate()
                .filter_map(|(index, prior)| {
                    successful_boot(prior.checks(), prior.attestation(), prior.boot())
                        .map(|boot| (index, boot))
                }),
        );
        match envelope.claim() {
            Some(ClaimEvidence::Operator {
                organization,
                wallet,
                path,
                public_key,
                digests,
            }) => {
                row(&mut s, "Signed organization", organization);
                row(&mut s, "Signed wallet", wallet);
                row(&mut s, "Signed role path", path);
                row(&mut s, "Signed role public key", public_key);
                for digest in digests {
                    row(&mut s, "Signed request-context digest", digest);
                }
            }
            Some(ClaimEvidence::Quorum {
                organization,
                id,
                public_key,
                threshold,
                recipients,
                digests,
            }) => {
                row(&mut s, "Signed organization", organization);
                row(&mut s, "Signed quorum key ID", id);
                row(&mut s, "Signed quorum public key", public_key);
                row(&mut s, "Signed threshold", threshold);
                for (i, key) in recipients.iter().enumerate() {
                    row(&mut s, &format!("Signed recipient [{i}]"), key);
                }
                for digest in digests {
                    row(&mut s, "Signed request-context digest", digest);
                }
            }
            None => row(&mut s, "Authenticated signed claim", "unavailable"),
        }
        let _ = writeln!(s);
    }
    section(&mut s, "VERIFICATION SCOPE");
    let _ = writeln!(s, "  Supplied values are operands, authenticated only by the checks marked PASS.
  Boot binding is one combined check; FAIL does not identify a particular PCR.
  Manifest approvals use the supplied envelope policy; no independent manifest or PCR allowlist.
  Namespace quorum key is compared with the selected Signer key only when present.
  AWS certificate validity is checked at attestation issuance; no current-time freshness or revocation policy.
  Operator IDs and share IDs are unsigned routing labels. Quorum key ID is covered by the creation claim; acceptance requires it to match the result.
  Policy outcome envelopes are excluded from creation acceptance.");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::{ErrorMessage, TestShell};
    use serde_json::json;
    use turnkey_client::{TurnkeyClient, TurnkeyP256ApiKey, generated::CreateTvcOperatorIntent};
    use turnkey_proofs::tvc::{OperatorExpectation, TvcProvenance};
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

    fn completed<T>(outcome: turnkey_client::SubmissionOutcome<T>) -> T {
        match outcome {
            turnkey_client::SubmissionOutcome::Completed(value) => value,
            turnkey_client::SubmissionOutcome::Pending { .. } => {
                panic!("expected completed proof fixture")
            }
        }
    }

    async fn proof_report(
        app_proofs: serde_json::Value,
    ) -> (
        std::result::Result<turnkey_proofs::tvc::VerifiedOperatorKeys, TvcProvenanceError>,
        ProofReport,
    ) {
        let server = MockServer::start().await;
        Mock::given(method("POST")).respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "activity": {
                "id": "activity-1", "organizationId": "org-id", "type": "ACTIVITY_TYPE_CREATE_TVC_OPERATOR",
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
                "org-id".into(),
                100,
                CreateTvcOperatorIntent {
                    wallet_name: Some("wallet".into()),
                    wallet_id: None,
                    path: "m/0".into(),
                    operator_name: "private-request-sentinel".into(),
                },
                async |_| Ok(()),
            )
            .await
            .map(completed)
            .unwrap()
            .into_parts()
            .1;
        let expected = OperatorExpectation::parse("org-id".into(), None, "m/0".into()).unwrap();
        TvcProvenance::for_api_url("https://api.turnkey.com")
            .unwrap()
            .accept_operator_reported(&client, &expected, receipt)
            .await
            .into_parts()
    }

    #[tokio::test]
    async fn human_failure_has_truthful_states_full_values_and_terminal_safe_rows() {
        let (result, report) = proof_report(json!([])).await;
        assert!(matches!(result, Err(TvcProvenanceError::ProofUnavailable)));
        let mut shell = TestShell::with_human_formatter();
        present(&mut shell, &result, &report, true);
        let output = String::from_utf8(shell.into_stderr()).unwrap();
        assert!(output.contains("[FAIL] Required claim set and caller intent"));
        assert!(output.contains("[NOT RUN] Signed claims equal returned result"));
        assert!(output.contains(report.signer_quorum_key()));
        assert!(output.contains("public-output-key"));
        assert!(output.contains(r"unsigned\n\u{1b}[2Jlabel"));
        assert!(!output.contains('\u{1b}'));
        assert!(!output.contains("Proof verification: passed"));
        assert!(
            !output.contains("________________________________________________________________")
        );
        assert!(!output.contains("private-request-sentinel"));
        assert!(!output.contains("untrusted-fingerprint"));
        assert!(output.lines().any(|line| line == r"  Signer endorsement message: turnkey.boot-proof.attestation.v1\0 || raw attestation document"));
    }

    #[tokio::test]
    async fn only_intentionally_excluded_policy_envelopes_collapse() {
        let policy = json!({
            "scheme": "SIGNATURE_SCHEME_UNSPECIFIED", "publicKey": "ump-key",
            "proofPayload": r#"{"type":"APP_PROOF_TYPE_POLICY_OUTCOME"}"#, "signature": ""
        });
        let invalid = json!({
            "scheme": "SIGNATURE_SCHEME_UNSPECIFIED", "publicKey": "invalid-key",
            "proofPayload": "invalid JSON", "signature": ""
        });
        let (result, report) = proof_report(json!([policy, invalid, policy])).await;
        assert!(result.is_err());
        let output = render(&report);
        let excluded = output
            .split("PROOF ENVELOPE 0")
            .nth(1)
            .unwrap()
            .split("PROOF ENVELOPE 1")
            .next()
            .unwrap();
        assert_eq!(
            excluded,
            ": Policy outcome excluded; cryptographic checks intentionally NOT RUN\n\n"
        );
        let invalid = output
            .split("PROOF ENVELOPE 1\n")
            .nth(1)
            .unwrap()
            .split("PROOF ENVELOPE 2\n")
            .next()
            .unwrap();
        assert!(invalid.contains("  [FAIL] Envelope size and routing\n"));
        assert!(invalid.contains("  AWS-authenticated attestation: unavailable\n"));
        let unvisited = output
            .split("PROOF ENVELOPE 2\n")
            .nth(1)
            .unwrap()
            .split("VERIFICATION SCOPE")
            .next()
            .unwrap();
        assert!(unvisited.contains("  [NOT RUN] Envelope size and routing\n"));
        assert!(unvisited.contains("  Manifest operands: unavailable\n"));
    }

    fn boot_evidence() -> (AttestationEvidence, BootEvidence, Vec<Check>) {
        (
            AttestationEvidence {
                issued_at_ms: 1_000_123,
                public_key: Some("04".to_owned() + &"ab".repeat(129)),
                manifest_commitment: Some("commitment".into()),
                pcrs: (0..32).map(|i| (i, format!("pcr-{i}"))).collect(),
            },
            BootEvidence {
                manifest_hash: "manifest".into(),
                envelope_manifest_hash: "envelope-manifest".into(),
                manifest_quorum_key: "04".to_owned() + &"cd".repeat(129),
                ephemeral_key: "04".to_owned() + &"ab".repeat(129),
                pcrs: std::array::from_fn(|i| format!("pcr-{i}")),
            },
            [
                ProofCheck::DocumentEndorsement,
                ProofCheck::AwsAttestationAtIssuance,
                ProofCheck::BootBinding,
            ]
            .into_iter()
            .map(|kind| Check {
                kind,
                state: CheckState::Pass,
            })
            .collect(),
        )
    }

    #[test]
    fn boot_rows_show_full_keys_checked_pcrs_and_readable_numeric_times() {
        let (attestation, boot, checks) = boot_evidence();
        let mut output = String::new();
        render_boot_evidence(
            &mut output,
            &checks,
            Some(&attestation),
            Some(&boot),
            std::iter::empty(),
        );
        assert_eq!(
            output,
            format!(
                r#"  AWS-authenticated attestation timestamp: 1970-01-01T00:16:40.123Z (Unix ms: 1000123)
  Certificate validation time: 1970-01-01T00:16:40Z (Unix seconds: 1000)
  AWS-authenticated ephemeral public key: {key}
  AWS-authenticated manifest commitment: commitment
  AWS-authenticated PCR0: pcr-0
  AWS-authenticated PCR1: pcr-1
  AWS-authenticated PCR2: pcr-2
  AWS-authenticated PCR3: pcr-3
  AWS-authenticated PCR17: pcr-17
  Supplied manifest hash: manifest
  Supplied envelope manifest hash: envelope-manifest
  Supplied manifest namespace quorum key: {quorum_key}
  Supplied boot ephemeral public key: {key}
  Supplied manifest PCR0: pcr-0
  Supplied manifest PCR1: pcr-1
  Supplied manifest PCR2: pcr-2
  Supplied manifest PCR3: pcr-3
"#,
                key = attestation.public_key.as_deref().unwrap(),
                quorum_key = boot.manifest_quorum_key
            )
        );
    }

    #[test]
    fn only_identical_successful_boot_metadata_is_referenced() {
        let (attestation, boot, checks) = boot_evidence();
        let mut output = String::new();
        render_boot_evidence(
            &mut output,
            &checks,
            Some(&attestation),
            Some(&boot),
            successful_boot(&checks, Some(&attestation), Some(&boot))
                .into_iter()
                .map(|boot| (2, boot)),
        );
        assert_eq!(
            output,
            "  Verified boot evidence: Identical to PROOF ENVELOPE 2\n"
        );

        for mutation in 0..10 {
            let (mut changed_attestation, mut changed_boot, _) = boot_evidence();
            match mutation {
                0 => changed_attestation.issued_at_ms += 1,
                1 => changed_attestation.public_key = Some("different-key".into()),
                2 => changed_attestation.manifest_commitment = Some("different-commitment".into()),
                3 => changed_attestation.pcrs[0].1 = "different-pcr".into(),
                4 => changed_attestation.pcrs[31].1 = "different-unshown-pcr".into(),
                5 => changed_boot.manifest_hash = "different-manifest".into(),
                6 => changed_boot.envelope_manifest_hash = "different-envelope".into(),
                7 => changed_boot.manifest_quorum_key = "different-quorum-key".into(),
                8 => changed_boot.ephemeral_key = "different-boot-key".into(),
                9 => changed_boot.pcrs[0] = "different-manifest-pcr".into(),
                _ => panic!("unknown mutation"),
            }
            output.clear();
            render_boot_evidence(
                &mut output,
                &checks,
                Some(&changed_attestation),
                Some(&changed_boot),
                successful_boot(&checks, Some(&attestation), Some(&boot))
                    .into_iter()
                    .map(|boot| (2, boot)),
            );
            assert!(
                output.starts_with("  AWS-authenticated attestation timestamp:"),
                "mutation {mutation}: {output}"
            );
            assert!(!output.contains("Identical to"));
        }
    }

    #[test]
    fn failed_and_unreached_boot_checks_never_reuse_a_prior_pass() {
        let (attestation, boot, mut checks) = boot_evidence();
        for index in 0..checks.len() {
            for state in [CheckState::Fail, CheckState::NotRun] {
                checks[index].state = state;
                let mut output = String::new();
                render_boot_evidence(
                    &mut output,
                    &checks,
                    Some(&attestation),
                    Some(&boot),
                    std::iter::once((0, (&attestation, &boot))),
                );
                assert!(output.starts_with("  AWS-authenticated attestation timestamp:"));
                assert!(!output.contains("Identical to"));
                checks[index].state = CheckState::Pass;
            }
        }
        let mut output = String::new();
        checks[2].state = CheckState::Fail;
        let prior = successful_boot(&checks, Some(&attestation), Some(&boot));
        checks[2].state = CheckState::Pass;
        render_boot_evidence(
            &mut output,
            &checks,
            Some(&attestation),
            Some(&boot),
            prior.into_iter().map(|boot| (0, boot)),
        );
        assert!(output.starts_with("  AWS-authenticated attestation timestamp:"));
    }

    #[test]
    fn out_of_range_timestamps_keep_the_original_numbers() {
        let (mut attestation, boot, checks) = boot_evidence();
        attestation.issued_at_ms = u64::MAX;
        let mut output = String::new();
        render_boot_evidence(
            &mut output,
            &checks,
            Some(&attestation),
            Some(&boot),
            std::iter::empty(),
        );
        assert_eq!(
            output.lines().take(2).collect::<Vec<_>>(),
            [
                "  AWS-authenticated attestation timestamp: UTC time out of range (Unix ms: 18446744073709551615)",
                "  Certificate validation time: UTC time out of range (Unix seconds: 18446744073709551)",
            ]
        );
    }

    #[tokio::test]
    async fn verbose_json_is_quiet_and_keeps_the_original_error_classification() {
        let (result, report) = proof_report(json!([])).await;
        let mut shell = TestShell::with_json_formatter();
        present(&mut shell, &result, &report, true);
        let error = anyhow::Error::from(result.unwrap_err());
        shell.emit(&ErrorMessage::from_error(&error)).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&shell.into_stdout()).unwrap();
        assert_eq!(json["code"], "proof_verification_failed");
        let (result, report) = proof_report(json!([])).await;
        let mut shell = TestShell::with_json_formatter();
        present(&mut shell, &result, &report, true);
        assert!(shell.into_stderr().is_empty());
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
    async fn diagnostic_write_failure_preserves_proof_failure_and_success_line_is_exact() {
        let (result, report) = proof_report(json!([])).await;
        let mut shell: Shell<Vec<u8>, BrokenWriter> = Shell::default();
        present(&mut shell, &result, &report, true);
        assert!(matches!(result, Err(TvcProvenanceError::ProofUnavailable)));
        let accepted_material =
            Ok::<_, TvcProvenanceError>("verified material continues to persistence");
        present(&mut shell, &accepted_material, &report, true);
        assert!(matches!(
            accepted_material,
            Ok("verified material continues to persistence")
        ));
        let mut shell = TestShell::with_human_formatter();
        present(&mut shell, &Ok::<_, TvcProvenanceError>(()), &report, false);
        assert_eq!(shell.into_stderr(), b"Proof verification: passed\n");

        let mut shell = TestShell::with_human_formatter();
        present(&mut shell, &accepted_material, &report, true);
        let output = String::from_utf8(shell.into_stderr()).unwrap();
        assert_eq!(
            output.strip_prefix(&format!("{}\n", render(&report))),
            Some(
                r#"Proof verification: passed
________________________________________________________________

"#
            )
        );

        let mut shell = TestShell::with_json_formatter();
        present(&mut shell, &accepted_material, &report, true);
        assert!(shell.into_stderr().is_empty());
        let mut shell = TestShell::with_json_formatter();
        present(&mut shell, &accepted_material, &report, true);
        assert!(shell.into_stdout().is_empty());
    }

    #[test]
    fn public_values_are_not_shortened() {
        let key = "04".to_owned() + &"ab".repeat(64);
        let mut text = String::new();
        row(&mut text, "Public key", &key);
        assert_eq!(text, format!("  Public key: {key}\n"));
    }
}
