# Experimental CLI for Turnkey Verifiable Cloud

## Usage

### Create an App

```bash
# Login to Turnkey
tvc login

# Generate app config template
tvc app init --name my-app --output my-app.json

# Edit my-app.json to fill in required values (quorumPublicKey, operator keys, etc.)

# Create the app
tvc app create my-app.json
```

### Create and Approve a Deployment

```bash
# Generate deployment config template
tvc deploy init --output my-deploy.json

# Edit my-deploy.json to fill in required values (appId, container images, etc.)

# Optional: validate the digest of the file at pivotPath inside the image locally
tvc deploy validate-pivot-digest \
  --image-url ghcr.io/tkhq/helloworld:latest \
  --pivot-path /helloworld \
  --expected-digest <EXPECTED_PIVOT_DIGEST>

# Create the deployment and validate the pivot digest locally first
tvc deploy create my-deploy.json --validate-pivot-digest

# Recommended: uses GetTvcDeployment to fetch the manifest automatically and
# validates the pivot digest against the deployment manifest before approval
tvc deploy approve \
  --deploy-id <DEPLOYMENT_UUID> \
  --validate-pivot-digest \
  --operator-id <OPERATOR_UUID> # Turnkey's ID for your operator (from app create response)

# Alternative: provide manifest file and IDs manually
tvc deploy approve \
  --manifest manifest.json \
  --manifest-id <MANIFEST_UUID> \  # Turnkey's ID for the manifest (from deploy create response)
  --operator-id <OPERATOR_UUID>
```

## Pivot Digest Validation

`tvc deploy validate-pivot-digest` computes the SHA-256 digest of the file at
`pivotPath` inside a Linux container image. The command resolves the image with
the CLI's native OCI client and does not require Docker.

For private images, pass `--pull-secret` with an unencrypted Docker-style
`config.json` containing credentials for the image registry.

```bash
tvc deploy validate-pivot-digest \
  --image-url ghcr.io/tkhq/helloworld@sha256:f8132a6236609e4c67d9d29e5694989f18e528240844638e850897ee6319676d \
  --pivot-path /helloworld \
  --expected-digest cbe01169428f144086bfaef348bbf3db70f9217628996cafd2ecb85d5f2b47a1
```

Notes:

- Validation is Linux-only and resolves the image as `linux/amd64`.
- `tvc deploy approve --validate-pivot-digest` only works with `--deploy-id`.
- `--pull-secret` expects an unencrypted Docker-style JSON file, not the
  encrypted pull secret stored in deployment config.
