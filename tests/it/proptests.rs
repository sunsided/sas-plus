//! Property tests over arbitrary seeds and generator parameters, beyond the
//! fixed seed range of `properties`. Raise the case count with
//! `task test:prop` (`PROPTEST_CASES`).

use proptest::prelude::*;
use sas_plus::random::{RandomTaskParams, Rng, random_task};
use sas_plus::{CausalGraph, State, Task};

/// Generator parameters small enough for `perfect_heuristic` to stay fast.
fn small_params() -> impl Strategy<Value = RandomTaskParams> {
    (
        1..=5_usize,
        2..=4_u32,
        0..=12_usize,
        1..=6_u32,
        0..=2_usize,
        0..=3_usize,
    )
        .prop_map(
            |(num_vars, max_domain, num_ops, max_cost, max_conditions, num_derived)| {
                RandomTaskParams {
                    num_vars,
                    max_domain,
                    num_ops,
                    max_cost,
                    max_conditions,
                    num_derived,
                }
            },
        )
}

/// Whether every value of `state` lies in its variable's domain.
fn in_domain(task: &Task, state: &State) -> bool {
    state.0.len() == task.num_variables()
        && state
            .0
            .iter()
            .zip(&task.variables)
            .all(|(&value, var)| value < var.domain_size)
}

proptest! {
    /// `Rng::range` stays within `lo..=hi`, including the full `u64` range.
    #[test]
    fn rng_range_stays_in_bounds(seed: u64, a: u64, b: u64) {
        let (lo, hi) = (a.min(b), a.max(b));
        let mut rng = Rng::new(seed);

        for _ in 0..16 {
            let x = rng.range(lo, hi);
            prop_assert!(lo <= x && x <= hi, "{x} outside {lo}..={hi}");
        }
    }

    /// Every generated task passes `validate`, for any seed and parameters.
    #[test]
    fn generated_tasks_are_valid(seed: u64, params in small_params()) {
        let task = random_task(seed, params);
        prop_assert_eq!(task.validate(), Ok(()));
    }

    /// The causal graph lists each arc on both ends, sorted, without self-loops.
    #[test]
    fn causal_graph_arcs_are_mirrored(seed: u64, params in small_params()) {
        let task = random_task(seed, params);
        let graph = CausalGraph::new(&task);

        for u in 0..task.num_variables() {
            let successors = graph.successors(u);
            prop_assert!(successors.is_sorted());
            prop_assert!(!successors.contains(&u));
            prop_assert!(graph.predecessors(u).is_sorted());

            for &v in successors {
                prop_assert!(graph.predecessors(v).contains(&u), "{u} -> {v} not mirrored");
            }
        }
    }

    /// Random walks stay inside the variable domains, and `validate_plan`
    /// charges exactly the summed operator costs of a walk that ends in a goal.
    #[test]
    fn walks_stay_in_domain_and_cost_their_operators(
        seed: u64,
        walk_seed: u64,
        params in small_params(),
    ) {
        let task = random_task(seed, params);
        let mut rng = Rng::new(walk_seed);
        let mut state = task.initial_state();
        let mut plan = Vec::new();

        for _ in 0..12 {
            prop_assert!(in_domain(&task, &state));
            let ops: Vec<_> = task.applicable_operators(&state).collect();
            let Some(&op) = ops.get(usize::try_from(rng.next_u64() % 16).unwrap()) else {
                break;
            };
            state = task.apply(op, &state);
            plan.push(op);
        }

        let cost: u64 = plan.iter().map(|&op| u64::from(task.operators[op].cost)).sum();
        let expected = task.is_goal(&state).then_some(cost);
        prop_assert_eq!(task.validate_plan(&plan), expected);
    }

    /// `perfect_heuristic` is zero in the initial state exactly when it is a goal.
    #[test]
    fn perfect_heuristic_is_zero_exactly_on_goals(seed: u64, params in small_params()) {
        let task = random_task(seed, params);
        let h = task.perfect_heuristic();
        let initial = task.initial_state();

        prop_assert_eq!(h(&initial) == Some(0), task.is_goal(&initial));
    }
}
