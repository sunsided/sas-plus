use std::path::PathBuf;

use sas_plus::fdr::{ConversionError, SasFile};

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
        .join("output.sas");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

const ALL: [&str; 4] = [
    "gripper-costs",
    "gripper-unit",
    "conditional-effects",
    "derived-predicates",
];

#[test]
fn writes_back_exactly_what_it_reads() {
    for name in ALL {
        let text = fixture(name);
        let file: SasFile = text.parse().unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(file.to_string(), text, "{name}");
    }
}

#[test]
fn gripper_optimal_costs_follow_the_metric() {
    for (name, optimum) in [("gripper-costs", 7), ("gripper-unit", 5)] {
        let file = SasFile::parse(&fixture(name)).unwrap();
        assert_eq!(file.mutex_groups.len(), 5);
        let task = file.to_task().unwrap();
        task.validate().unwrap();
        assert_eq!(task.num_variables(), 5);
        assert_eq!(task.operators.len(), 18);
        assert_eq!(
            task.perfect_heuristic()(&task.initial),
            Some(optimum),
            "{name}"
        );
    }
}

#[test]
fn effect_preconditions_become_operator_preconditions() {
    let file = SasFile::parse(&fixture("gripper-costs")).unwrap();
    let task = file.to_task().unwrap();
    let pick = task
        .operators
        .iter()
        .position(|o| o.name == "pick ball1 rooma left")
        .unwrap();
    // prevail at-robby(rooma), plus pre-values of both effects
    let pre: Vec<&str> = task.operators[pick]
        .pre
        .iter()
        .map(|&f| file.fact_name(f))
        .collect();
    assert_eq!(
        pre,
        [
            "Atom at-robby(rooma)",
            "Atom at(ball1, rooma)",
            "Atom free(left)"
        ]
    );
}

#[test]
fn format_plan_matches_fast_downward() {
    let file = SasFile::parse(&fixture("gripper-costs")).unwrap();
    let task = file.to_task().unwrap();
    let id = |n: &str| task.operators.iter().position(|o| o.name == n).unwrap();
    let plan = [
        id("pick ball1 rooma left"),
        id("pick ball2 rooma right"),
        id("move rooma roomb"),
        id("drop ball1 roomb left"),
        id("drop ball2 roomb right"),
    ];
    assert_eq!(task.validate_plan(&plan), Some(7));
    assert_eq!(
        file.format_plan(&plan),
        "(pick ball1 rooma left)\n(pick ball2 rooma right)\n(move rooma roomb)\n\
         (drop ball1 roomb left)\n(drop ball2 roomb right)\n; cost = 7 (general cost)\n"
    );
}

#[test]
fn rejects_features_plain_tasks_cannot_express() {
    let cond = SasFile::parse(&fixture("conditional-effects")).unwrap();
    assert_eq!(cond.operators[0].effects[1].conditions.len(), 1);
    assert!(matches!(
        cond.to_task(),
        Err(ConversionError::ConditionalEffect { operator }) if operator == "move home office"
    ));

    let derived = SasFile::parse(&fixture("derived-predicates")).unwrap();
    assert_eq!(derived.variables[2].axiom_layer, Some(0));
    assert!(matches!(
        derived.to_task(),
        Err(ConversionError::Axioms { count: 1 })
    ));
}

#[test]
fn reports_errors_with_line_numbers() {
    let text = fixture("gripper-costs");
    let cases = [
        (
            text.replacen("begin_metric\n1", "begin_metric\n2", 1),
            5,
            "metric",
        ),
        (
            text.replacen("begin_version\n3", "begin_version\n2", 1),
            2,
            "version",
        ),
        (
            text.replacen("begin_goal\n2\n1 1", "begin_goal\n2\n1 9", 1),
            0,
            "out of range",
        ),
        (text.replacen("0 1 2 0\n", "0 1 2\n", 1), 0, "integers"),
        (
            text.replacen("end_operator", "end_op", 1),
            0,
            "expected `end_operator`",
        ),
        (
            text.trim_end().rsplit_once('\n').unwrap().0.to_string(),
            0,
            "end of file",
        ),
        (format!("{text}junk\n"), 0, "unexpected content"),
    ];
    for (input, line, needle) in cases {
        let err = SasFile::parse(&input).expect_err(needle);
        assert!(err.message.contains(needle), "{needle}: {err}");
        if line > 0 {
            assert_eq!(err.line, line, "{needle}: {err}");
        }
    }
}
