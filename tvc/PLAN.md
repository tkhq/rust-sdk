# Machine-readable output — `--format <text|json>`

Add a global `--format` flag and machine-readable JSON output to every `tvc`
command, in a modular way that keeps business logic, text rendering, and JSON
serialization separate. The unlock is scripting/CI:
`tvc deploy create … --format json | jq -r '.deploymentId'`.

## Design

### The global flag

Add to the `Cli` struct in `cli.rs`:

```rust
#[arg(long, global = true, default_value = "text", value_enum, env = "TVC_FORMAT")]
pub format: OutputFormat,   // Text | Json
```

`global = true` lets it appear before or after the subcommand
(`tvc --format json app status …` and `tvc app status … --format json` both
work). Parsed once in `Cli::run()`, then an `Emitter` is constructed and
threaded into every command's `run()`.

### `src/output.rs` — the shared machinery

- `enum OutputFormat { Text, Json }` (clap `ValueEnum`).
- `struct Emitter { format }` exposing:
  - `progress(&self, msg)` → writes human progress chatter to **stderr**;
    suppressed in JSON mode. Replaces the in-command progress `println!`s
    (e.g. `"Creating app..."`, `"Verifying credentials..."`).
  - `emit<T: Report>(&self, &T)` → the **single result exit point**:
    `serde_json::to_writer(stdout)` in JSON mode, `report.render_text(stdout)`
    in text mode.
- `trait Report: Serialize { fn render_text(&self, w: &mut impl Write) -> io::Result<()>; }`

### Per-command pattern (no per-command format branching)

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateDeployReport { deployment_id: String, app_id: String }

impl Report for CreateDeployReport {
    fn render_text(&self, w: &mut impl Write) -> io::Result<()> { /* today's println!s */ }
}

pub async fn run(args: Args, out: &Emitter) -> Result<()> {
    out.progress("Creating deployment...");   // → stderr
    let report = /* business logic builds the struct */;
    out.emit(&report)                          // JSON or text, decided centrally
}
```

Business logic builds a typed struct; rendering is uniform behind the trait.
The text path reproduces today's exact output, so existing tests keep passing.

### Cross-cutting rules

1. **stdout is sacred in JSON mode.** Exactly one JSON document on stdout.
   All progress chatter goes to stderr via `Emitter::progress`.
2. **`--format json` implies non-interactive.** You can't prompt a pipe.
   The `Emitter` forces the `prompts::is_interactive() == false` path; missing
   required input bails as a JSON error (see rule 3). Extends the
   `TVC_NON_INTERACTIVE` fence already in `non_interactive.rs`.
3. **JSON error envelope.** `Cli::run()` wraps dispatch; on `Err` in JSON mode,
   emit `{"error": "..."}` to stdout and exit non-zero — never a bare anyhow
   string.

### Security rules (no sensitive data in output)

Today no `println!`/`eprintln!` prints any private key or seed — only public
keys, ciphertext (encrypted shares), signatures, and paths. JSON mode must not
regress this. The landmine: `StoredApiKey` and `StoredQosOperatorKey` both
`#[derive(Serialize)]` and carry a `private_key` field (for on-disk
persistence), so naively serializing them into a report would leak the secret.

1. **Reports are hand-authored allowlists, never passthroughs.** `Report` is
   implemented only on purpose-built structs with explicitly enumerated fields.
   Credential/config/domain types (`StoredApiKey`, `StoredQosOperatorKey`,
   `DeployConfig`, …) are never passed to `emit` or embedded in a report. The
   `login` report carries only `public_key`s + file paths — the same fields the
   text output already shows.
2. **Leak-guard test** (in the step-6 harness): tests control the fixtures
   (`fixtures/seed.hex`, in-test generated keys), so the harness asserts emitted
   JSON stdout never contains the known secret value. A standing regression
   fence.

## Scope — offline-first

Most commands hit the live Turnkey API via `build_client()`, and there is no
mock backend in the test suite yet. So full JSON + round-trip tests land for the
**offline** commands now; the **API-dependent** commands get the flag plumbing
and the JSON error envelope (assertable, since they fail at auth in tests), with
their success-path JSON deferred behind a mock backend.

**Offline (full JSON + round-trip tests):**
- `keys generate-quorum-key` — pure crypto + file
- `keys re-encrypt-share` — local; already emits JSON, fold into `emit`
- `deploy approve` — local approval path (`--skip-post --dangerous-skip-interactive`)
- `deploy init`, `app init`, `keys init-quorum-key` — scaffolds (`{ path }` reports)

**API-dependent (stub + flag for later, error envelope only):**
`deploy create/status/get-status/provisioning-details/post-share/delete/restore`,
`app status/list/create/set-live-deploy/delete`, `login`. Thread the `Emitter`,
get the JSON error envelope for free; mark success-path report structs with
`// TODO(json-mock-backend)`. `app list` is `todo!()` today — out of scope for
output beyond the stub.

## Execution order (each step ships its own tests)

| Step | Scope | Tests written in the same step |
|---|---|---|
| **1. Scaffold** | `output.rs` (`OutputFormat`, `Emitter`, `Report`, error envelope), `--format` global, thread `&Emitter` through all `run()`s. Compiles; every failure path is JSON. | unit: `emit` text-vs-json + error envelope; integration: one command's `--format json` failure parses as `{"error":...}` |
| **2. Keys (offline)** | `generate-quorum-key` report; fold `re-encrypt-share` into `emit` | offline round-trip: parse quorum key / threshold / shares; parse re-encrypted-share fields |
| **3. Approve (offline)** | `deploy approve` local approval → report (posted-IDs optional, deferred) | offline round-trip with fixture manifest + `--skip-post` |
| **4. Scaffolds (offline)** | `deploy init`, `app init`, `keys init-quorum-key` → `{ path }` reports | round-trip: parse `{path}` → assert file exists & parses; `--format json` forces non-interactive |
| **5. API stubs + flags** | wire `Emitter` through API commands (error envelope only), add `TODO(json-mock-backend)` + checklist below | error-envelope assertion per command (fail at auth) |
| **6. Comprehensive e2e harness** | `tests/common/` `run_json(args) -> Value` helper; consolidate offline round-trips; matrix asserting no command leaks non-JSON to stdout in json mode + the secret leak-guard | the harness is the deliverable |

## API-command success-JSON checklist (awaiting mock backend)

- [ ] `deploy create`
- [ ] `deploy status`
- [ ] `deploy get-status`
- [ ] `deploy provisioning-details`
- [ ] `deploy post-share`
- [ ] `deploy delete`
- [ ] `deploy restore`
- [ ] `app status`
- [ ] `app list` (also needs the listing itself implemented)
- [ ] `app create`
- [ ] `app set-live-deploy`
- [ ] `app delete`
- [ ] `login`
