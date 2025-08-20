//! Advanced performance techniques for high-throughput systems

use std::fs::File;
use std::path::Path;
use memmap2::{Mmap, MmapMut, MmapOptions};
use crate::{Result, WritemagicError};

/// Memory-mapped file wrapper for zero-copy data access
pub struct MappedFile {
    mmap: Mmap,
    len: usize,
}

impl MappedFile {
    /// Open and memory-map a file for reading
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)
            .map_err(|e| WritemagicError::internal_with_source("Failed to open file", e))?;
        
        let mmap = unsafe {
            MmapOptions::new()
                .map(&file)
                .map_err(|e| WritemagicError::internal_with_source("Failed to memory-map file", e))?
        };
        
        let len = mmap.len();
        
        Ok(Self { mmap, len })
    }
    
    /// Get the entire file as a slice
    pub fn as_slice(&self) -> &[u8] {
        &self.mmap[..]
    }
    
    /// Get a slice of the file
    pub fn slice(&self, start: usize, len: usize) -> Option<&[u8]> {
        if start + len <= self.len {
            Some(&self.mmap[start..start + len])
        } else {
            None
        }
    }
    
    /// Parse records from the mapped file without copying
    pub fn parse_records<T>(&self, record_size: usize) -> impl Iterator<Item = &[u8]> + '_ {
        self.mmap.chunks_exact(record_size)
    }
    
    /// Find patterns in the mapped data using SIMD
    pub fn find_pattern(&self, pattern: &[u8]) -> Vec<usize> {
        let mut positions = Vec::new();
        let data = self.as_slice();
        
        if pattern.is_empty() || pattern.len() > data.len() {
            return positions;
        }
        
        // Use Boyer-Moore-like search for larger patterns
        if pattern.len() > 4 {
            positions.extend(self.boyer_moore_search(pattern));
        } else {
            // Use SIMD search for small patterns
            positions.extend(self.simd_search(pattern[0]));
        }
        
        positions
    }
    
    fn boyer_moore_search(&self, pattern: &[u8]) -> Vec<usize> {
        let mut positions = Vec::new();
        let text = self.as_slice();
        
        if pattern.len() > text.len() {
            return positions;
        }
        
        // Build bad character table
        let mut bad_char = [pattern.len(); 256];
        for (i, &byte) in pattern.iter().enumerate().take(pattern.len() - 1) {
            bad_char[byte as usize] = pattern.len() - 1 - i;
        }
        
        let mut skip = 0;
        while skip <= text.len() - pattern.len() {
            let mut j = pattern.len() - 1;
            
            while j > 0 && pattern[j] == text[skip + j] {
                j -= 1;
            }
            
            if j == 0 && pattern[0] == text[skip] {
                positions.push(skip);
                skip += pattern.len();
            } else {
                skip += bad_char[text[skip + j] as usize].max(1);
            }
        }
        
        positions
    }
    
    fn simd_search(&self, byte: u8) -> Vec<usize> {
        crate::simd_optimizations::text_processing::find_delimiter(self.as_slice(), byte)
            .into_iter()
            .collect()
    }
}

/// Mutable memory-mapped file for zero-copy writes
pub struct MappedFileMut {
    mmap: MmapMut,
    len: usize,
}

impl MappedFileMut {
    /// Create and memory-map a file for writing
    pub fn create<P: AsRef<Path>>(path: P, size: usize) -> Result<Self> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|e| WritemagicError::internal_with_source("Failed to create file", e))?;
        
        file.set_len(size as u64)
            .map_err(|e| WritemagicError::internal_with_source("Failed to set file size", e))?;
        
        let mmap = unsafe {
            MmapOptions::new()
                .map_mut(&file)
                .map_err(|e| WritemagicError::internal_with_source("Failed to memory-map file", e))?
        };
        
        Ok(Self { mmap, len: size })
    }
    
    /// Get the entire file as a mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.mmap[..]
    }
    
    /// Write data at a specific offset
    pub fn write_at(&mut self, offset: usize, data: &[u8]) -> Result<()> {
        if offset + data.len() > self.len {
            return Err(WritemagicError::validation("Write would exceed file bounds"));
        }
        
        self.mmap[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }
    
    /// Flush changes to disk
    pub fn flush(&self) -> Result<()> {
        self.mmap.flush()
            .map_err(|e| WritemagicError::internal_with_source("Failed to flush memory-mapped file", e))
    }
    
    /// Flush a specific range to disk
    pub fn flush_range(&self, offset: usize, len: usize) -> Result<()> {
        if offset + len > self.len {
            return Err(WritemagicError::validation("Flush range exceeds file bounds"));
        }
        
        self.mmap.flush_range(offset, len)
            .map_err(|e| WritemagicError::internal_with_source("Failed to flush range", e))
    }
}

