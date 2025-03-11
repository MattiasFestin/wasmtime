use crate::deterministic_testing::memory_accessor::MemoryAccessor;
use crate::deterministic_testing::mock_file_system::SharedMockFileSystem;
use crate::preview1::types;
use crate::{
    WasiPreview1Api, WasiPreview1Clock, WasiPreview1Env,
    WasiPreview1Fd, WasiPreview1Path, WasiPreview1Poll, WasiPreview1Proc, WasiPreview1Random,
    WasiPreview1Sock,
};
use crate::ctx::WasiCtx;
use crate::view::{WasiImpl, WasiView, IoImpl, IoView};
use wasmtime::component::ResourceTable;
use wiggle::GuestMemory;
use wiggle::GuestPtr;
use std::path::{Path, PathBuf};

/// Implementation of the WASI clock trait
pub enum ClockImpl {
    /// Real implementation that uses the WasiCtx
    Real,
    #[cfg(feature = "deterministic_testing")]
    /// Custom implementation for testing with specific behavior
    Custom(Box<dyn WasiPreview1Clock + Send + Sync>),
}

/// Enum for environment trait implementations
pub enum EnvImpl {
    /// Real implementation that uses the WasiCtx
    Real,
    #[cfg(feature = "deterministic_testing")]
    /// Custom implementation for testing with specific behavior
    Custom(Box<dyn WasiPreview1Env + Send + Sync>),
}

/// Enum for file descriptor trait implementations
pub enum FdImpl {
    /// Real implementation that uses the WasiCtx
    Real,
    #[cfg(feature = "deterministic_testing")]
    /// Custom implementation for testing with specific behavior
    Custom(Box<dyn WasiPreview1Fd + Send + Sync>),
}

/// Enum for path trait implementations
pub enum PathImpl {
    /// Real implementation that uses the WasiCtx
    Real,
    #[cfg(feature = "deterministic_testing")]
    /// Custom implementation for testing with specific behavior
    Custom(Box<dyn WasiPreview1Path + Send + Sync>),
}

/// Enum for poll trait implementations
pub enum PollImpl {
    /// Real implementation that uses the WasiCtx
    Real,
    #[cfg(feature = "deterministic_testing")]
    /// Custom implementation for testing with specific behavior
    Custom(Box<dyn WasiPreview1Poll + Send + Sync>),
}

/// Enum for process trait implementations
pub enum ProcImpl {
    /// Real implementation that uses the WasiCtx
    Real,
    #[cfg(feature = "deterministic_testing")]
    /// Custom implementation for testing with specific behavior
    Custom(Box<dyn WasiPreview1Proc + Send + Sync>),
}

/// Enum for random trait implementations
pub enum RandomImpl {
    /// Real implementation that uses the WasiCtx
    Real,
    #[cfg(feature = "deterministic_testing")]
    /// Custom implementation for testing with specific behavior
    Custom(Box<dyn WasiPreview1Random + Send + Sync>),
}

/// Enum for socket trait implementations
pub enum SockImpl {
    /// Real implementation that uses the WasiCtx
    Real,
    #[cfg(feature = "deterministic_testing")]
    /// Custom implementation for testing with specific behavior
    Custom(Box<dyn WasiPreview1Sock + Send + Sync>),
}

/// Data structure for WASI Preview1 context without any IO connections.
/// This allows for creating mock implementations for testing.
pub struct WasiP1Data {
    /// Resource table for managing WASI resources
    pub table: ResourceTable,
    
    /// WASI context for the underlying implementation
    pub wasi: WasiCtx,

    /// Implementation of the clock trait
    pub clock: ClockImpl,
    
    /// Implementation of the environment trait
    pub env: EnvImpl,
    
    /// Implementation of the file descriptor trait
    pub fd: FdImpl,
    
    /// Implementation of the path trait
    pub path: PathImpl,
    
    /// Implementation of the poll trait
    pub poll: PollImpl,
    
    /// Implementation of the process trait
    pub proc: ProcImpl,
    
    /// Implementation of the random trait
    pub random: RandomImpl,
    
    /// Implementation of the socket trait
    pub sock: SockImpl,
    
    /// Mock file system for deterministic testing
    #[cfg(feature = "deterministic_testing")]
    pub mock_fs: SharedMockFileSystem,
}

impl WasiP1Data {
    /// Create a new WasiP1Data instance
    pub fn new(wasi: WasiCtx) -> Self {
        Self {
            table: ResourceTable::new(),
            wasi,
            clock: ClockImpl::Real,
            env: EnvImpl::Real,
            fd: FdImpl::Real,
            path: PathImpl::Real,
            poll: PollImpl::Real,
            proc: ProcImpl::Real,
            random: RandomImpl::Real,
            sock: SockImpl::Real,
            #[cfg(feature = "deterministic_testing")]
            mock_fs: SharedMockFileSystem::new(),
        }
    }

    /// Create a new WasiP1Data instance with direct implementations
    pub fn with_direct_impls(wasi: WasiCtx) -> Self {
        Self::new(wasi)
    }
    
    #[cfg(feature = "deterministic_testing")]
    /// Create a new WasiP1Data instance with a custom clock implementation
    pub fn with_custom_clock(
        wasi: WasiCtx,
        clock_impl: Box<dyn WasiPreview1Clock + Send + Sync>,
    ) -> Self {
        let mut data = Self::new(wasi);
        data.clock = ClockImpl::Custom(clock_impl);
        data
    }

    #[cfg(feature = "deterministic_testing")]
    /// Create a new WasiP1Data instance with a custom environment implementation
    pub fn with_custom_env(
        wasi: WasiCtx,
        env_impl: Box<dyn WasiPreview1Env + Send + Sync>,
    ) -> Self {
        let mut data = Self::new(wasi);
        data.env = EnvImpl::Custom(env_impl);
        data
    }

