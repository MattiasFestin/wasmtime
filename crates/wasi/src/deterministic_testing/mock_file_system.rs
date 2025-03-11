use crate::preview1::types;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

// Constants for file flags as u16 values
const FDFLAGS_APPEND: u16 = 0x0001;
const FDFLAGS_DSYNC: u16 = 0x0002;
const FDFLAGS_NONBLOCK: u16 = 0x0004;
const FDFLAGS_RSYNC: u16 = 0x0008;
const FDFLAGS_SYNC: u16 = 0x0010;
const FDFLAGS_WRITE: u16 = 0x0020;

/// Represents a file or directory in the mock file system
#[derive(Debug, Clone)]
pub enum FsNode {
    File {
        content: Vec<u8>,
        created: u64,
        modified: u64,
        accessed: u64,
    },
    Directory {
        created: u64,
        modified: u64,
        accessed: u64,
    },
}

impl FsNode {
    pub fn is_file(&self) -> bool {
        matches!(self, FsNode::File { .. })
    }

    pub fn is_directory(&self) -> bool {
        matches!(self, FsNode::Directory { .. })
    }

    pub fn created(&self) -> u64 {
        match self {
            FsNode::File { created, .. } => *created,
            FsNode::Directory { created, .. } => *created,
        }
    }

    pub fn modified(&self) -> u64 {
        match self {
            FsNode::File { modified, .. } => *modified,
            FsNode::Directory { modified, .. } => *modified,
        }
    }

    pub fn accessed(&self) -> u64 {
        match self {
            FsNode::File { accessed, .. } => *accessed,
            FsNode::Directory { accessed, .. } => *accessed,
        }
    }

    pub fn update_modified(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
            
        match self {
            FsNode::File { modified, .. } => {
                *modified = now;
            }
            FsNode::Directory { modified, .. } => {
                *modified = now;
            }
        }
    }

    pub fn update_accessed(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
            
        match self {
            FsNode::File { accessed, .. } => {
                *accessed = now;
            }
            FsNode::Directory { accessed, .. } => {
                *accessed = now;
            }
        }
    }
}

/// A simple file descriptor for the mock file system
#[derive(Debug, Clone)]
pub struct MockFd {
    pub path: PathBuf,
    pub position: u64,
    pub append: bool,
    pub read: bool,
    pub write: bool,
}

/// A simple in-memory file system for testing
#[derive(Debug, Default)]
pub struct MockFileSystem {
    nodes: HashMap<PathBuf, FsNode>,
    pub open_fds: HashMap<u32, MockFd>,
    next_fd: u32,
    cwd: PathBuf,
    preopen_dirs: HashSet<PathBuf>,
}

impl MockFileSystem {
    pub fn new() -> Self {
        let mut fs = Self {
            nodes: HashMap::new(),
            open_fds: HashMap::new(),
            next_fd: 3, // Start at 3 (after stdin/stdout/stderr)
            cwd: PathBuf::from("/"),
            preopen_dirs: HashSet::new(),
        };

        // Create root directory
        fs.create_directory(&PathBuf::from("/")).unwrap();
        
        fs
    }

    pub fn create_directory(&mut self, path: &Path) -> Result<(), types::Error> {
        if self.nodes.contains_key(path) {
            if self.nodes[path].is_directory() {
                return Ok(()); // Directory already exists
            } else {
                return Err(types::Errno::Exist.into()); // Path exists but is not a directory
            }
        }

        // Ensure parent exists
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !self.nodes.contains_key(parent) {
                return Err(types::Errno::Noent.into()); // Parent directory doesn't exist
            }
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        self.nodes.insert(
            path.to_path_buf(),
            FsNode::Directory {
                created: now,
                modified: now,
                accessed: now,
            },
        );

        Ok(())
    }

    pub fn create_file(&mut self, path: &Path, content: Vec<u8>) -> Result<(), types::Error> {
        if self.nodes.contains_key(path) {
            return Err(types::Errno::Exist.into()); // File already exists
        }

        // Ensure parent exists
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !self.nodes.contains_key(parent) {
                return Err(types::Errno::Noent.into()); // Parent directory doesn't exist
            }
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        self.nodes.insert(
            path.to_path_buf(),
            FsNode::File {
                content,
                created: now,
                modified: now,
                accessed: now,
            },
        );

        Ok(())
    }

