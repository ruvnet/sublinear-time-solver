# CLI Guide

## Installation

```bash
# Global installation
npm install -g sublinear-time-solver

# Or use with npx (no installation needed)
npx sublinear-time-solver --help
```

## Command Overview

```
sublinear-time-solver [options] [command]

Commands:
  solve      Solve linear system Ax = b
  serve      Start HTTP streaming server
  verify     Verify solution accuracy
  benchmark  Run performance benchmarks
  convert    Convert between matrix formats
  flow-nexus Flow-Nexus platform integration
```

## Solve Command

Solve linear systems with various methods.

### Basic Usage

```bash
# Solve a random system
sublinear-time-solver solve --size 100 --method jacobi

# Solve from files
sublinear-time-solver solve --matrix A.json --vector b.json

# With specific options
sublinear-time-solver solve \
  --matrix data/matrix.mtx \
  --vector data/rhs.csv \
  --method conjugate_gradient \
  --tolerance 1e-8 \
  --max-iterations 1000 \
  --output solution.json
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `-m, --matrix <file>` | Matrix file (JSON, CSV, MTX) | Random |
| `-b, --vector <file>` | RHS vector file | Random |
| `-s, --size <n>` | Random matrix size | 100 |
| `--method <name>` | Solving method | `jacobi` |
| `--tolerance <tol>` | Convergence tolerance | `1e-6` |
| `--max-iterations <n>` | Maximum iterations | `1000` |
| `--stream` | Enable streaming output | `false` |
| `-o, --output <file>` | Output file | stdout |
| `--format <type>` | Output format (json/csv/text) | `json` |

### Methods Available

- `jacobi` - Jacobi iteration
- `gauss_seidel` - Gauss-Seidel iteration
- `conjugate_gradient` - Conjugate Gradient (symmetric positive definite)
- `neumann` - Neumann series expansion
- `forward_push` - Forward push algorithm
- `backward_push` - Backward push algorithm
- `hybrid` - Hybrid random-walk method
- `auto` - Automatic method selection

### Examples

```bash
# Large sparse system with progress
sublinear-time-solver solve \
  --size 10000 \
  --sparsity 0.01 \
  --method hybrid \
  --stream \
  --verbose

# Solve and save results
sublinear-time-solver solve \
  --matrix system.json \
  --vector rhs.json \
  --output solution.json \
  --format json

# Quick test with small system
sublinear-time-solver solve --size 10 --verbose
```

## Serve Command

Start HTTP server for solving via API.

### Usage

```bash
# Start server on default port 3000
sublinear-time-solver serve

# Custom port and options
sublinear-time-solver serve \
  --port 8080 \
  --host 0.0.0.0 \
  --cors \
  --max-connections 100
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `-p, --port <n>` | Server port | `3000` |
| `-h, --host <addr>` | Host address | `localhost` |
| `--cors` | Enable CORS | `false` |
| `--auth <token>` | Require auth token | none |
| `--max-connections <n>` | Max concurrent connections | `100` |
| `--workers <n>` | Worker threads | CPU count |

### API Endpoints

Once running, the server provides:

- `POST /solve` - Single solve request
- `POST /solve-stream` - Streaming solve (NDJSON)
- `GET /status` - Server status
- `WebSocket /ws` - Real-time bidirectional solving

## Verify Command

Verify solution accuracy.

```bash
# Verify a computed solution
sublinear-time-solver verify \
  --matrix A.json \
  --solution x.json \
  --vector b.json

# With tolerance checking
sublinear-time-solver verify \
  --matrix system.mtx \
  --solution sol.csv \
  --vector rhs.csv \
  --tolerance 1e-6
```

### Output

```
Verification Results:
  Max Error: 2.34e-7
  Avg Error: 5.67e-8
  Residual Norm: 1.23e-7
  Status: VALID ✓
```

## Benchmark Command

Run performance benchmarks.

```bash
# Basic benchmark
sublinear-time-solver benchmark --size 1000

# Comprehensive benchmark
sublinear-time-solver benchmark \
  --sizes 100,500,1000,5000,10000 \
  --methods jacobi,conjugate_gradient,hybrid \
  --iterations 10 \
  --output results.csv
```

### Output Format

```
Size    Method              Time(ms)  Iterations  Residual
────────────────────────────────────────────────────────
1000    jacobi              23.4      156         9.8e-7
1000    conjugate_gradient  15.2      42          3.2e-8
1000    hybrid              18.7      89          5.4e-7
```

## Convert Command

Convert between matrix formats.

```bash
# Convert Matrix Market to JSON
sublinear-time-solver convert \
  --input matrix.mtx \
  --output matrix.json \
  --format json

# Convert CSV to binary
sublinear-time-solver convert \
  --input data.csv \
  --output data.bin \
  --format binary
```

### Supported Formats

- JSON (`.json`)
- CSV (`.csv`)
- Matrix Market (`.mtx`)
- Binary (`.bin`)
- NumPy (`.npy`)

## Flow-Nexus Command

Integration with Flow-Nexus platform.

```bash
# Register as Flow-Nexus service
sublinear-time-solver flow-nexus --register

# Start as Flow-Nexus agent
sublinear-time-solver flow-nexus \
  --serve \
  --swarm-id abc123 \
  --capabilities solve,verify

# Join existing swarm
sublinear-time-solver flow-nexus \
  --join \
  --swarm-endpoint http://swarm.flow-nexus.io
```

## Global Options

Available for all commands:

| Option | Description |
|--------|-------------|
| `-V, --version` | Show version |
| `-v, --verbose` | Verbose output |
| `-q, --quiet` | Suppress output |
| `--debug` | Debug mode |
| `--config <file>` | Config file |
| `-h, --help` | Show help |

## Configuration File

Create `.solverrc.json` for default settings:

```json
{
  "method": "conjugate_gradient",
  "tolerance": 1e-8,
  "maxIterations": 1000,
  "verbose": true,
  "output": {
    "format": "json",
    "precision": 6
  },
  "server": {
    "port": 3000,
    "cors": true,
    "workers": 4
  }
}
```

## Environment Variables

```bash
# Set default options via environment
export SOLVER_METHOD=hybrid
export SOLVER_TOLERANCE=1e-8
export SOLVER_PORT=8080
export SOLVER_DEBUG=true
```

## Tips and Tricks

### Performance Optimization

```bash
# Use hybrid for large sparse systems
sublinear-time-solver solve --size 100000 --method hybrid --parallel

# Stream for real-time feedback
sublinear-time-solver solve --matrix big.mtx --stream | tee progress.log
```

### Debugging

```bash
# Enable debug output
DEBUG=* sublinear-time-solver solve --debug --verbose

# Profile performance
sublinear-time-solver benchmark --profile --output profile.json
```

### Scripting

```bash
#!/bin/bash
# Batch processing script

for matrix in data/*.mtx; do
  echo "Processing $matrix..."
  sublinear-time-solver solve \
    --matrix "$matrix" \
    --vector "${matrix%.mtx}.rhs" \
    --output "${matrix%.mtx}.sol" \
    --method auto
done
```

## Common Issues

### Memory Limits

For large systems, increase Node.js memory:

```bash
NODE_OPTIONS="--max-old-space-size=8192" sublinear-time-solver solve --size 1000000
```

### Performance Tuning

```bash
# Use multiple workers for parallel solving
sublinear-time-solver serve --workers 8

# Optimize for specific matrix structure
sublinear-time-solver solve --method forward_push --optimize-for sparse
```