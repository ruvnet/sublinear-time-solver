#!/bin/bash

# Multi-system validation script for Temporal Neural Solver
#
# This script demonstrates how to run validation across different
# hardware configurations and operating systems.

set -euo pipefail

# Configuration
DOCKER_IMAGE="rust:1.70"
OUTPUT_BASE="multi_system_validation"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Native system validation
run_native_validation() {
    log_info "Running validation on native system..."

    local output_dir="${OUTPUT_BASE}/native_${TIMESTAMP}"
    mkdir -p "$output_dir"

    # Collect system info
    {
        echo "=== NATIVE SYSTEM INFORMATION ==="
        echo "OS: $(uname -a)"
        echo "Architecture: $(uname -m)"
        if command -v lscpu &> /dev/null; then
            echo "CPU: $(lscpu | grep 'Model name' | cut -d: -f2 | xargs)"
        fi
        echo "Rust: $(rustc --version)"
        echo "Timestamp: $(date)"
    } > "${output_dir}/system_info.txt"

    # Run validation
    ITERATIONS=5000 WARMUP=500 OUTPUT_DIR="$output_dir" ./scripts/run_validation.sh

    log_success "Native validation completed: $output_dir"
}

# Docker-based validation (simulates different environments)
run_docker_validation() {
    local variant="$1"
    local dockerfile="$2"

    log_info "Running validation in Docker: $variant"

    if ! command -v docker &> /dev/null; then
        log_warning "Docker not available - skipping Docker validation"
        return
    fi

    local output_dir="${OUTPUT_BASE}/docker_${variant}_${TIMESTAMP}"
    mkdir -p "$output_dir"

    # Create Dockerfile
    cat > "${output_dir}/Dockerfile" << EOF
$dockerfile

# Copy project
WORKDIR /workspace
COPY . .

# Install dependencies
RUN cargo --version && rustc --version

# Set up validation
RUN chmod +x scripts/run_validation.sh

# Run validation with reduced iterations for Docker
ENV ITERATIONS=1000
ENV WARMUP=100
ENV OUTPUT_DIR=/workspace/validation_results

CMD ["./scripts/run_validation.sh"]
EOF

    # Build and run
    local image_name="temporal-solver-validation:$variant"

    if docker build -t "$image_name" -f "${output_dir}/Dockerfile" . > "${output_dir}/docker_build.log" 2>&1; then
        docker run --rm -v "${PWD}/${output_dir}:/output" "$image_name" \
            bash -c "cd /workspace && ./scripts/run_validation.sh && cp -r validation_results/* /output/" \
            > "${output_dir}/docker_run.log" 2>&1

        if [[ $? -eq 0 ]]; then
            log_success "Docker validation ($variant) completed: $output_dir"
        else
            log_error "Docker validation ($variant) failed - check logs in $output_dir"
        fi
    else
        log_error "Docker build failed for $variant - check ${output_dir}/docker_build.log"
    fi
}

# GitHub Actions configuration
generate_github_actions() {
    log_info "Generating GitHub Actions workflow..."

    mkdir -p .github/workflows

    cat > .github/workflows/validation.yml << 'EOF'
name: Temporal Neural Solver Validation

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]
  schedule:
    # Run daily at 2 AM UTC
    - cron: '0 2 * * *'

env:
  CARGO_TERM_COLOR: always

