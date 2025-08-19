//! SIMD optimizations for high-performance data processing

use std::arch::x86_64::*;

/// SIMD-optimized text processing utilities
pub mod text_processing {
    use super::*;

    /// Fast byte search using SIMD when available
    pub fn find_delimiter(haystack: &[u8], needle: u8) -> Option<usize> {
        // Use SIMD if available, fallback to scalar
        if is_x86_feature_detected!("avx2") {
            unsafe { find_delimiter_avx2(haystack, needle) }
        } else if is_x86_feature_detected!("sse2") {
            unsafe { find_delimiter_sse2(haystack, needle) }
        } else {
            find_delimiter_scalar(haystack, needle)
        }
    }

    /// AVX2 implementation for finding delimiters
    #[target_feature(enable = "avx2")]
    unsafe fn find_delimiter_avx2(haystack: &[u8], needle: u8) -> Option<usize> {
        const LANES: usize = 32;
        
        if haystack.len() < LANES {
            return find_delimiter_scalar(haystack, needle);
        }

        let needle_vec = _mm256_set1_epi8(needle as i8);
        let mut chunks = haystack.chunks_exact(LANES);
        let mut offset = 0;

        for chunk in &mut chunks {
            let chunk_vec = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
            let matches = _mm256_cmpeq_epi8(chunk_vec, needle_vec);
            let mask = _mm256_movemask_epi8(matches);

            if mask != 0 {
                let position = mask.trailing_zeros() as usize;
                return Some(offset + position);
            }

            offset += LANES;
        }

        // Handle remainder with scalar code
        chunks.remainder()
            .iter()
            .position(|&b| b == needle)
            .map(|pos| offset + pos)
    }

    /// SSE2 implementation for finding delimiters
    #[target_feature(enable = "sse2")]
    unsafe fn find_delimiter_sse2(haystack: &[u8], needle: u8) -> Option<usize> {
        const LANES: usize = 16;
        
        if haystack.len() < LANES {
            return find_delimiter_scalar(haystack, needle);
        }

        let needle_vec = _mm_set1_epi8(needle as i8);
        let mut chunks = haystack.chunks_exact(LANES);
        let mut offset = 0;

        for chunk in &mut chunks {
            let chunk_vec = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
            let matches = _mm_cmpeq_epi8(chunk_vec, needle_vec);
            let mask = _mm_movemask_epi8(matches);

            if mask != 0 {
                let position = mask.trailing_zeros() as usize;
                return Some(offset + position);
            }

            offset += LANES;
        }

        chunks.remainder()
            .iter()
            .position(|&b| b == needle)
            .map(|pos| offset + pos)
    }

    /// Scalar fallback implementation
    fn find_delimiter_scalar(haystack: &[u8], needle: u8) -> Option<usize> {
        haystack.iter().position(|&b| b == needle)
    }

    /// Fast case conversion using SIMD
    pub fn to_uppercase_ascii(input: &[u8], output: &mut [u8]) -> Result<(), &'static str> {
        if input.len() != output.len() {
            return Err("Input and output slices must have the same length");
        }

        if is_x86_feature_detected!("avx2") {
            unsafe { to_uppercase_ascii_avx2(input, output) }
        } else if is_x86_feature_detected!("sse2") {
            unsafe { to_uppercase_ascii_sse2(input, output) }
        } else {
            to_uppercase_ascii_scalar(input, output);
        }

