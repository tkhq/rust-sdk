# PURE_DEPS_PLAN_0 — Dependencies are parameters (SYS-116)

Audit and refactor plan for making rust-sdk's production code the golden
example of the "dependencies are parameters / pure functions preferred"
style. Scope: `tvc/` first (Part 1), then the rest of the workspace
(Part 2). No commits/pushes from this plan; all edits land locally for
human review.

Rule under enforcement: a function receives its dependencies fully
constructed; effectful acquisition (env, config, disk, clock, randomness,
network, prompts) happens at the wiring layer — `main` and each command's
top-level `run()` — and everything below is pure over injected values.
Passing raw fragments so a function can construct the real dependency
internally is the same antipattern one level removed.

---

## Part 1 — tvc audit

### 1.1 Systemic finding: no wiring layer

Effectful acquisition happens at every depth of the call stack. The same
config file is loaded up to four times in one command invocation.

| # | Site | Function | Problem | Severity |
|---|------|----------|---------|----------|
| T1 | `tvc/src/client.rs:47` | `build_client()` | Zero-arg: reads env vars, falls back to disk config, then constructs. 21 command call sites. Credentials travel as `(String, String, String, String)`. | high |
| T2 | `tvc/src/client.rs:102` | `load_credentials_from_config()` | Calls `Config::load()` internally; returns the stringly tuple. | high |
| T3 | `tvc/src/config/turnkey/mod.rs:373` | `config_dir()` | Reads `$HOME` at the bottom of the stack; every config consumer transitively depends on the environment. Tests must fake `HOME`. | high |
| T4 | `tvc/src/config/turnkey/mod.rs:405,426` | `Config::load()` / `save()` | Zero-arg path resolution inside; 19 call sites, many below entrypoints. Path-parameterized `load_from_path`/`save_to_path` already exist (good) but are private. | high |
| T5 | `tvc/src/operator.rs:409` | `resolve_operator()` | Calls `Config::load()` internally (twice on some paths) instead of taking `&Config`. | high |
| T6 | `tvc/src/local_operator_key.rs:47` | `resolve_local_operator()` | Loads config internally on the non-explicit path. 5 callers. | high |
| T7 | `tvc/src/commands/deploy/approve.rs:409` | `build_post_target()` | Loads config AND prompts interactively deep inside resolution logic. | high |
| T8 | `tvc/src/commands/deploy/approve.rs:489,588,782` | `run_with_resolved_inputs()` etc. | Builds clients mid-function via zero-arg `build_client()`. | medium |
| T9 | `tvc/src/commands/app/create.rs:158,232,261` | `load_saved_operator_ids()`, `load_saved_operator_public_key()`, `run_with_config()` | Three independent `Config::load()`s per invocation (+1 hidden in `build_client()`). | high |
| T10 | `tvc/src/commands/app/init.rs:107`, `tvc/src/commands/keys/init_local_quorum_key.rs:71`, `tvc/src/commands/app/create.rs:232` | `load_operator_public_key()` variants | Near-identical best-effort config loaders duplicated across commands, each acquiring independently. | high |
| T11 | `tvc/src/provisioning.rs:65` | `fetch_provisioning_details()` | Mixes clock read + network fetch + pure parse. The pure `provisioning_details_from_response` split already exists; the clock read is trapped above it. | medium |
| T12 | ~10 command sites (`app/create.rs:247`, `deploy/approve.rs:600`, `deploy/create.rs:434`, `deploy/delete.rs:33`, `deploy/post_share.rs:42`, `deploy/restore.rs:34`, `app/delete.rs:33`, `app/set_live_deploy.rs:35`, `keys/create_quorum_key.rs:135`, …) | inline `SystemTime::now()` for `timestamp_ms` | Clock reads inlined despite `operator.rs:606` `timestamp_ms()` helper existing. Acceptable at entrypoints but inconsistent and unnameable in tests. | low |

Codegraph blast radius: none of `build_client`, `resolve_operator`,
`resolve_local_operator`, `load_credentials_from_config`, `config_dir`
have covering tests today.

