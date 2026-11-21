use sas_plus::random::{RandomTaskParams, Rng, random_task};
use sas_plus::{
    Abstraction, Axiom, CausalGraph, Effect, Fact, LabeledTransitionSystem, Operator, Projection,
    State, Task, TaskError, Transition, Variable,
};

fn tiny() -> Task {
    let var = |name: &str| Variable::new(name, 2);
    Task {
        variables: vec![var("a"), var("b")],
        operators: vec![Operator {
            name: "op".into(),
            pre: vec![Fact::new(0, 0)],
            eff: vec![Effect::new(Fact::new(0, 1)), Effect::new(Fact::new(1, 1))],
            cost: 1,
        }],
        initial: State(vec![0, 0]),
        goal: vec![Fact::new(1, 1)],
        axioms: vec![],
    }
}

/// `tiny` plus a derived variable `c` (layer 0, default 0) with `c = 1 ← b = 1`.
fn tiny_derived() -> Task {
    let mut t = tiny();
    t.variables.push(Variable {
        axiom_layer: Some(0),
        ..Variable::new("c", 2)
    });
    t.initial.0.push(0);
    t.goal = vec![Fact::new(2, 1)];
    t.axioms.push(Axiom {
        conditions: vec![Fact::new(1, 1)],
        head: Fact::new(2, 1),
    });
    t
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
    t.operators[0].eff[1] = Effect::new(Fact::new(0, 0));
    assert_eq!(
        t.validate(),
        Err(TaskError::DuplicateEffect { op: 0, var: 0 })
    );

    let mut t = tiny();
    t.operators[0].pre.push(Fact::new(0, 1));
    assert_eq!(
        t.validate(),
        Err(TaskError::ContradictoryPrecondition { op: 0, var: 0 })
    );
    // A contradictory operator is never applicable, also in projections.
    let p = Projection::new(&t, vec![0]);
    assert!(p.lts().transitions.is_empty());

    let mut t = tiny();
    t.operators[0].pre.push(Fact::new(0, 0));
    t.goal.extend([Fact::new(1, 1), Fact::new(1, 0)]);
    assert_eq!(t.validate(), Err(TaskError::ContradictoryGoal { var: 1 }));

    // Conditional effects on the same variable are fine.
    let mut t = tiny();
    t.operators[0].eff[1] = Effect {
        conditions: vec![Fact::new(1, 0)],
        fact: Fact::new(0, 0),
    };
    assert_eq!(t.validate(), Ok(()));

    assert_eq!(tiny_derived().validate(), Ok(()));

    let mut t = tiny_derived();
    t.operators[0].eff[1].fact = Fact::new(2, 1);
    assert_eq!(
        t.validate(),
        Err(TaskError::DerivedVariableEffect { op: 0, var: 2 })
    );

    let mut t = tiny_derived();
    t.axioms[0].head = Fact::new(1, 1);
    assert_eq!(
        t.validate(),
        Err(TaskError::AxiomHeadNotDerived { axiom: 0 })
    );

    let mut t = tiny_derived();
    t.variables.push(Variable {
        axiom_layer: Some(1),
        ..Variable::new("d", 2)
    });
    t.initial.0.push(0);
    t.axioms[0].conditions.push(Fact::new(3, 0));
    assert_eq!(t.validate(), Err(TaskError::AxiomLayer { axiom: 0 }));

    // Negation within one layer: c = 1 ← c2 = 0 with c2 in c's layer.
    let mut t = tiny_derived();
    t.variables.push(Variable {
        axiom_layer: Some(0),
        ..Variable::new("c2", 2)
    });
    t.initial.0.push(0);
    t.axioms[0].conditions.push(Fact::new(3, 0));
    assert_eq!(t.validate(), Err(TaskError::AxiomLayer { axiom: 0 }));
    // A positive condition on the same layer is fine.
    t.axioms[0].conditions[1] = Fact::new(3, 1);
    assert_eq!(t.validate(), Ok(()));
}

#[test]
fn conditional_effects_read_the_state_before_the_operator() {
    let mut t = tiny();
    // b = 1 only if a was 1 before: `op` itself sets a = 1, which must not count.
    t.operators[0].eff[1].conditions.push(Fact::new(0, 1));
    let next = t.apply(0, &t.initial);
    assert_eq!(next, State(vec![1, 0]));
    assert_eq!(t.perfect_heuristic()(&t.initial), None);
    assert_eq!(t.validate_plan(&[0]), None);
}

#[test]
fn axioms_are_evaluated_after_each_operator() {
    let t = tiny_derived();
    assert_eq!(t.initial_state(), State(vec![0, 0, 0]));
    assert_eq!(t.apply(0, &t.initial), State(vec![1, 1, 1]));
    assert_eq!(t.validate_plan(&[0]), Some(1));
    assert_eq!(t.perfect_heuristic()(&t.initial), Some(1));

    // Negation as failure: d = 1 ← c = 0 sits one layer above c.
    let mut t = tiny_derived();
    t.variables.push(Variable {
        axiom_layer: Some(1),
        ..Variable::new("d", 2)
    });
    t.initial.0.push(0);
    t.axioms.push(Axiom {
        conditions: vec![Fact::new(2, 0)],
        head: Fact::new(3, 1),
    });
    t.validate().unwrap();
    assert_eq!(t.initial_state(), State(vec![0, 0, 0, 1]));
    assert_eq!(t.apply(0, &t.initial), State(vec![1, 1, 1, 0]));

    // A derived variable without rules is reset to its default.
    let mut t = tiny_derived();
    t.axioms.clear();
    let mut s = State(vec![0, 0, 1]);
    t.evaluate_axioms(&mut s);
    assert_eq!(s, State(vec![0, 0, 0]));

    // A rule deriving the default value never fires, so evaluation terminates.
    let mut t = tiny_derived();
    t.axioms.push(Axiom {
        conditions: vec![Fact::new(1, 1)],
        head: Fact::new(2, 0),
    });
    assert_eq!(t.apply(0, &t.initial), State(vec![1, 1, 1]));
}

