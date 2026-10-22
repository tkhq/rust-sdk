# tvc deploy approve

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the deploy approve command.*

## Purpose (informative)

The command signs a QOS manifest with an operator's manifest-set key.
By default it posts the approval to the Turnkey API.
A manifest-set operator runs it after a review of a deployment's manifest.
The manifest comes from a fetched deployment (`--deploy-id`) or a local file (`--manifest`).
With `--manifest`, `--skip-post`, and a local key the command works offline.
Implementation: tvc/src/commands/deploy/approve.rs, dispatched at tvc/src/cli.rs:244.

## Acceptance scenario (normative)

Run the scenario from the `tvc/` crate directory, in a TTY, in human mode.

| Step | Action | Expected observation |
|---|---|---|
| 1 | Point `HOME` at an empty directory. | No tvc config exists. |
| 2 | Run `tvc deploy approve --manifest fixtures/manifest.json --operator-seed-path fixtures/seed.hex --skip-post --approval-out /tmp/approval.json`. | stdout prints the `MANIFEST APPROVAL` banner, then the `MANIFEST SCHEMA` section with `Version:       v1 (legacy)` (approve.rs:1001-1005, 1042). |
| 3 | Answer `y` to `Approve manifest schema and DNS?`. | The `NAMESPACE` section prints with `Name:       turnkey-prod` (approve.rs:1062-1076). |
| 4 | Answer `y` to `Approve namespace?`, `Approve enclave configuration?`, `Approve pivot binary?`, `Approve manifest set?` (section shows `Threshold: 2 of 3`), and `Approve share set?`. | The `ALL SECTIONS APPROVED` banner prints (approve.rs:1012-1014). |
| 5 | Let the command exit. | stdout ends with `Approval written to: /tmp/approval.json`; exit code 0; a default tvc config now exists (cli.rs:219-222). |
| 6 | Read `/tmp/approval.json`. | One JSON object with a hex `signature` and a `member` whose `alias` and `pubKey` match one fixture manifest set member (approve.rs:843-846). |

All steps pass in one run from a clean start.

## Inputs (normative)

Flag and env resolution MUST follow the Part 00 value resolution order.
The command consumes no command config.

| Input | Flag | Env | Other source | Default | Prompted |
|---|---|---|---|---|---|
| manifest file | `-m, --manifest <PATH>` | `TVC_MANIFEST` | none | none (one manifest source required) | no |
| deployment id | `-d, --deploy-id <UUID>` | `TVC_DEPLOY_ID` | none | none | no |
| manifest id | `--manifest-id <UUID>` | `TVC_MANIFEST_ID` | derived from the `--deploy-id` fetch (approve.rs:430-434) | none | no |
| operator id | `--operator-id <UUID>` | `TVC_OPERATOR_ID` | none | none | no |
| YubiKey serial | `--serial <SERIAL>` | none (approve.rs:91-97) | none | none | no |
| operator seed | `--operator-seed <HEX_SEED>` | `TVC_OPERATOR_SEED` | none | none | no |
| operator seed file | `--operator-seed-path <PATH>` | `TVC_OPERATOR_SEED_PATH` | none | none | no |
| operator (no selector given) | none | none | operator records of the active org, all operator kinds (approve.rs:242-315) | sole eligible candidate auto-selected (approve.rs:336-339) | picker when several |
| YubiKey PIN | none | never (approve.rs:406-411) | never | none | always (approve.rs:414-416) |
| dry run | `--dry-run` | `TVC_DRY_RUN` | none | false | no |
| review skip | `--dangerous-skip-interactive` | `TVC_DANGEROUS_SKIP_INTERACTIVE` | none | false | no |
| approval output file | `-o, --approval-out <PATH>` | `TVC_APPROVAL_OUT` | none | approval inline in the outcome | no |
| skip posting | `--skip-post` | `TVC_SKIP_POST` | none | false | no |
| API auth | none | `TVC_ORG_ID` + `TVC_API_KEY_PUBLIC` + `TVC_API_KEY_PRIVATE` (client.rs:20-23) | active org `api_key_path` | none | no |

Clap enforces two argument groups (approve.rs:51-57).
`--manifest` and `--deploy-id` MUST NOT combine.
The four operator selectors (`--operator-id`, `--serial`, `--operator-seed`, `--operator-seed-path`) MUST NOT combine.
`TryFrom<Args>` re-checks selector exclusivity for env-supplied values (approve.rs:684-690).

**eligible candidate**: an operator record of the active org whose public key belongs to the manifest set under approval. The selector, when given, further filters the candidates (approve.rs:301-314).

