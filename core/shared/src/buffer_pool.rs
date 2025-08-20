//! High-performance buffer pool for zero-allocation request processing

use std::sync::Mutex;
use smallvec::SmallVec;
use arrayvec::ArrayVec;

/// Object pool for reusable buffers to minimize allocations
pub struct BufferPool {
    pool: Mutex<Vec<Vec<u8>>>,
    buffer_size: usize,
}

impl BufferPool {
    /// Create a new buffer pool with the specified buffer size and initial capacity
    pub fn new(buffer_size: usize, initial_capacity: usize) -> Self {
        let pool = (0..initial_capacity)
            .map(|_| Vec::with_capacity(buffer_size))
            .collect();
            
        Self { 
            pool: Mutex::new(pool), 
            buffer_size 
        }
    }
    
    /// Acquire a buffer from the pool
    pub fn acquire(&self) -> Option<PooledBuffer> {
        let mut pool = self.pool.lock().ok()?;
        let mut buffer = pool.pop()
            .unwrap_or_else(|| Vec::with_capacity(self.buffer_size));
        buffer.clear();
        
        Some(PooledBuffer {
            buffer,
            pool_ptr: self as *const Self,
        })
    }
    
    /// Return a buffer to the pool (called by PooledBuffer::drop)
    fn return_buffer(&self, buffer: Vec<u8>) {
        if let Ok(mut pool) = self.pool.lock() {
            if pool.len() < 32 { // Limit pool size to prevent unbounded growth
                pool.push(buffer);
            }
        }
    }
}

/// A pooled buffer that automatically returns to the pool when dropped
pub struct PooledBuffer {
    buffer: Vec<u8>,
    pool_ptr: *const BufferPool,
}

impl PooledBuffer {
    /// Get mutable access to the underlying buffer
    pub fn as_mut_vec(&mut self) -> &mut Vec<u8> {
        &mut self.buffer
    }
    
    /// Get immutable access to the underlying buffer
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }
    
    /// Get the current length of the buffer
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    
    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

impl Drop for PooledBuffer {
    fn drop(&mut self) {
        // Return buffer to pool
        unsafe {
            let pool = &*self.pool_ptr;
            pool.return_buffer(std::mem::take(&mut self.buffer));
        }
    }
}

// Safety: PooledBuffer is safe to send across threads
unsafe impl Send for PooledBuffer {}

/// Stack-allocated collections for small data that avoids heap allocation
pub type SmallString = SmallVec<[u8; 64]>;
pub type SmallMessageList = SmallVec<[crate::types::Message; 8]>;

/// Fixed-capacity buffer for performance-critical operations
pub type FixedBuffer = ArrayVec<u8, 65536>; // 64KB on stack

/// Zero-copy string processing utilities
pub mod zero_copy {
    use std::borrow::Cow;
    
    /// Process text data with zero-copy when possible
    pub fn process_text_data<'a>(input: Cow<'a, str>) -> Cow<'a, str> {
        // Only allocate if we need to modify
        if needs_preprocessing(&input) {
            let mut owned = input.into_owned();
            preprocess_text(&mut owned);
            Cow::Owned(owned)
        } else {
            input
        }
    }
    
    fn needs_preprocessing(text: &str) -> bool {
        // Simple heuristic - check for characters that need escaping
        text.contains('\n') || text.contains('\t') || text.contains('\"')
    }
    
    fn preprocess_text(text: &mut String) {
        // In-place text preprocessing
        *text = text.replace('\n', "\\n")
                   .replace('\t', "\\t")
                   .replace('\"', "\\\"");
    }
}

/// Pre-allocated working memory for request processing
pub struct WorkingMemory {
    pub request_buffer: FixedBuffer,
    pub response_buffer: FixedBuffer,
    pub temp_strings: SmallVec<[String; 4]>,
}

impl WorkingMemory {
    pub fn new() -> Self {
        Self {
            request_buffer: ArrayVec::new(),
            response_buffer: ArrayVec::new(),
            temp_strings: SmallVec::new(),
        }
    }
    
    pub fn reset(&mut self) {
        self.request_buffer.clear();
        self.response_buffer.clear();
        self.temp_strings.clear();
    }
}

impl Default for WorkingMemory {
    fn default() -> Self {
        Self::new()
    }
}

thread_local! {
    /// Thread-local working memory to avoid allocations in hot paths
    static WORKING_MEMORY: std::cell::RefCell<WorkingMemory> = std::cell::RefCell::new(WorkingMemory::new());
}

/// Execute function with thread-local working memory
pub fn with_working_memory<F, R>(f: F) -> R
where
    F: FnOnce(&mut WorkingMemory) -> R,
{
    WORKING_MEMORY.with(|wm| {
        let mut wm = wm.borrow_mut();
        let result = f(&mut wm);
        wm.reset(); // Reclaim memory for reuse
        result
    })
}