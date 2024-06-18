# Turnkey Rust SDK

This repository contains tooling to interact with the Turnkey API using Rust. 

## Usage

The [examples](/examples) directory shows typical usage. eg to [list the wallets](/examples/list_wallets.rs) in
a turnkey organisation: 

set the following envirtoment variables (or create a .env file):

```
TURNKEY_BASE_URL=https://api.turnkey.com
TURNKEY_API_PUBLIC_KEY=___my_api_public_key__
TURNKEY_API_PRIVATE_KEY=__my_api_private_key__
TURNKEY_ORGANIZATION_ID=__my_organstion_id__
```

Then run:

```
 cargo run --example list_wallets
```

## Generating code from protos

To generate code from the checked-in protos, ensure that protoc is on your path, then:
```
OUT_DIR=src/gen cargo run --bin generate --features build_deps
```
