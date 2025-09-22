//! SIMD optimizations for performance-critical operations

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// SIMD-optimized batch operations
pub struct SimdOps;

impl SimdOps {
    /// Fast memory copy using SIMD
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    #[inline(always)]
    pub unsafe fn fast_copy(src: &[u8], dst: &mut [u8]) {
        let len = src.len().min(dst.len());
        let chunks = len / 32;

        for i in 0..chunks {
            let offset = i * 32;
            let src_ptr = src.as_ptr().add(offset) as *const __m256i;
            let dst_ptr = dst.as_mut_ptr().add(offset) as *mut __m256i;

            let data = _mm256_loadu_si256(src_ptr);
            _mm256_storeu_si256(dst_ptr, data);
        }

        // Handle remainder
        let remainder_start = chunks * 32;
        for i in remainder_start..len {
            dst[i] = src[i];
        }
    }

    /// SIMD-accelerated sum of u64 values
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    #[inline(always)]
    pub unsafe fn sum_u64(values: &[u64]) -> u64 {
        if values.len() < 4 {
            return values.iter().sum();
        }

        let chunks = values.len() / 4;
        let mut sum = _mm256_setzero_si256();

        for i in 0..chunks {
            let ptr = values.as_ptr().add(i * 4) as *const __m256i;
            let chunk = _mm256_loadu_si256(ptr);
            sum = _mm256_add_epi64(sum, chunk);
        }

        // Extract sum from SIMD register
        let mut result = [0u64; 4];
        _mm256_storeu_si256(result.as_mut_ptr() as *mut __m256i, sum);
        let partial_sum = result[0] + result[1] + result[2] + result[3];

        // Add remainder
        let remainder_start = chunks * 4;
        partial_sum + values[remainder_start..].iter().sum::<u64>()
    }

    /// Prefetch data for better cache performance
    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    pub unsafe fn prefetch<T>(ptr: *const T) {
        #[cfg(target_feature = "sse")]
        _mm_prefetch(ptr as *const i8, _MM_HINT_T0);
    }
}

/// Non-SIMD fallback implementations
#[cfg(not(target_arch = "x86_64"))]
impl SimdOps {
    #[inline(always)]
    pub fn fast_copy(src: &[u8], dst: &mut [u8]) {
        let len = src.len().min(dst.len());
        dst[..len].copy_from_slice(&src[..len]);
    }

    #[inline(always)]
    pub fn sum_u64(values: &[u64]) -> u64 {
        values.iter().sum()
    }

    #[inline(always)]
    pub fn prefetch<T>(_ptr: *const T) {
        // No-op on non-x86_64
    }
}