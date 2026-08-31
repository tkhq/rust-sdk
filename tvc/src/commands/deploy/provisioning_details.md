# tvc deploy provisioning-details

## Purpose

Fetches a deployment's provisioning details (attestation document + manifest envelope) from
the Turnkey API, verifies them (Nitro attestation chain, PCRs vs manifest, manifest-set
approvals), and prints an attestation summary. Optionally writes a `ProvisionBundle` JSON
file that `tvc keys re-encrypt-local-share` consumes (`--provision-bundle`). Run it to
inspect what an enclave attested to before provisioning, or to capture the bundle input for
local-share re-encryption.

Entry point: `tvc/src/commands/deploy/provisioning_details.rs:65` (`run`), dispatched from
`tvc/src/cli.rs:248-250`. Shared flow: `tvc/src/provisioning.rs`.

## Inputs

| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| deployment id (UUID) | `-d` / `--deploy-id` | `TVC_DEPLOY_ID` | — | required | never |
| skip verification | `--dangerous-skip-verification` | `TVC_DANGEROUS_SKIP_VERIFICATION` | — | false | never |
| bundle output path | `--provision-bundle-out <PATH>` | `TVC_PROVISION_BUNDLE_OUT` | — | none (no file written) | never |
| auth | — | `TVC_ORG_ID` + `TVC_API_KEY_PUBLIC` + `TVC_API_KEY_PRIVATE` (+ optional `TVC_API_BASE_URL`) | active org in `~/.config/turnkey` | base URL `https://api.turnkey.com` | never |

Resolution follows the global flag > env > config order; auth follows the all-three-env-or-none
rule (`tvc/src/client.rs:48-64`, partial env set errors at `client.rs:225-234`). No input has a
config-file key — consistent with the sibling deploy commands (`status`, `get-status`, `delete`,
`debug-logs` all take the same required `--deploy-id`/`TVC_DEPLOY_ID` pair).

## Interactive behavior

None. `run` never touches `ctx` for prompting (`provisioning_details.rs:65` takes `_ctx`), so
behavior is identical in interactive, `--non-interactive`, and JSON modes. Missing
`--deploy-id` is a clap usage error (exit 2) in every mode, not a prompt.

## Outputs

Human mode: one text block (golden test `provisioning_details.rs:446-472`) — optional
"Provision bundle written to: <path>" header, then Deployment, Verification, Ephemeral Key,
Module ID, Digest, Timestamp (ms), User Data, Nonce, PCRs (indices 16/17 annotated as
setup/live manifest commitments, `provisioning_details.rs:284-291`), Certificate Length,
CA Bundle Certificates, Manifest Set Approvals `<count>/<threshold>` with alias/pubkey lines,
Share Set Approvals (count only, `(none)` when empty).

JSON mode: a single NDJSON object with `"reason": "provisioning_details"`
(`tvc/src/outcome.rs:42`), camelCase fields mirroring the human block; byte fields hex-encoded;
`bundlePath` present only when `--provision-bundle-out` wrote a file
(`provisioning_details.rs:189-191`). Verification status is the literal string
`"verified (attestation + approvals)"` or
`"skipped attestation, PCR, and approval verification (--dangerous-skip-verification)"`
(`provisioning_details.rs:93-97`).

Only PCR indices <= 17 are included — in both human and JSON output
(`SUMMARY_PCR_MAX_INDEX`, `provisioning_details.rs:61,143-147`).

## Side effects

- Reads `~/.config/turnkey` config; a missing config file is created with defaults as a
  side effect of dispatch (`cli.rs:219-224`). No config mutation otherwise.
- One read-only API call: `get_tvc_deployment_provisioning_details`
  (`provisioning.rs:81-85`). No activities submitted.
- Optional file write: `--provision-bundle-out` serializes a `ProvisionBundle`
  (base64 attestation doc, manifest envelope, `fetchedAtUnixMs`, deployment id, hex ephemeral
  key — `provisioning.rs:121-146`) via `write_file` = `tokio::fs::write`
  (`tvc/src/util.rs:31-35`): silently overwrites an existing file, fails if the parent
  directory does not exist. The bundle is written only after verification succeeds (summary is
  built first, `provisioning_details.rs:69-91`) — or unconditionally when verification is skipped.
- No device (YubiKey) interaction.

## Failure modes

- Missing/malformed `--deploy-id`, bad flag values: clap `usage_error`, exit 2 (JSON parse
  errors routed through `handle_parse_error`, `cli.rs:154-182`).
- No active org / no stored API key: "Run `tvc login` first" (`client.rs:103-117`) —
  `command_error`, exit 1. Partial env auth names the missing vars (`client.rs:225-234`).
