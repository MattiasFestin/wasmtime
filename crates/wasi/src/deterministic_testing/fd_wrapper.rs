use crate::preview1::types;
use std::collections::HashMap;

/// An enum wrapper around file descriptors to support both real and mock implementations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FdWrapper {
    /// A real file descriptor from the system
    Real(types::Fd),
    /// A mock file descriptor for testing (just an integer ID)
    Mock(u32),
}

/// Metadata for a mock file descriptor
#[derive(Debug, Clone)]
pub struct FdMetadata {
    /// Path associated with this file descriptor (if any)
    pub path: Option<String>,
    /// Type of file
    pub filetype: types::Filetype,
    /// Current position in the file
    pub position: u64,
    /// Size of the file
    pub size: u64,
    /// Content of the file (for regular files)
    pub content: Option<Vec<u8>>,
}

/// A virtual filesystem for mock file descriptors
#[derive(Debug, Default)]
pub struct MockFilesystem {
    /// Map of mock file descriptors to their metadata
    fds: HashMap<u32, FdMetadata>,
    /// Next available mock file descriptor
    next_fd: u32,
}

impl MockFilesystem {
    /// Create a new mock filesystem
    pub fn new() -> Self {
        let mut fs = Self {
            fds: HashMap::new(),
            next_fd: 3, // Start after the standard file descriptors
        };

        // Initialize standard file descriptors (0, 1, 2 for stdin, stdout, stderr)
        fs.fds.insert(0, FdMetadata {
            path: None,
            filetype: types::Filetype::CharacterDevice,
            position: 0,
            size: 0,
            content: None,
        });
        fs.fds.insert(1, FdMetadata {
            path: None,
            filetype: types::Filetype::CharacterDevice,
            position: 0,
            size: 0,
            content: None,
        });
        fs.fds.insert(2, FdMetadata {
            path: None,
            filetype: types::Filetype::CharacterDevice,
            position: 0,
            size: 0,
            content: None,
        });

        fs
    }

    /// Create a new file descriptor and return it
    pub fn create_fd(&mut self, path: Option<String>, filetype: types::Filetype) -> FdWrapper {
        let fd = self.next_fd;
        self.next_fd += 1;
        
        self.fds.insert(fd, FdMetadata {
            path,
            filetype,
            position: 0,
            size: 0,
            content: if filetype == types::Filetype::RegularFile {
                Some(Vec::new())
            } else {
                None
            },
        });
        
        FdWrapper::Mock(fd)
    }
    
    /// Check if a file descriptor exists
    pub fn fd_exists(&self, fd: u32) -> bool {
        self.fds.contains_key(&fd)
    }
    
    /// Get metadata for a file descriptor
    pub fn get_fd_metadata(&self, fd: u32) -> Option<&FdMetadata> {
        self.fds.get(&fd)
    }
    
    /// Get mutable metadata for a file descriptor
    pub fn get_fd_metadata_mut(&mut self, fd: u32) -> Option<&mut FdMetadata> {
        self.fds.get_mut(&fd)
    }
    
    /// Close a file descriptor
    pub fn close_fd(&mut self, fd: u32) -> Result<(), types::Error> {
        // Don't allow closing standard file descriptors
        if fd < 3 {
            return Err(types::Error::from(types::Errno::Badf));
        }
        
        if self.fds.remove(&fd).is_none() {
            return Err(types::Error::from(types::Errno::Badf));
        }
        
        Ok(())
    }

