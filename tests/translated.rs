//! Real Fast Downward translator output (see tests/fixtures/translated/README.md).

#![cfg(feature = "fdr")]

use sas_plus::fdr::SasFile;

const SMALL: [(&str, u64); 5] = [
    ("sussman-anomaly", 3),
    ("gripper-2", 3),
    ("gripper-4", 9),
    ("gripper-costs", 7),
    ("courier", 16),
];
/// Plain SAS+ tasks: no conditional effects, no axioms.
const PLAIN: [&str; 6] = [
    "sussman-anomaly",
    "gripper-2",
    "gripper-4",
    "gripper-costs",
    "bw-large-a",
    "logistics-e",
];
/// Tasks whose operators keep conditional effects after translation.
const CONDITIONAL: [&str; 2] = ["courier", "get-paid4"];

fn read(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/translated/{name}/output.sas",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

#[test]
fn translator_output_round_trips_byte_for_byte() {
    for name in PLAIN.iter().chain(&CONDITIONAL) {
        let text = read(name);
        let file = SasFile::parse(&text).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(file.to_string(), text, "{name}");
    }
}

#[test]
fn all_tasks_convert_to_valid_tasks() {
    for name in PLAIN.iter().chain(&CONDITIONAL) {
        let file = SasFile::parse(&read(name)).unwrap();
        file.to_task()
            .validate()
            .unwrap_or_else(|e| panic!("{name}: {e}"));
    }
}

#[test]
fn small_tasks_have_their_known_optimal_cost() {
    for (name, optimum) in SMALL {
        let task = SasFile::parse(&read(name)).unwrap().to_task();
        assert_eq!(
            task.perfect_heuristic()(&task.initial),
            Some(optimum),
            "{name}"
        );
    }
}

#[test]
fn translator_output_shape() {
    let file = SasFile::parse(&read("logistics-e")).unwrap();
    assert!(!file.use_metric);
    assert_eq!(
        (file.variables.len(), file.operators.len(), file.goal.len()),
        (19, 756, 11)
    );
    assert!(file.axioms.is_empty());
    assert!(file.variables.iter().all(|v| v.axiom_layer.is_none()));
}

/// The same task as the hand-encoded `fixtures/gripper-costs`, so both must agree.
#[test]
fn action_costs_survive_translation() {
    let file = SasFile::parse(&read("gripper-costs")).unwrap();
    assert!(file.use_metric);
    let costs = |prefix: &str| {
        let mut c: Vec<u32> = file
            .operators
            .iter()
            .filter(|o| o.name.starts_with(prefix))
            .map(|o| o.cost)
            .collect();
        c.dedup();
        c
    };
    assert_eq!(costs("move "), [3]);
    assert_eq!(costs("pick "), [1]);
    assert_eq!(costs("drop "), [1]);

    let hand = format!(
        "{}/tests/fixtures/gripper-costs/output.sas",
        env!("CARGO_MANIFEST_DIR")
    );
    let hand = SasFile::parse(&std::fs::read_to_string(hand).unwrap())
        .unwrap()
        .to_task();
    let translated = file.to_task();
    assert_eq!(
        hand.perfect_heuristic()(&hand.initial),
        translated.perfect_heuristic()(&translated.initial)
    );
}

#[test]
fn conditional_effects_survive_translation_and_conversion() {
    for name in CONDITIONAL {
        let file = SasFile::parse(&read(name)).unwrap();
        let in_file = file
            .operators
            .iter()
            .flat_map(|o| &o.effects)
            .filter(|e| !e.conditions.is_empty())
            .count();
        assert!(in_file > 0, "{name}: no effect conditions left");
        let task = file.to_task();
        task.validate().unwrap_or_else(|e| panic!("{name}: {e}"));
        let in_task = task
            .operators
            .iter()
            .flat_map(|o| &o.eff)
            .filter(|e| !e.conditions.is_empty())
            .count();
        assert_eq!(in_file, in_task, "{name}");
    }
}

/// Courier: driving costs the road length, and moving the van moves every
/// loaded parcel (one conditional effect per parcel and location fact).
#[test]
fn courier_combines_action_costs_and_conditional_effects() {
    let file = SasFile::parse(&read("courier")).unwrap();
    assert!(file.use_metric);
    let op = |name: &str| file.operators.iter().find(|o| o.name == name).unwrap();
    for (name, length) in [
        ("drive depot north", 4),
        ("drive north south", 2),
        ("drive depot south", 7),
    ] {
        let drive = op(name);
        assert_eq!(drive.cost, length, "{name}");
        let conditions: Vec<&str> = drive
            .effects
            .iter()
            .flat_map(|e| &e.conditions)
            .map(|&c| file.fact_name(c))
            .collect();
        assert!(
            conditions.iter().all(|c| c.starts_with("Atom loaded(")),
            "{name}: {conditions:?}"
        );
        assert_eq!(conditions.len(), 4, "{name}: {conditions:?}");
    }
    assert_eq!(op("load parcel1 depot").cost, 1);
}
