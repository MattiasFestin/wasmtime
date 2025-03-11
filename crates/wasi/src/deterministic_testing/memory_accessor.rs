use wiggle::{GuestMemory, GuestPtr};
use crate::preview1::types;
use std::string::String;

/// Trait that abstracts memory access operations to make them testable
#[cfg(feature = "deterministic_testing")]
pub trait MemoryAccessor {
    /// Read a string from guest memory
    fn read_string(&mut self, str_ptr: GuestPtr<str>) -> Result<String, types::Error>;
    
    /// Read a byte buffer from guest memory
    fn read_bytes(&mut self, ptr: GuestPtr<[u8]>) -> Result<Vec<u8>, types::Error>;
    
    /// Get a mutable byte buffer from guest memory
    fn get_bytes_mut(&mut self, ptr: GuestPtr<[u8]>) -> Result<&mut [u8], types::Error>;
    
    /// Write a string to guest memory 
    fn write_string(&mut self, ptr: GuestPtr<u8>, string: &str) -> Result<(), types::Error>;
    
    /// Copy data from a slice to guest memory
    fn copy_from_slice(&mut self, src: &[u8], dst: GuestPtr<[u8]>) -> Result<(), types::Error>;
    
    /// Process iovecs and execute a callback with each buffer
    fn process_iovecs<F>(&mut self, iovs: types::IovecArray, callback: F) -> Result<u32, types::Error>
    where
        F: FnMut(&mut [u8]) -> Result<usize, types::Error>;
    
    /// Process ciovecs and execute a callback with each buffer
    fn process_ciovecs<F>(&mut self, ciovs: types::CiovecArray, callback: F) -> Result<u32, types::Error>
    where
        F: FnMut(&[u8]) -> Result<usize, types::Error>;
        
    /// Read an iovec from the given pointer
    fn read_iovec(&mut self, ptr: GuestPtr<types::Iovec>) -> Result<types::Iovec, types::Error>;
    
    /// Read a ciovec from the given pointer
    fn read_ciovec(&mut self, ptr: GuestPtr<types::Ciovec>) -> Result<types::Ciovec, types::Error>;
    
    /// Get the length of an iovec array
    fn read_iovec_array_len(&mut self, iovs: types::IovecArray) -> Result<u32, types::Error>;
    
    /// Get the length of a ciovec array
    fn read_ciovec_array_len(&mut self, ciovs: types::CiovecArray) -> Result<u32, types::Error>;
    
    /// Get an iovec at the specified index
    fn get_iovec_at_index(&mut self, iovs: types::IovecArray, idx: u32) -> Result<GuestPtr<types::Iovec>, types::Error>;
    
    /// Get a ciovec at the specified index
    fn get_ciovec_at_index(&mut self, ciovs: types::CiovecArray, idx: u32) -> Result<GuestPtr<types::Ciovec>, types::Error>;
}

