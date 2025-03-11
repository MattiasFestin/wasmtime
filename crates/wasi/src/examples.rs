//! Examples of how to use the WASI API traits for testing and debugging.

pub use crate::deterministic_testing::{
    run_deterministic_test,
    create_wasi_snapshot,
    replay_from_snapshot,
}; 