    pub fn open(&mut self, path: &Path, read: bool, write: bool, append: bool, create: bool) -> Result<u32, types::Error> {
        let path = self.normalize_path(path);
        
        if !self.nodes.contains_key(&path) {
            if create && write {
                self.create_file(&path, Vec::new())?;
            } else {
                return Err(types::Errno::Noent.into()); // File doesn't exist
            }
        }

        // Check if it's a directory
        if self.nodes[&path].is_directory() && write {
            return Err(types::Errno::Isdir.into()); // Can't open directory for writing
        }

        let fd = self.next_fd;
        self.next_fd += 1;

        let position = if append {
            // Get file size if append mode
            if let FsNode::File { content, .. } = &self.nodes[&path] {
                content.len() as u64
            } else {
                0
            }
        } else {
            0
        };

        // Update access time
        if let Some(node) = self.nodes.get_mut(&path) {
            node.update_accessed();
        }

        self.open_fds.insert(
            fd,
            MockFd {
                path,
                position,
                append,
                read,
                write,
            },
        );

        Ok(fd)
    }

    pub fn close(&mut self, fd: u32) -> Result<(), types::Error> {
        if !self.open_fds.contains_key(&fd) {
            return Err(types::Errno::Badf.into()); // Bad file descriptor
        }

        self.open_fds.remove(&fd);
        Ok(())
    }

    pub fn read(&mut self, fd: u32, buffer: &mut [u8]) -> Result<usize, types::Error> {
        // Get the file descriptor
        let mock_fd = match self.open_fds.get(&fd) {
            Some(fd) => fd.clone(),
            None => return Err(types::Errno::Badf.into()),
        };

        // Check if the file descriptor is readable
        if !mock_fd.read {
            return Err(types::Errno::Perm.into());
        }

        // Get the node
        let node = match self.nodes.get_mut(&mock_fd.path) {
            Some(node) => node,
            None => return Err(types::Errno::Noent.into()),
        };

        // Check if the node is a file
        match node {
            FsNode::File { content, accessed, .. } => {
                // Update accessed time
                *accessed = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                
                // Read the file content based on position
                let position_usize = mock_fd.position as usize;
                if position_usize >= content.len() {
                    return Ok(0);
                }

                let bytes_to_read = std::cmp::min(buffer.len(), content.len() - position_usize);
                buffer[0..bytes_to_read].copy_from_slice(&content[position_usize..position_usize + bytes_to_read]);

                // Update the fd position
                if let Some(fd) = self.open_fds.get_mut(&fd) {
                    fd.position += bytes_to_read as u64;
                }

                Ok(bytes_to_read)
            }
            FsNode::Directory { .. } => Err(types::Errno::Isdir.into()),
        }
    }

    pub fn write(&mut self, fd: u32, buffer: &[u8]) -> Result<usize, types::Error> {
        // Get the file descriptor
        let mock_fd = match self.open_fds.get_mut(&fd) {
            Some(fd) => fd,
            None => return Err(types::Errno::Badf.into()),
        };

        // Check if the file descriptor is writable
        if !mock_fd.write {
            return Err(types::Errno::Perm.into());
        }

        // Get the node
        let node = match self.nodes.get_mut(&mock_fd.path) {
            Some(node) => node,
            None => return Err(types::Errno::Noent.into()),
        };

        // Check if the node is a file
        match node {
            FsNode::File { content, modified, .. } => {
                // Update modified time
                *modified = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                
                let position_usize = mock_fd.position as usize;
                
                // Ensure the file is big enough
                if position_usize > content.len() {
                    content.resize(position_usize, 0);
                }

                // Write the buffer to the file
                let end_position = position_usize + buffer.len();
                if end_position > content.len() {
                    content.resize(end_position, 0);
                }
                content[position_usize..end_position].copy_from_slice(buffer);

                // Update the fd position
                if let Some(fd) = self.open_fds.get_mut(&fd) {
                    fd.position += buffer.len() as u64;
                }

                Ok(buffer.len())
            }
            FsNode::Directory { .. } => Err(types::Errno::Isdir.into()),
        }
    }