/// Implementation of MemoryAccessor for GuestMemory
impl MemoryAccessor for GuestMemory<'_> {
    fn read_string(&mut self, str_ptr: GuestPtr<str>) -> Result<String, types::Error> {
        self.as_str(str_ptr)
            .map_err(|e| -> types::Error { e.into() })?
            .map(|s| s.to_string())
            .ok_or_else(|| types::Errno::Inval.into())
    }
    
    fn read_bytes(&mut self, ptr: GuestPtr<[u8]>) -> Result<Vec<u8>, types::Error> {
        self.to_vec(ptr).map_err(|e| -> types::Error { e.into() })
    }
    
    fn get_bytes_mut(&mut self, ptr: GuestPtr<[u8]>) -> Result<&mut [u8], types::Error> {
        match self.as_slice_mut(ptr).map_err(|e| -> types::Error { e.into() })? {
            Some(slice) => Ok(slice),
            None => Err(types::Errno::Fault.into()),
        }
    }
    
    fn write_string(&mut self, ptr: GuestPtr<u8>, string: &str) -> Result<(), types::Error> {
        let bytes = string.as_bytes();
        let slice_ptr = ptr.as_array(bytes.len() as u32);
        
        self.copy_from_slice(bytes, slice_ptr).map_err(|e| -> types::Error { e.into() })
    }
    
    fn copy_from_slice(&mut self, src: &[u8], dst: GuestPtr<[u8]>) -> Result<(), types::Error> {
        self.copy_from_slice(src, dst).map_err(|e| -> types::Error { e.into() })
    }
    
    fn process_iovecs<F>(&mut self, iovs: types::IovecArray, mut callback: F) -> Result<u32, types::Error>
    where
        F: FnMut(&mut [u8]) -> Result<usize, types::Error>
    {
        let iovs_count = iovs.len();
        let mut total_written = 0;
        
        for i in 0..iovs_count {
            let iovec_ptr = iovs.as_ptr().add(i).map_err(|e| -> types::Error { e.into() })?;
            let iovec = self.read(iovec_ptr).map_err(|e| -> types::Error { e.into() })?;
            
            if iovec.buf_len > 0 {
                let buffer = iovec.buf.as_array(iovec.buf_len);
                match self.as_slice_mut(buffer).map_err(|e| -> types::Error { e.into() })? {
                    Some(slice) => {
                        let written = callback(slice)?;
                        total_written += written as u32;
                    },
                    None => return Err(types::Errno::Fault.into()),
                }
            }
        }
        
        Ok(total_written)
    }
    
    fn process_ciovecs<F>(&mut self, ciovs: types::CiovecArray, mut callback: F) -> Result<u32, types::Error>
    where
        F: FnMut(&[u8]) -> Result<usize, types::Error>
    {
        let ciovs_count = ciovs.len();
        let mut total_written = 0;
        
        for i in 0..ciovs_count {
            let ciovec_ptr = ciovs.as_ptr().add(i).map_err(|e| -> types::Error { e.into() })?;
            let ciovec = self.read(ciovec_ptr).map_err(|e| -> types::Error { e.into() })?;
            
            if ciovec.buf_len > 0 {
                let buffer = ciovec.buf.as_array(ciovec.buf_len);
                match self.as_slice(buffer).map_err(|e| -> types::Error { e.into() })? {
                    Some(slice) => {
                        let written = callback(slice)?;
                        total_written += written as u32;
                    },
                    None => {
                        let buffer_vec = self.to_vec(buffer).map_err(|e| -> types::Error { e.into() })?;
                        let written = callback(&buffer_vec)?;
                        total_written += written as u32;
                    },
                }
            }
        }
        
        Ok(total_written)
    }
    
    fn read_iovec(&mut self, ptr: GuestPtr<types::Iovec>) -> Result<types::Iovec, types::Error> {
        self.read(ptr).map_err(|e| -> types::Error { e.into() })
    }
    
    fn read_ciovec(&mut self, ptr: GuestPtr<types::Ciovec>) -> Result<types::Ciovec, types::Error> {
        self.read(ptr).map_err(|e| -> types::Error { e.into() })
    }
    
    fn read_iovec_array_len(&mut self, iovs: types::IovecArray) -> Result<u32, types::Error> {
        Ok(iovs.len())
    }
    
    fn read_ciovec_array_len(&mut self, ciovs: types::CiovecArray) -> Result<u32, types::Error> {
        Ok(ciovs.len())
    }
    
    fn get_iovec_at_index(&mut self, iovs: types::IovecArray, idx: u32) -> Result<GuestPtr<types::Iovec>, types::Error> {
        let ptr = iovs.as_ptr().add(idx).map_err(|e| -> types::Error { e.into() })?;
        Ok(ptr)
    }
    
    fn get_ciovec_at_index(&mut self, ciovs: types::CiovecArray, idx: u32) -> Result<GuestPtr<types::Ciovec>, types::Error> {
        let ptr = ciovs.as_ptr().add(idx).map_err(|e| -> types::Error { e.into() })?;
        Ok(ptr)
    }
}

/// Mock implementation of MemoryAccessor for testing
#[cfg(feature = "deterministic_testing")]
pub struct MockMemoryAccessor {
    pub strings: std::collections::HashMap<u32, String>,
    pub buffers: std::collections::HashMap<u32, Vec<u8>>,
    pub iovecs: std::collections::HashMap<u32, Vec<(u32, u32)>>, // address, len pairs
    pub ciovecs: std::collections::HashMap<u32, Vec<(u32, u32)>>, // address, len pairs
    next_ptr: u32,
}

#[cfg(feature = "deterministic_testing")]
impl MockMemoryAccessor {
    pub fn new() -> Self {
        Self {
            strings: std::collections::HashMap::new(),
            buffers: std::collections::HashMap::new(),
            iovecs: std::collections::HashMap::new(),
            ciovecs: std::collections::HashMap::new(),
            next_ptr: 1000,
        }
    }
    
    pub fn alloc_string(&mut self, s: &str) -> GuestPtr<str> {
        let ptr = self.next_ptr;
        self.next_ptr += 1;
        self.strings.insert(ptr, s.to_string());
        GuestPtr::new((ptr, s.len() as u32))
    }
    
