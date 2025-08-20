//! SIMD optimizations for high-performance data processing
//! 
//! This module provides SIMD-accelerated operations for x86_64 targets,
//! with fallback implementations for other architectures including WASM.

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(target_arch = "wasm32")))]
use std::arch::x86_64::*;

/// SIMD-optimized text processing utilities
pub mod text_processing {
    use super::*;

    /// Fast byte search using SIMD when available
    pub fn find_delimiter(haystack: &[u8], needle: u8) -> Option<usize> {
        // Use SIMD if available on x86_64, fallback to scalar for WASM
        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(target_arch = "wasm32")))]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe { find_delimiter_avx2(haystack, needle) }
            } else if is_x86_feature_detected!("sse2") {
                unsafe { find_delimiter_sse2(haystack, needle) }
            } else {
                find_delimiter_scalar(haystack, needle)
            }
        }
        
        #[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), not(target_arch = "wasm32"))))]
        {
            find_delimiter_scalar(haystack, needle)
        }
    }

    // SIMD implementations only available on x86/x86_64 non-WASM targets
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(target_arch = "wasm32")))]
    mod simd_impl {
        use super::*;
        
        /// AVX2 implementation for finding delimiters
        #[target_feature(enable = "avx2")]
        pub(super) unsafe fn find_delimiter_avx2(haystack: &[u8], needle: u8) -> Option<usize> {
            const LANES: usize = 32;
            
            if haystack.len() < LANES {
                return find_delimiter_scalar(haystack, needle);
            }

            let needle_vec = _mm256_set1_epi8(needle as i8);
            let mut offset = 0;

            while offset + LANES <= haystack.len() {
                let data = _mm256_loadu_si256(haystack.as_ptr().add(offset) as *const _);
                let matches = _mm256_cmpeq_epi8(data, needle_vec);
                let mask = _mm256_movemask_epi8(matches);

                if mask != 0 {
                    let position = mask.trailing_zeros() as usize;
                    return Some(offset + position);
                }

                offset += LANES;
            }

            haystack[offset..]
                .iter()
                .position(|&b| b == needle)
                .map(|pos| offset + pos)
        }

        /// SSE2 implementation for finding delimiters
        #[target_feature(enable = "sse2")]
        pub(super) unsafe fn find_delimiter_sse2(haystack: &[u8], needle: u8) -> Option<usize> {
            const LANES: usize = 16;
            
            if haystack.len() < LANES {
                return find_delimiter_scalar(haystack, needle);
            }

            let needle_vec = _mm_set1_epi8(needle as i8);
            let mut offset = 0;

            while offset + LANES <= haystack.len() {
                let data = _mm_loadu_si128(haystack.as_ptr().add(offset) as *const _);
                let matches = _mm_cmpeq_epi8(data, needle_vec);
                let mask = _mm_movemask_epi8(matches);

                if mask != 0 {
                    let position = mask.trailing_zeros() as usize;
                    return Some(offset + position);
                }

                offset += LANES;
            }

            haystack[offset..]
                .iter()
                .position(|&b| b == needle)
                .map(|pos| offset + pos)
        }
    }

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(target_arch = "wasm32")))]
    use simd_impl::*;

    /// Scalar fallback implementation
    pub fn find_delimiter_scalar(haystack: &[u8], needle: u8) -> Option<usize> {
        haystack.iter().position(|&b| b == needle)
    }

    /// Fast case conversion using SIMD when available
    pub fn to_lowercase_fast(input: &str) -> String {
        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(target_arch = "wasm32")))]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe { to_lowercase_avx2(input) }
            } else if is_x86_feature_detected!("sse2") {
                unsafe { to_lowercase_sse2(input) }
            } else {
                input.to_lowercase()
            }
        }
        
        #[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), not(target_arch = "wasm32"))))]
        {
            input.to_lowercase()
        }
    }

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(target_arch = "wasm32")))]
    mod case_conversion {
        // Removed unused import: use super::*;
        
        #[target_feature(enable = "avx2")]
        pub(super) unsafe fn to_lowercase_avx2(input: &str) -> String {
            // For simplicity, fall back to standard conversion for complex UTF-8
            // A full implementation would need proper UTF-8 handling
            input.to_lowercase()
        }

        #[target_feature(enable = "sse2")]  
        pub(super) unsafe fn to_lowercase_sse2(input: &str) -> String {
            // For simplicity, fall back to standard conversion for complex UTF-8
            input.to_lowercase()
        }
    }

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(target_arch = "wasm32")))]
    use case_conversion::*;
}

/// SIMD-optimized numerical operations
pub mod numerical {
    use super::*;

