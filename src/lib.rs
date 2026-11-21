//! Minimal SAS+ planning tasks, labeled transition systems, and the
//! [`Abstraction`] trait for abstraction heuristics.
//!
//! SAS+ (*Simplified Action Structures*, extended; Bäckström & Nebel 1995)
//! describes planning tasks with finite-domain state variables, operators
//! with preconditions and effects, and partial goals.
//!
//! Tasks may also use Fast Downward's two extensions of SAS+: conditional
//! effects and axioms (derived variables).
//!
//! Tasks come from Fast Downward's translator via `fdr::SasFile` (the
//! `output.sas` format, feature `fdr`), from your own grounder, or from
//! `random` for tests (feature `random`).
//!
//! ```mermaid
//! flowchart LR
//!     pddl[PDDL] -->|Fast Downward translator| sas[output.sas]
//!     sas -->|SasFile::parse| file[fdr::SasFile]
//!     file -->|to_task| task[Task]
//!     grounder[your grounder] --> task
//!     random[random::random_task] --> task
//!     task -->|Projection::new| proj[Projection]
//!     proj -->|Abstraction::lts| lts[LabeledTransitionSystem]
//!     lts -->|goal_distances| h[admissible estimates]
//!     task -->|perfect_heuristic| hstar[exact h*]
//! ```

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

mod causal_graph;
#[cfg(feature = "fdr")]
pub mod fdr;
mod lts;
mod projection;
#[cfg(feature = "random")]
pub mod random;
mod ranking;

/// Doctests for the code examples in README.md (not part of the API).
#[cfg(all(doctest, feature = "fdr"))]
#[doc = include_str!("../README.md")]
mod readme_doctests {}

pub use causal_graph::CausalGraph;
pub use lts::{LabeledTransitionSystem, Transition};
pub use projection::Projection;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

pub type VarId = usize;
pub type Value = u32;
pub type OpId = usize;

/// A variable/value assignment `var = value`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Fact {
    pub var: VarId,
    pub value: Value,
}