Three resolution rules deviate from the common shape.

- `--serial` has no env var (approve.rs:91-97). Gap 7 records this.
- When `--deploy-id` supplies the manifest, the command MUST use the fetched manifest id and MUST ignore an explicit `--manifest-id` (approve.rs:430-434). Gap 6 proposes a conflict error.
- The default operator kind MUST NOT participate in operator selection (operator.rs:172-174). The operator records of the active org form the candidate pool.

## Interactive behavior (normative)

1. Review gate. The gate applies when `--dangerous-skip-interactive` is absent and the run cannot prompt (non-interactive mode, or stdin is no TTY). The command MUST then fail with `MissingRequiredInput("--dangerous-skip-interactive")` before any file or network I/O (approve.rs:145-147). Every CI or JSON mode approval therefore takes the full review skip (Gap 8).
2. Manifest review. Unless skipped, the command MUST run six confirm prompts in this order: schema and DNS, namespace, enclave PCRs, pivot binary, manifest set, share set. Each prompt defaults to No (approve.rs:1005-1010). A decline MUST fail with `operation cancelled by user: approval` (prompts.rs:70-75).
3. Operator picker. When several eligible candidates remain, interactive mode MUST prompt `Select approving operator` (approve.rs:345). Non-interactive mode MUST fail with `multiple configured operators can approve this manifest; provide one operator selector` (approve.rs:340-343).
4. YubiKey PIN. A YubiKey candidate MUST prompt for the PIV PIN with masked input (approve.rs:414-416). When prompts are unavailable the command MUST fail: the PIN never comes from the tvc config or the env (approve.rs:406-411).
5. Dry run. With `--dry-run` the command MUST run the review prompts and MUST skip operator selection, signing, and posting (approve.rs:175-177, 805-807).

## Outputs (normative)

Human mode narration goes to stdout.
The `--deploy-id` path prints `Fetching deployment <id>...` and `✓ Manifest loaded (manifest_id: <id>)` (approve.rs:1218, 1283).
Both paths print the manifest review sections and, when posting, `Posting approval to Turnkey...` (approve.rs:934-935).
For each existing approval with a verdict other than valid, the command MUST warn on stderr: `warning: existing approval from <name> (<id>) is <verdict>; enclave will reject this approval and fail to start` (approve.rs:809-824, output.rs:146-152).
JSON mode drops these warnings (Gap 9).

The human mode outcome prints the approval JSON inline, or `Approval written to: <PATH>` when `--approval-out` is set (approve.rs:506-513).
A posted outcome adds `Approval posted successfully!`, the approval IDs, the manifest ID, and the operator ID (approve.rs:571-596).
When the post-check fetch reports quorum state, the outcome MUST end with one quorum line (approve.rs:584-592; test approve.rs:1552-1573).
The line reads `Manifest approval quorum reached. Your deployment will be available soon.` on reached quorum and `Your approval has been posted. Deployment requires additional manifest approvals before it can be deployed on TVC.` otherwise (approve.rs:45-46, 588).
Only the `--deploy-id` path with a successful post-check fetch yields quorum state (approve.rs:960-991).

In JSON mode the command MUST emit exactly one NDJSON outcome object on stdout.

| `reason` | Fields | Pinned by |
|---|---|---|
| `manifest_approval_posted` | `approval` or `writtenTo`, `manifestId`, `operatorId`, `approvalIds`, `quorumReached` (bool or null) | approve.rs:1466-1481 |
| `manifest_approval_generated` | `approval` or `writtenTo` | approve.rs:1484-1513 |
| `manifest_approval_already_posted` | `operatorId`, `approvalId` | approve.rs:1516-1534 |
| `manifest_approval_dry_run` | none | approve.rs:1537-1545 |

JSON mode errors MUST follow the Part 00 error taxonomy: `missing_required_input` for the review gate, `command_error` for command bails, and HTTP classifications for API failures.

## Side effects (normative)

- The command MUST read the manifest file (`--manifest`), or fetch the deployment with one `get_tvc_deployment` query (approve.rs:1213-1293).
- Candidate enumeration reads registered local operator key files and the YubiKey registry cache (approve.rs:242-315).
- With `--approval-out` the command MUST write the file as a plain overwrite (util.rs:31-35), after signing and before posting (approve.rs:843-846). The file exists even when the outcome is `manifest_approval_already_posted` or the post fails.
- Posting submits one `create_tvc_manifest_approvals` activity (approve.rs:954-958). On the `--deploy-id` path a post-check `get_tvc_deployment` reads quorum state. A failed post-check MUST log at debug and leave quorum unknown (approve.rs:960-991).
- Hosted signing submits one `sign_raw_payload` activity (operator/hosted.rs:304-348).
- A YubiKey signer opens the device, verifies the derived key pair against the registry cache, and signs. This costs one PIN entry plus a device touch per operation (yubikey/pair.rs:112-152).
- The command MUST NOT write the tvc config. Bare dispatch creates a default tvc config when the file is absent (cli.rs:219-222, INV-G4).

