use crate::preview1::types;
use wiggle::GuestMemory;
use wiggle::GuestPtr;

/// Trait defining the WASI Preview1 environment and arguments operations.
/// This trait allows for mocking and intercepting environment and arguments operations for testing and debugging.
#[async_trait::async_trait]
pub trait WasiPreview1Env {
    /// Get command-line arguments.
    fn args_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
        argv: GuestPtr<GuestPtr<u8>>,
        argv_buf: GuestPtr<u8>,
    ) -> Result<(), types::Error>;

    /// Get sizes of command-line argument data.
    fn args_sizes_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
    ) -> Result<(types::Size, types::Size), types::Error>;

    /// Get environment variables.
    fn environ_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
        environ: GuestPtr<GuestPtr<u8>>,
        environ_buf: GuestPtr<u8>,
    ) -> Result<(), types::Error>;

    /// Get sizes of environment variable data.
    fn environ_sizes_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
    ) -> Result<(types::Size, types::Size), types::Error>;
} 