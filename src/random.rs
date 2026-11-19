//! Tiny deterministic random task generator for property tests across the
//! workspace (no external RNG dependency).

use alloc::format;
use alloc::vec::Vec;

use crate::{Axiom, Effect, Fact, Operator, State, Task, Variable};

/// xorshift64* — good enough for test-case generation.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    /// Uniform in `lo..=hi`.
    pub fn range(&mut self, lo: u64, hi: u64) -> u64 {
        lo + self.next_u64() % (hi - lo + 1)
    }
    fn distinct_vars(&mut self, n: usize, k: usize) -> Vec<usize> {
        let mut vars: Vec<usize> = (0..n).collect();
        for i in 0..k.min(n) {
            let j = self.range(i as u64, n as u64 - 1) as usize;
            vars.swap(i, j);
        }
        vars.truncate(k.min(n));
        vars
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RandomTaskParams {
    pub num_vars: usize,
    pub max_domain: u32,
    pub num_ops: usize,
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

pub fn random_task(seed: u64, p: RandomTaskParams) -> Task {
    let mut rng = Rng::new(seed);
    let variables: Vec<Variable> = (0..p.num_vars)
        .map(|i| {
            Variable::new(
                format!("v{i}"),
                rng.range(2, u64::from(p.max_domain.max(2))) as u32,
            )
        })
        .collect();
    let fact = |rng: &mut Rng, var: usize| {
        Fact::new(
            var,
            rng.range(0, u64::from(variables[var].domain_size) - 1) as u32,
        )
    };
    let mut operators: Vec<Operator> = (0..p.num_ops)
        .map(|i| {
            let n_pre = rng.range(0, 2) as usize;
            let n_eff = rng.range(1, 2) as usize;
            let pre = rng
                .distinct_vars(p.num_vars, n_pre)
                .into_iter()
                .map(|v| fact(&mut rng, v))
                .collect();
            let eff = rng
                .distinct_vars(p.num_vars, n_eff)
                .into_iter()
                .map(|v| {
                    let head = fact(&mut rng, v);
                    // No draw without conditions, so seeds keep their tasks.
                    let conditions = if p.max_conditions == 0 {
                        Vec::new()
                    } else {
                        let n = rng.range(0, p.max_conditions as u64) as usize;
                        rng.distinct_vars(p.num_vars, n)
                            .into_iter()
                            .map(|c| fact(&mut rng, c))
                            .collect()
                    };
                    Effect {
                        conditions,
                        fact: head,
                    }
                })
                .collect();
            Operator {
                name: format!("op{i}"),
                pre,
                eff,
                cost: rng.range(1, u64::from(p.max_cost.max(1))) as u32,
            }
        })
        .collect();
    let mut initial = State((0..p.num_vars).map(|v| fact(&mut rng, v).value).collect());
    let n_goal = rng.range(1, 2) as usize;
    let mut goal: Vec<Fact> = rng
        .distinct_vars(p.num_vars, n_goal)
        .into_iter()
        .map(|v| fact(&mut rng, v))
        .collect();

    // Derived variables come last, so tasks without them stay the same per seed.
    let mut variables = variables;
    let mut axioms = Vec::new();
    for layer in 0..p.num_derived {
        let var = variables.len();
        for _ in 0..rng.range(1, 2) {
            // Conditions on state variables and lower layers (at any value, which
            // includes negation as failure).
            let n = rng.range(1, 2) as usize;
            let conditions = rng
                .distinct_vars(var, n)
                .into_iter()
                .map(|c| {
                    let domain = variables[c].domain_size;
                    Fact::new(c, rng.range(0, u64::from(domain) - 1) as u32)
                })
                .collect();
            axioms.push(Axiom {
                conditions,
                head: Fact::new(var, 1),
            });
        }
        variables.push(Variable {
            axiom_layer: Some(layer as u32),
            ..Variable::new(format!("d{layer}"), 2)
        });
        initial.0.push(0);
        let derived = Fact::new(var, rng.range(0, 1) as u32);
        if rng.range(0, 1) == 0 {
            goal.push(derived);
        } else {
            let op = rng.range(0, p.num_ops.max(1) as u64 - 1) as usize;
            if let Some(op) = operators.get_mut(op) {
                op.pre.push(derived);
            }
        }
    }
    Task {
        variables,
        operators,
        initial,
        goal,
        axioms,
    }
}