Existing good patterns inside tvc to preserve and extend:
`Config::load_from_path`/`save_to_path` (path-parameterized I/O),
`provisioning_details_from_response` (pure response parse),
`build_*_intent` functions (pure, unit-tested),
`validation_time_secs(Option<u64>)` override in provisioning,
`resolve_hosted_operator(config: &Config, …)` (already takes config).

### 1.2 Remaining tvc files

| # | Site | Function | Problem | Severity |
|---|------|----------|---------|----------|
| T13 | `tvc/src/commands/login.rs:561` | `find_or_generate_operator_key()` | Three levels below the entrypoint: reads the key file, generates a pair, writes to disk, detects TTY (`stdin_can_prompt` at :609 — the only unambiguous below-entrypoint TTY detection in the crate), prompts twice, copies files. None of its branching is testable without a real terminal and home dir. | high |
| T14 | `tvc/src/commands/login.rs:553` | `wait_for_dashboard_registration()` | Reads raw stdin (`std::io::stdin().lock().read_line`) bypassing the `prompts` module — cannot be stubbed. | high |
| T15 | `tvc/src/commands/login.rs:665` | `verify_credentials(api_key, org_id, api_base_url)` | Takes three raw fragments and builds the stamper + client internally before the whoami call — the raw-fragment antipattern verbatim; should receive a constructed client. | high |
| T16 | `tvc/src/commands/login.rs:311,512,239,283,391,431` | `execute_login`, `generate_api_key`, `resolve_profile_alias`, `build_login_plan_interactive`, `prompt_for_org_plan`, `prompt_for_new_org_inputs` | A de-facto second wiring layer below `run`: config saves, key generation fused to disk writes, prompts inside resolvers/builders, `$HOME` resolution fused to output assembly, raw `Option<&str>` fragments forwarded so callees construct the dashboard URL internally (:435). | medium |
| T17 | `tvc/src/commands/deploy/init.rs:63` | `execute()` | Loads `Config` at :96 **and again at :109**, plus a third transitive load via `build_client()` at :89; clock at :72; `exists()` probe; prompts; file write — every axis at once. | high |
| T18 | `tvc/src/commands/deploy/create.rs:164` | `build_inputs_interactive()` | `Config::load()` at :191 **inside the retry loop** — disk re-read on every failed validation pass; plus prompts and a config write below the entrypoint. | high |
| T19 | `tvc/src/config/deploy.rs:139` | `DeployConfig::fill_interactively()` | Interactive prompting inside a domain config type; the "which placeholder fields remain" logic is provably separable (`has_placeholders` exists) but untestable without a TTY. | high |
| T20 | `tvc/src/config/app.rs:114` | `AppConfig::fill_interactively()` | Same as T19; reached from four different commands. | high |
| T21 | `tvc/src/commands/keys/create_quorum_key.rs:154` | `resolve_operator_encrypt_keys()` | `Config::load()` inside a resolver; the adjacent pure `resolve_operator_ids(&config, …)` at :167 already proves the load hoists into `run`. | high |
| T22 | `tvc/src/commands/keys/backup_operator_key.rs:106` | `prompt_for_backup_destination()` | Prompts + fs probes below two different entrypoints (also called from `login.rs:626`). | medium |
| T23 | `tvc/src/commands/deploy/debug_logs.rs:181` | `query_debug_logs()` | Client injected (good) but the poll loop fuses dedupe/print accounting with real `tokio::time::sleep` — undriveable in tests. | medium |
| T24 | `tvc/src/operator.rs:114` | `create_hosted_operator(auth, spec)` | `auth` injected but calls `timestamp_ms()` internally — the clock is the lone holdout. | medium |
| T25 | `tvc/src/errors.rs:126` | `binary_name()` | Reads `std::env::args()` inside otherwise-pure message formatting; version-rejection rendering only testable by manipulating argv. | medium |
| T26 | `tvc/src/commands/operator/create.rs:101`, `tvc/src/commands/deploy/provision.rs:66` | `run()` | Entrypoint placement is legal, but each loads `Config` then calls `build_client()` which loads it again — the duplication defect is T1's root cause surfacing. | medium |
| T27 | `tvc/src/commands/app/init.rs:46,107` | `execute()`, `load_operator_public_key()` | Nullary acquirer below the entrypoint; one of the byte-identical duplicate trio (with `keys/init_local_quorum_key.rs:71` and `app/create.rs:232`). | high |
| T28 | `tvc/src/commands/confirmation.rs` | whole module | **Dead code**: zero call sites outside itself; duplicates `prompts::confirm`/`confirm_or_bail`. Delete candidate. | low |
| T29 | `tvc/src/cli.rs:151`, `tvc/src/output.rs:40`, `tvc/src/commands/deploy/create.rs:317`, `tvc/src/commands/keys/re_encrypt_local_share.rs:212`, `tvc/src/commands/keys/backup_operator_key.rs:135` | various | Borderline/low: argv read one hop below `Cli::run` (decision fn already parameterized+tested); TTY detection in `Shell::standard` (called once from `Cli::run`); prompt-and-write pairs; validation fused to copy in `back_up`. | low |