/// High-performance custom serialization for hot paths
pub mod fast_serialization {
    
    /// Custom serializer that writes directly to a buffer without intermediate allocations
    pub struct FastSerializer {
        buffer: Vec<u8>,
        position: usize,
    }
    
    impl FastSerializer {
        pub fn new(capacity: usize) -> Self {
            Self {
                buffer: Vec::with_capacity(capacity),
                position: 0,
            }
        }
        
        pub fn with_buffer(mut buffer: Vec<u8>) -> Self {
            buffer.clear();
            Self { buffer, position: 0 }
        }
        
        /// Write raw bytes
        pub fn write_bytes(&mut self, data: &[u8]) {
            self.ensure_capacity(data.len());
            
            let end = self.position + data.len();
            if end > self.buffer.len() {
                self.buffer.resize(end, 0);
            }
            
            self.buffer[self.position..end].copy_from_slice(data);
            self.position = end;
        }
        
        /// Write a u32 in little-endian format
        pub fn write_u32_le(&mut self, value: u32) {
            self.write_bytes(&value.to_le_bytes());
        }
        
        /// Write a u64 in little-endian format
        pub fn write_u64_le(&mut self, value: u64) {
            self.write_bytes(&value.to_le_bytes());
        }
        
        /// Write a length-prefixed string
        pub fn write_string(&mut self, s: &str) {
            let bytes = s.as_bytes();
            self.write_u32_le(bytes.len() as u32);
            self.write_bytes(bytes);
        }
        
        /// Write a variable-length integer (varint)
        pub fn write_varint(&mut self, mut value: u64) {
            while value >= 0x80 {
                self.write_bytes(&[(value & 0x7F | 0x80) as u8]);
                value >>= 7;
            }
            self.write_bytes(&[value as u8]);
        }
        
        fn ensure_capacity(&mut self, additional: usize) {
            let required = self.position + additional;
            if self.buffer.capacity() < required {
                let new_capacity = (required * 2).next_power_of_two();
                self.buffer.reserve(new_capacity - self.buffer.capacity());
            }
        }
        
        /// Get the serialized data
        pub fn finish(mut self) -> Vec<u8> {
            self.buffer.truncate(self.position);
            self.buffer
        }
        
        /// Get current position
        pub fn position(&self) -> usize {
            self.position
        }
        
        /// Reset for reuse
        pub fn reset(&mut self) {
            self.buffer.clear();
            self.position = 0;
        }
    }
    
    /// Custom deserializer for zero-copy reading
    pub struct FastDeserializer<'a> {
        data: &'a [u8],
        position: usize,
    }
    
    impl<'a> FastDeserializer<'a> {
        pub fn new(data: &'a [u8]) -> Self {
            Self { data, position: 0 }
        }
        
        /// Read raw bytes
        pub fn read_bytes(&mut self, len: usize) -> Option<&'a [u8]> {
            if self.position + len <= self.data.len() {
                let slice = &self.data[self.position..self.position + len];
                self.position += len;
                Some(slice)
            } else {
                None
            }
        }
        
        /// Read a u32 in little-endian format
        pub fn read_u32_le(&mut self) -> Option<u32> {
            self.read_bytes(4).map(|bytes| {
                u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
            })
        }
        
        /// Read a u64 in little-endian format
        pub fn read_u64_le(&mut self) -> Option<u64> {
            self.read_bytes(8).map(|bytes| {
                let mut array = [0u8; 8];
                array.copy_from_slice(bytes);
                u64::from_le_bytes(array)
            })
        }
        
        /// Read a length-prefixed string
        pub fn read_string(&mut self) -> Option<&'a str> {
            let len = self.read_u32_le()? as usize;
            let bytes = self.read_bytes(len)?;
            std::str::from_utf8(bytes).ok()
        }
        
        /// Read a variable-length integer
        pub fn read_varint(&mut self) -> Option<u64> {
            let mut value = 0u64;
            let mut shift = 0;
            
            loop {
                if shift >= 64 {
                    return None; // Overflow
                }
                
                let byte = self.read_bytes(1)?[0];
                value |= ((byte & 0x7F) as u64) << shift;
                
                if byte & 0x80 == 0 {
                    break;
                }
                
                shift += 7;
            }
            
            Some(value)
        }
        
        /// Get remaining bytes
        pub fn remaining(&self) -> &'a [u8] {
            &self.data[self.position..]
        }
        
        /// Check if at end of data
        pub fn is_empty(&self) -> bool {
            self.position >= self.data.len()
        }
    }
}