    pub fn seek(&mut self, fd: u32, offset: i64, whence: types::Whence) -> Result<u64, types::Error> {
        // Get the file descriptor
        let mock_fd = match self.open_fds.get(&fd) {
            Some(fd) => fd.clone(),
            None => return Err(types::Errno::Badf.into()),
        };

        // Get the file node
        let node = match self.nodes.get(&mock_fd.path) {
            Some(node) => node,
            None => return Err(types::Errno::Noent.into()),
        };

        // Check if it's a file
        let content_len = match node {
            FsNode::File { content, .. } => content.len() as u64,
            FsNode::Directory { .. } => return Err(types::Errno::Isdir.into()),
        };

        // Calculate new position based on whence
        let new_pos = match whence {
            types::Whence::Set => {
                if offset < 0 {
                    return Err(types::Errno::Inval.into());
                }
                offset as u64
            }
            types::Whence::Cur => {
                let current = mock_fd.position;
                if offset < 0 && current < offset.unsigned_abs() {
                    return Err(types::Errno::Inval.into());
                }
                if offset < 0 {
                    current - offset.unsigned_abs()
                } else {
                    current + offset as u64
                }
            }
            types::Whence::End => {
                if offset < 0 && content_len < offset.unsigned_abs() {
                    return Err(types::Errno::Inval.into());
                }
                if offset < 0 {
                    content_len - offset.unsigned_abs()
                } else {
                    content_len + offset as u64
                }
            }
        };

        // Update the file position
        if let Some(fd) = self.open_fds.get_mut(&fd) {
            fd.position = new_pos;
        }

        Ok(new_pos)
    }

    pub fn get_filestat(&self, fd: u32) -> Result<types::Filestat, types::Error> {
        // Get file descriptor
        let mock_fd = match self.open_fds.get(&fd) {
            Some(fd) => fd,
            None => return Err(types::Errno::Badf.into()), // Bad file descriptor
        };

        // Get file/directory stats
        match self.nodes.get(&mock_fd.path) {
            Some(FsNode::File { content, created, modified, accessed }) => {
                Ok(types::Filestat {
                    dev: 1,
                    ino: 1,
                    filetype: types::Filetype::RegularFile,
                    nlink: 1,
                    size: content.len() as u64,
                    atim: *accessed,
                    mtim: *modified,
                    ctim: *created,
                })
            }
            Some(FsNode::Directory { created, modified, accessed }) => {
                Ok(types::Filestat {
                    dev: 1,
                    ino: 1,
                    filetype: types::Filetype::Directory,
                    nlink: 1,
                    size: 0,
                    atim: *accessed,
                    mtim: *modified,
                    ctim: *created,
                })
            }
            None => Err(types::Errno::Badf.into()), // Bad file descriptor
        }
    }

    pub fn path_filestat_get(&self, dirfd: u32, path: &Path) -> Result<types::Filestat, types::Error> {
        // Get directory from dirfd
        let base_path = if dirfd == 3 { // Preopen directory (usually fd 3)
            Path::new("/").to_path_buf()
        } else if dirfd == u32::MAX { // Use current working directory
            self.cwd.clone()
        } else {
            // Get directory handle
            match self.open_fds.get(&dirfd) {
                Some(fd) if self.is_directory(&fd.path) => fd.path.clone(),
                Some(_) => return Err(types::Errno::Notdir.into()),
                None => return Err(types::Errno::Badf.into()),
            }
        };

        let full_path = self.normalize_path(&base_path.join(path));

        // Get file/directory stats
        match self.nodes.get(&full_path) {
            Some(FsNode::File { content, created, modified, accessed }) => {
                Ok(types::Filestat {
                    dev: 1,
                    ino: 1,
                    filetype: types::Filetype::RegularFile,
                    nlink: 1,
                    size: content.len() as u64,
                    atim: *accessed,
                    mtim: *modified,
                    ctim: *created,
                })
            }
            Some(FsNode::Directory { created, modified, accessed }) => {
                Ok(types::Filestat {
                    dev: 1,
                    ino: 1,
                    filetype: types::Filetype::Directory,
                    nlink: 1,
                    size: 0,
                    atim: *accessed,
                    mtim: *modified,
                    ctim: *created,
                })
            }
            None => Err(types::Errno::Noent.into()), // No such file or directory
        }
    }

