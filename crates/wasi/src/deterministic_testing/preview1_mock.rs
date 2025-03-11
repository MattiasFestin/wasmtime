use crate::ctx::WasiCtx;
use crate::deterministic_testing::fd_wrapper::FdWrapper;
use crate::preview1::types;
use crate::traits::*;
use wiggle::GuestMemory;
use wiggle::GuestPtr;
use std::sync::{Arc, Mutex};
use crate::deterministic_testing::MockFilesystem;
use crate::deterministic_testing::mock_file_system::SharedMockFileSystem;

/// A fully deterministic mock implementation of the WASI Preview1 API for testing.
/// This implementation records all calls and returns configurable deterministic results.
pub struct MockWasiPreview1 {
    /// Records of all calls made to this mock
    call_records: Arc<Mutex<CallRecords>>,
    
    /// Deterministic responses for clock operations
    clock_responses: Vec<(types::Clockid, types::Timestamp)>,
    
    /// Deterministic data for random operations
    random_data: Vec<u8>,
    
    /// Arguments for the program
    args: Vec<String>,
    
    /// Environment variables
    env_vars: Vec<(String, String)>,
    
    /// Current time for each clock
    clock_times: Vec<(types::Clockid, types::Timestamp)>,
    
    /// Clock resolutions
    clock_resolutions: Vec<(types::Clockid, types::Timestamp)>,

    /// Mock filesystem for handling file descriptors
    filesystem: MockFilesystem,

    /// Add the shared mock file system
    fs: SharedMockFileSystem,
}

/// Metadata for a mock file descriptor
#[derive(Debug, Clone)]
struct FdMetadata {
    /// Path associated with this file descriptor (if any)
    path: Option<String>,
    /// Type of file
    filetype: types::Filetype,
    /// Current position in the file
    position: u64,
    /// Size of the file
    size: u64,
}

/// Records of all calls made to the mock
#[derive(Default)]
pub struct CallRecords {
    /// Records of args_get calls
    pub args_get: Vec<()>,
    
    /// Records of args_sizes_get calls
    pub args_sizes_get: Vec<()>,
    
    /// Records of environ_get calls
    pub environ_get: Vec<()>,
    
    /// Records of environ_sizes_get calls
    pub environ_sizes_get: Vec<()>,
    
    /// Records of clock_res_get calls
    pub clock_res_get: Vec<types::Clockid>,
    
    /// Records of clock_time_get calls
    pub clock_time_get: Vec<(types::Clockid, types::Timestamp)>,
    
    /// Records of random_get calls
    pub random_get: Vec<types::Size>,
    
    // Add more records as needed
}

impl MockWasiPreview1 {
    /// Create a new mock with default settings
    pub fn new(_wasi: WasiCtx) -> Self {
        let mut clock_times = Vec::new();
        clock_times.push((types::Clockid::Realtime, 1_000_000_000));
        clock_times.push((types::Clockid::Monotonic, 1_000_000_000));
        
        let mut clock_resolutions = Vec::new();
        clock_resolutions.push((types::Clockid::Realtime, 1_000));
        clock_resolutions.push((types::Clockid::Monotonic, 1_000));
        
        Self {
            call_records: Arc::new(Mutex::new(CallRecords::default())),
            clock_responses: Vec::new(),
            random_data: vec![0, 1, 2, 3, 4, 5, 6, 7],
            args: vec!["program".to_string()],
            env_vars: vec![("PATH".to_string(), "/usr/bin".to_string())],
            clock_times,
            clock_resolutions,
            filesystem: MockFilesystem::new(),
            fs: SharedMockFileSystem::new(),
        }
    }
    
    /// Set a deterministic response for a clock operation
    pub fn set_clock_time(&mut self, id: types::Clockid, time: types::Timestamp) {
        // Remove any existing entry for this clock ID
        self.clock_responses.retain(|(clock_id, _)| *clock_id != id);
        // Add the new entry
        self.clock_responses.push((id, time));
        // Also update the current time
        self.clock_times.retain(|(clock_id, _)| *clock_id != id);
        self.clock_times.push((id, time));
    }
    
