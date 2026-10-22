# tvc deploy create

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the deploy create command.*

## Purpose (informative)

`tvc deploy create` creates a TVC deployment for an app.
It resolves a deployment configuration from a command config (JSON file), flags, and env vars.
It checks that the app exists, then validates the container image server side.
With `--pivot-pull-secret` it encrypts a pivot pull secret for the active org's API environment.
It pins the image URL to the resolved digest and submits the `create_tvc_deployment` activity.
Typical use: after `tvc deploy init` and an edit of the generated command config, or fully flag driven in CI.

Citations shorten paths under `tvc/src/`: `create.rs` means `tvc/src/commands/deploy/create.rs`.
Other cited files keep their `tvc/src/` relative path, and generated files carry full paths.
Dispatch: cli.rs:260-262, with `long_about = LONG_ABOUT` (create.rs:22-65) and `after_help = PORT_GUIDANCE` (commands/deploy.rs:17-23, cli.rs:406-411).

## Acceptance scenario (normative)

| Step | Action | Expected observation |
|---|---|---|
| 1 | Write `deploy.json`: `appId` = `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`, `qosVersion` = `0.12.1`, `pivotContainerImageUrl` = `ghcr.io/acme/pivot:v1`, `pivotPath` = `/usr/bin/pivot`, `pivotArgs` = `[]`, `expectedPivotDigest` = `sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`, `healthCheckType` = `TVC_HEALTH_CHECK_TYPE_HTTP`, `healthCheckPort` = `3000`, `publicIngressPort` = `3000`. | The file parses as a `DeployConfig` (config/deploy.rs:27-51). |
| 2 | Run `tvc deploy create --config-file deploy.json --message-format json`. | No prompt appears (INV-G1). |
| 3 | Read stdout. | Exactly one NDJSON object: `reason` = `deployment_created`, `appId` = `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`, `configPath` = `deploy.json`, `pinnedImageUrl` = `ghcr.io/acme/pivot:v1@` plus the resolved digest, and a `deploymentId` (create.rs:460-480, outcome.rs:46). |
| 4 | Check the exit code. | 0 (cli.rs:120). |

All steps pass in one run from a clean start with a logged-in profile whose active org owns the app.

## Inputs (normative)

