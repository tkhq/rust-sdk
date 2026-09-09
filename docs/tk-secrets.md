# Secrets preview

`tk secret` lists metadata and transfers secret bytes using authenticated enclave
bundles. It uses the shared `--profile`, `--config`, and `--message-format json`
options. The production signer quorum key is pinned; there is no trust override.
The import initialization endpoint must be enabled for your organization. It is
currently marked internal in the API definition, so do not assume broad public
availability from the presence of this command.

## Import and metadata

```sh
tk --profile demo secret import --name service-token --input-file token.bin
tk --profile demo secret import --name service-token --input-file token.bin \
  --static-properties-file properties.json
tk --profile demo secret list --limit 50
```

Input is raw bytes (including empty, binary, or newline-containing values), with a
1 MiB CLI limit. `--input-file -` reads stdin. There is no plaintext argument or
plaintext JSON output. Name and static properties are visible metadata: never put
secret values there. Static properties are a JSON object of string values, for
example `{"purpose":"demo"}`. Names should be unique within the organization.

List returns `data.items` and `data.nextCursor`. Pass a non-null cursor as
`--cursor`; an exactly full final page may require one empty page to confirm the
end. Only the typed metadata fields are returned.

Initialization can require approval. The command returns `phase: init-import`
with its activity ID. After approval, repeat the same import with
`--init-activity-id ID` and the original local input. This reads the initialization
activity without creating another ingress target. A pending final import uses
ordinary `tk activity get` or `tk activity wait`. Completed imports return
`data.secretIds`. An ambiguous submission reports its fingerprint: inspect
activities before retrying; a timeout does not mean the submission failed.

## Export and recovery

```sh
tk --profile demo secret export SECRET_ID --output token.bin \
  --state-file export-state.json --timeout 60
tk --profile demo secret resume --state-file export-state.json --timeout 60
```

Output and state must be different, new files. Protected file handling requires
Unix. The CLI saves a mode-0600 state file containing the recipient key material,
exact proposal and fingerprint, organization, endpoint, credential public key,
and output path **before** submitting the export. Treat that file as a secret.
Keep it out of source control, logs, chat, and review packets.

Use the original identity and endpoint to resume. Resume only queries activities;
it never repeats a submission, including after an unknown network outcome. It
walks older activity pages by fingerprint if the activity ID is not yet known.
The matched activity must have the expected organization, type, fingerprint and
ID. Approval remains a separate, explicitly authorized activity operation.

The command verifies the export signature, organization, and recipient before
writing bytes into a new mode-0600 file. It publishes output atomically and syncs
it before removing recipient key material from the recovery state. It never
prints plaintext or key material. Pending, denied, and unknown outcomes do not
create output. Pending timeout and failures return a nonzero exit code with
metadata and the saved recovery path.

If a process stops after publishing output but before updating state, resume can
reconcile the existing file only when it is private, regular, has one link, and
matches the authenticated plaintext exactly. Other existing files are never
overwritten. Completed recovery checks that the protected output still exists;
a deleted output cannot be reconstructed after the key has been removed.
Concurrent operations on the same state path are rejected by a `.lock` file. If
a process crashes, remove that lock only after confirming the process has stopped.

This surface does not implement provider token rotation, secret update/delete,
environment injection, or automatic policy creation. Live endpoint availability
and organization-specific policy behavior require separate acceptance testing.
