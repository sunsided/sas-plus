use alloc::vec;
use alloc::vec::Vec;

use crate::ranking::Ranking;
use crate::{Abstraction, LabeledTransitionSystem, State, Task, Transition, Value, VarId};

/// Projection of a task onto a subset of variables (a "pattern"); the
/// abstraction underlying pattern databases.
///
/// Derived variables are dropped from the pattern, so conditions on them are
/// treated as satisfiable. A conditional effect whose conditions mention
/// variables outside the pattern may or may not fire, and both outcomes get a
/// transition. Both keep the abstraction admissible.
#[derive(Clone, Debug)]
pub struct Projection {
    ranking: Ranking,
    lts: LabeledTransitionSystem,
}

impl Projection {
    pub fn new(task: &Task, mut pattern: Vec<VarId>) -> Self {
        pattern.retain(|&v| !task.variables[v].is_derived());
        pattern.sort_unstable();
        pattern.dedup();
        let ranking = Ranking::new(task, pattern);

        let mut transitions = Vec::new();
        let mut fired = Vec::new();
        let mut dsts = Vec::new();
        for (label, op) in task.operators.iter().enumerate() {
            // Effects on pattern variables: (position, value, conditions, sure to fire if
            // their pattern conditions hold, i.e. no conditions outside the pattern).
            let effects: Vec<(usize, Value, &[crate::Fact], bool)> = op
                .eff
                .iter()
                .filter_map(|e| {
                    let i = ranking.position(e.fact.var)?;
                    let sure = e
                        .conditions
                        .iter()
                        .all(|c| ranking.position(c.var).is_some());
                    Some((i, e.fact.value, e.conditions.as_slice(), sure))
                })
                .collect();
            let unconditional = effects.iter().all(|e| e.2.is_empty());
            // Only abstract states consistent with the precondition.
            for src in ranking.matching(&op.pre) {
                if unconditional {
                    let dst = effects.iter().fold(src, |d, e| ranking.set(d, e.0, e.1));
                    transitions.push(Transition { src, label, dst });
                    continue;
                }
                fired.clear();
                fired.extend(
                    effects
                        .iter()
                        .filter(|e| ranking.holds(src, e.2))
                        .map(|e| (e.0, e.1, e.3)),
                );
                let maybe = fired.iter().filter(|e| !e.2).count();
                assert!(maybe < 24, "too many uncertain conditional effects");
                dsts.clear();
                for mask in 0..1usize << maybe {
                    let mut dst = src;
                    let mut bit = 0;
                    for &(i, val, sure) in &fired {
                        let fires = sure || {
                            bit += 1;
                            mask & 1 << (bit - 1) != 0
                        };
                        if fires {
                            dst = ranking.set(dst, i, val);
                        }
                    }
                    dsts.push(dst);
                }
                dsts.sort_unstable();
                dsts.dedup();
                transitions.extend(dsts.iter().map(|&dst| Transition { src, label, dst }));
            }
        }
        let mut goal = vec![false; ranking.num_states];
        ranking.matching(&task.goal).for_each(|s| goal[s] = true);
        let lts = LabeledTransitionSystem {
            num_states: ranking.num_states,
            num_labels: task.operators.len(),
            transitions,
            goal,
            initial: Some(ranking.rank(&task.initial)),
        };
        Self { ranking, lts }
    }

    pub fn pattern(&self) -> &[VarId] {
        &self.ranking.vars
    }

    /// Index of the abstract state that `state` projects to.
    pub fn rank(&self, state: &State) -> usize {
        self.ranking.rank(state)
    }
}

impl Abstraction for Projection {
    fn lts(&self) -> &LabeledTransitionSystem {
        &self.lts
    }
    fn abstract_state(&self, state: &State) -> Option<usize> {
        Some(self.rank(state))
    }
}
