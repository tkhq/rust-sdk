# tvc deploy init

## Purpose

Generates a deployment config JSON file for `tvc deploy create` to consume. Three modes:
blank placeholder template (default), seeded from an existing deployment
(`--from-deployment`), or prompt-filled (`--interactive`). Run it when starting a new
deployment or cloning an existing one's settings.

## Inputs

| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| output path | `-o, --output <PATH>` | `TVC_DEPLOY_CONFIG_OUT` | — | `deploy-<YYYY-MM-DD-HHMMSS>.json` (local time; init.rs:71-74) | no |
| seed deployment id | `--from-deployment <DEPLOY_ID>` | `TVC_FROM_DEPLOYMENT` | — | none (blank template) | no |
| interactive fill | `--interactive` | — (no env; init.rs:46-47) | — | false | n/a |
| app id (template prefill) | — (no flag) | — | `last_created_app_id[active org]` in `~/.config/turnkey` (turnkey/mod.rs:710-713) | `<FILL_IN_APP_ID>` | yes, in `--interactive` (saved id offered as default; deploy.rs:151) |
| auth (only used with `--from-deployment`) | — | `TVC_ORG_ID` / `TVC_API_KEY_PUBLIC` / `TVC_API_KEY_PRIVATE` (+optional `TVC_API_BASE_URL`) | active org creds from `tvc login` | — | no |

Resolution-order note: the app id has no flag or env, so the config-file layer (last
*created* app for the active org) is the highest — and only — priority; there is no way
to explicitly override it at the CLI (init.rs:85, 98).

Every field of the written file (schema: `DeployConfig`, config/deploy.rs:29-51):
`appId`, `qosVersion` (default `0.12.1`, deploy.rs:24), `pivotContainerImageUrl`,
`pivotPath`, `pivotArgs` (`[]`), `expectedPivotDigest`, `dangerousDeployDebugMode`
(`false`), `pivotContainerEncryptedPullSecret` (sentinel
`<REMOVE_ME_IF_PIVOT_CONTAINER_URL_IS_PUBLIC>`), `healthCheckType` (hard-coded
`TVC_HEALTH_CHECK_TYPE_HTTP`, deploy.rs:134), `healthCheckPort` (3000),
`publicIngressPort` (3000). `replicas` is never emitted (see Gaps).

## Interactive behavior

`--interactive` is gated up front (init.rs:53-59): under `--non-interactive` /
`TVC_NON_INTERACTIVE=true` / `--message-format json` it bails
("--interactive conflicts with..."; prompts.rs:29-31); otherwise stdin must be a TTY
(prompts.rs:33-38).

`fill_interactively` (deploy.rs:145-179) prompts only for fields still holding
`<FILL_IN...>` placeholders, in order: App ID (default = saved last-created app id),
QOS version, pivot container image URL, pivot path, expected pivot digest
("(sha256:...)" hint), then — if the pull-secret sentinel is present — a confirm "Is the
container image in a public registry?". Both answers clear the field; answering "no"
additionally prints a note to pass `--pivot-pull-secret` to `deploy create`
(deploy.rs:167-177). With `--from-deployment` all fields arrive filled, so at most the
pull-secret question fires. Ports, health-check type, pivot args, debug mode, and
replicas are never prompted.

Without `--interactive` the command is fully non-interactive in both modes: it writes the
template/seeded file and exits. Blank-template mode needs no auth or network.

## Outputs

Human mode: `Created deployment config[ template]: <path>`, then (from-deployment only)
guidance that digest/debug were copied and (if the source used a private image) that the
pull secret must be re-supplied (init.rs:128-138, 163-169), then the
`tvc deploy create --config-file <path>` next step and shared `PORT_GUIDANCE`
(deploy/mod.rs:17-23).

JSON mode: one message, reason `deployment_config_created` (outcome.rs:30, 47), camelCase
fields `command` ("deploy init"), `path`, `template`, `interactive`, `fromDeployment`,
`needsPullSecret` (init.rs:140-149).

## Side effects

- Creates `~/.config/turnkey/<config>` with defaults if absent (all commands;
  cli.rs:219-223). Reads (never writes) `last_created_app_id`.
- Writes the output JSON file; refuses to overwrite an existing file (init.rs:77-79).
- With `--from-deployment`: builds an authenticated client (env creds preferred over
  `tvc login` config; client.rs:48-64) and calls `get_tvc_deployment` for the active
  org (init.rs:93-96), then decodes the QOS manifest to recover
  `expectedPivotDigest` (bare hex of `pivot_hash()`) and debug mode (deploy.rs:82-110).
- No Turnkey activities submitted; no YubiKey interaction.

## Failure modes

- Output file exists → `bail!` (init.rs:78) → `command_error`, exit 1 (fallback
  classification, errors.rs:93-103).
