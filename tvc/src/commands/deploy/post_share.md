# tvc deploy post-share

## Purpose

Posts a re-encrypted quorum key share (produced offline by `tvc keys
re-encrypt-local-share`) to Turnkey via the `post_tvc_quorum_key_share`
activity, attributing it to a share-set operator UUID. Run it as the final,
online step of local/YubiKey operator provisioning: provisioning-details ->
re-encrypt-local-share -> post-share. Hosted operators skip this flow entirely
(`tvc deploy provision` re-encrypts and posts server-side in one step).

## Inputs

| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| re-encrypted share file | `--re-encrypted-share <PATH>` | `TVC_RE_ENCRYPTED_SHARE` | none | none (required) | never |
| share-set operator UUID | `--share-operator-id <UUID>` | `TVC_SHARE_OPERATOR_ID` | none | none (required) | never |
| auth (org id / API keys) | none | `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, `TVC_API_KEY_PRIVATE`, `TVC_API_BASE_URL` | active org in `~/.config/turnkey/config.toml` | login config | never |

- No config-file key exists for either command input; resolution is flag > env
  only (`tvc/src/commands/deploy/post_share.rs:19-29`). No deviation from the
  global order, but no third tier either.
- Deployment ID, ephemeral public key, share ciphertext, and approval signature
  all come from inside the file (`post_share.rs:71-84`); there is deliberately
  no `--deploy-id` (asserted by `tvc/tests/deploy_post_share.rs:29`).
- Auth follows the global rule: all-three env vars beat the active org's stored
  credentials; partial env auth errors (`tvc/src/client.rs:48-64,192-242`).

## Interactive behavior

None. The command never prompts (`_ctx` is unused, `post_share.rs:33`). Both
inputs are clap-required, so a missing one is a usage error (exit 2) even on a
TTY — there is no interactive fallback, and nothing changes under
`--non-interactive` / JSON mode. Contrast `deploy approve`, which prompts to
select among candidate operators (`approve.rs:340-346`).

## Outputs

- Human: one line — `Provisioning Share ID: <id>` (`post_share.rs:65-69`).
- JSON: one terminal message, reason `quorum_key_share_posted`, payload
  `{"provisioningShareId": "..."}` (`tvc/src/outcome.rs:44`, camelCase via
  `post_share.rs:59-63`).

## Side effects

- Reads the re-encrypted share JSON file (`post_share.rs:34-35` via
  `util::read_json_file`).
- Loads `~/.config/turnkey/config.toml`; if absent, a default config file is
  created before dispatch (global behavior, `tvc/src/cli.rs:219-223`), plus the
  active org's API key file when env auth is not set.
- One Turnkey activity: `post_tvc_quorum_key_share` with
  `PostTvcQuorumKeyShareIntent { deployment_id, ephemeral_public_key_hex,
  share_approval_bundle { operator_id, re_encrypted_share_hex, signature } }`
  (`post_share.rs:48-52,71-84`). The file's `share_approval.member`
  (alias + pub_key) is dropped; only the signature is forwarded, hex-encoded.
- No device (YubiKey) interaction; no files written.

## Failure modes

- Missing/invalid flags (non-UUID `--share-operator-id`): clap parse failure,
  `usage_error`, exit 2 (JSON mode still emits the NDJSON usage_error line,
  `cli.rs:154-182`).
- File unreadable/unparseable: `failed to read/parse re-encrypted share
  output: <path>` -> `command_error`, exit 1 (`tvc/src/util.rs:18-28`). A
  missing `deploymentId` (etc.) fails at deserialization
  (`post_share.rs:127-142` test).
- No active org / no API key / partial env auth: `command_error`, exit 1
  (`client.rs:103-125,226-234`).
- API failures classify via the preserved `TurnkeyClientError`:
  `unauthorized`/`not_found`/`api_error`/`approval_required`/`network_error`
  (`tvc/src/errors.rs:93-103,212-249`); the `.context("failed to post quorum
  key share")` wrapper keeps the typed source (`post_share.rs:48-52`).
- No local verification of the bundle: nothing checks that the hex fields
  decode, that the signature verifies, or that the operator UUID corresponds to
  the share's member — those surface only as server-side rejections.

## Gaps

1. **[capability] The share-set operator ID must be hand-supplied as a raw UUID
   even though everything needed to resolve it locally already exists.** The
   consumed file identifies the operator by public key
   (`share_approval.member.pub_key`, `re_encrypt_local_share.rs:75-80`), the
   deployment's share set — fetchable by the `deployment_id` already in the
   file — maps public key -> operator UUID (`TvcDeployment.share_set:
   Option<TvcOperatorSet>` with per-operator `id` + `public_key`,
   `client/src/generated/external.data.v1.rs:714-802`), and config can store a
   local operator's Turnkey ID (`LocalOperatorRecord.operator_id`,
   `config/turnkey/mod.rs:340-347`). `deploy approve` derives exactly this
   (`post_operator_id`) from the fetched deployment by matching the selected
   key (`approve.rs:349-379`); post-share does none of it
   (`post_share.rs:26-28`).

