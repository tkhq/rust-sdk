# tvc deploy provision

## Purpose

Provisions one hosted quorum-key share for a deployment: fetches the deployment's
provisioning details (attestation document + manifest envelope), verifies them, checks the
chosen hosted operator belongs to the manifest share set, then submits a
`ReEncryptTvcQuorumKeyShare` activity so Turnkey re-encrypts that operator's share to the
enclave — the hosted-operator counterpart of the local three-step flow
(`deploy provisioning-details` → `keys re-encrypt-local-share` → `deploy post-share`).
Run it once per hosted share-set operator after a deployment's manifest approvals reach
quorum. Dispatched at `tvc/src/cli.rs:251-253`; implementation
`tvc/src/commands/deploy/provision.rs`.

## Inputs

| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| deployment ID | `-d, --deploy-id <UUID>` | `TVC_DEPLOY_ID` | — | none (required) | never |
| hosted operator | `--operator-id <UUID>` | `TVC_OPERATOR_ID` | — | none (required) | never |
| skip verification | `--dangerous-skip-verification` | `TVC_DANGEROUS_SKIP_VERIFICATION` | — | false | never |
| API credentials | — | `TVC_ORG_ID` + `TVC_API_KEY_PUBLIC` + `TVC_API_KEY_PRIVATE` (all-or-none), `TVC_API_BASE_URL` optional | active org in `~/.config/turnkey/tvc.config.toml` | prod base URL | never |
| operator identity/keys | — | — | active org's hosted `operators` record (name, encrypt/sign public keys, org id) | — | never |

Flag/env resolution follows the global flag > env order (provision.rs:33-43); neither ID
has a config-file source. The operator's key material is config-only: `--operator-id` must
match a hosted record registered in the active org (hosted.rs:264-276), and env auth's
`TVC_ORG_ID` must equal that record's org (provision.rs:81, operator.rs:149-158) — so
despite env-var auth existing for CI, the command still requires the local config file for
the operator registry.

## Interactive behavior

None. No prompts exist on any path — interactive and `--non-interactive`/JSON runs are
identical. Both UUIDs are hard clap requirements, so a missing or malformed value fails at
parse time as `usage_error` (exit 2, JSON NDJSON line when `--message-format json`,
cli.rs:154-182) rather than prompting or emitting `missing_required_input`. With
`--dangerous-skip-verification` a warning line is printed to stderr in human mode only
(provision.rs:72-77; `shell_eprintln` is suppressed in JSON mode, output.rs:267-276).

## Outputs

- Human: one line, `Provisioning Share ID: <id>` (provision.rs:53-61), plus the stderr
  warning when verification is skipped.
- JSON: one outcome message `{"reason": "provisioning_share_created",
  "provisioningShareId": "..."}` (pinned at provision.rs:475-482). Errors emit
  `command_error` / `missing_required_input` per the global taxonomy.

## Side effects

- Reads `~/.config/turnkey/tvc.config.toml` (cli.rs:216) and the active org's API key file
  (client.rs:103-125). Writes no files and mutates no config.
- Three Turnkey API calls: `get_tvc_deployment_provisioning_details` (provisioning.rs:81-85),
  `get_tvc_deployment` (provision.rs:84-85, client.rs:83-100), then the
  `re_encrypt_tvc_quorum_key_share` activity (provision.rs:105-109) — the only mutation,
  server-side.
- Local verification before submission: manifest-set approvals + Nitro attestation chain +
  PCR/manifest-hash binding (provisioning.rs:206-240), deployment manifest must byte-equal
  the provisioning envelope's manifest (provision.rs:137-140), and the operator's composite
  key must be in the manifest share set (provision.rs:141-151). `--dangerous-skip-verification`
  skips only approvals/attestation (still parses the attestation doc, provision.rs:123-126);
  manifest equality and share-set membership are always enforced (pinned at
  provision.rs:374-457).
- No YubiKey/device interaction.

## Failure modes

- Missing/malformed `--deploy-id` / `--operator-id`: clap `usage_error`, exit 2.
- No active org: "No active organization. Run `tvc login` first." (hosted.rs:271) —
  `command_error`, exit 1.
- Operator ID not a hosted record in the active org, duplicate IDs, or malformed registry
  keys (hosted.rs:253-261, 196-203) — `command_error`, exit 1.
- Env-auth org ≠ operator's configured org (operator.rs:149-158) — `command_error`, exit 1.
- Partial env auth / missing API key file (client.rs:226-234, 115-117) — `command_error`.
- Deployment lookup returns no deployment: `MissingResource` → `not_found` (client.rs:97-99).
- Deployment present but manifest missing/empty: plain context strings (provision.rs:93-97)
  → `command_error` (see gap 6).
