use crate::preview1::types;
use wiggle::GuestMemory;
use wiggle::GuestPtr;

/// Trait defining the WASI Preview1 random operations.
/// This trait allows for mocking and intercepting random operations for testing and debugging.
#[async_trait::async_trait]
pub trait WasiPreview1Random {
    /// Write random data into a buffer.
    fn random_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
        buf: GuestPtr<u8>,
        buf_len: types::Size,
    ) -> Result<(), types::Error>;
} 