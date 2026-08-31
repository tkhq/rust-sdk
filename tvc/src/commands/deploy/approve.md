# tvc deploy approve

## Purpose
Cryptographically approve a QOS manifest with an operator's manifest-set key and
(by default) post the approval to the Turnkey API. Run it as one of the manifest-set
operators after reviewing a deployment's manifest, either against a fetched deployment
(`--deploy-id`) or a local manifest file (`--manifest`, offline-capable with `--skip-post`).
Implementation: `tvc/src/commands/deploy/approve.rs` (dispatched at `tvc/src/cli.rs:244`).

## Inputs

| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| manifest file | `-m, --manifest <PATH>` | `TVC_MANIFEST` | — | — (one manifest source required) | no |
| deployment id | `-d, --deploy-id <UUID>` | `TVC_DEPLOY_ID` | — | — | no |
| manifest id | `--manifest-id <UUID>` | `TVC_MANIFEST_ID` | — | derived from the `--deploy-id` fetch | no |
| operator id | `--operator-id <UUID>` | `TVC_OPERATOR_ID` | — | — | no |
| YubiKey serial | `--serial <HEX>` | — | — | — | no |
| operator seed | `--operator-seed <HEX>` | `TVC_OPERATOR_SEED` | — | — | no |
| operator seed file | `--operator-seed-path <PATH>` | `TVC_OPERATOR_SEED_PATH` | — | — | no |
| operator (no selector given) | — | — | `orgs.<active>.operators` registry (all kinds) | sole eligible candidate auto-selected | yes, picker when several |
| YubiKey PIN | — | — (deliberately never) | — (deliberately never) | — | yes, always |
| dry run | `--dry-run` | `TVC_DRY_RUN` | — | false | no |
| skip manifest review | `--dangerous-skip-interactive` | `TVC_DANGEROUS_SKIP_INTERACTIVE` | — | false | no |
| approval output file | `-o, --approval-out <PATH>` | `TVC_APPROVAL_OUT` | — | approval inline in outcome | no |
| skip posting | `--skip-post` | `TVC_SKIP_POST` | — | false | no |
| API auth | — | `TVC_ORG_ID` + `TVC_API_KEY_PUBLIC` + `TVC_API_KEY_PRIVATE` | `orgs.<active>.api_key_path` | — | no |

Mutual exclusion (clap ArgGroups, approve.rs:51-57): `--manifest` xor `--deploy-id`;
exactly one of `--operator-id` / `--serial` / `--operator-seed` / `--operator-seed-path`
(re-checked in `TryFrom<Args>`, approve.rs:684-690).

Resolution-order deviations:
- `--serial` is the only input with no env var (approve.rs:91-97).
- With `--deploy-id`, an explicit `--manifest-id` is silently overridden by the fetched
  deployment's manifest id (`fetched.map(..).or(args.manifest_id)`, approve.rs:430-434).
- No command-specific config-file inputs; the operator registry acts as the candidate
  pool, not a default — `default_operator_kind` deliberately does not participate
  (operator.rs:172-174 "operator defaults never participate in approval signing").

## Interactive behavior
1. Review gate first: unless `--dangerous-skip-interactive`, a non-interactive context
   (`--non-interactive`, `TVC_NON_INTERACTIVE`, JSON mode, or stdin not a TTY) bails
   with `MissingRequiredInput("--dangerous-skip-interactive")` before any I/O
   (approve.rs:143-147). So every CI/JSON approval must take the DANGEROUS full skip.
2. Manifest review (unless skipped): six sequential `confirm_or_bail` prompts, default
   No — schema+DNS, namespace, enclave PCRs, pivot binary, manifest set, share set
   (approve.rs:1000-1017). Declining any bails "operation cancelled by user: approval".
3. Operator picker: when no selector narrows the eligible candidates (configured
   operators whose public key is in the manifest set) to one, interactive mode prompts
   "Select approving operator" (approve.rs:345); non-interactive bails "multiple
   configured operators can approve this manifest; provide one operator selector"
   (approve.rs:340-343).
