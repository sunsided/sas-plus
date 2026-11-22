//! Random tasks from arbitrary seeds and generator parameters are valid, and
//! a projection onto an arbitrary pattern never overestimates the optimal
//! cost `perfect_heuristic` computes, nor prunes a solvable state.
//!
//! The tests in `tests/it/properties.rs` check this for a fixed seed range;
//! the fuzzer explores seeds and parameters beyond it.

#![no_main]
#![forbid(unsafe_code)]

use libfuzzer_sys::fuzz_target;
use sas_plus::random::{RandomTaskParams, random_task};
use sas_plus::{Abstraction, Projection};

/// Generator parameters from raw fuzz bytes, kept small enough for the
/// exhaustive `perfect_heuristic` to stay fast.
fn params(bytes: [u8; 6]) -> RandomTaskParams {
    let [vars, domain, ops, cost, conditions, derived] = bytes;

    RandomTaskParams {
        num_vars: 1 + usize::from(vars % 5),
        max_domain: 2 + u32::from(domain % 3),
        num_ops: usize::from(ops % 16),
        max_cost: 1 + u32::from(cost % 8),
        max_conditions: usize::from(conditions % 3),
        num_derived: usize::from(derived % 4),
    }
}

fuzz_target!(|input: (u64, [u8; 6], u8)| {
    let (seed, bytes, mask) = input;
    let task = random_task(seed, params(bytes));
    assert_eq!(task.validate(), Ok(()), "seed {seed}");

    let pattern = (0..task.num_variables())
        .filter(|&v| v < 8 && mask & (1 << v) != 0)
        .collect();
    let projection = Projection::new(&task, pattern);
    let distances = projection.lts().goal_distances(&task.costs());
    let state = task.initial_state();
    let estimate = projection.abstract_state(&state).and_then(|s| distances[s]);

    match (estimate, task.perfect_heuristic()(&state)) {
        (Some(estimate), Some(optimal)) => assert!(estimate <= optimal, "seed {seed}"),
        (None, Some(_)) => panic!("seed {seed}: projection prunes a solvable state"),
        (_, None) => {}
    }
});