#[test]
fn causal_graph_links_conditions_to_effects() {
    let mut t = tiny_derived();
    t.operators[0].eff[1].conditions.push(Fact::new(2, 0));
    let cg = CausalGraph::new(&t);
    // pre a → b, eff a ↔ b, condition c → b, axiom b → c; no self-loops.
    assert_eq!(cg.successors(0), [1]);
    assert_eq!(cg.successors(1), [0, 2]);
    assert_eq!(cg.successors(2), [1]);
    assert_eq!(cg.predecessors(1), [0, 2]);
    assert_eq!(cg.predecessors(2), [1]);
}

/// Projections stay admissible with conditional effects, whose conditions
/// may lie outside the pattern.
#[test]
fn projections_with_conditional_effects_are_admissible() {
    let params = RandomTaskParams {
        max_conditions: 2,
        ..RandomTaskParams::default()
    };
    let mut rng = Rng::new(2);
    let mut conditional = 0;
    for seed in 0..100 {
        let task = random_task(seed, params);
        task.validate().unwrap();
        conditional += task
            .operators
            .iter()
            .flat_map(|o| &o.eff)
            .filter(|e| !e.conditions.is_empty())
            .count();
        let h_star = task.perfect_heuristic()(&task.initial);
        let pattern: Vec<usize> = (0..task.num_variables())
            .filter(|_| rng.range(0, 1) == 1)
            .collect();
        let p = Projection::new(&task, pattern);
        let h = p.lts().goal_distances(&task.costs())[p.abstract_state(&task.initial).unwrap()];
        match (h, h_star) {
            (Some(h), Some(h_star)) => assert!(h <= h_star, "seed {seed}: {h} > {h_star}"),
            (None, Some(_)) => panic!("seed {seed}: projection claims a dead end"),
            (_, None) => {}
        }
        // Onto all variables, the projection is exact.
        let full = Projection::new(&task, (0..task.num_variables()).collect());
        let h = full.lts().goal_distances(&task.costs())[full.rank(&task.initial)];
        assert_eq!(h, h_star, "seed {seed}");
    }
    assert!(conditional > 50);
}

/// Derived variables are left out of projections, which keeps them admissible.
#[test]
fn projections_ignore_derived_variables() {
    let t = tiny_derived();
    let p = Projection::new(&t, vec![1, 2]);
    assert_eq!(p.pattern(), [1]);
    let h = p.lts().goal_distances(&t.costs());
    assert_eq!(h[p.rank(&t.initial)], Some(0));
}

/// Many conditional effects on one variable, all conditioned outside the
/// pattern, give one transition per value the variable may end with, not
/// one per subset of effects.
#[test]
fn projections_handle_many_uncertain_effects() {
    let n = 30;
    let mut variables = vec![Variable::new("x", 3)];
    variables.extend((0..n).map(|i| Variable::new(format!("c{i}"), 2)));
    let eff = (0..n)
        .map(|i| Effect {
            conditions: vec![Fact::new(i + 1, 1)],
            fact: Fact::new(0, 1 + (i % 2) as u32),
        })
        .collect();
    let task = Task {
        variables,
        operators: vec![Operator {
            name: "op".into(),
            pre: vec![],
            eff,
            cost: 1,
        }],
        initial: State(vec![0; n + 1]),
        goal: vec![Fact::new(0, 2)],
        axioms: vec![],
    };
    task.validate().unwrap();
    let p = Projection::new(&task, vec![0]);
    let mut from_0: Vec<usize> = p
        .lts()
        .transitions
        .iter()
        .filter(|t| t.src == 0)
        .map(|t| t.dst)
        .collect();
    from_0.sort_unstable();
    assert_eq!(from_0, [0, 1, 2]);
    // From x = 1 or x = 2 only the other non-zero value is new.
    assert_eq!(p.lts().transitions.len(), 7);
}

#[test]
fn init_distances_follow_transitions_forward() {
    let t = |src, label, dst| Transition { src, label, dst };
    let lts = LabeledTransitionSystem {
        num_states: 4,
        num_labels: 2,
        transitions: vec![t(0, 0, 1), t(1, 1, 2), t(3, 0, 0)],
        goal: vec![false, false, true, false],
        initial: Some(0),
    };
    assert_eq!(
        lts.init_distances(&[2, 5]),
        [Some(0), Some(2), Some(7), None]
    );
    assert_eq!(
        lts.goal_distances(&[2, 5]),
        [Some(7), Some(5), Some(0), Some(9)]
    );
}

#[test]
fn distances_saturate_instead_of_wrapping() {
    let t = |src, label, dst| Transition { src, label, dst };
    let lts = LabeledTransitionSystem {
        num_states: 3,
        num_labels: 2,
        transitions: vec![t(0, 0, 1), t(1, 1, 2)],
        goal: vec![false, false, true],
        initial: Some(0),
    };
    assert_eq!(lts.goal_distances(&[1, u64::MAX])[0], Some(u64::MAX));
}

#[test]
fn rng_range_covers_the_full_domain() {
    let mut rng = Rng::new(1);
    rng.range(0, u64::MAX);
    assert_eq!(rng.range(7, 7), 7);
}

#[test]
#[should_panic(expected = "empty range")]
fn rng_range_rejects_empty_ranges() {
    Rng::new(1).range(5, 3);
}
