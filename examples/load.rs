//! Loads a Fast Downward `output.sas` file and prints what it contains.
//!
//! ```bash
//! cargo run --example load -- path/to/output.sas
//! ```

use sas_plus::fdr::SasFile;

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
    // Exhaustive search over the full state space: only for small tasks.
    match task.perfect_heuristic()(&task.initial) {
        Some(cost) => println!("optimal plan cost: {cost}"),
        None => println!("unsolvable"),
    }
    Ok(())
}
