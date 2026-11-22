//! Parsing and abstraction construction on real translator output.
//!
//! ```bash
//! cargo bench --bench construction
//! ```

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use sas_plus::fdr::SasFile;
use sas_plus::{Projection, Task};

/// The text of `tests/fixtures/translated/<name>/output.sas`.
fn read(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/translated/{name}/output.sas",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The translated fixture `name` as a task.
fn task(name: &str) -> Task {
    SasFile::parse(&read(name)).unwrap().to_task()
}

/// Parsing the largest fixture, `logistics-e`.
fn parse(c: &mut Criterion) {
    let text = read("logistics-e");
    c.bench_function("parse/logistics-e", |b| {
        b.iter(|| SasFile::parse(black_box(&text)).unwrap());
    });
}

/// Building projections of `logistics-e` onto 2, 4 and 6 variables.
fn projection(c: &mut Criterion) {
    let task = task("logistics-e");
    let mut group = c.benchmark_group("projection/logistics-e");
    for size in [2, 4, 6] {
        let pattern: Vec<usize> = (0..size).collect();
        group.bench_with_input(BenchmarkId::from_parameter(size), &pattern, |b, p| {
            b.iter(|| Projection::new(&task, p.clone()));
        });
    }
    group.finish();
}

/// Building h* by exhaustive search on small fixtures with costs, axioms and conditional effects.
fn perfect_heuristic(c: &mut Criterion) {
    let mut group = c.benchmark_group("perfect_heuristic");
    for name in ["gripper-4", "wiring", "courier"] {
        let task = task(name);
        group.bench_function(name, |b| b.iter(|| task.perfect_heuristic()(&task.initial)));
    }
    group.finish();
}

criterion_group!(benches, parse, projection, perfect_heuristic);
criterion_main!(benches);
