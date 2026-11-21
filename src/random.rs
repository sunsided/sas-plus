//! Tiny deterministic random task generator for property tests across the
//! workspace (no external RNG dependency).
//!
//! A seed always produces the same task, so a failing property names the seed
//! that reproduces it.

use alloc::format;
use alloc::vec::Vec;

use crate::{Axiom, Effect, Fact, Operator, State, Task, Variable};

/// xorshift64* — good enough for test-case generation.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    /// A generator for `seed`; every seed, including 0, gives a usable stream.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    /// The next 64 pseudo-random bits.
    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in `lo..=hi` (up to a negligible modulo bias).
    ///
    /// # Panics
    ///
    /// If `lo > hi`.
    pub fn range(&mut self, lo: u64, hi: u64) -> u64 {
        assert!(lo <= hi, "empty range {lo}..={hi}");
        let x = self.next_u64();
        // The full `u64` range has no modulus that fits.
        (hi - lo).checked_add(1).map_or(x, |n| lo + x % n)
    }

    /// [`Rng::range`] for `u32` bounds; draws exactly as `range` does.
    fn range_u32(&mut self, lo: u32, hi: u32) -> u32 {
        let x = self.range(u64::from(lo), u64::from(hi));

        u32::try_from(x).expect("a draw in u32 bounds fits in u32")
    }

    /// [`Rng::range`] for `usize` bounds; draws exactly as `range` does.
    fn range_usize(&mut self, lo: usize, hi: usize) -> usize {
        let x = self.range(lo as u64, hi as u64);

        usize::try_from(x).expect("a draw in usize bounds fits in usize")
    }

    /// `min(k, n)` distinct variables out of `0..n`, by a partial
    /// Fisher-Yates shuffle.
    fn distinct_vars(&mut self, n: usize, k: usize) -> Vec<usize> {
        let mut vars: Vec<usize> = (0..n).collect();
        for i in 0..k.min(n) {
            let j = self.range_usize(i, n - 1);
            vars.swap(i, j);
        }

        vars.truncate(k.min(n));
        vars
    }

    /// A fact on `var` with a uniform value of its domain in `variables`.
    fn fact(&mut self, variables: &[Variable], var: usize) -> Fact {
        Fact::new(var, self.range_u32(0, variables[var].domain_size - 1))
    }

    /// Facts on up to `k` distinct variables out of the first `n`, with
    /// uniform values.
    fn facts(&mut self, variables: &[Variable], n: usize, k: usize) -> Vec<Fact> {
        self.distinct_vars(n, k)
            .into_iter()
            .map(|v| self.fact(variables, v))
            .collect()
    }
}

/// The shape of the tasks [`random_task`] generates.
#[derive(Clone, Copy, Debug)]
pub struct RandomTaskParams {
    /// Number of state variables.
    pub num_vars: usize,
    /// Largest domain size; each variable gets 2 to `max(max_domain, 2)` values.
    pub max_domain: u32,
    /// Number of operators, each with 0 to 2 preconditions and 1 or 2 effects.
    pub num_ops: usize,
    /// Largest operator cost; costs are 1 to `max(max_cost, 1)`.
    pub max_cost: u32,
    /// Up to this many conditions per effect; `0` means no conditional effects.
    pub max_conditions: usize,
    /// Number of binary derived variables, each in its own axiom layer, with
    /// one or two rules apiece. Goals and preconditions may then mention them.
    pub num_derived: usize,
}

impl Default for RandomTaskParams {
    fn default() -> Self {
        Self {
            num_vars: 4,
            max_domain: 3,
            num_ops: 10,
            max_cost: 5,
            max_conditions: 0,
            num_derived: 0,
        }
    }
}

/// A random task of shape `p`, the same for every call with the same `seed`.
/// The task passes [`Task::validate`].
#[must_use]
pub fn random_task(seed: u64, p: RandomTaskParams) -> Task {
    let mut rng = Rng::new(seed);
    let variables: Vec<Variable> = (0..p.num_vars)
        .map(|i| Variable::new(format!("v{i}"), rng.range_u32(2, p.max_domain.max(2))))
        .collect();
    let operators = (0..p.num_ops)
        .map(|i| random_operator(&mut rng, &p, &variables, i))
        .collect();
    let initial = State(
        (0..p.num_vars)
            .map(|v| rng.fact(&variables, v).value)
            .collect(),
    );
    let n_goal = rng.range_usize(1, 2);
    let goal = rng.facts(&variables, p.num_vars, n_goal);

    let mut task = Task {
        variables,
        operators,
        initial,
        goal,
        axioms: Vec::new(),
    };

    // Derived variables come last, so tasks without them stay the same per seed.
    for layer in 0..p.num_derived {
        add_derived_variable(&mut rng, &mut task, layer, p.num_ops);
    }

    task
}

/// Operator `op{index}` over `variables`, with conditional effects if
/// `p.max_conditions` allows them.
fn random_operator(
    rng: &mut Rng,
    p: &RandomTaskParams,
    variables: &[Variable],
    index: usize,
) -> Operator {
    let n_pre = rng.range_usize(0, 2);
    let n_eff = rng.range_usize(1, 2);
    let pre = rng.facts(variables, p.num_vars, n_pre);
    let eff = rng
        .distinct_vars(p.num_vars, n_eff)
        .into_iter()
        .map(|v| {
            let fact = rng.fact(variables, v);
            // No draw without conditions, so seeds keep their tasks.
            let conditions = if p.max_conditions == 0 {
                Vec::new()
            } else {
                let n = rng.range_usize(0, p.max_conditions);
                rng.facts(variables, p.num_vars, n)
            };

            Effect { conditions, fact }
        })
        .collect();

    Operator {
        name: format!("op{index}"),
        pre,
        eff,
        cost: rng.range_u32(1, p.max_cost.max(1)),
    }
}

/// Appends binary derived variable `d{layer}` (default 0, in axiom layer
/// `layer`) with one or two rules, and makes either the goal or one of the
/// first `num_ops` operators' preconditions mention it.
fn add_derived_variable(rng: &mut Rng, task: &mut Task, layer: usize, num_ops: usize) {
    let var = task.variables.len();
    for _ in 0..rng.range(1, 2) {
        // Conditions on state variables and lower layers (at any value, which
        // includes negation as failure).
        let n = rng.range_usize(1, 2);
        let conditions = rng.facts(&task.variables, var, n);
        task.axioms.push(Axiom {
            conditions,
            head: Fact::new(var, 1),
        });
    }

    task.variables.push(Variable {
        axiom_layer: Some(u32::try_from(layer).expect("at most u32::MAX axiom layers")),
        ..Variable::new(format!("d{layer}"), 2)
    });
    task.initial.0.push(0);

    let derived = Fact::new(var, rng.range_u32(0, 1));
    if rng.range(0, 1) == 0 {
        task.goal.push(derived);
    } else {
        let op = rng.range_usize(0, num_ops.max(1) - 1);
        if let Some(op) = task.operators.get_mut(op) {
            op.pre.push(derived);
        }
    }
}
