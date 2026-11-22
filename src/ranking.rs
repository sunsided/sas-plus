//! Perfect hashing of partial assignments, shared by projections and the
//! perfect heuristic.

use alloc::vec;
use alloc::vec::Vec;

use crate::{Fact, State, Task, Value, VarId};

/// Perfect hash of the assignments to a set of variables (mixed radix, the
/// first variable varying fastest).
#[derive(Clone, Debug)]
pub(crate) struct Ranking {
    /// The ranked variables, in ranking order (the first varies fastest).
    pub vars: Vec<VarId>,
    /// Per position, the rank step of one value of that variable: the
    /// product of the domain sizes before it.
    multipliers: Vec<usize>,
    /// Per position, the domain size of that variable.
    domains: Vec<usize>,
    /// Position of each task variable in `vars`.
    position: Vec<Option<usize>>,
    /// The number of ranks: the product of all domain sizes.
    pub num_states: usize,
}

impl Ranking {
    /// Ranks the assignments to `vars` of `task`.
    ///
    /// # Panics
    ///
    /// If the product of the domain sizes overflows `usize`.
    pub fn new(task: &Task, vars: Vec<VarId>) -> Self {
        let domains: Vec<usize> = vars
            .iter()
            .map(|&v| task.variables[v].domain_size as usize)
            .collect();
        let mut multipliers = Vec::with_capacity(vars.len());
        let mut num_states = 1usize;
        for &d in &domains {
            multipliers.push(num_states);
            num_states = num_states
                .checked_mul(d)
                .expect("state space too large to enumerate");
        }

        let mut position = vec![None; task.num_variables()];
        for (i, &v) in vars.iter().enumerate() {
            position[v] = Some(i);
        }

        Self {
            vars,
            multipliers,
            domains,
            position,
            num_states,
        }
    }

    /// The rank of the values `state` assigns to the ranked variables.
    pub fn rank(&self, state: &State) -> usize {
        self.vars
            .iter()
            .zip(&self.multipliers)
            .map(|(&v, &m)| state.0[v] as usize * m)
            .sum()
    }

    /// Writes the values of rank `index` into `state`.
    pub fn unrank(&self, index: usize, state: &mut State) {
        for (i, &v) in self.vars.iter().enumerate() {
            state.0[v] = self.value(index, i);
        }
    }

    /// The position of task variable `var` in [`Ranking::vars`], if ranked.
    pub fn position(&self, var: VarId) -> Option<usize> {
        self.position[var]
    }

    /// Value of the `i`-th ranked variable in rank `index`.
    pub fn value(&self, index: usize, i: usize) -> Value {
        let value = (index / self.multipliers[i]) % self.domains[i];

        Value::try_from(value).expect("a value below a u32 domain size fits in u32")
    }

    /// `index` with the `i`-th ranked variable set to `value`.
    pub fn set(&self, index: usize, i: usize, value: Value) -> usize {
        let old = self.value(index, i) as usize;

        index - old * self.multipliers[i] + value as usize * self.multipliers[i]
    }

    /// Whether the facts on ranked variables hold in rank `index`; other facts are ignored.
    pub fn holds(&self, index: usize, facts: &[Fact]) -> bool {
        facts.iter().all(|f| match self.position[f.var] {
            Some(i) => self.value(index, i) == f.value,
            None => true,
        })
    }

    /// All ranks in which `facts` hold (ignoring facts on unranked
    /// variables), without visiting the others.
    pub fn matching(&self, facts: &[Fact]) -> impl Iterator<Item = usize> + '_ {
        let mut fixed = vec![None; self.vars.len()];
        let mut consistent = true;
        for f in facts {
            if let Some(i) = self.position[f.var] {
                consistent &= fixed[i].is_none_or(|v| v == f.value);
                fixed[i] = Some(f.value);
            }
        }

        let base: usize = (0..self.vars.len())
            .filter_map(|i| Some(fixed[i]? as usize * self.multipliers[i]))
            .sum();
        let free: Vec<usize> = (0..self.vars.len())
            .filter(|&i| fixed[i].is_none())
            .collect();
        // Odometer over the free positions: amortized O(1) per rank.
        let mut digits = vec![0; free.len()];
        let mut index = base;
        let mut done = !consistent;
        core::iter::from_fn(move || {
            if done {
                return None;
            }

            let current = index;
            done = true;
            for (d, &i) in digits.iter_mut().zip(&free) {
                if *d + 1 < self.domains[i] {
                    *d += 1;
                    index += self.multipliers[i];
                    done = false;
                    break;
                }

                index -= *d * self.multipliers[i];
                *d = 0;
            }

            Some(current)
        })
    }
}
