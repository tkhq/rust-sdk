# `turnkey_enclave_encrypt`

This crate contains primitives to encrypt and decrypt data, sent to and from Turnkey secure enclaves (see Enclave to [end-user secure channels](https://docs.turnkey.com/security/enclave-secure-channels)).

Encryption and decryption are "one-shot" using the HPKE standard ([RFC 9180](https://datatracker.ietf.org/doc/rfc9180/)).  Neither the client or the server should ever be reused to send/receive more than one message. We want to avoid the recipient target key being used more then once in order to improve forward secrecy; see [security profile](#security-profile) section for important details and caveats.

The flows where encryption and decryption are relevant are:
* Authentication: [Email](https://docs.turnkey.com/authentication/email), [SMS](https://docs.turnkey.com/authentication/sms), or [Social](https://docs.turnkey.com/authentication/social-logins) logins return encrypted authentication bundles produced by Turnkey secure enclaves
* Key or Wallet export: key material (private key or wallet mnemonic phase) are encrypted by Turnkey enclaves to end-user public keys. See [Export Wallets and Keys](https://docs.turnkey.com/wallets/export-wallets#export-wallets-and-keys).
* Key or Wallet import: Turnkey enclaves send a bundle containing a signed public key, to which end-users encrypt their key material (private key or wallet mnemonic phrase). See [Import Wallets and Keys](https://docs.turnkey.com/wallets/import-wallets).

## Usage

### Authentication bundles

```rust,no_run
use turnkey_enclave_encrypt::{AuthenticationClient, QuorumPublicKey};

let mut client = AuthenticationClient::new();
let target_public_key = client.target_public_key(); // can be used in auth activity params

let bundle = "<auth bundle>";

let decrypted = client.decrypt(bundle).expect("decryption should succeed");
```

If you persist client key material between initiation and decryption you may use `dangerous_from_bytes` to create an `AuthenticationClient`. Keep in mind the one-shot encryption semantics, you may not use the same client IKM to decrypt many different bundles.

```rust,no_run
use turnkey_enclave_encrypt::{AuthenticationClient, QuorumPublicKey};

let client_ikm = hex::decode("...private bytes from client...").expect("cannot decode client secret bytes");
let bundle = "<auth bundle goes here, it's a base58-encoded string>";

let decrypted = AuthenticationClient::dangerous_from_bytes(client_ikm)
    .decrypt(bundle)
    .expect("decryption should succeed");
```

### Export bundles

To decrypt an exported private key or wallet, use `decrypt_private_key` or `decrypt_wallet`:

```rust,no_run
use turnkey_enclave_encrypt::{ExportClient, QuorumPublicKey};

let mut client = ExportClient::new(&QuorumPublicKey::production_signer());

// Decrypt a wallet (result: string)
let wallet_mnemonic_phrase = client.decrypt_wallet_mnemonic_phrase("<bundle>", "<organization id>");

// Decrypt a private key (result: bytes)
let private_key = client.decrypt_private_key("<bundle>", "<organization id>");
```

### Import bundles

To encrypt private keys or wallets, use `encrypt_private_key_with_bundle` or `encrypt_wallet_with_bundle`. The resulting value is a string, ready to use as an activity param. The organization and user IDs need to match the organization and user who initiated import.

```rust,no_run
use turnkey_enclave_encrypt::{ImportClient, QuorumPublicKey};

let mut client = ImportClient::new(&QuorumPublicKey::production_signer());

// Encrypt private keys
let encrypted_key = client.encrypt_private_key_with_bundle(
    "<bytes to encrypt>",
    "<bundle>",
    "<organization id>",
    "<user id>",
).expect("encryption should succeed");

// Encrypt wallet seed phrase
let encrypted_wallet = client.encrypt_wallet_with_bundle(
    "<mnemonic phrase>",
    "<bundle>",
    "<organization id>",
    "<user id>",
).expect("encryption should succeed");
```

## Running test

```sh
cargo test
```

## HPKE Protocol Details

### Terms

- Encapsulated ("Encapped") Key - the public key of the sender used for ECDH.
- Target Key Pair - the key pair of the receiver that the sender encrypts to the public key of. Only one message should ever be encrypted to the public key.
- Server - a server inside of the enclave; normally an enclave application.
- Client - a client outside of the enclave; normally a turnkey end user.
- Enclave Auth Key Pair - a key pair derived from the quorum master seed specifically for the purpose of authentication with clients.

### Overview

This protocol builds on top of the HPKE standard ([RFC 9180](https://datatracker.ietf.org/doc/html/rfc9180)) by adding recipient pre-flight authentication so the client can verify it is sending ciphertext to a turnkey controlled enclave and the enclave can verify its sending ciphertext to the correct client. See the [security profile](#security-profile) section more details.

### HPKE Configuration

* KEM: `KEM_P256_HKDF_SHA256`
* KDF: `KDF_HKDF_SHA256`
* AEAD: `AEAD_AES256GCM`
* INFO: `b"turnkey_hpke"`
* ADDITIONAL ASSOCIATED DATA: `EncappedPublicKey||ReceiverPublicKey`

### Protocol Flow

#### Server to Client

1. Client generates target pair and sends clientTargetPub key to server. The authenticity of the clientTargetPub is assumed to have been verified by the Ump policy engine.
1. Server computes ciphertext, `serverEncappedPub` = `ENCRYPT(plaintext, clientTargetPub)` and clears `clientTargetPub` from memory.
1. Server computes `serverEncappedPub_sig_enclaveAuthPriv` = `SIGN(serverEncappedPub, enclaveAuthPriv)`.
1. Server sends `(ciphertext, serverEncappedPub, serverEncappedPub_sig_enclaveAuthPriv)` to client.
1. Client runs `VERIFY(serverEncappedPub, serverEncappedPub_sig_enclaveAuthPriv)`.
1. Client recovers plaintext by computing `DECRYPT(ciphertext, serverEncappedPub, clientTargetPriv)` and the client target pair is cleared from memory. If the target pair is used multiple times we increase the count of messages that an attacker with the compromised target private key can decrypt.

Note there is no mechanism to prevent a faulty client from resubmitting the same target public key.

#### Client to Server

1. Client sends request to server for target key.
1. Server generates server target pair and computes `serverTargetPub_sig_enclaveAuthPriv` = `SIGN(serverTargetPub, enclaveAuthPriv)`.
1. Server sends `(serverTargetPub, serverTargetPub_sig_enclaveAuthPriv)` to client.
1. Client runs `VERIFY(serverTargetPub, serverTargetPub_sig_enclaveAuthPriv)`.
1. Client computes ciphertext, `clientEncappedPub` = `ENCRYPT(plaintext, serverTargetPub)` and clears `serverTargetPub` from memory.
1. Client sends `(ciphertext, clientEncappedPub)` to server and the client is cleared from memory.
1. Server assumes the authenticity of `clientEncappedPub` has been verified by the Ump policy engine.
1. Server recovers plaintext by computing `DECRYPT(ciphertext, clientEncappedPub, clientTargetPriv)` and server target pair is cleared from memory. If the target pair is used multiple times we increase the count of messages that an attacker with the compromised target private key can decrypt.

### Security profile

#### Receiver pre-flight authentication

We achieve recipient authentication for both the server and client:

- **Client to Server**: client verifies that the server's target key is signed by the enclaveAuth key. This check is critical for import/export flows. If the client accepts key material (e.g. a wallet seed) from a malicious party, they might not realize they have the wrong wallet (compromised seed because known or with bad entropy). If the client encrypts their seed to a malicious party, they lose funds directly. This is NOT required for email recovery and authentication flows: the client can afford to decrypt and use a bad API key. A bad API key will simply produce an invalid signature when used.
- **Server to Client**: server relies on upstream checks by Ump + activity signing scheme to enforce rules that guarantee authenticity of the client's target key. Specifically, when the client "sends" clientTargetPub it actually submits a signed payload (activity), and that payload must be signed with an existing credential persisted in org data.

#### Forward secrecy

The underlying HPKE spec does not provide forward secrecy on the recipient side since the target key can be long lived. To improve forward secrecy we specify that the target key should only be used once by the sender and receiver. We cannot enforce this strictly on the client-side because a client may choose to reuse their key. We could implement timestamp-based validation or rate limiting client-side but it wouldn't be a complete solution. For now we accept that a client can use an encryption bundle multiple times if it so desires. However we enforce one-time use of the key pair on the enclave side by deleting it once a successful decryption happens.

#### Sender authentication

We use `OpMode` Base because the sender's KEM private key is not long lived and thus does not need HPKE authentication. In order for this to be exploited one side's private key data would have to be leaked or an attacker would need to spoof a message from the sender. Turnkey mitigates this attack by layering a signature from an authentication key over payloads that contain ciphertext + encappedPub. Note that in the case of client to server the authentication signature is implicitly verified by the Ump policy engine. Read more about HPKE asymmetric authentication [here](https://datatracker.ietf.org/doc/html/rfc9180#name-authentication-using-an-asy).
