use crate::{Abstraction, LabeledTransitionSystem, State, Task, Transition, VarId};

/// Projection of a task onto a subset of variables (a "pattern"); the
/// abstraction underlying pattern databases.
#[derive(Clone, Debug)]
pub struct Projection {
    pattern: Vec<VarId>,
    multipliers: Vec<usize>,
    lts: LabeledTransitionSystem,
}

impl Projection {
    pub fn new(task: &Task, mut pattern: Vec<VarId>) -> Self {
        pattern.sort_unstable();
        pattern.dedup();
        let domains: Vec<usize> = pattern
            .iter()
            .map(|&v| task.variables[v].domain_size as usize)
            .collect();
        let mut multipliers = Vec::with_capacity(pattern.len());
        let mut num_states = 1usize;
        for &d in &domains {
            multipliers.push(num_states);
            num_states = num_states
                .checked_mul(d)
                .expect("projection too large to enumerate");
        }
        let mut position = vec![None; task.num_variables()];
        for (i, &v) in pattern.iter().enumerate() {
            position[v] = Some(i);
        }
        let value = |s: usize, i: usize| (s / multipliers[i]) % domains[i];

        let mut transitions = Vec::new();
        for (label, op) in task.operators.iter().enumerate() {
            for src in 0..num_states {
                let consistent = op.pre.iter().all(|f| match position[f.var] {
                    Some(i) => value(src, i) == f.value as usize,
                    None => true,
                });
                if !consistent {
                    continue;
                }
                let mut dst = src;
                for f in &op.eff {
                    if let Some(i) = position[f.var] {
                        dst = dst - value(dst, i) * multipliers[i]
                            + f.value as usize * multipliers[i];
                    }
                }
                transitions.push(Transition { src, label, dst });
            }
        }
        let goal = (0..num_states)
            .map(|s| {
                task.goal.iter().all(|f| match position[f.var] {
                    Some(i) => value(s, i) == f.value as usize,
                    None => true,
                })
            })
            .collect();
        let mut this = Self {
            pattern,
            multipliers,
            lts: LabeledTransitionSystem {
                num_states,
                num_labels: task.operators.len(),
                transitions,
                goal,
                initial: None,
            },
        };
        this.lts.initial = Some(this.rank(&task.initial));
        this
    }

    pub fn pattern(&self) -> &[VarId] {
        &self.pattern
    }

    /// Index of the abstract state that `state` projects to.
    pub fn rank(&self, state: &State) -> usize {
        self.pattern
            .iter()
            .zip(&self.multipliers)
            .map(|(&v, &m)| state.0[v] as usize * m)
            .sum()
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
