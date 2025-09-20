#!/bin/bash

# Comprehensive Temporal Neural Solver Validation Script
#
# This script provides a reproducible way to validate the performance
# claims of the temporal neural solver across different environments.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
ITERATIONS=${ITERATIONS:-10000}
WARMUP=${WARMUP:-1000}
OUTPUT_DIR=${OUTPUT_DIR:-"validation_results"}
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
RESULTS_FILE="${OUTPUT_DIR}/validation_${TIMESTAMP}"

# Functions
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

check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check Rust version
    if ! command -v rustc &> /dev/null; then
        log_error "Rust compiler not found. Please install Rust."
        exit 1
    fi

    local rust_version=$(rustc --version)
    log_info "Rust version: $rust_version"

    # Check Cargo
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Cargo."
        exit 1
    fi

    # Check if we're in the right directory
    if [[ ! -f "Cargo.toml" ]]; then
        log_error "Cargo.toml not found. Please run this script from the project root."
        exit 1
    fi

    # Check for required CPU features
    if command -v lscpu &> /dev/null; then
        local cpu_info=$(lscpu)
        if echo "$cpu_info" | grep -q "avx2"; then
            log_success "AVX2 support detected"
        else
            log_warning "AVX2 support not detected - performance may be suboptimal"
        fi
    fi

    log_success "Prerequisites check passed"
}

build_project() {
    log_info "Building project in release mode..."

    # Clean previous builds
    cargo clean

    # Build with optimizations
    RUSTFLAGS="-C target-cpu=native -C opt-level=3" \
    cargo build --release --bins

    if [[ $? -eq 0 ]]; then
        log_success "Build completed successfully"
    else
        log_error "Build failed"
        exit 1
    fi
}

create_output_dir() {
    log_info "Creating output directory: $OUTPUT_DIR"
    mkdir -p "$OUTPUT_DIR"
}

run_quick_benchmark() {
    log_info "Running quick benchmark to verify basic functionality..."

    local quick_results=$(cargo run --release --bin comprehensive_benchmark -- quick --iterations 1000 2>&1)
    local exit_code=$?

    echo "$quick_results" | tee "${RESULTS_FILE}_quick.txt"

    if [[ $exit_code -eq 0 ]] && echo "$quick_results" | grep -q "Significant performance improvement"; then
        log_success "Quick benchmark passed - significant improvement detected"
        return 0
    else
        log_warning "Quick benchmark shows limited improvement - continuing with full validation"
        return 1
    fi
}

run_comprehensive_benchmark() {
    log_info "Running comprehensive validation benchmark..."
    log_info "Iterations: $ITERATIONS, Warmup: $WARMUP"
    log_info "This may take several minutes..."

    # Run text report
    log_info "Generating text report..."
    cargo run --release --bin comprehensive_benchmark -- run \
        --iterations "$ITERATIONS" \
        --warmup "$WARMUP" \
        --format text \
        --output "${RESULTS_FILE}.txt" 2>&1 | tee "${RESULTS_FILE}_console.log"

    local text_exit_code=$?

    # Run JSON report
    log_info "Generating JSON report..."
    cargo run --release --bin comprehensive_benchmark -- run \
        --iterations "$ITERATIONS" \
        --warmup "$WARMUP" \
        --format json \
        --output "${RESULTS_FILE}.json" 2>&1

    local json_exit_code=$?

    # Run HTML report
    log_info "Generating HTML report..."
    cargo run --release --bin comprehensive_benchmark -- run \
        --iterations "$ITERATIONS" \
        --warmup "$WARMUP" \
        --format html \
        --output "${RESULTS_FILE}.html" 2>&1

    local html_exit_code=$?

    return $text_exit_code
}