| Input | Flag | Env var | Command config key | Default | Prompt |
|---|---|---|---|---|---|
| command config path | `-c, --config-file` | `TVC_DEPLOY_CONFIG` | (none) | none | never |
| app id | `--app-id` | `TVC_APP_ID` | `appId` | none | yes; default is the saved last app id (config/turnkey.rs:710) |
| QOS version | `--qos-version` | `TVC_QOS_VERSION` | `qosVersion` | `0.12.1` (config/deploy.rs:24,127) | only while the field holds a placeholder |
| pivot image URL | `--pivot-image-url` | `TVC_PIVOT_IMAGE_URL` | `pivotContainerImageUrl` | none | yes |
| pivot path | `--pivot-path` | `TVC_PIVOT_PATH` | `pivotPath` | none | yes |
| expected pivot digest | `--expected-pivot-digest` | `TVC_EXPECTED_PIVOT_DIGEST` | `expectedPivotDigest` | none | yes |
| pivot args | `--pivot-args` (comma split) | `TVC_PIVOT_ARGS` | `pivotArgs` | `[]` | never |
| debug mode | `--dangerous-deploy-debug-mode` | `TVC_DANGEROUS_DEPLOY_DEBUG_MODE` | `dangerousDeployDebugMode` | `false` | never |
| health check port | `--health-check-port` | `TVC_HEALTH_CHECK_PORT` | `healthCheckPort` | `3000` | never |
| public ingress port | `--public-ingress-port` | `TVC_PUBLIC_INGRESS_PORT` | `publicIngressPort` | `3000` | never |
| replicas | `--replicas` | `TVC_REPLICAS` | `replicas` | none (backend default) | never |
| health check type | (none) | (none) | `healthCheckType` | `TVC_HEALTH_CHECK_TYPE_HTTP` | never |
| pull secret plaintext path | `--pivot-pull-secret` | `TVC_PIVOT_PULL_SECRET` | `pivotContainerEncryptedPullSecret` (holds a pre-encrypted value; never a path) | none | public registry question only; the path itself is never prompted |
| auth | (none) | `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, `TVC_API_KEY_PRIVATE`, `TVC_API_BASE_URL` | tvc config active org (written by `tvc login`) | prod API base URL (client.rs:40) | never |

The command MUST resolve each value in the Part 00 order: flag, env var, command config, built-in default (create.rs:290-323; tests create.rs:593-623).
Three exceptions apply. The first two match the Part 00 exceptions (cli.rs:25-32).

- `--pivot-args` MUST replace the command config list and MUST NOT append (create.rs:306-308; test create.rs:653-667).
- `--dangerous-deploy-debug-mode` MUST only turn debug mode on. An absent flag MUST NOT clear a `true` command config value (create.rs:309-312; tests create.rs:671-706).
- An absent `--replicas` flag MUST NOT clear a command config value (create.rs:319-322; test create.rs:800-818).

Without `--config-file`, the command MUST seed missing optional fields from the flag-only template (create.rs:282-288).
The seeded values: QOS version `0.12.1`, both ports `3000`, HTTP health check, empty pivot args, no pull secret (test create.rs:625-641).

## Interactive behavior (normative)

The mode split lives at `run` (create.rs:161-169).
The command MUST enter non-interactive mode when `--non-interactive` is set, when `TVC_NON_INTERACTIVE` parses true, or in JSON mode (output.rs:209-210, INV-G1).
The command performs no TTY detection on any path; contrast `deploy init --interactive`, which calls `ensure_stdin_is_tty` (commands/deploy/init.rs:57). See gap 6.

In interactive mode the command MUST loop: validate the resolved config, then fill placeholder fields (create.rs:194-206, config/deploy.rs:145-179).
It MUST prompt only for fields that still hold a `<FILL_IN...>` placeholder, in this order:

1. `App ID`, with the saved last app id as the default when one exists (config/deploy.rs:150-151, config/turnkey.rs:710).
2. `QOS version` (config/deploy.rs:153-154).
3. `Pivot container image URL` (config/deploy.rs:156-158).
4. `Pivot path (inside container)` (config/deploy.rs:160-161).
5. `Expected pivot digest (sha256:...)` (config/deploy.rs:163-165).
6. When the pull-secret field equals the sentinel `<REMOVE_ME_IF_PIVOT_CONTAINER_URL_IS_PUBLIC>`: `Is the container image in a public registry?`, default Yes (config/deploy.rs:167-177).

Either answer to the public registry question MUST clear the field.
On No the command MUST print the note that points at `--pivot-pull-secret` (config/deploy.rs:170-176).
An empty answer to a text prompt fails the command with `<prompt> cannot be empty` (prompts.rs:42-48).
A validation error other than a placeholder MUST end the command, in interactive mode too (create.rs:196-199). See gap 8.

When at least one prompt ran and a command config path exists, the command MUST offer to save (create.rs:208-210).
The offer reads `Save filled config to <path>?`, default Yes (create.rs:332-343).
On Yes it MUST write the filled command config to that path and print `Wrote <path>` (create.rs:343-348).

In non-interactive mode the command MUST NOT prompt and MUST NOT write the command config (INV-1; test create.rs:934-957).
A validation failure MUST name every offending field in one error (create.rs:240-242, config/deploy.rs:306-317).
A leftover pull-secret sentinel MUST fail with the message that names `--pivot-pull-secret` (config/deploy.rs:254-258, config/deploy.rs:273-277). See gap 1.

## Outputs (normative)

In human mode the command MUST print, in order:

1. `Creating deployment for app '<app id>'...` (create.rs:401-405).
2. The port summary block, then one blank line (commands/deploy.rs:25-43, create.rs:406-407). The shared-port shape appears when both ports match; the two-line shape appears otherwise.
3. `Using pinned image reference for deployment request: <pinned url>`, only when pinning changed the URL (create.rs:436-441).
4. The success block (create.rs:482-508):

```
Deployment created successfully!

Deployment ID: <deployment id>
App ID: <app id>
Config: <path>

Next steps:
  - Run `tvc deploy status --deploy-id <deployment id>` to check deployment status
  - Run `tvc deploy approve --deploy-id <deployment id> --operator-id <operator-id>` to approve the manifest