## Failure modes (normative)

Each condition MUST produce the listed observation.

| Condition | Observation | Mechanism |
|---|---|---|
| Non-interactive run without `--dangerous-skip-interactive` | `missing_required_input`, exit 1, before any I/O | approve.rs:145-147 |
| No manifest source | `a manifest source is required`, `command_error`, exit 1 | approve.rs:922; test deploy_approve.rs:207 |
| `--manifest` without `--manifest-id` when posting | `--manifest-id is required to post approval to API (or use --deploy-id). Use --skip-post to only generate the approval locally.`, `command_error`, exit 1, checked early | approve.rs:149-152, 624-630; test deploy_approve.rs:803 |
| Unknown `--serial` | `no YubiKey operator has serial <serial>`, exit 1, before manifest I/O; skipped under `--dry-run` | approve.rs:154-163; test deploy_approve.rs:372 |
| `--operator-id` absent from the fetched manifest set | `operator ID <uuid> is not in the deployment's manifest set`, exit 1 | approve.rs:199-207 |
| No eligible candidate | selector-specific message, for example `no configured operator public key belongs to this manifest set`, exit 1 | approve.rs:318-335; test deploy_approve.rs:488 |
| Several eligible candidates, non-interactive | `multiple configured operators can approve this manifest; provide one operator selector`, `command_error`, exit 1 | approve.rs:340-343; Gap 5 |
| No active org and no explicit seed | `No active organization` message that points to `tvc login`, `--operator-seed`, and `--operator-seed-path`; exit 1 | approve.rs:235-240; test deploy_approve.rs:649 |
| Hosted candidate with `--skip-post` | `--skip-post is not supported for hosted operators`, exit 1, before credential loading | operator.rs:358-361; tests deploy_approve.rs:240, 441 |
| YubiKey candidate without a prompt | `a YubiKey operator needs its PIN typed at an interactive prompt; the PIN is never read from config or the environment`, exit 1, no device access | approve.rs:406-411; test deploy_approve.rs:262 |
| Declined review prompt | `operation cancelled by user: approval`, `command_error`, exit 1 | prompts.rs:70-75 |
| Requested operator id conflicts with the configured local operator id | `requested operator ID (<uuid>) does not match configured local operator ID (<uuid>)`, exit 1 | operator.rs:341-350 |
| Resolved operator lacks a UUID at post time | `resolved operator ID required to post approval`, exit 1, after signing | approve.rs:850-852; Gap 2 |
| Same operator already approved this manifest | outcome `manifest_approval_already_posted`, exit 0 | approve.rs:870-879 |
| Manifest set member key does not parse | `manifest set contains an invalid public key for member <alias>`, exit 1, at `ValidatedManifest` construction | approvals.rs:242-271 |
| Conflicting flags inside one argument group | clap usage error, exit 2 | approve.rs:51-57; tests deploy_approve.rs:726, 764, 781 |
| Post or fetch HTTP failure | Part 00 taxonomy code (`unauthorized`, `not_found`, `api_error`, `network_error`), exit 1 | Part 00 error taxonomy |

## Test vectors (normative)

