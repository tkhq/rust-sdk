# tvc app delete

## Purpose

Submits `ACTIVITY_TYPE_DELETE_TVC_APP_AND_DEPLOYMENTS` to mark a TVC app **and every one of its deployments** for deletion (`tvc/src/commands/app/delete.rs:27-50`). Run it to retire an app entirely; there is no CLI or API path to restore an app afterwards (deployments have `deploy restore`; apps do not — `proto/activities.json:383-397`).

## Inputs

| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| app id | `--app-id <APP_ID>` (UUID, clap-required) | `TVC_APP_ID` | — | — | never |
| auth: org id | — | `TVC_ORG_ID` | active org `id` in `~/.config/turnkey/` | — | never |
| auth: API key | — | `TVC_API_KEY_PUBLIC` + `TVC_API_KEY_PRIVATE` | active org's stored key file | — | never |
| API base URL | — | `TVC_API_BASE_URL` | active org `api_base_url` | `https://api.turnkey.com` | never |

- App id parses as `uuid::Uuid` at the clap boundary (`delete.rs:22`); a non-UUID is a usage error before any config/network work.
- Auth deviates from per-value flag > env > config: env auth is all-or-nothing. All three env vars set → env wins; none → config; a partial set errors listing the missing names, with no env/config merge (`tvc/src/client.rs:38-46`, `226-234`).
- No config-file fallback for the app id. `deploy init`/`deploy create` fall back to the per-org `last_created_app_id` config key (`tvc/src/config/turnkey/mod.rs:710-712`); this command deliberately (and defensibly, for a delete) does not.
- Inherited globals `--non-interactive` / `--message-format` have no command-specific effect: the command never prompts either way (`ctx` is bound as `_ctx`, `delete.rs:27`).

## Interactive behavior

None. There is no confirmation prompt, no app selection, and no difference between interactive and `--non-interactive`/JSON runs. A missing `--app-id` is a clap usage error (exit 2) even on a TTY (`tvc/tests/app_delete.rs:39-48`); it is never prompted for. Deletion of the app and all its deployments proceeds immediately once arguments parse and auth resolves.

## Outputs

Human mode (single terminal message, `delete.rs:73-88`):

```
App delete accepted.
App and deployments marked for deletion.

App ID: <id>
Activity ID: <id>
Activity Status: ACTIVITY_STATUS_COMPLETED
```

JSON mode: one NDJSON object with `reason: "app_deleted"` (variant name is the wire reason, `tvc/src/outcome.rs:30,56`) carrying camelCase `appId`, `activityId`, `activityStatus`. On success `activityStatus` is always `ACTIVITY_STATUS_COMPLETED`: the client's activity poller only returns `Ok` for completed activities (`client/src/lib.rs:361`); every other status becomes an error.

## Side effects

- Reads `~/.config/turnkey/` config at dispatch; if the file is missing, dispatch writes a fresh default config file before running the command (`tvc/src/cli.rs:219-223`).
- Submits and polls the `DELETE_TVC_APP_AND_DEPLOYMENTS` activity against the Turnkey API (`delete.rs:39-43`; polling in `client/src/lib.rs:348-388`).
- No local config mutation by the command itself — notably it does NOT clear `last_created_app_id` (see Gaps).
- No YubiKey or other device interaction; no files written.

## Failure modes

- Missing/invalid `--app-id`: clap usage error — exit 2; in JSON mode a `reason: "command_error"`, `code: "usage_error"` NDJSON line (`tvc/src/cli.rs:154-182`, `tvc/src/output.rs:344-352`).
- Partial env auth / no active org / no stored API key: `command_error`, exit 1 (`tvc/src/client.rs:104-117`, `226-234`).
- HTTP 401/403 → `unauthorized`; 404 → `not_found`; other statuses → `api_error` (with `httpStatus`) (`tvc/src/errors.rs:212-226`).
- Activity `CONSENSUS_NEEDED`/`AUTHENTICATORS_NEEDED` → `TurnkeyClientError::ActivityRequiresApproval` → `approval_required` (`client/src/lib.rs:376-378`, `tvc/src/errors.rs:236-238`). No follow-up command exists to resume/inspect a pending app deletion.
- Activity `FAILED`, unexpected status, or poll retries exhausted → `api_error` (`tvc/src/errors.rs:246-258`).
- Connect/timeout/DNS → `network_error` (`tvc/src/errors.rs:227-235`).
- All runtime failures exit 1.

