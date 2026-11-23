# Repository guidance

When working under `tvc/`, read and follow `tvc/AGENTS.md`.
When working under `enclave_encrypt/`, read and follow `enclave_encrypt/AGENTS.md`.

## Rust tests

Do not add table-driven tests in Rust. Give each independent case a named test
so the test runner can run cases in parallel and report failures by name. Share
setup or assertions through test helpers where useful.