    /// Set the resolution for a clock
    pub fn set_clock_resolution(&mut self, id: types::Clockid, resolution: types::Timestamp) {
        self.clock_resolutions.retain(|(clock_id, _)| *clock_id != id);
        self.clock_resolutions.push((id, resolution));
    }
    
    /// Advance all clocks by the specified amount
    pub fn tick(&mut self, delta: types::Timestamp) {
        for (_, time) in self.clock_times.iter_mut() {
            *time += delta;
        }
    }
    
    /// Set deterministic data for random operations
    pub fn set_random_data(&mut self, data: Vec<u8>) {
        self.random_data = data;
    }
    
    /// Set program arguments
    pub fn set_args(&mut self, args: Vec<String>) {
        self.args = args;
    }
    
    /// Set environment variables
    pub fn set_env_vars(&mut self, env_vars: Vec<(String, String)>) {
        self.env_vars = env_vars;
    }
    
    /// Get a clone of the call records
    pub fn get_call_records(&self) -> Arc<Mutex<CallRecords>> {
        self.call_records.clone()
    }
    
    /// Create a new file descriptor and return it
    fn create_fd(&mut self, path: Option<String>, filetype: types::Filetype) -> FdWrapper {
        self.filesystem.create_fd(path, filetype)
    }
    
    /// Check if a file descriptor exists
    fn fd_exists(&self, fd: FdWrapper) -> bool {
        if let Some(mock_fd) = fd.as_mock() {
            self.filesystem.fd_exists(mock_fd)
        } else {
            false
        }
    }
    
    /// Get metadata for a file descriptor
    fn get_fd_metadata(&self, fd: FdWrapper) -> Option<&crate::deterministic_testing::FdMetadata> {
        if let Some(mock_fd) = fd.as_mock() {
            self.filesystem.get_fd_metadata(mock_fd)
        } else {
            None
        }
    }
    
    /// Close a file descriptor
    fn close_fd(&mut self, fd: FdWrapper) -> Result<(), types::Error> {
        if let Some(mock_fd) = fd.as_mock() {
            self.filesystem.close_fd(mock_fd)
        } else {
            Err(types::Error::from(types::Errno::Badf))
        }
    }
    
    /// Helper function to read a string from guest memory
    fn read_string(&self, memory: &GuestMemory<'_>, ptr: GuestPtr<str>) -> Result<String, types::Error> {
        match memory.as_str(ptr) {
            Ok(Some(s)) => Ok(s.to_string()),
            Ok(None) => Ok("".to_string()),
            Err(_) => Err(types::Error::from(types::Errno::Inval)),
        }
    }

    // Add some helper methods to manipulate the file system
    pub fn create_file(&self, path: &str, content: Vec<u8>) -> Result<(), types::Error> {
        self.fs.with_fs(|fs| fs.create_file(&std::path::Path::new(path), content))
    }

    pub fn create_directory(&self, path: &str) -> Result<(), types::Error> {
        self.fs.with_fs(|fs| fs.create_directory(&std::path::Path::new(path)))
    }

    pub fn add_preopen_dir(&self, path: &str) -> Result<u32, types::Error> {
        self.fs.with_fs(|fs| fs.add_preopen_dir(&std::path::Path::new(path)))
    }

    // Access to the shared file system
    pub fn fs(&self) -> &SharedMockFileSystem {
        &self.fs
    }
}

