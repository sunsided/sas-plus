//! Reader and writer for Fast Downward's translator output format
//! (`output.sas`, file version 3).
//!
//! Fast Downward's translator grounds PDDL into a finite-domain
//! representation (FDR / SAS+). [`SasFile`] mirrors that file faithfully,
//! including mutex groups, conditional effects and axioms. It writes back
//! byte-for-byte what it reads, and [`SasFile::to_task`] converts it to a
//! [`Task`].
//!
//! ```no_run
//! let text = std::fs::read_to_string("output.sas")?;
//! let file: sas_plus::fdr::SasFile = text.parse()?;
//! let task = file.to_task();
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::{self, Write as _};
use core::str::FromStr;

use crate::{Axiom, Effect, Fact, OpId, Operator, State, Task, Value, VarId, Variable};

/// The only file version this module understands (Fast Downward ≥ 20.06).
pub const SAS_FILE_VERSION: u32 = 3;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SasFile {
    /// `false` means unit costs: every operator costs 1 whatever its cost field says.
    pub use_metric: bool,
    pub variables: Vec<SasVariable>,
    pub mutex_groups: Vec<Vec<Fact>>,
    pub initial: Vec<Value>,
    pub goal: Vec<Fact>,
    pub operators: Vec<SasOperator>,
    pub axioms: Vec<SasAxiom>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SasVariable {
    pub name: String,
    /// `None` for ordinary state variables, `Some(layer)` for derived variables.
    pub axiom_layer: Option<u32>,
    /// One name per value, e.g. `Atom at(ball1, rooma)` or `<none of those>`.
    pub values: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SasOperator {
    /// Ground action name without parentheses, e.g. `pick ball1 rooma left`.
    pub name: String,
    /// Preconditions on variables the operator does not change.
    pub prevail: Vec<Fact>,
    pub effects: Vec<SasEffect>,
    /// The cost field as written; see [`SasFile::operator_cost`].
    pub cost: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SasEffect {
    /// Effect conditions; non-empty only for conditional effects.
    pub conditions: Vec<Fact>,
    pub var: VarId,
    /// Required value before the effect (`None` = no precondition, written `-1`).
    pub pre: Option<Value>,
    pub post: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SasAxiom {
    pub conditions: Vec<Fact>,
    pub var: VarId,
    /// Must equal the derived variable's value in [`SasFile::initial`] (its
    /// default); the reader rejects files where it does not, and
    /// [`SasFile::to_task`] drops it.
    pub pre: Value,
    pub post: Value,
}

/// A syntax or consistency error, with the 1-based line it was found on.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "output.sas line {}: {}", self.line, self.message)
    }
}

impl core::error::Error for ParseError {}

impl SasFile {
    pub fn parse(text: &str) -> Result<Self, ParseError> {
        Parser::new(text).file()
    }

    /// The cost search should use for operator `op`.
    pub fn operator_cost(&self, op: OpId) -> u32 {
        if self.use_metric {
            self.operators[op].cost
        } else {
            1
        }
    }

    /// The name of a fact, e.g. `Atom at(ball1, rooma)`.
    pub fn fact_name(&self, fact: Fact) -> &str {
        &self.variables[fact.var].values[fact.value as usize]
    }

    /// Converts to a [`Task`]. Prevail conditions and effect preconditions
    /// become operator preconditions; costs follow the metric. Mutex groups
    /// and fact names are not part of a task and stay here.
    pub fn to_task(&self) -> Task {
        let operators = self
            .operators
            .iter()
            .enumerate()
            .map(|(id, op)| {
                let mut pre = op.prevail.clone();
                pre.extend(
                    op.effects
                        .iter()
                        .filter_map(|e| Some(Fact::new(e.var, e.pre?))),
                );
                pre.sort_unstable();
                pre.dedup();
                Operator {
                    name: op.name.clone(),
                    pre,
                    eff: op
                        .effects
                        .iter()
                        .map(|e| Effect {
                            conditions: e.conditions.clone(),
                            fact: Fact::new(e.var, e.post),
                        })
                        .collect(),
                    cost: self.operator_cost(id),
                }
            })
            .collect();
        Task {
            variables: self
                .variables
                .iter()
                .map(|v| Variable {
                    name: v.name.clone(),
                    domain_size: v.values.len() as u32,
                    axiom_layer: v.axiom_layer,
                })
                .collect(),
            operators,
            initial: State(self.initial.clone()),
            goal: self.goal.clone(),
            // A rule's `pre` value is the derived variable's default, which the task keeps in `initial`.
            axioms: self
                .axioms
                .iter()
                .map(|a| Axiom {
                    conditions: a.conditions.clone(),
                    head: Fact::new(a.var, a.post),
                })
                .collect(),
        }
    }

    /// Formats a plan the way Fast Downward writes `sas_plan`, for use with
    /// plan validators such as VAL.
    pub fn format_plan(&self, plan: &[OpId]) -> String {
        let mut out = String::new();
        for &op in plan {
            let _ = writeln!(out, "({})", self.operators[op].name);
        }
        let cost: u64 = plan
            .iter()
            .map(|&op| u64::from(self.operator_cost(op)))
            .sum();
        // Fast Downward labels by the costs themselves, not by the metric flag.
        let unit = (0..self.operators.len()).all(|op| self.operator_cost(op) == 1);
        let kind = if unit { "unit cost" } else { "general cost" };
        let _ = writeln!(out, "; cost = {cost} ({kind})");
        out
    }
}

impl FromStr for SasFile {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, ParseError> {
        Self::parse(s)
    }
}

/// Writes the file exactly as Fast Downward's translator does.
impl fmt::Display for SasFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "begin_version\n{SAS_FILE_VERSION}\nend_version")?;
        writeln!(f, "begin_metric\n{}\nend_metric", u8::from(self.use_metric))?;
        writeln!(f, "{}", self.variables.len())?;
        for v in &self.variables {
            let layer = v.axiom_layer.map_or(-1, i64::from);
            writeln!(f, "begin_variable\n{}\n{layer}\n{}", v.name, v.values.len())?;
            for value in &v.values {
                writeln!(f, "{value}")?;
            }
            writeln!(f, "end_variable")?;
        }
        writeln!(f, "{}", self.mutex_groups.len())?;
        for group in &self.mutex_groups {
            writeln!(f, "begin_mutex_group\n{}", group.len())?;
            write_facts(f, group)?;
            writeln!(f, "end_mutex_group")?;
        }
        writeln!(f, "begin_state")?;
        for v in &self.initial {
            writeln!(f, "{v}")?;
        }
        writeln!(f, "end_state\nbegin_goal\n{}", self.goal.len())?;
        write_facts(f, &self.goal)?;
        writeln!(f, "end_goal\n{}", self.operators.len())?;
        for op in &self.operators {
            writeln!(f, "begin_operator\n{}\n{}", op.name, op.prevail.len())?;
            write_facts(f, &op.prevail)?;
            writeln!(f, "{}", op.effects.len())?;
            for e in &op.effects {
                write!(f, "{} ", e.conditions.len())?;
                for c in &e.conditions {
                    write!(f, "{} {} ", c.var, c.value)?;
                }
                let pre = e.pre.map_or(-1, i64::from);
                writeln!(f, "{} {pre} {}", e.var, e.post)?;
            }
            writeln!(f, "{}\nend_operator", op.cost)?;
        }
        writeln!(f, "{}", self.axioms.len())?;
        for a in &self.axioms {
            writeln!(f, "begin_rule\n{}", a.conditions.len())?;
            write_facts(f, &a.conditions)?;
            writeln!(f, "{} {} {}\nend_rule", a.var, a.pre, a.post)?;
        }
        Ok(())
    }
}

