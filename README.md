# sas-plus

[![crates.io](https://img.shields.io/crates/v/sas-plus.svg)](https://crates.io/crates/sas-plus)
[![docs.rs](https://img.shields.io/docsrs/sas-plus)](https://docs.rs/sas-plus)
[![CI](https://github.com/sunsided/sas-plus/actions/workflows/ci.yml/badge.svg)](https://github.com/sunsided/sas-plus/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/sunsided/sas-plus/graph/badge.svg)](https://codecov.io/gh/sunsided/sas-plus)
[![MSRV](https://img.shields.io/crates/msrv/sas-plus)](https://crates.io/crates/sas-plus)
[![License](https://img.shields.io/crates/l/sas-plus)](https://github.com/sunsided/sas-plus#license)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

SAS+ / finite-domain planning tasks for Rust, **compatible with Fast Downward's translator**.

## What is SAS+?

**SAS** stands for **Simplified Action Structures** (Bäckström & Klein, 1991). The **+** is the extension by Bäckström & Nebel (1995, *Complexity Results for SAS+ Planning*): preconditions and goals may leave variables *undefined*, so they are partial states.

A SAS+ task has:
- finite-domain state variables instead of Boolean facts, e.g. `ball1 ∈ {at rooma, at roomb, carry left, carry right}`;
- operators with preconditions, effects and a cost;
- a complete initial state and a partial goal.

Because the variables capture mutual exclusion directly, many heuristics (pattern databases, merge-and-shrink, cost partitioning) are defined on SAS+ rather than on STRIPS. Fast Downward calls its version the *finite-domain representation* (FDR): SAS+ plus conditional effects and axioms.

## What it's for

`sas-plus` provides building blocks for planning heuristics and search in Rust, in the spirit of crates like `tower`: small pieces you can reason about one at a time. It is not a planner itself, but meant for people who write or test such algorithms.

`Task` and `LabeledTransitionSystem` are plain structs with public fields: a task is its variables, operators, initial state, goal and axioms, and a transition system is a list of labeled transitions with a goal flag per state. The one abstraction on top is the `Abstraction` trait, which maps a concrete state to a state of a transition system; anything that implements it, a `Projection` or a type of your own, can stand in wherever a heuristic needs abstract goal distances. The crate has no required dependencies and is `no_std`; it only needs `alloc`.

For testing, `Task::perfect_heuristic()` computes the exact goal distance h\* on small tasks, and `random::random_task` generates reproducible tasks from a seed, so a heuristic can be checked against h\* over as many seeds as you care to run. On top of that, the `fdr` module reads Fast Downward's translator output, which brings real benchmark domains in without writing a grounder.

There is no search algorithm and no PDDL parser. To ground PDDL in pure Rust, the [pddl](https://github.com/sunsided/pddl-rs) crate's `sas-plus` feature grounds a domain and problem into an `fdr::SasFile`.

## Who this is for

- Authors of planning heuristics and search algorithms who want one plain, validated task model to build on, with Fast Downward's translator output as a source of real benchmark domains.
- Researchers who need ground truth while developing a heuristic: `perfect_heuristic` and the seeded random tasks turn admissibility checks into ordinary property tests.
- Teams that plan on embedded or robotic hardware: the task model, projections and transition systems are `no_std` with `alloc` and have no required dependencies.
- Anyone who wants ready-made heuristics rather than the building blocks should look at [planning-heuristics](https://github.com/sunsided/planning-heuristics), [merge-and-shrink](https://github.com/sunsided/merge-and-shrink) and [cost-partition](https://github.com/sunsided/cost-partition); for search over a `Task`, see [heuristic-search](https://github.com/sunsided/heuristic-search), [symbolic-search](https://github.com/sunsided/symbolic-search) and [decoupled-search](https://github.com/sunsided/decoupled-search).

## Installation

```sh
cargo add sas-plus
```

Or in `Cargo.toml`:

```toml
[dependencies]
sas-plus = "0.2"
```

## Usage

Ground PDDL with Fast Downward's translator, then load the result in Rust:

```rust,no_run
use sas_plus::fdr::SasFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file: SasFile = std::fs::read_to_string("output.sas")?.parse()?;
    let task = file.to_task(); // Task with conditional effects and axioms
    println!("{} operators", task.operators.len());
    let plan = Vec::new(); // ... your search over `task` ...
    print!("{}", file.format_plan(&plan)); // sas_plan format, ready for VAL
    Ok(())
}
```

To try both steps with Docker and [Task](https://taskfile.dev), run `task ground:load`. It grounds a fixture with the `aibasel/downward` image and loads the result with [`examples/load.rs`](https://github.com/sunsided/sas-plus/blob/main/examples/load.rs). Pass `DOMAIN=... PROBLEM=...` to ground your own PDDL (paths inside the repository).

## What's in it
- **`fdr`**: reader and writer for `output.sas` (file version 3).
  - Faithful to the format: mutex groups, conditional effects, axioms, and fact names are all kept.
  - Writes back byte-for-byte what it reads.
  - Parse errors carry line numbers.
  - `to_task()` converts to a `Task`, keeping conditional effects and axioms.
  - Metric `0` means unit costs, exactly as in Fast Downward's search.
- **`Task`, `Operator`, `Effect`, `Axiom`, `Fact`, `State`**, with validation, applicability and plan checking. Effects may have conditions, and derived variables are computed by layered axioms with Fast Downward's semantics.
- **`LabeledTransitionSystem`** and the **`Abstraction` trait**, the common interface for abstraction heuristics such as projections, merge-and-shrink, and cost partitioning over them.
- **`Projection`**: the pattern-database abstraction. Derived variables are left out of patterns, and conditional effects whose conditions lie outside the pattern may or may not fire, so projections stay admissible.
- **`CausalGraph`**: arcs from preconditions and effect conditions to effects, between co-occurring effects, and from axiom conditions to derived variables.
- **Test oracles**: `Task::perfect_heuristic()` (exact h\*) and `random::random_task`.

## Features
- `fdr` (default): the `output.sas` reader and writer.
- `random` (default): the random task generator.
- `serde`: `Serialize` and `Deserialize` for the task model, transition systems, the causal graph and `SasFile`.

Use `default-features = false` if you only need the task model.

## Fixtures
- `tests/fixtures/translated/`: real Fast Downward translator output for nine problems, from the VHPOP and Potoroo corpora plus three written for this repo. It covers unit costs, action costs, conditional effects and derived predicates. Every file round-trips byte for byte, and the small tasks are checked against their known optima. The folder includes GPL-2.0 / Apache-2.0 test inputs and is excluded from the published crate.
- `tests/fixtures/`: small hand-encoded files, including a minimal axiom (derived-variable) example.

## Testing
- `task test`: unit, fixture and property tests. `tests/it/properties.rs` checks on random tasks, with conditional effects and axioms, that projections simulate every concrete transition and never overestimate, and that `perfect_heuristic` satisfies the Bellman equation. `tests/it/proptests.rs` runs [proptest](https://crates.io/crates/proptest) over arbitrary seeds and generator parameters; `task test:prop` raises it to 10000 cases.
- `task fuzz`: fuzzes the `output.sas` reader with [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) (nightly), seeded with the fixtures. Accepted input must round-trip and convert to a task that `validate` rejects or that steps without panics. A second target, `projection`, checks that projections of arbitrary random tasks never overestimate h\*. `task fuzz:smoke` runs both briefly.
- `task bench`: [criterion](https://crates.io/crates/criterion) benchmarks for parsing, projections and the perfect heuristic on translator output.

## Minimum supported Rust version

The library builds with Rust 1.85 (`rust-version` in `Cargo.toml`), with and without default features, and on `no_std` targets. Tests, benchmarks and fuzzing use current stable or nightly. Raising the MSRV is a minor release.

## Contributing

See [CONTRIBUTING.md](https://github.com/sunsided/sas-plus/blob/main/CONTRIBUTING.md). `task ci` runs what CI runs.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/sunsided/sas-plus/blob/main/LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](https://github.com/sunsided/sas-plus/blob/main/LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option. The example in [`examples/`](https://github.com/sunsided/sas-plus/tree/main/examples) is public domain ([CC0-1.0](https://github.com/sunsided/sas-plus/blob/main/LICENSES/CC0-1.0.txt)). The third-party fixtures in [`tests/fixtures/translated/`](https://github.com/sunsided/sas-plus/tree/main/tests/fixtures/translated) keep their own licenses (GPL-2.0-or-later and Apache-2.0) and are not part of the published crate. [`REUSE.toml`](https://github.com/sunsided/sas-plus/blob/main/REUSE.toml) lists the license of every file.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