#[async_trait::async_trait]
impl WasiPreview1Env for MockWasiPreview1 {
    fn args_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
        argv: GuestPtr<GuestPtr<u8>>,
        argv_buf: GuestPtr<u8>,
    ) -> Result<(), types::Error> {
        self.call_records.lock().unwrap().args_get.push(());
        
        // Write the arguments to guest memory
        let mut next_argv = argv;
        let mut next_argv_buf = argv_buf;
        
        for arg in &self.args {
            // Write the argument string to argv_buf
            let arg_bytes = arg.as_bytes();
            let mut ptr = next_argv_buf;
            
            for &byte in arg_bytes {
                memory.write(ptr, byte)?;
                ptr = ptr.add(1)?;
            }
            
            // Write null terminator
            memory.write(ptr, 0u8)?;
            ptr = ptr.add(1)?;
            
            // Write the pointer to the argument string to argv
            memory.write(next_argv, next_argv_buf)?;
            
            // Update pointers for next iteration
            next_argv = next_argv.add(1)?;
            next_argv_buf = ptr;
        }
        
        Ok(())
    }

    fn args_sizes_get(
        &mut self,
        _memory: &mut GuestMemory<'_>,
    ) -> Result<(types::Size, types::Size), types::Error> {
        self.call_records.lock().unwrap().args_sizes_get.push(());
        
        // Calculate sizes
        let argc = self.args.len() as types::Size;
        let argv_buf_size = self.args.iter()
            .map(|arg| arg.len() + 1) // +1 for null terminator
            .sum::<usize>() as types::Size;
        
        Ok((argc, argv_buf_size))
    }

    fn environ_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
        environ: GuestPtr<GuestPtr<u8>>,
        environ_buf: GuestPtr<u8>,
    ) -> Result<(), types::Error> {
        self.call_records.lock().unwrap().environ_get.push(());
        
        // Write the environment variables to guest memory
        let mut next_environ = environ;
        let mut next_environ_buf = environ_buf;
        
        for (key, value) in &self.env_vars {
            // Format as KEY=VALUE
            let env_var = format!("{}={}", key, value);
            let env_bytes = env_var.as_bytes();
            let mut ptr = next_environ_buf;
            
            for &byte in env_bytes {
                memory.write(ptr, byte)?;
                ptr = ptr.add(1)?;
            }
            
            // Write null terminator
            memory.write(ptr, 0u8)?;
            ptr = ptr.add(1)?;
            
            // Write the pointer to the environment variable to environ
            memory.write(next_environ, next_environ_buf)?;
            
            // Update pointers for next iteration
            next_environ = next_environ.add(1)?;
            next_environ_buf = ptr;
        }
        
        Ok(())
    }

    fn environ_sizes_get(
        &mut self,
        _memory: &mut GuestMemory<'_>,
    ) -> Result<(types::Size, types::Size), types::Error> {
        self.call_records.lock().unwrap().environ_sizes_get.push(());
        
        // Calculate sizes
        let environ_count = self.env_vars.len() as types::Size;
        let environ_buf_size = self.env_vars.iter()
            .map(|(key, value)| key.len() + value.len() + 2) // +2 for '=' and null terminator
            .sum::<usize>() as types::Size;
        
        Ok((environ_count, environ_buf_size))
    }
}

#[async_trait::async_trait]
impl WasiPreview1Clock for MockWasiPreview1 {
    fn clock_res_get(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        id: types::Clockid,
    ) -> Result<types::Timestamp, types::Error> {
        self.call_records.lock().unwrap().clock_res_get.push(id);
        
        // Return the configured resolution for this clock
        for (clock_id, resolution) in &self.clock_resolutions {
            if *clock_id == id {
                return Ok(*resolution);
            }
        }
        
        // Default resolution if not found
        Err(types::Error::from(types::Errno::Inval))
    }

    fn clock_time_get(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        id: types::Clockid,
        _precision: types::Timestamp,
    ) -> Result<types::Timestamp, types::Error> {
        self.call_records.lock().unwrap().clock_time_get.push((id, _precision));
        
        // If we have a deterministic response, use it
        for (clock_id, time) in &self.clock_responses {
            if *clock_id == id {
                return Ok(*time);
            }
        }
        
        // Otherwise use the current time for this clock
        for (clock_id, time) in &self.clock_times {
            if *clock_id == id {
            return Ok(*time);
            }
        }
        
        // Default time if not found
        Err(types::Error::from(types::Errno::Inval))
    }
}