4. YubiKey PIN: always a masked prompt; non-interactive YubiKey approval is refused by
   design — "the PIN is never read from config or the environment" (approve.rs:406-416).
5. `--dry-run` still runs the review prompts (its purpose) but skips operator
   selection, signing, and posting entirely (approve.rs:175-177, 805-807).

## Outputs
Human mode: narration on stdout ("Fetching deployment …", "✓ Manifest loaded",
MANIFEST APPROVAL section banners, "Posting approval to Turnkey..."), warnings on
stderr for existing invalid approvals ("existing approval from X is …; enclave will
reject…", approve.rs:809-824), then the outcome: approval JSON inline or
"Approval written to: PATH", plus posted IDs and a quorum line when quorum state is
known (only on the `--deploy-id` path; approve.rs:960-991).

JSON mode outcome `reason`s (one NDJSON object): `manifest_approval_posted`
(with `approval`/`writtenTo`, `manifestId`, `operatorId`, `approvalIds`,
`quorumReached: bool|null`), `manifest_approval_generated`,
`manifest_approval_already_posted` (`operatorId`, `approvalId`),
`manifest_approval_dry_run` (approve.rs:481-501, unit tests approve.rs:1465-1545).
Errors: `command_error` / `missing_required_input` per the global taxonomy.

## Side effects
- Reads the manifest file (`--manifest`) or fetches the deployment
  (`get_tvc_deployment`, approve.rs:1213-1293).
- Reads registered local operator key files and the YubiKey registry cache while
  enumerating candidates (approve.rs:242-315).
- Writes `--approval-out` (plain overwrite, util.rs:31-35) after signing but before
  posting — the file exists even when the outcome is `already_posted` or the post fails
  (approve.rs:843-846).
- Turnkey API: `create_tvc_manifest_approvals` activity (approve.rs:954-958); a second
  `get_tvc_deployment` post-check for quorum on the `--deploy-id` path (best-effort:
  failure logs at debug and leaves quorum unknown, approve.rs:960-991); hosted signing
  submits a `sign_raw_payload` activity (operator/hosted.rs:304-348).
- YubiKey: opens the device, re-derives and verifies the key pair against the registry
  cache, then signs — PIN entry plus a touch per operation (yubikey/pair.rs:112-152).
- Never mutates the config, but bare dispatch creates a default
  `~/.config/turnkey/tvc.config.toml` when none exists (cli.rs:219-223).

## Failure modes
- Missing review escape hatch in non-interactive mode → `missing_required_input`, exit 1
  (approve.rs:145-147).
- `--manifest` without `--manifest-id` when posting → `ApproveInputError::MissingManifestId`
  ("--manifest-id is required to post approval to API…"), `command_error`, exit 1;
  checked early (approve.rs:149-152) and again at post-target build (approve.rs:430-434).
- Unknown `--serial` → "no YubiKey operator has serial …" before manifest I/O
  (approve.rs:155-163; integration test deploy_approve.rs:372-393) — skipped under `--dry-run`.
- No eligible candidate → selector-specific bail (operator id not in set / serial key not
  in set / explicit key not in set / "no configured operator public key belongs to this
  manifest set") (approve.rs:318-335).
- Hosted operator + `--skip-post` → "--skip-post is not supported for hosted operators",
  refused at resolution before credential loading (operator.rs:359-361).
- Declined review prompt → "operation cancelled by user: approval", `command_error`.
- Duplicate approval (same operator id or member key already posted) → success
  outcome `manifest_approval_already_posted`, exit 0 (approve.rs:870-879).
- Manifest set with an unparseable member key → hard error at `ValidatedManifest`
  construction (approvals.rs:242-271).
- Post/fetch HTTP failures classify via the global taxonomy (`unauthorized`,
  `not_found`, `api_error`, `network_error`); clap conflicts exit 2.

## Gaps

1. **[capability] `--operator-id` cannot select an operator when the manifest comes from a file — it only filters via a fetched deployment.**
   `requested_approval_key` is derived only when `fetched` exists (approve.rs:191-210), and the
   candidate filter (approve.rs:301-314) never compares `requested_operator_id` against the
   locally-known IDs on hosted records (`hosted.operator_id()`) or local records
   (`configured_operator_id`). With `--manifest` + `--operator-id` and two eligible configured
   operators, non-interactive runs bail "provide one operator selector" (approve.rs:340-343)
   even though one was provided, and interactive runs re-prompt; the ID only acts as a
   post-selection assertion (approve.rs:396-402, operator.rs:341-350).

2. **[bug?] The operator UUID needed for posting is validated only after signing.**
   On the `--manifest` file path with no `--operator-id`, a YubiKey / explicit-seed /
   ID-less-local selection carries `post_operator_id: None`; the command prompts for the PIN,
   signs (device touch, or a hosted `sign_raw_payload` activity), writes `--approval-out`, and
   only then fails "resolved operator ID required to post approval" (approve.rs:839-852). The
   condition is fully determinable before signing, next to the existing early
   `MissingManifestId` check (approve.rs:149-152).

3. **[bug?] The duplicate-approval short-circuit also runs after signing.**
   The `existing` match uses `operator_id` and the signer public key (approve.rs:870-879), both
   known before `approve_manifest` (`candidate.public_key`, approve.rs:347); a duplicate still
   costs a PIN + YubiKey touch or a billed hosted signing activity, then discards the signature.

4. **[consistency] `--skip-post` (`SignerRequirement::OfflineApproval`) does not filter candidate enumeration.**
   The requirement is computed and enforced only after selection (approve.rs:826-836,
   operator.rs:359-361), so hosted operators remain candidates: the interactive picker offers an
   operator guaranteed to fail, and a non-interactive run with hosted+local candidates bails
   "multiple configured operators…" even though only one can approve offline.

5. **[consistency] The ambiguous-operator bail classifies as `command_error`, unlike the sibling.**
   `keys re-encrypt-local-share` wraps the same "several operators, give me a selector"
   condition in `MissingRequiredInput::new("--serial")` (re_encrypt_local_share.rs:155-158) so
   JSON consumers get `missing_required_input`; approve uses a plain `bail!`
   (approve.rs:340-343).

6. **[consistency] `--manifest-id` is silently ignored when `--deploy-id` is the manifest source.**
   `fetched.map(|f| f.manifest_id).or(args.manifest_id)` (approve.rs:430-434) lets fetched state
   beat an explicit flag with no conflict error — against the documented flag-first resolution
   order (cli.rs:19-23). `--manifest-id` conflicts with nothing in the ArgGroups, so the
   combination parses fine.

7. **[consistency] `--serial` has no env var while every other input on the command has one** (approve.rs:91-97).
   Same omission on `keys re-encrypt-local-share --serial` (re_encrypt_local_share.rs:59-62), so
   it is systemic rather than local, but `TVC_OPERATOR_ID`/`TVC_OPERATOR_SEED` siblings in the
   same selector group all have env forms.

8. **[capability] Non-interactive approval forces the DANGEROUS full review skip.**
   The only escape hatch from the six review prompts is `--dangerous-skip-interactive`
   (approve.rs:143-147); there is no CI-safe middle ground such as pinning an expected manifest
   hash reviewed out-of-band, so every JSON/CI approval must approve blind.

9. **[consistency] Invalid existing-approval warnings vanish in JSON mode.**
   The "enclave will reject this approval and fail to start" warning goes through
   `human().warn` (approve.rs:815-824), a no-op under `--message-format json`
   (output.rs:146-152), so machine consumers never learn the deployment carries a
   boot-blocking approval; the sibling status flow serializes verdicts structurally
   (approvals.rs:149-179).

10. **[docs] The dead defensive YubiKey-registry bail carries remediation that contradicts the load-time invariant.**
    Config load already rejects any org operator referencing an unregistered serial
    (config/turnkey/mod.rs:122-146), so the in-loop bail (approve.rs:281-289) is unreachable via
    normal dispatch, and its "install its certificates and run `tvc keys refresh-yubikey`"
    advice differs from the load-time message ("edit tvc.config.toml…"). Also `long_about = None`
    (approve.rs:50): the help nowhere states that `--manifest-id` is ignored with `--deploy-id`
    or that YubiKey approval is interactive-only.
