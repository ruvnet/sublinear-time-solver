//! Build script for strange-loop crate with SIMD optimizations

fn main() {
    // Enable SIMD optimizations
    println!("cargo:rerun-if-changed=build.rs");

    // Detect CPU features and enable optimizations
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            println!("cargo:rustc-cfg=avx2_available");
        }
        if is_x86_feature_detected!("fma") {
            println!("cargo:rustc-cfg=fma_available");
        }
        if is_x86_feature_detected!("sse4.2") {
            println!("cargo:rustc-cfg=sse42_available");
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        println!("cargo:rustc-cfg=neon_available");
    }

    // WASM-specific optimizations
    #[cfg(target_arch = "wasm32")]
    {
        println!("cargo:rustc-cfg=wasm_simd");
        println!("cargo:rustc-link-arg=--import-memory");
        println!("cargo:rustc-link-arg=--shared-memory");
        println!("cargo:rustc-link-arg=--max-memory=4294967296"); // 4GB max
    }
}