#[async_trait::async_trait]
impl WasiPreview1Random for MockWasiPreview1 {
    fn random_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
        buf: GuestPtr<u8>,
        buf_len: types::Size,
    ) -> Result<(), types::Error> {
        self.call_records.lock().unwrap().random_get.push(buf_len);
        
        // Write deterministic random data to the buffer
            let len = std::cmp::min(buf_len as usize, self.random_data.len());
            let data = &self.random_data[..len];
            
            // Write the data to the guest memory
            let mut ptr = buf;
            for &byte in data {
                memory.write(ptr, byte)?;
                ptr = ptr.add(1)?;
            }
            
        // If we need more data than we have, cycle through the data
        if buf_len as usize > len {
            let mut remaining = buf_len as usize - len;
            while remaining > 0 {
                let cycle_len = std::cmp::min(remaining, self.random_data.len());
                let cycle_data = &self.random_data[..cycle_len];
                
                for &byte in cycle_data {
                    memory.write(ptr, byte)?;
                    ptr = ptr.add(1)?;
                }
                
                remaining -= cycle_len;
            }
        }
        
        Ok(())
    }
}

// Implement stubs for other traits
// These are minimal implementations that just record the calls
// You can expand these as needed for your testing

#[async_trait::async_trait]
impl WasiPreview1Fd for MockWasiPreview1 {
    async fn fd_advise(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        _offset: types::Filesize,
        _len: types::Filesize,
        _advice: types::Advice,
    ) -> Result<(), types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(())
    }

    fn fd_allocate(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        _offset: types::Filesize,
        _len: types::Filesize,
    ) -> Result<(), types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(())
    }

    async fn fd_close(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
    ) -> Result<(), types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        self.close_fd(fd_wrapper)
    }

    async fn fd_datasync(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
    ) -> Result<(), types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(())
    }

    async fn fd_fdstat_get(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
    ) -> Result<types::Fdstat, types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        
        // Check if the file descriptor exists
        if let Some(metadata) = self.get_fd_metadata(fd_wrapper) {
            Ok(types::Fdstat {
                fs_filetype: metadata.filetype,
                fs_flags: types::Fdflags::empty(),
                fs_rights_base: types::Rights::all(),
                fs_rights_inheriting: types::Rights::all(),
            })
        } else {
            Err(types::Error::from(types::Errno::Badf))
        }
    }

    fn fd_fdstat_set_flags(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        _flags: types::Fdflags,
    ) -> Result<(), types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(())
    }

    fn fd_fdstat_set_rights(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        _fs_rights_base: types::Rights,
        _fs_rights_inheriting: types::Rights,
    ) -> Result<(), types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(())
    }

    async fn fd_filestat_get(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
    ) -> Result<types::Filestat, types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        
        // Check if the file descriptor exists
        if let Some(metadata) = self.get_fd_metadata(fd_wrapper) {
            Ok(types::Filestat {
                dev: 1,
                ino: 1,
                filetype: metadata.filetype,
                nlink: 1,
                size: metadata.size,
                atim: 0,
                mtim: 0,
                ctim: 0,
            })
        } else {
            Err(types::Error::from(types::Errno::Badf))
        }
    }

    async fn fd_filestat_set_size(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        _size: types::Filesize,
    ) -> Result<(), types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(())
    }

    async fn fd_filestat_set_times(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        _atim: types::Timestamp,
        _mtim: types::Timestamp,
        _fst_flags: types::Fstflags,
    ) -> Result<(), types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(())
    }

    async fn fd_read(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        _iovs: types::IovecArray,
    ) -> Result<types::Size, types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(0)
    }

    async fn fd_pread(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        _iovs: types::IovecArray,
        _offset: types::Filesize,
    ) -> Result<types::Size, types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(0)
    }

    async fn fd_write(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        _ciovs: types::CiovecArray,
    ) -> Result<types::Size, types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(0)
    }

    async fn fd_pwrite(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        _ciovs: types::CiovecArray,
        _offset: types::Filesize,
    ) -> Result<types::Size, types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(0)
    }

    fn fd_prestat_get(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
    ) -> Result<types::Prestat, types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Err(types::Error::from(types::Errno::Badf))
    }

    fn fd_prestat_dir_name(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        _path: GuestPtr<u8>,
        _path_max_len: types::Size,
    ) -> Result<(), types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Err(types::Error::from(types::Errno::Badf))
    }

    fn fd_renumber(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        from: types::Fd,
        to: types::Fd,
    ) -> Result<(), types::Error> {
        // Convert to FdWrapper
        let from_wrapper = FdWrapper::from_real(from);
        let to_wrapper = FdWrapper::from_real(to);
        
        if !self.fd_exists(from_wrapper) || !self.fd_exists(to_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(())
    }

    async fn fd_seek(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        _offset: types::Filedelta,
        _whence: types::Whence,
    ) -> Result<types::Filesize, types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(0)
    }

    async fn fd_sync(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
    ) -> Result<(), types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(())
    }

    fn fd_tell(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
    ) -> Result<types::Filesize, types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(0)
    }

    async fn fd_readdir(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        _buf: GuestPtr<u8>,
        _buf_len: types::Size,
        _cookie: types::Dircookie,
    ) -> Result<types::Size, types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(0)
    }
}