/// High-performance batch processing utilities
pub mod batch_processing {
    // Remove unused import since parallel processing is not currently used
    // use rayon::prelude::*;
    
    /// Process large datasets in parallel chunks with optimal memory usage
    pub struct BatchProcessor<T> {
        chunk_size: usize,
        worker_threads: usize,
        _phantom: std::marker::PhantomData<T>,
    }
    
    impl<T> BatchProcessor<T> 
    where
        T: Send + Sync,
    {
        pub fn new(chunk_size: usize) -> Self {
            Self {
                chunk_size,
                #[cfg(not(target_arch = "wasm32"))]
                worker_threads: num_cpus::get(),
                #[cfg(target_arch = "wasm32")]
                worker_threads: 1, // Single-threaded for WASM
                _phantom: std::marker::PhantomData,
            }
        }
        
        pub fn with_threads(mut self, threads: usize) -> Self {
            self.worker_threads = threads;
            self
        }
        
        /// Process items in parallel batches
        pub fn process_parallel<F, R>(&self, items: &[T], processor: F) -> Vec<R>
        where
            F: Fn(&T) -> R + Send + Sync,
            R: Send,
        {
            #[cfg(not(target_arch = "wasm32"))]
            {
                use rayon::prelude::*;
                
                // Configure rayon thread pool
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(self.worker_threads)
                    .build()
                    .unwrap();
                
                pool.install(|| {
                    items
                        .par_chunks(self.chunk_size)
                        .flat_map(|chunk| {
                            chunk.iter().map(&processor).collect::<Vec<_>>()
                        })
                        .collect()
                })
            }
            
            #[cfg(target_arch = "wasm32")]
            {
                // Single-threaded fallback for WASM
                items.iter().map(processor).collect()
            }
        }
        
        /// Process items with a stateful processor that can accumulate results
        pub fn process_with_state<F, S, R>(&self, items: &[T], init_state: S, processor: F) -> Vec<R>
        where
            F: Fn(&mut S, &T) -> Option<R> + Send + Sync,
            S: Clone + Send + Sync,
            R: Send,
        {
            #[cfg(not(target_arch = "wasm32"))]
            {
                use rayon::prelude::*;
                
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(self.worker_threads)
                    .build()
                    .unwrap();
                
                pool.install(|| {
                    items
                        .par_chunks(self.chunk_size)
                        .flat_map(|chunk| {
                            let mut local_state = init_state.clone();
                            chunk
                                .iter()
                                .filter_map(|item| processor(&mut local_state, item))
                            .collect::<Vec<_>>()
                    })
                    .collect()
                })
            }
            
            #[cfg(target_arch = "wasm32")]
            {
                // Single-threaded fallback for WASM
                let mut state = init_state;
                items.iter().filter_map(|item| processor(&mut state, item)).collect()
            }
        }
    }
}

/// Lock-free data structures for high-concurrency scenarios
pub mod lock_free {
    use crossbeam::epoch::{self, Atomic, Owned};
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    /// Lock-free queue implementation using epoch-based memory management
    pub struct LockFreeQueue<T> {
        head: Atomic<Node<T>>,
        tail: Atomic<Node<T>>,
        size: AtomicUsize,
    }
    
    struct Node<T> {
        data: Option<T>,
        next: Atomic<Node<T>>,
    }
    
    impl<T> LockFreeQueue<T> {
        pub fn new() -> Self {
            let sentinel = Owned::new(Node {
                data: None,
                next: Atomic::null(),
            });
            
            let sentinel_ptr = unsafe { sentinel.into_shared(epoch::unprotected()) };
            
            Self {
                head: Atomic::from(sentinel_ptr),
                tail: Atomic::from(sentinel_ptr),
                size: AtomicUsize::new(0),
            }
        }
        