- API errors classify via the standard taxonomy (`tvc/src/errors.rs:212-221`): 401/403
  `unauthorized`, 404 `not_found`, other HTTP `api_error`, transport `network_error`.
- Response missing/empty attestation doc or manifest envelope, or undecodable envelope:
  bail (`provisioning.rs:101-111`) — `command_error`, exit 1.
- Verification failures — manifest approvals invalid/below threshold, attestation doc
  signature/cert-chain invalid or expired, PCR0-3/manifest-hash mismatch
  (`provisioning.rs:206-240`) — `command_error`, exit 1.
- Bundle write failure ("failed to write file: <path>"): `command_error`, exit 1, after the
  API call succeeded.

## Gaps

1. **[consistency] No stderr warning when `--dangerous-skip-verification` is set, unlike both
   sibling commands in the same flow.** `deploy provision` and `keys re-encrypt-local-share`
   print "WARNING: Skipping attestation, PCR, and manifest approval verification! ..."
   (`provision.rs:72-77`, `re_encrypt_local_share.rs:117-122`); this command only embeds
   "skipped ..." in the outcome's verification field (`provisioning_details.rs:93-97`), which a
   consumer piping the summary can easily miss.

2. **[capability] `--provision-bundle-out` silently overwrites an existing file with no
   `--overwrite` gate, confirmation, or bail.** `write_file` is plain `tokio::fs::write`
   (`util.rs:31-35`, call at `provisioning_details.rs:109-113`). Siblings guard writes:
   `deploy init`/`app init` bail on an existing file (`deploy/init.rs:77-78`,
   `app/init.rs:47-49`), `keys backup-operator-key` requires `--overwrite` or an interactive
   confirm (`backup_operator_key.rs:93-96`).

3. **[capability] The bundle records `fetchedAtUnixMs` but nothing can validate a bundle "as of
   fetch time", so bundles silently expire with the attestation cert chain.** Verification at
   consumption always uses `SystemTime::now()` (`provisioning.rs:261-268`); the
   `validation_time_override` parameter is `None` on every production path
   (`provisioning.rs:160-165`, `provisioning_details.rs:73`, `provision.rs:103`) — test-only.
   A bundle consumed after the Nitro cert chain expires (hours) can only pass
   `keys re-encrypt-local-share` by passing `--dangerous-skip-verification` there, discarding
   all verification instead of pinning it to the recorded fetch time.

4. **[docs] The status line "verified (attestation + approvals)" overstates what was checked:
   share-set approvals are displayed but never verified.** `check_approvals` validates only
   manifest-set approvals — signatures, membership, uniqueness, threshold (qos_core 0.14.1
   `boot.rs:663-704`, called from `provisioning.rs:211-213`) — yet the output lists share-set
   approvals directly under that banner (`provisioning_details.rs:306-315`). Their signatures
   could be garbage and the command still reports "verified". Relatedly, manifest-set approvals
   render as `count/threshold` but share-set approvals render count-only, though the share-set
   threshold is available on the manifest.

5. **[consistency] `TVC_DANGEROUS_SKIP_VERIFICATION` uses clap's strict bool env parsing while
   the global `TVC_NON_INTERACTIVE` accepts boolish values.** The global flag opts into
   `BoolishValueParser` (`cli.rs:72-79`, accepts `1`/`yes`/`on`); the dangerous flag is a bare
   bool with `env` (`provisioning_details.rs:30-32`), so `TVC_DANGEROUS_SKIP_VERIFICATION=1`
   fails value validation where `TVC_NON_INTERACTIVE=1` works. Same defect in `provision.rs:41-43`
   and `re_encrypt_local_share.rs:64-66`.

6. **[consistency] A bundle produced under `--dangerous-skip-verification` is indistinguishable
   from a verified one.** `ProvisionBundle` has no field recording that verification was skipped
   (`provisioning.rs:121-129`), and the ephemeral key it pins came from an unverified attestation
   doc. Mitigated: `re-encrypt-local-share` re-verifies by default
   (`provisioning.rs:167-203`), so this only bites when skip is passed at both ends — but then no
   artifact anywhere records that nothing was ever verified.

7. **[docs] The PCR cutoff at index 17 is undocumented and unconditional, in JSON mode too.**
   `SUMMARY_PCR_MAX_INDEX = 17` drops any higher-indexed PCRs from both outputs
   (`provisioning_details.rs:61,143-147`) with no flag to include them and no mention in help
   text; a machine consumer of the JSON has no way to know the list was filtered.