#[async_trait::async_trait]
impl WasiPreview1Path for MockWasiPreview1 {
    async fn path_create_directory(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        dirfd: types::Fd,
        _path: GuestPtr<str>,
    ) -> Result<(), types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(dirfd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(())
    }

    async fn path_filestat_get(
        &mut self,
        memory: &mut GuestMemory<'_>,
        dirfd: types::Fd,
        _flags: types::Lookupflags,
        path: GuestPtr<str>,
    ) -> Result<types::Filestat, types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(dirfd);
        
        // Check if the directory file descriptor exists
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        
        // Try to read the path string from guest memory
        let _path_str = self.read_string(memory, path)?;
        
        // For mock purposes, create a fake file stat
        // In a real implementation, you might want to track files by path
        Ok(types::Filestat {
            dev: 1,
            ino: 1,
            filetype: types::Filetype::RegularFile,
            nlink: 1,
            size: 0,
            atim: 0,
            mtim: 0,
            ctim: 0,
        })
    }

    async fn path_filestat_set_times(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        dirfd: types::Fd,
        _flags: types::Lookupflags,
        _path: GuestPtr<str>,
        _atim: types::Timestamp,
        _mtim: types::Timestamp,
        _fst_flags: types::Fstflags,
    ) -> Result<(), types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(dirfd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(())
    }

    async fn path_link(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        src_fd: types::Fd,
        _src_flags: types::Lookupflags,
        _src_path: GuestPtr<str>,
        target_fd: types::Fd,
        _target_path: GuestPtr<str>,
    ) -> Result<(), types::Error> {
        // Convert to FdWrapper
        let src_fd_wrapper = FdWrapper::from_real(src_fd);
        let target_fd_wrapper = FdWrapper::from_real(target_fd);
        
        if !self.fd_exists(src_fd_wrapper) || !self.fd_exists(target_fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(())
    }

    async fn path_open(
        &mut self,
        memory: &mut GuestMemory<'_>,
        dirfd: types::Fd,
        _dirflags: types::Lookupflags,
        path: GuestPtr<str>,
        _oflags: types::Oflags,
        _fs_rights_base: types::Rights,
        _fs_rights_inheriting: types::Rights,
        _fdflags: types::Fdflags,
    ) -> Result<types::Fd, types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(dirfd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        
        // Try to read the path string from guest memory
        let path_str = self.read_string(memory, path)?;
        
        // Create a new file descriptor for this path
        let new_fd = self.create_fd(Some(path_str), types::Filetype::RegularFile);
        
        // Convert back to types::Fd
        if let Some(real_fd) = new_fd.as_real() {
            Ok(real_fd)
        } else if let Some(mock_fd) = new_fd.as_mock() {
            // This is a mock FD, but we need to return a real FD
            // For simplicity, we'll just use the mock_fd value as the real_fd
            // In a real implementation, you might want to maintain a mapping
            Ok(types::Fd::from(mock_fd))
        } else {
            Err(types::Error::from(types::Errno::Badf))
        }
    }

    async fn path_readlink(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        dirfd: types::Fd,
        _path: GuestPtr<str>,
        _buf: GuestPtr<u8>,
        _buf_len: types::Size,
    ) -> Result<types::Size, types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(dirfd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(0)
    }

    async fn path_remove_directory(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        dirfd: types::Fd,
        _path: GuestPtr<str>,
    ) -> Result<(), types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(dirfd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(())
    }

    async fn path_rename(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        src_fd: types::Fd,
        _src_path: GuestPtr<str>,
        dest_fd: types::Fd,
        _dest_path: GuestPtr<str>,
    ) -> Result<(), types::Error> {
        // Convert to FdWrapper
        let src_fd_wrapper = FdWrapper::from_real(src_fd);
        let dest_fd_wrapper = FdWrapper::from_real(dest_fd);
        
        if !self.fd_exists(src_fd_wrapper) || !self.fd_exists(dest_fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(())
    }

    async fn path_symlink(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        _src_path: GuestPtr<str>,
        dirfd: types::Fd,
        _dest_path: GuestPtr<str>,
    ) -> Result<(), types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(dirfd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(())
    }

    async fn path_unlink_file(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        dirfd: types::Fd,
        _path: GuestPtr<str>,
    ) -> Result<(), types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(dirfd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl WasiPreview1Poll for MockWasiPreview1 {
    async fn poll_oneoff(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        _subs: GuestPtr<types::Subscription>,
        _events: GuestPtr<types::Event>,
        _nsubscriptions: types::Size,
    ) -> Result<types::Size, types::Error> {
        Ok(0) // No events
    }
}

#[async_trait::async_trait]
impl WasiPreview1Proc for MockWasiPreview1 {
    fn proc_exit(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        status: types::Exitcode,
    ) -> anyhow::Error {
        anyhow::anyhow!("exit with status {}", status)
    }

    fn proc_raise(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        _sig: types::Signal,
    ) -> Result<(), types::Error> {
        Ok(())
    }

    fn sched_yield(
        &mut self,
        _memory: &mut GuestMemory<'_>,
    ) -> Result<(), types::Error> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl WasiPreview1Sock for MockWasiPreview1 {
    fn sock_accept(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        _flags: types::Fdflags,
    ) -> Result<types::Fd, types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        
        // Check if the socket file descriptor exists
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        
        // For mock purposes, create a new file descriptor for the accepted connection
        let new_fd = self.create_fd(None, types::Filetype::SocketStream);
        
        // Convert back to types::Fd
        if let Some(real_fd) = new_fd.as_real() {
            Ok(real_fd)
        } else if let Some(mock_fd) = new_fd.as_mock() {
            // This is a mock FD, but we need to return a real FD
            // For simplicity, we'll just use the mock_fd value as the real_fd
            Ok(types::Fd::from(mock_fd))
        } else {
            Err(types::Error::from(types::Errno::Badf))
        }
    }

    fn sock_recv(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        _ri_data: types::IovecArray,
        _ri_flags: types::Riflags,
    ) -> Result<(types::Size, types::Roflags), types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Err(types::Error::from(types::Errno::Badf))
    }

    fn sock_send(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        _si_data: types::CiovecArray,
        _si_flags: types::Siflags,
    ) -> Result<types::Size, types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Err(types::Error::from(types::Errno::Badf))
    }

    fn sock_shutdown(
        &mut self,
        _memory: &mut GuestMemory<'_>,
        fd: types::Fd,
        _how: types::Sdflags,
    ) -> Result<(), types::Error> {
        // Convert to FdWrapper
        let fd_wrapper = FdWrapper::from_real(fd);
        if !self.fd_exists(fd_wrapper) {
            return Err(types::Error::from(types::Errno::Badf));
        }
        Err(types::Error::from(types::Errno::Badf))
    }
}

// Implement the combined trait
#[async_trait::async_trait]
impl WasiPreview1Api for MockWasiPreview1 {} 