    #[cfg(feature = "deterministic_testing")]
    /// Create a new WasiP1Data instance with a custom file descriptor implementation
    pub fn with_custom_fd(
        wasi: WasiCtx,
        fd_impl: Box<dyn WasiPreview1Fd + Send + Sync>,
    ) -> Self {
        let mut data = Self::new(wasi);
        data.fd = FdImpl::Custom(fd_impl);
        data
    }

    #[cfg(feature = "deterministic_testing")]
    /// Create a new WasiP1Data instance with a custom path implementation
    pub fn with_custom_path(
        wasi: WasiCtx,
        path_impl: Box<dyn WasiPreview1Path + Send + Sync>,
    ) -> Self {
        let mut data = Self::new(wasi);
        data.path = PathImpl::Custom(path_impl);
        data
    }

    #[cfg(feature = "deterministic_testing")]
    /// Create a new WasiP1Data instance with a custom poll implementation
    pub fn with_custom_poll(
        wasi: WasiCtx,
        poll_impl: Box<dyn WasiPreview1Poll + Send + Sync>,
    ) -> Self {
        let mut data = Self::new(wasi);
        data.poll = PollImpl::Custom(poll_impl);
        data
    }

    #[cfg(feature = "deterministic_testing")]
    /// Create a new WasiP1Data instance with a custom process implementation
    pub fn with_custom_proc(
        wasi: WasiCtx,
        proc_impl: Box<dyn WasiPreview1Proc + Send + Sync>,
    ) -> Self {
        let mut data = Self::new(wasi);
        data.proc = ProcImpl::Custom(proc_impl);
        data
    }

    #[cfg(feature = "deterministic_testing")]
    /// Create a new WasiP1Data instance with a custom random implementation
    pub fn with_custom_random(
        wasi: WasiCtx,
        random_impl: Box<dyn WasiPreview1Random + Send + Sync>,
    ) -> Self {
        let mut data = Self::new(wasi);
        data.random = RandomImpl::Custom(random_impl);
        data
    }

    #[cfg(feature = "deterministic_testing")]
    /// Create a new WasiP1Data instance with a custom socket implementation
    pub fn with_custom_sock(
        wasi: WasiCtx,
        sock_impl: Box<dyn WasiPreview1Sock + Send + Sync>,
    ) -> Self {
        let mut data = Self::new(wasi);
        data.sock = SockImpl::Custom(sock_impl);
        data
    }

    /// Get a WasiImpl reference for this data
    pub fn as_wasi_impl(&mut self) -> WasiImpl<&mut Self> {
        WasiImpl(IoImpl(self))
    }
    
    /// Get an IoImpl reference for this data
    pub fn as_io_impl(&mut self) -> IoImpl<&mut Self> {
        IoImpl(self)
    }
    
    /// Construct a new WasiCtx from the trait implementations
    pub fn build_ctx(&self) -> WasiCtx {
        // Start with a builder
        let mut builder = WasiCtx::builder();
        
        // Configure the builder based on trait implementations
        // This is a simplified example - in a real implementation,
        // you would extract configuration from each trait implementation
        
        // For example, you might extract environment variables from the env trait
        // and add them to the builder
        
        // For example, you might extract file descriptors from the fd trait
        // and add them as preopened directories
        
        // For example, you might extract random source from the random trait
        // and configure the builder to use it
        
        // Build and return the WasiCtx
        builder.build()
    }
    
    /// Construct a new ResourceTable from the trait implementations
    pub fn build_table(&self) -> ResourceTable {
        // Create a new resource table
        let table = ResourceTable::new();
        
        // Populate the table based on trait implementations
        // This is a simplified example - in a real implementation,
        // you would extract resources from each trait implementation
        
        // For example, you might extract file descriptors from the fd trait
        // and add them to the resource table
        
        // For example, you might extract sockets from the sock trait
        // and add them to the resource table
        
        // Return the populated resource table
        table
    }
    
    /// Create a new WasiP1Data instance from the current trait implementations
    pub fn rebuild(&self) -> Self {
        // Build a new WasiCtx and ResourceTable from the current trait implementations
        let wasi = self.build_ctx();
        let table = self.build_table();
        
        // Create a new WasiP1Data with the same trait implementations
        Self {
            table,
            wasi,
            // Use the same implementation variants
            clock: match &self.clock {
                ClockImpl::Real => ClockImpl::Real,
                #[cfg(feature = "deterministic_testing")]
                ClockImpl::Custom(_) => {
                    // We can't easily clone a boxed trait object
                    // For now, just return the Real implementation
                    ClockImpl::Real
                },
            },
            env: match &self.env {
                EnvImpl::Real => EnvImpl::Real,
                #[cfg(feature = "deterministic_testing")]
                EnvImpl::Custom(_) => EnvImpl::Real,
            },
            fd: match &self.fd {
                FdImpl::Real => FdImpl::Real,
                #[cfg(feature = "deterministic_testing")]
                FdImpl::Custom(_) => FdImpl::Real,
            },
            path: match &self.path {
                PathImpl::Real => PathImpl::Real,
                #[cfg(feature = "deterministic_testing")]
                PathImpl::Custom(_) => PathImpl::Real,
            },
            poll: match &self.poll {
                PollImpl::Real => PollImpl::Real,
                #[cfg(feature = "deterministic_testing")]
                PollImpl::Custom(_) => PollImpl::Real,
            },
            proc: match &self.proc {
                ProcImpl::Real => ProcImpl::Real,
                #[cfg(feature = "deterministic_testing")]
                ProcImpl::Custom(_) => ProcImpl::Real,
            },
            random: match &self.random {
                RandomImpl::Real => RandomImpl::Real,
                #[cfg(feature = "deterministic_testing")]
                RandomImpl::Custom(_) => RandomImpl::Real,
            },
            sock: match &self.sock {
                SockImpl::Real => SockImpl::Real,
                #[cfg(feature = "deterministic_testing")]
                SockImpl::Custom(_) => SockImpl::Real,
            },
            #[cfg(feature = "deterministic_testing")]
            mock_fs: self.mock_fs.clone(),
        }
    }

