#![cfg(feature = "fdr")]

use std::path::PathBuf;

use sas_plus::fdr::SasFile;

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
        let task = file.to_task();
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
    let task = file.to_task();
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
    let task = file.to_task();
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

/// Fast Downward writes "unit cost" whenever every operator costs 1, even
/// with metric 1 (checked against its search on such a file).
#[test]
fn format_plan_labels_unit_costs_by_the_costs() {
    let text = fixture("gripper-costs").replace("\n3\nend_operator", "\n1\nend_operator");
    let file = SasFile::parse(&text).unwrap();
    assert!(file.use_metric);
    assert_eq!(file.format_plan(&[]), "; cost = 0 (unit cost)\n");
}

#[test]
fn conditional_effects_and_axioms_carry_over_to_the_task() {
    let file = SasFile::parse(&fixture("conditional-effects")).unwrap();
    assert_eq!(file.operators[0].effects[1].conditions.len(), 1);
    let task = file.to_task();
    task.validate().unwrap();
    let move_op = &task.operators[0];
    assert_eq!(move_op.name, "move home office");
    assert_eq!(
        move_op.eff[1].conditions,
        file.operators[0].effects[1].conditions
    );

    let file = SasFile::parse(&fixture("derived-predicates")).unwrap();
    assert_eq!(file.variables[2].axiom_layer, Some(0));
    let task = file.to_task();
    task.validate().unwrap();
    assert_eq!(task.variables[2].axiom_layer, Some(0));
    assert_eq!(task.axioms.len(), 1);
    // connected(n1, n3) needs both edges open.
    assert_eq!(task.perfect_heuristic()(&task.initial), Some(2));
    assert_eq!(task.validate_plan(&[0]), None);
    assert_eq!(task.validate_plan(&[0, 1]), Some(2));
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
        (
            format!("{text}\n\njunk\n"),
            text.lines().count() + 3,
            "unexpected content",
        ),
    ];
    let derived = fixture("derived-predicates");
    let rule = derived.lines().position(|l| l == "2 1 0").unwrap() + 1;
    let cases = cases.into_iter().chain([
        (
            derived.replacen("\n2 1 0\nend_rule", "\n2 0 1\nend_rule", 1),
            rule,
            "differs from the default",
        ),
        (
            derived.replacen("var2\n0\n", "var2\n-1\n", 1),
            rule,
            "axiom sets state variable",
        ),
    ]);
    for (input, line, needle) in cases {
        let err = SasFile::parse(&input).expect_err(needle);
        assert!(err.message.contains(needle), "{needle}: {err}");
        if line > 0 {
            assert_eq!(err.line, line, "{needle}: {err}");
        }
    }
}
