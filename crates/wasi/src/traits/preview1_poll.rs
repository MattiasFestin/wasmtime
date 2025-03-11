use crate::preview1::types;
use wiggle::GuestMemory;
use wiggle::GuestPtr;

/// Trait defining the WASI Preview1 polling operations.
/// This trait allows for mocking and intercepting polling operations for testing and debugging.
#[async_trait::async_trait]
pub trait WasiPreview1Poll {
    /// Wait for events on file descriptors.
    async fn poll_oneoff(
        &mut self,
        memory: &mut GuestMemory<'_>,
        subs: GuestPtr<types::Subscription>,
        events: GuestPtr<types::Event>,
        nsubscriptions: types::Size,
    ) -> Result<types::Size, types::Error>;
} 