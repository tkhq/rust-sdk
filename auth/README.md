# `turnkey-auth`

- Git SSH signing with a Turnkey Ed25519 private key.
- SSH agent for signing SSH requests with a Turnkey Ed25519 private key.

> Warning: `turnkey-auth` is experimental and has not been audited.

## Installation

```bash
# from the root of this repo run
cargo install -p turnkey-auth
```

The installed binary is named `auth`.

## Commands

```bash
auth config
auth public-key
auth git-sign
auth ssh-agent
```
wher
## Configuration

`auth` resolves configuration in this order:

1. Environment variables
2. Global config file
3. Built-in defaults

The default global config file path is:

```bash
~/.config/turnkey/auth.toml
```

You can inspect or update config with:

```bash
auth config list
auth config get turnkey.organizationId
auth config set turnkey.organizationId "<org-id>"
auth config set turnkey.apiPublicKey "<api-public-key>"
auth config set turnkey.apiPrivateKey "<api-private-key>"
auth config set turnkey.privateKeyId "<ed25519-private-key-id>"
auth config set turnkey.apiBaseUrl "https://api.turnkey.com"
```

`auth config list` prints the fully resolved effective configuration, so environment-variable overrides appear in its output. Secret values such as `turnkey.apiPrivateKey` are redacted in both `config list` and `config get`.

### Environment Overrides

```bash
export TURNKEY_ORGANIZATION_ID="<org-id>"
export TURNKEY_API_PUBLIC_KEY="<api-public-key>"
export TURNKEY_API_PRIVATE_KEY="<api-private-key>"
export TURNKEY_PRIVATE_KEY_ID="<ed25519-private-key-id>"
export TURNKEY_API_BASE_URL="https://api.turnkey.com" # optional
```

These environment variables override values stored in the global config file. This can be helpful for CI.

## Git setup

Use `turnkey-auth` as Git's SSH signing program after configuring your Turnkey credentials. The full setup guide lives in [Git signing](docs/git-signing.md).
