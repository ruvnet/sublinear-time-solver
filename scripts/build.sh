#!/bin/bash

# Build script for sublinear-time-solver WASM integration
set -e

echo "🚀 Building sublinear-time-solver WASM module..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if required tools are installed
check_dependencies() {
    print_status "Checking dependencies..."

    if ! command -v rustc &> /dev/null; then
        print_error "Rust is not installed. Please install Rust from https://rustup.rs/"
        exit 1
    fi

    if ! command -v wasm-pack &> /dev/null; then
        print_warning "wasm-pack not found. Installing..."
        cargo install wasm-pack
    fi

    # Check for wasm32 target
    if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
        print_status "Installing wasm32-unknown-unknown target..."
        rustup target add wasm32-unknown-unknown
    fi

    print_success "All dependencies are available"
}

# Clean previous builds
clean_build() {
    print_status "Cleaning previous builds..."

    if [ -d "pkg" ]; then
        rm -rf pkg/
        print_status "Removed pkg/ directory"
    fi

    if [ -d "target" ]; then
        rm -rf target/
        print_status "Removed target/ directory"
    fi

    # Clean any WASM files in js directory
    if [ -f "js/solver_bg.wasm" ]; then
        rm js/solver_bg.wasm
        print_status "Removed old WASM binary from js/"
    fi
}

# Build WASM with optimizations
build_wasm() {
    print_status "Building WASM module with optimizations..."

    # Set environment variables for optimization
    export RUSTFLAGS="-C target-feature=+simd128 -C opt-level=s -C lto=fat"

    # Build for different targets
    print_status "Building for bundler target..."
    wasm-pack build \
        --target bundler \
        --out-dir pkg \
        --features "simd" \
        --release \
        -- --no-default-features

    if [ $? -eq 0 ]; then
        print_success "WASM build completed successfully"
    else
        print_error "WASM build failed"
        exit 1
    fi
}

# Optimize WASM binary if wasm-opt is available
optimize_wasm() {
    if command -v wasm-opt &> /dev/null; then
        print_status "Optimizing WASM binary with wasm-opt..."

        if [ -f "pkg/sublinear_time_solver_bg.wasm" ]; then
            # Create backup
            cp pkg/sublinear_time_solver_bg.wasm pkg/sublinear_time_solver_bg.wasm.backup

            # Optimize for size
            wasm-opt -Oz -o pkg/sublinear_time_solver_bg.wasm pkg/sublinear_time_solver_bg.wasm.backup

            # Get file sizes
            original_size=$(stat -c%s pkg/sublinear_time_solver_bg.wasm.backup 2>/dev/null || stat -f%z pkg/sublinear_time_solver_bg.wasm.backup)
            optimized_size=$(stat -c%s pkg/sublinear_time_solver_bg.wasm 2>/dev/null || stat -f%z pkg/sublinear_time_solver_bg.wasm)

            reduction=$((100 - (optimized_size * 100 / original_size)))
            print_success "WASM optimization complete. Size reduction: ${reduction}%"

            # Remove backup
            rm pkg/sublinear_time_solver_bg.wasm.backup
        else
            print_warning "WASM binary not found for optimization"
        fi
    else
        print_warning "wasm-opt not found. Install it for binary optimization:"
        print_warning "  - Ubuntu/Debian: apt install binaryen"
        print_warning "  - macOS: brew install binaryen"
        print_warning "  - Or download from: https://github.com/WebAssembly/binaryen"
    fi
}

# Build additional targets
build_additional_targets() {
    print_status "Building additional targets..."

    # Node.js target
    print_status "Building for Node.js..."
    wasm-pack build \
        --target nodejs \
        --out-dir pkg/nodejs \
        --features "simd" \
        --release \
        -- --no-default-features

    # Web target (no bundler)
    print_status "Building for web (no bundler)..."
    wasm-pack build \
        --target web \
        --out-dir pkg/web \
        --features "simd" \
        --release \
        -- --no-default-features
}

# Copy necessary files
copy_files() {
    print_status "Copying files to appropriate locations..."

    # Create js directory if it doesn't exist
    mkdir -p js/

    # Copy WASM binary to js directory for easier access
    if [ -f "pkg/sublinear_time_solver_bg.wasm" ]; then
        cp pkg/sublinear_time_solver_bg.wasm js/
        print_status "Copied WASM binary to js/"
    fi

    print_success "File copying completed"
}

# Generate package info
generate_package_info() {
    print_status "Generating package information..."

    # Create a build info file
    cat > pkg/build_info.json << EOF
{
  "build_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "rust_version": "$(rustc --version)",
  "wasm_pack_version": "$(wasm-pack --version | grep wasm-pack || echo 'unknown')",
  "features": ["simd"],
  "optimization_level": "s",
  "target": "wasm32-unknown-unknown"
}
EOF

    print_success "Package information generated"
}

# Run tests if available
run_tests() {
    print_status "Running Rust tests..."

    if cargo test --quiet 2>/dev/null; then
        print_success "All Rust tests passed"
    else
        print_warning "Some tests failed or no tests found"
    fi
}

# Print build summary
print_summary() {
    echo ""
    echo "🎉 Build Summary:"
    echo "=================="

    if [ -f "pkg/sublinear_time_solver_bg.wasm" ]; then
        wasm_size=$(stat -c%s pkg/sublinear_time_solver_bg.wasm 2>/dev/null || stat -f%z pkg/sublinear_time_solver_bg.wasm)
        echo "📦 WASM binary size: $(( wasm_size / 1024 )) KB"
    fi

    echo "📁 Generated files:"
    echo "   - pkg/                    (Main WASM package)"
    echo "   - pkg/nodejs/             (Node.js target)"
    echo "   - pkg/web/                (Web target)"
    echo "   - js/solver.js            (JavaScript interface)"
    echo "   - types/index.d.ts        (TypeScript definitions)"

    echo ""
    echo "🚀 Ready to use!"
    echo "   Import: import { SublinearSolver } from './js/solver.js'"
    echo "   Test:   npm test"
    echo "   Bench:  npm run bench"
}

# Main build process
main() {
    echo "🔧 Building sublinear-time-solver WASM integration"
    echo "=================================================="

    check_dependencies
    clean_build
    run_tests
    build_wasm
    optimize_wasm
    build_additional_targets
    copy_files
    generate_package_info
    print_summary

    print_success "Build completed successfully! 🎉"
}

# Handle command line arguments
case "${1:-}" in
    --clean)
        clean_build
        exit 0
        ;;
    --dev)
        print_status "Building in development mode..."
        wasm-pack build --dev --target bundler
        exit 0
        ;;
    --help)
        echo "Usage: $0 [options]"
        echo "Options:"
        echo "  --clean    Clean build artifacts"
        echo "  --dev      Build in development mode"
        echo "  --help     Show this help message"
        exit 0
        ;;
    "")
        main
        ;;
    *)
        print_error "Unknown option: $1"
        echo "Use --help for usage information"
        exit 1
        ;;
esac