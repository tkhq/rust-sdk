# Identity and tools

Users authenticate API requests, select operator identities, and export the
CLI's version-matched agent instructions. API keys and operator keys have
different roles.

## Sub-features

- `identity.login` creates or selects a saved organization profile.
- `identity.operator` registers a local, hosted, or YubiKey operator.
- `identity.delete` removes a profile and its local key files.
- `tools.version` identifies the binary.
- `tools.skills` exports bundled instructions without silently replacing edits.

## How to get to it (user POV)

Use `tvc login`, `tvc operator create`, `tvc profile delete`, `tvc version`,
and `tvc skills save`. CI can authenticate with all three required
`TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, and `TVC_API_KEY_PRIVATE` variables.

## Driving it with verify.sh

Preconditions: offline checks need the build only. Successful login and hosted
operator creation need a dedicated organization. Local profile deletion needs
a disposable saved profile.

- Run `tvc/.agents/skills/verify/scripts/verify.sh smoke`. Inspect version output
  and exported `tvc-deployments/SKILL.md`. The second export writes zero files.
- Run `tvc/.agents/skills/verify/scripts/verify.sh test skills_save` to cover
  explicit, default, and global destinations, changed-file refusal, and force.
- Run `tvc/.agents/skills/verify/scripts/verify.sh test login` and
  `tvc/.agents/skills/verify/scripts/verify.sh test auth_env` for profile and
  complete-versus-partial environment authentication.
- Run `tvc/.agents/skills/verify/scripts/verify.sh test operator_create` for
  backend selection and persisted operator identity at the API boundary.
- Run `tvc/.agents/skills/verify/scripts/verify.sh test pty` for real prompts,
  profile ordering, operator selection, and cancellation.
- For a live login, run `tvc login --help` and follow the current login flow in
  the disposable home. Read back its active profile and use `tvc app list` to
  prove authentication. Creating config alone does not prove credentials work.

## Gotchas

- Even `version` and `skills save` load and may create TVC config. `--help` does
  neither. Malformed saved config can stop offline commands before dispatch.
- Partial environment authentication errors; it does not merge with disk keys.
- `profile delete` removes local key files. It does not revoke remote access.
  Require before-and-after file evidence for a disposable profile.
- Local and hosted operator successes differ. A hosted operator has no private
  key file to export. Hardware behavior belongs to [YubiKey](yubikey.md).
