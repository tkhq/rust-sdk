# tvc keys init-local-quorum-key

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the keys init-local-quorum-key command.*

## Purpose (informative)

The command writes a JSON template for the quorum key config that `tvc keys generate-local-quorum-key` consumes.
The user runs it once, then hand-edits the file.
The template carries every field the generator honors: `QuorumKeyConfig` holds exactly `shares`, `threshold`, and `operatorPublicKeys` (tvc/src/config/quorum_key.rs:21-25).
Slot 1 of `operatorPublicKeys` prefills, best effort, with the public key of the active org's default operator record.
Every other value is a fixed 2 of 2 placeholder skeleton.

## Acceptance scenario (normative)

The scenario starts clean: `HOME=/home/op` and the working directory `/home/op/qk`, both empty.

| Step | Action | Expected observation |
|---|---|---|
| 1 | Confirm the clean start. | No file exists at `/home/op/.config/turnkey/tvc.config.toml`; `/home/op/qk` is empty. |
| 2 | Run `tvc keys init-local-quorum-key`. | Exit code 0. stdout carries `Created quorum key config template: quorum_key.json`, the constraints block, and the next step line `tvc keys generate-local-quorum-key --config-file quorum_key.json` (tvc/src/commands/keys/init_local_quorum_key.rs:53-68). |
| 3 | Read `/home/op/qk/quorum_key.json`. | Pretty printed JSON: `shares` = 2, `threshold` = 2, `operatorPublicKeys` = `["<FILL_IN_OPERATOR_PUBLIC_KEY_1>", "<FILL_IN_OPERATOR_PUBLIC_KEY_2>"]` (tvc/src/config/quorum_key.rs:29-40). |
| 4 | Check `/home/op/.config/turnkey/tvc.config.toml`. | The tvc config now exists with defaults (INV-G4; tvc/src/cli.rs:219-223). |

All steps pass in one run from a clean start.

## Inputs (normative)

| Input | Flag | Env | Default | Prompted |
|---|---|---|---|---|
| Output path | `-o`, `--output <PATH>` | `TVC_QUORUM_KEY_CONFIG_OUT` | `quorum_key.json` | no |
| Slot 1 prefill | none | none | `<FILL_IN_OPERATOR_PUBLIC_KEY_1>` | no |
| `shares`, `threshold` | none | none | fixed `2` and `2` | no |

The command MUST resolve the output path in the Part 00 value resolution order (tvc/src/commands/keys/init_local_quorum_key.rs:16-26).
The order here is flag, then environment variable, then the built-in default.
No command config exists for this command.
The slot 1 prefill comes from the tvc config: the active org's default operator kind selects one operator record (tvc/src/operator.rs:282-304).
The lookup needs an active org; without one the prefill misses (tvc/src/operator.rs:283).
A prefill miss MUST NOT fail the command (INV-3).

## Interactive behavior (normative)

The command MUST NOT prompt.
The run function ignores its context argument (tvc/src/commands/keys/init_local_quorum_key.rs:29).
Behavior MUST be identical in interactive mode, non-interactive mode, and JSON mode (V-10).
No `--interactive` fill mode exists; Gap 4 records the difference from the sibling init commands.

## Outputs (normative)

In human mode a successful run MUST print this block on stdout (tvc/src/commands/keys/init_local_quorum_key.rs:53-68).
Both `<output>` markers MUST render as the resolved output path.

```
Created quorum key config template: <output>

Constraints:
  shares    : 1..=255
  threshold : >= 2 and <= shares

Edit the file to fill in your values, then run:
  tvc keys generate-local-quorum-key --config-file <output>
```

Gap 6 records the misleading `1..=255` bound; the block above states current behavior.

In JSON mode a successful run MUST emit exactly one NDJSON object: `{"reason":"quorum_key_config_created","path":"<output>"}` (tvc/src/outcome.rs:65; tvc/tests/message_format.rs:89-112).

The template MUST serialize as pretty printed JSON: `shares` = 2, `threshold` = 2, and a two-entry `operatorPublicKeys` array (tvc/src/config/quorum_key.rs:29-40).
Entry 1 MUST hold the prefill when the lookup succeeds and `<FILL_IN_OPERATOR_PUBLIC_KEY_1>` otherwise.
Entry 2 MUST hold `<FILL_IN_OPERATOR_PUBLIC_KEY_2>`.

## Side effects (normative)

- The command MUST create the file at the resolved output path (tvc/src/commands/keys/init_local_quorum_key.rs:39-40). It MUST NOT overwrite an existing file (INV-1; tvc/src/commands/keys/init_local_quorum_key.rs:30-32).
- Dispatch loads the tvc config before the command body runs, and creates a default one when absent (INV-G4; tvc/src/cli.rs:219-223).
- When the default operator kind is `local`, the prefill reads the sole local operator record's key file from disk (tvc/src/operator.rs:286-290).
- When the default operator kind is `hosted`, the prefill concatenates the sole hosted operator record's two stored public keys (tvc/src/operator.rs:291-297).
- When the default operator kind is `yubikey`, the prefill reads the cached key in the tvc config (tvc/src/operator.rs:298-302). The command MUST NOT perform device I/O for the prefill.
- The command MUST NOT call the Turnkey API and MUST NOT submit an activity (INV-2).