impl Fact {
    pub fn new(var: VarId, value: Value) -> Self {
        Self { var, value }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Variable {
    pub name: String,
    pub domain_size: u32,
    /// `None` for state variables, `Some(layer)` for derived variables, which
    /// only axioms set. Axioms of lower layers are evaluated first.
    pub axiom_layer: Option<u32>,
}

impl Variable {
    /// A state variable (not derived).
    pub fn new(name: impl Into<String>, domain_size: u32) -> Self {
        Self {
            name: name.into(),
            domain_size,
            axiom_layer: None,
        }
    }

    pub fn is_derived(&self) -> bool {
        self.axiom_layer.is_some()
    }
}

/// An effect `fact` that fires when all `conditions` hold in the state the
/// operator is applied in. Unconditional effects have no conditions.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Effect {
    pub conditions: Vec<Fact>,
    pub fact: Fact,
}

impl Effect {
    /// An unconditional effect.
    pub fn new(fact: Fact) -> Self {
        Self {
            conditions: Vec::new(),
            fact,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Operator {
    pub name: String,
    pub pre: Vec<Fact>,
    /// Applied in order, so a later effect on the same variable wins.
    pub eff: Vec<Effect>,
    pub cost: u32,
}

/// A rule `conditions → head` that sets the derived variable `head.var`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Axiom {
    pub conditions: Vec<Fact>,
    pub head: Fact,
}

/// A complete assignment: `state.0[var]` is the value of `var`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct State(pub Vec<Value>);

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Task {
    pub variables: Vec<Variable>,
    pub operators: Vec<Operator>,
    /// As in Fast Downward, the values of derived variables here are their
    /// defaults, which hold unless an axiom derives otherwise. Use
    /// [`Task::initial_state`] for the state with axioms evaluated.
    pub initial: State,
    pub goal: Vec<Fact>,
    pub axioms: Vec<Axiom>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TaskError {
    InitialStateArity,
    ValueOutOfRange {
        fact: Fact,
    },
    UnknownVariable {
        fact: Fact,
    },
    DuplicateEffect {
        op: OpId,
        var: VarId,
    },
    /// An operator requires two values of one variable.
    ContradictoryPrecondition {
        op: OpId,
        var: VarId,
    },
    /// The goal requires two values of one variable.
    ContradictoryGoal {
        var: VarId,
    },
    /// An operator sets a derived variable.
    DerivedVariableEffect {
        op: OpId,
        var: VarId,
    },
    /// An axiom sets a state variable.
    AxiomHeadNotDerived {
        axiom: usize,
    },
    /// An axiom depends on a derived variable of a higher layer, or on one of
    /// its own layer being at its default (negation needs a lower layer).
    AxiomLayer {
        axiom: usize,
    },
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InitialStateArity => write!(f, "initial state has the wrong number of variables"),
            Self::ValueOutOfRange { fact } => write!(f, "value out of range in {fact:?}"),
            Self::UnknownVariable { fact } => write!(f, "unknown variable in {fact:?}"),
            Self::DuplicateEffect { op, var } => {
                write!(f, "operator {op} has two effects on variable {var}")
            }
            Self::ContradictoryPrecondition { op, var } => {
                write!(f, "operator {op} requires two values of variable {var}")
            }
            Self::ContradictoryGoal { var } => {
                write!(f, "the goal requires two values of variable {var}")
            }
            Self::DerivedVariableEffect { op, var } => {
                write!(f, "operator {op} sets derived variable {var}")
            }
            Self::AxiomHeadNotDerived { axiom } => {
                write!(f, "axiom {axiom} sets a state variable")
            }
            Self::AxiomLayer { axiom } => {
                write!(
                    f,
                    "axiom {axiom} depends on a higher axiom layer or negates its own layer"
                )
            }
        }
    }
}

impl core::error::Error for TaskError {}

/// Something that maps concrete states onto an abstract transition system
/// whose labels are operator ids. Goal distances in the abstract system are
/// admissible estimates for the concrete task.
pub trait Abstraction {
    fn lts(&self) -> &LabeledTransitionSystem;
    /// `None` means the state was pruned (provably a dead end or unreachable).
    fn abstract_state(&self, state: &State) -> Option<usize>;
}

impl<A: Abstraction + ?Sized> Abstraction for &A {
    fn lts(&self) -> &LabeledTransitionSystem {
        (**self).lts()
    }
    fn abstract_state(&self, state: &State) -> Option<usize> {
        (**self).abstract_state(state)
    }
}

impl Task {
    pub fn validate(&self) -> Result<(), TaskError> {
        if self.initial.0.len() != self.variables.len() {
            return Err(TaskError::InitialStateArity);
        }
        let check = |fact: &Fact| -> Result<(), TaskError> {
            let var = self
                .variables
                .get(fact.var)
                .ok_or(TaskError::UnknownVariable { fact: *fact })?;
            if fact.value >= var.domain_size {
                return Err(TaskError::ValueOutOfRange { fact: *fact });
            }
            Ok(())
        };
        for (var, &value) in self.initial.0.iter().enumerate() {
            check(&Fact::new(var, value))?;
        }
        self.goal.iter().try_for_each(check)?;
        if let Some(var) = contradiction(&self.goal) {
            return Err(TaskError::ContradictoryGoal { var });
        }
        for (id, op) in self.operators.iter().enumerate() {
            op.pre.iter().try_for_each(check)?;
            if let Some(var) = contradiction(&op.pre) {
                return Err(TaskError::ContradictoryPrecondition { op: id, var });
            }
            for e in &op.eff {
                e.conditions.iter().try_for_each(check)?;
                check(&e.fact)?;
                if self.variables[e.fact.var].is_derived() {
                    return Err(TaskError::DerivedVariableEffect {
                        op: id,
                        var: e.fact.var,
                    });
                }
            }
            // Conditional effects on one variable may coexist; the last firing one wins.
            let mut vars: Vec<_> = op
                .eff
                .iter()
                .filter(|e| e.conditions.is_empty())
                .map(|e| e.fact.var)
                .collect();
            vars.sort_unstable();
            if let Some(w) = vars.windows(2).find(|w| w[0] == w[1]) {
                return Err(TaskError::DuplicateEffect { op: id, var: w[0] });
            }
        }
        for (id, axiom) in self.axioms.iter().enumerate() {
            axiom.conditions.iter().try_for_each(check)?;
            check(&axiom.head)?;
            let Some(layer) = self.variables[axiom.head.var].axiom_layer else {
                return Err(TaskError::AxiomHeadNotDerived { axiom: id });
            };
            if axiom.conditions.iter().any(|c| {
                self.variables[c.var]
                    .axiom_layer
                    .is_some_and(|l| l > layer || (l == layer && c.value == self.initial.0[c.var]))
            }) {
                return Err(TaskError::AxiomLayer { axiom: id });
            }
        }
        Ok(())
    }

    pub fn num_variables(&self) -> usize {
        self.variables.len()
    }

    pub fn is_applicable(&self, op: OpId, state: &State) -> bool {
        holds(&self.operators[op].pre, state)
    }

    /// The successor state. Effect conditions are evaluated in `state`;
    /// derived variables are then re-evaluated.
    ///
    /// Like [`Task::is_applicable`] and [`Task::is_goal`], this reads derived
    /// variables from `state` as given, so `state` must have them evaluated:
    /// start from [`Task::initial_state`] (not [`Task::initial`]) or call
    /// [`Task::evaluate_axioms`] first. States returned by `apply` are.
    pub fn apply(&self, op: OpId, state: &State) -> State {
        let mut next = state.clone();
        for e in &self.operators[op].eff {
            if holds(&e.conditions, state) {
                next.0[e.fact.var] = e.fact.value;
            }
        }
        self.evaluate_axioms(&mut next);
        next
    }

