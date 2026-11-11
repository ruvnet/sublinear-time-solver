//! SIMD optimizations and cache-aligned data structures for nano-agents

#[cfg(all(target_arch = "x86_64", not(target_family = "wasm")))]
use std::arch::x86_64::*;
use std::mem;

/// Vector of `f32` used by the SIMD helpers.
///
/// `repr(align(64))` aligns this struct, not the heap buffer behind `data`;
/// the buffer has only the alignment of `f32`. The SIMD paths therefore use
/// unaligned loads and stores and are sound for any allocation.
#[repr(align(64))]
pub struct AlignedVector {
    data: Vec<f32>,
    capacity: usize,
}

impl AlignedVector {
    /// Create new cache-aligned vector with specified capacity
    pub fn new(capacity: usize) -> Self {
        let aligned_capacity = (capacity + 15) & !15; // Round up to multiple of 16
        let mut data = Vec::with_capacity(aligned_capacity);
        data.resize(aligned_capacity, 0.0);

        Self {
            data,
            capacity: aligned_capacity,
        }
    }

    /// Get raw pointer for SIMD operations
    pub fn as_ptr(&self) -> *const f32 {
        self.data.as_ptr()
    }

    /// Get mutable raw pointer for SIMD operations
    pub fn as_mut_ptr(&mut self) -> *mut f32 {
        self.data.as_mut_ptr()
    }

    /// Get length of vector
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if vector is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// SIMD-accelerated vector addition (x86_64 only)
    ///
    /// # Safety
    ///
    /// The CPU must support AVX2 (check with `is_x86_feature_detected!("avx2")`).
    /// No alignment is required: since 0.3.1 every load and store is unaligned
    /// (`loadu`/`storeu`). 0.3.0 used aligned loads on a `Vec<f32>` buffer that is
    /// only guaranteed 4-byte alignment, which faulted (SIGSEGV) on most allocations.
    #[cfg(all(target_arch = "x86_64", not(target_family = "wasm")))]
    #[target_feature(enable = "avx2")]
    pub unsafe fn simd_add(&mut self, other: &AlignedVector) -> Result<(), &'static str> {
        if self.len() != other.len() {
            return Err("Vector lengths must match");
        }

        let len = self.len();
        let chunks = len / 8; // Process 8 f32s at a time with AVX2

        let self_ptr = self.as_mut_ptr();
        let other_ptr = other.as_ptr();

        // Process chunks of 8 elements
        for i in 0..chunks {
            let offset = i * 8;

            // Load 8 f32 values from each vector
            let a = _mm256_loadu_ps(self_ptr.add(offset));
            let b = _mm256_loadu_ps(other_ptr.add(offset));

            // Perform SIMD addition
            let result = _mm256_add_ps(a, b);

            // Store result back
            _mm256_storeu_ps(self_ptr.add(offset), result);
        }

        // Handle remaining elements
        for i in (chunks * 8)..len {
            *self_ptr.add(i) += *other_ptr.add(i);
        }

