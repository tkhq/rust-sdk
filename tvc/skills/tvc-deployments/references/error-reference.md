# TVC CLI error reference

Every runtime error in JSON mode is a single NDJSON line:

```json
{ "reason": "command_error", "code": "<code>", "httpStatus": 404, "message": "<full error chain>" }
```

- `reason` is `command_error` for all runtime errors (and `missing_required_input` for that one special case). It identifies the *shape*; it is not the classification. Reasons are snake_case, like every other outcome reason.
- `code` is the stable classification, **branch on this**, never on `message` text.
- `httpStatus` is present only when the failure came from an HTTP response.
- `message` is the full error chain. Creation and boot-evidence failures retain safe causes and omit coordinator response bodies. Show it to humans; do not parse it programmatically.

Exit codes: `0` success, `1` runtime error, `2` usage error.

## Code taxonomy

| `code` | Trigger | Recovery |
|---|---|---|
| `missing_required_input` | A value that would have been prompted for interactively was absent in non-interactive mode. Only the prompting commands emit it (`deploy approve` without `--dangerous-skip-interactive`, `keys re-encrypt-local-share` or `deploy provision` without `--dangerous-skip-interactive`, `login` without `--org`); a missing clap-required flag is `usage_error` instead. `reason` is `missing_required_input`. | The message names the missing flag. Supply it or its `TVC_*` env var. |
| `usage_error` | Bad flags/args or unknown subcommand (clap parse failure). Exit code 2. | Fix the command. Re-check subcommand and flag names against the skill; do not invent flags. |
| `unauthorized` | HTTP 401/403 from the API. | Verify `TVC_ORG_ID` / `TVC_API_KEY_PUBLIC` / `TVC_API_KEY_PRIVATE` (or your `tvc login` profile), and that the key is permitted for the action. |
| `not_found` | HTTP 404, or an OK response with an empty resource. | Confirm the `--app-id` / `--deploy-id`. There is no `deploy list`, but `tvc app status --app-id <APP_ID>` returns the app's `deployments[]` with their IDs once the app has a live deployment (before that the backend answers with a 500 `api_error`, and an unapproved deployment is not listed), and `tvc app list` gives the app IDs. When a deployment lookup 404s the message reads `cannot find deployment <id> in organization <org-id>`, where `<org-id>` is the organization the credentials are scoped to. The most common cause is that the deployment belongs to a different organization: make sure the credentials (`TVC_ORG_ID` or the active `tvc login` profile) point at the deployment's organization before assuming the ID is wrong. |
| `approval_required` | The activity needs more approvals (consensus) before it can proceed. | Collect additional operator approvals via `deploy approve`, then continue. |
| `network_error` | Connect, timeout, or DNS failure. Creation submission state is reported separately. | Check connectivity. Follow the creation submission warning before retrying. |
| `api_error` | Any other non-2xx HTTP status, or a failed/unexpected activity. | Read `message` for the server's explanation. Not automatically retryable. |
| `client_version_too_old` | The backend rejected this `tvc` release as older than the minimum it supports (HTTP 400 with `turnkeyErrorCode: TVC_CLIENT_VERSION_TOO_OLD`). | Not a config problem and never retryable: upgrade the binary (`cargo install tvc`) and rerun. The upgrade instruction is in `message`. |
| `command_error` | Fallback for anything not classified above. | Read `message`. |
| `invalid_input` | Invalid parsed provenance expectations. | Fix the requested organization, paths, threshold, or recipients. |
| `malformed_response` | Hosted operator response IDs cannot be converted to the CLI's required types. The operator ID is an unsigned routing label. | Preserve the untrusted recovery labels and investigate. No record is saved. Do not recreate blindly. |
| `provenance_unsupported_environment` | The local API URL has no independent Signer root. No creation POST is sent. | Use the production or preproduction API environment. |
| `provenance_unavailable` | Required request-bound claims are missing, legacy, or use unsupported context. | Check producer compatibility. Do not recreate automatically. |
| `provenance_malformed` | Proof evidence violates the strict wire contract or acceptance bounds. | Preserve recovery labels and investigate the producer. |
| `provenance_authentication_failed` | Independent endorsement or the exact-byte signature fails. | Investigate trusted policy and evidence. Never accept the returned key. |
| `provenance_receipt_mismatch` | Claims do not share a request actually sent by this invocation. | Remote creation may have succeeded. Investigate without automatic recreation. |
| `provenance_claim_mismatch` | Signed output disagrees with intent or result. | Reject the result and investigate. |
| `operator_key_reuse` | An accepted role key collides with a stored role. | Inspect existing records and derivation paths. |
| `untrusted_recipient` | A hosted record lacks recognized creation provenance. | Independently verify recipient keys before explicit manual input. |
| `key_inventory_unavailable` | A stored local, hosted, or hardware role cannot be read or parsed. | Restore a complete valid inventory before creation. |
| `config_conflict` | Another writer changed the exact loaded config snapshot. | Reload and recover the local mutation. Never repeat remote creation blindly. |
| `config_save_failed` | The config was not replaced. | Fix local storage and use a verified recovery record if supplied. |
| `config_durability_uncertain` | Replacement happened, but durability was not confirmed. | Inspect the config before recovery or destructive cleanup. |
| `entropy_unavailable` | Fresh signing could not obtain OS entropy before sending that attempt. Earlier Pending attempts may have been submitted. | Restore OS entropy and inspect the environment. Investigate earlier creation attempts before retrying. |
| `invalid_receipt_stamp` | A receipt stamp is malformed or unsupported. | Fix the supported client stamping configuration. |