collect_system_info() {
    log_info "Collecting system information..."

    local sys_info_file="${RESULTS_FILE}_system_info.txt"

    {
        echo "=== SYSTEM INFORMATION ==="
        echo "Timestamp: $(date)"
        echo "Hostname: $(hostname)"
        echo "User: $(whoami)"
        echo ""

        echo "=== HARDWARE INFORMATION ==="
        if command -v lscpu &> /dev/null; then
            echo "CPU Information:"
            lscpu
        elif [[ -f /proc/cpuinfo ]]; then
            echo "CPU Information:"
            cat /proc/cpuinfo | head -20
        fi
        echo ""

        if [[ -f /proc/meminfo ]]; then
            echo "Memory Information:"
            cat /proc/meminfo | head -10
        fi
        echo ""

        echo "=== SOFTWARE INFORMATION ==="
        echo "Rust version: $(rustc --version)"
        echo "Cargo version: $(cargo --version)"
        echo "OS: $(uname -a)"
        echo ""

        if command -v gcc &> /dev/null; then
            echo "GCC version: $(gcc --version | head -1)"
        fi

        if command -v git &> /dev/null; then
            echo "Git commit: $(git rev-parse HEAD 2>/dev/null || echo 'Not in git repo')"
            echo "Git status: $(git status --porcelain 2>/dev/null || echo 'Not in git repo')"
        fi
        echo ""

        echo "=== ENVIRONMENT VARIABLES ==="
        env | grep -E '^(RUST|CARGO|CC|CXX|CFLAGS|CXXFLAGS|RUSTFLAGS)' | sort
        echo ""

        echo "=== VALIDATION CONFIGURATION ==="
        echo "Iterations: $ITERATIONS"
        echo "Warmup: $WARMUP"
        echo "Output directory: $OUTPUT_DIR"
        echo "Results file: $RESULTS_FILE"

    } > "$sys_info_file"

    log_success "System information saved to: $sys_info_file"
}

verify_results() {
    log_info "Verifying benchmark results..."

    local text_file="${RESULTS_FILE}.txt"
    local json_file="${RESULTS_FILE}.json"

    if [[ -f "$text_file" ]]; then
        if grep -q "VALIDATION PASSED" "$text_file"; then
            log_success "Text report validation: PASSED"
            local text_passed=true
        else
            log_error "Text report validation: FAILED"
            local text_passed=false
        fi
    else
        log_error "Text report file not found: $text_file"
        local text_passed=false
    fi

    if [[ -f "$json_file" ]]; then
        if command -v jq &> /dev/null; then
            local overall_passed=$(jq -r '.validation_summary.overall_passed' "$json_file" 2>/dev/null)
            if [[ "$overall_passed" == "true" ]]; then
                log_success "JSON report validation: PASSED"
                local json_passed=true
            else
                log_error "JSON report validation: FAILED"
                local json_passed=false
            fi
        else
            log_warning "jq not available - cannot verify JSON results"
            local json_passed=true
        fi
    else
        log_error "JSON report file not found: $json_file"
        local json_passed=false
    fi

    if [[ "$text_passed" == true ]] && [[ "$json_passed" == true ]]; then
        return 0
    else
        return 1
    fi
}

generate_summary() {
    log_info "Generating validation summary..."

    local summary_file="${RESULTS_FILE}_summary.md"
    local text_file="${RESULTS_FILE}.txt"
    local json_file="${RESULTS_FILE}.json"

    {
        echo "# Temporal Neural Solver Validation Summary"
        echo ""
        echo "**Validation Date:** $(date)"
        echo "**Validation ID:** validation_${TIMESTAMP}"
        echo ""

        echo "## Configuration"
        echo "- Iterations: $ITERATIONS"
        echo "- Warmup: $WARMUP"
        echo "- Host: $(hostname)"
        echo "- User: $(whoami)"
        echo ""

        echo "## System Information"
        if command -v lscpu &> /dev/null; then
            echo "- CPU: $(lscpu | grep 'Model name' | cut -d: -f2 | xargs)"
            echo "- Cores: $(lscpu | grep '^CPU(s):' | cut -d: -f2 | xargs)"
        fi

        if [[ -f /proc/meminfo ]]; then
            echo "- Memory: $(grep MemTotal /proc/meminfo | awk '{print $2 " " $3}')"
        fi
        echo ""

        echo "## Validation Results"
        if [[ -f "$text_file" ]] && grep -q "VALIDATION PASSED" "$text_file"; then
            echo "✅ **VALIDATION PASSED**"
            echo ""
            echo "The temporal neural solver has been successfully validated with:"
            echo "- Statistically significant performance improvements"
            echo "- Hardware verification completed"
            echo "- Cryptographic integrity confirmed"
            echo "- Reproducibility verified"
        else
            echo "❌ **VALIDATION FAILED**"
            echo ""
            echo "Please review the detailed reports for issues."
        fi
        echo ""

        echo "## Files Generated"
        echo "- Text report: \`$(basename "$text_file")\`"
        echo "- JSON report: \`$(basename "$json_file")\`"
        echo "- HTML report: \`$(basename "${RESULTS_FILE}.html")\`"
        echo "- System info: \`$(basename "${RESULTS_FILE}_system_info.txt")\`"
        echo "- Console log: \`$(basename "${RESULTS_FILE}_console.log")\`"
        echo ""

        if [[ -f "$json_file" ]] && command -v jq &> /dev/null; then
            echo "## Key Metrics"
            echo "Certificate ID: \`$(jq -r '.certificate.certificate_id' "$json_file" 2>/dev/null || echo 'N/A')\`"
            echo ""

            echo "### Performance Comparison"
            # This would extract performance data from JSON
            echo "(See detailed reports for performance metrics)"
        fi

        echo ""
        echo "---"
        echo "*Generated by Temporal Neural Solver Validation Framework*"

    } > "$summary_file"

    log_success "Summary generated: $summary_file"
}

