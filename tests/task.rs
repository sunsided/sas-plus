use sas_plus::random::{RandomTaskParams, Rng, random_task};
use sas_plus::{Abstraction, Fact, Operator, Projection, State, Task, TaskError, Variable};

fn tiny() -> Task {
    let var = |name: &str| Variable {
        name: name.into(),
        domain_size: 2,
    };
    Task {
        variables: vec![var("a"), var("b")],
        operators: vec![Operator {
            name: "op".into(),
            pre: vec![Fact::new(0, 0)],
            eff: vec![Fact::new(0, 1), Fact::new(1, 1)],
            cost: 1,
        }],
        initial: State(vec![0, 0]),
        goal: vec![Fact::new(1, 1)],
    }
}

#[test]
fn validate_reports_each_kind_of_error() {
    assert_eq!(tiny().validate(), Ok(()));

    let mut t = tiny();
    t.initial.0.push(0);
    assert_eq!(t.validate(), Err(TaskError::InitialStateArity));

    let mut t = tiny();
    t.goal[0].value = 2;
    let fact = Fact::new(1, 2);
    assert_eq!(t.validate(), Err(TaskError::ValueOutOfRange { fact }));

    let mut t = tiny();
    t.operators[0].pre[0].var = 2;
    let fact = Fact::new(2, 0);
    assert_eq!(t.validate(), Err(TaskError::UnknownVariable { fact }));

    let mut t = tiny();
    t.operators[0].eff[1] = Fact::new(0, 0);
    assert_eq!(
        t.validate(),
        Err(TaskError::DuplicateEffect { op: 0, var: 0 })
    );
}

/// Projections onto a strict subset of the variables never overestimate h*.
#[test]
fn partial_projections_are_admissible() {
    let mut rng = Rng::new(1);
    for seed in 0..50 {
        let task = random_task(seed, RandomTaskParams::default());
        let h_star = task.perfect_heuristic()(&task.initial);
        let n = task.num_variables();
        let pattern: Vec<usize> = (0..n)
            .filter(|_| rng.range(0, 1) == 1)
            .take(n - 1)
            .collect();
        let p = Projection::new(&task, pattern);
        let h = p.lts().goal_distances(&task.costs())[p.abstract_state(&task.initial).unwrap()];
        match (h, h_star) {
            (Some(h), Some(h_star)) => assert!(h <= h_star, "seed {seed}: {h} > {h_star}"),
            (None, Some(_)) => panic!("seed {seed}: projection claims a dead end"),
            (_, None) => {}
        }
    }
}