Run every vector from the `tvc/` crate directory with `HOME` pointed at an empty directory, unless Given states otherwise.
Vector comparison excludes the Part 00 nondeterministic fields plus the approval `signature` bytes.
`SEED` stands for the content of `fixtures/seed.hex` (`55952c7d4e4e39d9b75f948bd833d10bc948e7fb130c1d4d9de7fd1fc424316c`).

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | clean start | `tvc deploy approve --manifest fixtures/manifest.json --operator-seed-path fixtures/seed.hex --dangerous-skip-interactive --skip-post` | Exit 0; stdout carries the approval JSON (`signature`, `member`); no quorum line (test deploy_approve.rs:666; approve.rs:898-901). |
| V-2 | clean start | V-1 invocation plus `--message-format json` | Exit 0; one NDJSON object with `reason` = `manifest_approval_generated` and an `approval` object with hex `signature` and `member` (`alias`, `pubKey`) (approve.rs:898-901; test approve.rs:1484-1513). |
| V-3 | tvc config with one hosted operator record whose key is fixture manifest set member 0 | `tvc deploy approve --manifest fixtures/manifest.json --dry-run --dangerous-skip-interactive` | Exit 0; stdout contains `Dry run complete. No approval generated.` (test deploy_approve.rs:219; approve.rs:611-613, 805-807). |
| V-4 | as V-3 | V-3 invocation plus `--message-format json` | Exit 0; one NDJSON object equal to `{"reason":"manifest_approval_dry_run"}` (test approve.rs:1537-1545). |
| V-5 | clean start | `tvc deploy approve --manifest fixtures/manifest.json --operator-seed SEED --skip-post --message-format json` | Exit 1; one NDJSON object with `code` = `missing_required_input`; the `reason` names `--dangerous-skip-interactive` (approve.rs:145-147; output.rs:283-293). |
| V-6 | clean start | `tvc deploy approve --dry-run --dangerous-skip-interactive` | Exit 1; stderr contains `a manifest source is required` (test deploy_approve.rs:207; approve.rs:922). |
| V-7 | clean start | `tvc deploy approve --manifest fixtures/manifest.json --operator-seed-path fixtures/seed.hex --dangerous-skip-interactive` | Exit 1; stderr contains `--manifest-id is required to post approval to API` (test deploy_approve.rs:803; approve.rs:149-152). |
| V-8 | tvc config with one YubiKey operator record and its registry entry, serial other than `deadbeef` | `tvc deploy approve --manifest does-not-exist.json --skip-post --dangerous-skip-interactive --serial deadbeef` | Exit 1; stderr contains `no YubiKey operator has serial deadbeef` and never names `does-not-exist.json` (test deploy_approve.rs:372; approve.rs:154-163). |
| V-9 | as V-3, no API key file on disk | `tvc deploy approve --manifest fixtures/manifest.json --skip-post --dangerous-skip-interactive` | Exit 1; stderr contains `--skip-post is not supported for hosted operators` (test deploy_approve.rs:240; operator.rs:358-361). |
| V-10 | tvc config with one YubiKey operator record whose registered key is fixture manifest set member 0; stdin no TTY | `tvc deploy approve --manifest fixtures/manifest.json --skip-post --dangerous-skip-interactive` | Exit 1; stderr contains `a YubiKey operator needs its PIN typed at an interactive prompt`; no device access (test deploy_approve.rs:262; approve.rs:406-411). |
| V-11 | tvc config with an active org and zero operator records | `tvc deploy approve --manifest fixtures/manifest.json --manifest-id 11111111-1111-4111-8111-111111111111 --dangerous-skip-interactive` | Exit 1; stderr contains `no configured operator public key belongs to this manifest set` (test deploy_approve.rs:488; approve.rs:332-334). |
| V-12 | clean start | `tvc deploy approve --manifest fixtures/manifest.json --deploy-id 5376f492-d014-4e01-a6bb-20fc97448e25 --dangerous-skip-interactive` | Exit 2; stderr contains `the argument '--manifest <PATH>' cannot be used with '--deploy-id <DEPLOY_ID>'` (test deploy_approve.rs:764; approve.rs:51). |

## Invariants (normative)

INV-G1 through INV-G4 apply (Part 00). Per-command invariants:

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | Without `--dangerous-skip-interactive`, a run that cannot prompt MUST fail before any file or network I/O. | The gate is the first statement after arg resolution in `Run::run`, before `load_manifest` (approve.rs:145-147). Behavioral check: V-5. |
| INV-2 | Every generated approval MUST verify against the manifest hash and a manifest set member key before use. | The candidate filter requires set membership (approve.rs:225, 309); `approve_manifest` re-verifies through `ValidatedManifest::verify_approval` (operator.rs:115-147; approvals.rs:343-360). Behavioral check: deploy_approve.rs:577. |
| INV-3 | The YubiKey PIN MUST come from a masked prompt only. | In this flow `Pin` is constructed from `prompts::password` alone (approve.rs:414-416); the no-prompt path bails first (approve.rs:406-411). Behavioral check: deploy_approve.rs:262 (V-10). |
| INV-4 | When the deployment was fetched, the posted operator UUID MUST identify the manifest-set operator whose key matches the signer key. | `expected_operator` resolves by key at selection (approve.rs:349-375); post-time `ensure!` checks re-compare id and key (approve.rs:854-866). Behavioral check: deploy_approve.rs:289. |
| INV-5 | The command MUST NOT write the tvc config. | No save path exists in the command: `rg -n "save" tvc/src/commands/deploy/approve.rs` returns nothing; the only dispatch-path save is the absent-file default (cli.rs:219-222). |

