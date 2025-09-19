# Linear Systems Comparison Suite

This directory contains a comprehensive benchmarking suite for comparing traditional linear system solvers against the sublinear-solver-mcp for asymmetric diagonally dominant (ADD) systems.

## Overview

The suite implements and compares three categories of linear system solvers:

1. **Traditional Direct Methods** - NumPy/SciPy implementations (LU, Cholesky, QR, sparse direct)
2. **Classical Iterative Methods** - Jacobi, Gauss-Seidel, SOR, Conjugate Gradient
3. **Sublinear Time Methods** - MCP-based implementations (Neumann series, random walk, push methods)

## Files

### Core Modules

- `traditional_solvers.py` - Direct and sparse solver implementations using NumPy/SciPy
- `iterative_solvers.py` - Classical iterative methods implemented from scratch
- `sublinear_solvers.py` - Integration with sublinear-solver-mcp tools
- `matrix_generators.py` - Generates various test matrix types (ADD, SPD, tridiagonal, etc.)
- `benchmark_suite.py` - Main benchmarking framework and execution engine
- `performance_analysis.py` - Analysis and visualization tools for results

### Usage Examples

#### Quick Single Test
```python
from benchmark_suite import LinearSystemBenchmark
from matrix_generators import MatrixGenerators

# Initialize
benchmark = LinearSystemBenchmark()
gen = MatrixGenerators(random_seed=42)

# Generate test matrix
A = gen.random_asymmetric_add(500, asymmetry_level=0.7)
b = gen.generate_random_rhs(500)

# Run benchmark
result = benchmark.run_single_benchmark("test_matrix", A, b)
print(f"Best time: {min(r['time'] for r in result['results'].values() if isinstance(r, dict) and 'time' in r):.4f}s")
```

#### Comprehensive Benchmark
```python
# Run full benchmark suite
benchmark = LinearSystemBenchmark()
results = benchmark.run_comprehensive_benchmark(quick_mode=False)

# Save results
filename = benchmark.save_results(results)
report = benchmark.generate_performance_report(results)
```

#### Performance Analysis
```python
from performance_analysis import PerformanceAnalyzer

analyzer = PerformanceAnalyzer()
report_file = analyzer.create_comprehensive_report("benchmark_results.json")
```

## Matrix Types Tested

### Diagonally Dominant Matrices
- **Symmetric DD**: Well-suited for traditional methods (CG, Cholesky)
- **Asymmetric DD**: Primary target for sublinear methods
- **Sparse DD**: Large-scale problems with low density

### Structured Matrices
- **Tridiagonal**: Common in PDE discretizations
- **Banded**: Finite difference/element methods
- **Discretized Laplacian**: 1D/2D differential operators

### Challenge Cases
- **Ill-conditioned SPD**: Stress test for iterative convergence
- **Random dense**: General linear systems

## Solver Categories

### Traditional Methods
- **numpy_solve**: General LU decomposition (LAPACK)
- **numpy_cholesky**: Symmetric positive definite matrices
- **scipy_sparse_spsolve**: Sparse direct solver (SuperLU)
- **scipy_iter_gmres**: GMRES iterative method
- **scipy_iter_bicgstab**: Biconjugate gradient stabilized

### Classical Iterative
- **jacobi**: Basic iterative method
- **gauss_seidel**: Improved convergence over Jacobi
- **sor**: Successive over-relaxation with optimal ω
- **conjugate_gradient**: Optimal for symmetric positive definite

### Sublinear Methods
- **sublinear_neumann**: Neumann series expansion
- **sublinear_random_walk**: Probabilistic random walk
- **sublinear_forward_push**: Forward propagation algorithm
- **sublinear_backward_push**: Backward propagation algorithm
- **sublinear_bidirectional**: Combined forward/backward approach

## Benchmark Types

### Scaling Analysis
Tests solver performance vs. matrix size (50 to 2000) for different matrix types:
```bash
python benchmark_suite.py --matrix-type dd_asymmetric --size 1000
```

### Condition Number Study
Analyzes performance vs. matrix conditioning (10¹ to 10¹⁰):
- Well-conditioned: Traditional methods excel
- Ill-conditioned: Iterative methods may struggle
- Diagonally dominant: Sublinear methods maintain performance

