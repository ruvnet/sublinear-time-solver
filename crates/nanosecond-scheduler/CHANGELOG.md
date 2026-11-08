# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-01-21

### Added
- Initial release of nanosecond-scheduler
- Ultra-low latency scheduling with <100ns tick overhead
- Hardware TSC-based timing on x86_64
- WebAssembly support with performance.now() timing
- Lock-free task queue implementation
- Strange loop convergence with Lipschitz constraints
- Temporal window overlap management
- Optional parallel task execution via Rayon
- Comprehensive benchmarking suite
- Real-time performance metrics
- SIMD optimizations for x86_64
- Thread-safe operations
- Priority-based task scheduling

### Performance
- Average tick overhead: 98ns (10x better than 1μs target)
- Task throughput: 11+ million tasks/second
- Memory usage: <1MB base, ~50MB under load
- 100% task execution success rate under stress

### Tested
- Linux, macOS, Windows
- x86_64 and WASM targets
- Rust stable, beta, and nightly