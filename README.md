# sas-plus

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

There is no search algorithm and no PDDL parser. To ground PDDL in Rust, see the roadmap.

## Usage

Ground PDDL with Fast Downward's translator, then load the result in Rust:

```rust
use sas_plus::fdr::SasFile;

let file: SasFile = std::fs::read_to_string("output.sas")?.parse()?;
let task = file.to_task();               // Task with conditional effects and axioms
// ... search ...
print!("{}", file.format_plan(&plan));   // sas_plan format, ready for VAL
```

To try both steps with Docker and [Task](https://taskfile.dev), run `task ground:load`. It grounds a fixture with the `aibasel/downward` image and loads the result with `examples/load.rs`. Pass `DOMAIN=... PROBLEM=...` to ground your own PDDL (paths inside the repository).

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
- `tests/fixtures/translated/`: real Fast Downward translator output for eight problems, from the VHPOP and Potoroo corpora plus two written for this repo. It covers unit costs, action costs, conditional effects and derived predicates. Every file round-trips byte for byte, and the small tasks are checked against their known optima. The folder includes GPL-2.0 / Apache-2.0 test inputs and is excluded from the published crate.
- `tests/fixtures/`: small hand-encoded files, including a minimal axiom (derived-variable) example.

## Testing
- `task test`: unit, fixture and property tests. `tests/properties.rs` checks on random tasks, with conditional effects and axioms, that projections simulate every concrete transition and never overestimate, and that `perfect_heuristic` satisfies the Bellman equation.
- `task fuzz`: fuzzes the `output.sas` reader with [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) (nightly). Accepted input must round-trip and convert to a task that `validate` rejects or that steps without panics.
- `task bench`: [criterion](https://crates.io/crates/criterion) benchmarks for parsing, projections and the perfect heuristic on translator output.

## Roadmap
- A PDDL bridge in the pure-Rust [`pddl`](https://crates.io/crates/pddl) crate, behind its optional `sas-plus` feature, that grounds PDDL into an `fdr::SasFile`.
