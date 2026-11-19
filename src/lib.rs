//! Minimal SAS+ planning tasks, labeled transition systems, and the
//! `Abstraction` trait shared by `merge-and-shrink` and `cost-partition`.

mod lts;

pub use lts::{LabeledTransitionSystem, Transition};
