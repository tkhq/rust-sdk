# tvc app init

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the app init command.*

## Purpose (informative)

`tvc app init` writes a command config that `tvc app create` consumes.
By default the file is a placeholder template with `<FILL_IN_...>` markers for the user to edit by hand.
With `--interactive` the command prompts for each placeholder field and writes a filled command config.
The command runs once per new app, before `tvc app create`.
It needs no login, no network access, and no device.

## Acceptance scenario (normative)

Start in an empty directory, with a `HOME` that holds no tvc config.

| Step | Action | Expected observation |
|---|---|---|
| 1 | Run `tvc app init`. | stdout carries `Created app config template: app.json` plus the edit hint (Outputs); exit code 0. |
| 2 | Read `app.json`. | The file equals the template JSON in Outputs: `name` = `<FILL_IN_APP_NAME>`, `quorumPublicKey` = the well-known quorum key, `shareSetParams` = `null`. |
| 3 | Run `tvc app init --output app2.json --message-format json`. | stdout carries one NDJSON object: `reason` = `app_config_created`, `command` = `app init`, `path` = `app2.json`, `template` = `true`, `interactive` = `false`; exit code 0. |
| 4 | Run `tvc app init` again. | stderr carries `File already exists: app.json`; exit code 1; `app.json` is unchanged. |

All steps pass in one run from a clean start.

## Inputs (normative)

| Input | Flag | Env | Config source | Default | Prompted |
|---|---|---|---|---|---|
| Output path | `-o, --output <PATH>` | `TVC_APP_CONFIG_OUT` | none | `app.json` | never (tvc/src/commands/app/init.rs:19-26) |
| Interactive fill | `--interactive` | none | none | off (tvc/src/commands/app/init.rs:30-31) | n/a |
| Operator public key seed | none | none | tvc config: the active org's default operator kind's sole operator record | `<FILL_IN_OPERATOR_PUBLIC_KEY>` (tvc/src/config/app.rs:81-83) | with `--interactive` only |
| App name | none | none | none | `<FILL_IN_APP_NAME>` (tvc/src/config/app.rs:72) | with `--interactive` only |
| Manifest set name | none | none | none | `<FILL_IN_MANIFEST_SET_NAME>` (tvc/src/config/app.rs:77) | with `--interactive` only |
| Quorum public key | none | none | none | `KNOWN_QUORUM_KEY`, well known and insecure (tvc/src/config/app.rs:66,73) | never |
| Manifest threshold and operator count | none | none | none | threshold 1, one operator named `operator-1` (tvc/src/config/app.rs:78-80) | never |
| Share set | none | none | none | `shareSetParams` = `null`; `app create` later resolves that to the dev known share set (tvc/src/config/app.rs:88,213-223) | never |
| `enableEgress`, `dangerousEnableDebugModeDeployments` | none | none | none | `false` (tvc/src/config/app.rs:74,89) | never |

The command MUST resolve the output path in the Part 00 value resolution order (tvc/src/commands/app/init.rs:19-26).
No config file source exists for the output path.

The operator public key seed comes from tvc config state only.
No flag, environment variable, or prompt selects which operator record seeds the template (tvc/src/operator.rs:282-304).
The seed read is best effort (tvc/src/operator.rs:282-304).
The `local` operator kind reads the sole registered key file.
The `hosted` kind joins the sole record's stored key halves.
The `yubikey` kind reads the cached registry key and does not touch the device.
On any miss the template MUST keep the `<FILL_IN_OPERATOR_PUBLIC_KEY>` placeholder (tvc/src/config/app.rs:81-83).

## Interactive behavior (normative)

Without `--interactive` the command MUST NOT prompt.
The command is safe in non-interactive mode and in JSON mode.

With `--interactive` the command MUST apply this gate before any other work, including the output path check (tvc/src/commands/app/init.rs:36-42).