2. **[capability] The required UUID is not discoverable through any tvc
   command.** `app create` persists only `manifest_set_operator_ids`
   (`commands/app/create.rs:270-273`) and the API result's
   `share_set_operator_ids` field is used nowhere in tvc
   (`client/src/generated/immutable.activity.v1.rs:3594`, zero references under
   `tvc/src`); `deploy status` discards `share_set` (`deploy/status.rs:68`);
   `provisioning-details` prints share-set approvals by alias/pub-key, not
   Turnkey operator UUIDs. The one mandatory input of this command cannot be
   obtained from the CLI that demands it.

3. **[capability] No local cross-check that `--share-operator-id` matches the
   share being posted.** The intent carries the user's UUID next to the file's
   signature, and the file's member identity is dropped (`post_share.rs:78-82`),
   so a transposed or stale UUID is only caught (if at all) server-side after
   signing and a network round trip. Siblings check this class of mismatch
   locally: `provision` verifies the operator is in the manifest share set
   (`deploy/provision.rs:142-151`); `approve` ensures the requested operator ID
   is linked to the selected key (`approve.rs:349-379,396-401`).

4. **[consistency] Env-var naming drifts from the family's hand-off pattern.**
   The provisioning-details -> re-encrypt pair pairs `TVC_PROVISION_BUNDLE_OUT`
   with `TVC_PROVISION_BUNDLE`; the re-encrypt -> post-share pair is
   `TVC_RE_ENCRYPTED_OUT` (`re_encrypt_local_share.rs:69-71`) vs
   `TVC_RE_ENCRYPTED_SHARE` (`post_share.rs:23`). Likewise `provision` and
   `approve` both use `TVC_OPERATOR_ID` while this command introduces
   `TVC_SHARE_OPERATOR_ID` (`post_share.rs:27`) — arguably deliberate
   (share-set vs manifest-set identity), but a pipeline wiring the same
   operator through provision and post-share sets two differently named vars.

5. **[consistency] Result passes through unvalidated and the API error loses
   the activity-approval enrichment `provision` applies.** `provision` rejects
   an empty `provisioning_share_id` (`deploy/provision.rs:165-177`) and wraps
   client errors in `hosted_activity_error`, which names the activity that
   needs approvals (`provision.rs:105-109`, `operator/hosted.rs:355-364`);
   post-share prints whatever comes back (`post_share.rs:54-56`) and uses a
   bare `.context` (`post_share.rs:48-52`). Classification survives either way;
   the human message and empty-ID tripwire do not.

6. **[consistency] Timestamp computed inline instead of the shared helper.**
   `post_share.rs:43-46` duplicates `operator::timestamp_ms()`
   (`operator.rs:478-483`), which `provision` uses (`provision.rs:107`).
   Trivial.

7. **[capability] (producer-side, hand-off context) The upstream command
   constrains which operator can produce the file this command consumes.**
   `keys re-encrypt-local-share` picks its backend solely from
   `default_operator_kind` (`operator.rs:411-440`): a configured YubiKey
   operator is only reachable when YubiKey is the org default
   (`re_encrypt_local_share.rs:125-128`), multiple local operators are a dead
   end (`SelectLocalOperatorError::MultipleLocalOperators`,
   `config/turnkey/mod.rs:386-392`) with no selector flag or prompt, and a
   hosted default hard-bails (`operator.rs:435-439`). This is Richard's
   canonical gap; it gates what shares can ever reach post-share.
