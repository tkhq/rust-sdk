# AGENTS.md

Default guidance for coding-agent runs in this repository.

## Coding style

- Prefer raw string literals (`r#"..."#`) over escaped quotation marks (`\"`) or
  escaped newlines (`\n`) for any nontrivial string or multi-line output (e.g.
  `human_message` bodies, help text, JSON fixtures, test goldens). Lay the text
  out on real lines so it reads as it renders. If a line ends in significant
  trailing whitespace, still use a raw literal but add a comment noting the
  trailing whitespace so editors/formatters don't silently strip it.
- Prefer moving owned values over cloning them. If you already own a value and
  this is its last use (e.g. building the return value at the end of a
  function), move it out — use `.into_iter()` or partial field moves instead of
  `.clone()`.
- Don't hide clones inside functions. A function or `From`/`TryFrom` impl should
  take an owned value (`T`, not `&T`) rather than clone internally; prefer
  `From<T>` over `From<&T>`. When a clone is genuinely needed, make it explicit
  at the call site — e.g. `value.clone().into()` or
  `items.iter().cloned().map(Into::into)`.
- Prefer short (imported) names over fully-qualified paths. Add a `use` and
  write `impl Display for Foo { fn fmt(&self, f: &mut Formatter<'_>) … }` rather
  than `impl std::fmt::Display for Foo { … std::fmt::Formatter … }`. Only keep a
  longer/module-qualified form when it disambiguates from another in-scope name —
  e.g. `fmt::Result` stays qualified (via `use std::fmt::{self, Display, Formatter}`)
  so it doesn't collide with `anyhow::Result`, and `std::fmt::Write` may need
  `as _` where `std::io::Write` is also in scope. Merge imports from the same
  module where practical.
- In doc comments and module docs, describe responsibilities, contracts, and
  relationships without naming specific source files or inventorying current
  consumers. File paths and call-site lists go stale when code moves. When a
  relationship matters, prefer Rustdoc intra-doc links to stable items (for
  example, [`ErrorCode`] or [`crate::errors::classify`]) and describe other
  participants by their role, such as "the CLI output layer" or "callers."
  Reference an exact path only when the path itself is part of an operational
  or compatibility contract (for example, a user-facing config location or
  migration input), or when no stable symbol exists.
- When converting from an external/generated type (e.g. the API's `TvcApp`,
  `TvcDeployment`, `AppStatus`) into one of our own structs, destructure it
  exhaustively — `let Foo { a, b, c: _ } = value;` with no trailing `..` —
  rather than reading fields with `value.a`. Bind the fields you use and
  `_`-bind the ones you don't. This way, when the upstream type gains a field, the destructure
  fails to compile and forces a deliberate decision about whether the new field
  belongs in our output — instead of it being silently dropped. Skip this only
  where it adds noise for no value, e.g. reading one or two fields off a large
  API response result.
- In `tracing` calls, use field shorthand when the variable name matches the
  field name — `%value` for `Display`, `?value` for `Debug` — rather than
  `value = %value`. Prefer `#[instrument]` on functions over manually built
  spans: it captures arguments, propagates async context, and keeps the
  function body clean. Use `#[instrument(skip(arg))]` for noisy or sensitive
  arguments and `#[instrument(level = "debug", ret, err)]` when return or
  error logging helps.

## CLI boundaries

- Use Clap field types, defaults, value parsers, argument groups, and conflicts
  to enforce CLI invariants during parsing instead of recreating the same
  validation in command execution.
- Keep fields on Clap `Args` structs private unless another construction path is
  intentional. Downstream functions should accept validated domain inputs, not
  a publicly constructible bag of CLI options.
- Reject incompatibilities that can be determined from parsed CLI arguments
  immediately after parsing, before loading configuration or credentials,
  authenticating, signing, or making network requests. If validation requires
  configuration, load only the configuration needed for that validation first.

## Types and data flow

- Prefer types that cannot represent invalid state combinations. Use enums,
  domain wrappers, and private constructors to encode mutually exclusive
  choices and required relationships.
- Keep each identity or decision in one authoritative place. Do not duplicate
  values across related structs when they could diverge.
- Parse identifiers whose domain contract guarantees a specific format into the
  corresponding domain type, such as `Uuid`, at CLI, config, and API boundaries.
  Do not assume that every field named `*_id` is a UUID; preserve opaque or
  forward-compatible identifiers as strings or dedicated domain wrappers.
  Compare typed values internally and convert them to strings only when an
  external wire type requires it.
