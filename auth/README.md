# `auth`

Git SSH signing with a Turnkey Ed25519 private key.

## Instalation

```bash
# from the root of this repo run
cargo install -p auth
```

## Commands

```bash
auth config
auth public-key
auth git-sign
```

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

`auth config list` prints the fully resolved effective configuration, so environment-variable overrides appear in its output.

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

```bash
auth config set turnkey.organizationId "<org-id>"
auth config set turnkey.apiPublicKey "<api-public-key>"
auth config set turnkey.apiPrivateKey "<api-private-key>"
auth config set turnkey.privateKeyId "<ed25519-private-key-id>"

git config --global gpg.format ssh
git config --global gpg.ssh.program "$(which auth)"
git config --global user.signingkey "key::$(auth public-key)"
printf '%s %s\n' "you@example.com" "$(auth public-key)" >> ~/.config/git/allowed_signers
git config --global gpg.ssh.allowedSignersFile ~/.config/git/allowed_signers
```
