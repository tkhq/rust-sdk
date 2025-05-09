# `turnkey_proofs`

This crate contains utilities to parse and verify Turnkey secure enclave proofs. As outlined in [the Turnkey whitepaper](https://whitepaper.turnkey.com) there are two types of proofs:
* App proofs, signing structured data with enclave ephemeral keys.
* Boot proofs, which are proofs that a given enclave was provisioned correctly. Boot proofs reference via their `public_key` field the enclave ephemeral key. This links App and Boot proofs together.

## Boot proofs

> 🚧 **Experimental**: Turnkey Boot proofs are not fully baked yet and may change significantly in the near future

If you have a Turnkey organization you can request a Boot proof from any enclave. This boot proof is an attestation document from Amazon, signed by a root certificate associated with AWS Nitro Attestation PKI (located in [`aws_root.pem`](./static/aws_root.pem)). This top-level certificate can be downloaded from <https://aws-nitro-enclaves.amazonaws.com/AWS_NitroEnclaves_Root-G1.zip>.

Resources on AWS Nitro Enclaves, attestations, and verifying attestations can be found at the following:

- <https://docs.aws.amazon.com/enclaves/latest/user/nitro-enclave.html>
- <https://docs.aws.amazon.com/enclaves/latest/user/set-up-attestation.html>
- <https://aws.amazon.com/blogs/compute/validating-attestation-documents-produced-by-aws-nitro-enclaves/>
- <https://docs.aws.amazon.com/enclaves/latest/user/verify-root.html>

### Usage

You may request a fresh attestation with the `turnkey` CLI (available [here](https://github.com/tkhq/tkcli)):
```sh
$ turnkey request --host api.turnkey.com --path /public/v1/query/get_attestation --body '{ "organizationId": "<your organization ID>", "enclaveType": "signer" }' --organization <your organization ID>

{
   "attestationDocument": "<base64-encoded attestation document>"
}
```

This crate contains a function to parse and verify this attestation: `parse_and_verify_aws_nitro_attestation`. This returns an `AttestationDoc` containing PCR values. You can display these values like so:

```rust,no_run
use hex;
use turnkey_proofs::parse_and_verify_aws_nitro_attestation;

let attestation_document = "<base64-encoded attestation doc>".to_string();
let attestation = parse_and_verify_aws_nitro_attestation(attestation_document)
   .expect("cannot parse and verify attestation document");

// Display PCR values
println!("PCR0: {}", hex::encode(attestation.pcrs.get(&0).unwrap()));
println!("PCR1: {}", hex::encode(attestation.pcrs.get(&1).unwrap()));
println!("PCR2: {}", hex::encode(attestation.pcrs.get(&2).unwrap()));
println!("PCR3: {}", hex::encode(attestation.pcrs.get(&3).unwrap()));

// Display user data and public key fields
println!("user_data: {}", hex::encode(attestation.user_data.unwrap()));
println!(
   "public_key: {}",
   hex::encode(attestation.public_key.unwrap())
);
```

Head over to the [QuorumOS](https://github.com/tkhq/qos) repository if you're looking to reproduce these PCR values independently.