    /// Recomputes all derived variables of `state` from its state variables:
    /// each derived variable is reset to its default (its value in
    /// [`Task::initial`]), then the axioms fire layer by layer until nothing
    /// changes, as in Fast Downward. A variable keeps the first non-default
    /// value derived for it.
    ///
    /// This allocates nothing but rescans the rules of a layer until none
    /// fires; search code that needs speed on axiom-heavy tasks should index
    /// the rules itself.
    pub fn evaluate_axioms(&self, state: &mut State) {
        // Reset first: a derived variable without rules still holds its default.
        for (var, v) in self.variables.iter().enumerate() {
            if v.is_derived() {
                state.0[var] = self.initial.0[var];
            }
        }
        if self.axioms.is_empty() {
            return;
        }
        let mut done: Option<u32> = None;
        // Layers in ascending order, without collecting them.
        while let Some(layer) = self
            .variables
            .iter()
            .filter_map(|v| v.axiom_layer)
            .filter(|&l| done.is_none_or(|d| l > d))
            .min()
        {
            done = Some(layer);
            let mut changed = true;
            while changed {
                changed = false;
                for a in &self.axioms {
                    if self.variables[a.head.var].axiom_layer != Some(layer) {
                        continue;
                    }
                    // Only a variable still at its default is derived, so the fixpoint is reached.
                    let default = self.initial.0[a.head.var];
                    if state.0[a.head.var] == default
                        && a.head.value != default
                        && holds(&a.conditions, state)
                    {
                        state.0[a.head.var] = a.head.value;
                        changed = true;
                    }
                }
            }
        }
    }

    /// The initial state with derived variables evaluated.
    pub fn initial_state(&self) -> State {
        let mut state = self.initial.clone();
        self.evaluate_axioms(&mut state);
        state
    }

    pub fn applicable_operators<'a>(&'a self, state: &'a State) -> impl Iterator<Item = OpId> + 'a {
        (0..self.operators.len()).filter(move |&op| self.is_applicable(op, state))
    }

    pub fn is_goal(&self, state: &State) -> bool {
        holds(&self.goal, state)
    }

    /// Operator costs indexed by operator id (the label cost function).
    pub fn costs(&self) -> Vec<u64> {
        self.operators.iter().map(|o| u64::from(o.cost)).collect()
    }

    /// Checks that `plan` is applicable from the initial state and reaches the goal.
    /// Returns the plan cost.
    pub fn validate_plan(&self, plan: &[OpId]) -> Option<u64> {
        let mut state = self.initial_state();
        let mut cost = 0;
        for &op in plan {
            if !self.is_applicable(op, &state) {
                return None;
            }
            state = self.apply(op, &state);
            cost += u64::from(self.operators[op].cost);
        }
        self.is_goal(&state).then_some(cost)
    }

    /// Optimal plan cost from `state` (with derived variables evaluated),
    /// by building the full state space over all state variables.
    /// Exponential; intended for tests on tiny tasks.
    pub fn perfect_heuristic(&self) -> impl Fn(&State) -> Option<u64> + '_ {
        let vars = (0..self.num_variables())
            .filter(|&v| !self.variables[v].is_derived())
            .collect();
        let ranking = ranking::Ranking::new(self, vars);
        let mut state = self.initial.clone();
        let mut transitions = Vec::new();
        for (label, op) in self.operators.iter().enumerate() {
            // Preconditions on state variables pick the candidates; derived ones are checked after.
            for src in ranking.matching(&op.pre) {
                ranking.unrank(src, &mut state);
                self.evaluate_axioms(&mut state);
                if self.is_applicable(label, &state) {
                    let dst = ranking.rank(&self.apply(label, &state));
                    transitions.push(Transition { src, label, dst });
                }
            }
        }
        let mut goal = vec![false; ranking.num_states];
        for index in ranking.matching(&self.goal) {
            ranking.unrank(index, &mut state);
            self.evaluate_axioms(&mut state);
            goal[index] = self.is_goal(&state);
        }
        let lts = LabeledTransitionSystem {
            num_states: ranking.num_states,
            num_labels: self.operators.len(),
            transitions,
            goal,
            initial: Some(ranking.rank(&self.initial)),
        };
        let h = lts.goal_distances(&self.costs());
        move |s| h[ranking.rank(s)]
    }
}

/// A variable that `facts` assign two different values, if any.
fn contradiction(facts: &[Fact]) -> Option<VarId> {
    let mut sorted = facts.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    sorted
        .windows(2)
        .find(|w| w[0].var == w[1].var)
        .map(|w| w[0].var)
}

fn holds(facts: &[Fact], state: &State) -> bool {
    facts.iter().all(|f| state.0[f.var] == f.value)
}
