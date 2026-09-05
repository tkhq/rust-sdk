# Local unified CLI preview

This preview provides a locally testable `tk` CLI in the existing `tvc`
package. It is an implementation preview, not a production release or a complete
replacement for `tkcli`. No live organization acceptance run has been performed.

Build the two entry points with `cargo build -p tvc --bins`. The existing `tvc`
entry point keeps its command grammar and configuration. Cloud operations are
also available through `tk tvc app|deploy|operator|keys`.

## Shared identity setup

Generate a credential file without contacting Turnkey:

```sh
tk api-key generate --output ./agent-key.json
```

The output contains only the public key and destination. Register the public
key with Turnkey before using it to authenticate; generating a file does not
register remote access. Existing registered credentials use the same JSON
shape as TVC: `public_key`, `private_key`, and `curve: "p256"`.

Save and select an existing registered identity:

```sh
tk login --profile admin --organization-id ORG_UUID --api-key-file ./admin-key.json
tk --profile admin auth status
tk --profile admin whoami --message-format json
```

Login verifies the credential with `whoami` before saving the registry. It does
not create an operator, wallet, or remote user. The current login requires an
existing credential file; browser onboarding is not implemented.

The new registry is `~/.config/turnkey/tk.config.toml`. Profiles identify users
or credentials, so admin and agent profiles can share one organization. Use
`tk profile import --help` for explicit TVC and experimental-tk migration.
Imports preserve their sources and do not automatically select the new profile.
`profile delete` removes a registry entry only; logout clears selection only.

For CI, supply the complete `TURNKEY_ORGANIZATION_ID`, `TURNKEY_API_PUBLIC_KEY`,
and `TURNKEY_API_PRIVATE_KEY` bundle. Environment-only calls need no HOME or
registry. Explicit `--profile` selection overrides ambient credentials. Partial,
empty, or mixed canonical/legacy bundles fail rather than falling back to a
saved admin identity.

Shared identity selectors are currently rejected on `tk tvc`; those commands
continue using the existing TVC identity environment and configuration.

## Implemented command families

```text
tk auth login|status|whoami|logout
tk profile list|show|use|import|delete
tk api-key generate|register|list|delete
tk user list|get|create|update|delete|tag
tk policy list|get|create|create-batch|update|delete|evaluations
tk wallet list|get|create|update|account
tk sign payload|transaction
tk request
tk activity list|get|approve|reject|wait
```

Resource mutations and signing accept `--input-json` or `--input-file` with a
parameters object matching the generated API type. `--input-file -` reads stdin.
Local input parsing occurs before credential resolution. See
[resource input examples](tk-resource-commands.md) and each command's help.

`tk request` takes an API-relative `--path` and either `--body` or `--body-file`
(`-` reads stdin). It preserves original body bytes and requires an organization
matching the selected identity. `--stamp-only` produces signed metadata without
posting. Signing and raw submission do not imply transaction broadcasting.

## Automation and recovery

Use `--message-format json` for newline-delimited records. Shared results carry
`schemaVersion`, `reason`, `command`, `status`, `data`, and activity metadata when
available. Accepted pending submissions exit successfully with `status: pending`;
that is not a completed action. Resume with `tk activity wait ACTIVITY_ID`, whose
positive `--timeout` defaults to 60 seconds. Activity inspection can succeed when
the inspected activity is rejected; waiting for that activity exits with failure.

Uncertain mutation outcomes retain recovery information where available. Do not
resubmit solely because a transport response was lost; inspect activities first.
The CLI emits failures with a nonzero exit status, including structured
operation failures. Output-write failure also produces a nonzero exit status.

## Remaining release work

Outstanding acceptance includes live
agent lifecycle verification in an authorized test organization, TVC identity
injection, complete profile/readiness UX and browser onboarding, pagination
completion, managed transaction delivery, import/export cryptography, remaining
legacy curves, distribution, and SSH/Git consolidation. Machine contracts and
skill examples must be validated against this preview before release claims.