Config-loaded-more-than-once-per-invocation table (root cause = T1):
`deploy init` (×3), `deploy create` (×2, one in a loop), `deploy provision`
(×2), `operator create` (×2), `keys create-quorum-key` (×2),
`app create` (×4).

Additional tvc exemplars to preserve/cite:
`build_re_encrypted_share_output(metadata, bundle, &dyn Pair, flag)`
(`keys/re_encrypt_local_share.rs:129`) — every dependency injected
including the signer behind a trait object; best in-crate example of the
target shape. `build_re_encrypt_intent(…, validation_time_override:
Option<u64>)` (`deploy/provision.rs:117`) — injected clock.
`generate_local_quorum_key.rs` — run() owns all I/O, pure core.
Clean thin commands: `deploy/{post_share,delete,restore,status,get_status,
provisioning_details}.rs`, `app/{status,list,delete,set_live_deploy}.rs`.

---

## Part 2 — workspace audit (client, api_key_stamper, enclave_encrypt, proofs, codegen, examples)

| # | Site | Function | Problem | Severity |
|---|------|----------|---------|----------|
| W1 | `client/src/lib.rs:204-225` | `TurnkeyClientBuilder::build` | Constructs `reqwest::Client`, default headers (User-Agent), and 20s timeout internally; a finished `reqwest::Client` or `HeaderMap` cannot be injected — only a builder-mutating closure. | medium |
| W2 | `client/src/lib.rs:363-411` | `process_request` | Mixes pure request construction (URL, JSON, stamp) + HTTP send + pure response validation; all 12 tests need a MockServer. | medium |
| W3 | `client/src/lib.rs:306-346` | `process_activity` | Polling state machine welded to real `tokio::time::sleep`; retry-exhaustion tests burn wall clock. | medium |
| W4 | `client/src/lib.rs:349-355` | `current_timestamp` | Clock read on a `&self` method (the `&self` is unused), not overridable. Blast radius is small: codegraph confirms all 8 call sites are in `examples/` — no library or tvc code calls it, since generated methods already take `timestamp_ms` as a parameter (good). | low |
| W4b | `client/src/lib.rs:158` | `TurnkeyClientBuilder::new` | Acquires `reqwest::Client::builder()` in the constructor; mitigated by the `with_reqwest_builder` closure. Subsumed by W1. | low |
| W5 | `enclave_encrypt/src/lib.rs:221-240` | `encrypt` (private primitive) | Acquires `&mut OsRng` internally; no HPKE test vectors possible. Idiom is `rng: &mut impl CryptoRng + RngCore`. | medium |
| W6 | `enclave_encrypt/src/client.rs:340-415` | `EnclaveEncryptClient::encrypt` | Substantial pure verification logic welded to the nondeterministic seal (downstream of W5). | medium |
| W7 | `enclave_encrypt/src/client.rs:53-61` | `AuthenticationClient::dangerous_from_bytes` | "Deterministic" constructor secretly consumes entropy to fabricate a never-used dummy key. | medium |
| W8 | `enclave_encrypt/src/client.rs:563-579`, `server.rs:66,108` | reusable/server `encrypt` paths | Same OsRng root cause as W5. | medium |
| W9 | `proofs/src/lib.rs:236` | `parse_and_verify_aws_nitro_attestation` | Re-parses the embedded AWS root cert on every call with `.unwrap()`; trust anchor not injectable even though the layer below (`parse_and_verify_der_attestation`) takes `root_cert: &[u8]`. | medium |
| W10 | `proofs/src/lib.rs:241` | same | `duration_since(UNIX_EPOCH).unwrap()` can panic on caller-supplied pre-epoch time. | low |
| W11 | `api_key_stamper/src/lib.rs:128-146` | `from_files` | Named loader mixing fs reads with construction; delegates parsing to pure `from_strings` — borderline acceptable. No `Secp256k1` counterpart (asymmetry). | low |
| W12 | `examples/src/bin/proofs.rs:12-56` | `create_wallet`/`delete_wallet` helpers | Helpers read the clock AND build intents/assert results; `main` should pass `timestamp_ms`. | low |
| W13 | `examples/src/lib.rs:8-16,32-46` | `load_api_key_from_env`/`load_base_url_from_env` | Dotenv bootstrap duplicated verbatim in both; silently depends on cwd; panics on `current_dir`. Hoist into a single `init_env()` called from `main`. | low |
| W14 | `proofs/src/lib.rs:383-389` | `verify` | `.expect(…)` panics on missing user_data/public_key instead of returning `VerifyError` — failure modes untestable. | low |
| W15 | `api_key_stamper/src/lib.rs:171,261` | `stamp` impls | `serde_json::to_string(&stamp).unwrap()` — panic inside otherwise-pure header construction; return `StamperError`. | low |
| W16 | `enclave_encrypt/src/client.rs:90-95,109-121,180-186,199-211` | `ExportClient`/`ImportClient` constructors | Panic (`.expect`) on a caller-supplied quorum key instead of returning `Result`. | low |
| W17 | `codegen/src/main.rs:298-300` | `main` | **Latent bug found during audit:** `create_dir_all("../client/src/generated")` disagrees with `GENERATED_CLIENT_DIR = "client/src/generated"` used for the write — only works because tonic_build already created the dir. Fix: `create_dir_all(&out_dir)`. | low |
| W18 | `examples/src/bin/sub_organization.rs:96` | `main` | Timestamp read off the parent client for a request sent by the sub-org client — harmless but confusing. | low |

