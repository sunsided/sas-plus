//! Minimal SAS+ planning tasks, labeled transition systems, and the
//! [`Abstraction`] trait shared by `merge-and-shrink` and `cost-partition`.
//!
//! SAS+ (*Simplified Action Structures*, extended; Bäckström & Nebel 1995)
//! describes planning tasks with finite-domain state variables, operators
//! with preconditions and effects, and partial goals.

mod lts;

pub use lts::{LabeledTransitionSystem, Transition};

use std::fmt;

pub type VarId = usize;
pub type Value = u32;
pub type OpId = usize;

/// A variable/value assignment `var = value`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Fact {
    pub var: VarId,
    pub value: Value,
}

impl Fact {
    pub fn new(var: VarId, value: Value) -> Self {
        Self { var, value }
    }
}

#[derive(Clone, Debug)]
pub struct Variable {
    pub name: String,
    pub domain_size: u32,
}

#[derive(Clone, Debug)]
pub struct Operator {
    pub name: String,
    pub pre: Vec<Fact>,
    pub eff: Vec<Fact>,
    pub cost: u32,
}

/// A complete assignment: `state.0[var]` is the value of `var`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct State(pub Vec<Value>);

#[derive(Clone, Debug)]
pub struct Task {
    pub variables: Vec<Variable>,
    pub operators: Vec<Operator>,
    pub initial: State,
    pub goal: Vec<Fact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskError {
    InitialStateArity,
    ValueOutOfRange { fact: Fact },
    UnknownVariable { fact: Fact },
    DuplicateEffect { op: OpId, var: VarId },
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
        }
    }
}

impl std::error::Error for TaskError {}

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
        for (id, op) in self.operators.iter().enumerate() {
            op.pre.iter().try_for_each(check)?;
            op.eff.iter().try_for_each(check)?;
            let mut vars: Vec<_> = op.eff.iter().map(|f| f.var).collect();
            vars.sort_unstable();
            if let Some(w) = vars.windows(2).find(|w| w[0] == w[1]) {
                return Err(TaskError::DuplicateEffect { op: id, var: w[0] });
            }
        }
        Ok(())
    }

    pub fn num_variables(&self) -> usize {
        self.variables.len()
    }

    pub fn is_applicable(&self, op: OpId, state: &State) -> bool {
        self.operators[op]
            .pre
            .iter()
            .all(|f| state.0[f.var] == f.value)
    }

    pub fn apply(&self, op: OpId, state: &State) -> State {
        let mut next = state.clone();
        for f in &self.operators[op].eff {
            next.0[f.var] = f.value;
        }
        next
    }

    pub fn applicable_operators<'a>(&'a self, state: &'a State) -> impl Iterator<Item = OpId> + 'a {
        (0..self.operators.len()).filter(move |&op| self.is_applicable(op, state))
    }

    pub fn is_goal(&self, state: &State) -> bool {
        self.goal.iter().all(|f| state.0[f.var] == f.value)
    }

    /// Operator costs indexed by operator id (the label cost function).
    pub fn costs(&self) -> Vec<u64> {
        self.operators.iter().map(|o| u64::from(o.cost)).collect()
    }

    /// Checks that `plan` is applicable from the initial state and reaches the goal.
    /// Returns the plan cost.
    pub fn validate_plan(&self, plan: &[OpId]) -> Option<u64> {
        let mut state = self.initial.clone();
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
}
