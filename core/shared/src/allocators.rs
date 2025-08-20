//! Custom allocator support for performance-critical workloads

use std::alloc::{GlobalAlloc, Layout, System};
use std::ptr;

/// Configuration for different allocation strategies
#[derive(Debug, Clone)]
pub struct AllocatorConfig {
    pub use_jemalloc: bool,
    pub arena_size: usize,
    pub thread_cache_size: usize,
}

impl Default for AllocatorConfig {
    fn default() -> Self {
        Self {
            use_jemalloc: true,
            arena_size: 1024 * 1024, // 1MB
            thread_cache_size: 64 * 1024, // 64KB
        }
    }
}

/// jemalloc configuration for high-performance scenarios
#[cfg(feature = "jemalloc")]
pub mod jemalloc_allocator {
    use tikv_jemallocator::Jemalloc;
    
    /// Global jemalloc allocator
    #[global_allocator]
    static GLOBAL: Jemalloc = Jemalloc;
    
    /// Initialize jemalloc with optimal settings
    pub fn configure_jemalloc() {
        // Set jemalloc options for better performance
        unsafe {
            // Increase dirty decay time for better performance at cost of memory
            tikv_jemalloc_ctl::opt::dirty_decay_ms::write(10000).ok();
            
            // Increase muzzy decay time  
            tikv_jemalloc_ctl::opt::muzzy_decay_ms::write(10000).ok();
            
            // Enable background threads for better performance
            tikv_jemalloc_ctl::opt::background_thread::write(true).ok();
        }
    }
    
    /// Get jemalloc statistics
    pub fn get_stats() -> JemallocStats {
        let allocated = tikv_jemalloc_ctl::stats::allocated::read().unwrap_or(0);
        let active = tikv_jemalloc_ctl::stats::active::read().unwrap_or(0);
        let mapped = tikv_jemalloc_ctl::stats::mapped::read().unwrap_or(0);
        let retained = tikv_jemalloc_ctl::stats::retained::read().unwrap_or(0);
        
        JemallocStats {
            allocated,
            active, 
            mapped,
            retained,
        }
    }
    
    #[derive(Debug, Clone)]
    pub struct JemallocStats {
        pub allocated: usize,
        pub active: usize,
        pub mapped: usize,
        pub retained: usize,
    }
}

/// Arena allocator for request-scoped memory management
pub struct ArenaAllocator {
    buffer: Vec<u8>,
    position: usize,
    alignment: usize,
}

impl ArenaAllocator {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            position: 0,
            alignment: std::mem::align_of::<usize>(),
        }
    }
    
    pub fn with_alignment(capacity: usize, alignment: usize) -> Self {
        assert!(alignment.is_power_of_two(), "Alignment must be a power of two");
        
        Self {
            buffer: Vec::with_capacity(capacity),
            position: 0,
            alignment,
        }
    }
    
    /// Allocate memory from the arena
    pub fn alloc(&mut self, layout: Layout) -> Option<*mut u8> {
        let align = layout.align().max(self.alignment);
        let size = layout.size();
        
        // Calculate aligned position
        let aligned_pos = (self.position + align - 1) & !(align - 1);
        let end_pos = aligned_pos + size;
        
        if end_pos > self.buffer.capacity() {
            return None; // Out of arena space
        }
        
        // Ensure buffer has enough initialized space
        if end_pos > self.buffer.len() {
            self.buffer.resize(end_pos, 0);
        }
        
        self.position = end_pos;
        
        Some(unsafe { self.buffer.as_mut_ptr().add(aligned_pos) })
    }
    
    /// Allocate and initialize memory
    pub fn alloc_zeroed(&mut self, layout: Layout) -> Option<*mut u8> {
        self.alloc(layout).map(|ptr| {
            unsafe {
                ptr::write_bytes(ptr, 0, layout.size());
            }
            ptr
        })
    }
    
    /// Reset the arena, reclaiming all memory
    pub fn reset(&mut self) {
        self.position = 0;
        // Keep the buffer allocated for reuse
    }
    
    /// Get current memory usage
    pub fn used(&self) -> usize {
        self.position
    }
    
    /// Get total capacity
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }
    
    /// Get utilization percentage
    pub fn utilization(&self) -> f64 {
        if self.buffer.capacity() == 0 {
            0.0
        } else {
            self.position as f64 / self.buffer.capacity() as f64
        }
    }
}