```

The `Config:` line MUST appear only when the run used a command config path (create.rs:494-496).

In JSON mode the command MUST emit exactly one terminal NDJSON object with `reason` = `deployment_created` (outcome.rs:30, outcome.rs:46).
Its fields: `deploymentId`, `appId`, `pinnedImageUrl`, and `configPath` (create.rs:471-480).
`configPath` MUST appear only when the run used a command config (create.rs:477-479).
Progress lines MUST NOT appear in JSON mode (output.rs:242-249).
Errors follow the Part 00 error taxonomy and INV-G3.

## Side effects (normative)

- Reads: the command config, the pull-secret plaintext file, and the tvc config.
- Writes: the interactive save offer is the only write to the command config (INV-1). Dispatch creates the tvc config when absent, per INV-G4 (cli.rs:219-223). The command MUST NOT update the saved last app id; only `tvc app create` writes it (commands/app/create.rs:272).
- Local crypto: the command MUST HPKE-encrypt a pull-secret plaintext before any other use (pull_secret.rs:42-66). The target is the hardcoded public key for the org's API base URL (pull_secret.rs:8-39).
- Turnkey API calls MUST run in this order: `get_tvc_app`, `validate_tvc_image`, then the `create_tvc_deployment` activity (create.rs:412, create.rs:425-429, create.rs:454-458). The app fetch is an existence check; the command discards the response (client.rs:67-80). Image validation also resolves the digest for pinning (create.rs:431-434).
- The command MUST NOT touch YubiKeys or other devices.

## Failure modes (normative)

On each runtime failure below the command MUST exit 1 and, in JSON mode, MUST emit one error object per INV-G3.

| Failure | Observable behavior | `code` | Exit |
|---|---|---|---|
| Bad flags or args | clap usage error; JSON path per GV-1 (cli.rs:154-182) | `usage_error` | 2 |
| Command config path unreadable | `failed to read config file: <path>` (create.rs:256-258) | `command_error` | 1 |
| Command config unparseable | `failed to parse config file: <path>` (create.rs:184-185, create.rs:233-234) | `command_error` | 1 |
| Validation failure: placeholders, non-UUID app id, or pull-secret sentinel | `invalid deploy config: <error>; <error>; ...` (create.rs:325-327, config/deploy.rs:264-278) | `command_error` (gap 4) | 1 |
| Pull-secret file empty after trim | `pivot pull secret file is empty after trimming whitespace: <path>` (create.rs:272-277) | `command_error` | 1 |
| Unsupported API base URL for pull-secret encryption | `unsupported API base URL for pivot pull secret encryption key inference: '<url>'. expected one of: ...` (pull_secret.rs:25-35) | `command_error` | 1 |
| App does not exist in the org | `MissingResource` from `fetch_tvc_app` (client.rs:77-80, errors.rs:95-96) | `not_found` | 1 |
| Image validation or create activity fails | context `failed to validate TVC image` (create.rs:429) or `failed to create TVC deployment` (create.rs:458); classified from the HTTP or activity error | `api_error`, `unauthorized`, `network_error`, ... | 1 |
| Debug deploy against an app created without `--dangerous-enable-debug-mode-deployments` | the server rejects the create activity (gap 11) | `api_error` | 1 |

## Test vectors (normative)

Vector comparison excludes the Part 00 global nondeterministic fields, plus `deploymentId` and the server-resolved digest inside `pinnedImageUrl`.
`deploy.json` below means the acceptance scenario file unless a row changes a field.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | `deploy.json`; a logged-in profile whose active org owns app `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` | `tvc deploy create --config-file deploy.json --message-format json` | One NDJSON object: `reason` = `deployment_created`, `appId` = `6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b`, `configPath` = `deploy.json`, `pinnedImageUrl` = `ghcr.io/acme/pivot:v1@<resolved digest>`; exit 0 (create.rs:460-480, outcome.rs:46). |
| V-2 | no command config | `tvc deploy create --app-id 6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b --pivot-image-url ghcr.io/acme/pivot:v1 --pivot-path /usr/bin/pivot --expected-pivot-digest sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 --non-interactive` | Resolution fills template defaults: `qosVersion` `0.12.1`, ports `3000`/`3000`, HTTP health check, empty `pivotArgs`, no pull secret (test create.rs:625-641); human success block per Outputs (create.rs:482-508) with no `Config:` line; exit 0. |
| V-3 | `deploy.json` with `pivotArgs` = `["a","b"]` | `tvc deploy create --config-file deploy.json --pivot-args c --non-interactive` | The resolved pivot args equal `["c"]`; no append (create.rs:306-308; test create.rs:653-667). |
| V-4 | `deploy.json` with `dangerousDeployDebugMode` = `true` | `tvc deploy create --config-file deploy.json --non-interactive` (no debug flag) | The resolved debug mode stays `true` (create.rs:309-312; test create.rs:687-706). |
| V-5 | no command config, no override flags | `tvc deploy create --non-interactive` | Error `invalid deploy config: app_id contains placeholder value <FILL_IN_APP_ID>; pivot_container_image_url contains placeholder value <FILL_IN_PIVOT_CONTAINER_IMAGE_URL>; pivot_path contains placeholder value <FILL_IN_PIVOT_PATH>; expected_pivot_digest contains placeholder value <FILL_IN_EXPECTED_PIVOT_DIGEST>`; `qos_version` absent from the list (gap 5); `code` = `command_error` (gap 4); exit 1 (create.rs:325-327; test create.rs:643-651). |
| V-6 | `deploy.json` with `pivotContainerEncryptedPullSecret` = `<REMOVE_ME_IF_PIVOT_CONTAINER_URL_IS_PUBLIC>` | `tvc deploy create --config-file deploy.json --non-interactive` | Error `invalid deploy config: pivotContainerEncryptedPullSecret contains placeholder value <REMOVE_ME_IF_PIVOT_CONTAINER_URL_IS_PUBLIC>; pass --pivot-pull-secret <PATH> or remove pivotContainerEncryptedPullSecret for public images`; exit 1 (config/deploy.rs:273-277; test create.rs:712-732). |
| V-7 | `deploy.json` with `appId` = `not-a-uuid` | `tvc deploy create --config-file deploy.json --non-interactive` | Error `invalid deploy config: app_id is not a valid UUID: not-a-uuid`; exit 1 (config/deploy.rs:271-272; test config/deploy.rs:533-542). |
| V-8 | no file exists at `/tmp/absent.json` | `tvc deploy create --config-file /tmp/absent.json --non-interactive` | Error `failed to read config file: /tmp/absent.json`; `code` = `command_error`; exit 1 (create.rs:256-258; gap 2). |
| V-9 | `deploy.json`; `secret.txt` contains only whitespace | `tvc deploy create --config-file deploy.json --pivot-pull-secret secret.txt --non-interactive` | Error `pivot pull secret file is empty after trimming whitespace: secret.txt`; exit 1 (create.rs:272-277). |
| V-10 | active org API base URL `https://api.example.test`; `secret.txt` nonempty | `tvc deploy create --config-file deploy.json --pivot-pull-secret secret.txt --non-interactive` | Error `unsupported API base URL for pivot pull secret encryption key inference: 'https://api.example.test'. expected one of: 'http://localhost:8081', 'https://api.dev.turnkey.engineering', 'https://api.preprod.turnkey.engineering', 'https://api.turnkey.com'`; exit 1 (pull_secret.rs:25-35; gap 7). |
| V-11 | `deploy.json` with an `appId` absent from the org | `tvc deploy create --config-file deploy.json --non-interactive` | `code` = `not_found`; exit 1 (client.rs:77-80, errors.rs:95-96). |
| V-12 | `deploy.json` replaced by the untouched `deploy init` template (all `<FILL_IN...>` placeholders, no pull-secret sentinel) | V-2's invocation plus `--config-file deploy.json --qos-version 0.12.1` | Resolution completes from the flags; the file on disk keeps its placeholders, byte for byte (INV-1; test create.rs:934-957). |