fn write_facts(f: &mut fmt::Formatter<'_>, facts: &[Fact]) -> fmt::Result {
    facts
        .iter()
        .try_for_each(|x| writeln!(f, "{} {}", x.var, x.value))
}

struct Parser<'a> {
    lines: core::str::Lines<'a>,
    line: usize,
    /// Domain sizes of the variables read so far, for range checks.
    domains: Vec<u32>,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            lines: text.lines(),
            line: 0,
            domains: Vec::new(),
        }
    }

    fn error<T>(&self, message: impl Into<String>) -> Result<T, ParseError> {
        Err(ParseError {
            line: self.line,
            message: message.into(),
        })
    }

    fn next(&mut self) -> Result<&'a str, ParseError> {
        self.line += 1;
        match self.lines.next() {
            Some(l) => Ok(l.trim_end_matches('\r')),
            None => self.error("unexpected end of file"),
        }
    }

    fn expect(&mut self, keyword: &str) -> Result<(), ParseError> {
        let l = self.next()?;
        if l.trim() == keyword {
            Ok(())
        } else {
            self.error(format!("expected `{keyword}`, found `{l}`"))
        }
    }

    fn numbers(&mut self) -> Result<Vec<i64>, ParseError> {
        let l = self.next()?;
        l.split_whitespace()
            .map(|t| t.parse::<i64>())
            .collect::<Result<_, _>>()
            .or_else(|_| self.error(format!("expected integers, found `{l}`")))
    }

    fn number<T: TryFrom<i64>>(&mut self, what: &str) -> Result<T, ParseError> {
        match self.numbers()?.as_slice() {
            &[n] => T::try_from(n).or_else(|_| self.error(format!("{what} out of range: {n}"))),
            _ => self.error(format!("expected a single integer ({what})")),
        }
    }

    fn var(&self, n: i64) -> Result<VarId, ParseError> {
        match usize::try_from(n) {
            Ok(v) if v < self.domains.len() => Ok(v),
            _ => self.error(format!("variable {n} does not exist")),
        }
    }

    fn value(&self, var: VarId, n: i64) -> Result<Value, ParseError> {
        match u32::try_from(n) {
            Ok(v) if v < self.domains[var] => Ok(v),
            _ => self.error(format!("value {n} out of range for variable {var}")),
        }
    }

    fn fact_from(&self, var: i64, value: i64) -> Result<Fact, ParseError> {
        let var = self.var(var)?;
        Ok(Fact::new(var, self.value(var, value)?))
    }

    fn fact(&mut self) -> Result<Fact, ParseError> {
        match self.numbers()?.as_slice() {
            &[var, value] => self.fact_from(var, value),
            _ => self.error("expected `<var> <value>`"),
        }
    }

    fn facts(&mut self, what: &str) -> Result<Vec<Fact>, ParseError> {
        let n: usize = self.number(what)?;
        (0..n).map(|_| self.fact()).collect()
    }

    fn file(mut self) -> Result<SasFile, ParseError> {
        self.expect("begin_version")?;
        let version: u32 = self.number("version")?;
        if version != SAS_FILE_VERSION {
            return self.error(format!(
                "unsupported file version {version} (expected {SAS_FILE_VERSION})"
            ));
        }
        self.expect("end_version")?;

        self.expect("begin_metric")?;
        let use_metric = match self.number::<i64>("metric")? {
            0 => false,
            1 => true,
            m => return self.error(format!("metric must be 0 or 1, found {m}")),
        };
        self.expect("end_metric")?;

        let num_vars: usize = self.number("number of variables")?;
        let variables = (0..num_vars)
            .map(|_| self.variable())
            .collect::<Result<Vec<_>, _>>()?;

        let num_mutex: usize = self.number("number of mutex groups")?;
        let mutex_groups = (0..num_mutex)
            .map(|_| {
                self.expect("begin_mutex_group")?;
                let facts = self.facts("mutex group size")?;
                self.expect("end_mutex_group")?;
                Ok(facts)
            })
            .collect::<Result<_, ParseError>>()?;

        self.expect("begin_state")?;
        let initial: Vec<Value> = (0..num_vars)
            .map(|var| {
                let n = self.number::<i64>("initial value")?;
                self.value(var, n)
            })
            .collect::<Result<_, _>>()?;
        self.expect("end_state")?;

        self.expect("begin_goal")?;
        let goal = self.facts("number of goal facts")?;
        self.expect("end_goal")?;

        let num_ops: usize = self.number("number of operators")?;
        let operators = (0..num_ops)
            .map(|_| self.operator())
            .collect::<Result<_, _>>()?;

        let num_axioms: usize = self.number("number of axioms")?;
        let axioms = (0..num_axioms)
            .map(|_| self.axiom(&variables, &initial))
            .collect::<Result<_, _>>()?;

        while let Some(extra) = self.lines.next() {
            self.line += 1;
            if !extra.trim().is_empty() {
                return self.error(format!(
                    "unexpected content after the last axiom: `{extra}`"
                ));
            }
        }
        Ok(SasFile {
            use_metric,
            variables,
            mutex_groups,
            initial,
            goal,
            operators,
            axioms,
        })
    }

    fn variable(&mut self) -> Result<SasVariable, ParseError> {
        self.expect("begin_variable")?;
        let name = self.next()?.to_string();
        let axiom_layer = match self.number::<i64>("axiom layer")? {
            -1 => None,
            l => {
                Some(u32::try_from(l).or_else(|_| self.error(format!("invalid axiom layer {l}")))?)
            }
        };
        let size: u32 = self.number("domain size")?;
        if size == 0 {
            return self.error("empty variable domain");
        }
        let values = (0..size)
            .map(|_| self.next().map(str::to_string))
            .collect::<Result<_, _>>()?;
        self.expect("end_variable")?;
        self.domains.push(size);
        Ok(SasVariable {
            name,
            axiom_layer,
            values,
        })
    }

    fn operator(&mut self) -> Result<SasOperator, ParseError> {
        self.expect("begin_operator")?;
        let name = self.next()?.to_string();
        let prevail = self.facts("number of prevail conditions")?;
        let num_effects: usize = self.number("number of effects")?;
        let effects = (0..num_effects)
            .map(|_| self.effect())
            .collect::<Result<_, _>>()?;
        let cost: u32 = self.number("operator cost")?;
        self.expect("end_operator")?;
        Ok(SasOperator {
            name,
            prevail,
            effects,
            cost,
        })
    }

    /// `<#conds> [<var> <val>]* <var> <pre> <post>` on one line.
    fn effect(&mut self) -> Result<SasEffect, ParseError> {
        let nums = self.numbers()?;
        let Some((&count, rest)) = nums.split_first() else {
            return self.error("empty effect line");
        };
        let count = usize::try_from(count).or_else(|_| self.error("negative condition count"))?;
        if rest.len() != 2 * count + 3 {
            return self.error(format!(
                "effect with {count} conditions needs {} integers",
                2 * count + 4
            ));
        }
        let conditions = rest[..2 * count]
            .chunks(2)
            .map(|c| self.fact_from(c[0], c[1]))
            .collect::<Result<_, _>>()?;
        let var = self.var(rest[2 * count])?;
        let pre = match rest[2 * count + 1] {
            -1 => None,
            p => Some(self.value(var, p)?),
        };
        let post = self.value(var, rest[2 * count + 2])?;
        Ok(SasEffect {
            conditions,
            var,
            pre,
            post,
        })
    }

    fn axiom(
        &mut self,
        variables: &[SasVariable],
        initial: &[Value],
    ) -> Result<SasAxiom, ParseError> {
        self.expect("begin_rule")?;
        let conditions = self.facts("number of axiom conditions")?;
        let (var, pre, post) = match self.numbers()?.as_slice() {
            &[var, pre, post] => {
                let var = self.var(var)?;
                (var, self.value(var, pre)?, self.value(var, post)?)
            }
            _ => return self.error("expected `<var> <pre> <post>`"),
        };
        if variables[var].axiom_layer.is_none() {
            return self.error(format!("axiom sets state variable {var}"));
        }
        // `to_task` relies on this: the initial state holds the default.
        if pre != initial[var] {
            return self.error(format!(
                "axiom pre value {pre} differs from the default {} of variable {var}",
                initial[var]
            ));
        }
        self.expect("end_rule")?;
        Ok(SasAxiom {
            conditions,
            var,
            pre,
            post,
        })
    }
}