1. In non-interactive mode the command MUST fail with `--interactive conflicts with --non-interactive or TVC_NON_INTERACTIVE=true` (tvc/src/prompts.rs:29-31; test tvc/tests/non_interactive.rs:234-249). JSON mode forces non-interactive mode (INV-G1, tvc/src/output.rs:210), so the same failure applies there.
2. Otherwise stdin MUST be a TTY. A piped stdin fails with `--interactive requires a TTY` (tvc/src/prompts.rs:33-38).

After the gate, the command checks the output path, reads the seed, and then prompts (tvc/src/commands/app/init.rs:46-59).
The placeholders drive the prompts (tvc/src/config/app.rs:114-141), in this order:

1. `App name` (tvc/src/config/app.rs:115-117).
2. `Manifest set name` (tvc/src/config/app.rs:119-121).
3. One `Operator '<name>' public key` prompt per placeholder operator; the seed is the prompt default (tvc/src/config/app.rs:122-127).

Share set prompts exist in the code (tvc/src/config/app.rs:129-139).
They are unreachable from a fresh template, which sets `shareSetParams` to `null`.
An empty or whitespace-only answer MUST fail with `<prompt> cannot be empty` (tvc/src/prompts.rs:42-48).
The command MUST NOT prompt for the quorum public key, threshold, operator count, share set, `enableEgress`, or `dangerousEnableDebugModeDeployments`.

## Outputs (normative)

### Human mode

Without `--interactive` the command MUST print this exact stdout message, with both `<path>` occurrences replaced (tvc/src/commands/app/init.rs:93-102):

```
Created app config template: <path>

Edit the file to fill in your values, then run:
  tvc app create --config-file <path>
```

With `--interactive` the command MUST print this shorter message, which drops the edit hint (tvc/src/commands/app/init.rs:85-92):

```
Created app config: <path>

Run: tvc app create --config-file <path>
```

### JSON mode

The command MUST emit exactly one NDJSON object (test tvc/tests/message_format.rs:60-86).

| Field | Value |
|---|---|
| `reason` | `app_config_created` (tvc/src/outcome.rs:30,54) |
| `command` | `app init` (tvc/src/commands/app/init.rs:68) |
| `path` | The output path, rendered as given. |
| `template` | `true` when `--interactive` is absent (tvc/src/commands/app/init.rs:70). |
| `interactive` | `false`, always, in JSON mode. |

`template` MUST equal the negation of `interactive` (INV-3).
The `interactive` = `true` shape is unreachable in JSON mode: forced non-interactive mode conflicts with `--interactive` (INV-G1; tvc/src/commands/app/init.rs:36-39).

### The written file

From a clean start with no seed, the written file MUST hold exactly this pretty printed JSON.
The template builds at tvc/src/config/app.rs:70-91 and serializes at tvc/src/commands/app/init.rs:61.

```json
{
  "name": "<FILL_IN_APP_NAME>",
  "quorumPublicKey": "04451028fc9d42cef6d8f2a3ebe17d65783c470dbc6f04663d500c12009930cf9b209e733f6ac6103cc28f07ecde2dbb55095738b828d6b7a55caf4ddf9d67f2ae047827dcd2325b8d58694c2ea14e8f1e1f8a36c84438d291ff9b1b067debdb3e2ba3822984cde8bed4de2c237bd323526da4961d368bcc63cbd2d37d00e936683e",
  "enableEgress": false,
  "manifestSetId": null,
  "manifestSetParams": {
    "name": "<FILL_IN_MANIFEST_SET_NAME>",
    "threshold": 1,
    "newOperators": [
      {
        "name": "operator-1",
        "publicKey": "<FILL_IN_OPERATOR_PUBLIC_KEY>"
      }
    ],
    "existingOperatorIds": []
  },
  "shareSetId": null,
  "shareSetParams": null,
  "dangerousEnableDebugModeDeployments": false
}
```

When the seed read succeeds, the operator `publicKey` MUST hold the seed value in place of the placeholder (tvc/src/config/app.rs:81-83).
With `--interactive`, the prompted answers MUST replace the placeholder values and every other field MUST stay as above (tvc/src/config/app.rs:109-141).

## Side effects (normative)

