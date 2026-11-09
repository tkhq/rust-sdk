# CI recipe: ship a deployment unattended

A GitHub Actions job that builds the image, pins both digests, creates and approves a deployment, waits for it to serve, and switches traffic when needed. Everything runs with `--message-format json` and env-var auth. Adapt the build step to your toolchain; the `tvc` steps are the contract.

## One-time human setup (not in CI)

CI cannot create credentials or operators, and env-var auth alone carries no operator key. A human does this once:

1. `tvc login`, choosing the **local key file** operator type. This writes an API key profile and a local operator key (`operator.json`, with `public_key` and a hex `private_key`) under `~/.config/turnkey/`.
2. Create the app with that operator's key in the manifest set (`tvc app init` prefills it when the profile has exactly one operator; see [config-files.md](config-files.md)). Save `appId` from `app create`. Do this on the machine with the profile: under env-only auth, `app create` creates the app and then fails with `no active organization set`, and the printed ids are lost (recoverable only via `app list -n <name>`).
3. Export the operator seed for CI: `tvc keys backup-operator-key -o operator-backup.json`, then copy the `private_key` value (64 hex chars) into a CI secret. Delete the backup file. This seed signs approvals on behalf of that operator; treat it like the API private key.
4. Store as repository secrets: `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, `TVC_API_KEY_PRIVATE`, `TVC_OPERATOR_SEED`. Store `TVC_APP_ID` as a variable.
5. Decide how TVC will pull the image. The job below assumes a **public** package. GHCR packages are private by default, even in public repositories, and the runner's `docker login` authenticates only the runner, not TVC's pull. Either publish once and set the package visibility to public, or follow **Private registry** under Variations and keep the pull-secret file in a CI secret.

Why a local seed and not a hosted operator: `deploy approve` resolves hosted and YubiKey operators from the login profile on disk. With env vars only, it fails with `No active organization. Run tvc login first or provide --operator-seed or --operator-seed-path.` The explicit seed path is the one that needs no profile. Restoring a full `~/.config/turnkey/` profile onto the runner also works, but is more state to protect.

## The job

```yaml
name: deploy-tvc
on:
  push:
    tags: ["v*.*.*"]

jobs:
  deploy:
    runs-on: ubuntu-latest
    timeout-minutes: 30
    permissions:
      contents: read
      packages: write
    env:
      TVC_ORG_ID: ${{ secrets.TVC_ORG_ID }}
      TVC_API_KEY_PUBLIC: ${{ secrets.TVC_API_KEY_PUBLIC }}
      TVC_API_KEY_PRIVATE: ${{ secrets.TVC_API_KEY_PRIVATE }}
      TVC_APP_ID: ${{ vars.TVC_APP_ID }}
      TVC_NON_INTERACTIVE: "true"
      IMAGE: ghcr.io/${{ github.repository }}
      PIVOT_PATH: /tvc_app            # where your Dockerfile puts the binary
    steps:
      - uses: actions/checkout@v4

      - name: Install tvc
        run: cargo install tvc --locked   # or download a release binary and cache it

      - name: Build and push a single-platform linux/amd64 image
        id: image
        run: |
          echo "${{ secrets.GITHUB_TOKEN }}" | docker login ghcr.io -u "${{ github.actor }}" --password-stdin
          docker buildx build --platform linux/amd64 --provenance=false --sbom=false \
            --tag "$IMAGE:${GITHUB_REF_NAME}" --push .
          # Image digest: the registry's digest for the pushed manifest.
          IMAGE_DIGEST=$(docker buildx imagetools inspect "$IMAGE:${GITHUB_REF_NAME}" --format '{{json .Manifest.Digest}}' | tr -d '"')
          echo "url=$IMAGE@$IMAGE_DIGEST" >> "$GITHUB_OUTPUT"
          # Pivot digest: sha256 of the binary inside that exact image.
          docker pull "$IMAGE@$IMAGE_DIGEST"
          docker create --name extract "$IMAGE@$IMAGE_DIGEST" /bin/true
          docker cp "extract:$PIVOT_PATH" ./pivot
          docker rm extract
          echo "pivot_digest=$(sha256sum ./pivot | awk '{print $1}')" >> "$GITHUB_OUTPUT"

      - name: Create deployment
        id: create
        run: |
          OUT=$(tvc deploy create \
            --qos-version 0.15.0 \
            --pivot-image-url "${{ steps.image.outputs.url }}" \
            --pivot-path "$PIVOT_PATH" \
            --expected-pivot-digest "${{ steps.image.outputs.pivot_digest }}" \
            --health-check-port 3000 --public-ingress-port 3000 \
            --message-format json)
          echo "$OUT"
          echo "deploy_id=$(echo "$OUT" | jq -r 'select(.reason=="deployment_created") | .deploymentId')" >> "$GITHUB_OUTPUT"

      - name: Approve
        env:
          TVC_OPERATOR_SEED: ${{ secrets.TVC_OPERATOR_SEED }}
        run: |
          tvc deploy approve --deploy-id "${{ steps.create.outputs.deploy_id }}" \
            --dangerous-skip-interactive --message-format json

      - name: Wait until live (bounded)
        run: |
          ID="${{ steps.create.outputs.deploy_id }}"
          deadline=$((SECONDS + 600))
          while [ $SECONDS -lt $deadline ]; do
            LINE=$(tvc deploy get-status --deploy-id "$ID" --message-format json || true)
            echo "$LINE"
            REASON=$(echo "$LINE" | jq -r '.reason // empty')
            if [ "$REASON" = "deployment_runtime_status" ]; then
              TARGETED=$(echo "$LINE" | jq -r '.isTargeted')
              READY=$(echo "$LINE" | jq -r '.replicas.ready // 0')
              DESIRED=$(echo "$LINE" | jq -r '.replicas.desired // -1')
              if [ "$TARGETED" = "false" ]; then
                tvc app set-live-deploy --deploy-id "$ID" --message-format json
              elif [ "$READY" = "$DESIRED" ] && [ "$READY" != "0" ]; then
                echo "live: $READY/$DESIRED replicas"; exit 0
              fi
            fi
            # not_found, replicas: null, and HTTP 500 api_error right after approval all mean "not ready yet"
            sleep 5
          done
          echo "::error::deployment $ID not live after 10 minutes; see references/failure-diagnosis.md"; exit 1

      - name: Health check
        run: |
          DOMAIN=$(tvc app list --message-format json \
            | jq -r --arg id "$TVC_APP_ID" '.apps[] | select(.id == $id) | .publicDomain | split(",")[0]')
          curl --fail --silent --show-error --max-time 10 "https://$DOMAIN/health" >/dev/null