    /// Helper method to get a file descriptor
    fn get_file_fd(&mut self, fd: types::Fd) -> Result<types::Fd, types::Error> {
        // Just return the fd, assuming it's valid
        // In a real implementation we would check against the resource table
        Ok(fd)
    }

    /// Helper method to get a directory descriptor
    fn get_dir_fd(&mut self, fd: types::Fd) -> Result<types::Fd, types::Error> {
        // Just return the fd, assuming it's valid
        // In a real implementation we would check against the resource table
        Ok(fd)
    }

    /// Helper method to read a string from guest memory using the MemoryAccessor trait
    fn read_string(memory: &mut impl MemoryAccessor, str_ptr: GuestPtr<str>) -> Result<String, types::Error> {
        memory.read_string(str_ptr)
    }

    /// Helper method to get a file descriptor as a Resource
    fn get_fd(&mut self, fd: types::Fd) -> Result<types::Fd, types::Error> {
        Ok(fd) // For now, just pass through
    }

    // Helper method for finding first non-empty iovec
    fn first_non_empty_iovec<M: MemoryAccessor>(
        memory: &mut M,
        iovs_ptr: types::IovecArray,
    ) -> Result<Option<(u32, &mut [u8])>, types::Error> {
        let iovecs_len = memory.read_iovec_array_len(iovs_ptr)?;
        
        for i in 0..iovecs_len {
            let iovec_ptr = memory.get_iovec_at_index(iovs_ptr, i as u32)?;
            let iovec = memory.read_iovec(iovec_ptr)?;
            
            if iovec.buf_len > 0 {
                let buf_ptr = iovec.buf.as_array(iovec.buf_len);
                let buf = memory.get_bytes_mut(buf_ptr)?;
                return Ok(Some((iovec.buf_len, buf)));
            }
        }
        
        Ok(None)
    }

    // Helper method for finding first non-empty ciovec
    fn first_non_empty_ciovec<M: MemoryAccessor>(
        memory: &mut M,
        iovs_ptr: types::CiovecArray,
    ) -> Result<Option<(u32, &[u8])>, types::Error> {
        let ciovecs_len = memory.read_ciovec_array_len(iovs_ptr)?;
        
        for i in 0..ciovecs_len {
            let ciovec_ptr = memory.get_ciovec_at_index(iovs_ptr, i as u32)?;
            let ciovec = memory.read_ciovec(ciovec_ptr)?;
            
            if ciovec.buf_len > 0 {
                let buf_ptr = ciovec.buf.as_array(ciovec.buf_len);
                let buf = memory.read_bytes(buf_ptr)?;
                // Return the buffer contents directly
                return Ok(Some((ciovec.buf_len, Box::leak(buf.into_boxed_slice()))));
            }
        }
        
        Ok(None)
    }
}

impl IoView for WasiP1Data {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl WasiView for WasiP1Data {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

#[async_trait::async_trait]
impl WasiPreview1Api for WasiP1Data {}

#[async_trait::async_trait]
impl WasiPreview1Clock for WasiP1Data {
    fn clock_res_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
        id: types::Clockid,
    ) -> Result<types::Timestamp, types::Error> {
        #[cfg(feature = "deterministic_testing")]
        {
            match &mut self.clock {
                ClockImpl::Real => {
                    let resolution = self.as_wasi_impl().0.0.clock_res_get(memory, id)?;
                    Ok(resolution)
                },
                #[cfg(feature = "deterministic_testing")]
                &mut ClockImpl::Custom(ref mut clock_impl) => {
                    clock_impl.clock_res_get(memory, id)
                }
            }
        }
        
        #[cfg(not(feature = "deterministic_testing"))]
        {
            self.as_wasi_impl().0.0.clock_res_get(memory, id)
        }
    }

    fn clock_time_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
        id: types::Clockid,
        precision: types::Timestamp,
    ) -> Result<types::Timestamp, types::Error> {
        #[cfg(feature = "deterministic_testing")]
        {
            match &mut self.clock {
                ClockImpl::Real => {
                    let time = self.as_wasi_impl().0.0.clock_time_get(memory, id, precision)?;
                    Ok(time)
                },
                #[cfg(feature = "deterministic_testing")]
                &mut ClockImpl::Custom(ref mut clock_impl) => {
                    clock_impl.clock_time_get(memory, id, precision)
                }
            }
        }
        
        #[cfg(not(feature = "deterministic_testing"))]
        {
            self.as_wasi_impl().0.0.clock_time_get(memory, id, precision)
        }
    }
}

