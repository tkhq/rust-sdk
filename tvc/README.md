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

> **Note**: A `GetTvcDeployment` API endpoint is not yet available, so the manifest file and IDs must be provided manually. Once this endpoint exists, `tvc deploy approve --deploy-id <ID>` will be able to fetch the manifest automatically.

```bash
# Generate deployment config template
tvc deploy init --output my-deploy.json

# Edit my-deploy.json to fill in required values (appId, container images, etc.)

# Create the deployment
tvc deploy create my-deploy.json

# Approve the deployment manifest
tvc deploy approve \
  --manifest manifest.json \
  --manifest-id <MANIFEST_UUID> \  # Turnkey's ID for the manifest (from deploy create response)
  --operator-id <OPERATOR_UUID>    # Turnkey's ID for your operator (from dashboard)
```