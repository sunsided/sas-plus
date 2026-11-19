//! Properties checked on many random tasks, with conditional effects and
//! axioms. Failures name the seed, which reproduces the task.

#![cfg(feature = "random")]

use sas_plus::random::{RandomTaskParams, Rng, random_task};
use sas_plus::{Abstraction, Projection, State, Task, Transition};

const SEEDS: u64 = 200;

fn params(seed: u64) -> RandomTaskParams {
    RandomTaskParams {
        max_conditions: (seed % 3) as usize,
        num_derived: (seed % 4) as usize,
        ..RandomTaskParams::default()
    }
}

/// A uniformly random state, with derived variables evaluated.
fn random_state(task: &Task, rng: &mut Rng) -> State {
    let mut s = State(
        task.variables
            .iter()
            .map(|v| rng.range(0, u64::from(v.domain_size) - 1) as u32)
            .collect(),
    );
    task.evaluate_axioms(&mut s);
    s
}

fn random_pattern(task: &Task, rng: &mut Rng) -> Vec<usize> {
    (0..task.num_variables())
        .filter(|_| rng.range(0, 1) == 1)
        .collect()
}

#[test]
fn generated_tasks_are_valid() {
    let mut derived = 0;
    for seed in 0..SEEDS {
        let task = random_task(seed, params(seed));
        task.validate()
            .unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        derived += task.axioms.len();
    }
    assert!(derived > 100);
}

/// Every concrete transition shows up in the projection, and every concrete
/// goal state projects to an abstract goal.
#[test]
fn projections_simulate_the_task() {
    let mut rng = Rng::new(3);
    for seed in 0..SEEDS {
        let task = random_task(seed, params(seed));
        let p = Projection::new(&task, random_pattern(&task, &mut rng));
        let lts = p.lts();
        for _ in 0..20 {
            let s = random_state(&task, &mut rng);
            let src = p.rank(&s);
            if task.is_goal(&s) {
                assert!(lts.goal[src], "seed {seed}: goal {s:?} not abstract goal");
            }
            for op in task.applicable_operators(&s) {
                let t = Transition {
                    src,
                    label: op,
                    dst: p.rank(&task.apply(op, &s)),
                };
                assert!(
                    lts.transitions.contains(&t),
                    "seed {seed}: {t:?} from {s:?} missing"
                );
            }
        }
    }
}

/// h* is 0 exactly on goals, consistent, and achieved by some operator.
#[test]
fn perfect_heuristic_satisfies_the_bellman_equation() {
    let mut rng = Rng::new(4);
    for seed in 0..SEEDS {
        let task = random_task(seed, params(seed));
        let h = task.perfect_heuristic();
        for _ in 0..20 {
            let s = random_state(&task, &mut rng);
            let successors: Vec<Option<u64>> = task
                .applicable_operators(&s)
                .map(|op| Some(u64::from(task.operators[op].cost) + h(&task.apply(op, &s))?))
                .collect();
            let best = successors.iter().flatten().min().copied();
            let expected = if task.is_goal(&s) { Some(0) } else { best };
            assert_eq!(h(&s), expected, "seed {seed}: {s:?}");
        }
    }
}

/// Projections never overestimate h*, with axioms and conditional effects.
#[test]
fn projections_are_admissible_in_every_state() {
    let mut rng = Rng::new(5);
    for seed in 0..SEEDS {
        let task = random_task(seed, params(seed));
        let h_star = task.perfect_heuristic();
        let p = Projection::new(&task, random_pattern(&task, &mut rng));
        let h = p.lts().goal_distances(&task.costs());
        for _ in 0..20 {
            let s = random_state(&task, &mut rng);
            let abstract_h = h[p.abstract_state(&s).unwrap()];
            match (abstract_h, h_star(&s)) {
                (Some(a), Some(b)) => assert!(a <= b, "seed {seed}: {a} > {b} in {s:?}"),
                (None, Some(_)) => panic!("seed {seed}: projection prunes solvable {s:?}"),
                (_, None) => {}
            }
        }
    }
}