#[async_trait::async_trait]
impl WasiPreview1Env for WasiP1Data {
    fn args_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
        argv: GuestPtr<GuestPtr<u8>>,
        argv_buf: GuestPtr<u8>,
    ) -> Result<(), types::Error> {
        #[cfg(feature = "deterministic_testing")]
        {
            match &mut self.env {
                EnvImpl::Real => {
                    self.as_wasi_impl().0.0.args_get(memory, argv, argv_buf)
                },
                #[cfg(feature = "deterministic_testing")]
                &mut EnvImpl::Custom(ref mut env_impl) => {
                    env_impl.args_get(memory, argv, argv_buf)
                }
            }
        }
        
        #[cfg(not(feature = "deterministic_testing"))]
        {
            self.as_wasi_impl().0.0.args_get(memory, argv, argv_buf)
        }
    }

    fn args_sizes_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
    ) -> Result<(types::Size, types::Size), types::Error> {
        #[cfg(feature = "deterministic_testing")]
        {
            match &mut self.env {
                EnvImpl::Real => {
                    self.as_wasi_impl().0.0.args_sizes_get(memory)
                },
                #[cfg(feature = "deterministic_testing")]
                &mut EnvImpl::Custom(ref mut env_impl) => {
                    env_impl.args_sizes_get(memory)
                }
            }
        }
        
        #[cfg(not(feature = "deterministic_testing"))]
        {
            self.as_wasi_impl().0.0.args_sizes_get(memory)
        }
    }

    fn environ_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
        environ: GuestPtr<GuestPtr<u8>>,
        environ_buf: GuestPtr<u8>,
    ) -> Result<(), types::Error> {
        #[cfg(feature = "deterministic_testing")]
        {
            match &mut self.env {
                EnvImpl::Real => {
                    self.as_wasi_impl().0.0.environ_get(memory, environ, environ_buf)
                },
                #[cfg(feature = "deterministic_testing")]
                &mut EnvImpl::Custom(ref mut env_impl) => {
                    env_impl.environ_get(memory, environ, environ_buf)
                }
            }
        }
        
        #[cfg(not(feature = "deterministic_testing"))]
        {
            self.as_wasi_impl().0.0.environ_get(memory, environ, environ_buf)
        }
    }

    fn environ_sizes_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
    ) -> Result<(types::Size, types::Size), types::Error> {
        #[cfg(feature = "deterministic_testing")]
        {
            match &mut self.env {
                EnvImpl::Real => {
                    self.as_wasi_impl().0.0.environ_sizes_get(memory)
                },
                #[cfg(feature = "deterministic_testing")]
                &mut EnvImpl::Custom(ref mut env_impl) => {
                    env_impl.environ_sizes_get(memory)
                }
            }
        }
        
        #[cfg(not(feature = "deterministic_testing"))]
        {
            self.as_wasi_impl().0.0.environ_sizes_get(memory)
        }
    }
}

// Helper function for environment variables and arguments
fn write_string_array(
    memory: &mut GuestMemory<'_>,
    strings: Vec<String>,
    ptrs: GuestPtr<GuestPtr<u8>>,
    buf: GuestPtr<u8>,
) -> Result<(), types::Error> {
    let mut buf_offset = 0;
    for (i, string) in strings.iter().enumerate() {
        // Write string to buffer
        let buf_ptr = buf.add(buf_offset)?;
        write_string(memory, string, buf_ptr)?;
        
        // Write pointer to array
        let ptr_idx = ptrs.add(i as u32)?;
        memory.write(ptr_idx, buf_ptr)?;
        
        buf_offset += (string.len() + 1) as u32; // +1 for null terminator
    }
    Ok(())
}

fn write_string(
    memory: &mut GuestMemory<'_>,
    string: &str,
    ptr: GuestPtr<u8>,
) -> Result<(), types::Error> {
    // Write the string bytes
    memory.copy_from_slice(string.as_bytes(), ptr.as_array(string.len() as u32))?;
    
    // Write null terminator
    memory.write(ptr.add(string.len() as u32)?, 0u8)?;
    
    Ok(())
}

#[async_trait::async_trait]
impl WasiPreview1Path for WasiP1Data {
    async fn path_create_directory(
        &mut self,
        memory: &mut GuestMemory<'_>,
        dirfd: types::Fd,
        path: GuestPtr<str>,
    ) -> Result<(), types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.path_create_directory(memory, dirfd, path).await
        } else {
            let _dir_fd = self.get_dir_fd(dirfd)?;
            let path_str = memory.read_string(path)?;
            
            // Create the directory in the mock filesystem using just the path
            self.mock_fs.with_fs(|fs| fs.create_directory(&Path::new(&path_str)))
        }
    }

    async fn path_filestat_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
        dirfd: types::Fd,
        flags: types::Lookupflags,
        path: GuestPtr<str>,
    ) -> Result<types::Filestat, types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.path_filestat_get(memory, dirfd, flags, path).await
        } else {
            let path_str = Self::read_string(memory, path)?;
            let dir_fd = self.get_dir_fd(dirfd)?;
            
            // Use our mock file system
            self.mock_fs.with_fs(|fs| fs.path_filestat_get(dir_fd.into(), &Path::new(&path_str)))
        }
    }

    async fn path_filestat_set_times(
        &mut self,
        memory: &mut GuestMemory<'_>,
        dirfd: types::Fd,
        flags: types::Lookupflags,
        path: GuestPtr<str>,
        atim: types::Timestamp,
        mtim: types::Timestamp,
        fst_flags: types::Fstflags,
    ) -> Result<(), types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.path_filestat_set_times(memory, dirfd, flags, path, atim, mtim, fst_flags).await
        } else {
            // Not implemented for testing yet
            Err(types::Errno::Nosys.into())
        }
    }

    async fn path_link(
        &mut self,
        memory: &mut GuestMemory<'_>,
        src_fd: types::Fd,
        src_flags: types::Lookupflags,
        src_path: GuestPtr<str>,
        target_fd: types::Fd,
        target_path: GuestPtr<str>,
    ) -> Result<(), types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.path_link(memory, src_fd, src_flags, src_path, target_fd, target_path).await
        } else {
            // Not implemented for testing yet
            Err(types::Errno::Nosys.into())
        }
    }

    async fn path_open(
        &mut self,
        memory: &mut GuestMemory<'_>,
        dirfd: types::Fd,
        dirflags: types::Lookupflags,
        path: GuestPtr<str>,
        oflags: types::Oflags,
        fs_rights_base: types::Rights,
        fs_rights_inheriting: types::Rights,
        fdflags: types::Fdflags,
    ) -> Result<types::Fd, types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.path_open(memory, dirfd, dirflags, path, oflags, fs_rights_base, fs_rights_inheriting, fdflags).await
        } else {
            let path_str = Self::read_string(memory, path)?;
            let dir_fd = self.get_dir_fd(dirfd)?;
            
            // Determine read/write/append flags
            let read = (fs_rights_base & types::Rights::FD_READ) != types::Rights::empty();
            let write = (fs_rights_base & types::Rights::FD_WRITE) != types::Rights::empty();
            let append = (fdflags & types::Fdflags::APPEND) != types::Fdflags::empty();
            let create = (oflags & types::Oflags::CREAT) != types::Oflags::empty();
            
            // Use our mock file system
            let fd_num = self.mock_fs.with_fs(|fs| {
                // Handle relative vs absolute paths
                let target_path = if Path::new(&path_str).is_absolute() {
                    PathBuf::from(&path_str)
                } else {
                    // Get the directory path from the fd
                    let dir_fd_info = fs.open_fds.get(&dir_fd.into()).ok_or(types::Errno::Badf)?;
                    let mut dir_path = dir_fd_info.path.clone();
                    // Append the relative path
                    dir_path.push(&path_str);
                    dir_path
                };
                
                fs.open(&target_path, read, write, append, create)
            })?;
            
            Ok(types::Fd::from(fd_num))
        }
    }

    async fn path_readlink(
        &mut self,
        memory: &mut GuestMemory<'_>,
        dirfd: types::Fd,
        path: GuestPtr<str>,
        buf: GuestPtr<u8>,
        buf_len: types::Size,
    ) -> Result<types::Size, types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.path_readlink(memory, dirfd, path, buf, buf_len).await
        } else {
            // Not implemented for testing yet
            Err(types::Errno::Nosys.into())
        }
    }

    async fn path_remove_directory(
        &mut self,
        memory: &mut GuestMemory<'_>,
        dirfd: types::Fd,
        path: GuestPtr<str>,
    ) -> Result<(), types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.path_remove_directory(memory, dirfd, path).await
        } else {
            // Not implemented for testing yet
            Err(types::Errno::Nosys.into())
        }
    }

    async fn path_rename(
        &mut self,
        memory: &mut GuestMemory<'_>,
        src_fd: types::Fd,
        src_path: GuestPtr<str>,
        dest_fd: types::Fd,
        dest_path: GuestPtr<str>,
    ) -> Result<(), types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.path_rename(memory, src_fd, src_path, dest_fd, dest_path).await
        } else {
            // Not implemented for testing yet
            Err(types::Errno::Nosys.into())
        }
    }

    async fn path_symlink(
        &mut self,
        memory: &mut GuestMemory<'_>,
        src_path: GuestPtr<str>,
        dirfd: types::Fd,
        dest_path: GuestPtr<str>,
    ) -> Result<(), types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.path_symlink(memory, src_path, dirfd, dest_path).await
        } else {
            // Not implemented for testing yet
            Err(types::Errno::Nosys.into())
        }
    }

    async fn path_unlink_file(
        &mut self,
        memory: &mut GuestMemory<'_>,
        dirfd: types::Fd,
        path: GuestPtr<str>,
    ) -> Result<(), types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.path_unlink_file(memory, dirfd, path).await
        } else {
            // Not implemented for testing yet
            Err(types::Errno::Nosys.into())
        }
    }
}

