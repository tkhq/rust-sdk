# Git signing

Use `auth` as Git's SSH signing program after configuring your Turnkey credentials.

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

After this setup, Git can use `auth git-sign` through the configured SSH signing program when creating signed commits or tags. It is invoked with `auth -Y` since that is how Git expects to invoke the given ssh program.