## Gaps

1. **[consistency] The CLI's most destructive command has no confirmation, while the strictly less destructive `profile delete` requires one.** `app delete` irreversibly deletes an app plus all deployments with zero prompt and no `--yes`-style acknowledgment flag (`delete.rs:27-50`, `ctx` unused), whereas `profile delete` — a purely local operation — demands interactive confirmation or `--yes` (and requires `--yes` in non-interactive mode) (`tvc/src/commands/login.rs:56-58`, `129-136`, `205`). The original implementation had a double confirmation (`confirm_yes_no` + typed app-id echo) with a `--dangerous-skip-confirmation` escape hatch; commit `8234cacf` ("review") stripped it while keeping the helpers.

2. **[consistency] `tvc/src/commands/confirmation.rs` is orphaned machinery built for exactly this command.** `confirm_yes_no`/`confirm_typed` have zero callers outside their own unit tests (`confirmation.rs:11`, `20`) — the tests still say "Type app id" (`confirmation.rs:129-147`). Live commands that confirm use `prompts::confirm_or_bail` instead (`tvc/src/prompts.rs:70`; e.g. `yubikey/unregister.rs:104`). Either wire `confirm_typed` back into `app delete` (its intended consumer) or delete the module.

3. **[bug?] The shared `TVC_APP_ID` env var silently selects the deletion target.** The same env var feeds `app status` and `deploy create` (`app/status.rs:27`, `deploy/create.rs:105`), so an exported `TVC_APP_ID` from an unrelated workflow makes a bare `tvc app delete` valid — and, combined with gap 1, an immediate, unconfirmed, unrecoverable deletion (`delete.rs:21`). This is the audit's "state silently constraining/supplying an explicit choice" shape, in its most dangerous direction.

4. **[capability] No restore path for a deleted app.** `deploy delete` is paired with `deploy restore` (`tvc/src/cli.rs:418-421`), but no `app restore` exists and no `ACTIVITY_TYPE_RESTORE_TVC_APP` exists upstream (`proto/activities.json:383-397`) — partly an API gap, but the CLI surface asymmetry is real. Unclear (and undocumented) whether the cascade-deleted deployments can be individually revived via `deploy restore`.

5. **[capability] No interactive app selection or name-based targeting.** `--app-id` is a hard clap requirement even on a TTY (`tvc/tests/app_delete.rs:39-48`), despite the global contract that commands may prompt for missing values (`cli.rs:41-42`) and despite `deploy create` filling missing inputs interactively (`deploy/create.rs:161-168`) and `profile delete` offering a picker (`login.rs:313-328`). You must know the UUID; there's no select-from-`app list` flow. Reasonable strictness for a delete — but only if a confirmation step (gap 1) exists to back it up.

6. **[bug?] Deleting an app leaves a stale `last_created_app_id` pointing at it.** `app create` records the id per org (`app/create.rs:272`); `deploy init` and `deploy create` default to it (`deploy/init.rs:85`, `deploy/create.rs:202`); `app delete` never clears it, so subsequent `deploy init`/`deploy create` silently target a deleted app.

7. **[docs] Help text omits the soft-delete framing and the lack of a restore path.** The subcommand help is one line — "Delete an app and all of its deployments" (`cli.rs:454-455`) with `long_about = None` (`delete.rs:18`). Only the post-success output reveals "marked for deletion" (`delete.rs:77-78`), phrasing that implies recoverability the CLI does not offer (gap 4); sibling `deploy delete` at least says "by marking it for deletion" up front (`cli.rs:418`).
