# AGENTS.md

Guidance for changes to `turnkey_enclave_encrypt`.

## Secrets

- New APIs must zeroize secret material. Return decrypted plaintext as
  `Zeroizing<Vec<u8>>`, hold private keys and seeds in `Zeroizing` or a type
  deriving `ZeroizeOnDrop`, and do not make unzeroized copies (`.to_vec()`,
  `.clone()`, `String::from_utf8`, `hex::encode` on secret bytes).
- Existing APIs that return a plain `Vec<u8>` stay as they are for
  compatibility. Do not copy that pattern into new APIs.

## Ingress purpose bindings

- Every new activity type that encrypts a payload to the enclave (ingress)
  must bind the ciphertext to its purpose. Never ship a new ingress flow that
  passes an empty `purpose_binding`.
- Bind the activity with `purpose_binding::encode`, passing the
  `EncryptedIngressPurpose` proto enum string name for that activity. If the
  enclave stores or acts on request fields other than a name and key-value
  properties alongside the plaintext, add an encoder to `purpose_binding` that
  starts with the length-prefixed purpose, followed by every such field, each
  length-prefixed.
- Give each activity its own purpose string, so a binding for one activity
  never equals a binding for another.
- Expose a client helper that builds the binding from the request fields, so
  callers do not assemble binding bytes themselves.
- Add a test vector for the encoder and tests that decryption fails when any
  bound field changes.

## Compatibility

- Do not change the bytes of the existing AAD (`encapped_public ||
  receiver_public`) for an empty binding. Messages encrypted by released
  versions must still decrypt.
