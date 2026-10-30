# Quorum and provisioning

Quorum shares unlock encrypted app state inside an approved enclave. Local
operators re-encrypt stored shares; hosted operators request that operation
through Turnkey. Both paths depend on the deployment manifest and attestation.

## Sub-features

- `keys.local` initializes config and generates encrypted local quorum shares.
- `keys.hosted` creates a hosted quorum key from hosted operators.
- `keys.backup` copies a local operator key to an explicit destination.
- `provision.bundle` obtains and verifies attestation and manifest inputs.
- `provision.local` re-encrypts a local share and posts its output.
- `provision.hosted` requests hosted share re-encryption and submission.

## How to get to it (user POV)

Local setup uses `keys init-local-quorum-key`, then `keys generate-local-quorum-key`.
Hosted setup uses `keys create-quorum-key`. Back up local identity with
`keys backup-operator-key`. Local provisioning connects `deploy provisioning-details`
to `keys re-encrypt-local-share` to `deploy post-share`. Hosted provisioning uses
`deploy provision`.

## Driving it with verify.sh

Preconditions: share generation needs distinct real composite operator public
keys and a valid threshold. Provisioning needs a matching app, quorum key,
manifest, authorized operator, and fresh attestation bundle. Local backup needs
a disposable profile with a local operator key.

- Run `tvc/.agents/skills/verify/scripts/verify.sh smoke` to create quorum config.
  It contains placeholders; this step does not prove share generation.
- Run `tvc/.agents/skills/verify/scripts/verify.sh test keys_generate_local_quorum_key`.
  Existing tests drive generation, decrypt shares, and reconstruct the public
  key from a threshold subset.
- Run `tvc/.agents/skills/verify/scripts/verify.sh test keys_create_quorum_key`
  and `tvc/.agents/skills/verify/scripts/verify.sh test keys_backup_operator_key`
  for hosted API and local backup behavior.
- Run `tvc/.agents/skills/verify/scripts/verify.sh test keys_re_encrypt_local_share`
  for CLI round trips with decrypted result and signature checks. Run
  `tvc/.agents/skills/verify/scripts/verify.sh test deploy_provisioning_details`,
  `tvc/.agents/skills/verify/scripts/verify.sh test deploy_post_share`, and
  `tvc/.agents/skills/verify/scripts/verify.sh test deploy_provision` for API boundaries.
- With live preconditions satisfied, run
  `tvc --message-format json deploy provisioning-details --deploy-id "$deploy_id" --provision-bundle-out bundle.json`.
  Retain its verification result and bundle as sensitive evidence.
- For a local share, run
  `tvc --message-format json keys re-encrypt-local-share --quorum-key-metadata "$metadata_path" --provision-bundle bundle.json --re-encrypted-out share.json`.
  Inspect the output's deployment and manifest identity before running
  `tvc --message-format json deploy post-share --re-encrypted-share share.json --share-operator-id "$share_operator_id"`.
- For a hosted share, run
  `tvc --message-format json deploy provision --deploy-id "$deploy_id" --operator-id "$operator_id"`.
  After enough shares, prove readiness with `deploy get-status` and a workload request.

## Gotchas

- The fixture crypto round trip uses `--dangerous-skip-verification` for a
  synthetic bundle. It proves share processing, not production attestation.
- Fresh attestation validates certificate time, PCRs, approvals, and manifest
  commitment. Never bypass those checks to make a connected journey pass.
- A hosted key cannot be backed up as a local private key. A backup can contain
  private key material; keep disposable copies in scratch, not shared evidence.
- The public manifest fixture is enough for offline approval but cannot seed a
  live provisioning journey. Missing keys, bundle, or deployment blocks that path.