        Ok(())
    }

    #[target_feature(enable = "avx2")]
    unsafe fn to_uppercase_ascii_avx2(input: &[u8], output: &mut [u8]) {
        const LANES: usize = 32;
        
        let lower_a = _mm256_set1_epi8(b'a' as i8);
        let lower_z = _mm256_set1_epi8(b'z' as i8);
        let case_diff = _mm256_set1_epi8((b'a' - b'A') as i8);

        let mut i = 0;
        while i + LANES <= input.len() {
            let chunk = _mm256_loadu_si256(input.as_ptr().add(i) as *const __m256i);
            
            // Check if bytes are lowercase letters
            let ge_a = _mm256_cmpgt_epi8(chunk, _mm256_sub_epi8(lower_a, _mm256_set1_epi8(1)));
            let le_z = _mm256_cmpgt_epi8(_mm256_add_epi8(lower_z, _mm256_set1_epi8(1)), chunk);
            let is_lower = _mm256_and_si256(ge_a, le_z);
            
            // Convert to uppercase by subtracting case difference
            let to_subtract = _mm256_and_si256(case_diff, is_lower);
            let result = _mm256_sub_epi8(chunk, to_subtract);
            
            _mm256_storeu_si256(output.as_mut_ptr().add(i) as *mut __m256i, result);
            i += LANES;
        }

        // Handle remainder
        for j in i..input.len() {
            output[j] = if input[j].is_ascii_lowercase() {
                input[j] - (b'a' - b'A')
            } else {
                input[j]
            };
        }
    }

    #[target_feature(enable = "sse2")]
    unsafe fn to_uppercase_ascii_sse2(input: &[u8], output: &mut [u8]) {
        const LANES: usize = 16;
        
        let lower_a = _mm_set1_epi8(b'a' as i8);
        let lower_z = _mm_set1_epi8(b'z' as i8);
        let case_diff = _mm_set1_epi8((b'a' - b'A') as i8);

        let mut i = 0;
        while i + LANES <= input.len() {
            let chunk = _mm_loadu_si128(input.as_ptr().add(i) as *const __m128i);
            
            let ge_a = _mm_cmpgt_epi8(chunk, _mm_sub_epi8(lower_a, _mm_set1_epi8(1)));
            let le_z = _mm_cmpgt_epi8(_mm_add_epi8(lower_z, _mm_set1_epi8(1)), chunk);
            let is_lower = _mm_and_si128(ge_a, le_z);
            
            let to_subtract = _mm_and_si128(case_diff, is_lower);
            let result = _mm_sub_epi8(chunk, to_subtract);
            
            _mm_storeu_si128(output.as_mut_ptr().add(i) as *mut __m128i, result);
            i += LANES;
        }

        for j in i..input.len() {
            output[j] = if input[j].is_ascii_lowercase() {
                input[j] - (b'a' - b'A')
            } else {
                input[j]
            };
        }
    }

    fn to_uppercase_ascii_scalar(input: &[u8], output: &mut [u8]) {
        for (i, &byte) in input.iter().enumerate() {
            output[i] = if byte.is_ascii_lowercase() {
                byte - (b'a' - b'A')
            } else {
                byte
            };
        }
    }
}

/// SIMD-optimized numerical operations
pub mod numerical {
    use super::*;

    /// Fast checksum calculation using SIMD
    pub fn calculate_checksum(data: &[u8]) -> u32 {
        if is_x86_feature_detected!("avx2") {
            unsafe { checksum_avx2(data) }
        } else if is_x86_feature_detected!("sse2") {
            unsafe { checksum_sse2(data) }
        } else {
            checksum_scalar(data)
        }
    }

    #[target_feature(enable = "avx2")]
    unsafe fn checksum_avx2(data: &[u8]) -> u32 {
        const LANES: usize = 32;
        
        let mut sum = _mm256_setzero_si256();
        let mut chunks = data.chunks_exact(LANES);

        for chunk in &mut chunks {
            let chunk_vec = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
            
            // Unpack bytes to words to avoid overflow
            let low = _mm256_unpacklo_epi8(chunk_vec, _mm256_setzero_si256());
            let high = _mm256_unpackhi_epi8(chunk_vec, _mm256_setzero_si256());
            
            // Add to accumulator
            sum = _mm256_add_epi16(sum, low);
            sum = _mm256_add_epi16(sum, high);
        }

        // Horizontal sum
        let sum_lo = _mm256_extracti128_si256(sum, 0);
        let sum_hi = _mm256_extracti128_si256(sum, 1);
        let sum_combined = _mm_add_epi16(sum_lo, sum_hi);
        
        // Further reduction
        let sum_64 = _mm_add_epi16(sum_combined, _mm_srli_si128(sum_combined, 8));
        let sum_32 = _mm_add_epi16(sum_64, _mm_srli_si128(sum_64, 4));
        let sum_16 = _mm_add_epi16(sum_32, _mm_srli_si128(sum_32, 2));
        
        let mut result = _mm_extract_epi16(sum_16, 0) as u32;

        // Handle remainder
        for &byte in chunks.remainder() {
            result += byte as u32;
        }

        result
    }

