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