```

## What each step relies on

- **Env-only auth.** All three `TVC_*` credentials are set, so no profile is read. `TVC_NON_INTERACTIVE=true` is redundant with `--message-format json` but makes the intent explicit for any human-format call you add.
- **Digest-only image URL.** `--pivot-image-url` takes `repo@sha256:...`. `--provenance=false --sbom=false` keeps buildx from publishing an index with attestation manifests, so the pushed digest is the single `linux/amd64` manifest TVC needs. If your builder emits an index anyway, select the `linux/amd64` child digest from `imagetools inspect`.
- **Pivot digest from the pushed image**, never from a local build artifact. The pull-by-digest before extraction guarantees you hash what you pinned.
- **Flags instead of `deploy.json`.** `deploy create` accepts every field as a flag or `TVC_*` env var, so CI needs no config file. `healthCheckType` defaults to HTTP on this path; set it in a config file only if you need gRPC.
- **Approval with an explicit seed.** `TVC_OPERATOR_SEED` is read by `--operator-seed`; `--operator-seed-path` reads the same hex from a file. The CLI matches the seed's public key against the manifest set and posts under the matching operator id, so `--operator-id` is unnecessary. `--dangerous-skip-interactive` is required in JSON mode; the approval is blind, which is acceptable only because the pipeline built the inputs it approves.
- **Conditional set-live.** The first deployment of an app auto-targets on approval. Later ones report `isTargeted == false` until `app set-live-deploy` runs, which this loop does once it sees that state. Calling set-live on an already-targeted deployment fails with HTTP 400, which the gate avoids.
- **Bounded wait.** Ten minutes leaves headroom: the verified run took about 5.5 minutes from approval to 3/3 replicas, with the first 45 seconds answering HTTP 500. On timeout, fail the job and leave the deployment in place for diagnosis rather than deleting it.

## Variations

- **Private registry:** add `--pivot-pull-secret <file>` to `deploy create`. The file is a Kubernetes-style Docker config, `{"auths": {"ghcr.io": {"username": "<user>", "password": "<token>"}}}`, with a read-only token scoped to that one package; the CLI encrypts it for the org's environment and only the ciphertext is sent (`hasPullSecret: true` in status output). Write the file from a CI secret at job time and never commit it. Encryption needs a `TVC_API_BASE_URL` the CLI recognizes (prod, preprod, or dev); a custom URL fails before any request. The encrypted secret expires two hours after `deploy create` by default, and Turnkey checks the expiry when it accepts the secret. Two hours is ample when this job approves immediately; if the app's manifest set waits on a human co-approver, add `--pull-secret-expires-at <RFC 3339>` with a later deadline. Omit both flags for public images.
- **Reuse the previous config:** `tvc deploy init --from-deployment <OLD_ID> -o deploy.json` copies every field, including the old digest; override `--pivot-image-url` and `--expected-pivot-digest` on `deploy create`.
- **Replica count:** `--replicas <n>` (or `replicas` in the config) overrides the backend default.
- **Debug builds:** never in this pipeline. Debug mode needs an app created with `--dangerous-enable-debug-mode-deployments`, runs one replica, and marks the quorum key insecure; keep it on a separate dev app.

## Verified

The `tvc` steps were run end to end against a TVC dev organization on 2026-11-04 with tvc 0.17.1, from a shell whose `HOME` had no `~/.config/turnkey/` and whose only credentials were the three `TVC_*` variables plus `TVC_API_BASE_URL`:

- `app list` worked env-only. `deploy create` from flags alone (including `--pivot-args=--host` style values that start with dashes) returned `deployment_created`.
- `deploy approve` without a seed failed with `No active organization. Run tvc login first or provide --operator-seed or --operator-seed-path.`, as the recipe says. With `--operator-seed-path` it returned `manifest_approval_posted` with `quorumReached: true` and the operator id the manifest set assigned to that key, without `--operator-id`.
- `get-status` returned HTTP 500 `api_error` for about 45 seconds after approval, then `isTargeted: true` with 1/3 replicas ready at about 50 seconds, 2/3 at about 2 minutes, and 3/3 at about 5.5 minutes. `set-live-deploy` was never needed, since the first deployment auto-targeted.
- `publicDomain` from `app list` resolved and `/health` returned 200.
- The image was an already-public package, so no pull secret was exercised.

The image build step was not part of that run; it is the pattern from `tkhq/tvc-examples` and your toolchain will differ. The one-time setup also surfaced a CLI quirk: `app create` under env-only auth creates the app and then fails with `no active organization set`, losing the printed ids. That is why app creation is listed under human setup above, and why the recipe approves by seed rather than by `--operator-id`.