        Ok(())
    }

    /// Fallback vector addition for WASM and other targets
    #[cfg(any(not(target_arch = "x86_64"), target_family = "wasm"))]
    pub fn simd_add(&mut self, other: &AlignedVector) -> Result<(), &'static str> {
        if self.len() != other.len() {
            return Err("Vector lengths must match");
        }

        for i in 0..self.len() {
            self.data[i] += other.data[i];
        }

        Ok(())
    }

    /// SIMD-accelerated dot product (x86_64 only)
    ///
    /// # Safety
    ///
    /// The CPU must support AVX2 (check with `is_x86_feature_detected!("avx2")`).
    /// No alignment is required: since 0.3.1 every load and store is unaligned
    /// (`loadu`/`storeu`). 0.3.0 used aligned loads on a `Vec<f32>` buffer that is
    /// only guaranteed 4-byte alignment, which faulted (SIGSEGV) on most allocations.
    #[cfg(all(target_arch = "x86_64", not(target_family = "wasm")))]
    #[target_feature(enable = "avx2")]
    pub unsafe fn simd_dot(&self, other: &AlignedVector) -> Result<f32, &'static str> {
        if self.len() != other.len() {
            return Err("Vector lengths must match");
        }

        let len = self.len();
        let chunks = len / 8;

        let self_ptr = self.as_ptr();
        let other_ptr = other.as_ptr();

        // Accumulator for sum
        let mut sum_vec = _mm256_setzero_ps();

        // Process chunks of 8 elements
        for i in 0..chunks {
            let offset = i * 8;

            let a = _mm256_loadu_ps(self_ptr.add(offset));
            let b = _mm256_loadu_ps(other_ptr.add(offset));

            // Multiply and accumulate
            let product = _mm256_mul_ps(a, b);
            sum_vec = _mm256_add_ps(sum_vec, product);
        }

        // Horizontal sum of the accumulated vector
        let mut result_array = [0.0f32; 8];
        _mm256_storeu_ps(result_array.as_mut_ptr(), sum_vec);
        let mut dot_product: f32 = result_array.iter().sum();

        // Handle remaining elements
        for i in (chunks * 8)..len {
            dot_product += *self_ptr.add(i) * *other_ptr.add(i);
        }

        Ok(dot_product)
    }

    /// Fallback dot product for WASM and other targets
    #[cfg(any(not(target_arch = "x86_64"), target_family = "wasm"))]
    pub fn simd_dot(&self, other: &AlignedVector) -> Result<f32, &'static str> {
        if self.len() != other.len() {
            return Err("Vector lengths must match");
        }

        let dot_product: f32 = self.data.iter()
            .zip(&other.data)
            .map(|(a, b)| a * b)
            .sum();

        Ok(dot_product)
    }

    /// SIMD-accelerated vector scaling (x86_64 only)
    ///
    /// # Safety
    ///
    /// The CPU must support AVX2 (check with `is_x86_feature_detected!("avx2")`).
    /// No alignment is required: since 0.3.1 every load and store is unaligned
    /// (`loadu`/`storeu`). 0.3.0 used aligned loads on a `Vec<f32>` buffer that is
    /// only guaranteed 4-byte alignment, which faulted (SIGSEGV) on most allocations.
    #[cfg(all(target_arch = "x86_64", not(target_family = "wasm")))]
    #[target_feature(enable = "avx2")]
    pub unsafe fn simd_scale(&mut self, scalar: f32) {
        let len = self.len();
        let chunks = len / 8;

        let self_ptr = self.as_mut_ptr();
        let scalar_vec = _mm256_set1_ps(scalar); // Broadcast scalar to all elements

        // Process chunks of 8 elements
        for i in 0..chunks {
            let offset = i * 8;

            let a = _mm256_loadu_ps(self_ptr.add(offset));
            let result = _mm256_mul_ps(a, scalar_vec);
            _mm256_storeu_ps(self_ptr.add(offset), result);
        }

        // Handle remaining elements
        for i in (chunks * 8)..len {
            *self_ptr.add(i) *= scalar;
        }
    }

    /// Fallback vector scaling for WASM and other targets
    #[cfg(any(not(target_arch = "x86_64"), target_family = "wasm"))]
    pub fn simd_scale(&mut self, scalar: f32) {
        for value in &mut self.data {
            *value *= scalar;
        }
    }
}

/// Cache-optimized agent state structure
#[repr(align(64))]
pub struct AgentState {
    // Hot data (frequently accessed) - first cache line
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub acceleration: [f32; 3],
    pub energy: f32,
    pub active: bool,
    _padding1: [u8; 31], // Pad to cache line boundary

    // Warm data - second cache line
    pub parameters: AlignedVector,
    pub last_update_ns: u128,
    pub performance_score: f32,
    _padding2: [u8; 36],

    // Cold data - third cache line
    pub debug_info: String,
    pub creation_time: std::time::Instant,
}

