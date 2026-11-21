//! The causal graph of a task ([`CausalGraph`]).

use alloc::vec;
use alloc::vec::Vec;

use crate::{Task, VarId};

/// The causal graph of a task: an arc `u → v` means that changing `v` may
/// depend on `u`, or that `u` and `v` change together.
///
/// As in Fast Downward, arcs run from every precondition and effect condition
/// variable of an operator to each variable it sets, between every two
/// variables one operator sets, and from every condition of an axiom to the
/// variable it derives. Self-loops are left out.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CausalGraph {
    /// Per variable, the variables it has an arc to; sorted, no duplicates.
    successors: Vec<Vec<VarId>>,
    /// Per variable, the variables with an arc to it; sorted, no duplicates.
    predecessors: Vec<Vec<VarId>>,
}

impl CausalGraph {
    /// Builds the causal graph of `task`.
    ///
    /// # Panics
    ///
    /// If a fact of `task` names a variable it does not have; a task that
    /// passes [`Task::validate`] never does.
    #[must_use]
    pub fn new(task: &Task) -> Self {
        let mut arcs = Vec::new();
        for op in &task.operators {
            for e in &op.eff {
                let v = e.fact.var;
                arcs.extend(op.pre.iter().map(|f| (f.var, v)));
                arcs.extend(e.conditions.iter().map(|f| (f.var, v)));
                arcs.extend(op.eff.iter().map(|other| (other.fact.var, v)));
            }
        }

        for a in &task.axioms {
            arcs.extend(a.conditions.iter().map(|f| (f.var, a.head.var)));
        }
        arcs.retain(|(u, v)| u != v);
        arcs.sort_unstable();
        arcs.dedup();

        let n = task.num_variables();
        let mut successors = vec![Vec::new(); n];
        let mut predecessors = vec![Vec::new(); n];
        for &(u, v) in &arcs {
            successors[u].push(v);
            predecessors[v].push(u);
        }

        Self {
            successors,
            predecessors,
        }
    }

    /// Variables with an arc from `var`, sorted.
    ///
    /// # Panics
    ///
    /// If `var` is not a variable of the task.
    #[must_use]
    pub fn successors(&self, var: VarId) -> &[VarId] {
        &self.successors[var]
    }

    /// Variables with an arc to `var`, sorted.
    ///
    /// # Panics
    ///
    /// If `var` is not a variable of the task.
    #[must_use]
    pub fn predecessors(&self, var: VarId) -> &[VarId] {
        &self.predecessors[var]
    }
}