    pub fn readdir(&self, fd: u32, cookie: u64) -> Result<Vec<types::Dirent>, types::Error> {
        // Get file descriptor
        let mock_fd = match self.open_fds.get(&fd) {
            Some(fd) => fd,
            None => return Err(types::Errno::Badf.into()), // Bad file descriptor
        };

        // Check if it's a directory
        if !self.nodes.get(&mock_fd.path).map_or(false, |n| n.is_directory()) {
            return Err(types::Errno::Notdir.into()); // Not a directory
        }

        let dir_path = &mock_fd.path;
        let mut entries = Vec::new();
        let mut idx = 0;

        // Skip entries based on cookie
        if cookie > 0 {
            idx = cookie as usize;
        }

        // Add "." and ".." entries if cookie is 0
        if cookie == 0 {
            entries.push(types::Dirent {
                d_next: 1,
                d_ino: 1,
                d_namlen: 1,
                d_type: types::Filetype::Directory,
            });
            
            entries.push(types::Dirent {
                d_next: 2,
                d_ino: 1,
                d_namlen: 2,
                d_type: types::Filetype::Directory,
            });

            idx = 2;
        }

        // Add directory contents
        for (path, node) in self.nodes.iter() {
            // Skip if not a direct child of this directory
            if path == dir_path || path.parent() != Some(dir_path) {
                continue;
            }

            // Get the file name
            let name = match path.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };

            let file_type = if node.is_directory() {
                types::Filetype::Directory
            } else {
                types::Filetype::RegularFile
            };

            entries.push(types::Dirent {
                d_next: idx as u64 + 1,
                d_ino: idx as u64 + 1,
                d_namlen: name.len() as u32,
                d_type: file_type,
            });

            idx += 1;
        }

        Ok(entries)
    }

    // Add file descriptor as a pre-opened directory
    pub fn add_preopen_dir(&mut self, path: &Path) -> Result<u32, types::Error> {
        let path = self.normalize_path(path);
        
        // Create directory if it doesn't exist
        if !self.nodes.contains_key(&path) {
            self.create_directory(&path)?;
        }

        // Check if it's a directory
        if !self.nodes[&path].is_directory() {
            return Err(types::Errno::Notdir.into()); // Not a directory
        }

        // Open the directory
        let fd = self.next_fd;
        self.next_fd += 1;

        self.open_fds.insert(
            fd,
            MockFd {
                path: path.clone(),
                position: 0,
                append: false,
                read: true,
                write: false,
            },
        );

        // Add to preopened directories
        self.preopen_dirs.insert(path);

        Ok(fd)
    }

    // Helper function to normalize a path
    fn normalize_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            return path.to_path_buf();
        }
        
        let mut result = self.cwd.clone();
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    result.pop();
                }
                std::path::Component::Normal(name) => {
                    result.push(name);
                }
                _ => {}
            }
        }
        
        result
    }

    // Check if a path exists
    pub fn exists(&self, path: &Path) -> bool {
        let path = self.normalize_path(path);
        self.nodes.contains_key(&path)
    }

    // Check if a path is a directory
    pub fn is_directory(&self, path: &Path) -> bool {
        let path = self.normalize_path(path);
        self.nodes.get(&path).map_or(false, |n| n.is_directory())
    }

    // Check if a file descriptor exists
    pub fn fd_exists(&self, fd: u32) -> bool {
        self.open_fds.contains_key(&fd)
    }

    // Check if a file descriptor points to a directory
    pub fn fd_is_directory(&self, fd: u32) -> bool {
        match self.open_fds.get(&fd) {
            Some(mock_fd) => self.nodes.get(&mock_fd.path).map_or(false, |n| n.is_directory()),
            None => false,
        }
    }

    // Check if a file descriptor points to a regular file
    pub fn fd_is_file(&self, fd: u32) -> bool {
        match self.open_fds.get(&fd) {
            Some(mock_fd) => self.nodes.get(&mock_fd.path).map_or(false, |n| n.is_file()),
            None => false,
        }
    }

    // Get current working directory
    pub fn getcwd(&self) -> PathBuf {
        self.cwd.clone()
    }

    // Set current working directory
    pub fn set_cwd(&mut self, path: &Path) -> Result<(), types::Error> {
        let path = self.normalize_path(path);
        
        if !self.nodes.contains_key(&path) {
            return Err(types::Errno::Noent.into()); // No such file or directory
        }
        
        if !self.nodes[&path].is_directory() {
            return Err(types::Errno::Notdir.into()); // Not a directory
        }
        
        self.cwd = path;
        Ok(())
    }
}

/// A thread-safe wrapper around MockFileSystem
#[derive(Debug, Default)]
pub struct SharedMockFileSystem {
    inner: Arc<Mutex<MockFileSystem>>,
}

impl SharedMockFileSystem {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(MockFileSystem::new())),
        }
    }
    
    pub fn with_fs<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut MockFileSystem) -> R,
    {
        let mut fs = self.inner.lock().unwrap();
        f(&mut fs)
    }
}

impl Clone for SharedMockFileSystem {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
} 