impl AgentState {
    pub fn new(param_count: usize) -> Self {
        Self {
            position: [0.0; 3],
            velocity: [0.0; 3],
            acceleration: [0.0; 3],
            energy: 1.0,
            active: true,
            _padding1: [0; 31],
            parameters: AlignedVector::new(param_count),
            last_update_ns: 0,
            performance_score: 0.0,
            _padding2: [0; 36],
            debug_info: String::new(),
            creation_time: std::time::Instant::now(),
        }
    }

    /// SIMD-optimized state update (x86_64 only)
    #[cfg(all(target_arch = "x86_64", not(target_family = "wasm")))]
    pub fn simd_update(&mut self, dt: f32) {
        unsafe {
            // Update position using SIMD for 3D vector operations
            let pos_ptr = self.position.as_mut_ptr();
            let vel_ptr = self.velocity.as_ptr();

            // Load position and velocity vectors (pad to 4 elements for SIMD)
            let mut pos_padded = [0.0f32; 4];
            let mut vel_padded = [0.0f32; 4];

            pos_padded[..3].copy_from_slice(&self.position);
            vel_padded[..3].copy_from_slice(&self.velocity);

            let pos_vec = _mm_loadu_ps(pos_padded.as_ptr());
            let vel_vec = _mm_loadu_ps(vel_padded.as_ptr());
            let dt_vec = _mm_set1_ps(dt);

            // position += velocity * dt
            let vel_scaled = _mm_mul_ps(vel_vec, dt_vec);
            let new_pos = _mm_add_ps(pos_vec, vel_scaled);

            // Store result back (only first 3 elements)
            _mm_storeu_ps(pos_padded.as_mut_ptr(), new_pos);
            self.position.copy_from_slice(&pos_padded[..3]);
        }
    }

    /// Fallback state update for WASM and other targets
    #[cfg(any(not(target_arch = "x86_64"), target_family = "wasm"))]
    pub fn simd_update(&mut self, dt: f32) {
        // Simple scalar version
        for i in 0..3 {
            self.position[i] += self.velocity[i] * dt;
        }
    }
}

/// SIMD-optimized batch operations for multiple agents
pub struct BatchProcessor {
    positions: AlignedVector,
    velocities: AlignedVector,
    accelerations: AlignedVector,
    agent_count: usize,
}

impl BatchProcessor {
    pub fn new(max_agents: usize) -> Self {
        Self {
            positions: AlignedVector::new(max_agents * 3),
            velocities: AlignedVector::new(max_agents * 3),
            accelerations: AlignedVector::new(max_agents * 3),
            agent_count: 0,
        }
    }

    /// Batch update all agent positions using SIMD (x86_64 only)
    ///
    /// # Safety
    ///
    /// The CPU must support AVX2 (check with `is_x86_feature_detected!("avx2")`).
    /// No alignment is required: since 0.3.1 every load and store is unaligned
    /// (`loadu`/`storeu`). 0.3.0 used aligned loads on a `Vec<f32>` buffer that is
    /// only guaranteed 4-byte alignment, which faulted (SIGSEGV) on most allocations.
    #[cfg(all(target_arch = "x86_64", not(target_family = "wasm")))]
    #[target_feature(enable = "avx2")]
    pub unsafe fn batch_update_positions(&mut self, dt: f32) {
        // positions += velocities * dt + 0.5 * accelerations * dt^2

        let len = self.agent_count * 3;
        let chunks = len / 8;

        let pos_ptr = self.positions.as_mut_ptr();
        let vel_ptr = self.velocities.as_ptr();
        let acc_ptr = self.accelerations.as_ptr();

        let dt_vec = _mm256_set1_ps(dt);
        let dt2_vec = _mm256_set1_ps(dt * dt * 0.5);

        for i in 0..chunks {
            let offset = i * 8;

            let pos = _mm256_loadu_ps(pos_ptr.add(offset));
            let vel = _mm256_loadu_ps(vel_ptr.add(offset));
            let acc = _mm256_loadu_ps(acc_ptr.add(offset));

            // velocity * dt
            let vel_term = _mm256_mul_ps(vel, dt_vec);

            // 0.5 * acceleration * dt^2
            let acc_term = _mm256_mul_ps(acc, dt2_vec);

            // position + vel_term + acc_term
            let result = _mm256_add_ps(pos, _mm256_add_ps(vel_term, acc_term));

            _mm256_storeu_ps(pos_ptr.add(offset), result);
        }

        // Handle remaining elements
        for i in (chunks * 8)..len {
            *pos_ptr.add(i) += *vel_ptr.add(i) * dt + 0.5 * *acc_ptr.add(i) * dt * dt;
        }
    }

