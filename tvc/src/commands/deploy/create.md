# tvc deploy create

## Purpose

Creates a TVC deployment for an app. Resolves a `DeployConfig` from a JSON config file,
flags, and env vars (flag > env > file > template default), verifies the app exists,
optionally encrypts a pivot pull secret for the org's API environment, validates the
container image server-side, pins the image URL to the resolved digest, and submits the
`create_tvc_deployment` activity. Typically run after `tvc deploy init` + editing the
generated config, or fully flag/env-driven in CI.

Dispatch: `tvc/src/cli.rs:260-262`, with `long_about = LONG_ABOUT` (`create.rs:22-65`) and
`after_help = PORT_GUIDANCE` (`tvc/src/commands/deploy/mod.rs:17-23`, `cli.rs:406-411`).

## Inputs

| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| deploy config path | `-c, --config-file` | `TVC_DEPLOY_CONFIG` | — | none (flag-only template) | no |
| app id | `--app-id` | `TVC_APP_ID` | `appId` | none | yes (default: last app id from `tvc app create`) |
| QOS version | `--qos-version` | `TVC_QOS_VERSION` | `qosVersion` | `0.12.1` (`config/deploy.rs:24,127`) | only if file has placeholder |
| pivot image URL | `--pivot-image-url` | `TVC_PIVOT_IMAGE_URL` | `pivotContainerImageUrl` | none | yes |
| pivot path | `--pivot-path` | `TVC_PIVOT_PATH` | `pivotPath` | none | yes |
| expected pivot digest | `--expected-pivot-digest` | `TVC_EXPECTED_PIVOT_DIGEST` | `expectedPivotDigest` | none | yes |
| pivot args | `--pivot-args` (comma-split) | `TVC_PIVOT_ARGS` | `pivotArgs` | `[]` | no |
| debug mode | `--dangerous-deploy-debug-mode` | `TVC_DANGEROUS_DEPLOY_DEBUG_MODE` | `dangerousDeployDebugMode` | `false` | no |
| health check port | `--health-check-port` | `TVC_HEALTH_CHECK_PORT` | `healthCheckPort` | `3000` | no |
| public ingress port | `--public-ingress-port` | `TVC_PUBLIC_INGRESS_PORT` | `publicIngressPort` | `3000` | no |
| replicas | `--replicas` | `TVC_REPLICAS` | `replicas` | backend default (`None`) | no |
| health check type | — | — | `healthCheckType` | `TVC_HEALTH_CHECK_TYPE_HTTP` | no |
| pull secret (plaintext file path) | `--pivot-pull-secret` | `TVC_PIVOT_PULL_SECRET` | `pivotContainerEncryptedPullSecret` (pre-encrypted value, not a path) | none | public/private question only; the path itself is never prompted |
| auth | — | `TVC_ORG_ID` / `TVC_API_KEY_PUBLIC` / `TVC_API_KEY_PRIVATE` / `TVC_API_BASE_URL` | `~/.config/turnkey` (via `tvc login`) | prod API URL | no |

Deviations from flag > env > file > default (`create.rs:290-323`, documented at
`cli.rs:25-32`):

- `--pivot-args` **replaces** the file's list entirely; never appends (`create.rs:306-308`).
- `--dangerous-deploy-debug-mode` is one-way: it can flip file `false` → `true`, but its
  absence never turns off a file's `true` (`create.rs:309-312`).
- `--replicas` is one-way the same way: an absent flag never resets a file value
  (`create.rs:319-322`).
- Without `--config-file`, missing optional fields silently take template defaults
  (`flag_only_template()`, `create.rs:282-288`): qos version `0.12.1`, ports 3000/3000,
  HTTP health check, empty args, no pull secret.

## Interactive behavior

Mode split is at `run()` (`create.rs:161-169`): non-interactive iff `--non-interactive` /
`TVC_NON_INTERACTIVE` or `--message-format json` (`output.rs:209-210`). There is **no TTY
detection** anywhere in this path (contrast `deploy init --interactive`, which calls
`ensure_stdin_is_tty`, `init.rs:52-59`).

Interactive flow (`create.rs:171-218`): read config file (or flag-only template), apply
overrides, then loop `validate()` → `fill_interactively()` (`config/deploy.rs:145-179`).
Prompt order, each only while the field still holds a `<FILL_IN...>` placeholder:

