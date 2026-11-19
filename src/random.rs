//! Tiny deterministic random task generator for property tests across the
//! workspace (no external RNG dependency).

use alloc::format;
use alloc::vec::Vec;

use crate::{Fact, Operator, State, Task, Variable};

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
}

impl Default for RandomTaskParams {
    fn default() -> Self {
        Self {
            num_vars: 4,
            max_domain: 3,
            num_ops: 10,
            max_cost: 5,
        }
    }
}

pub fn random_task(seed: u64, p: RandomTaskParams) -> Task {
    let mut rng = Rng::new(seed);
    let variables: Vec<Variable> = (0..p.num_vars)
        .map(|i| Variable {
            name: format!("v{i}"),
            domain_size: rng.range(2, u64::from(p.max_domain.max(2))) as u32,
        })
        .collect();
    let fact = |rng: &mut Rng, var: usize| {
        Fact::new(
            var,
            rng.range(0, u64::from(variables[var].domain_size) - 1) as u32,
        )
    };
    let operators = (0..p.num_ops)
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
                .map(|v| fact(&mut rng, v))
                .collect();
            Operator {
                name: format!("op{i}"),
                pre,
                eff,
                cost: rng.range(1, u64::from(p.max_cost.max(1))) as u32,
            }
        })
        .collect();
    let initial = State((0..p.num_vars).map(|v| fact(&mut rng, v).value).collect());
    let n_goal = rng.range(1, 2) as usize;
    let goal = rng
        .distinct_vars(p.num_vars, n_goal)
        .into_iter()
        .map(|v| fact(&mut rng, v))
        .collect();
    let task = Task {
        variables,
        operators,
        initial,
        goal,
    };
    debug_assert!(task.validate().is_ok());
    task
}