## Gaps (informative)

1. **[capability] `--operator-id` cannot select an operator when the manifest comes from a file. It only filters through a fetched deployment.**
   `requested_approval_key` derives only when `fetched` exists (approve.rs:191-210). The candidate filter (approve.rs:301-314) never compares the requested id against the locally known ids on hosted records (`hosted.operator_id()`) or local records (`configured_operator_id`). With `--manifest`, `--operator-id`, and two eligible candidates, a non-interactive run bails `provide one operator selector` (approve.rs:340-343) even though the user provided one. An interactive run re-prompts. The id then acts only as a post-selection assertion (approve.rs:396-402, operator.rs:341-350).

2. **[bug?] The command validates the posting operator UUID only after signing.**
   The check sits at approve.rs:850-852. Under `--manifest` with no `--operator-id`, a YubiKey, explicit-seed, or id-less local selection carries `post_operator_id: None`. The command prompts for the PIN, signs (device touch, or a hosted `sign_raw_payload` activity), and writes `--approval-out`. Only then does it fail with `resolved operator ID required to post approval`. The condition is fully determinable before signing, next to the early `MissingManifestId` check (approve.rs:149-152).

3. **[bug?] The duplicate-approval short-circuit also runs after signing.**
   The `existing` match uses `operator_id` and the signer public key (approve.rs:870-879). Both exist before `approve_manifest` runs (`candidate.public_key`, approve.rs:347). A duplicate still costs a PIN entry plus a YubiKey touch, or a billed hosted signing activity, and then discards the signature.

4. **[consistency] `--skip-post` (`SignerRequirement::OfflineApproval`) does not filter candidate enumeration.**
   The requirement computation runs only after selection (approve.rs:826-830, operator.rs:358-361). Hosted operators therefore remain candidates. The interactive picker offers an operator guaranteed to fail. A non-interactive run with one hosted and one local candidate bails `multiple configured operators...` even though only one can approve offline.

5. **[consistency] The ambiguous-operator bail classifies as `command_error`, unlike its sibling.**
   The sibling is `keys re-encrypt-local-share`. It wraps the same several-operators condition in `MissingRequiredInput::new("--serial")` (re_encrypt_local_share.rs:155-158), so JSON consumers get `missing_required_input`. Approve uses a plain `bail!` (approve.rs:340-343).

6. **[consistency] The command silently ignores `--manifest-id` when `--deploy-id` supplies the manifest.**
   The mechanism is `fetched.map(|f| f.manifest_id).or(args.manifest_id)` (approve.rs:430-434). Fetched state beats an explicit flag with no conflict error. This contradicts the documented flag-first resolution order (cli.rs:19-23). `--manifest-id` conflicts with nothing in the argument groups, so the combination parses.

7. **[consistency] `--serial` has no env var while every other input on the command has one** (approve.rs:91-97).
   `keys re-encrypt-local-share --serial` shares the omission (re_encrypt_local_share.rs:59-62), so it is systemic. The `TVC_OPERATOR_ID` and `TVC_OPERATOR_SEED` siblings in the same selector group both have env forms.

8. **[capability] Non-interactive approval forces the DANGEROUS full review skip.**
   The only escape hatch from the six review prompts is `--dangerous-skip-interactive` (approve.rs:145-147). No CI-safe middle ground exists, such as pinning an expected manifest hash reviewed out of band. Every JSON or CI approval approves blind.

9. **[consistency] Invalid existing-approval warnings vanish in JSON mode.**
   The `enclave will reject this approval and fail to start` warning goes through `human().warn` (approve.rs:815-824). That channel is a no-op in JSON mode (output.rs:146-152). Machine consumers never learn the deployment carries a boot-blocking approval. The sibling status flow serializes verdicts structurally (approvals.rs:149-179).

10. **[docs] The dead defensive YubiKey-registry bail carries remediation that contradicts the load-time invariant.**
    The bail sits at approve.rs:282-289. Config load already rejects any org operator that references an unregistered serial (config/turnkey.rs:125-144), so normal dispatch never reaches the in-loop bail. Its `install its certificates and run tvc keys refresh-yubikey` advice differs from the load-time `edit tvc.config.toml...` message. Also `long_about = None` (approve.rs:50): the help nowhere states that `--deploy-id` overrides `--manifest-id` or that YubiKey approval needs a prompt.