jobs:
  validation:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, nightly]
        include:
          - os: ubuntu-latest
            iterations: 10000
            warmup: 1000
          - os: windows-latest
            iterations: 5000
            warmup: 500
          - os: macos-latest
            iterations: 5000
            warmup: 500

    runs-on: ${{ matrix.os }}

    steps:
    - uses: actions/checkout@v3

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: ${{ matrix.rust }}
        profile: minimal
        override: true

    - name: Cache cargo registry
      uses: actions/cache@v3
      with:
        path: ~/.cargo/registry
        key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

    - name: Cache cargo index
      uses: actions/cache@v3
      with:
        path: ~/.cargo/git
        key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}

    - name: Cache cargo build
      uses: actions/cache@v3
      with:
        path: target
        key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}

    - name: Install system dependencies (Ubuntu)
      if: matrix.os == 'ubuntu-latest'
      run: |
        sudo apt-get update
        sudo apt-get install -y build-essential

    - name: Build project
      run: |
        cd tns-engine/temporal-neural-solver
        cargo build --release --bins

    - name: Run quick validation
      run: |
        cd tns-engine/temporal-neural-solver
        cargo run --release --bin comprehensive_benchmark -- quick --iterations 1000

    - name: Run comprehensive validation
      env:
        ITERATIONS: ${{ matrix.iterations }}
        WARMUP: ${{ matrix.warmup }}
      run: |
        cd tns-engine/temporal-neural-solver
        chmod +x scripts/run_validation.sh
        ./scripts/run_validation.sh

    - name: Upload validation results
      uses: actions/upload-artifact@v3
      if: always()
      with:
        name: validation-results-${{ matrix.os }}-${{ matrix.rust }}
        path: tns-engine/temporal-neural-solver/validation_results/
        retention-days: 30

    - name: Check validation status
      run: |
        cd tns-engine/temporal-neural-solver/validation_results
        if ls validation_*.txt 1> /dev/null 2>&1; then
          if grep -q "VALIDATION PASSED" validation_*.txt; then
            echo "✅ Validation passed on ${{ matrix.os }} with ${{ matrix.rust }}"
            exit 0
          else
            echo "❌ Validation failed on ${{ matrix.os }} with ${{ matrix.rust }}"
            exit 1
          fi
        else
          echo "❌ No validation results found"
          exit 1
        fi

  aggregate-results:
    needs: validation
    runs-on: ubuntu-latest
    if: always()

    steps:
    - uses: actions/checkout@v3

    - name: Download all artifacts
      uses: actions/download-artifact@v3

    - name: Aggregate results
      run: |
        echo "# Validation Results Summary" > validation_summary.md
        echo "" >> validation_summary.md
        echo "**Date:** $(date)" >> validation_summary.md
        echo "**Commit:** ${{ github.sha }}" >> validation_summary.md
        echo "" >> validation_summary.md

        for dir in validation-results-*; do
          if [[ -d "$dir" ]]; then
            os_rust=$(echo "$dir" | sed 's/validation-results-//')
            echo "## $os_rust" >> validation_summary.md

            if find "$dir" -name "validation_*.txt" -exec grep -l "VALIDATION PASSED" {} \; | head -1 > /dev/null; then
              echo "✅ **PASSED**" >> validation_summary.md
            else
              echo "❌ **FAILED**" >> validation_summary.md
            fi
            echo "" >> validation_summary.md
          fi
        done

    - name: Upload summary
      uses: actions/upload-artifact@v3
      with:
        name: validation-summary
        path: validation_summary.md
        retention-days: 90
EOF

    log_success "GitHub Actions workflow generated: .github/workflows/validation.yml"
}

# Docker configurations for different environments
get_dockerfile_ubuntu() {
    cat << 'EOF'
FROM ubuntu:22.04

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Verify installation
RUN rustc --version && cargo --version
EOF
}

get_dockerfile_alpine() {
    cat << 'EOF'
FROM alpine:3.18

# Install system dependencies
RUN apk add --no-cache \
    curl \
    build-base \
    musl-dev

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Verify installation
RUN rustc --version && cargo --version
EOF
}

get_dockerfile_centos() {
    cat << 'EOF'
FROM centos:7

# Install system dependencies
RUN yum update -y && yum install -y \
    curl \
    gcc \
    gcc-c++ \
    make \
    && yum clean all

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Verify installation
RUN rustc --version && cargo --version
EOF
}