thread_local! {
    /// Thread-local arena allocator for zero-contention allocation
    static THREAD_ARENA: std::cell::RefCell<ArenaAllocator> = 
        std::cell::RefCell::new(ArenaAllocator::new(1024 * 1024)); // 1MB per thread
}

/// Allocate from thread-local arena
pub fn alloc_in_thread_arena(layout: Layout) -> Option<*mut u8> {
    THREAD_ARENA.with(|arena| {
        arena.borrow_mut().alloc(layout)
    })
}

/// Reset thread-local arena
pub fn reset_thread_arena() {
    THREAD_ARENA.with(|arena| {
        arena.borrow_mut().reset()
    });
}

/// Get thread arena statistics
pub fn thread_arena_stats() -> (usize, usize, f64) {
    THREAD_ARENA.with(|arena| {
        let arena = arena.borrow();
        (arena.used(), arena.capacity(), arena.utilization())
    })
}

/// Stack allocator for very short-lived allocations
pub struct StackAllocator<const N: usize> {
    buffer: [u8; N],
    top: usize,
}

impl<const N: usize> Default for StackAllocator<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> StackAllocator<N> {
    pub const fn new() -> Self {
        Self {
            buffer: [0; N],
            top: 0,
        }
    }
    
    /// Push allocation onto stack
    pub fn push(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        let aligned_top = (self.top + align - 1) & !(align - 1);
        let new_top = aligned_top + size;
        
        if new_top > N {
            return None;
        }
        
        let ptr = unsafe { self.buffer.as_mut_ptr().add(aligned_top) };
        self.top = new_top;
        Some(ptr)
    }
    
    /// Pop allocation from stack (LIFO)
    pub fn pop(&mut self, size: usize, align: usize) {
        let aligned_size = (size + align - 1) & !(align - 1);
        self.top = self.top.saturating_sub(aligned_size);
    }
    
    /// Get current stack usage
    pub fn used(&self) -> usize {
        self.top
    }
    
    /// Reset stack to empty
    pub fn reset(&mut self) {
        self.top = 0;
    }
}

/// Pool allocator for fixed-size objects
pub struct PoolAllocator<T> {
    free_list: Vec<*mut T>,
    chunks: Vec<Vec<T>>,
    chunk_size: usize,
}

impl<T> PoolAllocator<T> {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            free_list: Vec::new(),
            chunks: Vec::new(),
            chunk_size,
        }
    }
    
    /// Allocate an object from the pool
    pub fn alloc(&mut self) -> *mut T {
        if let Some(ptr) = self.free_list.pop() {
            return ptr;
        }
        
        // Need to allocate a new chunk
        self.chunks.push(Vec::with_capacity(self.chunk_size));
        let chunk = self.chunks.last_mut().unwrap();
        
        // Fill chunk with uninitialized objects
        for _ in 0..self.chunk_size {
            chunk.push(unsafe { std::mem::zeroed() }); // Not ideal, but needed for generic T
        }
        
        // Add all but one to free list
        for item in chunk.iter_mut().skip(1) {
            self.free_list.push(item as *mut T);
        }
        
        // Return the first one
        &mut chunk[0] as *mut T
    }
    
    /// Free an object back to the pool
    pub fn free(&mut self, ptr: *mut T) {
        // In a real implementation, you'd want to verify the pointer belongs to this pool
        self.free_list.push(ptr);
    }
    
    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        let total_capacity = self.chunks.len() * self.chunk_size;
        let free_count = self.free_list.len();
        let used_count = total_capacity - free_count;
        
        PoolStats {
            total_capacity,
            used_count,
            free_count,
            chunks: self.chunks.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_capacity: usize,
    pub used_count: usize,
    pub free_count: usize,
    pub chunks: usize,
}

/// Custom allocator that tracks allocation patterns
pub struct TrackingAllocator {
    inner: System,
    allocations: std::sync::atomic::AtomicUsize,
    deallocations: std::sync::atomic::AtomicUsize,
    bytes_allocated: std::sync::atomic::AtomicUsize,
}

