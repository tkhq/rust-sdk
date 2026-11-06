# Failure diagnosis: approved, but never ready

Use this when `deploy approve` succeeded and `deploy get-status` keeps reporting `isTargeted: true` with `replicas.ready` below `replicas.desired` (usually pinned at 0) past your timeout. Work top to bottom; stop at the first branch that applies. Every check here is read-only.

## 0. Is it still "no state yet"?

Within the first minute or two after approval, these are normal and mean keep polling:

- `get-status` returns `replicas: null`
- `code: not_found` with "app status not found"
- a run of `code: api_error` with HTTP 500 (the backend answers 500 for an app with no live deployment yet; 25 to 50 seconds of these have been observed before the first real status)

A healthy three-replica deployment has been observed to go 1/3 at about 50 seconds, 2/3 at about 2 minutes, and 3/3 at about 5.5 minutes after approval. Only continue below once real status has been returned and stayed unhealthy for your full timeout (10 minutes is a safe bound).

## 1. Is the approval actually complete?

```bash
tvc deploy status --deploy-id <ID> --message-format json
```

Read `manifestApprovals`: `quorumReached` must be `true`. If it is `false`, the manifest set threshold is not met; collect the remaining approvals (`deploy approve` from another manifest-set operator). If `markedForDeletion` is `true`, the deployment was deleted; `deploy restore` or create a new one. If `debugMode` is `true` on an app you thought was production, stop and tell the human.

## 2. Is it supposed to be live?

`isTargeted == false` is not a failure: an older deployment is still receiving traffic. If this one should take over, run `tvc app set-live-deploy --deploy-id <ID> --message-format json` and poll again. If `app status` shows `targetedDeploymentId` pointing elsewhere, same answer.

## 3. `isTargeted == true`, `replicas.ready` stays at 0

The enclave is failing to start or failing its health probe. The CLI cannot tell these apart, and a non-debug deployment produces no logs, so diagnose from the inputs. Check in this order; the first three cause the vast majority of cases.

1. **Pivot digest mismatch.** `expectedPivotDigest` must be the sha256 of the file at `pivotPath` inside the exact image pinned by `pivotContainerImageUrl`. Recompute from the pushed image, not a local artifact:
   ```bash
   docker pull <repo>@sha256:<image-digest>
   docker create --name x <repo>@sha256:<image-digest> /bin/true && docker cp x:<pivotPath> ./pivot && docker rm x
   sha256sum ./pivot
   ```
   A mismatch, or `docker cp` failing because `pivotPath` does not exist, is your answer. Also confirm you did not paste the image digest into `expectedPivotDigest`; they are different hashes.
2. **Wrong image platform or an index digest.** The enclave runs `linux/amd64` only, and the pinned digest must be a single-platform manifest, not a multi-arch index. `docker buildx imagetools inspect <repo>@sha256:<digest>` should show one manifest with `linux/amd64`. If it shows a list, pin the `linux/amd64` child's digest and redeploy.
3. **Health probe never passes.** TVC probes `healthCheckPort` with `healthCheckType` (HTTP by default, path `/health` by convention) and expects a 200. Compare `deploy status` → `pivotContainer.args` and your deploy config against what the binary actually binds: the app must listen on `healthCheckPort` inside the container and answer `/health` with 200. A port the app does not listen on looks identical to an app that never started. gRPC servers need `healthCheckType: TVC_HEALTH_CHECK_TYPE_GRPC`, which is config-file only.
4. **Private image without a pull secret.** If the registry needs auth and the deployment was created without `--pivot-pull-secret` (or the scaffold's `<REMOVE_ME_IF_PIVOT_CONTAINER_URL_IS_PUBLIC>` placeholder was left in), the image cannot be pulled. GHCR packages are private by default even in public repositories, so a freshly pushed package is the usual case. Recreate with the secret, or make the package public. A secret that was supplied can still have expired: the encrypted envelope defaults to two hours, and a deployment approved after that cannot pull. Recreate the deployment with a fresh secret and, if approval is slow, a later `--pull-secret-expires-at`.
5. **Bad `pivotArgs`.** The binary is started with exactly `pivotContainer.args`. An unknown flag or a missing required argument exits immediately. Run the same binary with the same args locally.
6. **Egress needed but disabled.** If the app must reach the network at startup (fetch config, connect to a database) and `tvc app list` shows `egressEnabled: false`, it cannot. Egress is fixed at `app create`; the fix is a new app with `enableEgress: true`.
7. **Needs something the enclave does not provide.** Writable disk at unexpected paths, environment variables, a network listener on a port other than the two configured, or more CPU and memory than the instance size. These need a human and often a code change.

Then, to confirm rather than infer: **only if the app was created with `--dangerous-enable-debug-mode-deployments`**, ship a new deployment of the same image with `--dangerous-deploy-debug-mode` and read `tvc deploy debug-logs --deploy-id <NEW_ID> --tail-lines 200 --message-format json`. The existing deployment can never produce logs retroactively. If the app does not allow debug deployments, do not try; escalate with the bundle below.

## 4. `replicas.ready` flaps between 0 and `desired`

The process starts, passes the probe, then dies or stops answering: a crash loop or an intermittent probe. Same causes as section 3 items 3, 5, and 7, plus resource exhaustion. Debug deployment on a dev app is the only self-serve way to see why.

## 5. Live, but `curl https://<publicDomain>/health` fails

- `publicDomain` can hold several comma-separated hostnames; use the first.
- A 404 from the edge while `replicas.ready < desired` is expected; wait for full readiness.
- `/health` is only a convention for the probe; the public path your app serves may differ. Hit a path the app actually implements.
- Never substitute a constructed hostname for the returned `publicDomain`.

## What not to do

- **Do not delete and recreate with the same config.** A bad input fails identically; you only lose the evidence.
- **Do not keep polling indefinitely.** Past the timeout, nothing changes without a new input.
- **Do not invent diagnostics.** The CLI exposes exactly `get-status`, `status`, and (for debug deployments) `debug-logs`.

## Escalation bundle

When sections 1 to 3 do not resolve it, hand the human: the `appId` and `deploymentId`; the last `deploy get-status` and `deploy status` output; the exact `deploy.json` or flags used; `docker buildx imagetools inspect` output for the pinned digest; the recomputed pivot digest; and whether the app allows debug deployments.