## Failure modes (normative)

Every error from this command MUST classify as `command_error` through the fallback (tvc/src/errors.rs:93-102).
The exit code MUST be 1 (INV-G2).
In JSON mode the error MUST surface as one NDJSON object: `reason` = `command_error`, `code` = `command_error`, chain in `message` (tvc/src/output.rs:304-342).
In human mode the error line goes to stderr with an `error: ` prefix (tvc/src/output.rs:161-174).

| Condition | Message | Mechanism |
|---|---|---|
| Output file exists | `File already exists: <output>` | tvc/src/commands/keys/init_local_quorum_key.rs:30-32 |
| Write failure | `failed to write file: <output>` heads the chain | tvc/src/commands/keys/init_local_quorum_key.rs:39-40 |
| `HOME` unset | `HOME environment variable not set` | tvc/src/cli.rs:215 |
| tvc config unreadable | `failed to read config file: <path>` heads the chain | tvc/src/cli.rs:225-227 |
| tvc config unparseable | `failed to parse config file: <path>` heads the chain | tvc/src/cli.rs:229-230 |

The `HOME` and tvc config failures happen in dispatch, before the command body runs (tvc/src/cli.rs:215-230).
A prefill miss MUST NOT fail the command; every miss degrades silently to the placeholder (tvc/src/operator.rs:282-304).

## Test vectors (normative)

Vector comparison excludes the nondeterministic fields Part 00 lists.
Every vector starts clean with `HOME=/home/op` and working directory `/home/op/qk`, plus the listed Given.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | Nothing extra. | `tvc keys init-local-quorum-key` | Exit 0. stdout is the Outputs block with `<output>` = `quorum_key.json`. `quorum_key.json` holds the 2 of 2 placeholder template (tvc/src/commands/keys/init_local_quorum_key.rs:53-68; tvc/src/config/quorum_key.rs:29-40). |
| V-2 | Nothing extra. | `tvc --message-format=json keys init-local-quorum-key --output /home/op/qk/quorum_key.json` | stdout is exactly one NDJSON object `{"reason":"quorum_key_config_created","path":"/home/op/qk/quorum_key.json"}`; exit 0 (tvc/tests/message_format.rs:89-112). |
| V-3 | Env `TVC_QUORUM_KEY_CONFIG_OUT=/home/op/qk/custom.json`. | `tvc keys init-local-quorum-key` | Writes `/home/op/qk/custom.json`; the human first line names that path; exit 0 (tvc/src/commands/keys/init_local_quorum_key.rs:16-26). |
| V-4 | Env `TVC_QUORUM_KEY_CONFIG_OUT=/home/op/qk/env.json`. | `tvc keys init-local-quorum-key --output /home/op/qk/flag.json` | Writes `/home/op/qk/flag.json`; `/home/op/qk/env.json` stays absent (tvc/src/commands/keys/init_local_quorum_key.rs:16-26; tvc/src/cli.rs:19). |
| V-5 | File `/home/op/qk/quorum_key.json` already exists. | `tvc keys init-local-quorum-key` | Exit 1; stderr carries `error: File already exists: quorum_key.json`; the existing file is unchanged (tvc/src/commands/keys/init_local_quorum_key.rs:30-32). |
| V-6 | File `/home/op/qk/quorum_key.json` already exists. | `tvc --message-format=json keys init-local-quorum-key` | stdout carries one NDJSON object with `reason` = `command_error`, `code` = `command_error`, `message` = `File already exists: quorum_key.json`; exit 1 (tvc/src/output.rs:323-342; tvc/src/errors.rs:93-102). |
| V-7 | tvc config with an active org, default operator kind `yubikey`, a YubiKey operator record with serial `29973535`, and a cached public key of `07` repeated 130 times. | `tvc keys init-local-quorum-key` | Exit 0. `operatorPublicKeys[0]` equals the 260 character hex string; `operatorPublicKeys[1]` = `<FILL_IN_OPERATOR_PUBLIC_KEY_2>` (tvc/src/operator.rs:298-302; test tvc/src/operator.rs:566-577). |
| V-8 | tvc config as in V-7, without the cached public key. | `tvc keys init-local-quorum-key` | Exit 0. Both `operatorPublicKeys` entries are the placeholders (tvc/src/operator.rs:300; test tvc/src/operator.rs:580-584). |
| V-9 | `/home/op/.config/turnkey/tvc.config.toml` contains `not toml [[`. | `tvc --message-format=json keys init-local-quorum-key` | stdout carries one NDJSON object with `code` = `command_error` whose `message` starts with `failed to parse config file: /home/op/.config/turnkey/tvc.config.toml`; exit 1; no output file appears (tvc/src/cli.rs:229-230). |
| V-10 | Nothing extra. | `tvc --non-interactive keys init-local-quorum-key` | Identical observation to V-1 (tvc/src/commands/keys/init_local_quorum_key.rs:29). |