- The command MUST create the output file, and MUST NOT overwrite an existing one (tvc/src/commands/app/init.rs:48-50,64-65).
- The write is the last step (tvc/src/commands/app/init.rs:36-65). A failure at the gate, the path check, or a prompt MUST leave no file behind.
- Dispatch loads the tvc config first and creates it when absent, per INV-G4 (tvc/src/cli.rs:219-223).
- The seed read MAY read the sole local operator record's key file or the cached yubikey registry entry (tvc/src/operator.rs:282-304).
- The command MUST NOT call the Turnkey API, submit an activity, or open a device.

## Failure modes (normative)

Every runtime failure of this command MUST classify as JSON `code` = `command_error` with exit code 1.
`classify` recognizes only typed errors, which this command never produces (tvc/src/errors.rs:93-103).
In human mode the error renders on stderr; in JSON mode it renders as one NDJSON object on stdout (INV-G3).

| Failure | Trigger | Message | JSON `code` | Exit code |
|---|---|---|---|---|
| Output file exists | The output path exists before the write. | `File already exists: <path>` (tvc/src/commands/app/init.rs:48-50) | `command_error` | 1 |
| Interactive conflict | `--interactive` in non-interactive mode. | `--interactive conflicts with --non-interactive or TVC_NON_INTERACTIVE=true` (tvc/src/prompts.rs:29-31) | `command_error` | 1 |
| No TTY | `--interactive` with a non-TTY stdin. | `--interactive requires a TTY` (tvc/src/prompts.rs:33-38) | `command_error` | 1 |
| Empty prompt answer | Empty or whitespace-only submission. | `<prompt> cannot be empty` (tvc/src/prompts.rs:42-48) | `command_error` | 1 |
| Serialize failure | `serde_json` fails on the config. | `failed to serialize config` (tvc/src/commands/app/init.rs:61) | `command_error` | 1 |
| Write failure | The output path is unwritable. | `failed to write file: <path>` (tvc/src/commands/app/init.rs:64-65) | `command_error` | 1 |
| Unknown flag or argument | clap parse failure. | clap usage text; `usage_error` NDJSON in JSON mode (tvc/src/cli.rs:154-176) | `usage_error` | 2 |

## Test vectors (normative)

Every vector starts in an empty directory, with a `HOME` that holds no tvc config, unless its Given says otherwise.
Vector comparison excludes only the Part 00 global nondeterministic fields; this command adds none.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | Clean start. | `tvc app init` | `app.json` equals the template JSON in Outputs; stdout is the human template message with `<path>` = `app.json`; exit code 0 (tvc/src/commands/app/init.rs:93-102). |
| V-2 | Clean start. | `tvc app init --message-format json --output app2.json` | One NDJSON object: `reason` = `app_config_created`, `command` = `app init`, `path` = `app2.json`, `template` = `true`, `interactive` = `false`; exit code 0 (test tvc/tests/message_format.rs:60-86). |
| V-3 | `app.json` already exists. | `tvc app init` | stderr contains `File already exists: app.json`; exit code 1; the file is unchanged (tvc/src/commands/app/init.rs:48-50). |
| V-4 | Env `TVC_NON_INTERACTIVE=1`. | `tvc app init --interactive` | stderr contains `--interactive conflicts with --non-interactive or TVC_NON_INTERACTIVE=true`; exit code 1; no file written (test tvc/tests/non_interactive.rs:234-249). |
| V-5 | Clean start. | `tvc app init --interactive --message-format json` | One NDJSON error object with `reason` = `command_error`, `code` = `command_error`, `message` containing the conflict text; exit code 1 (tvc/src/output.rs:210; tvc/src/prompts.rs:29-31). |
| V-6 | stdin is a pipe, no non-interactive flag or env. | `tvc app init --interactive < /dev/null` | stderr contains `--interactive requires a TTY`; exit code 1; no file written (tvc/src/prompts.rs:33-38). |
| V-7 | stdin is a TTY; the user submits an empty line at `App name`. | `tvc app init --interactive` | stderr contains `App name cannot be empty`; exit code 1; no file written (tvc/src/prompts.rs:42-48; tvc/src/config/app.rs:115-117). |
| V-8 | stdin is a TTY; answers `my-app`, `my-manifest-set`, `04deadbeef04c0ffee`. | `tvc app init --interactive` | `app.json` holds `name` = `my-app`, `manifestSetParams.name` = `my-manifest-set`, `newOperators[0].publicKey` = `04deadbeef04c0ffee`; stdout is the interactive message with `<path>` = `app.json`; exit code 0 (tvc/src/commands/app/init.rs:85-92; tvc/src/config/app.rs:114-141). |
| V-9 | Clean start. | `tvc app init --name my-app --output my-app.json` | clap usage error on stderr; exit code 2; no file written (tvc/src/commands/app/init.rs:17-32; tvc/src/cli.rs:154-176; tvc/README.md:46 documents this invocation, see Gap 6). |
| V-10 | Env `TVC_APP_CONFIG_OUT=env-app.json`. | `tvc app init` | `env-app.json` is written; `path` in the outcome is `env-app.json`; exit code 0 (tvc/src/commands/app/init.rs:19-26). |
| V-11 | Env `TVC_APP_CONFIG_OUT=env-app.json`. | `tvc app init --output flag-app.json` | `flag-app.json` is written and `env-app.json` is absent: the flag outranks the environment variable (tvc/src/commands/app/init.rs:19-26; tvc/src/cli.rs:19). |
| V-12 | tvc config with an active org, default operator kind `hosted`, and one hosted operator record with `encrypt_public_key` = `04aa11`, `sign_public_key` = `04bb22`. | `tvc app init` | `app.json` holds `manifestSetParams.newOperators[0].publicKey` = `04aa1104bb22`; exit code 0 (tvc/src/operator.rs:291-297). |

