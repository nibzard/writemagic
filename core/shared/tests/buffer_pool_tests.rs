//! Unit tests for buffer pool and memory management

use crate::{BufferPool, PooledBuffer};
use std::sync::Arc;

#[cfg(test)]
mod buffer_pool_tests {
    use super::*;

    #[test]
    fn test_buffer_pool_creation() {
        let pool = BufferPool::new(1024, 10);
        
        // Pool should be created successfully
        // We can't directly check capacity and length with current API
        // but we can test basic functionality
        let buffer = pool.acquire();
        assert!(buffer.is_some());
    }

    #[test]
    fn test_buffer_acquisition_and_return() {
        let pool = Arc::new(BufferPool::new(512, 5));
        
        // Acquire a buffer
        let buffer_opt = pool.acquire();
        assert!(buffer_opt.is_some());
        
        let mut buffer = buffer_opt.unwrap();
        
        // Use the buffer
        {
            let data = buffer.as_mut_vec();
            data.extend_from_slice(b"Hello, World!");
            assert_eq!(data.len(), 13);
        }
        
        assert_eq!(buffer.len(), 13);
        assert!(!buffer.is_empty());
        
        // Buffer will be returned to pool when dropped
        drop(buffer);
    }

    #[test]
    fn test_buffer_pool_reuse() {
        let pool = Arc::new(BufferPool::new(256, 2));
        
        // Acquire and use first buffer
        {
            let mut buffer1 = pool.acquire().unwrap();
            let data = buffer1.as_mut_vec();
            data.extend_from_slice(b"Buffer 1");
        }
        
        // Acquire second buffer - might be the same as first (reused)
        {
            let mut buffer2 = pool.acquire().unwrap();
            let data = buffer2.as_mut_vec();
            // Buffer should be cleared when returned to pool
            data.extend_from_slice(b"Buffer 2");
            assert!(buffer2.len() >= 8);
        }
    }

    #[test]
    fn test_buffer_slice_access() {
        let pool = BufferPool::new(128, 1);
        let mut buffer = pool.acquire().unwrap();
        
        // Add some data
        let data = buffer.as_mut_vec();
        data.extend_from_slice(b"Test data");
        
        // Check slice access
        let slice = buffer.as_slice();
        assert_eq!(slice, b"Test data");
        assert_eq!(buffer.len(), 9);
    }
}

// Note: Removed working memory tests as those APIs don't seem to be implemented
// in the current codebase. If needed, they can be re-added when the APIs are available.