    /// Fast sum calculation using SIMD when available
    pub fn sum_f32_slice(values: &[f32]) -> f32 {
        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(target_arch = "wasm32")))]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe { sum_f32_avx2(values) }
            } else if is_x86_feature_detected!("sse2") {
                unsafe { sum_f32_sse2(values) }
            } else {
                values.iter().sum()
            }
        }
        
        #[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), not(target_arch = "wasm32"))))]
        {
            values.iter().sum()
        }
    }

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(target_arch = "wasm32")))]
    mod simd_numerical {
        use super::*;
        
        #[target_feature(enable = "avx2")]
        pub(super) unsafe fn sum_f32_avx2(values: &[f32]) -> f32 {
            const LANES: usize = 8;
            
            if values.len() < LANES {
                return values.iter().sum();
            }

            let mut sum_vec = _mm256_setzero_ps();
            let mut offset = 0;

            while offset + LANES <= values.len() {
                let data = _mm256_loadu_ps(values.as_ptr().add(offset));
                sum_vec = _mm256_add_ps(sum_vec, data);
                offset += LANES;
            }

            // Horizontal sum of the vector
            let mut result = std::mem::MaybeUninit::<[f32; 8]>::uninit();
            _mm256_storeu_ps(result.as_mut_ptr() as *mut f32, sum_vec);
            let result = result.assume_init();
            
            let partial_sum: f32 = result.iter().sum();
            partial_sum + values[offset..].iter().sum::<f32>()
        }

        #[target_feature(enable = "sse2")]
        pub(super) unsafe fn sum_f32_sse2(values: &[f32]) -> f32 {
            const LANES: usize = 4;
            
            if values.len() < LANES {
                return values.iter().sum();
            }

            let mut sum_vec = _mm_setzero_ps();
            let mut offset = 0;

            while offset + LANES <= values.len() {
                let data = _mm_loadu_ps(values.as_ptr().add(offset));
                sum_vec = _mm_add_ps(sum_vec, data);
                offset += LANES;
            }

            // Horizontal sum of the vector
            let mut result = std::mem::MaybeUninit::<[f32; 4]>::uninit();
            _mm_storeu_ps(result.as_mut_ptr() as *mut f32, sum_vec);
            let result = result.assume_init();
            
            let partial_sum: f32 = result.iter().sum();
            partial_sum + values[offset..].iter().sum::<f32>()
        }
    }

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(target_arch = "wasm32")))]
    use simd_numerical::*;
}

/// Memory operations with SIMD optimization
pub mod memory {
    use super::*;

    /// Fast memory comparison using SIMD when available  
    pub fn compare_bytes(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(target_arch = "wasm32")))]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe { compare_bytes_avx2(a, b) }
            } else if is_x86_feature_detected!("sse2") {
                unsafe { compare_bytes_sse2(a, b) }
            } else {
                a == b
            }
        }
        
        #[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), not(target_arch = "wasm32"))))]
        {
            a == b
        }
    }

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(target_arch = "wasm32")))]
    mod simd_memory {
        use super::*;
        
        #[target_feature(enable = "avx2")]
        pub(super) unsafe fn compare_bytes_avx2(a: &[u8], b: &[u8]) -> bool {
            const LANES: usize = 32;
            
            if a.len() < LANES {
                return a == b;
            }

            let mut offset = 0;
            while offset + LANES <= a.len() {
                let data_a = _mm256_loadu_si256(a.as_ptr().add(offset) as *const _);
                let data_b = _mm256_loadu_si256(b.as_ptr().add(offset) as *const _);
                let comparison = _mm256_cmpeq_epi8(data_a, data_b);
                let mask = _mm256_movemask_epi8(comparison);
                
                if mask != -1 {
                    return false;
                }
                
                offset += LANES;
            }

            a[offset..] == b[offset..]
        }

        #[target_feature(enable = "sse2")]
        pub(super) unsafe fn compare_bytes_sse2(a: &[u8], b: &[u8]) -> bool {
            const LANES: usize = 16;
            
            if a.len() < LANES {
                return a == b;
            }

            let mut offset = 0;
            while offset + LANES <= a.len() {
                let data_a = _mm_loadu_si128(a.as_ptr().add(offset) as *const _);
                let data_b = _mm_loadu_si128(b.as_ptr().add(offset) as *const _);
                let comparison = _mm_cmpeq_epi8(data_a, data_b);
                let mask = _mm_movemask_epi8(comparison);
                
                if mask != 0xFFFF {
                    return false;
                }
                
                offset += LANES;
            }

            a[offset..] == b[offset..]
        }
    }

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(target_arch = "wasm32")))]
    use simd_memory::*;
}