# TVC config files (scaffold, then edit)

`app create`, `deploy create`, and the local-quorum-key flow all read a JSON config file. The reliable pattern is **scaffold with `init`, then fill the sentinels**, rather than hand-authoring JSON. The `init` commands prefill real context (operator pubkey, last app ID, current QOS version) and drop `<FILL_IN_*>` sentinels where you must decide a value. `init` never overwrites an existing file.

Always treat the scaffolded file as the source of truth for the current schema, fields evolve, and the CLI's scaffold is authoritative for the version you have installed.

Scaffold-then-edit is a hard rule for `deploy.json`, not a preference: the parser has **no field defaults**. Eight of its eleven fields are required at parse time — `appId`, `qosVersion`, `pivotContainerImageUrl`, `pivotPath`, `expectedPivotDigest`, `healthCheckType`, `healthCheckPort`, `publicIngressPort` — and a hand-written file missing any of them fails with `command_error` ("missing field `...`"). Only `pivotArgs` (defaults `[]`), `dangerousDeployDebugMode` (defaults `false`), and `pivotContainerEncryptedPullSecret` (defaults absent) may be omitted. The 3000/HTTP defaults you see in scaffolded files live in `deploy init` and the flag-only path, not in the file parser.

## App config

```bash
tvc app init --output app.json     # reason: app_config_created
# open app.json, replace every <FILL_IN_*> sentinel (operator set / quorum params)
tvc app create --config-file app.json --message-format json
```

- `--config-file` / `-c` is required for `app create` (env `TVC_APP_CONFIG`).
- `--output` / `-o` is optional for `app init` (env `TVC_APP_CONFIG_OUT`); it defaults to `app.json`. Pass it explicitly in scripts so the path is deterministic.

### Wiring the operator into `app.json`

The scaffold prefills `manifestSetParams.newOperators[0].publicKey` with your profile's saved default operator when the profile resolves exactly one; otherwise it drops a `<FILL_IN_OPERATOR_PUBLIC_KEY>` sentinel. Branch on which you got:

- **Prefilled with a real key:** the profile already has a usable operator. Keep the scaffolded key and do **not** run `tvc operator create` — a freshly created operator's key would not be the one in the scaffold, so the app's manifest set would not contain it and it could never approve this app's deployments.
- **`<FILL_IN_OPERATOR_PUBLIC_KEY>` sentinel:** run `tvc operator create --message-format json` and paste the returned `compositePublicKey` (the `encryptPublicKey` and `signPublicKey` concatenated, emitted as one field) into `newOperators[0].publicKey`.

Either way, the identities allowed to approve this app's deployments are the `manifestSetOperatorIds` returned by `app create` — save those and pass one of them to `deploy approve --operator-id`. The `operatorId` printed by `operator create` is only valid for approval if that operator's public key actually made it into the manifest set. Note that on the prefilled path the approver id is unknowable before `app create` returns: the scaffold shows only a public key, and no command lists operators — so if asked for the operator id up front, answer that it arrives with `app create`.

## Deploy config

```bash
tvc deploy init --output deploy.json         # reason: deployment_config_created
# open deploy.json, set the pivot image + digest, ports, and args
tvc deploy create --config-file deploy.json --app-id <APP_ID> --message-format json
```

Almost every deploy field also has a CLI flag / env var; flags override the file. The one exception is `healthCheckType`, which can only be set in the config file. The knobs:

