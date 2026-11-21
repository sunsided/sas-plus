//! Projections of a task onto a pattern of variables ([`Projection`]).

use alloc::vec;
use alloc::vec::Vec;

use crate::ranking::Ranking;
use crate::{
    Abstraction, Fact, LabeledTransitionSystem, Operator, State, Task, Transition, Value, VarId,
};

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
    /// The perfect hash of the pattern's assignments, i.e. the abstract states.
    ranking: Ranking,
    /// The abstract transition system, labeled by operator id.
    lts: LabeledTransitionSystem,
}

impl Projection {
    /// Projects `task` onto `pattern`. Derived variables and duplicates are
    /// dropped from the pattern, which is then sorted.
    ///
    /// # Panics
    ///
    /// If `pattern` names a variable `task` does not have, or the abstract
    /// state space has more than `usize::MAX` states.
    #[must_use]
    pub fn new(task: &Task, mut pattern: Vec<VarId>) -> Self {
        pattern.retain(|&v| !task.variables[v].is_derived());
        pattern.sort_unstable();
        pattern.dedup();
        let ranking = Ranking::new(task, pattern);

        let mut builder = TransitionBuilder::new(&ranking);
        for (label, op) in task.operators.iter().enumerate() {
            builder.add_operator(label, op);
        }

        let transitions = builder.transitions;
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

    /// The pattern: the projected state variables, sorted.
    #[must_use]
    pub fn pattern(&self) -> &[VarId] {
        &self.ranking.vars
    }

    /// Index of the abstract state that `state` projects to.
    ///
    /// # Panics
    ///
    /// If `state` lacks a pattern variable.
    #[must_use]
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

/// An effect of an operator on a pattern variable.
struct PatternEffect<'a> {
    /// Position of the effect's variable in the pattern.
    position: usize,
    /// The value the effect sets.
    value: Value,
    /// The effect's conditions, inside and outside the pattern.
    conditions: &'a [Fact],
    /// Sure to fire when its pattern conditions hold, i.e. it has no
    /// conditions outside the pattern.
    sure: bool,
}

/// The effects of `op` on pattern variables, in effect order.
fn pattern_effects<'a>(ranking: &Ranking, op: &'a Operator) -> Vec<PatternEffect<'a>> {
    op.eff
        .iter()
        .filter_map(|e| {
            let position = ranking.position(e.fact.var)?;
            let sure = e
                .conditions
                .iter()
                .all(|c| ranking.position(c.var).is_some());

            Some(PatternEffect {
                position,
                value: e.fact.value,
                conditions: e.conditions.as_slice(),
                sure,
            })
        })
        .collect()
}

/// Collects the abstract transitions of a projection, with scratch buffers
/// reused across abstract states and operators.
struct TransitionBuilder<'r> {
    /// The abstract states.
    ranking: &'r Ranking,
    /// The transitions collected so far, in operator and source order.
    transitions: Vec<Transition>,
    /// Per pattern position, the values it may end with in the current
    /// source state; non-empty only for positions in `touched`.
    outcomes: Vec<Vec<Value>>,
    /// The positions with outcomes in the current source state, in the order
    /// their first effect appears.
    touched: Vec<usize>,
    /// Odometer over the outcomes of the touched positions.
    digits: Vec<usize>,
}

impl<'r> TransitionBuilder<'r> {
    /// An empty builder for the abstract states of `ranking`.
    fn new(ranking: &'r Ranking) -> Self {
        Self {
            ranking,
            transitions: Vec::new(),
            outcomes: vec![Vec::new(); ranking.vars.len()],
            touched: Vec::new(),
            digits: Vec::new(),
        }
    }

    /// Adds the transitions of operator `op` (with id `label`) from every
    /// abstract state consistent with its precondition.
    fn add_operator(&mut self, label: usize, op: &Operator) {
        let ranking = self.ranking;
        let effects = pattern_effects(ranking, op);
        let unconditional = effects.iter().all(|e| e.conditions.is_empty());

        for src in ranking.matching(&op.pre) {
            if unconditional {
                let dst = effects
                    .iter()
                    .fold(src, |d, e| ranking.set(d, e.position, e.value));
                self.transitions.push(Transition { src, label, dst });
                continue;
            }

            self.collect_outcomes(src, &effects);
            self.push_combinations(src, label);
        }
    }

    /// Fills `outcomes` and `touched` with the values each pattern variable
    /// may end with when the operator is applied in `src`.
    ///
    /// Effects apply in order, so a variable ends with the value of its last
    /// firing effect. A sure effect fixes that value; a maybe effect adds an
    /// alternative.
    fn collect_outcomes(&mut self, src: usize, effects: &[PatternEffect<'_>]) {
        for &i in &self.touched {
            self.outcomes[i].clear();
        }

        self.touched.clear();
        for e in effects {
            if !self.ranking.holds(src, e.conditions) {
                continue;
            }

            let values = &mut self.outcomes[e.position];
            if values.is_empty() {
                self.touched.push(e.position);
                values.push(self.ranking.value(src, e.position));
            }

            if e.sure {
                values.clear();
                values.push(e.value);
            } else if !values.contains(&e.value) {
                values.push(e.value);
            }
        }
    }

    /// Pushes one transition from `src` per combination of the collected
    /// outcomes. Each effect sets one variable, so the variables are
    /// independent and the successors are all combinations.
    fn push_combinations(&mut self, src: usize, label: usize) {
        self.digits.clear();
        self.digits.resize(self.touched.len(), 0);
        loop {
            let dst = self
                .touched
                .iter()
                .zip(&self.digits)
                .fold(src, |d, (&i, &k)| {
                    self.ranking.set(d, i, self.outcomes[i][k])
                });
            self.transitions.push(Transition { src, label, dst });

            let Some(j) = (0..self.touched.len())
                .find(|&j| self.digits[j] + 1 < self.outcomes[self.touched[j]].len())
            else {
                break;
            };
            self.digits[j] += 1;
            self.digits[..j].fill(0);
        }
    }
}