- Verification failures (approvals, attestation, manifest mismatch, operator not in share
  set, empty share ID in the result: provision.rs:169-172) — `command_error`, exit 1.
- Activity failure: typed `TurnkeyClientError` preserved via `hosted_activity_error`
  (provision.rs:109, hosted.rs:355-364) → `api_error` / `unauthorized` / `not_found` /
  `approval_required` / `network_error` with `httpStatus` when applicable.

## Gaps

1. **[capability] The hosted operator must be supplied as a raw UUID — no default, no
   name-based selection, no prompt.** Even when the org's `default_operator_kind` is
   `Hosted` with exactly one hosted record — the state `select_hosted_operator`
   (config/turnkey/mod.rs:473-483) resolves as the org default elsewhere
   (operator.rs:291-296) — provision refuses to run without `--operator-id`
   (provision.rs:38-39). This is the canonical gap mirrored: `keys re-encrypt-local-share`
   with a hosted default explicitly redirects users to this command
   (operator.rs:435-439) without ever needing the UUID, and `deploy approve` shows the
   expected shape — enumerate configured operators, filter to manifest members, auto-pick a
   sole candidate, prompt on multiple (approve.rs:318-346). Operators have registry names
   (hosted.rs:217-219) that cannot be used here.

2. **[capability] Only hosted operators registered in this machine's local config can
   provision; there is no escape hatch.** `resolve_hosted_operator` errors with "was not
   found in org" for anything absent from `tvc.config.toml` (hosted.rs:273-275), and no
   flags accept the encrypt/sign public keys directly, nor is there an API lookup fallback —
   yet the operator was created server-side (`tvc operator create`) and the intent only
   needs its two public keys (provision.rs:158-159). A teammate on a second machine (env-var
   auth works fine, client.rs:48-64) must hand-copy the hosted record into local config
   first; the `--operator-id` help text does not mention this requirement (provision.rs:37).

3. **[consistency] The same domain role has different flag/env names across the two
   share-posting paths, and `TVC_OPERATOR_ID` is overloaded across commands.** Provision's
   share-set operator is `--operator-id` / `TVC_OPERATOR_ID` (provision.rs:38); `deploy
   post-share`'s is `--share-operator-id` / `TVC_SHARE_OPERATOR_ID` (post_share.rs:26-28);
   `deploy approve` also reads `TVC_OPERATOR_ID` for its (manifest-set) operator selector
   (approve.rs:83-89), so an env var exported for a provision pipeline silently becomes
   approve's operator selection in the same shell.

4. **[consistency] A dangerous (verification-skipped) provision is indistinguishable in
   JSON mode.** The only trace of `--dangerous-skip-verification` is a human-mode stderr
   line (provision.rs:72-77) that JSON mode suppresses (output.rs:267-276), and
   `ProvisioningShareCreated` carries no verification field (provision.rs:49-51) — while
   sibling `deploy provisioning-details` records `"verification":
   "skipped attestation, ..."` in its machine outcome (provisioning_details.rs:93-97, 176).
   CI logs cannot prove which mode produced a share.

5. **[consistency] `TVC_DANGEROUS_SKIP_VERIFICATION` accepts only literal `true`/`false`,
   unlike the global bool envs.** The global `--non-interactive` uses
   `BoolishValueParser` (`1`/`yes`/`on`..., cli.rs:72-78); provision's bool flag declares
   `env` with clap's default bool parser (provision.rs:42-43), so
   `TVC_DANGEROUS_SKIP_VERIFICATION=1` is a parse error where `TVC_NON_INTERACTIVE=1`
   works. Same family-wide trait in provisioning_details.rs:31-32 and
   re_encrypt_local_share.rs:65-66.

6. **[consistency] A deployment response missing its manifest classifies as
   `command_error` instead of `not_found`.** provision.rs:93 uses a bare
   `.context("deployment response missing manifest")` for exactly the "OK response with an
   empty resource" case that the repo contract routes through `MissingResource` (and that
   `fetch_tvc_deployment` itself uses one line earlier for the missing deployment,
   client.rs:97-99), so machine consumers see the generic fallback code
   (errors.rs:93-103). Related: the `invalid_input` code advertised in `LONG_ABOUT`
   (cli.rs:55-56) is `#[allow(dead_code)]` — none of this command's semantic validation
   failures ever produce it (errors.rs:54-56).

7. **[capability] No dry-run/preview: the activity is submitted immediately.** `deploy
   approve` offers `--dry-run` (approve.rs:119-120) and the local flow externalizes every
   artifact for inspection (`--provision-bundle-out`, `--re-encrypted-out`); provision has
   no way to run its full verification + share-set membership check for a specific operator
   without also submitting the re-encrypt activity (provision.rs:98-109).