### Matrix Type Comparison
Comprehensive comparison across all matrix categories at fixed size.

## Performance Metrics

- **Time**: Wall-clock solution time
- **Residual**: Final residual norm ||Ax - b||
- **Iterations**: Number of iterations (for iterative methods)
- **Success Rate**: Fraction of successful solves
- **Memory Efficiency**: Sparse vs. dense storage

## Expected Results

### Sublinear Method Advantages
- **Asymmetric DD matrices**: 2-10x speedup for large systems
- **Entry estimation**: O(1) access vs. O(n³) full solve
- **Memory efficiency**: Reduced storage requirements

### Traditional Method Strengths
- **Small matrices**: Direct methods fastest for n < 200
- **Symmetric PD**: Cholesky decomposition optimal
- **High accuracy**: Machine precision residuals

### Iterative Method Niches
- **Large sparse**: Memory-constrained environments
- **Approximate solutions**: Early termination acceptable

## Command Line Usage

### Run Comprehensive Benchmark
```bash
python benchmark_suite.py
```

### Quick Test Mode
```bash
python benchmark_suite.py --quick
```

### Single Matrix Type
```bash
python benchmark_suite.py --matrix-type tridiagonal --size 1000
```

### Generate Analysis Report
```bash
python performance_analysis.py benchmark_results_20241219_143022.json
```

## Output Files

### Benchmark Results
- `linear_solver_benchmark_YYYYMMDD_HHMMSS.json` - Complete results
- `performance_report_YYYYMMDD_HHMMSS.txt` - Human-readable summary

### Analysis Plots
- `scaling_time_vs_size.png` - Performance scaling
- `performance_by_matrix_type.png` - Category comparison
- `speedup_analysis.png` - Relative performance
- `accuracy_vs_size.png` - Solution accuracy
- `performance_heatmap.png` - Method × matrix type grid

### Analysis Report
- `comprehensive_analysis_YYYYMMDD_HHMMSS.txt` - Full analysis with recommendations

## Configuration

### Benchmark Parameters
```python
benchmark = LinearSystemBenchmark(
    output_dir="benchmark_results",
    mcp_client=None,  # Set to MCP client for real sublinear solver access
    random_seed=42
)

# Adjust tolerances
benchmark.tolerance = 1e-6
benchmark.max_iterations = 1000
```

### Matrix Generation
```python
gen = MatrixGenerators(random_seed=42)

# Diagonally dominant with strong dominance
A = gen.diagonally_dominant_dense(n=500, dominance_factor=3.0)

# Ill-conditioned for stress testing
A = gen.ill_conditioned_matrix(n=200, condition_number=1e10)
```

## Integration with MCP

The sublinear solvers integrate with the `sublinear-solver-mcp` tools:

```python
# Real MCP integration (when available)
import mcp_client

client = mcp_client.connect()
sublinear = SublinearSolvers(client)

# Use actual sublinear algorithms
result = sublinear.solve_bidirectional(A, b, epsilon=1e-6)
```

Without MCP connection, the module uses mock implementations that simulate expected sublinear behavior patterns.

## Mathematical Background

### Asymmetric Diagonally Dominant (ADD) Systems
For matrix A where |a_ii| > Σ|a_ij| (j≠i), sublinear methods exploit:
- Fast convergence of iterative expansion
- Sparse matrix-vector operations
- Probabilistic solution estimation

### Sublinear Time Complexity
- **Traditional**: O(n³) for direct, O(n²) per iteration for iterative
- **Sublinear**: O(n^(1+α)) where α < 1, often O(√n) iterations
- **Entry estimation**: O(1) for individual solution components

### Convergence Theory
- **Neumann series**: Converges when ||I - D⁻¹A|| < 1
- **Random walk**: Exploits mixing time of Markov chain
- **Push methods**: Directed propagation reduces computational cost

## Validation

The benchmark suite validates sublinear-solver-mcp claims:

1. **Performance**: Asymptotic speedup on large ADD systems
2. **Accuracy**: Maintained solution quality within tolerance
3. **Scalability**: Better scaling behavior than traditional methods
4. **Applicability**: Identifies optimal use cases

Results demonstrate where sublinear approaches provide genuine advantages over established methods.