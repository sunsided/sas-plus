//! Arbitrary input never panics the `output.sas` reader. Whatever it accepts
//! writes back to text it reads again identically, and converts to a task
//! that `validate` either rejects or can be stepped through without panics.
//! (The reader checks syntax and ranges; `Task::validate` checks semantics.)

#![no_main]

use libfuzzer_sys::fuzz_target;
use sas_plus::fdr::SasFile;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(file) = SasFile::parse(text) else {
        return;
    };
    let written = file.to_string();
    assert_eq!(SasFile::parse(&written).as_ref(), Ok(&file));
    let task = file.to_task();
    if task.validate().is_err() {
        return;
    }
    let mut state = task.initial_state();
    for _ in 0..8 {
        let Some(op) = task.applicable_operators(&state).next() else {
            break;
        };
        state = task.apply(op, &state);
    }
    let _ = task.is_goal(&state);
});
