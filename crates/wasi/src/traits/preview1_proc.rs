use crate::preview1::types;
use wiggle::GuestMemory;

/// Trait defining the WASI Preview1 process operations.
/// This trait allows for mocking and intercepting process operations for testing and debugging.
#[async_trait::async_trait]
pub trait WasiPreview1Proc {
    /// Terminate the process with an exit code.
    fn proc_exit(
        &mut self,
        memory: &mut GuestMemory<'_>,
        status: types::Exitcode,
    ) -> anyhow::Error;

    /// Send a signal to the process.
    fn proc_raise(
        &mut self,
        memory: &mut GuestMemory<'_>,
        sig: types::Signal,
    ) -> Result<(), types::Error>;

    /// Temporarily yield execution of the calling thread.
    fn sched_yield(
        &mut self,
        memory: &mut GuestMemory<'_>,
    ) -> Result<(), types::Error>;
} 