## Invariants (normative)

Global invariants INV-G1 through INV-G4 apply. Per-command invariants:

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | In non-interactive mode the command MUST NOT write the command config. | The save offer exists only on the interactive path (create.rs:208-210); `build_inputs_non_interactive` contains no write (create.rs:220-249). Behavioral check: create.rs:934-957. |
| INV-2 | The submitted `pivot_container_image_url` MUST carry an `@` digest suffix. | `pin_image_url` appends the server-resolved digest to any URL without `@`, on every path (create.rs:386-392, create.rs:431-434). Behavioral checks: create.rs:520-530. |
| INV-3 | The pull-secret plaintext MUST NOT enter an API request or a file; only HPKE ciphertext leaves the process. | The sole consumer of the plaintext is `encrypt_pivot_pull_secret` (create.rs:414-417); the request builders accept only the ciphertext (create.rs:352-362, create.rs:364-384). Behavioral check: round trip in pull_secret.rs:81-113. |
| INV-4 | Interactive prompts MUST replace only `<FILL_IN...>` placeholder fields and the pull-secret sentinel. | Every `fill_interactively` branch gates on the placeholder check (config/deploy.rs:145-179). Behavioral check: config/deploy.rs:485-506. |

## Gaps (informative)

1. **[bug?] `--pivot-pull-secret` does not satisfy the command config's pull-secret placeholder; non-interactive mode bails telling the user to pass the flag they already passed**.
   `validate()` hard errors on the sentinel (config/deploy.rs:254-258) before the command consults `pivot_pull_secret` (create.rs:240-242).
   The flag's encrypted value only overrides the config field later, at execution (create.rs:414-417).
   An init-generated private-image config plus `--pivot-pull-secret` plus `--non-interactive` therefore fails with `pass --pivot-pull-secret <PATH>`.
   Interactive mode likewise asks `Is the container image in a public registry?` when the user already passed the flag.
   The answer has no effect: both branches clear the field, and the command attaches the flag's secret regardless.

