# tvc app list

## Purpose
Lists all TVC apps in the authenticated organization, optionally narrowed by a
client-side name filter. Run it to discover app IDs (every other `app`/`deploy`
command addresses apps by `--app-id`) or to eyeball quorum key / live deployment
state across the org.

## Inputs
| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| name filter | `-n`, `--name` | `TVC_APP_NAME` | — | none (no filter) | never |
| org / credentials | — | `TVC_ORG_ID` + `TVC_API_KEY_PUBLIC` + `TVC_API_KEY_PRIVATE` (all-or-none) | active org in `~/.config/turnkey/tvc.config.toml` | — | never |
| API base URL | — | `TVC_API_BASE_URL` | active org's `api_base_url` | `https://api.turnkey.com` | never |

- Name filter follows the standard flag > env order (`tvc/src/commands/app/list.rs:24`); there is no config-file tier for it.
- Auth deviates from per-value resolution: the three auth env vars resolve as a set — all three set wins wholesale over config, a partial set is a hard error, never a merge (`tvc/src/client.rs:48-64`, `tvc/src/client.rs:221-234`).
- No `--org` flag: with env auth absent, the org is always the config file's `active_org` (`tvc/src/client.rs:104-106`).

## Interactive behavior
None. The command never prompts in any mode — `ctx` is unused
(`tvc/src/commands/app/list.rs:30`). Behavior is identical under
`--non-interactive` and `--message-format json`; nothing becomes a hard
requirement because nothing is ever collected interactively.

## Outputs
Human mode: one block per app —
`Name / ID / Quorum Public Key / Live Deployment / Egress Enabled / Debug Mode Deployments [/ Public Domain]`
followed by a 40-char `─` separator (including after the last app)
(`tvc/src/commands/app/list.rs:123-147`). `Live Deployment: (none)` when unset;
`Public Domain` line omitted when empty. Empty result prints `No apps found.`
(`tvc/src/commands/app/list.rs:109-111`).

JSON mode: one NDJSON line, reason `apps_listed` (`tvc/src/outcome.rs:52`):
`{"reason":"apps_listed","apps":[{"id","name","quorumPublicKey","liveDeploymentId","egressEnabled","debugModeDeploymentsEnabled","publicDomain"}]}`.
camelCase keys; `liveDeploymentId` is `null` when absent; `publicDomain` is
omitted entirely when empty (`tvc/src/commands/app/list.rs:56-74`). Empty
result is `"apps": []`, still exit 0.

The API's `manifest_set`, `share_set`, `created_at`, `updated_at`, and
`organization_id` are deliberately dropped in the exhaustive destructure
(`tvc/src/commands/app/list.rs:88-92`).

## Side effects
- Reads `~/.config/turnkey/tvc.config.toml`; if the file does not exist the
  dispatcher writes a default one before running (`tvc/src/cli.rs:219-223`) —
  the command's only write.
- Reads the active org's stored API key file when env auth is absent
  (`tvc/src/client.rs:115-117`).
- One read-only API call: `get_tvc_apps` (`tvc/src/commands/app/list.rs:33-39`).
  No activities submitted, no YubiKey interaction.
- Name filtering happens client-side after the full fetch
  (`tvc/src/commands/app/list.rs:50-54`).

## Failure modes
- Partial auth env vars → bail naming the missing vars (`tvc/src/client.rs:226-233`); classifies `command_error`, exit 1.
- No active org / no stored API key → "Run `tvc login` first." anyhow errors (`tvc/src/client.rs:104-117`); `command_error`, exit 1.
- HTTP errors from `get_tvc_apps`: 401/403 → `unauthorized`, 404 → `not_found`, other statuses → `api_error` (or `client_version_too_old`), connect/timeout/DNS → `network_error` (`tvc/src/errors.rs:212-229`); exit 1.
- Bad flags → clap usage error, exit 2; emitted as `usage_error` NDJSON when `--message-format json` was requested (`tvc/src/cli.rs:154-182`).
- A filter matching nothing is success (exit 0, `No apps found.` / `"apps": []`), not `not_found`.

## Gaps
1. **[capability] Cannot list apps for any org other than the active one.**
   `build_client` only knows the env-var triple or `config.active_org_config()`
   (`tvc/src/client.rs:48-64`, `tvc/src/client.rs:104-106`); there is no
   `--org` flag, so listing another configured org's apps requires re-running
   `tvc login` to switch or exporting three env vars. Sibling
   `keys backup-operator-key` already has `--org` / `TVC_ORG` selecting any
   configured profile (`tvc/src/commands/keys/backup_operator_key.rs:28-29`,
   `:51-61`) — the exact "state silently constrains explicit choice" shape.

2. **[capability] Manifest set, share set, and timestamps are unviewable after creation.**
   The list output drops `manifest_set`, `share_set`, `created_at`,
   `updated_at` (`tvc/src/commands/app/list.rs:88-92`); `app create` prints
   manifest-set IDs only at creation time
   (`tvc/src/commands/app/create.rs:287-313`), and no `app get`-style command
   exists even though the single-app endpoint is already wrapped as
   `fetch_tvc_app` (`tvc/src/client.rs:67-80`). Once created, no CLI path shows
   an app's share set or age.

3. **[capability] Name substring is the only filter — no `--app-id`, no exact match.**
   `filter_by_name` is a case-sensitive `contains`
   (`tvc/src/commands/app/list.rs:50-54`). Every sibling command addresses apps
   by `--app-id` / `TVC_APP_ID` (e.g. `tvc/src/commands/app/status.rs:27`,
   `tvc/src/commands/app/delete.rs:21`), yet the list cannot be narrowed by the
   identifier the rest of the CLI runs on, and names like `app` vs `app-2`
   cannot be isolated.

4. **[consistency] An ambient `TVC_APP_NAME` silently narrows JSON output to possibly `[]`.**
   The filter's env tier (`tvc/src/commands/app/list.rs:24`) means a leftover
   `TVC_APP_NAME` in a CI environment yields `{"reason":"apps_listed","apps":[]}`
   with exit 0 — indistinguishable from "org has no apps". Legal under the
   documented flag > env order, but no other command consumes `TVC_APP_NAME`,
   so nothing else warns that the variable is load-bearing here.

5. **[docs] Help text does not say the filter is a case-sensitive substring match.**
   `/// Filter by app name.` (`tvc/src/commands/app/list.rs:23`) reads as exact
   match; the tests pin substring semantics
   (`tvc/src/commands/app/list.rs:186-194`). One doc-comment word ("substring")
   would fix it.

6. **[capability] No pagination — upstream, not CLI.** `GetTvcAppsRequest` carries
   only `organization_id` and the response is the bare full list
   (`client/src/generated/services.coordinator.public.v1.rs:1112-1124`), so the
   CLI has nothing to expose; the whole org is fetched in one call. Noted so
   nobody files this against the CLI.