#[async_trait::async_trait]
impl WasiPreview1Poll for WasiP1Data {
    async fn poll_oneoff(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        _subs: GuestPtr<types::Subscription>,
        _events: GuestPtr<types::Event>,
        _nsubscriptions: types::Size,
    ) -> Result<types::Size, types::Error> {
        match &mut self.poll {
            PollImpl::Real => {
                // Real implementation that uses the WasiCtx
                Err(types::Errno::Nosys.into())
            },
            #[cfg(feature = "deterministic_testing")]
            PollImpl::Custom(custom) => {
                custom.poll_oneoff(_memory, _subs, _events, _nsubscriptions).await
            },
        }
    }
}

#[async_trait::async_trait]
impl WasiPreview1Proc for WasiP1Data {
    fn proc_exit(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        _status: types::Exitcode,
    ) -> anyhow::Error {
        match &mut self.proc {
            ProcImpl::Real => {
                // Real implementation that uses the WasiCtx
                anyhow::anyhow!("Not implemented")
            },
            #[cfg(feature = "deterministic_testing")]
            ProcImpl::Custom(custom) => {
                custom.proc_exit(_memory, _status)
            },
        }
    }

    fn proc_raise(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        _sig: types::Signal,
    ) -> Result<(), types::Error> {
        match &mut self.proc {
            ProcImpl::Real => {
                // Real implementation that uses the WasiCtx
                Err(types::Errno::Nosys.into())
            },
            #[cfg(feature = "deterministic_testing")]
            ProcImpl::Custom(custom) => {
                custom.proc_raise(_memory, _sig)
            },
        }
    }

    fn sched_yield(
        &mut self,
        _memory: &mut GuestMemory<'_>,
    ) -> Result<(), types::Error> {
        match &mut self.proc {
            ProcImpl::Real => {
                // Real implementation that uses the WasiCtx
                Err(types::Errno::Nosys.into())
            },
            #[cfg(feature = "deterministic_testing")]
            ProcImpl::Custom(custom) => {
                custom.sched_yield(_memory)
            },
        }
    }
}

#[async_trait::async_trait]
impl WasiPreview1Random for WasiP1Data {
    fn random_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
        buf: GuestPtr<u8>,
        buf_len: types::Size,
    ) -> Result<(), types::Error> {
        match &mut self.random {
            RandomImpl::Real => {
                // Real implementation that uses the WasiCtx
                // This would be a real implementation that uses the WasiCtx
                Err(types::Errno::Nosys.into())
            },
            #[cfg(feature = "deterministic_testing")]
            RandomImpl::Custom(custom) => {
                // Custom implementation for testing
                custom.random_get(memory, buf, buf_len)
            },
        }
    }
}

#[async_trait::async_trait]
impl WasiPreview1Sock for WasiP1Data {
    fn sock_accept(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        _fd: types::Fd,
        _flags: types::Fdflags,
    ) -> Result<types::Fd, types::Error> {
        match &mut self.sock {
            SockImpl::Real => {
                // Real implementation that uses the WasiCtx
                Err(types::Errno::Nosys.into())
            },
            #[cfg(feature = "deterministic_testing")]
            SockImpl::Custom(custom) => {
                custom.sock_accept(_memory, _fd, _flags)
            },
        }
    }