1. App ID (default prefilled from saved last app id, `config/turnkey/mod.rs:710`)
2. QOS version
3. Pivot container image URL
4. Pivot path (inside container)
5. Expected pivot digest (sha256:...)
6. If the pull-secret sentinel `<REMOVE_ME_IF_PIVOT_CONTAINER_URL_IS_PUBLIC>` is present:
   "Is the container image in a public registry?" — either answer clears the field; a
   "no" only prints a note to pass `--pivot-pull-secret` (`config/deploy.rs:167-177`).

If any prompt ran and a config path was given, offers to save the filled config back to
that path (default Yes, `create.rs:208-210,332-350`). Non-placeholder validation errors
(e.g. non-UUID app id) bail immediately even in interactive mode (`create.rs:196-199`).

Non-interactive: no prompts, no config-file writes (test `create.rs:934-957`). Everything
must resolve from file+flags+env; `validate()` failure bails listing every offending
field (`create.rs:240-242`). A leftover pull-secret sentinel is a hard error pointing at
`--pivot-pull-secret` (`config/deploy.rs:254-258,273-277`).

## Outputs

Human mode: "Creating deployment for app '<id>'..." + a port summary block
(`format_port_summary`, `mod.rs:25-43`), optionally "Using pinned image reference for
deployment request: <url@digest>" when the input URL had no digest (`create.rs:436-441`),
then the success block (deployment ID, app ID, config path if used, next steps pointing
at `deploy status` and `deploy approve`) (`create.rs:482-508`).

JSON mode: exactly one terminal NDJSON line, reason `deployment_created`, fields
`deploymentId`, `appId`, `pinnedImageUrl`, and `configPath` (only when a config file was
used) (`create.rs:471-480`, `outcome.rs:46`). All progress prints are human-only no-ops
in JSON mode. Errors emit `command_error` envelopes with a `code` (`cli.rs:129-133`).

## Side effects

- Reads: deploy config file, plaintext pull-secret file, `~/.config/turnkey` login config.
- Writes: optionally rewrites the deploy config file (interactive save offer only). The
  global dispatcher creates a default `~/.config/turnkey` config if absent
  (`cli.rs:219-223`) — not create-specific. Does **not** update the saved last-app-id
  (only `app create` does, `commands/app/create.rs:272`).
- Local crypto: the pull secret is HPKE-encrypted to a hardcoded per-environment public
  key chosen by `api_base_url` (`pull_secret.rs:18-39,42-66`).
- Turnkey API: `get_tvc_app` (existence check only; response discarded,
  `create.rs:412`), `validate_tvc_image` (also resolves the image digest,
  `create.rs:425-429`), `create_tvc_deployment` activity (`create.rs:454-458`).
- No YubiKey / device interaction.

## Failure modes

- Bad flags/args: clap usage error, exit 2 (`cli.rs:154-182`).
- Missing/unparseable config file, empty pull-secret file, unsupported API base URL for
  pull-secret encryption, validation failures (placeholders, non-UUID app id, pull-secret
  sentinel): all surface as bare anyhow errors → code `command_error`, exit 1.
- App not found: `MissingResource` → code `not_found` (`client.rs:77-80`,
  `errors.rs:95-97`).
- Image validation / create activity failures: classified from the HTTP/activity error
  (`api_error`, `unauthorized`, `network_error`, ...), exit 1.
- Debug-mode deploy against an app created without
  `--dangerous-enable-debug-mode-deployments`: rejected server-side at creation.

## Gaps

