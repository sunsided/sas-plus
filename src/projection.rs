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
///
/// An operator gets one transition per abstract state matching its
/// precondition, times the product, over the pattern variables it may set
/// through such uncertain effects, of the number of values each may end with.
/// That is never more than the number of abstract states, but on a large
/// pattern the transition list can grow toward
/// `operators × abstract states × abstract states`; size patterns accordingly.
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
        // Per pattern position, the values it may end with; reused across states.
        let mut outcomes: Vec<Vec<Value>> = vec![Vec::new(); ranking.vars.len()];
        let mut touched: Vec<usize> = Vec::new();
        let mut digits: Vec<usize> = Vec::new();
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
                // Effects apply in order, so a variable ends with the value of its last
                // firing effect. A sure effect fixes that value; a maybe effect adds an
                // alternative. Each effect sets one variable, so the variables are
                // independent and the successors are all combinations.
                for &i in &touched {
                    outcomes[i].clear();
                }
                touched.clear();
                for &(i, value, conditions, sure) in &effects {
                    if !ranking.holds(src, conditions) {
                        continue;
                    }
                    let values = &mut outcomes[i];
                    if values.is_empty() {
                        touched.push(i);
                        values.push(ranking.value(src, i) as Value);
                    }
                    if sure {
                        values.clear();
                        values.push(value);
                    } else if !values.contains(&value) {
                        values.push(value);
                    }
                }
                digits.clear();
                digits.resize(touched.len(), 0);
                loop {
                    let dst = touched
                        .iter()
                        .zip(&digits)
                        .fold(src, |d, (&i, &k)| ranking.set(d, i, outcomes[i][k]));
                    transitions.push(Transition { src, label, dst });
                    let Some(j) =
                        (0..touched.len()).find(|&j| digits[j] + 1 < outcomes[touched[j]].len())
                    else {
                        break;
                    };
                    digits[j] += 1;
                    digits[..j].fill(0);
                }
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