## Handling patterns

- After a hosted creation submission failure, investigate before retrying, including for `network_error`, `entropy_unavailable`, and `invalid_receipt_stamp`. A later attempt can fail locally after an earlier POST returned Pending. Response decode failures retain `api_error`.
- **Retryable:** `network_error` (backoff + retry). `api_error` may be transient (5xx) or permanent (4xx-ish), inspect before retrying.
- **Not retryable without a change:** `usage_error`, `missing_required_input`, `unauthorized`, `not_found`, `invalid_input`, `client_version_too_old` (the change is upgrading the binary).
- **Needs a human/side action:** `approval_required` (get more approvals).
- **Never loop blindly on the same command.** If a retry would send the identical request, only retry `network_error`.

## Gotchas

- JSON mode forces non-interactive, so the few commands that would prompt instead fail with `missing_required_input`. Most missing values surface as `usage_error` (a clap-required flag was omitted), so provide every required value up front and treat both codes as "fix the invocation".
- `deploy approve` in JSON mode fails fast (as `missing_required_input` on `--dangerous-skip-interactive`) unless that flag is passed; `keys re-encrypt-local-share` and `deploy provision` do the same.
- `keys re-encrypt-local-share` compares the local quorum key metadata with the provision bundle's manifest before the manifest review and before anything is decrypted or written, and refuses any difference as `command_error`: the quorum public key (`quorum key metadata quorumKeyPublic (<hex>) does not match provision bundle manifest quorumKey (<hex>)`), the threshold (`quorum key metadata threshold (<k>) does not match provision bundle manifest share set threshold (<k>)`), the share count (`quorum key metadata holds <n> shares but provision bundle manifest share set has <n> members`), or the share holders (`quorum key metadata holds a share for operator <hex>, which is not a provision bundle manifest share set member`, `quorum key metadata holds more than one share for operator <hex>`, or `provision bundle manifest share set names operator <hex> more than once`). `--dangerous-skip-verification` does not skip this comparison, and nothing is written. The share set was fixed by `shareSetParams` at `app create`, so do not retry: the key's `quorum_key.json` and the app's `shareSetParams` must agree, which means a new app (see [config-files.md](config-files.md#alternative-local-quorum-key-files-self-provisioned-operator)).
- `deploy approve --deploy-id ... --operator-id <ID>` with an id that is not in the deployment's manifest set is a `command_error`. When the id's key *is* a manifest-set member (the same key registered under another operator name for an earlier app), the message ends with `pass --operator-id <ID>`: retry with exactly that id. This retry works for local-key and YubiKey operators only. A hosted operator's id is fixed in the profile config, so approving with the manifest-set id fails with `selected deployment operator ID <id> does not match configured hosted operator ID <id>`; for a hosted operator, create the app under the operator name the profile already uses so the backend returns that same operator record. Otherwise the message ends with `manifest-set operators: <name> (<id>), ...`, so retry with one of those; when the id is unknown to the organization it also reads `operator ID <id> was not found in organization <org-id>`, naming the organization the credentials are scoped to, since an id from another organization is the usual cause.
- A 404-style "not found" can also mean "exists but has no state yet" (e.g. `deploy get-status` before replicas appear returns `replicas: null` rather than erroring, but a truly unknown ID errors as `not_found`). Distinguish by whether you hold a valid ID, and check the organization first when the message names one.