1. **[bug?] `--pivot-pull-secret` does not satisfy the config's pull-secret placeholder —
   non-interactive mode bails telling you to pass the flag you already passed.**
   `validate()` hard-errors on the sentinel (`config/deploy.rs:254-258`) before
   `pivot_pull_secret` is ever consulted (`create.rs:240-242`); the flag's encrypted value
   would only override the config field later, at execution (`create.rs:414-417`). So an
   init-generated private-image config + `--pivot-pull-secret` + `--non-interactive` fails
   with "pass --pivot-pull-secret <PATH>". Interactive mode likewise asks "Is the
   container image in a public registry?" even when the flag was passed, and the answer is
   irrelevant (both branches clear the field; the flag's secret is attached regardless).

2. **[bug?] The "Write a new config file at {path}?" offer is unreachable — pointing
   `--config-file` at a nonexistent path errors instead of bootstrapping a new file.**
   `read_config_file_bytes` propagates the read failure for any provided path
   (`create.rs:256-258`), and `file_loaded` is only `false` when no path was given — but
   the save offer requires a path (`create.rs:208-210`), so `offer_to_save_config` is
   always called with `file_loaded == true` and the `create.rs:341` branch is dead code.
   The prompt wording shows the intended (missing) capability.

3. **[capability] `healthCheckType` is config-file-only: no flag, no env var, no prompt.**
   The `Overrides` struct has no such field (`create.rs:88-152`), so flag-only mode
   hard-codes HTTP (`config/deploy.rs:134`) even though the enum has GRPC
   (`client/src/generated/immutable.common.v1.rs:1354-1361`). A gRPC service cannot be
   deployed without hand-writing a config file, while both ports next to it get all three
   input channels.

4. **[consistency] Missing required fields in non-interactive mode classify as
   `command_error`, not `missing_required_input` — and `invalid_input` is never emitted
   by anything.** `invalid_deploy_config_error` stringifies the typed errors into a bare
   `anyhow!` (`create.rs:325-327`) that `classify()` cannot recognize
   (`errors.rs:93-103`); sibling commands route missing inputs through
   `MissingRequiredInput` (`prompts.rs:21-27`, `output.rs:323-330`). Meanwhile
   `ErrorCode::InvalidInput` is `#[allow(dead_code)]` (`errors.rs:54-56`) despite being
   documented in the global taxonomy (`cli.rs:56`). CI consumers keying on `code` get the
   fallback for the most automatable failure this command has.

5. **[docs] LONG_ABOUT claims `--qos-version` is a required deployment field, but
   flag-only mode silently defaults it to `0.12.1`.** `create.rs:29-35` lists it among
   required flags; `flag_only_template()` seeds `DEFAULT_QOS_VERSION`
   (`config/deploy.rs:24,127`) which passes validation, and the missing-fields test
   deliberately omits it (`create.rs:644-651`). Either the doc or the default is wrong.

6. **[docs] LONG_ABOUT says prompts fill missing values "when stdin is a TTY", but no TTY
   check exists.** The split is purely the non-interactive flag (`create.rs:162`,
   `output.rs:209-210`), so piped stdin without `--non-interactive` attempts inquire
   prompts and dies on the raw inquire error rather than the precise missing-fields bail
   promised at `create.rs:47-50` (same claim at `cli.rs:41-42`).

7. **[capability] Pull-secret encryption only works against the four hardcoded API base
   URLs; there is no escape hatch.** `encryption_public_key_for_api_base_url` bails on
   anything else (`pull_secret.rs:18-39`), so with a custom `TVC_API_BASE_URL` the
   `--pivot-pull-secret` flag is unusable; the only workaround is producing the encrypted
   value out-of-band and pasting it into `pivotContainerEncryptedPullSecret` — there is no
   flag/env to pass a pre-encrypted secret or override the encryption key.

8. **[consistency] An interactive typo hard-bails instead of re-prompting.** A non-UUID
   answer to the App ID prompt becomes a non-placeholder `InvalidAppId` on the next
   validate pass (`config/deploy.rs:225-228`) and exits the loop as a fatal error
   (`create.rs:196-199`), discarding every other answer the user just typed.

9. **[consistency] Validation errors name snake_case struct fields that match neither the
   config file's camelCase keys nor the flags.** "app_id contains placeholder value ..."
   (`config/deploy.rs:220-252`) vs file key `appId` / flag `--app-id`; the pull-secret
   error alone names a flag (`config/deploy.rs:273-277`). `missing_required_fields()`,
   which produces proper flag names, is dead outside a test (`config/deploy.rs:194-212`).

10. **[capability] A config file's `pivotArgs` can be replaced but never cleared from the
    CLI.** Empty `Vec` means "flag not provided" (`create.rs:306-308`), so there is no
    flag/env spelling for "deploy with zero pivot args" once the file sets some.

11. **[capability] Debug-mode compatibility is not pre-checked despite the app being
    fetched.** `fetch_tvc_app` discards the response (`create.rs:412`) although `TvcApp`
    carries `enable_debug_mode_deployments`
    (`client/src/generated/external.data.v1.rs:708`); a debug deploy against a non-debug
    app fails only server-side at the create activity, after image validation.
