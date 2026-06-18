# Agent guidance for the Turnkey Rust SDK

This file gives AI coding agents (and humans) conventions to follow when working
in this repository. It complements, but does not replace, the automated checks
in `make lint` / `make test`.

## Imports: prefer short `use` imports over inline absolute paths

When you reference an item (function, type, trait, macro) from another module,
bring it into scope with a `use` import and call it by its short name. Do **not**
write out a fully-qualified absolute path inline at the call site.

This keeps call sites readable, makes dependencies explicit at the top of the
file, and matches the convention requested in code review (see PR #158).

**Avoid** — inline fully-qualified absolute path:

```rust
let app_status = crate::commands::app_status::sanitize_app_status(response)?;
println!("{}", crate::commands::display::format_egress_enabled(app.enable_egress));
```

**Prefer** — short `use` import + bare call:

```rust
use crate::commands::app_status::sanitize_app_status;
use crate::commands::display::format_egress_enabled;

// ...

let app_status = sanitize_app_status(response)?;
println!("{}", format_egress_enabled(app.enable_egress));
```

Notes and exceptions:

- This applies to `crate::`, `self::`, `super::`, and external-crate paths alike.
- Short, idiomatic fully-qualified references are fine and are **not** flagged,
  e.g. `std::collections::HashMap` or two-segment paths. The goal is to avoid
  long inline paths to internal helpers, not to ban every `::`.
- It is fine to keep a path inline to disambiguate a genuine name collision, or
  for a one-off reference where a `use` would be more confusing than helpful.
  Use judgment; readability is the goal.
- In the `tvc` crate this convention is mechanically enforced by the
  `clippy::absolute_paths` lint (configured in `clippy.toml`, enabled in
  `tvc/Cargo.toml`). Elsewhere it is a guideline. Generated code under
  `client/src/generated/` is exempt and should never be hand-edited.
