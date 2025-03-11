use crate::preview1::types;
use wiggle::GuestMemory;

/// Trait defining the WASI Preview1 socket operations.
/// This trait allows for mocking and intercepting socket operations for testing and debugging.
#[async_trait::async_trait]
pub trait WasiPreview1Sock {
    /// Accept a new connection on a socket.
    fn sock_accept(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        flags: types::Fdflags,
    ) -> Result<types::Fd, types::Error>;

    /// Receive a message from a socket.
    fn sock_recv(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        ri_data: types::IovecArray,
        ri_flags: types::Riflags,
    ) -> Result<(types::Size, types::Roflags), types::Error>;

    /// Send a message on a socket.
    fn sock_send(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        si_data: types::CiovecArray,
        si_flags: types::Siflags,
    ) -> Result<types::Size, types::Error>;

    /// Shut down socket send and receive channels.
    fn sock_shutdown(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        how: types::Sdflags,
    ) -> Result<(), types::Error>;
} 