- Parse, don't validate: run the fallible check once, at the boundary, and
  return a narrow domain type that carries the proof. Downstream code takes
  that type and transforms it infallibly instead of repeating the check deeper
  in the call stack. Watch for validation disguised as parsing: before writing
  a function named `is_*`/`check_*`/`verify_*`/`validate_*`, or one that
  inspects data and returns `bool` or `Result<(), E>`, ask where the proof
  goes — if callers continue with the same type they passed in, that is
  validation; return the parsed type instead. `Result<(), E>` is the usual
  disguise: it reads as parsing because it is fallible, but the unit return
  discards the evidence.
- Prefer infallible constructors. A constructor assembles an already-valid
  value; it does not acquire what it needs. Do fallible acquisition — loading
  config, resolving addresses, opening handles, fetching credentials — before
  construction and pass the finished dependencies in, concentrating failure
  handling at the wiring layer. Reserve a fallible constructor for a type
  that exists to hold a live, long-lived resource, where constructing and
  connecting are genuinely the same operation.
- Keep call stacks flat. Default to inlining one-use helpers — a let-bound
  block (`let approved = { … };`) names the result without adding a
  signature. A helper must earn its boundary: actual repeated callers
  (extract on the third occurrence, not in anticipation of reuse), a
  genuinely generic unit, or an intentional `pub` surface. A `.clone()`
  added only to satisfy an extracted signature means the boundary is
  wrong — dissolve the helper rather than pay the clone.
- Match enums exhaustively when variants require distinct behavior. Use a
  wildcard only when all current and future non-target variants are
  intentionally handled alike.

## I/O, errors, and compatibility

- Perform config loading, authentication, and network work only on paths that
  require them. Explicit or offline inputs must not depend on unrelated config
  being present or well-formed.
- Treat serialized TOML and JSON shapes as compatibility boundaries. Keep
  runtime types distinct from persisted schemas when a migration needs
  different fields, and make migration timing and write-back behavior explicit.
- Keep missing data distinct from malformed data. Default only for intentional
  absence; surface malformed persisted or API values with the field, path, and
  operation needed to diagnose them.
- Prefer typed errors when callers need to make recovery decisions. Add
  user-facing remediation at the command layer instead of embedding a specific
  CLI command in reusable helpers.
- Preserve typed errors through `anyhow` chains so machine classification can
  downcast them. Add operation and identifier context with `.context()` or
  `.with_context()`; do not stringify an error with `anyhow!("{error}")`,
  `bail!("{error}")`, or a formatting-only `map_err`, because that discards its
  type and source chain.
- Use `MissingResource::new` only when a lookup request succeeded but its
  decoded response omitted the expected resource, such as an optional payload
  being `None`. This means callers should verify or re-resolve the identifier
  or prerequisite state; it does not imply that blindly retrying the same
  lookup will help. Pass a stable resource noun and the most actionable
  identifier, for example `MissingResource::new("deployment", deployment_id)`.
  Do not use `MissingResource` for unsuccessful HTTP responses; propagate the
  typed `TurnkeyClientError` so its status and response body remain available.
- Do not assign `ErrorCode` values in command code. Preserve or introduce a
  typed error and let `crate::errors::classify` own the mapping. Classify new
  upstream error variants explicitly rather than adding a wildcard fallback.
- Do not render runtime errors at call sites. Pass the `anyhow::Error` to the
  output boundary so human and JSON modes use the same chain rendering,
  truncation, classification, and HTTP-status behavior.
- Prefer `thiserror` derives for error types whose `Display` is a straightforward
  field-formatting template. Reserve manual `Display` and `Error`
  implementations for behavior the derive cannot express clearly.
- Implement `Display` for domain values used in user-facing errors rather than
  hard-coding their variants at call sites.

## Tests and verification

- Prefer complete structural equality when the complete value or serialized
  shape is the contract. Use focused field or predicate assertions when a test
  deliberately covers only one property and unrelated fields are outside its
  scope; avoid substring assertions when an exact structured representation is
  available. Use test-only `Debug` and `PartialEq` derives when those traits
  should not expand the release API.
- Avoid `unreachable!()` in tests; use exact equality, pattern assertions, or a
  descriptive failure.
- Test CLI parsing for the defaults, conflicts, and typed values that form part
  of our interface. Avoid duplicating Clap's validation as deeper command
  validation tests.
- Test migrations and serialized compatibility by parsing the complete output
  into the target schema and comparing it with a complete expected value.
