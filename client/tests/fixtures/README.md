`signature_digest_vectors.json` is shared with Mono's `core_traits` fixtures.
Generate the oracle output in Mono with
`cargo run --locked -p core_traits --example signature_digest`, supplying tuples
on stdin. It uses the real `Signature`, `HexDigest`, and Borsh implementations.
Field-mutation cases exercise exact string encoding. The `captured-*` cases
contain real SDK stamps for both curves; loopback tests verify their signatures.

Run `client/tests/entropy_failure.sh` from the repository to inject an OS entropy
failure into the real client path on macOS. The test arms the injector
after its loopback server starts. For both curves it checks zero POSTs before
the first send and exactly one POST when Pending precedes entropy failure.
It also checks submission state, the safe activity label, and isolation of the
next successful invocation. The injector is test-only.