- `--interactive` + non-interactive mode, or no TTY → `command_error`, exit 1.
- `--from-deployment`: no active org and no env creds ("No active organization. Run
  `tvc login` first.", client.rs:106) or partial env creds → `command_error`; HTTP
  failures classify via `TurnkeyClientError` (`unauthorized`/`not_found`/`api_error`/
  `network_error`); deployment resolved-but-empty → `MissingResource` → `not_found`
  (client.rs:97-99); deployment with no manifest, undecodable manifest, no container
  spec, or port > u16 → `command_error` (deploy.rs:82-101).
- Serialization/write failures → `command_error`, exit 1.
- Bad flags → clap parse error, exit 2.

## Gaps

1. **[capability] `replicas` is honored by `deploy create` from the config file but
   `deploy init` never emits it, in any mode.** The field is
   `skip_serializing_if = "Option::is_none"` (config/deploy.rs:49-50) and both
   `DeployConfig::template` (deploy.rs:137) and `TryFrom<TvcDeployment>` (deploy.rs:116)
   set `None`, so neither the blank template nor a seeded config ever contains the key —
   while create reads it from the file and forwards it to the intent
   (create.rs:320-322, 382; test create.rs:768-778). A user editing the generated file
   cannot discover the field. This is the exact "field create accepts but init never
   emits" class; every other `DeployConfig` field is covered.

2. **[consistency] The blank template's `appId` is silently prefilled from
   per-machine state with no marker and no explicit override.** `init` has no
   `--app-id` flag (create does, create.rs:105-106); the last *created* — not last
   used — app id for the active org (turnkey/mod.rs:710-713) replaces the
   `<FILL_IN_APP_ID>` placeholder (init.rs:85, deploy.rs:126). The generated file then
   looks complete for possibly the wrong app, and `create` will happily accept it
   (UUID + app-exists checks only, deploy.rs:225, create.rs:412). State silently beating
   explicit choice, with no way to make the explicit choice.

3. **[capability] Interactive mode can only fill the five placeholder fields plus the
   pull-secret question; ports, `healthCheckType`, `pivotArgs`, debug mode, and
   `replicas` are unreachable.** `fill_interactively` (deploy.rs:145-179) never asks
   about them and init has no flags for them, so an interactive user still must
   hand-edit the "filled" config for anything non-default. Notably `healthCheckType` is
   hard-coded to HTTP (deploy.rs:134) even though `TVC_HEALTH_CHECK_TYPE_GRPC` exists
   (client/src/generated/immutable.common.v1.rs:1359) and no command — init or
   create — exposes a flag for it; JSON hand-editing is the only path.

4. **[consistency] `--from-deployment` takes a raw `String` while every sibling deploy
   command parses the deployment id as `Uuid` at the CLI boundary.** init.rs:42 vs
   `deploy status`/`get-status` (`pub deploy_id: Uuid`, status.rs:29-30,
   get_status.rs:28-29). A malformed id costs an auth + API round-trip and exits 1 as an
   API error instead of failing parse with exit 2.

5. **[consistency] The two fill paths produce different digest formats for the same
   field.** `--from-deployment` writes bare hex (`hex::encode(manifest.pivot_hash())`,
   deploy.rs:109); the interactive prompt hints `"Expected pivot digest (sha256:...)"`
   (deploy.rs:164-165). create never validates or normalizes the field, so one of the
   two conventions is misleading.

6. **[consistency] Interactive + `--from-deployment` with a private-image source loses
   the needs-pull-secret signal in the final output.** The pull-secret confirm clears the
   sentinel regardless of answer (deploy.rs:167-177), and `needs_pull_secret` is computed
   after the fill (init.rs:109), so `PULL_SECRET_GUIDANCE` is skipped (init.rs:166-168)
   and JSON `needsPullSecret` would be false; the only trace is a transient note printed
   mid-prompt-walk.

7. **[docs] `LONG_ABOUT` claims `--from-deployment` "copies every field from that
   deployment" but `replicas` is not copied.** init.rs:23-25; the API deployment type
   carries no desired replica count so `TryFrom` leaves it unset (deploy.rs:116, test
   deploy.rs:397-405) — only the pull secret is named as unrecoverable.

8. **[docs] Stale comment says `--from-deployment` "deliberately leaves blank" the
   expected pivot digest; it is actually copied.** init.rs:101-103 vs deploy.rs:109 and
   init's own `FROM_DEPLOYMENT_GUIDANCE` (init.rs:128-133), which correctly say the
   digest and debug mode are copied from the source manifest.

9. **[consistency] No `--force`/overwrite escape hatch, and sibling init defaults
   diverge.** Existing output bails as a generic `command_error` (init.rs:77-79), same as
   `app init` (app/init.rs:48-50) — no non-interactive way to regenerate over a stale
   file short of deleting it. `deploy init` defaults to a timestamped name while
   `app init` defaults to fixed `app.json` (app/init.rs:23), so the collision behavior
   differs between the two siblings for no documented reason.
