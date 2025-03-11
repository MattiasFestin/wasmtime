use crate::preview1::types;
use wiggle::GuestMemory;

/// Trait defining the WASI Preview1 clock operations.
/// This trait allows for mocking and intercepting clock operations for testing and debugging.
#[async_trait::async_trait]
pub trait WasiPreview1Clock {
    /// Return the resolution of a clock.
    fn clock_res_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
        id: types::Clockid,
    ) -> Result<types::Timestamp, types::Error>;

    /// Return the time value of a clock.
    fn clock_time_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
        id: types::Clockid,
        precision: types::Timestamp,
    ) -> Result<types::Timestamp, types::Error>;
} 