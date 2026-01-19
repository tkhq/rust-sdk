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

# Create the deployment
tvc deploy create my-deploy.json

# Recommended: uses GetTvcDeployment to fetch manifest and manifest_id automatically
tvc deploy approve \
  --deploy-id <DEPLOYMENT_UUID> \
  --operator-id <OPERATOR_UUID> # Turnkey's ID for your operator (from app create response)

# Alternative: provide manifest file and IDs manually
tvc deploy approve \
  --manifest manifest.json \
  --manifest-id <MANIFEST_UUID> \  # Turnkey's ID for the manifest (from deploy create response)
  --operator-id <OPERATOR_UUID>
```