    #[target_feature(enable = "sse2")]
    unsafe fn checksum_sse2(data: &[u8]) -> u32 {
        const LANES: usize = 16;
        
        let mut sum = _mm_setzero_si128();
        let mut chunks = data.chunks_exact(LANES);

        for chunk in &mut chunks {
            let chunk_vec = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
            
            let low = _mm_unpacklo_epi8(chunk_vec, _mm_setzero_si128());
            let high = _mm_unpackhi_epi8(chunk_vec, _mm_setzero_si128());
            
            sum = _mm_add_epi16(sum, low);
            sum = _mm_add_epi16(sum, high);
        }

        // Horizontal sum
        let sum_64 = _mm_add_epi16(sum, _mm_srli_si128(sum, 8));
        let sum_32 = _mm_add_epi16(sum_64, _mm_srli_si128(sum_64, 4));
        let sum_16 = _mm_add_epi16(sum_32, _mm_srli_si128(sum_32, 2));
        
        let mut result = _mm_extract_epi16(sum_16, 0) as u32;

        for &byte in chunks.remainder() {
            result += byte as u32;
        }

        result
    }

    fn checksum_scalar(data: &[u8]) -> u32 {
        data.iter().map(|&b| b as u32).sum()
    }

    /// Vectorized byte counting
    pub fn count_byte(haystack: &[u8], needle: u8) -> usize {
        if is_x86_feature_detected!("avx2") {
            unsafe { count_byte_avx2(haystack, needle) }
        } else if is_x86_feature_detected!("sse2") {
            unsafe { count_byte_sse2(haystack, needle) }
        } else {
            count_byte_scalar(haystack, needle)
        }
    }

    #[target_feature(enable = "avx2")]
    unsafe fn count_byte_avx2(haystack: &[u8], needle: u8) -> usize {
        const LANES: usize = 32;
        
        let needle_vec = _mm256_set1_epi8(needle as i8);
        let mut total_count = 0;
        let mut chunks = haystack.chunks_exact(LANES);

        for chunk in &mut chunks {
            let chunk_vec = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
            let matches = _mm256_cmpeq_epi8(chunk_vec, needle_vec);
            
            // Count set bits
            let mask = _mm256_movemask_epi8(matches);
            total_count += mask.count_ones() as usize;
        }

        // Handle remainder
        total_count + chunks.remainder().iter().filter(|&&b| b == needle).count()
    }

    #[target_feature(enable = "sse2")]
    unsafe fn count_byte_sse2(haystack: &[u8], needle: u8) -> usize {
        const LANES: usize = 16;
        
        let needle_vec = _mm_set1_epi8(needle as i8);
        let mut total_count = 0;
        let mut chunks = haystack.chunks_exact(LANES);

        for chunk in &mut chunks {
            let chunk_vec = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
            let matches = _mm_cmpeq_epi8(chunk_vec, needle_vec);
            
            let mask = _mm_movemask_epi8(matches);
            total_count += mask.count_ones() as usize;
        }

        total_count + chunks.remainder().iter().filter(|&&b| b == needle).count()
    }

    fn count_byte_scalar(haystack: &[u8], needle: u8) -> usize {
        haystack.iter().filter(|&&b| b == needle).count()
    }
}

/// SIMD-optimized data validation
pub mod validation {
    use super::*;

    /// Fast ASCII validation using SIMD
    pub fn is_ascii(data: &[u8]) -> bool {
        if is_x86_feature_detected!("avx2") {
            unsafe { is_ascii_avx2(data) }
        } else if is_x86_feature_detected!("sse2") {
            unsafe { is_ascii_sse2(data) }
        } else {
            is_ascii_scalar(data)
        }
    }

