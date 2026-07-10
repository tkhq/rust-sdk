# Cargo's logging & `--message-format` — what to copy for a clap CLI

All file references below are permalinks to cargo at commit [`8b241a6`](https://github.com/rust-lang/cargo/tree/8b241a6b6e197f61dde0f1478e405d7f994c6092).

## Context

You want to understand cargo's user-facing **logging/output** system before designing your own clap-based CLI:

- How does cargo cleanly switch between human / JSON / ANSI output with `--message-format`?
- What makes this design extensible?
- What does a callsite look like?
- Where do bytes actually hit the wire?
- Is the logger dependency-injected or global?

Cargo *also* has `tracing` for developer diagnostics, which is a separate concern. That's covered in [a footnote at the bottom](#a-note-on-tracing--a-separate-channel-dont-conflate-them) — the rest of this doc is about logging.

---

## The big picture

```
                 ┌────────────────────────┐
   producers     │ shell.status(...)      │
   (open;        │ shell.warn(...)        │
    callable     │ shell.error(...)       │
    from         │ machine_message::*     │
    anywhere)    └───────────┬────────────┘
                             │
                             ▼
                ┌────────────────────────┐
   renderer     │   Shell                │   ← knows verbosity, color choice,
   (closed;     │   (one chokepoint)     │     TTY, hyperlinks, MessageFormat
    one         └───────────┬────────────┘
    instance)               │
                            ▼
                ┌────────────────────────┐
   sink         │ anstream::AutoStream   │   ← strips ANSI per --color, writes
   (handles     │   over stdout/stderr   │     atomically, or buffers for tests
    bytes)      └────────────────────────┘
```

Three layers. Producers proliferate freely. The renderer centralizes every formatting decision. The sink owns the actual `Write`. That separation is why `--message-format` and `--color` can both work cleanly without producers caring about either.

---

## `Shell`: the user-output type

Defined at [`crates/cargo-util-terminal/src/shell.rs#L17-L26`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/crates/cargo-util-terminal/src/shell.rs#L17-L26):

```rust
pub struct Shell {
    output: ShellOut,        // AutoStream<stdout/stderr>, or a Write for tests
    verbosity: Verbosity,    // Quiet | Normal | Verbose
    needs_clear: bool,       // for line-clearing under progress bars
}
```

The public API is small and intention-revealing:

- [`status(label, msg)`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/crates/cargo-util-terminal/src/shell.rs#L163-L169) — right-justified, bold-green label (`   Compiling foo v0.1.0`).
- [`warn(msg)`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/crates/cargo-util-terminal/src/shell.rs#L224-L226) — bold-yellow `warning: ...`.
- [`error(msg)`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/crates/cargo-util-terminal/src/shell.rs#L215-L221) — bold-red `error: ...`.
- [`note(msg)`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/crates/cargo-util-terminal/src/shell.rs#L229-L234) — bold-cyan `note: ...`.

Why a type at all instead of just `println!`/`eprintln!`?

- **Verbosity gating in one place.** `Quiet` is checked once inside `Shell::print`; producers don't sprinkle `if !quiet { ... }` everywhere.
- **Color handling in one place.** `Shell` owns the `ColorChoice` and an `anstream::AutoStream` that strips ANSI when configured to. Producers always write style codes.
- **Test capture in one place.** [`Shell::from_write(Box<dyn Write + Send + Sync>)`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/crates/cargo-util-terminal/src/shell.rs#L69-L75) substitutes a `Vec<u8>` for tests, with color forced off.
- **Atomic line writes.** Status/warn/error each format into a `Vec<u8>` buffer before a single `write_all`, so threads can't tear a line.

---

## `--message-format`: the CLI flag

Switches the **format** of user output: human-readable, machine-readable JSON, or human-with-modifiers (short, ansi-rendered). Orthogonal to `--color`, which decides whether ANSI codes survive to the terminal.

### a) Clap flag declaration

[`src/cargo/util/command_prelude.rs#L395-L408`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/src/cargo/util/command_prelude.rs#L395-L408):

```rust
multi_opt("message-format", "FMT", "Error format")
    .value_parser([
        "human",
        "short",
        "json",
        "json-diagnostic-short",
        "json-diagnostic-rendered-ansi",
        "json-render-diagnostics",
    ])
    .value_delimiter(',')
    .ignore_case(true)
```

`value_parser` rejects bad input before any cargo code runs. `value_delimiter(',')` lets users write `--message-format json,json-diagnostic-short` and have the strings merged.

### b) The internal enum

[`src/cargo/core/compiler/build_config.rs#L150-L165`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/src/cargo/core/compiler/build_config.rs#L150-L165):

```rust
pub enum MessageFormat {
    Human,
    Json {
        render_diagnostics: bool,
        short: bool,
        ansi: bool,
    },
    Short,
}
```

Cargo **composes** the JSON variants instead of having `JsonShort`, `JsonAnsi`, `JsonRenderDiagnostics`, etc. The CLI strings are sugar over `(short, ansi, render_diagnostics)` booleans inside one `Json` variant. That's what keeps the modifier combinations cheap.

### c) String → enum conversion

[`src/cargo/util/command_prelude.rs#L746-L816`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/src/cargo/util/command_prelude.rs#L746-L816):

```rust
let default_json = MessageFormat::Json { short: false, ansi: false, render_diagnostics: false };
for fmt in self._values_of("message-format") {
    for fmt in fmt.split(',') {
        match fmt.to_ascii_lowercase().as_str() {
            "json"  => message_format = Some(default_json),
            "human" => message_format = Some(MessageFormat::Human),
            "short" => message_format = Some(MessageFormat::Short),
            "json-render-diagnostics" => {
                if message_format.is_none() { message_format = Some(default_json); }
                match &mut message_format {
                    Some(MessageFormat::Json { render_diagnostics, .. }) => *render_diagnostics = true,
                    _ => bail!("cannot specify two kinds of `message-format` arguments"),
                }
            }
            // ...same shape for `json-diagnostic-short` / `json-diagnostic-rendered-ansi`
        }
    }
}
build_config.message_format = message_format.unwrap_or(MessageFormat::Human);
```

Modifier formats mutate flags inside the existing `Json` variant. Conflicting base modes bail.

### d) `--color` is orthogonal

Colors are NOT part of `MessageFormat`. They live on `Shell` and are applied by `anstream::AutoStream`. The one place they cross paths is `MessageFormat::Json { ansi: true }`, which decides whether rustc's `rendered` field should *contain* ANSI codes — but `Shell` separately decides whether to *emit* ANSI to stderr.

### e) The dispatch site

[`src/cargo/core/compiler/mod.rs#L2240`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/src/cargo/core/compiler/mod.rs#L2240) does the actual `match options.format { ... }`. This is the **one place** that knows how to translate a `MessageFormat` into actual bytes. Add a variant → the compiler tells you to add a match arm here.

---

## `Message` trait: the JSON schema

[`src/cargo/util/machine_message.rs#L10-L26`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/src/cargo/util/machine_message.rs#L10-L26):

```rust
pub trait Message: ser::Serialize {
    fn reason(&self) -> &str;
    fn to_json_string(&self) -> String {
        #[derive(Serialize)]
        struct WithReason<'a, S: Serialize> {
            reason: &'a str,
            #[serde(flatten)]
            msg: &'a S,
        }
        serde_json::to_string(&WithReason { reason: self.reason(), msg: &self }).unwrap()
    }
}
```

Each message type ([`FromCompiler`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/src/cargo/util/machine_message.rs#L28-L40), [`Artifact`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/src/cargo/util/machine_message.rs#L42-L58), [`BuildScript`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/src/cargo/util/machine_message.rs#L80-L94), [`BuildFinished`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/src/cargo/util/machine_message.rs#L96-L105)) implements `Message` with a stable `reason` string. Consumers of `cargo --message-format=json` rely on `reason` to discriminate messages line-by-line. It's effectively cargo's externally stable schema.

---

## What makes this extensible

The trick is distinguishing **producers** (open-world) from **renderers/sinks** (centralized).

| Axis | Open or closed | How you extend it |
|------|----------------|-------------------|
| **JSON message types** | Open | Any module can add a new `struct` + `impl Message` and emit it. Nothing else needs to change — the `reason` discriminator keeps the stream parseable by consumers. |
| **`Shell` methods** (`status`, `warn`, …) | Open | Add a method on `Shell`; nothing else changes. All styling/verbosity logic stays centralized. |
| **`MessageFormat` variants** | Closed enum | Adding a format means touching ~4 sites (flag `value_parser`, enum, parser match, dispatch). The closed enum makes the compiler tell you every place that has to handle the new case. |
| **Color rendering** | Closed (single owner) | One library — `anstream::AutoStream` — wraps the writer. Everything that goes through `Shell` gets correct color handling for free. |

**Let producers proliferate, but funnel through one renderer.** That's why "easily decide to output as json or with ansi colours" works: the decision lives in exactly one place, while every feature in the codebase can contribute new things to render.

---

## What a callsite looks like

**User status** ([`src/cargo/ops/cargo_install.rs#L336`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/src/cargo/ops/cargo_install.rs#L336)):

```rust
self.gctx.shell().status("Installing", &self.pkg)?;
```

Renders as the familiar right-justified, bold-green `   Installing foo v0.1.0` on stderr.

**User warning** ([`src/cargo/ops/cargo_install.rs#L599`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/src/cargo/ops/cargo_install.rs#L599)):

```rust
self.gctx.shell().warn("aborting install due to dry run")?;
```

Renders as bold-yellow `warning: aborting install due to dry run`.

**User error** (from anywhere a `CargoResult` bubbles up; rendered via `Shell::error`):

```rust
gctx.shell().error("manifest path `Cargo.toml` does not exist")?;
```

Bold-red `error: manifest path \`Cargo.toml\` does not exist`.

**Machine-readable message** ([`src/cargo/core/compiler/job_queue/mod.rs#L881-L889`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/src/cargo/core/compiler/job_queue/mod.rs#L881-L889)):

```rust
if build_config.emit_json() {
    let mut shell = build_runner.bcx.gctx.shell();
    let msg = machine_message::BuildFinished { success: errors.count == 0 }.to_json_string();
    writeln!(shell.out(), "{}", msg)?;
}
```

Produces a single line of JSON like `{"reason":"build-finished","success":true}` on stdout. Even here, the write still goes through `Shell` — using `shell.out()` grabs the underlying writer rather than the formatted-status helpers, but `--color=never` and test capture still work.

The pattern: **always go through `Shell`, even when bypassing its formatting helpers**, so the single chokepoint stays the single chokepoint.

---

## Where bytes actually hit the wire

[`ShellOut::message_stderr`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/crates/cargo-util-terminal/src/shell.rs#L469-L488) is the single chokepoint for human-formatted output:

```rust
fn message_stderr(&mut self, status: &dyn fmt::Display, message: Option<&dyn fmt::Display>,
                  style: &Style, justified: bool) -> CargoResult<()> {
    let mut buffer = Vec::new();
    if justified {
        write!(&mut buffer, "{style}{status:>12}{style:#}")?;
    } else {
        write!(&mut buffer, "{style}{status}{style:#}:")?;
    }
    match message {
        Some(message) => writeln!(buffer, " {message}")?,
        None          => write!(buffer, " ")?,
    }
    self.stderr().write_all(&buffer)?;
    Ok(())
}
```

Three things to notice:

1. **Style codes use `anstyle::Style`'s display impl.** `{style}` writes the ANSI prefix, `{style:#}` writes the reset. Cargo never hand-rolls escape codes.
2. **`self.stderr()` is the `AutoStream<Stderr>`** — `anstream` inspects the writer and the configured `ColorChoice` and *strips* ANSI bytes on the way out if color is disabled. The producer always writes color codes; the sink decides whether they survive.
3. **The full line is buffered then written in one `write_all`.** Interleaved threads can't tear a single status line apart.

For JSON, there is no `message_stderr`-equivalent — producers just `writeln!(shell.out(), "{}", msg.to_json_string())` to stdout. The serialization happened inside `Message::to_json_string()`. But the path still goes through `Shell`, so the `AutoStream` wrapper still applies.

---

## DI or global?

**`Shell` is dependency-injected via `GlobalContext`.** Not a global; not a thread-local. An owned object threaded through every command.

```rust
// somewhere deep in cargo's call stack
pub fn install(gctx: &GlobalContext, …) -> CargoResult<()> {
    gctx.shell().status("Installing", &pkg)?;
    …
}
```

The accessor at [`src/cargo/util/context/mod.rs#L483`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/src/cargo/util/context/mod.rs#L483) returns a `MutexGuard<'_, Shell>`. The mutex lets cargo hold `&GlobalContext` (shared) most of the time and only briefly take an exclusive lock to print. The signature you almost always see is `fn foo(gctx: &GlobalContext, …)`, never `fn foo(shell: &mut Shell, …)` — the convention is to pass the context, and the function reaches in for what it needs.

Why DI and not a global?

- **Test capture.** [`Shell::from_write(Vec::new())`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/crates/cargo-util-terminal/src/shell.rs#L69-L75) substitutes an in-memory buffer for a real `Shell`, with color forced off. A test asserts on bytes by reading the buffer back. A global would need a thread-local or lock to do this, and tests can't run in parallel without contention.
- **Per-invocation state.** Verbosity, color choice, message format, and progress-bar-line-clearing are all *per-CLI-invocation* settings. Globals would force them into atomics or locks, or — worse — into module-scoped statics that can't be reset.
- **No hidden dependencies.** If a function uses `Shell`, its signature says so (via `gctx`). You can grep for callers and reason about who writes to the terminal.
- **Scoping.** Cargo sometimes wants a sub-context (e.g., for subcommand isolation). Owning a `Shell` per-context means you can branch, swap, or silence sub-trees without touching anything global.

The cost is signature noise (`gctx` threaded through every function), but cargo treats `GlobalContext` as a general "ambient services" bag — it also carries config, package cache, http handle, locking — so the cost is amortized.

**For your CLI:** thread a `Shell` (or a small context that holds it) through your call graph. Don't make it a global. Globals look easier on day one and cost you all of the above forever.

---

## How to do this in your clap CLI

Mirror cargo's split into three layers.

### Layer 1 — `Shell` for user output

```rust
// shell.rs
pub struct Shell {
    out: ShellOut,         // AutoStream<Stdout> + AutoStream<Stderr>, or a Write for tests
    verbosity: Verbosity,  // Quiet | Normal | Verbose
    format: MessageFormat, // see below
}

impl Shell {
    pub fn status(&mut self, label: &str, msg: impl Display) -> Result<()> { ... }
    pub fn warn(&mut self, msg: impl Display) -> Result<()> { ... }
    pub fn error(&mut self, msg: impl Display) -> Result<()> { ... }
    pub fn emit<M: Message>(&mut self, msg: &M) -> Result<()> {
        match self.format {
            MessageFormat::Human => write_human(&mut self.out, msg),
            MessageFormat::Json  => writeln!(self.out.stdout(), "{}", msg.to_json_string()),
            MessageFormat::Plain => write_plain(&mut self.out, msg),
        }
    }
}
```

Use `anstream::AutoStream` for color handling — it understands `--color={auto,always,never}`, `NO_COLOR`, and TTY detection so you don't have to.

### Layer 2 — `MessageFormat` enum + clap parsing

```rust
#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageFormat {
    Human,
    Json,
    Plain,  // ANSI-stripped human
}

#[derive(clap::Parser)]
pub struct GlobalArgs {
    #[arg(long, value_enum, global = true, default_value_t = MessageFormat::Human)]
    pub message_format: MessageFormat,

    #[arg(long, value_enum, global = true, default_value_t = ColorChoice::Auto)]
    pub color: ColorChoice,

    #[arg(short, long, global = true)]
    pub quiet: bool,

    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
}
```

`#[derive(ValueEnum)]` gives you cargo's `value_parser([...])` behavior for free — invalid strings are rejected by clap before your code runs. `global = true` makes the flags available on every subcommand without re-declaring.

(Note: cargo itself uses clap's *builder* API rather than `derive`, because its subcommands are pluggable at runtime — see `multi_opt(...).value_parser([...])` in [`command_prelude.rs#L395`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/src/cargo/util/command_prelude.rs#L395). Both APIs reach the same place.)

If you later need cargo's "modifier" pattern (`json-diagnostic-short`), keep `MessageFormat::Json { short: bool, ansi: bool }` as a struct variant and write a custom `FromStr`/`ValueParser`. Don't reach for this until you need it.

### Layer 3 — `Message` trait for the JSON schema

```rust
pub trait Message: serde::Serialize {
    fn reason(&self) -> &'static str;
    fn to_json_string(&self) -> String {
        #[derive(Serialize)]
        struct WithReason<'a, S: Serialize> {
            reason: &'a str,
            #[serde(flatten)]
            msg: &'a S,
        }
        serde_json::to_string(&WithReason { reason: self.reason(), msg: self }).unwrap()
    }
}
```

Same shape as cargo. The `reason` discriminator is what makes the JSON stream parseable line-by-line by downstream tools — they `match` on it. Every new message type adds one `impl Message`.

### Why this is clean and extensible

- **One place for color/TTY logic**: `Shell`, backed by `anstream`. New subcommands inherit it for free.
- **One place to add a format**: add a variant to `MessageFormat`, add a branch in `Shell::emit`. No coordination across the codebase.
- **One place to add a JSON message type**: derive `Serialize`, implement `Message`. The `reason` discriminator keeps the stream parseable.
- **Test-friendly**: `Shell::from_write(Vec::new())` captures all output for assertions.
- **`--color` stays orthogonal to `--message-format`**, exactly like cargo. Users can mix and match.

### Ripgrep contrast (since you peeked)

Ripgrep hand-rolls because it ships its *own* printer abstraction (`grep-printer`) optimized for streaming matched lines at very high throughput across many file encodings. It also doesn't have cargo's "many heterogeneous message types in one stream" problem — it has one message type (a match) emitted billions of times. For a normal CLI with a handful of message kinds and human-scale output volume, cargo's pattern is the right reference, not ripgrep's.

---

## Verification

To pressure-test the design against cargo's behavior directly:

```sh
# Switch formats
cargo build --message-format=human          # default
cargo build --message-format=json           # raw rustc JSON pass-through
cargo build --message-format=json-render-diagnostics   # cargo renders diagnostics, JSON for everything else
cargo build --message-format=json-diagnostic-rendered-ansi   # JSON, with `rendered` containing ANSI

# --color is orthogonal
cargo build --message-format=json --color=always   # JSON output, ANSI in `rendered` fields
cargo build --color=never                          # human output, no ANSI on Shell
```

---

## A note on `tracing` — a separate channel, don't conflate them

Cargo *also* uses [`tracing`](https://docs.rs/tracing), but **it is not the logging system described above**. The two are independent channels that happen to both write to stderr when stderr is a TTY. This is the single most common point of confusion when reading cargo's source.

| | `Shell` (this doc) | `tracing` |
|---|---|---|
| **Audience** | End users | Developers debugging cargo itself |
| **Activation** | Always on | Off unless `CARGO_LOG=...` is set |
| **Pattern** | Dependency-injected via `GlobalContext` | Global subscriber registered at startup |
| **Library** | `cargo-util-terminal::Shell` + `anstream` + `anstyle` | `tracing` + `tracing-subscriber` |
| **Init site** | Created with `GlobalContext` | [`setup_logger` in `main.rs#L64-L82`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/src/bin/cargo/main.rs#L64-L82) |
| **Callsite** | `gctx.shell().status(...)` | `tracing::debug!(target: "network", …)` |

A representative tracing callsite — [`src/cargo/util/network/http.rs#L197`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/src/cargo/util/network/http.rs#L197):

```rust
tracing::debug!(target: "network", "{:#?}", curl::Version::get());
```

A regular user running `cargo build` never sees this line. Run `CARGO_LOG=cargo::util::network=debug cargo build` and it appears — interleaved with `Shell` output, on the same stderr, but coming from a completely different mechanism.

**The hard rule:** if a user is meant to read it, it goes through `Shell`. If it's only useful when someone is debugging the tool itself, it goes through `tracing`. The fact that both can land on the same TTY is incidental — they should never be conflated in design or in code review.

For your CLI: copy this split. Build the `Shell`-equivalent described above for user output. If you also want a developer-diagnostics channel, set up `tracing_subscriber` with an `EnvFilter::from_env("MYAPP_LOG")` — lifted near-verbatim from [`src/bin/cargo/main.rs#L64-L82`](https://github.com/rust-lang/cargo/blob/8b241a6b6e197f61dde0f1478e405d7f994c6092/src/bin/cargo/main.rs#L64-L82) — and never use `tracing` macros for anything a regular user is expected to see.