impl Default for TrackingAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackingAllocator {
    pub const fn new() -> Self {
        Self {
            inner: System,
            allocations: std::sync::atomic::AtomicUsize::new(0),
            deallocations: std::sync::atomic::AtomicUsize::new(0),
            bytes_allocated: std::sync::atomic::AtomicUsize::new(0),
        }
    }
    
    pub fn stats(&self) -> AllocationStats {
        AllocationStats {
            allocations: self.allocations.load(std::sync::atomic::Ordering::Relaxed),
            deallocations: self.deallocations.load(std::sync::atomic::Ordering::Relaxed),
            bytes_allocated: self.bytes_allocated.load(std::sync::atomic::Ordering::Relaxed),
        }
    }
    
    pub fn reset_stats(&self) {
        self.allocations.store(0, std::sync::atomic::Ordering::Relaxed);
        self.deallocations.store(0, std::sync::atomic::Ordering::Relaxed);
        self.bytes_allocated.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

#[derive(Debug, Clone)]
pub struct AllocationStats {
    pub allocations: usize,
    pub deallocations: usize, 
    pub bytes_allocated: usize,
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);
        if !ptr.is_null() {
            self.allocations.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.bytes_allocated.fetch_add(layout.size(), std::sync::atomic::Ordering::Relaxed);
        }
        ptr
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.dealloc(ptr, layout);
        self.deallocations.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

/// Utilities for measuring allocation behavior
pub mod profiling {
    use std::time::Instant;
    
    /// Profile allocation behavior of a function
    pub fn profile_allocations<F, R>(f: F) -> (R, AllocationProfile)
    where
        F: FnOnce() -> R,
    {
        // This would need integration with a global tracking allocator
        let start_time = Instant::now();
        let result = f();
        let duration = start_time.elapsed();
        
        (result, AllocationProfile {
            duration,
            // In a real implementation, these would come from the tracking allocator
            allocations: 0,
            deallocations: 0,
            peak_memory: 0,
        })
    }
    
    #[derive(Debug)]
    pub struct AllocationProfile {
        pub duration: std::time::Duration,
        pub allocations: usize,
        pub deallocations: usize,
        pub peak_memory: usize,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_arena_allocator() {
        let mut arena = ArenaAllocator::new(1024);
        
        let layout = Layout::new::<u64>();
        let ptr1 = arena.alloc(layout).unwrap();
        let ptr2 = arena.alloc(layout).unwrap();
        
        assert_ne!(ptr1, ptr2);
        assert!(arena.used() > 0);
        
        arena.reset();
        assert_eq!(arena.used(), 0);
    }
    
    #[test]
    fn test_stack_allocator() {
        let mut stack = StackAllocator::<1024>::new();
        
        let ptr1 = stack.push(64, 8).unwrap();
        let ptr2 = stack.push(32, 4).unwrap();
        
        assert_ne!(ptr1, ptr2);
        
        stack.pop(32, 4);
        stack.pop(64, 8);
        assert_eq!(stack.used(), 0);
    }
    
    #[test]
    fn test_pool_allocator() {
        let mut pool = PoolAllocator::<u64>::new(10);
        
        let ptr1 = pool.alloc();
        let ptr2 = pool.alloc();
        
        assert_ne!(ptr1, ptr2);
        
        let stats = pool.stats();
        assert_eq!(stats.used_count, 2);
        
        pool.free(ptr1);
        let stats = pool.stats();
        assert_eq!(stats.free_count, 9); // 8 from initial chunk + 1 freed
    }
    
    #[test]
    fn test_thread_arena() {
        let layout = Layout::new::<u64>();
        let ptr = alloc_in_thread_arena(layout);
        assert!(ptr.is_some());
        
        let (used, capacity, util) = thread_arena_stats();
        assert!(used > 0);
        assert!(capacity > 0);
        assert!(util > 0.0);
        
        reset_thread_arena();
        let (used_after, _, _) = thread_arena_stats();
        assert_eq!(used_after, 0);
    }
}