    /// Fallback batch update for WASM and other targets
    #[cfg(any(not(target_arch = "x86_64"), target_family = "wasm"))]
    pub fn batch_update_positions(&mut self, dt: f32) {
        let len = self.agent_count * 3;
        for i in 0..len {
            self.positions.data[i] += self.velocities.data[i] * dt + 0.5 * self.accelerations.data[i] * dt * dt;
        }
    }

    /// Calculate forces between agents using SIMD (x86_64 only)
    ///
    /// # Safety
    ///
    /// The CPU must support AVX2 (check with `is_x86_feature_detected!("avx2")`).
    /// No alignment is required: since 0.3.1 every load and store is unaligned
    /// (`loadu`/`storeu`). 0.3.0 used aligned loads on a `Vec<f32>` buffer that is
    /// only guaranteed 4-byte alignment, which faulted (SIGSEGV) on most allocations.
    #[cfg(all(target_arch = "x86_64", not(target_family = "wasm")))]
    #[target_feature(enable = "avx2")]
    pub unsafe fn calculate_forces(&mut self) -> AlignedVector {
        let mut forces = AlignedVector::new(self.agent_count * 3);

        // Simplified force calculation (normally would be N^2 complexity)
        // This is a placeholder for actual force computation

        forces
    }

    /// Fallback force calculation for WASM and other targets
    #[cfg(any(not(target_arch = "x86_64"), target_family = "wasm"))]
    pub fn calculate_forces(&mut self) -> AlignedVector {
        let forces = AlignedVector::new(self.agent_count * 3);

        // Simplified force calculation (normally would be N^2 complexity)
        // This is a placeholder for actual force computation

        forces
    }
}

/// Memory pool for zero-allocation agent operations
pub struct AgentMemoryPool {
    states: Vec<AgentState>,
    free_indices: Vec<usize>,
    capacity: usize,
}

impl AgentMemoryPool {
    pub fn new(capacity: usize) -> Self {
        let mut states = Vec::with_capacity(capacity);
        let mut free_indices = Vec::with_capacity(capacity);

        for i in 0..capacity {
            states.push(AgentState::new(16)); // 16 parameters per agent
            free_indices.push(i);
        }

        Self {
            states,
            free_indices,
            capacity,
        }
    }

    pub fn allocate_agent(&mut self) -> Option<usize> {
        self.free_indices.pop()
    }

    pub fn deallocate_agent(&mut self, index: usize) {
        if index < self.capacity {
            self.free_indices.push(index);
        }
    }

    pub fn get_state(&self, index: usize) -> Option<&AgentState> {
        self.states.get(index)
    }