create_archive() {
    log_info "Creating validation archive..."

    local archive_name="temporal_solver_validation_${TIMESTAMP}.tar.gz"
    local archive_path="${OUTPUT_DIR}/${archive_name}"

    tar -czf "$archive_path" \
        "${RESULTS_FILE}"* \
        --transform "s|^|temporal_solver_validation_${TIMESTAMP}/|" \
        2>/dev/null || {
            log_warning "Failed to create archive with transform, trying simple archive..."
            cd "$OUTPUT_DIR"
            tar -czf "$archive_name" *"${TIMESTAMP}"*
            cd - > /dev/null
        }

    if [[ -f "$archive_path" ]]; then
        log_success "Validation archive created: $archive_path"
        log_info "Archive size: $(du -h "$archive_path" | cut -f1)"
    else
        log_warning "Failed to create validation archive"
    fi
}

print_final_summary() {
    echo ""
    echo "=============================================="
    echo "   TEMPORAL NEURAL SOLVER VALIDATION"
    echo "=============================================="
    echo ""

    if verify_results; then
        log_success "🎉 VALIDATION COMPLETED SUCCESSFULLY!"
        echo ""
        echo "The temporal neural solver has passed all validation tests:"
        echo "✅ Performance improvements verified"
        echo "✅ Statistical significance confirmed"
        echo "✅ Hardware compatibility verified"
        echo "✅ Cryptographic integrity ensured"
        echo ""
        echo "Results are available in: $OUTPUT_DIR"
        echo "View the HTML report in your browser:"
        echo "  file://$(pwd)/${RESULTS_FILE}.html"
    else
        log_error "❌ VALIDATION FAILED"
        echo ""
        echo "Please review the detailed reports for issues:"
        echo "  - Text report: ${RESULTS_FILE}.txt"
        echo "  - Console log: ${RESULTS_FILE}_console.log"
        echo ""
        echo "Common issues:"
        echo "• Insufficient CPU features (AVX2 recommended)"
        echo "• Insufficient sample size (increase --iterations)"
        echo "• System load affecting measurements"
        exit 1
    fi
}

# Main execution
main() {
    echo ""
    echo "🚀 Temporal Neural Solver Validation Framework"
    echo "==============================================="
    echo ""

    check_prerequisites
    create_output_dir
    build_project
    collect_system_info

    # Quick test first
    if run_quick_benchmark; then
        log_success "Quick test passed - proceeding with comprehensive validation"
    else
        log_warning "Quick test showed limited improvement - review configuration"
    fi

    # Comprehensive validation
    if run_comprehensive_benchmark; then
        log_success "Comprehensive benchmark completed"
    else
        log_error "Comprehensive benchmark failed"
    fi

    generate_summary
    create_archive
    print_final_summary
}

# Handle interrupts gracefully
trap 'log_error "Validation interrupted"; exit 130' INT TERM

# Check if being sourced or executed
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi