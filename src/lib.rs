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

/// Index of a variable in [`Task::variables`].
pub type VarId = usize;

/// A value of a variable, in `0..domain_size` ([`Variable::domain_size`]).
pub type Value = u32;

/// Index of an operator in [`Task::operators`]; also the label of its
/// transitions in a [`LabeledTransitionSystem`].
pub type OpId = usize;

/// A variable/value assignment `var = value`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Fact {
    /// The variable the fact assigns.
    pub var: VarId,
    /// The value it assigns, in `0..domain_size` of `var` in a valid task.
    pub value: Value,
}

impl Fact {
    /// The fact `var = value`.
    #[must_use]
    pub fn new(var: VarId, value: Value) -> Self {
        Self { var, value }
    }
}

/// A finite-domain variable of a [`Task`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Variable {
    /// A name for diagnostics, e.g. `var3` from the translator; not required
    /// to be unique.
    pub name: String,
    /// The number of values, `0..domain_size`. At least 1 in a valid task.
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

    /// Whether the variable is derived, i.e. set only by axioms.
    #[must_use]
    pub fn is_derived(&self) -> bool {
        self.axiom_layer.is_some()
    }
}

/// An effect `fact` that fires when all `conditions` hold in the state the
/// operator is applied in. Unconditional effects have no conditions.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Effect {
    /// Conditions on the state the operator is applied in; empty for an
    /// unconditional effect.
    pub conditions: Vec<Fact>,
    /// The assignment the effect makes when it fires.
    pub fact: Fact,
}

impl Effect {
    /// An unconditional effect.
    #[must_use]
    pub fn new(fact: Fact) -> Self {
        Self {
            conditions: Vec::new(),
            fact,
        }
    }
}

/// A ground action: applicable where its precondition holds, it applies its
/// effects at a cost.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Operator {
    /// The ground action name, e.g. `pick ball1 rooma left`.
    pub name: String,
    /// The precondition, a partial state; at most one value per variable in a
    /// valid task.
    pub pre: Vec<Fact>,
    /// Applied in order, so a later effect on the same variable wins.
    pub eff: Vec<Effect>,
    /// The cost of applying the operator, the label cost of its transitions.
    pub cost: u32,
}

/// A rule `conditions → head` that sets the derived variable `head.var`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Axiom {
    /// The facts that must all hold for the rule to fire.
    pub conditions: Vec<Fact>,
    /// The derived fact the rule sets; `head.var` is a derived variable.
    pub head: Fact,
}

/// A complete assignment: `state.0[var]` is the value of `var`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct State(pub Vec<Value>);

/// A SAS+ planning task with Fast Downward's extensions: conditional effects
/// and axioms (derived variables).
///
/// The fields are public and unchecked; [`Task::validate`] checks that they
/// are consistent.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Task {
    /// The variables, indexed by [`VarId`].
    pub variables: Vec<Variable>,
    /// The operators, indexed by [`OpId`].
    pub operators: Vec<Operator>,
    /// As in Fast Downward, the values of derived variables here are their
    /// defaults, which hold unless an axiom derives otherwise. Use
    /// [`Task::initial_state`] for the state with axioms evaluated.
    pub initial: State,
    /// The goal, a partial state.
    pub goal: Vec<Fact>,
    /// The axiom rules that compute the derived variables.
    pub axioms: Vec<Axiom>,
}

/// Why [`Task::validate`] rejected a task.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TaskError {
    /// The initial state does not have one value per variable.
    InitialStateArity,
    /// A fact assigns a value outside its variable's domain.
    ValueOutOfRange {
        /// The offending fact.
        fact: Fact,
    },
    /// A fact names a variable the task does not have.
    UnknownVariable {
        /// The offending fact.
        fact: Fact,
    },
    /// An operator has two unconditional effects on one variable.
    DuplicateEffect {
        /// The operator.
        op: OpId,
        /// The variable it sets twice.
        var: VarId,
    },
    /// An operator requires two values of one variable.
    ContradictoryPrecondition {
        /// The operator.
        op: OpId,
        /// The variable with two required values.
        var: VarId,
    },
    /// The goal requires two values of one variable.
    ContradictoryGoal {
        /// The variable with two required values.
        var: VarId,
    },
    /// An operator sets a derived variable.
    DerivedVariableEffect {
        /// The operator.
        op: OpId,
        /// The derived variable it sets.
        var: VarId,
    },
    /// An axiom sets a state variable.
    AxiomHeadNotDerived {
        /// Index of the axiom in [`Task::axioms`].
        axiom: usize,
    },
    /// An axiom depends on a derived variable of a higher layer, or on one of
    /// its own layer being at its default (negation needs a lower layer).
    AxiomLayer {
        /// Index of the axiom in [`Task::axioms`].
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
    /// The abstract transition system; its labels are the task's [`OpId`]s.
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
    /// Checks that the task is consistent, so the other methods can index
    /// it without panicking.
    ///
    /// # Errors
    ///
    /// Returns the first problem found, checking the initial state, the goal,
    /// the operators and then the axioms, in order:
    ///
    /// - [`TaskError::InitialStateArity`] if `initial` has the wrong length;
    /// - [`TaskError::UnknownVariable`] or [`TaskError::ValueOutOfRange`] for
    ///   a fact outside the variables or their domains;
    /// - [`TaskError::ContradictoryGoal`] and
    ///   [`TaskError::ContradictoryPrecondition`] for two required values of
    ///   one variable;
    /// - [`TaskError::DerivedVariableEffect`] for an operator that sets a
    ///   derived variable, and [`TaskError::DuplicateEffect`] for two
    ///   unconditional effects on one variable;
    /// - [`TaskError::AxiomHeadNotDerived`] and [`TaskError::AxiomLayer`] for
    ///   axioms that break the layering.
    pub fn validate(&self) -> Result<(), TaskError> {
        if self.initial.0.len() != self.variables.len() {
            return Err(TaskError::InitialStateArity);
        }

        for (var, &value) in self.initial.0.iter().enumerate() {
            self.check_fact(&Fact::new(var, value))?;
        }

        self.goal.iter().try_for_each(|f| self.check_fact(f))?;
        if let Some(var) = contradiction(&self.goal) {
            return Err(TaskError::ContradictoryGoal { var });
        }

        for (id, op) in self.operators.iter().enumerate() {
            self.validate_operator(id, op)?;
        }

        for (id, axiom) in self.axioms.iter().enumerate() {
            self.validate_axiom(id, axiom)?;
        }

        Ok(())
    }

    /// Checks that `fact` names an existing variable and a value in its domain.
    fn check_fact(&self, fact: &Fact) -> Result<(), TaskError> {
        let var = self
            .variables
            .get(fact.var)
            .ok_or(TaskError::UnknownVariable { fact: *fact })?;
        if fact.value >= var.domain_size {
            return Err(TaskError::ValueOutOfRange { fact: *fact });
        }

        Ok(())
    }

    /// The operator part of [`Task::validate`], for operator `id`.
    fn validate_operator(&self, id: OpId, op: &Operator) -> Result<(), TaskError> {
        op.pre.iter().try_for_each(|f| self.check_fact(f))?;
        if let Some(var) = contradiction(&op.pre) {
            return Err(TaskError::ContradictoryPrecondition { op: id, var });
        }

        for e in &op.eff {
            e.conditions.iter().try_for_each(|f| self.check_fact(f))?;
            self.check_fact(&e.fact)?;
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

        Ok(())
    }

    /// The axiom part of [`Task::validate`], for axiom `id`.
    fn validate_axiom(&self, id: usize, axiom: &Axiom) -> Result<(), TaskError> {
        axiom
            .conditions
            .iter()
            .try_for_each(|f| self.check_fact(f))?;
        self.check_fact(&axiom.head)?;
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

        Ok(())
    }

    /// The number of variables, state and derived.
    #[must_use]
    pub fn num_variables(&self) -> usize {
        self.variables.len()
    }

    /// Whether operator `op`'s precondition holds in `state`.
    ///
    /// # Panics
    ///
    /// If `op` is not an operator of the task, or `state` lacks a variable
    /// the precondition mentions.
    #[must_use]
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
    ///
    /// # Panics
    ///
    /// If `op` is not an operator of the task, or `state` has fewer values
    /// than the task has variables.
    #[must_use]
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
    ///
    /// # Panics
    ///
    /// If `state` has fewer values than the task has variables.
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
            while self.fire_layer(layer, state) {}
        }
    }

    /// One pass over the rules of axiom layer `layer`, setting the head of
    /// each rule that fires. Returns whether any variable changed.
    fn fire_layer(&self, layer: u32, state: &mut State) -> bool {
        let mut changed = false;
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

        changed
    }

    /// The initial state with derived variables evaluated.
    #[must_use]
    pub fn initial_state(&self) -> State {
        let mut state = self.initial.clone();
        self.evaluate_axioms(&mut state);
        state
    }

    /// The operators applicable in `state`, in id order.
    pub fn applicable_operators<'a>(&'a self, state: &'a State) -> impl Iterator<Item = OpId> + 'a {
        (0..self.operators.len()).filter(move |&op| self.is_applicable(op, state))
    }

    /// Whether the goal holds in `state`.
    ///
    /// # Panics
    ///
    /// If `state` lacks a variable the goal mentions.
    #[must_use]
    pub fn is_goal(&self, state: &State) -> bool {
        holds(&self.goal, state)
    }

    /// Operator costs indexed by operator id (the label cost function).
    #[must_use]
    pub fn costs(&self) -> Vec<u64> {
        self.operators.iter().map(|o| u64::from(o.cost)).collect()
    }

    /// Checks that `plan` is applicable from the initial state and reaches the goal.
    /// Returns the plan cost, or `None` if an operator is not applicable where
    /// the plan uses it or the plan does not end in a goal state.
    ///
    /// # Panics
    ///
    /// If `plan` names an operator the task does not have.
    #[must_use]
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
    ///
    /// The returned function gives `None` for states from which no goal is
    /// reachable.
    ///
    /// # Panics
    ///
    /// If the state space over the state variables has more than
    /// `usize::MAX` states. The returned function panics on states with
    /// fewer values than the task has variables.
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

/// Whether all `facts` hold in `state`.
fn holds(facts: &[Fact], state: &State) -> bool {
    facts.iter().all(|f| state.0[f.var] == f.value)
}
