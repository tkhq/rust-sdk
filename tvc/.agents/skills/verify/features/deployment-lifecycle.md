# Deployment lifecycle

Users register an existing container image, approve its execution manifest,
observe runtime readiness, and manage deployment retention.

## Sub-features

- `deploy.template` creates a blank, interactive, or copied configuration.
- `deploy.create` validates an image and registers a deployment.
- `deploy.approve` reviews a manifest, signs it, and optionally posts approval.
- `deploy.observe` distinguishes manifest validity from runtime readiness.
- `deploy.logs` fetches or polls debug logs.
- `deploy.retention` deletes and restores deployments.

## How to get to it (user POV)

Use `tvc deploy init`, `create`, `approve`, `status`, `get-status`, `debug-logs`,
`delete`, and `restore`. Follow creation with manifest approval, provisioning
when required, and readiness checks before switching traffic.

## Driving it with verify.sh

Preconditions: smoke uses the checked-in manifest and seed solely as public test
fixtures. Connected creation needs an app, a published image, its expected
pivot digest, and authentication. Logs need app and deployment debug opt-ins.

- Run `tvc/.agents/skills/verify/scripts/verify.sh smoke`. Inspect the deployment
  template and `artifacts/approval.json`; the outcome must name that file,
  whose signer must belong to the fixture manifest set.
- Run `tvc/.agents/skills/verify/scripts/verify.sh test deploy_approve` for
  explicit-seed and operator-based approval paths, source conflicts, and posting.
- Run `tvc/.agents/skills/verify/scripts/verify.sh test pty approve_walks_all_sections_with_yeses`
  and `tvc/.agents/skills/verify/scripts/verify.sh test pty approve_bails_when_user_rejects_pivot`.
  The first reviews schema, namespace, enclave, pivot, manifest set, and share
  set before signing. The second must stop after rejection without signing.
- Run `tvc/.agents/skills/verify/scripts/verify.sh test deploy_init_help`,
  `tvc/.agents/skills/verify/scripts/verify.sh test deploy_debug_logs`,
  `tvc/.agents/skills/verify/scripts/verify.sh test deploy_delete`, and
  `tvc/.agents/skills/verify/scripts/verify.sh test deploy_restore` for fixture coverage.
- With approved connected inputs, run
  `tvc --message-format json deploy create --config-file "$deploy_config"`.
  Save the returned deployment ID. For a copy journey, run
  `tvc --message-format json deploy init --from-deployment "$deploy_id" --output copied-deploy.json`
  and inspect recovered fields before reusing them.
- Run `tvc deploy approve --deploy-id "$deploy_id" --operator-id "$operator_id"`
  in a PTY. Review each section before acceptance. Check persisted approvals with
  `tvc --message-format json deploy status --deploy-id "$deploy_id"`.
- Poll `tvc --message-format json deploy get-status --deploy-id "$deploy_id"`
  with a bounded deadline. Record replica counts and terminal failures. Use
  `tvc --message-format json deploy debug-logs --deploy-id "$deploy_id"` for a
  single log window; add `--poll` only with PID-based teardown.
- For authorized retention checks on this run's resources, use
  `tvc --message-format json deploy delete --deploy-id "$deploy_id"`, then
  `tvc --message-format json deploy restore --deploy-id "$deploy_id"`.
  Read deployment state after each operation.

## Gotchas

- Deployment creation does not build an image. Copied config cannot recover a
  private-image pull secret, and a changed image requires a matching digest.
- `deploy status` checks configuration and approvals. `get-status` reports
  target and replica readiness. Neither replaces a workload health request.
- `--dry-run` does not sign. With `--deploy-id`, both dry-run and skip-post can
  fetch remotely. The smoke path's local manifest and seed avoid those fetches.
- JSON mode disables prompts. Fixture-only `--dangerous-skip-interactive`
  exercises signing, not the human review path. Use PTY tests for that path.
- A non-debug deployment cannot gain logs retroactively. Empty-home smoke has
  no image, live deployment, debug logs, or fresh attestation.