## Invariants (normative)

The Part 00 global invariants apply; INV-G4 governs the tvc config load in dispatch.

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | The command MUST NOT overwrite an existing file at the resolved output path. | The existence guard bails before any write (tvc/src/commands/keys/init_local_quorum_key.rs:30-32). Behavioral check: V-5. |
| INV-2 | The command MUST NOT perform network I/O. | `run` receives no API client; its only inputs are the context, the parsed args, and the tvc config (tvc/src/commands/keys/init_local_quorum_key.rs:29). |
| INV-3 | A prefill miss MUST NOT fail the command. | `Config::default_operator_public_key` returns `Option<String>`; every miss maps to `None` (tvc/src/operator.rs:282-304). Behavioral check: tvc/src/operator.rs:580-584. |
| INV-4 | The written template MUST parse as a `QuorumKeyConfig`. | The template serializes from the same struct the generator deserializes (tvc/src/config/quorum_key.rs:19-25). Behavioral check: tvc/src/config/quorum_key.rs:109-120 runs the template through the generator's `validate`. |

## Gaps (informative)

1. **[capability]** The template hard-wires 2 of 2 with exactly two key slots. No `--shares` or `--threshold` flag, environment variable, or prompt exists (tvc/src/commands/keys/init_local_quorum_key.rs:16-26). `QuorumKeyConfig::template` fixes the numbers and the two entries (tvc/src/config/quorum_key.rs:29-40). A 3 of 5 user hand-edits the JSON. The unstated rule that the key count equals `shares` lives only in the generator (tvc/src/commands/keys/generate_local_quorum_key.rs:117-122).

2. **[capability]** No explicit choice fills the operator key slots from the tvc config. The tvc config knows every operator record's public key: local key file, hosted stored keys, cached YubiKey key (tvc/src/operator.rs:282-304). A quorum config needs two or more keys, yet at most one slot prefills, always from the default operator kind's sole record. No repeatable `--operator-public-key` flag and no by-name operator selection exist. Stored operator records and the org default silently constrain the choice, which forces manual hex copy-paste for every other slot.

3. **[consistency]** The prefill silently vanishes whenever the default kind lookup finds anything other than exactly one operator record. All three selectors are sole-record.
   - Multiple locals miss (tvc/src/config/turnkey.rs:452-456).
   - Multiple hosted miss (tvc/src/config/turnkey.rs:478-482).
   - Multiple or unregistered YubiKeys miss (tvc/src/config/turnkey.rs:513-521; tvc/src/operator.rs:298-302).

   Every miss collapses to `None` with no warning, so the user sees a placeholder with no hint that a prefill ran. `app init` shares the helper and the latent issue; the impact is larger here because keys are the entire payload of this template.

4. **[consistency]** No `--interactive` fill mode exists, unlike both sibling init commands. `app init` offers `--interactive` (tvc/src/commands/app/init.rs:31); `deploy init` does too (tvc/src/commands/deploy/init.rs:47). Each walks prompts and writes a filled config. This command only ever emits placeholders. An interactive walk over shares, threshold, and per-slot operator choice would also resolve gaps 1 and 2 for the interactive path.

5. **[consistency]** The JSON outcome payload omits the `command`, `template`, and `interactive` fields that both sibling `*_config_created` outcomes carry. `QuorumKeyConfigCreated` holds only `path` (tvc/src/commands/keys/init_local_quorum_key.rs:47-51). Compare `AppConfigCreated` (tvc/src/commands/app/init.rs:76-81) and `DeploymentConfigCreated` (tvc/src/commands/deploy/init.rs:142-149). Machine consumers of the config-created family get an inconsistent shape.

6. **[docs]** The printed constraint `shares : 1..=255` is unsatisfiable at shares = 1. The human output advertises `1..=255` (tvc/src/commands/keys/init_local_quorum_key.rs:60). The generator requires `threshold >= 2` and `threshold <= shares` (tvc/src/config/quorum_key.rs:62-74), so shares = 1 always fails. The honest range is 2..=255. The same misleading bound lives in the `validate` error message (tvc/src/config/quorum_key.rs:56-61), which fires from the generate command.

7. **[consistency]** A malformed tvc config aborts the command even though the config only feeds a best-effort prefill. Dispatch parses the tvc config before any command body runs (tvc/src/cli.rs:229-230). Init succeeds with no tvc config, because dispatch creates the defaults (tvc/src/cli.rs:219-223), yet dies on a corrupt one. The command is otherwise fully offline; the issue lives at dispatch level and touches every command.