    /// Write data to a file descriptor
    pub fn write_fd(&mut self, fd: u32, data: &[u8], offset: Option<u64>) -> Result<usize, types::Error> {
        let metadata = self.get_fd_metadata_mut(fd)
            .ok_or_else(|| types::Error::from(types::Errno::Badf))?;
        
        // Only regular files can be written to
        if metadata.filetype != types::Filetype::RegularFile {
            return Err(types::Error::from(types::Errno::Isdir));
        }
        
        let content = metadata.content.as_mut()
            .ok_or_else(|| types::Error::from(types::Errno::Badf))?;
        
        let position = if let Some(offset) = offset {
            offset
        } else {
            metadata.position
        };
        
        // Ensure the content is large enough
        if position as usize > content.len() {
            content.resize(position as usize, 0);
        }
        
        // Write the data
        let start_pos = position as usize;
        let end_pos = start_pos + data.len();
        
        if end_pos > content.len() {
            content.resize(end_pos, 0);
        }
        
        content[start_pos..end_pos].copy_from_slice(data);
        
        // Update position and size
        if offset.is_none() {
            metadata.position = end_pos as u64;
        }
        metadata.size = content.len() as u64;
        
        Ok(data.len())
    }
    
    /// Read data from a file descriptor
    pub fn read_fd(&mut self, fd: u32, buf: &mut [u8], offset: Option<u64>) -> Result<usize, types::Error> {
        let metadata = self.get_fd_metadata_mut(fd)
            .ok_or_else(|| types::Error::from(types::Errno::Badf))?;
        
        // Only regular files can be read from
        if metadata.filetype != types::Filetype::RegularFile {
            return Err(types::Error::from(types::Errno::Isdir));
        }
        
        let content = metadata.content.as_ref()
            .ok_or_else(|| types::Error::from(types::Errno::Badf))?;
        
        let position = if let Some(offset) = offset {
            offset
        } else {
            metadata.position
        };
        
        // Check if we're at EOF
        if position as usize >= content.len() {
            return Ok(0);
        }
        
        // Read the data
        let start_pos = position as usize;
        let end_pos = std::cmp::min(start_pos + buf.len(), content.len());
        let bytes_read = end_pos - start_pos;
        
        buf[..bytes_read].copy_from_slice(&content[start_pos..end_pos]);
        
        // Update position
        if offset.is_none() {
            metadata.position = end_pos as u64;
        }
        
        Ok(bytes_read)
    }
    
    /// Seek to a position in a file
    pub fn seek_fd(&mut self, fd: u32, offset: i64, whence: types::Whence) -> Result<u64, types::Error> {
        let metadata = self.get_fd_metadata_mut(fd)
            .ok_or_else(|| types::Error::from(types::Errno::Badf))?;
        
        // Only regular files can be seeked
        if metadata.filetype != types::Filetype::RegularFile {
            return Err(types::Error::from(types::Errno::Isdir));
        }
        
        let content = metadata.content.as_ref()
            .ok_or_else(|| types::Error::from(types::Errno::Badf))?;
        
        let new_position = match whence {
            types::Whence::Set => {
                if offset < 0 {
                    return Err(types::Error::from(types::Errno::Inval));
                }
                offset as u64
            },
            types::Whence::Cur => {
                if offset < 0 && metadata.position < (-offset) as u64 {
                    return Err(types::Error::from(types::Errno::Inval));
                }
                if offset < 0 {
                    metadata.position - (-offset) as u64
                } else {
                    metadata.position + offset as u64
                }
            },
            types::Whence::End => {
                let size = content.len() as i64;
                if offset < 0 && size < (-offset) {
                    return Err(types::Error::from(types::Errno::Inval));
                }
                if offset < 0 {
                    size as u64 - (-offset) as u64
                } else {
                    size as u64 + offset as u64
                }
            },
        };
        
        metadata.position = new_position;
        Ok(new_position)
    }
}

impl FdWrapper {
    /// Convert from a real file descriptor
    pub fn from_real(fd: types::Fd) -> Self {
        Self::Real(fd)
    }
    
    /// Convert from a mock file descriptor
    pub fn from_mock(fd: u32) -> Self {
        Self::Mock(fd)
    }
    
    /// Get the mock file descriptor, if this is a mock
    pub fn as_mock(&self) -> Option<u32> {
        match self {
            Self::Mock(fd) => Some(*fd),
            _ => None,
        }
    }
    
    /// Get the real file descriptor, if this is real
    pub fn as_real(&self) -> Option<types::Fd> {
        match self {
            Self::Real(fd) => Some(*fd),
            _ => None,
        }
    }
} 