# tvc deploy status

## Purpose

Fetches the persisted record of a deployment from the Turnkey API and reports it:
identity (deployment/app/manifest IDs), QOS version, debug/deletion flags, pivot
container, timestamps, and a cryptographic validation of the posted manifest approvals
against the manifest set (per-approval verdicts + quorum reached). Run it to inspect a
deployment's configuration and approval progress. It is NOT live cluster state — that is
`tvc deploy get-status` (replicas, targeted, last-updated), which shares nothing with
this command except the deployment/app/egress header fields.

## Inputs

| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| deployment ID | `-d`, `--deploy-id` (required, UUID) | `TVC_DEPLOY_ID` | none | none | never |
| auth (org + API key) | none | `TVC_ORG_ID` + `TVC_API_KEY_PUBLIC` + `TVC_API_KEY_PRIVATE` (all-or-none) | active org in `~/.config/turnkey/` | none | never |
| API base URL | none | `TVC_API_BASE_URL` (env auth only) | per-org `api_base_url` | `https://api.turnkey.com` | never |

Resolution order is respected where sources exist (flag > env for deploy-id;
env > config for auth, tvc/src/client.rs:48-64), but deploy-id has no config-file
source and no default. Inherited globals (`--non-interactive`, `--message-format`,
`--color`) are not treated specially.

## Interactive behavior

None. The command never prompts in any mode; interactive and non-interactive runs are
identical. A missing or non-UUID `--deploy-id` is a clap parse failure in both modes
(exit 2), not a `missing_required_input`. The only mode-sensitive behavior is the
manifest-parse warning, which is human-mode-only (see Gaps 5).

## Outputs

Human: one block (tvc/src/commands/deploy/status.rs:149-217) — Deployment, App ID,
Egress Enabled, Manifest ID, QOS Version, Marked for deletion, Debug Mode; optional
Pivot Container (URL/Path/Args); optional `Created:`/`Updated:` epoch timestamps; then
`Manifest Approvals: <valid>/<threshold> valid`, one `name (id): verdict` line per
posted approval (verdicts: valid / invalid signature / not in manifest set / duplicate,
tvc/src/approvals.rs:131-140), and `Quorum reached: yes|no`. When the manifest bytes
cannot be parsed, a `warning:` line is printed and the approvals section renders
`<unknown>/<unknown>` with no approval lines.

JSON: single NDJSON terminal outcome, `"reason": "deployment_status"`
(tvc/src/outcome.rs:45), camelCase payload (status.rs:123-139): deploymentId, appId,
egressEnabled, manifestId, qosVersion, markedForDeletion, debugMode, pivotContainer
(url/path/args, nullable), createdAt/updatedAt ({seconds, nanos} strings, nullable),
manifestApprovals — flattened {approvals: [{id, operatorId, operatorName, signature
(hex), createdAt, verdict}], threshold, validCount, quorumReached}, or null when the
manifest could not be parsed.

## Side effects

Read-only. Two API queries: `get_tvc_deployment` (status.rs:39-48) and `get_tvc_app`
(status.rs:79, solely for the app-level `enable_egress` field). No activities
submitted, no files written, no YubiKey interaction, no config mutation — except the
global dispatch behavior of creating a default `~/.config/turnkey/turnkey.toml` when
none exists (tvc/src/cli.rs:219-223, shared by all commands).

## Failure modes

- Missing/invalid `--deploy-id`: clap usage error, exit 2; JSON mode emits reason
  `command_error`, code `usage_error` (tvc/src/cli.rs:154-182, output.rs:344-349).
- No active org / partial auth env vars: `command_error`, exit 1 (client.rs:103-117,
  226-234).
- HTTP 401/403 → `unauthorized`; 404 → `not_found` (+httpStatus); connect/timeout →
  `network_error`; other HTTP → `api_error`. Exit 1.
- API returns success with empty deployment or empty manifest payload:
  `MissingResource` → `not_found` without httpStatus (status.rs:50-52, 72-77,
  errors.rs:22-23), exit 1.
- Any single posted approval that fails boundary parsing (missing operator, non-UUID
  ids, bad hex / non-P256 key) aborts the whole command (status.rs:81-84):
  missing operator → `not_found`, the rest → `command_error`. Exit 1.
- Broken manifest set (unparseable member key, oversized threshold) at
  `ValidatedManifest` construction: hard error, `command_error`, exit 1
  (status.rs:98-102, approvals.rs:220-227) — intentional per approvals.rs:10-14.
- Unparseable manifest bytes: NOT an error — warn (human only) and degrade to
  `manifestApprovals: null`, exit 0 (status.rs:86-96).

## Gaps

1. **[consistency][docs] `status` vs `get-status` naming is incoherent across the CLI
   and the help text doesn't disambiguate.** `app status` is live cluster status
   (cli.rs:444-445) while `deploy status` is the persisted record and the live variant
   is `deploy get-status` (cli.rs:396-397, 404-405) — plain `status` means opposite
   things in the two groups. `deploy status`'s about text, "Get the status of a
   deployment" (cli.rs:404, status.rs:24), gives a user no way to know which of the two
   they want; neither command has a long_about (`long_about = None`, status.rs:26,
   get_status.rs:25) or cross-references the other.

2. **[capability] The deployment ID must arrive as a known UUID; there is no way to
   discover or pick one from the command.** `--deploy-id` is a required clap UUID with
   no prompt and no config fallback (status.rs:29-30), and no `deploy list` command
   exists (cli.rs:392-422) — discovery requires `app status`'s live listing
   (app/status.rs) which only shows deployments present in cluster state. An
   interactive user inspecting approval progress must paste a UUID obtained elsewhere.

3. **[bug?] Created/Updated nanos render space-padded, not zero-padded.**
   `TimestampPayload.nanos` is a String (app_status.rs:21-24) and `{:09}` does not
   zero-pad strings — `format!("{:09}", "5")` yields `"5        "` (verified with a
   standalone rustc test), so `Created: 1723473600.5        s` prints where
   `.000000005s` is meant. status.rs:185, 189 and get_status.rs:117-119 both have it;
   app/status.rs:136 already uses the correct `{:0>9}`.

4. **[consistency] One malformed approval row hard-fails the whole status report,
   contradicting the command's own degrade-gracefully design.** Approval parsing
   collects into `Result` and `?`s out (status.rs:81-84), so a single bad public-key
   hex on any posted approval hides the entire deployment record — while the very next
   step treats unparseable manifest bytes as a warning plus degraded output
   (status.rs:86-102), and approvals.rs:6-8 states validation "classifies every
   approval instead of failing fast". In the degraded manifest path, even successfully
   parsed approvals are dropped from the output entirely (report field is None,
   status.rs:98-104), so the user cannot see who has approved.

5. **[capability] JSON mode carries no signal for the degraded manifest-parse path.**
   The warning is emitted via `ctx.shell().human().warn(...)` which is a no-op in JSON
   mode (status.rs:88-90, output.rs:146-148), so a machine consumer sees only
   `"manifestApprovals": null` with no reason field or warning message distinguishing
   "manifest unparseable" — the parse error text is unrecoverable except via
   `RUST_LOG` debug.

6. **[consistency] The deployment fetch is re-implemented inline instead of using the
   shared helper.** status.rs:39-52 duplicates `client::fetch_tvc_deployment`
   (client.rs:83-100) verbatim (same request, same context string, same
   `MissingResource`); `deploy get-status` uses the shared helper (get_status.rs:44).
   Behavior is identical today but the two copies can drift.
