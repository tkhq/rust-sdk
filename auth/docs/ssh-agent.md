# SSH agent

Run `auth` as a foreground SSH agent when you want plain `ssh` to authenticate with your Turnkey Ed25519 key.

```bash
auth config set turnkey.organizationId "<org-id>"
auth config set turnkey.apiPublicKey "<api-public-key>"
auth config set turnkey.apiPrivateKey "<api-private-key>"
auth config set turnkey.privateKeyId "<ed25519-private-key-id>"
```

Use two terminals:

Terminal 1:

```bash
auth ssh-agent --socket /tmp/auth.sock
```

Terminal 2:

```bash
export SSH_AUTH_SOCK=/tmp/auth.sock

ssh-add -L
ssh user@host
```

`ssh-add -L` should print the Turnkey-backed OpenSSH public key while `ssh user@host` uses the agent socket for signing.

If you only need Git commit or tag signing, use the [Git signing guide](git-signing.md) instead.
