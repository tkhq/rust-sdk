# `turnkey_proofs`

This crate contains utilities to parse and verify Turnkey secure enclave proofs. To learn more about Turnkey verification, check out our [Turnkey Verified docs](https://docs.turnkey.com/security/turnkey-verified). As outlined in that doc, there are two types of proofs:
* App proofs, signing structured data with enclave ephemeral keys.
* Boot proofs, which are proofs that a given enclave was provisioned correctly. Boot proofs reference via their `public_key` field the enclave ephemeral key. This links App and Boot proofs together.

## Boot proofs

Boot Proof: a proof that a particular AWS Nitro Enclave booted with a particular configuration.

A boot proof contains
- AWS attestation document, which contains PCR measurements, a ceritifaction chain that proves the document was signed by AWS's root cert, a public key which is the ephemeral key unique to this particular enclave, and hash of the QOS Manifest
- A signed QOS Manifest, the validity of which is attested to by the Attestation Document. A hash of the application binary, the quorum public key, and more.

Resources on AWS Nitro Enclaves, attestations, and verifying attestations can be found at the following:

- <https://docs.aws.amazon.com/enclaves/latest/user/nitro-enclave.html>
- <https://docs.aws.amazon.com/enclaves/latest/user/set-up-attestation.html>
- <https://aws.amazon.com/blogs/compute/validating-attestation-documents-produced-by-aws-nitro-enclaves/>
- <https://docs.aws.amazon.com/enclaves/latest/user/verify-root.html>


## App Proofs

App Proof: a signature by an enclave ephemeral key to prove application-specific facts about functionality. An app proof, when combined with a boot proof, proves that your request was process:
- in the context of your Turnkey organization 
- with Turnkey’s signer application
- inside of a legitimate and precise version of QuorumOS
- inside of a legitimate AWS Nitro Enclave
- inside Turnkey’s canonical AWS production account

## Usage

### Verifying App Proofs

Given an app proof, you can request the boot proof for that app proof using `get_boot_proof_for_app_proof`.

To verify the app proof in conjunction with the boot proof, you call `verify(appProof, bootProof)`. 
This verification goes through the following steps:
 - Verify app proof signature
 - Verify the boot proof
   - Attestation doc was signed by AWS
   - Attestation doc's `user_data` is the hash of the qos manifest
 - Verify the app proof / boot proof connection - that the ephemeral keys match

### Attestation Document Verification

If you have a Turnkey organization you can request a an attestation document from Amazon, signed by a root certificate associated with AWS Nitro Attestation PKI (located in [`aws_root.pem`](./static/aws_root.pem)). This top-level certificate can be downloaded from <https://aws-nitro-enclaves.amazonaws.com/AWS_NitroEnclaves_Root-G1.zip>.

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
let attestation = parse_and_verify_aws_nitro_attestation(attestation_document, None)
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