| Flag | Env | Meaning |
|---|---|---|
| `--app-id` | `TVC_APP_ID` | The app this deployment belongs to |
| `--pivot-image-url` | `TVC_PIVOT_IMAGE_URL` | OCI image URL for the enclave pivot |
| `--expected-pivot-digest` | `TVC_EXPECTED_PIVOT_DIGEST` | Expected digest of the pivot binary (integrity pin) |
| `--pivot-path` | `TVC_PIVOT_PATH` | Path to the pivot binary inside the image |
| `--pivot-args` | `TVC_PIVOT_ARGS` | Args passed to the pivot (repeatable) |
| `--qos-version` | `TVC_QOS_VERSION` | QOS version to run |
| `--health-check-port` | `TVC_HEALTH_CHECK_PORT` | Port the health check probes |
| *(none)* | *(none)* | `healthCheckType` is config-file only: `TVC_HEALTH_CHECK_TYPE_HTTP` (default) or `TVC_HEALTH_CHECK_TYPE_GRPC`. There is no flag and no env var, so it must be set in `deploy.json`. |
| `--public-ingress-port` | `TVC_PUBLIC_INGRESS_PORT` | Public ingress port |
| `--pivot-pull-secret` | `TVC_PIVOT_PULL_SECRET` | Pull secret for a private image |
| `--dangerous-deploy-debug-mode` | `TVC_DANGEROUS_DEPLOY_DEBUG_MODE` | Debug-mode deploy (logs tailable; never for prod) |

Seed a new deployment's config from an existing one:

```bash
tvc deploy init --output deploy.json --from-deployment <OLD_DEPLOY_ID>
```

## The two digests, which are different fields

A deployment pins **two** separate sha256 values. Conflating them yields a config that looks correct and fails at deploy time.

| Field | Hashes | Obtained from |
|---|---|---|
| the `@sha256:` in `pivotContainerImageUrl` | the **container image manifest** | the registry, after you push |
| `expectedPivotDigest` | the **pivot binary** inside that image | extract the binary at `pivotPath`, then sha256 it |

The CLI computes neither for you, so both are derived out of band. A wrong `expectedPivotDigest` is only caught at deploy time; nothing local validates it.

### What the image has to satisfy

These are requirements, not a toolchain recommendation:

- A **`linux/amd64` OCI image** (the enclave runtime requires that architecture), in a registry TVC can pull from.
- **Referenced by digest, not by tag alone.** Tags are mutable, and pinning is the point of a verifiable deployment.
- **One digest to pin.** If your build publishes a multi-arch index, you must select the `linux/amd64` child manifest rather than the index digest. Builders often have flags to suppress the index and attestation layers so there is exactly one.
- If the registry is private, supply a pull secret (`pivotContainerEncryptedPullSecret` / `--pivot-pull-secret`); if it is public, remove that placeholder from the scaffold entirely.

### Getting the values

Any OCI-compatible tooling works. Common choices, none required:

- **Image digest:** `docker buildx imagetools inspect`, `crane digest`, `skopeo inspect`, or a plain registry API request.
- **Pivot binary:** extract the file at `pivotPath` out of the image, then hash it. `docker create` + `docker cp`, `podman create` + `podman cp`, or `crane export` piped through `tar` all work. Hash with `sha256sum` or `shasum -a 256`.

Whatever you use, the binary you hash must be the one inside the image you are pinning. Hashing a local build artifact that was not the one published produces a mismatch that only surfaces when the enclave refuses to start.

## Quorum keys: Turnkey-hosted by default, local files only when self-provisioning

**Prefer the Turnkey-hosted path.** It is the default, it runs no `tvc keys` commands, and it writes no key material to disk:

- `tvc app init` scaffolds `app.json` with `quorumPublicKey` already prefilled — leave it as scaffolded.
- The scaffold writes `"shareSetParams": null` and `"shareSetId": null` — keep both as scaffolded (do not delete the keys or fill them in). A null/absent share set selects Turnkey's default hosted share set at `app create` time.
- The approver identity is the operator wired into `manifestSetParams` — see "Wiring the operator into `app.json`" above.

Keep those scaffold defaults unless the app requires a quorum key you generate and hold yourself. Only then use the local flow below.

### Alternative: local quorum key files (self-provisioned operator)

```bash
tvc keys init-local-quorum-key -o quorum_key.json       # -o / TVC_QUORUM_KEY_CONFIG_OUT, defaults to quorum_key.json
# edit quorum_key.json
tvc keys generate-local-quorum-key \
  --config-file quorum_key.json \
  --quorum-key-metadata-out quorum_key_metadata.json    # -c required; metadata-out defaults to quorum_key_metadata.json
```

`quorum_key_metadata.json` and any re-encrypted share files contain sensitive shares. Keep them out of source control and out of the working tree you might commit. Prefer writing them to a path outside the repo.