## Invariants (normative)

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The command MUST NOT overwrite an existing file at the output path. | Existence check before any write (tvc/src/commands/app/init.rs:48-50). Behavioral check: V-3. |
| INV-2 | Without `--interactive` the command MUST NOT prompt. | The only prompt calls live in `fill_interactively`, whose sole call site is gated on `args.interactive` (tvc/src/commands/app/init.rs:57-59; tvc/src/config/app.rs:114-141). Behavioral check: V-2 completes with a piped stdin. |
| INV-3 | In every outcome `template` MUST equal the negation of `interactive`. | Both fields derive from the one `args.interactive` value at a single construction site (tvc/src/commands/app/init.rs:67-72). Behavioral check: tvc/tests/message_format.rs:84-85. |
| INV-4 | The command MUST NOT call the Turnkey API or open a device. | `run` receives only the output context, the parsed args, and the tvc config; the module imports no client or device type (tvc/src/commands/app/init.rs:1-12,35). Behavioral check: V-1 succeeds with no credentials and no network. |

## Gaps (informative)

1. **[capability]** The operator key seed always comes from the sole operator record of the active org's default operator kind.
The user cannot pick which operator record seeds the config.
`default_operator_public_key` consults only the default operator kind and then the sole record of that kind (tvc/src/operator.rs:282-304).
`select_local_operator` errors when the org holds two local operator records (tvc/src/config/turnkey.rs:452-456).
A second local operator record therefore makes the prefill vanish, and the placeholder appears with no warning.
No `--operator` or `--operator-public-key` flag exists, and no selection prompt exists.
Contrast `app create`, which enumerates every known candidate and prompts for a choice (tvc/src/commands/app/create.rs:71-96).
This is the "state or default silently constrains an explicit choice" shape from the audit brief.
The only escape is to paste a key by hand in `--interactive` mode.

2. **[capability]** No input supplies the quorum public key; the template defaults to the well-known insecure key, even in interactive mode.
The template hard codes `KNOWN_QUORUM_KEY` (tvc/src/config/app.rs:73).
The code documents that constant as "for applications that do not need secure quorum keys" (tvc/src/config/app.rs:65-66).
The value carries no `<FILL_IN` marker, so `fill_interactively` never prompts for it (tvc/src/config/app.rs:114-141).
No flag exists.
Neither the help text nor the success message warns that the default quorum key is public.
The interactive path then declares the command config done, with the insecure key baked in (tvc/src/commands/app/init.rs:85-92).
Meanwhile `keys generate-local-quorum-key` exists to mint a real one.