Acceptable by the generation-constructor carve-out (document, don't change):
`TurnkeyP256ApiKey::generate()` / `TurnkeySecp256k1ApiKey::generate()`
(`api_key_stamper/src/lib.rs:73,188`), `AuthenticationClient::new`,
`EnclaveEncryptClient::from_enclave_auth_key`,
`EnclaveEncryptServer::from_enclave_auth_key` (fresh-key constructors).
Optionally add `generate_with(rng)` variants later.

Clean files (no action): `client/src/retry.rs`, `client/src/well_known.rs`,
`proofs/src/syntactic_validation.rs`, `proofs/src/types.rs`,
`proofs/src/error.rs`, `enclave_encrypt/src/errors.rs`,
`enclave_encrypt/src/quorum_public_key.rs`, `codegen/src/transform.rs`,
`examples/src/bin/{whoami,wallet,sub_organization}.rs`.

Workspace exemplars to cite in the AGENTS.md rule:
- `proofs/src/lib.rs:228-231` — `validation_time: Option<SystemTime>` with documented `now()` default.
- `proofs/src/lib.rs:345-349` — caller derives validation time from the data under verification instead of the clock; `verify` is fully deterministic and tested with zero mocks.
- `client/src/retry.rs:43-67` — pure `compute_delay(&self, attempt)`; `Duration`s computed here, consumed elsewhere; unit-tested with no I/O.
- `api_key_stamper/src/lib.rs:163-177` — `stamp()` returns a `StampHeader` data struct; the HTTP layer attaches it. Signing decoupled from transport.
- `enclave_encrypt/src/client.rs:308` + `server.rs:49` — `*_and_target_key` injectable counterparts to the OsRng constructors; exactly how tests get deterministic e2e coverage. The pairing convention (`from_X` generates, `from_X_and_target_key` injects) is already established — the encrypt paths just need the same treatment.
- `enclave_encrypt/src/quorum_public_key.rs:48,58` — named constant-selectors (`production_signer()`/`preprod_signer()`) for an embedded trust anchor, passed *as parameters* by callers with a fully general `from_bytes` escape hatch. The right way to handle an embedded trust anchor — direct contrast with W9.
- `api_key_stamper/src/lib.rs:52-54` — the `Stamp` trait: the signing dependency is a trait parameter threaded through `TurnkeyClient<S: Stamp>`, which is why the client's crypto is fully swappable in tests.
- `codegen/` overall — the best-separated crate in the repo: all I/O confined to `main()`, every piece of real logic a pure `&str`-in/value-out function (`parse_rpcs`, `transform::transform`, `validate_activity_version_caps`, …). In-repo proof that this style is achievable here.
- Generated activity methods take `timestamp_ms: u128` as a parameter (`codegen/src/main.rs:216,236`) — the clock is genuinely per-request; `current_timestamp()` is opt-in convenience.

Highest-leverage workspace fixes: W5 (one rng signature change fixes W6–W8
across the crate), W9 (removes the last hardcoded dependency from an
otherwise exemplary verification path), W3 (unblocks deterministic testing
of the retry loop whose policy half is already pure).

---

## Part 3 — refactor plan (tvc first)

Principle for every phase: the pure core moves down, acquisition moves up.
Signatures change from "no args, acquires inside" to "takes the finished
value". Each phase compiles and passes `cargo fmt` + `cargo clippy` +
`cargo test` independently and is separately reviewable.

### Phase A — credentials & client seam (`tvc/src/client.rs`)

The golden example the ticket asks for.

> **STATUS: implemented, awaiting human review** (2026-08-12). Gate: clippy
> clean, 249 unit + all integration suites green, no integration test
> touched. T2 annotation removed (fully fixed); T1 annotation retained,
> rescoped to the remaining zero-arg acquisition that Phase C parameterizes
> (the 21 call sites' hidden config load). Deviation from the
> sketch below: `Credentials` uses plain `String` fields (open question 1
> resolved toward minimal diff); `build_client()` keeps its zero-arg
> signature as the wiring composition so the 21 command call sites are
> untouched until Phase C.

1. Introduce a `Credentials` struct (org_id, api_base_url, api_key_public,
   api_key_private — consider domain wrappers per AGENTS.md typing rules)
   replacing the 4-string tuple.
2. `load_credentials_from_env_vars() -> Result<Option<Credentials>>` stays
   effectful but becomes: pure `credentials_from_env_values(...)` over a
   read snapshot + thin env read. The partial-missing-vars logic becomes
   unit-testable without setting process env.
3. `load_credentials_from_config(config: &Config) -> Result<Credentials>`
   — takes the loaded config; stops loading internally.
4. `build_authed_client(creds: &Credentials) -> Result<AuthenticatedClient>`
   — pure construction (infallible except key parsing).
5. `build_client()` remains as the single wiring composition used by
   command entrypoints: acquire (env → config) then construct. Its body
   becomes 5 lines of composition; everything under it is pure.
6. Header assembly: `fn tvc_client_headers() -> HeaderMap` (pure) passed
   into `build_turnkey_client(stamper, api_base_url, headers)` — the exact
   shape from the PR #235 discussion, now the canonical example.

### Phase B — config path injection (`tvc/src/config/turnkey/mod.rs`)

1. Promote `load_from_path`/`save_to_path` to the primary API; `Config`
   remembers the path it was loaded from (or callers pass it to `save`).
2. `config_dir()` becomes a wiring-layer concern: resolve once (in
   `main`/`run()` or a small `ConfigPaths` value), pass down. `$HOME`
   is read in exactly one place.
3. Keep zero-arg `Config::load()` as a thin wiring convenience that
   composes `config_dir()` + `load_from_path` — but nothing below an
   entrypoint may call it (enforced in Phase D).

### Phase C — resolvers take `&Config`; commands load once

1. `resolve_operator(config: &Config, …)`, `resolve_local_operator(config:
   &Config, …)` — drop internal `Config::load()`; callers (entrypoints)
   load once and pass down. `resolve_hosted_operator` already has this
   shape — cite it.
2. `build_post_target(config: &Config, …)` — config in; move the
   interactive prompt decision up or inject a chooser resolved at the
   entrypoint.
3. Dissolve the duplicated `load_saved_operator_*` helpers (byte-identical
   trio: `app/init.rs:107`, `keys/init_local_quorum_key.rs:71`,
   `app/create.rs:232`): entrypoints load `Config` once, one pure helper
   extracts (`saved_operator_public_key(config: &Config) -> Option<…>`).
   `app create` goes from 4 config loads to 1; `deploy init` from 3 to 1
   (T17); `resolve_operator_encrypt_keys` load hoists into `run` (T21).
4. `fetch_provisioning_details(auth, deployment_id, fetched_at_unix_ms)` —
   clock read moves to the caller; the fetch + existing pure parse remain.
   Same for `create_hosted_operator` (T24).
5. Standardize `timestamp_ms()` (move from `operator.rs:606` to a
   neutral module) and use it at every entrypoint that stamps time.
6. `binary_name()` (T25): message formatters take the name as a parameter;
   argv is read once at the `cli.rs` layer.

### Phase D — interactivity moves to the edge

The second systemic cluster: prompting embedded in domain types and
helpers instead of at entrypoints.

1. `DeployConfig::fill_interactively` / `AppConfig::fill_interactively`
   (T19, T20): split into a pure "what's missing" step on the type
   (`missing_fields(&self) -> Vec<Field>` — partially exists already) and
   a prompting step at the command layer that folds answers back in. The
   domain types lose their `prompts::` imports entirely.
2. `login.rs` (T13–T16): restructure `run` into the plan/execute shape the
   file already gestures at (`build_login_plan_*`): all prompts, TTY
   detection, and stdin reads happen while building the plan at the top;
   `execute_login` becomes effectful-but-prompt-free execution of a
   complete plan; `verify_credentials` takes a constructed client;
   `find_or_generate_operator_key` splits into pure decision + injected
   key-file I/O + prompts hoisted to the plan step.
3. `build_inputs_interactive` (T18): hoist `Config::load` out of the retry
   loop; the loop prompts over an in-memory value only.
4. `prompt_for_backup_destination` (T22): keep as a shared prompt helper
   (it is prompt-layer code) but move the fs probes (`is_dir`/`exists`)
   into a pure validation function both callers use.
5. `query_debug_logs` (T23): extract the pure poll-step decision
   (what's new since the cursor, what to print, when to stop); the async
   shell keeps the sleep.
6. Delete dead `commands/confirmation.rs` (T28).

### Phase E — workspace items + guardrails

1. `client` crate: `TurnkeyClientBuilder::http_client(reqwest::Client)` and
   `default_headers(HeaderMap)` injection points (W1); split
   `process_request` into pure build/parse + thin await (W2); extract pure
   `next_action` polling decision (W3).
2. `enclave_encrypt`: thread `rng: &mut impl CryptoRng + RngCore` through
   the encrypt paths (W5–W8, one signature change fixes the crate); fix
   the dummy-key constructor (W7); `Result` instead of panics (W16).
3. `proofs`: injectable root cert with embedded default; error instead of
   unwrap/expect (W9, W10, W14).
4. Small fixes: codegen path bug (W17), examples helpers (W12, W13, W18),
   stamp unwraps (W15).
5. Guardrail (from SYS-116 item 3, only the piece that protects this
   refactor): `clippy.toml` `disallowed-methods` for `std::env::var` and
   friends outside wiring modules, so the antipattern can't silently
   return.

### Sequencing & gates

- Phase A → B → C are ordered (each builds on the previous seam).
  Phase D depends on C (commands must already load config once).
  Phase E is independent and can land in any order after A.
- Human review gate after each phase; no phase starts until the previous
  one's diff is approved.
- Each phase adds unit tests for the newly-pure functions — these seams
  currently have zero covering tests, so the tests are both the payoff
  and the safety net.

## Part 4 — regression safety

Test surface today: 244 unit test fns in `tvc/src` (mostly `mod tests` on
pure functions — intents, error rendering, parsing), 140 integration test
fns across 26 binaries in `tvc/tests/` (fake `$HOME` via
`common.rs::write_profiles_config`, dead-port API base URL so commands
stop at the first network step, PTY tests for interactive flows, plus
`message_format.rs` / `error_output.rs` / `non_interactive.rs` /
`auth_env.rs` asserting observable CLI behavior), and wiremock tests in
the client crate.

Rules for every phase:

1. **Integration tests are the contract and must not change.** They pin
   observable behavior (output, exit codes, files written, error text)
   through the real binary. A phase that needs an integration-test edit
   has changed behavior — stop and surface it for review, don't update
   the test.
2. **Unit-test assertions/goldens/snapshots never change.** Only
   construction plumbing inside tests may change mechanically when a
   signature it calls gains a parameter (e.g. the `build_turnkey_client`
   wiremock test gains a `headers` argument; its assertion — the version
   header rides every request — stays identical).
3. **Compiler-driven migration.** Every signature change breaks all call
   sites at compile time; there is no silent partial migration. Phases
   are sized so the crate never sits in a half-migrated state.
4. **Phase gate:** `cargo fmt --check`, `cargo clippy`, full
   `cargo test --workspace`, plus a binary smoke run (fake HOME + local
   whoami stub, the existing acceptance-script pattern) before review.
5. **Characterization first where coverage is thin.** The seams being
   refactored have no direct unit tests; before changing one, add tests
   that lock current behavior of the pure logic being extracted
   (env-var partial-missing errors, credential extraction, operator
   resolution branching). Net coverage strictly increases per phase.
6. **Enumerated semantic risks the compiler can't catch:**
   - Collapsing N config loads into one changes *failure* semantics:
     best-effort loaders (`load_saved_operator_ids` swallows errors)
     must stay best-effort when fed from a single hoisted load that is
     itself required elsewhere — preserve per-consumer error policy.
   - Hoisting acquisition earlier can reorder failures. AGENTS.md
     requires arg-validation failures before config/auth/network work;
     `error_output.rs`/`non_interactive.rs` pin some of this ordering.
     Acquisition moves up only to the point after parse-level
     validation, never above it.
   - Prompt sequencing in interactive flows is pinned by the PTY tests;
     Phase D changes *where* prompts run, and must keep the same
     question order and wording.
7. **Optional belt-and-suspenders:** a before/after E2E diff — run a
   fixed command matrix with `--message-format json` under a fake HOME
   on the base commit and on the phase branch, and diff stdout/stderr/
   exit codes. Catches anything the suites don't assert on.

## Open questions for review

1. Phase A: plain `String` fields on `Credentials`, or domain wrappers
   (e.g. `OrgId`) now? Wrappers touch more call sites.
2. Phase B: should `Config` carry its source path (self-saving), or should
   `save` take the path? Carrying the path couples runtime type to
   persistence; passing it keeps the type pure but threads a value.
3. Phase E enclave_encrypt rng threading changes public API of a published
   crate — semver-major. Gate behind the release train or do additive
   `*_with_rng` variants instead?
4. How far to take W1 (client builder injection) given the crate is
   published and the closure escape hatch exists?
5. Phase D login restructure (T13–T16) is the largest single-file change
   in the plan and touches the onboarding flow shipped in recent releases
   (operator key backup during login). Full plan/execute restructure, or
   minimal hoisting of prompts/TTY detection first?
6. Delete `commands/confirmation.rs` (T28) in this work, or ticket it
   separately? (It is dead code either way.)
