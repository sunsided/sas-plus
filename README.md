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

`Task` and `LabeledTransitionSystem` are plain structs with public fields: a task is its variables, operators, initial state and goal, and a transition system is a list of labeled transitions with a goal flag per state. The one abstraction on top is the `Abstraction` trait, which maps a concrete state to a state of a transition system; anything that implements it, a `Projection` or a type of your own, can stand in wherever a heuristic needs abstract goal distances. The crate has no dependencies.

For testing, `Task::perfect_heuristic()` computes the exact goal distance h\* on small tasks, and `random::random_task` generates reproducible tasks from a seed, so a heuristic can be checked against h\* over as many seeds as you care to run. On top of that, the `fdr` module reads Fast Downward's translator output, which brings real benchmark domains in without writing a grounder.

There is no search algorithm and no PDDL parser, and `Task` does not yet support conditional effects or axioms (see the roadmap).

## Usage

Ground PDDL with Fast Downward's translator, then load the result in Rust:

```rust
use sas_plus::fdr::SasFile;

let file: SasFile = std::fs::read_to_string("output.sas")?.parse()?;
let task = file.to_task()?;              // plain SAS+ task
// ... search ...
print!("{}", file.format_plan(&plan));   // sas_plan format, ready for VAL
```

To try both steps with Docker and [Task](https://taskfile.dev), run `task ground:load`. It grounds a fixture with the `aibasel/downward` image and loads the result with `examples/load.rs`. Pass `DOMAIN=... PROBLEM=...` to ground your own PDDL (paths inside the repository).

## What's in it
- **`fdr`**: reader and writer for `output.sas` (file version 3).
  - Faithful to the format: mutex groups, conditional effects, axioms, and fact names are all kept.
  - Writes back byte-for-byte what it reads.
  - Parse errors carry line numbers.
  - `to_task()` converts to a plain `Task` and rejects conditional effects and axioms with a typed error.
  - Metric `0` means unit costs, exactly as in Fast Downward's search.
- **`Task`, `Operator`, `Fact`, `State`**, with validation, applicability and plan checking.
- **`LabeledTransitionSystem`** and the **`Abstraction` trait** shared by [`merge-and-shrink`](https://github.com/sunsided/merge-and-shrink) and [`cost-partition`](https://github.com/sunsided/cost-partition).
- **`Projection`**: the pattern-database abstraction.
- **Test oracles**: `Task::perfect_heuristic()` (exact h\*) and `random::random_task`.

## Fixtures
- `tests/fixtures/translated/`: real Fast Downward translator output for eight problems, from the VHPOP and Potoroo corpora plus two written for this repo. It covers unit costs, action costs and conditional effects. Every file round-trips byte for byte, and the small tasks are checked against their known optima. The folder includes GPL-2.0 / Apache-2.0 test inputs and is excluded from the published crate.
- `tests/fixtures/`: small hand-encoded files, including the only axiom (derived-variable) example.

## Roadmap
- A translator fixture with axioms (derived predicates).
- Conditional effects and axioms in `Task` itself.
- A bridge from a pure-Rust PDDL parser and grounder.
- A causal graph.