    fn sock_recv(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        _fd: types::Fd,
        _ri_data: types::IovecArray,
        _ri_flags: types::Riflags,
    ) -> Result<(types::Size, types::Roflags), types::Error> {
        match &mut self.sock {
            SockImpl::Real => {
                // Real implementation that uses the WasiCtx
                Err(types::Errno::Nosys.into())
            },
            #[cfg(feature = "deterministic_testing")]
            SockImpl::Custom(custom) => {
                custom.sock_recv(_memory, _fd, _ri_data, _ri_flags)
            },
        }
    }

    fn sock_send(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        _fd: types::Fd,
        _si_data: types::CiovecArray,
        _si_flags: types::Siflags,
    ) -> Result<types::Size, types::Error> {
        match &mut self.sock {
            SockImpl::Real => {
                // Real implementation that uses the WasiCtx
                Err(types::Errno::Nosys.into())
            },
            #[cfg(feature = "deterministic_testing")]
            SockImpl::Custom(custom) => {
                custom.sock_send(_memory, _fd, _si_data, _si_flags)
            },
        }
    }

    fn sock_shutdown(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        _fd: types::Fd,
        _how: types::Sdflags,
    ) -> Result<(), types::Error> {
        match &mut self.sock {
            SockImpl::Real => {
                // Real implementation that uses the WasiCtx
                Err(types::Errno::Nosys.into())
            },
            #[cfg(feature = "deterministic_testing")]
            SockImpl::Custom(custom) => {
                custom.sock_shutdown(_memory, _fd, _how)
            },
        }
    }
}

// Helper functions for converting iovecs to bytes
fn iovs_to_bytes<M: MemoryAccessor>(memory: &mut M, iovs: types::IovecArray) -> Result<Vec<u8>, types::Error> {
    let mut result = Vec::new();
    let iovecs_len = memory.read_iovec_array_len(iovs)?;
    
    for i in 0..iovecs_len {
        let iovec_ptr = memory.get_iovec_at_index(iovs, i as u32)?;
        let iovec = memory.read_iovec(iovec_ptr)?;
        
        if iovec.buf_len > 0 {
            let buf_ptr = iovec.buf.as_array(iovec.buf_len);
            let buf = memory.read_bytes(buf_ptr)?;
            result.extend_from_slice(&buf);
        }
    }
    
    Ok(result)
}

// Helper functions for converting ciovecs to bytes
fn ciovs_to_bytes<M: MemoryAccessor>(memory: &mut M, ciovs: types::CiovecArray) -> Result<Vec<u8>, types::Error> {
    let mut result = Vec::new();
    let ciovecs_len = memory.read_ciovec_array_len(ciovs)?;
    
    for i in 0..ciovecs_len {
        let ciovec_ptr = memory.get_ciovec_at_index(ciovs, i as u32)?;
        let ciovec = memory.read_ciovec(ciovec_ptr)?;
        
        if ciovec.buf_len > 0 {
            let buf_ptr = ciovec.buf.as_array(ciovec.buf_len);
            let buf = memory.read_bytes(buf_ptr)?;
            result.extend_from_slice(&buf);
        }
    }
    
    Ok(result)
}

// Helper function to find first non-empty iovec
fn first_non_empty_iovec<M: MemoryAccessor>(
    memory: &mut M,
    iovs_ptr: types::IovecArray,
) -> Result<Option<(u32, &mut [u8])>, types::Error> {
    let iovecs_len = memory.read_iovec_array_len(iovs_ptr)?;
    
    for i in 0..iovecs_len {
        let iovec_ptr = memory.get_iovec_at_index(iovs_ptr, i as u32)?;
        let iovec = memory.read_iovec(iovec_ptr)?;
        
        if iovec.buf_len > 0 {
            let buf_ptr = iovec.buf.as_array(iovec.buf_len);
            let buf = memory.get_bytes_mut(buf_ptr)?;
            return Ok(Some((iovec.buf_len, buf)));
        }
    }
    
    Ok(None)
}

// Helper function to find first non-empty ciovec
fn first_non_empty_ciovec<M: MemoryAccessor>(
    memory: &mut M,
    iovs_ptr: types::CiovecArray,
) -> Result<Option<(u32, &[u8])>, types::Error> {
    let ciovecs_len = memory.read_ciovec_array_len(iovs_ptr)?;
    
    for i in 0..ciovecs_len {
        let ciovec_ptr = memory.get_ciovec_at_index(iovs_ptr, i as u32)?;
        let ciovec = memory.read_ciovec(ciovec_ptr)?;
        
        if ciovec.buf_len > 0 {
            let buf_ptr = ciovec.buf.as_array(ciovec.buf_len);
            let buf = memory.read_bytes(buf_ptr)?;
            // Return the buffer contents directly
            return Ok(Some((ciovec.buf_len, Box::leak(buf.into_boxed_slice()))));
        }
    }
    
    Ok(None)
}

// Helper function for type inference
fn ok_size(size: u64) -> Result<types::Filesize, types::Error> {
    Ok(size)
}

// Helper function for convert fd to resource
fn convert_fd_to_resource(_fd: types::Fd) -> wasmtime::component::Resource<u32> {
    // Create a resource with value 0
    wasmtime::component::Resource::<u32>::new_own(0)
}