        pub fn enqueue(&self, item: T) {
            let mut new_node = Owned::new(Node {
                data: Some(item),
                next: Atomic::null(),
            });
            
            let guard = epoch::pin();
            
            loop {
                let tail = self.tail.load(Ordering::Acquire, &guard);
                let next = unsafe { tail.deref() }.next.load(Ordering::Acquire, &guard);
                
                if next.is_null() {
                    match unsafe { tail.deref() }.next.compare_exchange(
                        next,
                        new_node,
                        Ordering::Release,
                        Ordering::Relaxed,
                        &guard,
                    ) {
                        Ok(new_node_ptr) => {
                            // Try to swing tail to new node
                            let _ = self.tail.compare_exchange(
                                tail,
                                new_node_ptr,
                                Ordering::Release,
                                Ordering::Relaxed,
                                &guard,
                            );
                            break;
                        }
                        Err(e) => {
                            new_node = e.new;
                        }
                    }
                } else {
                    // Help advance tail
                    let _ = self.tail.compare_exchange(
                        tail,
                        next,
                        Ordering::Release,
                        Ordering::Relaxed,
                        &guard,
                    );
                }
            }
            
            self.size.fetch_add(1, Ordering::Relaxed);
        }
        
        pub fn dequeue(&self) -> Option<T> {
            let guard = epoch::pin();
            
            loop {
                let head = self.head.load(Ordering::Acquire, &guard);
                let tail = self.tail.load(Ordering::Acquire, &guard);
                let next = unsafe { head.deref() }.next.load(Ordering::Acquire, &guard);
                
                if head == tail {
                    if next.is_null() {
                        return None; // Queue is empty
                    }
                    // Help advance tail
                    let _ = self.tail.compare_exchange(
                        tail,
                        next,
                        Ordering::Release,
                        Ordering::Relaxed,
                        &guard,
                    );
                } else {
                    if next.is_null() {
                        continue; // Inconsistent state, retry
                    }
                    
                    let data = unsafe { next.deref().data.as_ref().unwrap() };
                    
                    // Try to move head forward
                    if self.head.compare_exchange(
                        head,
                        next,
                        Ordering::Release,
                        Ordering::Relaxed,
                        &guard,
                    ).is_ok() {
                        unsafe {
                            guard.defer_destroy(head);
                            let result = std::ptr::read(data);
                            self.size.fetch_sub(1, Ordering::Relaxed);
                            return Some(result);
                        }
                    }
                }
            }
        }
        
        pub fn len(&self) -> usize {
            self.size.load(Ordering::Relaxed)
        }
        
        pub fn is_empty(&self) -> bool {
            self.size.load(Ordering::Relaxed) == 0
        }
    }
    
    impl<T> Default for LockFreeQueue<T> {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_memory_mapped_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Hello, World!").unwrap();
        temp_file.flush().unwrap();
        
        let mapped = MappedFile::open(temp_file.path()).unwrap();
        assert_eq!(mapped.as_slice(), b"Hello, World!");
        
        let slice = mapped.slice(0, 5).unwrap();
        assert_eq!(slice, b"Hello");
    }
    
    #[test]
    fn test_fast_serialization() {
        let mut serializer = fast_serialization::FastSerializer::new(1024);
        
        serializer.write_u32_le(42);
        serializer.write_string("Hello");
        serializer.write_varint(12345);
        
        let data = serializer.finish();
        
        let mut deserializer = fast_serialization::FastDeserializer::new(&data);
        assert_eq!(deserializer.read_u32_le().unwrap(), 42);
        assert_eq!(deserializer.read_string().unwrap(), "Hello");
        assert_eq!(deserializer.read_varint().unwrap(), 12345);
    }
    
    #[test]
    fn test_batch_processor() {
        let processor = batch_processing::BatchProcessor::new(100);
        let data: Vec<u32> = (0..1000).collect();
        
        let results = processor.process_parallel(&data, |&x| x * 2);
        
        assert_eq!(results.len(), 1000);
        assert_eq!(results[0], 0);
        assert_eq!(results[999], 1998);
    }
    
    #[test]
    fn test_lock_free_queue() {
        let queue = lock_free::LockFreeQueue::new();
        
        queue.enqueue(1);
        queue.enqueue(2);
        queue.enqueue(3);
        
        assert_eq!(queue.len(), 3);
        assert_eq!(queue.dequeue(), Some(1));
        assert_eq!(queue.dequeue(), Some(2));
        assert_eq!(queue.dequeue(), Some(3));
        assert_eq!(queue.dequeue(), None);
        assert!(queue.is_empty());
    }
}