    pub fn get_state_mut(&mut self, index: usize) -> Option<&mut AgentState> {
        self.states.get_mut(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(all(target_arch = "x86_64", not(target_family = "wasm")))]
    fn has_avx2() -> bool {
        is_x86_feature_detected!("avx2")
    }

    fn filled(n: usize, f: impl Fn(usize) -> f32) -> AlignedVector {
        let mut v = AlignedVector::new(n);
        for i in 0..v.len() {
            v.data[i] = f(i);
        }
        v
    }

    #[test]
    fn test_aligned_vector_creation() {
        let vec = AlignedVector::new(100);
        assert_eq!(vec.len(), 112); // Rounded up to multiple of 16
        // The struct is 64-byte aligned; the heap buffer is only f32-aligned
        // (0.3.0 asserted 64-byte heap alignment, which Vec never guarantees).
        assert_eq!(mem::align_of::<AlignedVector>(), 64);
        assert_eq!(vec.as_ptr() as usize % mem::align_of::<f32>(), 0);
    }

    #[test]
    fn test_simd_operations() {
        let mut a = filled(16, |i| i as f32);
        let b = filled(16, |i| (i * 2) as f32);

        #[cfg(all(target_arch = "x86_64", not(target_family = "wasm")))]
        {
            if !has_avx2() {
                return;
            }
            unsafe { a.simd_add(&b).unwrap() };
            let dot = unsafe { a.simd_dot(&b).unwrap() };
            // a = 3i, b = 2i: sum 6 i^2 for i < 16 = 6 * 1240
            assert_eq!(dot, 7440.0);
            unsafe { a.simd_scale(2.0) };
        }
        #[cfg(any(not(target_arch = "x86_64"), target_family = "wasm"))]
        {
            a.simd_add(&b).unwrap();
            assert_eq!(a.simd_dot(&b).unwrap(), 7440.0);
            a.simd_scale(2.0);
        }
        for i in 0..16 {
            assert_eq!(a.data[i], 6.0 * i as f32);
        }
    }

    /// Regression for the 0.3.0 SIGSEGV: aligned AVX loads on a buffer that is
    /// not 32-byte aligned. Keep allocating (and holding) vectors until some
    /// have a data pointer that is not 32-byte aligned, run every operation on
    /// each, and compare against a scalar reference. Miri with
    /// `-Zmiri-symbolic-alignment-check` is the allocator-independent guard.
    #[cfg(all(target_arch = "x86_64", not(target_family = "wasm")))]
    #[test]
    fn test_simd_misaligned_buffers() {
        if !has_avx2() {
            return;
        }
        let mut misaligned_seen = 0;
        for n in [8usize, 16, 24, 40, 100] {
            let mut held = Vec::new();
            for _ in 0..256 {
                let a = filled(n, |i| (i % 7) as f32 - 3.0);
                let b = filled(n, |i| (i % 5) as f32 + 0.5);
                let misaligned = a.as_ptr() as usize % 32 != 0 || b.as_ptr() as usize % 32 != 0;
                held.push((a, b));
                if misaligned {
                    break;
                }
            }
            for (a, b) in held.iter_mut() {
                if a.as_ptr() as usize % 32 != 0 || b.as_ptr() as usize % 32 != 0 {
                    misaligned_seen += 1;
                }
                let expect_dot: f32 = a.data.iter().zip(&b.data).map(|(x, y)| x * y).sum();
                let dot = unsafe { a.simd_dot(b).unwrap() };
                assert!((dot - expect_dot).abs() <= 1e-3, "dot n={n}");

                let expect_sum: Vec<f32> = a.data.iter().zip(&b.data).map(|(x, y)| (x + y) * 0.5).collect();
                unsafe {
                    a.simd_add(b).unwrap();
                    a.simd_scale(0.5);
                }
                assert_eq!(a.data, expect_sum, "add/scale n={n}");
            }
        }
        assert!(misaligned_seen > 0, "no misaligned buffer in 256 allocations per size");
    }

    #[test]
    fn test_agent_state_simd_update() {
        // Safe fn: 0.3.0 used aligned SSE loads on [f32; 4] stack arrays.
        let mut state = AgentState::new(16);
        state.position = [1.0, 2.0, 3.0];
        state.velocity = [0.5, -1.0, 2.0];
        state.simd_update(2.0);
        assert_eq!(state.position, [2.0, 0.0, 7.0]);
    }

    #[test]
    fn test_agent_state_alignment() {
        let state = AgentState::new(16);
        let ptr = &state as *const AgentState as usize;
        assert_eq!(ptr % 64, 0); // Cache-aligned
    }

    #[test]
    fn test_memory_pool() {
        let mut pool = AgentMemoryPool::new(10);

        let agent1 = pool.allocate_agent().unwrap();
        let agent2 = pool.allocate_agent().unwrap();

        assert_ne!(agent1, agent2);

        pool.deallocate_agent(agent1);
        let agent3 = pool.allocate_agent().unwrap();
        assert_eq!(agent1, agent3); // Reused index
    }
}
