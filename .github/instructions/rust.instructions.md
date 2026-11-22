---
applyTo: "**/*.rs"
---

# Rust code

- Every module (`//!`), type, field, variant, function, constant and macro has a doc
  comment, private and `pub(crate)` items included. Docs state intent, invariants, units and
  error/panic conditions (`# Errors`, `# Panics`). They do not restate the name.
- No `mod.rs`: modules with children are `foo.rs` plus `foo/`. Integration tests live under
  `tests/it/`; only `tests/translated.rs` stays separate, because the package excludes it.
- No separator comments (`// -----`, `// =====`). If a file needs sections, split it into
  submodules.
- Leave a blank line after the closing `}` of a block statement when another statement
  follows, and between logical phases of a function. No lumped code.
- One level of abstraction per function. Long loops (axiom fixpoints, projection building,
  parsers) are short drivers that call documented step functions. If a block needs a "what"
  comment, extract it into a function named after that comment.
- `clippy::pedantic` clean. Suppression only with a scoped
  `#[expect(lint, reason = "...")]`. Never `#[allow]`, and never crate-level suppression.
- No `unsafe`: the crate is `#![forbid(unsafe_code)]`.
- The crate is `#![no_std]` and needs only `alloc`. The `output.sas` reader sits behind
  `fdr`, the generator behind `random`, the serde derives behind `serde`. Flag any `std`
  use and any new `[dependencies]` entry.
- `fdr` writes back byte for byte what it reads, and parse errors carry 1-based line
  numbers. Keep both, and keep the round-trip tests and the `parse` fuzz target passing.
- `random::random_task` is reproducible per seed: a change that alters the task a seed
  produces breaks reproducibility for dependents' property tests. New draws go after the
  existing ones, as the derived variables do.
- Projections must stay admissible: they never overestimate `perfect_heuristic` and never
  prune a solvable state (`tests/it/properties.rs`, the `projection` fuzz target).
- Mermaid diagrams in doc comments must match the code they describe. Update them when the
  structure changes.
