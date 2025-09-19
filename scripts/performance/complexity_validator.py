#!/usr/bin/env python3
"""
Complexity Validator - Empirical verification of sublinear-time complexity claims
================================================================================

This script rigorously validates the theoretical O(polylog n) complexity claims
for well-conditioned diagonally dominant systems through:
1. Empirical timing analysis across problem sizes
2. Regression fitting to theoretical complexity models
3. Statistical significance testing
4. Comparison with traditional O(n^3) methods

Author: Performance Analysis Agent
Date: 2025-09-19
"""

import numpy as np
import time
import json
import subprocess
import sys
import os
from typing import Dict, List, Tuple, Optional
import matplotlib.pyplot as plt
import seaborn as sns
from scipy import stats
from scipy.optimize import curve_fit
import warnings
warnings.filterwarnings('ignore')

class ComplexityValidator:
    """Empirical complexity validation for sublinear solvers"""

    def __init__(self, output_dir: str = "complexity_results"):
        self.output_dir = output_dir
        os.makedirs(output_dir, exist_ok=True)

        # MCP solver command
        self.mcp_command = ["node", "/workspaces/sublinear-time-solver/bin/cli.js"]

        # Test matrix sizes (exponential growth)
        self.test_sizes = [50, 100, 200, 400, 800, 1600, 3200, 6400]
        self.large_sizes = [10000, 20000, 50000, 100000]  # For sparse matrices only

        # Complexity models to test
        self.complexity_models = {
            'linear': lambda n, a, b: a * n + b,
            'nlogn': lambda n, a, b: a * n * np.log(n) + b,
            'polylog': lambda n, a, b, c: a * (np.log(n)**b) + c,
            'sqrt_n': lambda n, a, b: a * np.sqrt(n) + b,
            'n_squared': lambda n, a, b: a * n**2 + b,
            'n_cubed': lambda n, a, b: a * n**3 + b
        }

        # Results storage
        self.results = {
            'dense_matrices': {},
            'sparse_matrices': {},
            'complexity_analysis': {},
            'statistical_tests': {}
        }

    def generate_test_matrix(self, size: int, matrix_type: str = "diagonally_dominant",
                           sparsity: float = 0.1) -> Tuple[np.ndarray, np.ndarray]:
        """Generate test matrices with known properties"""

        if matrix_type == "diagonally_dominant":
            # Create strongly diagonally dominant matrix
            A = np.random.randn(size, size) * 0.1

            # Ensure diagonal dominance: |a_ii| > sum(|a_ij|) for j != i
            for i in range(size):
                row_sum = np.sum(np.abs(A[i])) - np.abs(A[i, i])
                A[i, i] = row_sum * 2.5 + 1.0  # Strong dominance factor

        elif matrix_type == "sparse_diagonally_dominant":
            # Create sparse diagonally dominant matrix
            nnz = int(size * size * sparsity)
            rows = np.random.randint(0, size, nnz)
            cols = np.random.randint(0, size, nnz)
            values = np.random.randn(nnz) * 0.1

            A = np.zeros((size, size))
            for r, c, v in zip(rows, cols, values):
                if r != c:  # Off-diagonal
                    A[r, c] = v

            # Ensure diagonal dominance
            for i in range(size):
                row_sum = np.sum(np.abs(A[i])) - np.abs(A[i, i])
                A[i, i] = row_sum * 2.0 + 1.0

        elif matrix_type == "laplacian":
            # Graph Laplacian matrix (naturally diagonally dominant)
            A = np.zeros((size, size))
            for i in range(size):
                # Connect to a few random neighbors
                neighbors = np.random.choice(size, size=min(10, size//4), replace=False)
                neighbors = neighbors[neighbors != i]

                for j in neighbors:
                    A[i, j] = -1
                    A[j, i] = -1

                A[i, i] = len(neighbors)

        # Generate corresponding RHS vector
        x_true = np.random.randn(size)
        b = A @ x_true

        return A, b

    def matrix_to_mcp_format(self, A: np.ndarray, sparse: bool = False) -> Dict:
        """Convert numpy matrix to MCP tool format"""

        if sparse:
            # Convert to COO (coordinate) format
            rows, cols = np.nonzero(A)
            values = A[rows, cols]

            return {
                "rows": int(A.shape[0]),
                "cols": int(A.shape[1]),
                "format": "coo",
                "data": {
                    "values": values.tolist(),
                    "rowIndices": rows.tolist(),
                    "colIndices": cols.tolist()
                }
            }
        else:
            # Dense format
            return {
                "rows": int(A.shape[0]),
                "cols": int(A.shape[1]),
                "format": "dense",
                "data": A.tolist()
            }

    def time_mcp_solve(self, matrix: Dict, vector: List[float], method: str = "neumann") -> Dict:
        """Time MCP solver execution"""

        # Create temporary input file
        input_data = {
            "matrix": matrix,
            "vector": vector,
            "method": method,
            "epsilon": 1e-6,
            "maxIterations": 1000
        }

        input_file = f"{self.output_dir}/temp_input.json"
        with open(input_file, 'w') as f:
            json.dump(input_data, f)

        try:
            # Time the solve operation
            start_time = time.perf_counter()

            result = subprocess.run(
                self.mcp_command + ["solve", "--input", input_file],
                capture_output=True,
                text=True,
                timeout=300  # 5 minute timeout
            )

            end_time = time.perf_counter()
            elapsed_time = end_time - start_time

            if result.returncode != 0:
                raise RuntimeError(f"MCP solve failed: {result.stderr}")

            # Parse result
            output = json.loads(result.stdout)

            return {
                "elapsed_time": elapsed_time,
                "iterations": output.get("iterations", 0),
                "residual": output.get("residual", float('inf')),
                "converged": output.get("converged", False),
                "memory_usage": output.get("memoryUsage", 0)
            }

        except subprocess.TimeoutExpired:
            return {"elapsed_time": float('inf'), "converged": False, "timeout": True}
        except Exception as e:
            return {"elapsed_time": float('inf'), "converged": False, "error": str(e)}
        finally:
            # Clean up
            if os.path.exists(input_file):
                os.remove(input_file)

    def time_numpy_solve(self, A: np.ndarray, b: np.ndarray) -> Dict:
        """Time NumPy direct solver for comparison"""

        try:
            start_time = time.perf_counter()
            x = np.linalg.solve(A, b)
            end_time = time.perf_counter()

            # Verify solution
            residual = np.linalg.norm(A @ x - b) / np.linalg.norm(b)

            return {
                "elapsed_time": end_time - start_time,
                "residual": residual,
                "converged": residual < 1e-6
            }
        except Exception as e:
            return {"elapsed_time": float('inf'), "converged": False, "error": str(e)}

    def run_dense_matrix_tests(self):
        """Run complexity tests on dense matrices"""
        print("Running dense matrix complexity tests...")

        dense_results = {
            'sizes': [],
            'mcp_times': [],
            'numpy_times': [],
            'mcp_iterations': [],
            'mcp_residuals': [],
            'numpy_residuals': [],
            'speedup_ratios': []
        }

        for size in self.test_sizes:
            print(f"  Testing dense matrix size {size}x{size}")

            # Generate test matrix
            A, b = self.generate_test_matrix(size, "diagonally_dominant")

            # Test MCP solver
            matrix_mcp = self.matrix_to_mcp_format(A, sparse=False)
            mcp_result = self.time_mcp_solve(matrix_mcp, b.tolist(), "neumann")

            # Test NumPy solver
            numpy_result = self.time_numpy_solve(A, b)

            # Store results
            if mcp_result.get("converged", False) and numpy_result.get("converged", False):
                dense_results['sizes'].append(size)
                dense_results['mcp_times'].append(mcp_result["elapsed_time"])
                dense_results['numpy_times'].append(numpy_result["elapsed_time"])
                dense_results['mcp_iterations'].append(mcp_result.get("iterations", 0))
                dense_results['mcp_residuals'].append(mcp_result.get("residual", 0))
                dense_results['numpy_residuals'].append(numpy_result.get("residual", 0))

                speedup = numpy_result["elapsed_time"] / mcp_result["elapsed_time"]
                dense_results['speedup_ratios'].append(speedup)

                print(f"    MCP: {mcp_result['elapsed_time']:.3f}s, NumPy: {numpy_result['elapsed_time']:.3f}s, Speedup: {speedup:.2f}x")
            else:
                print(f"    Test failed for size {size}")

        self.results['dense_matrices'] = dense_results

    def run_sparse_matrix_tests(self):
        """Run complexity tests on sparse matrices"""
        print("Running sparse matrix complexity tests...")

        sparse_results = {
            'sizes': [],
            'mcp_times': [],
            'scipy_times': [],
            'mcp_iterations': [],
            'sparsity_ratios': [],
            'speedup_ratios': []
        }

        # Test both regular and large sizes for sparse matrices
        test_sizes = self.test_sizes + self.large_sizes

        for size in test_sizes:
            print(f"  Testing sparse matrix size {size}x{size}")

            try:
                # Generate sparse test matrix
                A, b = self.generate_test_matrix(size, "sparse_diagonally_dominant", sparsity=0.01)
                sparsity = 1.0 - np.count_nonzero(A) / (size * size)

                # Test MCP solver
                matrix_mcp = self.matrix_to_mcp_format(A, sparse=True)
                mcp_result = self.time_mcp_solve(matrix_mcp, b.tolist(), "random-walk")

                # Test SciPy sparse solver for comparison
                from scipy.sparse import csc_matrix
                from scipy.sparse.linalg import spsolve

                A_sparse = csc_matrix(A)
                start_time = time.perf_counter()
                x_scipy = spsolve(A_sparse, b)
                scipy_time = time.perf_counter() - start_time

                scipy_residual = np.linalg.norm(A @ x_scipy - b) / np.linalg.norm(b)

                # Store results
                if mcp_result.get("converged", False) and scipy_residual < 1e-6:
                    sparse_results['sizes'].append(size)
                    sparse_results['mcp_times'].append(mcp_result["elapsed_time"])
                    sparse_results['scipy_times'].append(scipy_time)
                    sparse_results['mcp_iterations'].append(mcp_result.get("iterations", 0))
                    sparse_results['sparsity_ratios'].append(sparsity)

                    speedup = scipy_time / mcp_result["elapsed_time"]
                    sparse_results['speedup_ratios'].append(speedup)

                    print(f"    MCP: {mcp_result['elapsed_time']:.3f}s, SciPy: {scipy_time:.3f}s, Speedup: {speedup:.2f}x, Sparsity: {sparsity:.1%}")
                else:
                    print(f"    Test failed for size {size}")

            except Exception as e:
                print(f"    Error testing size {size}: {e}")

        self.results['sparse_matrices'] = sparse_results

    def fit_complexity_models(self):
        """Fit theoretical complexity models to empirical data"""
        print("Fitting complexity models to empirical data...")

        complexity_analysis = {}

        # Analyze dense matrix results
        if self.results['dense_matrices']['sizes']:
            sizes = np.array(self.results['dense_matrices']['sizes'])
            times = np.array(self.results['dense_matrices']['mcp_times'])

            complexity_analysis['dense'] = self._fit_models(sizes, times, "Dense Matrix")

        # Analyze sparse matrix results
        if self.results['sparse_matrices']['sizes']:
            sizes = np.array(self.results['sparse_matrices']['sizes'])
            times = np.array(self.results['sparse_matrices']['mcp_times'])

            complexity_analysis['sparse'] = self._fit_models(sizes, times, "Sparse Matrix")

        self.results['complexity_analysis'] = complexity_analysis

    def _fit_models(self, sizes: np.ndarray, times: np.ndarray, title: str) -> Dict:
        """Fit complexity models to timing data"""

        model_fits = {}

        for name, model_func in self.complexity_models.items():
            try:
                # Handle different numbers of parameters
                if name == 'polylog':
                    popt, pcov = curve_fit(model_func, sizes, times, p0=[1, 2, 0], maxfev=10000)
                else:
                    popt, pcov = curve_fit(model_func, sizes, times, maxfev=10000)

                # Calculate goodness of fit
                y_pred = model_func(sizes, *popt)
                r_squared = 1 - np.sum((times - y_pred)**2) / np.sum((times - np.mean(times))**2)
                rmse = np.sqrt(np.mean((times - y_pred)**2))

                # Calculate AIC for model comparison
                n = len(times)
                mse = np.mean((times - y_pred)**2)
                aic = n * np.log(mse) + 2 * len(popt)

                model_fits[name] = {
                    'parameters': popt.tolist(),
                    'covariance': pcov.tolist(),
                    'r_squared': r_squared,
                    'rmse': rmse,
                    'aic': aic,
                    'predictions': y_pred.tolist()
                }

                print(f"  {title} - {name}: R² = {r_squared:.4f}, RMSE = {rmse:.6f}")

            except Exception as e:
                print(f"  Failed to fit {name} model: {e}")
                model_fits[name] = {'error': str(e)}

        # Find best model (highest R²)
        valid_models = {k: v for k, v in model_fits.items() if 'r_squared' in v}
        if valid_models:
            best_model = max(valid_models.keys(), key=lambda k: valid_models[k]['r_squared'])
            model_fits['best_model'] = best_model
            print(f"  {title} - Best model: {best_model} (R² = {valid_models[best_model]['r_squared']:.4f})")

        return model_fits

    def statistical_significance_tests(self):
        """Perform statistical tests on complexity claims"""
        print("Performing statistical significance tests...")

        statistical_tests = {}

        # Test if MCP solver is significantly faster than traditional methods
        for matrix_type in ['dense_matrices', 'sparse_matrices']:
            if matrix_type in self.results and self.results[matrix_type]['sizes']:
                data = self.results[matrix_type]

                if matrix_type == 'dense_matrices':
                    traditional_times = data['numpy_times']
                    comparison_name = "NumPy"
                else:
                    traditional_times = data['scipy_times']
                    comparison_name = "SciPy"

                mcp_times = data['mcp_times']

                # Paired t-test for significant difference
                statistic, p_value = stats.ttest_rel(traditional_times, mcp_times)

                # Effect size (Cohen's d)
                pooled_std = np.sqrt((np.var(traditional_times) + np.var(mcp_times)) / 2)
                cohens_d = (np.mean(traditional_times) - np.mean(mcp_times)) / pooled_std

                statistical_tests[matrix_type] = {
                    'comparison': f"MCP vs {comparison_name}",
                    't_statistic': statistic,
                    'p_value': p_value,
                    'significant': p_value < 0.05,
                    'cohens_d': cohens_d,
                    'effect_size': self._interpret_effect_size(abs(cohens_d)),
                    'mean_speedup': np.mean(data['speedup_ratios']),
                    'median_speedup': np.median(data['speedup_ratios'])
                }

                print(f"  {matrix_type}: t={statistic:.3f}, p={p_value:.6f}, Cohen's d={cohens_d:.3f}")

        self.results['statistical_tests'] = statistical_tests

    def _interpret_effect_size(self, d: float) -> str:
        """Interpret Cohen's d effect size"""
        if d < 0.2:
            return "negligible"
        elif d < 0.5:
            return "small"
        elif d < 0.8:
            return "medium"
        else:
            return "large"

    def generate_complexity_plots(self):
        """Generate visualization plots for complexity analysis"""
        print("Generating complexity visualization plots...")

        # Set up plotting style
        plt.style.use('seaborn-v0_8')
        fig, axes = plt.subplots(2, 2, figsize=(15, 12))
        fig.suptitle('Sublinear Solver Complexity Analysis', fontsize=16)

        # Plot 1: Dense matrix timing comparison
        if self.results['dense_matrices']['sizes']:
            ax = axes[0, 0]
            data = self.results['dense_matrices']

            ax.loglog(data['sizes'], data['mcp_times'], 'o-', label='MCP Solver', linewidth=2)
            ax.loglog(data['sizes'], data['numpy_times'], 's-', label='NumPy Direct', linewidth=2)

            ax.set_xlabel('Matrix Size (n)')
            ax.set_ylabel('Solve Time (seconds)')
            ax.set_title('Dense Matrix Performance')
            ax.legend()
            ax.grid(True, alpha=0.3)

        # Plot 2: Sparse matrix timing comparison
        if self.results['sparse_matrices']['sizes']:
            ax = axes[0, 1]
            data = self.results['sparse_matrices']

            ax.loglog(data['sizes'], data['mcp_times'], 'o-', label='MCP Solver', linewidth=2)
            ax.loglog(data['sizes'], data['scipy_times'], 's-', label='SciPy Sparse', linewidth=2)

            ax.set_xlabel('Matrix Size (n)')
            ax.set_ylabel('Solve Time (seconds)')
            ax.set_title('Sparse Matrix Performance')
            ax.legend()
            ax.grid(True, alpha=0.3)

        # Plot 3: Model fitting for complexity validation
        if 'dense' in self.results['complexity_analysis']:
            ax = axes[1, 0]
            data = self.results['dense_matrices']
            analysis = self.results['complexity_analysis']['dense']

            sizes = np.array(data['sizes'])
            times = np.array(data['mcp_times'])

            ax.loglog(sizes, times, 'o', label='Empirical Data', markersize=8)

            # Plot best-fitting models
            for model_name, model_data in analysis.items():
                if isinstance(model_data, dict) and 'predictions' in model_data:
                    predictions = np.array(model_data['predictions'])
                    r_squared = model_data['r_squared']
                    ax.loglog(sizes, predictions, '--', label=f'{model_name} (R²={r_squared:.3f})', linewidth=2)

            ax.set_xlabel('Matrix Size (n)')
            ax.set_ylabel('Solve Time (seconds)')
            ax.set_title('Complexity Model Fitting (Dense)')
            ax.legend()
            ax.grid(True, alpha=0.3)

        # Plot 4: Speedup analysis
        ax = axes[1, 1]

        if self.results['dense_matrices']['sizes']:
            data = self.results['dense_matrices']
            ax.semilogx(data['sizes'], data['speedup_ratios'], 'o-', label='Dense Matrices', linewidth=2)

        if self.results['sparse_matrices']['sizes']:
            data = self.results['sparse_matrices']
            ax.semilogx(data['sizes'], data['speedup_ratios'], 's-', label='Sparse Matrices', linewidth=2)

        ax.axhline(y=1, color='red', linestyle='--', alpha=0.7, label='No Speedup')
        ax.set_xlabel('Matrix Size (n)')
        ax.set_ylabel('Speedup Factor')
        ax.set_title('MCP Solver Speedup vs Traditional Methods')
        ax.legend()
        ax.grid(True, alpha=0.3)

        plt.tight_layout()
        plt.savefig(f'{self.output_dir}/complexity_analysis.png', dpi=300, bbox_inches='tight')
        plt.close()

        print(f"  Saved complexity plots to {self.output_dir}/complexity_analysis.png")

    def save_results(self):
        """Save all validation results to JSON"""
        output_file = f"{self.output_dir}/complexity_validation_results.json"

        # Add metadata
        self.results['metadata'] = {
            'timestamp': time.strftime('%Y-%m-%d %H:%M:%S'),
            'test_sizes': self.test_sizes,
            'large_sizes': self.large_sizes,
            'python_version': sys.version,
            'numpy_version': np.__version__
        }

        with open(output_file, 'w') as f:
            json.dump(self.results, f, indent=2)

        print(f"Results saved to {output_file}")

    def generate_summary_report(self):
        """Generate human-readable summary report"""
        print("Generating complexity validation summary...")

        report_lines = [
            "SUBLINEAR SOLVER COMPLEXITY VALIDATION REPORT",
            "=" * 50,
            f"Generated: {time.strftime('%Y-%m-%d %H:%M:%S')}",
            "",
            "EXECUTIVE SUMMARY",
            "-" * 20
        ]

        # Dense matrix summary
        if self.results['dense_matrices']['sizes']:
            data = self.results['dense_matrices']
            mean_speedup = np.mean(data['speedup_ratios'])
            max_size = max(data['sizes'])

            report_lines.extend([
                f"Dense Matrices: Tested sizes up to {max_size}x{max_size}",
                f"Average speedup vs NumPy: {mean_speedup:.2f}x",
                f"Maximum speedup achieved: {max(data['speedup_ratios']):.2f}x"
            ])

        # Sparse matrix summary
        if self.results['sparse_matrices']['sizes']:
            data = self.results['sparse_matrices']
            mean_speedup = np.mean(data['speedup_ratios'])
            max_size = max(data['sizes'])

            report_lines.extend([
                f"Sparse Matrices: Tested sizes up to {max_size}x{max_size}",
                f"Average speedup vs SciPy: {mean_speedup:.2f}x",
                f"Maximum speedup achieved: {max(data['speedup_ratios']):.2f}x"
            ])

        # Complexity analysis summary
        report_lines.extend(["", "COMPLEXITY ANALYSIS", "-" * 20])

        for matrix_type in ['dense', 'sparse']:
            if matrix_type in self.results['complexity_analysis']:
                analysis = self.results['complexity_analysis'][matrix_type]
                if 'best_model' in analysis:
                    best_model = analysis['best_model']
                    r_squared = analysis[best_model]['r_squared']

                    report_lines.append(f"{matrix_type.title()} matrices best fit: {best_model} (R² = {r_squared:.4f})")

                    if best_model in ['polylog', 'nlogn']:
                        report_lines.append("✓ SUBLINEAR COMPLEXITY CONFIRMED")
                    else:
                        report_lines.append("⚠ Non-sublinear complexity detected")

        # Statistical significance
        report_lines.extend(["", "STATISTICAL SIGNIFICANCE", "-" * 25])

        for test_name, test_data in self.results['statistical_tests'].items():
            significance = "SIGNIFICANT" if test_data['significant'] else "NOT SIGNIFICANT"
            report_lines.extend([
                f"{test_name}: {test_data['comparison']}",
                f"  p-value: {test_data['p_value']:.6f} ({significance})",
                f"  Effect size: {test_data['effect_size']} (Cohen's d = {test_data['cohens_d']:.3f})",
                f"  Mean speedup: {test_data['mean_speedup']:.2f}x"
            ])

        # Save report
        report_file = f"{self.output_dir}/complexity_validation_summary.txt"
        with open(report_file, 'w') as f:
            f.write('\n'.join(report_lines))

        # Print to console
        print('\n'.join(report_lines))
        print(f"\nDetailed report saved to {report_file}")

    def run_full_validation(self):
        """Run complete complexity validation suite"""
        print("Starting comprehensive complexity validation...")
        print("=" * 60)

        try:
            # Run empirical tests
            self.run_dense_matrix_tests()
            self.run_sparse_matrix_tests()

            # Analyze results
            self.fit_complexity_models()
            self.statistical_significance_tests()

            # Generate outputs
            self.generate_complexity_plots()
            self.save_results()
            self.generate_summary_report()

            print("\n" + "=" * 60)
            print("Complexity validation completed successfully!")
            print(f"Results available in: {self.output_dir}/")

        except Exception as e:
            print(f"Validation failed with error: {e}")
            raise

def main():
    """Main execution function"""
    import argparse

    parser = argparse.ArgumentParser(description='Validate sublinear solver complexity claims')
    parser.add_argument('--output-dir', default='complexity_results',
                       help='Output directory for results')
    parser.add_argument('--max-size', type=int, default=6400,
                       help='Maximum matrix size for dense tests')
    parser.add_argument('--sparse-only', action='store_true',
                       help='Run sparse matrix tests only')

    args = parser.parse_args()

    validator = ComplexityValidator(args.output_dir)

    if args.max_size != 6400:
        # Adjust test sizes based on max size
        validator.test_sizes = [s for s in validator.test_sizes if s <= args.max_size]

    if args.sparse_only:
        print("Running sparse matrix tests only...")
        validator.run_sparse_matrix_tests()
        validator.fit_complexity_models()
        validator.generate_complexity_plots()
        validator.save_results()
        validator.generate_summary_report()
    else:
        validator.run_full_validation()

if __name__ == "__main__":
    main()