2. **[bug?] The `Write a new config file at {path}?` offer is unreachable; pointing `--config-file` at a nonexistent path errors before any bootstrap can happen**.
   `read_config_file_bytes` propagates the read failure for any provided path (create.rs:256-258).
   `file_loaded` is only `false` when the user gave no path, and the save offer requires a path (create.rs:208-210).
   `offer_to_save_config` therefore always runs with `file_loaded == true`, and the create.rs:341 branch is dead code.
   The prompt wording shows the intended missing capability: bootstrapping a new file at the given path.

3. **[capability] `healthCheckType` is command-config-only: no flag, no env var, no prompt**.
   The `Overrides` struct has no such field (create.rs:88-152), so flag-only mode hard codes HTTP (config/deploy.rs:134).
   The enum also has GRPC (client/src/generated/immutable.common.v1.rs:1354-1361).
   Deploying a gRPC service therefore requires a hand-written command config, while both neighboring ports get all three input channels.

4. **[consistency] Missing required fields in non-interactive mode classify as `command_error`, the taxonomy reserves `missing_required_input` for exactly this case, and nothing emits `invalid_input`**.
   `invalid_deploy_config_error` stringifies the typed errors into a bare `anyhow!` (create.rs:325-327) that `classify()` cannot recognize (errors.rs:93-103).
   Sibling commands route missing inputs through `MissingRequiredInput` (prompts.rs:21-27, output.rs:323-333).
   `ErrorCode::InvalidInput` carries `#[allow(dead_code)]` (errors.rs:54-56) yet appears in the global taxonomy (cli.rs:56).
   CI consumers that key on `code` get the fallback for the most automatable failure this command has.

5. **[docs] LONG_ABOUT lists `--qos-version` among the required deployment fields, yet flag-only mode silently defaults it to `0.12.1`**.
   The help text at create.rs:29-35 lists it as required.
   `flag_only_template()` seeds `DEFAULT_QOS_VERSION` (config/deploy.rs:24,127), which passes validation, and the missing-fields test deliberately omits it (create.rs:643-651).
   Either the doc or the default is wrong.

6. **[docs] LONG_ABOUT says prompts fill missing values "when stdin is a TTY", yet no TTY check exists**.
   The split is purely the non-interactive flag (create.rs:162, output.rs:209-210).
   Piped stdin without `--non-interactive` therefore attempts inquire prompts and dies on the raw inquire error.
   The precise missing-fields bail promised at create.rs:47-50 (same claim at cli.rs:41-42) never runs on that path.

7. **[capability] Pull-secret encryption only works against the four hardcoded API base URLs; no escape hatch exists**.
   `encryption_public_key_for_api_base_url` bails on anything else (pull_secret.rs:18-39), so with a custom `TVC_API_BASE_URL` the `--pivot-pull-secret` flag is unusable.
   The only workaround is producing the encrypted value out of band and pasting it into `pivotContainerEncryptedPullSecret`.
   No flag or env var passes a pre-encrypted secret or overrides the encryption key.

8. **[consistency] An interactive typo hard bails and discards every other answer the user just typed**.
   A non-UUID answer to the App ID prompt becomes a non-placeholder `InvalidAppId` on the next validate pass (config/deploy.rs:225-228).
   The loop exits with a fatal error (create.rs:196-199).
   A re-prompt would preserve the session.

9. **[consistency] Validation errors name snake_case struct fields that match neither the command config's camelCase keys nor the flags**.
   `app_id contains placeholder value ...` (config/deploy.rs:220-252) sits against file key `appId` and flag `--app-id`.
   The pull-secret error alone names a flag (config/deploy.rs:273-277).
   `missing_required_fields()` produces proper flag names and is dead outside a test (config/deploy.rs:194-212).

10. **[capability] The CLI can replace a command config's `pivotArgs` and can never clear them**.
    An empty `Vec` means the flag was absent (create.rs:306-308).
    Once the file sets some args, no flag or env spelling deploys with zero pivot args.

11. **[capability] The command fetches the app and still skips a debug-mode compatibility pre-check**.
    `fetch_tvc_app` discards the response (create.rs:412), yet `TvcApp` carries `enable_debug_mode_deployments` (client/src/generated/external.data.v1.rs:708).
    A debug deploy against a non-debug app fails only server side, at the create activity, after image validation.