    #[target_feature(enable = "avx2")]
    unsafe fn is_ascii_avx2(data: &[u8]) -> bool {
        const LANES: usize = 32;
        
        let mut chunks = data.chunks_exact(LANES);
        let ascii_mask = _mm256_set1_epi8(0x80u8 as i8);

        for chunk in &mut chunks {
            let chunk_vec = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
            let test = _mm256_and_si256(chunk_vec, ascii_mask);
            
            if !_mm256_testz_si256(test, test) {
                return false;
            }
        }

        // Check remainder
        chunks.remainder().iter().all(|&b| b.is_ascii())
    }

    #[target_feature(enable = "sse2")]
    unsafe fn is_ascii_sse2(data: &[u8]) -> bool {
        const LANES: usize = 16;
        
        let mut chunks = data.chunks_exact(LANES);
        let ascii_mask = _mm_set1_epi8(0x80u8 as i8);

        for chunk in &mut chunks {
            let chunk_vec = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
            let test = _mm_and_si128(chunk_vec, ascii_mask);
            
            if _mm_movemask_epi8(test) != 0 {
                return false;
            }
        }

        chunks.remainder().iter().all(|&b| b.is_ascii())
    }

    fn is_ascii_scalar(data: &[u8]) -> bool {
        data.iter().all(|&b| b.is_ascii())
    }
}

/// Portable SIMD operations using std::simd when available
#[cfg(feature = "portable_simd")]
pub mod portable {
    use std::simd::*;

    /// Portable SIMD sum using std::simd
    pub fn sum_u8(data: &[u8]) -> u32 {
        const LANES: usize = 64;
        
        let mut sum = u8x64::splat(0);
        let mut chunks = data.chunks_exact(LANES);

        for chunk in &mut chunks {
            let chunk_vec = u8x64::from_slice(chunk);
            sum = sum.saturating_add(chunk_vec);
        }

        let sum_array: [u8; LANES] = sum.into();
        let mut total: u32 = sum_array.iter().map(|&x| x as u32).sum();

        // Handle remainder
        for &byte in chunks.remainder() {
            total += byte as u32;
        }

        total
    }

    /// Portable SIMD maximum
    pub fn max_u8(data: &[u8]) -> u8 {
        const LANES: usize = 64;
        
        if data.is_empty() {
            return 0;
        }

        let mut max = u8x64::splat(0);
        let mut chunks = data.chunks_exact(LANES);

        for chunk in &mut chunks {
            let chunk_vec = u8x64::from_slice(chunk);
            max = max.simd_max(chunk_vec);
        }

        let max_array: [u8; LANES] = max.into();
        let simd_max = *max_array.iter().max().unwrap();
        
        // Handle remainder
        let remainder_max = chunks.remainder().iter().copied().max().unwrap_or(0);
        
        simd_max.max(remainder_max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_delimiter() {
        let data = b"Hello, World! This is a test.";
        
        assert_eq!(text_processing::find_delimiter(data, b','), Some(5));
        assert_eq!(text_processing::find_delimiter(data, b'!'), Some(12));
        assert_eq!(text_processing::find_delimiter(data, b'z'), None);
    }

    #[test]
    fn test_to_uppercase() {
        let input = b"hello world";
        let mut output = vec![0u8; input.len()];
        
        text_processing::to_uppercase_ascii(input, &mut output).unwrap();
        assert_eq!(&output, b"HELLO WORLD");
    }

    #[test]
    fn test_checksum() {
        let data = b"test data for checksum";
        let checksum = numerical::calculate_checksum(data);
        
        // Verify it's not zero and consistent
        assert!(checksum > 0);
        assert_eq!(checksum, numerical::calculate_checksum(data));
    }

    #[test]
    fn test_count_byte() {
        let data = b"hello world hello";
        let count = numerical::count_byte(data, b'l');
        assert_eq!(count, 6);
    }

    #[test]
    fn test_ascii_validation() {
        assert!(validation::is_ascii(b"Hello, World!"));
        assert!(!validation::is_ascii("Hello, 世界!".as_bytes()));
    }

    #[cfg(feature = "portable_simd")]
    #[test]
    fn test_portable_simd() {
        let data = [1u8, 2, 3, 4, 5];
        assert_eq!(portable::sum_u8(&data), 15);
        assert_eq!(portable::max_u8(&data), 5);
    }
}