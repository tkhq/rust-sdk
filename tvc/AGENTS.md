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
  `as _` where `std::io::Write` is also in scope.
- When converting from an external/generated type (e.g. the API's `TvcApp`,
  `TvcDeployment`, `AppStatus`) into one of our own structs, destructure it
  exhaustively — `let Foo { a, b, c: _ } = value;` with no trailing `..` —
  rather than reading fields with `value.a`. Bind the fields you use and
  `_`-bind the ones you don't. This way, when the upstream type gains a field, the destructure
  fails to compile and forces a deliberate decision about whether the new field
  belongs in our output — instead of it being silently dropped. Skip this only
  where it adds noise for no value, e.g. reading one or two fields off a large
  API response result.

## CLI boundaries

- Use Clap field types, defaults, value parsers, argument groups, and conflicts
  to enforce CLI invariants during parsing instead of recreating the same
  validation in command execution.
- Keep fields on Clap `Args` structs private unless another construction path is
  intentional. Downstream functions should accept validated domain inputs, not
  a publicly constructible bag of CLI options.
- Reject incompatible modes before loading unrelated config, authenticating,
  signing, or making network requests.

## Types and data flow

- Prefer types that cannot represent invalid state combinations. Use enums,
  domain wrappers, and private constructors to encode mutually exclusive
  choices and required relationships.
- Keep each identity or decision in one authoritative place. Do not duplicate
  values across related structs when they could diverge.
- Parse identifiers into domain types such as `Uuid` at CLI, config, and API
  boundaries. Compare typed values internally and convert them to strings only
  when an external wire type requires it.
- Validate or resolve inputs once, then pass a narrow validated type into
  creation or transformation code. Prefer an infallible transformation after
  validation over a helper that repeats validation deeper in the call stack.
- Keep call stacks flat. Inline a one-use helper when it only adds nesting,
  caller-specific policy, or hidden cloning; retain helpers that provide a
  coherent reusable discovery or transformation operation.
- Borrow before cloning when the source outlives the operation. Make deliberate
  ownership-boundary clones visible at the call site.
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
- Implement `Display` for domain values used in user-facing errors rather than
  hard-coding their variants at call sites.

## Tests and verification

- Prefer complete structural equality over substring or partial-field
  assertions. Use test-only `Debug` and `PartialEq` derives when those traits
  should not expand the release API.
- Avoid `unreachable!()` in tests; use exact equality, pattern assertions, or a
  descriptive failure.
- Test CLI parsing for the defaults, conflicts, and typed values that form part
  of our interface. Avoid duplicating Clap's validation as deeper command
  validation tests.
- Test migrations and serialized compatibility by parsing the complete output
  into the target schema and comparing it with a complete expected value.