    pub fn alloc_buffer(&mut self, data: &[u8]) -> GuestPtr<[u8]> {
        let ptr = self.next_ptr;
        self.next_ptr += 1;
        self.buffers.insert(ptr, data.to_vec());
        GuestPtr::new((ptr, data.len() as u32))
    }
    
    pub fn alloc_iovecs(&mut self, iovecs: &[(u32, u32)]) -> u32 {
        let ptr = self.next_ptr;
        self.next_ptr += 1;
        self.iovecs.insert(ptr, iovecs.to_vec());
        ptr
    }
    
    pub fn alloc_ciovecs(&mut self, ciovecs: &[(u32, u32)]) -> u32 {
        let ptr = self.next_ptr;
        self.next_ptr += 1;
        self.ciovecs.insert(ptr, ciovecs.to_vec());
        ptr
    }
}

#[cfg(feature = "deterministic_testing")]
impl MemoryAccessor for MockMemoryAccessor {
    fn read_string(&mut self, str_ptr: GuestPtr<str>) -> Result<String, types::Error> {
        let addr = str_ptr.offset().0;
        match self.strings.get(&addr) {
            Some(s) => Ok(s.clone()),
            None => Err(types::Errno::Inval.into()),
        }
    }
    
    fn read_bytes(&mut self, _ptr: GuestPtr<[u8]>) -> Result<Vec<u8>, types::Error> {
        Ok(vec![0u8; 100])
    }
    
    fn get_bytes_mut(&mut self, _ptr: GuestPtr<[u8]>) -> Result<&mut [u8], types::Error> {
        let ptr = self.next_ptr;
        self.next_ptr += 1;
        let buffer = vec![0u8; 100];
        self.buffers.insert(ptr, buffer);
        Ok(&mut self.buffers.get_mut(&ptr).unwrap()[..])
    }
    
    fn write_string(&mut self, ptr: GuestPtr<u8>, string: &str) -> Result<(), types::Error> {
        let bytes = string.as_bytes().to_vec();
        let addr = ptr.offset();
        self.buffers.insert(addr, bytes);
        Ok(())
    }
    
    fn copy_from_slice(&mut self, src: &[u8], dst: GuestPtr<[u8]>) -> Result<(), types::Error> {
        let addr = dst.offset().0;
        self.buffers.insert(addr, src.to_vec());
        Ok(())
    }
    
    fn process_iovecs<F>(&mut self, _iovs: types::IovecArray, mut callback: F) -> Result<u32, types::Error>
    where
        F: FnMut(&mut [u8]) -> Result<usize, types::Error>
    {
        let mut buffer = vec![0u8; 100];
        let written = callback(&mut buffer)?;
        Ok(written as u32)
    }
    
    fn process_ciovecs<F>(&mut self, _ciovs: types::CiovecArray, mut callback: F) -> Result<u32, types::Error>
    where
        F: FnMut(&[u8]) -> Result<usize, types::Error>
    {
        let buffer = vec![0u8; 100];
        let read = callback(&buffer)?;
        Ok(read as u32)
    }
    
    fn read_iovec(&mut self, _ptr: GuestPtr<types::Iovec>) -> Result<types::Iovec, types::Error> {
        Ok(types::Iovec {
            buf: GuestPtr::new(0),
            buf_len: 100,
        })
    }
    
    fn read_ciovec(&mut self, _ptr: GuestPtr<types::Ciovec>) -> Result<types::Ciovec, types::Error> {
        Ok(types::Ciovec {
            buf: GuestPtr::new(0),
            buf_len: 100,
        })
    }
    
    fn read_iovec_array_len(&mut self, _iovs: types::IovecArray) -> Result<u32, types::Error> {
        Ok(1)
    }
    
    fn read_ciovec_array_len(&mut self, _ciovs: types::CiovecArray) -> Result<u32, types::Error> {
        Ok(1)
    }
    
    fn get_iovec_at_index(&mut self, _iovs: types::IovecArray, _idx: u32) -> Result<GuestPtr<types::Iovec>, types::Error> {
        let ptr = 1000u32;
        Ok(GuestPtr::new(ptr))
    }
    
    fn get_ciovec_at_index(&mut self, _ciovs: types::CiovecArray, _idx: u32) -> Result<GuestPtr<types::Ciovec>, types::Error> {
        let ptr = 1000u32;
        Ok(GuestPtr::new(ptr))
    }
} 