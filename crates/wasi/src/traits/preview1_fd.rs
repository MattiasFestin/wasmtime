use crate::preview1::types;
use wiggle::GuestMemory;
use wiggle::GuestPtr;

/// Trait defining the WASI Preview1 file descriptor operations.
/// This trait allows for mocking and intercepting file descriptor operations for testing and debugging.
#[async_trait::async_trait]
pub trait WasiPreview1Fd {
    /// Provide file advisory information on a file descriptor.
    async fn fd_advise(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        offset: types::Filesize,
        len: types::Filesize,
        advice: types::Advice,
    ) -> Result<(), types::Error>;

    /// Allocate space in a file.
    fn fd_allocate(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        offset: types::Filesize,
        len: types::Filesize,
    ) -> Result<(), types::Error>;

    /// Close a file descriptor.
    async fn fd_close(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
    ) -> Result<(), types::Error>;

    /// Synchronize the data of a file to disk.
    async fn fd_datasync(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
    ) -> Result<(), types::Error>;

    /// Get the status of a file descriptor.
    async fn fd_fdstat_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
    ) -> Result<types::Fdstat, types::Error>;

    /// Adjust the flags associated with a file descriptor.
    fn fd_fdstat_set_flags(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        flags: types::Fdflags,
    ) -> Result<(), types::Error>;

    /// Adjust the rights associated with a file descriptor.
    fn fd_fdstat_set_rights(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        fs_rights_base: types::Rights,
        fs_rights_inheriting: types::Rights,
    ) -> Result<(), types::Error>;

    /// Return the attributes of an open file.
    async fn fd_filestat_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
    ) -> Result<types::Filestat, types::Error>;

    /// Adjust the size of an open file.
    async fn fd_filestat_set_size(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        size: types::Filesize,
    ) -> Result<(), types::Error>;

    /// Adjust the timestamps of an open file.
    async fn fd_filestat_set_times(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        atim: types::Timestamp,
        mtim: types::Timestamp,
        fst_flags: types::Fstflags,
    ) -> Result<(), types::Error>;

    /// Read from a file descriptor.
    async fn fd_read(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        iovs: types::IovecArray,
    ) -> Result<types::Size, types::Error>;

    /// Read from a file descriptor at a given offset.
    async fn fd_pread(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        iovs: types::IovecArray,
        offset: types::Filesize,
    ) -> Result<types::Size, types::Error>;

    /// Write to a file descriptor.
    async fn fd_write(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        ciovs: types::CiovecArray,
    ) -> Result<types::Size, types::Error>;

    /// Write to a file descriptor at a given offset.
    async fn fd_pwrite(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        ciovs: types::CiovecArray,
        offset: types::Filesize,
    ) -> Result<types::Size, types::Error>;

    /// Get information about a pre-opened file descriptor.
    fn fd_prestat_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
    ) -> Result<types::Prestat, types::Error>;

    /// Get the path of a pre-opened directory.
    fn fd_prestat_dir_name(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        path: GuestPtr<u8>,
        path_max_len: types::Size,
    ) -> Result<(), types::Error>;

    /// Atomically replace a file descriptor by renumbering another file descriptor.
    fn fd_renumber(
        &mut self,
        memory: &mut GuestMemory<'_>,
        from: types::Fd,
        to: types::Fd,
    ) -> Result<(), types::Error>;

    /// Move the offset of a file descriptor.
    async fn fd_seek(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        offset: types::Filedelta,
        whence: types::Whence,
    ) -> Result<types::Filesize, types::Error>;

    /// Synchronize the data and metadata of a file to disk.
    async fn fd_sync(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
    ) -> Result<(), types::Error>;

    /// Return the current offset of a file descriptor.
    fn fd_tell(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
    ) -> Result<types::Filesize, types::Error>;

    /// Read directory entries from a directory.
    async fn fd_readdir(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        buf: GuestPtr<u8>,
        buf_len: types::Size,
        cookie: types::Dircookie,
    ) -> Result<types::Size, types::Error>;
} 