# tkhq_api_key_stamper

This crate contains structs and utilities to work with P-256 keys, which [Turnkey](https://docs.turnkey.com/) uses as a primary way of authentication.

## Creating a new P-256 API key

```rust
use tkhq_api_key_stamper::TurnkeyP256ApiKey

let api_key = TurnkeyP256ApiKey::generate();
```

## Loading API keys from env

If you keep API keys in env vars, load it with `from_bytes`:

```rust
use std::env;
use tkhq_api_key_stamper::TurnkeyP256ApiKey;

// Assuming the env var is a hex-encoded string
let api_key_hex = env::var("TURNKEY_API_PRIVATE_KEY").expect("cannot load TURNKEY_API_PRIVATE_KEY");
let api_key = TurnkeyP256ApiKey::from_bytes(
    hex::decode().expect("decoding hex string failed"), None
).expect("loading API key failed");
```

If you want to store API keys in `.env` files, use [`dotenvy`](https://docs.rs/dotenvy/latest/dotenvy/).


## Load API keys from files

If you have generated API keys with Turnkey's [command-line tool](https://github.com/tkhq/tkcli) you can load them with:

```rust
use tkhq_api_key_stamper::TurnkeyP256ApiKey

let api_key = TurnkeyP256ApiKey::load_from_files("/path/to/key.priv", "/path/to/key.pub").expect("loading should succeed");
```

## Creating an API stamp to sign Turnkey requests

The API is straightforward, once you have a handle on an API key, call `stamp`:

```rust
let stamp = api_key.stamp("POST request body goes here");
```

The stamp produced is a base64-encoded value, ready to be used as a stamp header. See https://docs.turnkey.com/developer-reference/api-overview/stamps#api-keys for more information.

## Error handling

Errors are centralized in `StamperError`.