# Continuous integration validation
run_ci_validation() {
    log_info "Setting up continuous integration validation..."

    generate_github_actions

    # Create validation badge
    cat > validation_badge.md << 'EOF'
# Temporal Neural Solver Validation Status

[![Validation Status](https://github.com/YOUR_USERNAME/YOUR_REPO/workflows/Temporal%20Neural%20Solver%20Validation/badge.svg)](https://github.com/YOUR_USERNAME/YOUR_REPO/actions)

This badge shows the current validation status across multiple platforms and configurations.

## Validation Matrix

The validation runs on:
- **Operating Systems:** Ubuntu, Windows, macOS
- **Rust Versions:** Stable, Nightly
- **Configurations:** Various iteration counts based on platform

## Viewing Results

Click the badge above to view detailed validation results, including:
- Performance benchmarks
- Statistical analysis
- Hardware verification
- Cryptographic integrity proofs

All results are archived for 30 days and summaries are kept for 90 days.
EOF

    log_success "CI validation setup completed"
}

# Cross-compilation validation
run_cross_compilation_validation() {
    log_info "Testing cross-compilation targets..."

    local targets=(
        "x86_64-unknown-linux-gnu"
        "x86_64-unknown-linux-musl"
        "x86_64-pc-windows-gnu"
        "aarch64-unknown-linux-gnu"
    )

    local output_dir="${OUTPUT_BASE}/cross_compilation_${TIMESTAMP}"
    mkdir -p "$output_dir"

    cd tns-engine/temporal-neural-solver

    for target in "${targets[@]}"; do
        log_info "Testing cross-compilation for $target..."

        # Add target if not already installed
        rustup target add "$target" 2>/dev/null || true

        # Attempt to build
        if cargo build --release --target "$target" --bins > "${output_dir}/${target}_build.log" 2>&1; then
            log_success "Cross-compilation successful: $target"
            echo "✅ $target" >> "${output_dir}/cross_compilation_results.txt"
        else
            log_warning "Cross-compilation failed: $target (see ${output_dir}/${target}_build.log)"
            echo "❌ $target" >> "${output_dir}/cross_compilation_results.txt"
        fi
    done

    cd - > /dev/null
    log_success "Cross-compilation test completed: $output_dir"
}

# Main execution
main() {
    echo ""
    echo "🌐 Multi-System Temporal Neural Solver Validation"
    echo "================================================="
    echo ""

    mkdir -p "$OUTPUT_BASE"

    # Check if we're in the right directory
    if [[ ! -f "tns-engine/temporal-neural-solver/Cargo.toml" ]]; then
        log_error "Please run this script from the project root directory"
        exit 1
    fi

    cd tns-engine/temporal-neural-solver

    # Make validation script executable
    chmod +x scripts/run_validation.sh

    cd - > /dev/null

    case "${1:-all}" in
        "native")
            run_native_validation
            ;;
        "docker")
            run_docker_validation "ubuntu" "$(get_dockerfile_ubuntu)"
            run_docker_validation "alpine" "$(get_dockerfile_alpine)"
            run_docker_validation "centos" "$(get_dockerfile_centos)"
            ;;
        "ci")
            run_ci_validation
            ;;
        "cross")
            run_cross_compilation_validation
            ;;
        "all")
            log_info "Running complete multi-system validation..."
            run_native_validation
            if command -v docker &> /dev/null; then
                run_docker_validation "ubuntu" "$(get_dockerfile_ubuntu)"
            fi
            run_ci_validation
            run_cross_compilation_validation
            ;;
        *)
            echo "Usage: $0 [native|docker|ci|cross|all]"
            echo ""
            echo "  native  - Run validation on current system"
            echo "  docker  - Run validation in Docker containers"
            echo "  ci      - Set up continuous integration"
            echo "  cross   - Test cross-compilation"
            echo "  all     - Run all validations (default)"
            exit 1
            ;;
    esac

    echo ""
    echo "=============================================="
    echo "   MULTI-SYSTEM VALIDATION COMPLETED"
    echo "=============================================="
    echo ""
    echo "Results are available in: $OUTPUT_BASE"
    echo ""
    echo "Next steps:"
    echo "1. Review validation results in each subdirectory"
    echo "2. Commit the GitHub Actions workflow if generated"
    echo "3. Push to trigger CI validation across platforms"
    echo "4. Monitor validation status via GitHub Actions"
}

# Handle interrupts gracefully
trap 'log_error "Multi-system validation interrupted"; exit 130' INT TERM

# Execute main function
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi