//! Integration tests, compiled as one test binary.
//!
//! The translator fixtures in `tests/fixtures/translated/` are tested by the
//! separate `tests/translated.rs`, which the published package leaves out
//! together with those third-party files.

#[cfg(feature = "fdr")]
mod fdr;
#[cfg(feature = "random")]
mod properties;
#[cfg(feature = "random")]
mod proptests;
#[cfg(feature = "random")]
mod task;
