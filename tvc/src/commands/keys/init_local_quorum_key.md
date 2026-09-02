# tvc keys init-local-quorum-key

## Purpose

Writes a JSON template for the quorum key config that `tvc keys
generate-local-quorum-key` consumes. Run it once before generating a local
quorum key, then hand-edit the file. The template covers every field the
generator honors — `QuorumKeyConfig` is exactly `shares`, `threshold`,
`operatorPublicKeys` (`tvc/src/config/quorum_key.rs:21-25`), all three are
emitted (`quorum_key.rs:29-40`), and the printed follow-up command names the
generator's real flag (`--config-file`,
`tvc/src/commands/keys/generate_local_quorum_key.rs:23`). Slot 1 of
`operatorPublicKeys` is best-effort prefilled with the active org's default
operator public key; everything else is a fixed 2-of-2 placeholder skeleton.

## Inputs

| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| output path | `-o`, `--output` | `TVC_QUORUM_KEY_CONFIG_OUT` | — | `quorum_key.json` | no |
| operator key prefill (slot 1) | — | — | active org's default operator (registry state) | `<FILL_IN_OPERATOR_PUBLIC_KEY_1>` | no |
| shares / threshold | — | — | — | hard-coded 2 / 2 | no |

Resolution order for `--output` is standard flag > env > default
(`tvc/src/commands/keys/init_local_quorum_key.rs:16-26`). Globals get no
special treatment: `_ctx` is unused (`init_local_quorum_key.rs:29`), so
`--non-interactive` changes nothing.

## Interactive behavior

None. The command never prompts and behaves identically in interactive,
`--non-interactive`, and JSON modes. There is no `--interactive` fill mode
(unlike the sibling init commands — see Gaps).

## Outputs

Human: `Created quorum key config template: <path>`, a constraints block
(`shares : 1..=255`, `threshold : >= 2 and <= shares`), and the next-step
command `tvc keys generate-local-quorum-key --config-file <path>`
(`init_local_quorum_key.rs:53-67`).

JSON: one line, `{"reason":"quorum_key_config_created","path":"<path>"}`
(`tvc/src/outcome.rs:65`, tested at `tvc/tests/message_format.rs:89-112`).

File content: pretty-printed JSON `{shares: 2, threshold: 2,
operatorPublicKeys: [<default-or-placeholder>, <placeholder>]}`
(`quorum_key.rs:29-40`).

## Side effects

- Writes the template to `--output`; refuses to overwrite an existing file
  (`init_local_quorum_key.rs:30-31`).
- Dispatch loads `~/.config/turnkey/config.toml`, and **creates it with
  defaults if absent** (`tvc/src/cli.rs:219-223`).
- When the org's default operator kind is local, reads the sole local
  operator's key file from disk for the prefill (`tvc/src/operator.rs:286-290`).
  YubiKey kind reads only the registry's cached key — no device I/O
  (`operator.rs:298-302`). No network calls, no Turnkey activities.

## Failure modes

All errors classify as `command_error` (plain anyhow, fallback at
`tvc/src/errors.rs:93-103`), exit 1:

- Output file already exists (`init_local_quorum_key.rs:30-31`).
- Write failure (`init_local_quorum_key.rs:39-40`).
- `HOME` unset, or global config unreadable/unparseable — fails in dispatch
  before the command body runs (`cli.rs:215-230`).

Prefill lookup misses are swallowed (`Option`, `operator.rs:282-304`) — never
an error, silently degrades to the placeholder.

## Gaps

1. **[capability] Shares and threshold cannot be supplied — the template is
   hard-wired to 2-of-2 with exactly two key slots.** No `--shares`/
   `--threshold` flag, env, or prompt exists (`init_local_quorum_key.rs:16-26`);
   `QuorumKeyConfig::template` fixes 2/2 and two entries
   (`quorum_key.rs:29-40`). A 3-of-5 user must hand-edit JSON and know the
   unstated rule that key count must equal `shares`
   (`generate_local_quorum_key.rs:117-122`).

2. **[capability] Operator key slots cannot be filled from the operator
   registry by explicit choice.** The config registry knows every configured
   operator's public key (local key file, hosted stored points, cached YubiKey
   key — `operator.rs:282-304`), and a quorum config needs >= 2 keys, yet at
   most one slot prefills, always from `org.default_operator_kind`'s sole
   record; there is no repeatable `--operator-public-key` flag and no by-name
   operator selection. This is the canonical shape: registry state and the
   org default silently constrain what the user can choose, forcing manual
   hex copy-paste for every other slot.

3. **[consistency] The prefill silently vanishes whenever the default-kind
   lookup is not exactly one record.** All three selectors are sole-record —
   multiple locals (`tvc/src/config/turnkey/mod.rs:452-456`), multiple hosted
   (`mod.rs:478-482`), multiple/unregistered YubiKeys (`mod.rs:513-521`,
   `operator.rs:298-302`) — and every miss collapses to `None` with no
   warning, so the user gets a placeholder with no hint the prefill was
   attempted. Same latent issue as `app init` (shared helper), but worse here
   since keys are the entire payload of this template.

4. **[consistency] No `--interactive` fill mode, unlike both sibling init
   commands.** `app init` and `deploy init` both offer `--interactive` to walk
   prompts and write a filled config (`tvc/src/commands/app/init.rs:30-31`,
   `tvc/src/commands/deploy/init.rs:46-47`); this command only ever emits
   placeholders. An interactive walk (shares, threshold, per-slot operator
   pick) would also resolve gaps 1-2 for the interactive path.

5. **[consistency] The JSON outcome payload omits the `command` / `template` /
   `interactive` fields both sibling `*_config_created` outcomes carry.**
   `QuorumKeyConfigCreated` is `{path}` only
   (`init_local_quorum_key.rs:47-51`) vs `AppConfigCreated`
   (`app/init.rs:76-81`) and `DeploymentConfigCreated`
   (`deploy/init.rs:140-149`), so machine consumers of the config-created
   family get an inconsistent shape.

6. **[docs] The printed constraint `shares : 1..=255` is unsatisfiable at
   shares=1.** `init_local_quorum_key.rs:60` advertises `1..={MAX_SHARES}`,
   but the generator requires `threshold >= 2` and `threshold <= shares`
   (`quorum_key.rs:62-74`), so shares=1 always fails; the honest range is
   2..=255. The same misleading bound lives in `validate()`'s own error
   message (`quorum_key.rs:56-61`), which fires from generate.

7. **[consistency] A malformed global config aborts the command even though
   config only feeds a best-effort prefill.** Dispatch parses
   `~/.config/turnkey/config.toml` before any command runs
   (`cli.rs:229-230`): init succeeds with NO config (defaults are created,
   `cli.rs:219-223`) but dies on a corrupt one, despite its only config use
   being the optional slot-1 prefill. Dispatch-level, shared by all commands —
   noted because this command is otherwise fully offline.