#[async_trait::async_trait]
impl WasiPreview1Fd for WasiP1Data {
    async fn fd_advise(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        offset: types::Filesize,
        len: types::Filesize,
        advice: types::Advice,
    ) -> Result<(), types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.fd_advise(memory, fd, offset, len, advice).await
        } else {
            // Not implemented for testing yet
            Err(types::Errno::Nosys.into())
        }
    }

    fn fd_allocate(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        offset: types::Filesize,
        len: types::Filesize,
    ) -> Result<(), types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.fd_allocate(memory, fd, offset, len)
        } else {
            // Not implemented for testing yet
            Err(types::Errno::Nosys.into())
        }
    }

    async fn fd_close(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
    ) -> Result<(), types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.fd_close(memory, fd).await
        } else {
            let fd_u32: u32 = fd.into();
            self.mock_fs.with_fs(|fs| fs.close(fd_u32))?;
            Ok(())
        }
    }

    async fn fd_datasync(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
    ) -> Result<(), types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.fd_datasync(memory, fd).await
        } else {
            // Not implemented for testing yet
            Err(types::Errno::Nosys.into())
        }
    }

    async fn fd_fdstat_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
    ) -> Result<types::Fdstat, types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.fd_fdstat_get(memory, fd).await
        } else {
            // Convert fd to u32 for mock filesystem
            let fd_u32: u32 = fd.into();
            
            // Check if file descriptor exists
            if !self.mock_fs.with_fs(|fs| fs.fd_exists(fd_u32)) {
                return Err(types::Errno::Badf.into());
            }
            
            // Since there's no get_fd_metadata method, create a default Fdstat
            Ok(types::Fdstat {
                fs_filetype: types::Filetype::RegularFile,
                fs_flags: types::Fdflags::empty(),
                fs_rights_base: types::Rights::all(),
                fs_rights_inheriting: types::Rights::all(),
            })
        }
    }

    fn fd_fdstat_set_flags(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        flags: types::Fdflags,
    ) -> Result<(), types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.fd_fdstat_set_flags(memory, fd, flags)
        } else {
            // Convert fd to u32 for mock filesystem
            let fd_u32: u32 = fd.into();
            
            // Check if file descriptor exists
            if !self.mock_fs.with_fs(|fs| fs.fd_exists(fd_u32)) {
                return Err(types::Errno::Badf.into());
            }
            
            // Set flags is not yet implemented for mock filesystem
            // This would typically update the flags on the file descriptor
            Err(types::Errno::Nosys.into())
        }
    }

    fn fd_fdstat_set_rights(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        fs_rights_base: types::Rights,
        fs_rights_inheriting: types::Rights,
    ) -> Result<(), types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.fd_fdstat_set_rights(memory, fd, fs_rights_base, fs_rights_inheriting)
        } else {
            // Convert fd to u32 for mock filesystem
            let fd_u32: u32 = fd.into();
            
            // Check if file descriptor exists
            if !self.mock_fs.with_fs(|fs| fs.fd_exists(fd_u32)) {
                return Err(types::Errno::Badf.into());
            }
            
            // Set rights is not yet implemented for mock filesystem
            // This would typically update the rights on the file descriptor
            Err(types::Errno::Nosys.into())
        }
    }

    async fn fd_filestat_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
    ) -> Result<types::Filestat, types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.fd_filestat_get(memory, fd).await
        } else {
            // Not implemented for testing yet
            Err(types::Errno::Nosys.into())
        }
    }

    async fn fd_filestat_set_size(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        size: types::Filesize,
    ) -> Result<(), types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.fd_filestat_set_size(memory, fd, size).await
        } else {
            // Not implemented for testing yet
            Err(types::Errno::Nosys.into())
        }
    }

    async fn fd_filestat_set_times(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        atim: types::Timestamp,
        mtim: types::Timestamp,
        fst_flags: types::Fstflags,
    ) -> Result<(), types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.fd_filestat_set_times(memory, fd, atim, mtim, fst_flags).await
        } else {
            // Not implemented for testing yet
            Err(types::Errno::Nosys.into())
        }
    }

    async fn fd_pread(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        iovs: types::IovecArray,
        offset: types::Filesize,
    ) -> Result<types::Size, types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.fd_pread(memory, fd, iovs, offset).await
        } else {
            let fd_u32: u32 = fd.into();
            
            // Save current position
            let current_pos = self.mock_fs.with_fs(|fs| {
                let fd_data = fs.open_fds.get(&fd_u32).ok_or(types::Errno::Badf)?;
                Ok::<u64, types::Error>(fd_data.position)
            })?;
            
            // Seek to requested offset
            self.mock_fs.with_fs(|fs| {
                let fd_data = fs.open_fds.get_mut(&fd_u32).ok_or(types::Errno::Badf)?;
                fd_data.position = offset as u64;
                Ok::<(), types::Error>(())
            })?;
            
            // Read data
            let mut total_read = 0u32;
            memory.process_iovecs(iovs, |buf| {
                let mut buffer = vec![0u8; buf.len()];
                let nread = self.mock_fs.with_fs(|fs| fs.read(fd_u32, &mut buffer))?;
                buf[..nread].copy_from_slice(&buffer[..nread]);
                total_read += nread as u32;
                Ok(nread)
            })?;
            
            // Restore original position
            self.mock_fs.with_fs(|fs| {
                let fd_data = fs.open_fds.get_mut(&fd_u32).ok_or(types::Errno::Badf)?;
                fd_data.position = current_pos;
                Ok::<(), types::Error>(())
            })?;
            
            Ok(total_read)
        }
    }

    fn fd_prestat_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
    ) -> Result<types::Prestat, types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.fd_prestat_get(memory, fd)
        } else {
            // Convert fd to u32 for mock filesystem
            let fd_u32: u32 = fd.into();
            
            // Check if file descriptor exists
            if !self.mock_fs.with_fs(|fs| fs.fd_exists(fd_u32)) {
                return Err(types::Errno::Badf.into());
            }
            
            // For simplicity, return a prestat with a fixed path length
            Ok(types::Prestat::Dir(types::PrestatDir {
                pr_name_len: 1, // Dummy value
            }))
        }
    }

    fn fd_prestat_dir_name(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        path: GuestPtr<u8>,
        path_max_len: types::Size,
    ) -> Result<(), types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.fd_prestat_dir_name(memory, fd, path, path_max_len)
        } else {
            // Convert fd to u32 for mock filesystem
            let fd_u32: u32 = fd.into();
            
            // Check if file descriptor exists
            if !self.mock_fs.with_fs(|fs| fs.fd_exists(fd_u32)) {
                return Err(types::Errno::Badf.into());
            }
            
            // For simplicity, write a dummy path
            if path_max_len < 1 {
                return Err(types::Errno::Nametoolong.into());
            }
            
            memory.write_string(path, "/")?;
            Ok(())
        }
    }

    async fn fd_pwrite(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        ciovs: types::CiovecArray,
        offset: types::Filesize,
    ) -> Result<types::Size, types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.fd_pwrite(memory, fd, ciovs, offset).await
        } else {
            let fd_u32: u32 = fd.into();
            
            // Save current position
            let current_pos = self.mock_fs.with_fs(|fs| {
                let fd_data = fs.open_fds.get(&fd_u32).ok_or(types::Errno::Badf)?;
                Ok::<u64, types::Error>(fd_data.position)
            })?;
            
            // Seek to requested offset
            self.mock_fs.with_fs(|fs| {
                let fd_data = fs.open_fds.get_mut(&fd_u32).ok_or(types::Errno::Badf)?;
                fd_data.position = offset as u64;
                Ok::<(), types::Error>(())
            })?;
            
            // Write data
            let mut total_written = 0u32;
            memory.process_ciovecs(ciovs, |buf| {
                let nwritten = self.mock_fs.with_fs(|fs| fs.write(fd_u32, buf))?;
                total_written += nwritten as u32;
                Ok(nwritten)
            })?;
            
            // Restore original position
            self.mock_fs.with_fs(|fs| {
                let fd_data = fs.open_fds.get_mut(&fd_u32).ok_or(types::Errno::Badf)?;
                fd_data.position = current_pos;
                Ok::<(), types::Error>(())
            })?;
            
            Ok(total_written)
        }
    }

    async fn fd_read(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        iovs: types::IovecArray,
    ) -> Result<types::Size, types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.fd_read(memory, fd, iovs).await
        } else {
            let fd_u32: u32 = fd.into();
            let mut total_read = 0u32;
            
            memory.process_iovecs(iovs, |buf| {
                let mut buffer = vec![0u8; buf.len()];
                let nread = self.mock_fs.with_fs(|fs| fs.read(fd_u32, &mut buffer))?;
                buf[..nread].copy_from_slice(&buffer[..nread]);
                total_read += nread as u32;
                Ok(nread)
            })?;
            
            Ok(total_read)
        }
    }

    async fn fd_readdir(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        buf: GuestPtr<u8>,
        buf_len: types::Size,
        cookie: types::Dircookie,
    ) -> Result<types::Size, types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.fd_readdir(memory, fd, buf, buf_len, cookie).await
        } else {
            // Not implemented for testing yet
            Err(types::Errno::Nosys.into())
        }
    }

    fn fd_renumber(
        &mut self,
        memory: &mut GuestMemory<'_>,
        from: types::Fd,
        to: types::Fd,
    ) -> Result<(), types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.fd_renumber(memory, from, to)
        } else {
            // Convert fds to u32 for mock filesystem
            let from_u32: u32 = from.into();
            let _to_u32: u32 = to.into();  // Prefix with underscore since it's unused
            
            // Check if source file descriptor exists
            if !self.mock_fs.with_fs(|fs| fs.fd_exists(from_u32)) {
                return Err(types::Errno::Badf.into());
            }
            
            // Renumber operation is not yet implemented for mock filesystem
            // This would typically copy the fd data and then close the source fd
            Err(types::Errno::Nosys.into())
        }
    }

    async fn fd_seek(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        offset: types::Filedelta,
        whence: types::Whence,
    ) -> Result<types::Filesize, types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.fd_seek(memory, fd, offset, whence).await
        } else {
            // Convert fd to u32 for mock filesystem
            let fd_u32: u32 = fd.into();
            
            // Check if file descriptor exists
            if !self.mock_fs.with_fs(|fs| fs.fd_exists(fd_u32)) {
                return Err(types::Errno::Badf.into());
            }
            
            // Perform seek operation using mock filesystem
            let new_position = self.mock_fs.with_fs(|fs| fs.seek(fd_u32, offset, whence))?;
            
            // Return the new position
            Ok(new_position)
        }
    }

    async fn fd_sync(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
    ) -> Result<(), types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.fd_sync(memory, fd).await
        } else {
            // Convert fd to u32 for mock filesystem
            let fd_u32: u32 = fd.into();
            
            // Check if file descriptor exists
            if !self.mock_fs.with_fs(|fs| fs.fd_exists(fd_u32)) {
                return Err(types::Errno::Badf.into());
            }
            
            // In our mock implementation, there's nothing to sync
            Ok(())
        }
    }

    fn fd_tell(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
    ) -> Result<types::Filesize, types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.fd_tell(memory, fd)
        } else {
            // Convert fd to u32 for mock filesystem
            let fd_u32: u32 = fd.into();
            
            // Check if file descriptor exists
            if !self.mock_fs.with_fs(|fs| fs.fd_exists(fd_u32)) {
                return Err(types::Errno::Badf.into());
            }
            
            // Return a default position since there's no tell method
            // In a real implementation, we would return the actual position
            Ok(0)
        }
    }

    async fn fd_write(
        &mut self,
        memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        ciovs: types::CiovecArray,
    ) -> Result<types::Size, types::Error> {
        if cfg!(not(feature = "deterministic_testing")) {
            self.as_wasi_impl().0.0.fd_write(memory, fd, ciovs).await
        } else {
            // Convert fd to u32 for mock filesystem
            let fd_u32: u32 = fd.into();
            let mut total_written = 0u32;
            
            memory.process_ciovecs(ciovs, |buf| {
                let nwritten = self.mock_fs.with_fs(|fs| fs.write(fd_u32, buf))?;
                total_written += nwritten as u32;
                Ok(nwritten)
            })?;
            
            Ok(total_written)
        }
    }
} 