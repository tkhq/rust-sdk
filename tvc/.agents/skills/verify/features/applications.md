# Applications

An app defines trust through its operator sets and quorum key. Deployments
belong to that app; switching its live deployment changes the traffic target.

## Sub-features

- `app.template` writes a blank or interactive configuration.
- `app.create` registers trust configuration and remembers returned IDs.
- `app.observe` lists hostnames and reports app-wide readiness.
- `app.target` selects a live deployment.
- `app.delete` removes a disposable app.

## How to get to it (user POV)

Use `tvc app init`, `create`, `list`, `status`, `set-live-deploy`, and `delete`.
Start with `app init --output app.json`, fill the template, then pass it to
`app create --config-file app.json`.

## Driving it with verify.sh

Preconditions: templates need no credentials. Connected checks need valid
operator and quorum keys. Target switching needs a ready second deployment.

- Run `tvc/.agents/skills/verify/scripts/verify.sh smoke` for template creation
  and a second write that must fail while preserving the original bytes.
- Run `tvc/.agents/skills/verify/scripts/verify.sh test app_create` and
  `tvc/.agents/skills/verify/scripts/verify.sh test config_validation` for
  input merging, operator selection, and placeholder rejection.
- Run `tvc/.agents/skills/verify/scripts/verify.sh test app_list`,
  `tvc/.agents/skills/verify/scripts/verify.sh test app_set_live_deploy`, and
  `tvc/.agents/skills/verify/scripts/verify.sh test app_delete` for API fixtures.
- In a connected run, use `tvc --message-format json app create --config-file "$app_config"`.
  Save its returned app ID, then run `tvc --message-format json app list` and
  `tvc --message-format json app status --app-id "$app_id"`.
- Once the chosen deployment is ready, run
  `tvc --message-format json app set-live-deploy --deploy-id "$deploy_id"`.
  Read app status again and check the workload at the hostname from `app list`.
- When authorized, run `tvc --message-format json app delete --app-id "$app_id"`
  for the run-created app. Verify its absence with `app list`.

## Gotchas

- `app init` has no `--name` flag despite the older README example. Edit the
  generated config or use `--interactive` with a real PTY.
- Placeholders are intentionally invalid. Template generation does not prove
  app creation. Empty-home smoke has no seed organization or deployed workload.
- The first deployment targets automatically. A later traffic switch still
  requires readiness checks and a workload request; an accepted API activity
  does not prove serving behavior.
