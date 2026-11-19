//! Loads a Fast Downward `output.sas` file and prints what it contains.
//!
//! ```bash
//! cargo run --example load -- path/to/output.sas
//! ```

use sas_plus::fdr::SasFile;

/// About 100 MB of transitions.
const MAX_TRANSITIONS: usize = 1 << 22;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("usage: load <output.sas>")?;
    let file: SasFile = std::fs::read_to_string(&path)?.parse()?;
    println!(
        "{path}: {} variables, {} operators, {} axioms, metric {}",
        file.variables.len(),
        file.operators.len(),
        file.axioms.len(),
        u8::from(file.use_metric)
    );
    let task = file.to_task()?;
    task.validate()?;
    // perfect_heuristic() stores one transition per operator and state.
    let transitions = task
        .variables
        .iter()
        .try_fold(task.operators.len().max(1), |n, v| {
            n.checked_mul(v.domain_size as usize)
        });
    match transitions {
        Some(n) if n <= MAX_TRANSITIONS => match task.perfect_heuristic()(&task.initial) {
            Some(cost) => println!("optimal plan cost: {cost}"),
            None => println!("unsolvable"),
        },
        _ => println!("too large for exhaustive search, skipping the optimal cost"),
    }
    Ok(())
}
