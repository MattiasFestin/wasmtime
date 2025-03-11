//! WASI Preview1 API traits for mocking and intercepting WASI calls.
//!
//! This module provides traits for each WASI Preview1 API category, allowing for:
//! 1. Mocking and intercepting WASI calls for testing
//! 2. Creating deterministic execution environments
//! 3. Debugging and inspecting WASI calls
//! 4. Capturing snapshots for snapshot testing

mod preview1_clock;
mod preview1_env;
mod preview1_fd;
mod preview1_path;
mod preview1_poll;
mod preview1_proc;
mod preview1_random;
mod preview1_sock;

pub use preview1_clock::WasiPreview1Clock;
pub use preview1_env::WasiPreview1Env;
pub use preview1_fd::WasiPreview1Fd;
pub use preview1_path::WasiPreview1Path;
pub use preview1_poll::WasiPreview1Poll;
pub use preview1_proc::WasiPreview1Proc;
pub use preview1_random::WasiPreview1Random;
pub use preview1_sock::WasiPreview1Sock;

/// Trait that combines all WASI Preview1 API traits.
/// This trait can be used to implement a complete WASI Preview1 API provider.
#[async_trait::async_trait]
pub trait WasiPreview1Api: 
    WasiPreview1Clock +
    WasiPreview1Env +
    WasiPreview1Fd +
    WasiPreview1Path +
    WasiPreview1Poll +
    WasiPreview1Proc +
    WasiPreview1Random +
    WasiPreview1Sock
{} 