3. **[capability]** No flag or prompt leads to a production share set; the template commits the app to the dev known share set.
The template writes `shareSetParams: null` (tvc/src/config/app.rs:88).
`app create` resolves that to `dev-known-share-set`, built from `KNOWN_SHARE_SET_KEYS` (tvc/src/config/app.rs:94-107,213-223).
The doc comment on those keys treats the secrets as well known (tvc/src/config/app.rs:52-63).
`--interactive` never asks, because only pre-existing placeholders get prompts, so a secure share set requires hand writing the JSON shape with no scaffold.
The same holds for the threshold (fixed 1), the operator count (one operator named `operator-1`), and `enableEgress` (tvc/src/config/app.rs:70-91).
Interactive mode cannot change any of them.

4. **[consistency]** No `--from-app` seeding exists, unlike `deploy init --from-deployment`.
`deploy init` fetches an existing deployment and copies every recoverable field (tvc/src/commands/deploy/init.rs:41-42,88-99).
`app init` has no equivalent to regenerate a command config from an existing app.
The API already exposes app data that `app list` and `app status` consume.

5. **[consistency]** The fixed default filename guarantees a collision on the second run.
`app init` defaults to `app.json` and bails when the file exists (tvc/src/commands/app/init.rs:23,48-50).
`deploy init` defaults to a timestamped `deploy-<ts>.json` that never collides (tvc/src/commands/deploy/init.rs:71-74).
Neither command offers `--force`, and only the `app init` default makes the error the norm.
`keys init-local-quorum-key` shares the fixed-name shape (tvc/src/commands/keys/init_local_quorum_key.rs:22,30-32).

6. **[docs]** The README documents a `--name` flag that does not exist.
`tvc/README.md:46` shows `tvc app init --name my-app --output my-app.json`.
`Args` has only `--output` and `--interactive` (tvc/src/commands/app/init.rs:17-32), so the documented invocation exits 2 with a usage error (V-9).
The app name is the first value the template asks the user to fill in, so a `--name` flag is also a plausible capability addition.

7. **[consistency]** `--interactive` exists on `app init` and `deploy init`, is missing from `keys init-local-quorum-key`, and has no env var anywhere.
The three template generators diverge: the quorum key init has no interactive fill at all (tvc/src/commands/keys/init_local_quorum_key.rs:16-26).
`--interactive` lacks an env or config equivalent, while its sibling input `--output` has `TVC_APP_CONFIG_OUT` (tvc/src/commands/app/init.rs:18-31).
This is minor and still breaks the flag, env, config resolution uniformity that the LONG_ABOUT advertises (tvc/src/cli.rs:18-23).

8. **[consistency]** Usage-shaped errors classify as `command_error` with exit 1 in place of `usage_error` with exit 2.
The `--interactive` conflict is a plain `bail!` in command code (tvc/src/prompts.rs:29-31), where a clap `conflicts_with` would classify it during parsing.
Semantic failures like "File already exists" cannot reach `invalid_input`, because `ErrorCode::InvalidInput` is dead code that nothing produces (tvc/src/errors.rs:54-56,93-103).
JSON consumers therefore see the fallback code for failures the help text taxonomy describes as distinct classes (tvc/src/cli.rs:51-64).
`deploy init` shares this gap.

9. **[bug?]** Interactive fill accepts any non-empty text as an operator public key.
`fill_interactively` uses `required_text` with no format check (tvc/src/config/app.rs:122-127).
An `OperatorPublicKey` parser exists (tvc/src/operator.rs:63), and the `app create` reuse logic parses candidate keys with it (tvc/src/operator.rs:458-460).
A typo'd key also survives `app create` validation, which checks only placeholders and thresholds (tvc/src/config/app.rs:160-211); the server rejects it first.
