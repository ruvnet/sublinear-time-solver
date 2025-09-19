#!/usr/bin/env python3
"""
Convergence Analysis Framework for Linear Systems Comparison

This module provides comprehensive convergence analysis capabilities for comparing
sublinear solvers against traditional linear algebra approaches. Analyzes iteration
counts, residual progression, convergence rates, and stability characteristics.

Author: Linear Systems Comparison Agent
"""

import numpy as np
import matplotlib.pyplot as plt
import time
import json
from typing import Dict, List, Tuple, Any, Optional, Callable
from dataclasses import dataclass, asdict
import warnings
from scipy.sparse import issparse
import pandas as pd
from pathlib import Path

@dataclass
class ConvergenceMetrics:
    """Container for convergence analysis results."""
    method_name: str
    converged: bool
    iterations: int
    final_residual: float
    solve_time: float
    residual_history: List[float]
    convergence_rate: float
    linear_convergence_rate: Optional[float]
    superlinear_factor: Optional[float]
    stagnation_count: int
    oscillation_index: float
    efficiency_score: float
    memory_peak: Optional[float]

@dataclass
class ComparisonResult:
    """Container for method comparison results."""
    matrix_properties: Dict[str, Any]
    convergence_metrics: List[ConvergenceMetrics]
    best_method: str
    speedup_factors: Dict[str, float]
    accuracy_comparison: Dict[str, float]
    stability_analysis: Dict[str, Any]

class ConvergenceAnalyzer:
    """
    Advanced convergence analysis for comparing iterative linear system solvers.
    Provides detailed metrics, visualization, and statistical analysis.
    """

    def __init__(self, tolerance: float = 1e-12, max_iterations: int = 10000):
        """
        Initialize convergence analyzer.

        Args:
            tolerance: Default convergence tolerance
            max_iterations: Default maximum iterations
        """
        self.tolerance = tolerance
        self.max_iterations = max_iterations
        self.results_history = []

    def analyze_solver_convergence(self,
                                 solve_function: Callable,
                                 A: np.ndarray,
                                 b: np.ndarray,
                                 method_name: str,
                                 true_solution: Optional[np.ndarray] = None,
                                 monitor_memory: bool = True) -> ConvergenceMetrics:
        """
        Analyze convergence behavior of a single solver method.

        Args:
            solve_function: Function that solves Ax=b and returns residual history
            A: Coefficient matrix
            b: Right-hand side vector
            method_name: Name of the solving method
            true_solution: Known true solution for accuracy analysis
            monitor_memory: Whether to monitor memory usage

        Returns:
            Comprehensive convergence metrics
        """
        # Track memory usage if requested
        initial_memory = self._get_memory_usage() if monitor_memory else None

        # Execute solver with residual tracking
        start_time = time.perf_counter()

        try:
            result = solve_function(A, b)
            solve_time = time.perf_counter() - start_time

            # Extract solution and metadata
            x_solution = result.get('solution')
            success = result.get('success', False)
            iterations = result.get('iterations', 0)
            final_residual = result.get('residual', float('inf'))

            # Get residual history if available, otherwise compute
            if 'residual_history' in result:
                residual_history = result['residual_history']
            else:
                # If no history provided, estimate from final residual
                if success and x_solution is not None:
                    actual_residual = np.linalg.norm(A @ x_solution - b)
                    residual_history = [actual_residual]
                else:
                    residual_history = [final_residual] if final_residual != float('inf') else []

        except Exception as e:
            solve_time = time.perf_counter() - start_time
            return ConvergenceMetrics(
                method_name=method_name,
                converged=False,
                iterations=0,
                final_residual=float('inf'),
                solve_time=solve_time,
                residual_history=[],
                convergence_rate=0.0,
                linear_convergence_rate=None,
                superlinear_factor=None,
                stagnation_count=0,
                oscillation_index=0.0,
                efficiency_score=0.0,
                memory_peak=None
            )

        # Memory usage analysis
        peak_memory = self._get_memory_usage() if monitor_memory else None
        memory_peak = (peak_memory - initial_memory) if (peak_memory and initial_memory) else None

        # Convergence rate analysis
        conv_rate, linear_rate, superlinear_factor = self._analyze_convergence_rate(residual_history)

        # Stability analysis
        stagnation_count = self._count_stagnation(residual_history)
        oscillation_index = self._compute_oscillation_index(residual_history)

        # Efficiency score (accuracy per unit time)
        accuracy_score = -np.log10(max(final_residual, 1e-16)) if final_residual > 0 else 16
        efficiency_score = accuracy_score / max(solve_time, 1e-6) if solve_time > 0 else 0

        return ConvergenceMetrics(
            method_name=method_name,
            converged=success and final_residual < self.tolerance,
            iterations=iterations,
            final_residual=final_residual,
            solve_time=solve_time,
            residual_history=residual_history,
            convergence_rate=conv_rate,
            linear_convergence_rate=linear_rate,
            superlinear_factor=superlinear_factor,
            stagnation_count=stagnation_count,
            oscillation_index=oscillation_index,
            efficiency_score=efficiency_score,
            memory_peak=memory_peak
        )

    def compare_multiple_solvers(self,
                               solvers: Dict[str, Callable],
                               A: np.ndarray,
                               b: np.ndarray,
                               true_solution: Optional[np.ndarray] = None) -> ComparisonResult:
        """
        Compare convergence behavior of multiple solver methods.

        Args:
            solvers: Dictionary mapping method names to solver functions
            A: Coefficient matrix
            b: Right-hand side vector
            true_solution: Known true solution for reference

        Returns:
            Comprehensive comparison results
        """
        # Analyze matrix properties
        matrix_props = self._analyze_matrix_properties(A)

        # Analyze each solver
        convergence_results = []
        solve_times = {}
        accuracies = {}

        for method_name, solver_func in solvers.items():
            print(f"Analyzing convergence for {method_name}...")

            metrics = self.analyze_solver_convergence(
                solver_func, A, b, method_name, true_solution
            )
            convergence_results.append(metrics)

            solve_times[method_name] = metrics.solve_time

            # Compute accuracy vs true solution if available
            if true_solution is not None and metrics.converged:
                try:
                    result = solver_func(A, b)
                    x_computed = result.get('solution')
                    if x_computed is not None:
                        accuracy = np.linalg.norm(x_computed - true_solution)
                        accuracies[method_name] = accuracy
                    else:
                        accuracies[method_name] = float('inf')
                except:
                    accuracies[method_name] = float('inf')
            else:
                accuracies[method_name] = metrics.final_residual

        # Find best performing method
        converged_methods = [m for m in convergence_results if m.converged]
        if converged_methods:
            best_method = min(converged_methods, key=lambda x: x.solve_time).method_name
        else:
            best_method = min(convergence_results, key=lambda x: x.final_residual).method_name

        # Compute speedup factors
        if converged_methods:
            baseline_time = max(m.solve_time for m in converged_methods)
            speedup_factors = {
                m.method_name: baseline_time / max(m.solve_time, 1e-9)
                for m in converged_methods
            }
        else:
            speedup_factors = {m.method_name: 1.0 for m in convergence_results}

        # Stability analysis
        stability_analysis = self._analyze_stability(convergence_results)

        return ComparisonResult(
            matrix_properties=matrix_props,
            convergence_metrics=convergence_results,
            best_method=best_method,
            speedup_factors=speedup_factors,
            accuracy_comparison=accuracies,
            stability_analysis=stability_analysis
        )

    def run_scalability_analysis(self,
                                solvers: Dict[str, Callable],
                                matrix_generator: Callable,
                                sizes: List[int],
                                num_trials: int = 3) -> Dict[str, Any]:
        """
        Analyze solver scalability across different matrix sizes.

        Args:
            solvers: Dictionary of solver functions
            matrix_generator: Function that generates test matrices
            sizes: List of matrix sizes to test
            num_trials: Number of trials per size for averaging

        Returns:
            Scalability analysis results
        """
        scalability_results = {
            'sizes': sizes,
            'methods': list(solvers.keys()),
            'avg_times': {method: [] for method in solvers.keys()},
            'avg_iterations': {method: [] for method in solvers.keys()},
            'avg_residuals': {method: [] for method in solvers.keys()},
            'success_rates': {method: [] for method in solvers.keys()},
            'scaling_coefficients': {},
            'complexity_estimates': {}
        }

        for size in sizes:
            print(f"\nAnalyzing scalability for size {size}x{size}...")

            size_results = {method: {'times': [], 'iterations': [], 'residuals': [], 'successes': []}
                          for method in solvers.keys()}

            for trial in range(num_trials):
                # Generate test matrix
                A, b = matrix_generator(size)

                for method_name, solver_func in solvers.items():
                    try:
                        metrics = self.analyze_solver_convergence(
                            solver_func, A, b, method_name, monitor_memory=False
                        )

                        size_results[method_name]['times'].append(metrics.solve_time)
                        size_results[method_name]['iterations'].append(metrics.iterations)
                        size_results[method_name]['residuals'].append(metrics.final_residual)
                        size_results[method_name]['successes'].append(metrics.converged)

                    except Exception as e:
                        print(f"Error with {method_name} at size {size}: {e}")
                        size_results[method_name]['times'].append(float('inf'))
                        size_results[method_name]['iterations'].append(0)
                        size_results[method_name]['residuals'].append(float('inf'))
                        size_results[method_name]['successes'].append(False)

            # Compute averages for this size
            for method_name in solvers.keys():
                valid_times = [t for t in size_results[method_name]['times'] if t != float('inf')]
                valid_iters = [i for i in size_results[method_name]['iterations'] if i > 0]
                valid_residuals = [r for r in size_results[method_name]['residuals'] if r != float('inf')]

                scalability_results['avg_times'][method_name].append(
                    np.mean(valid_times) if valid_times else float('inf')
                )
                scalability_results['avg_iterations'][method_name].append(
                    np.mean(valid_iters) if valid_iters else 0
                )
                scalability_results['avg_residuals'][method_name].append(
                    np.mean(valid_residuals) if valid_residuals else float('inf')
                )
                scalability_results['success_rates'][method_name].append(
                    np.mean(size_results[method_name]['successes'])
                )

        # Compute scaling coefficients and complexity estimates
        for method_name in solvers.keys():
            times = scalability_results['avg_times'][method_name]
            valid_indices = [i for i, t in enumerate(times) if t != float('inf')]

            if len(valid_indices) >= 3:
                valid_sizes = [sizes[i] for i in valid_indices]
                valid_times = [times[i] for i in valid_indices]

                # Fit scaling relationship: time = a * n^b
                log_sizes = np.log(valid_sizes)
                log_times = np.log(valid_times)

                # Linear regression in log space
                A_fit = np.vstack([log_sizes, np.ones(len(log_sizes))]).T
                scaling_coeff, log_a = np.linalg.lstsq(A_fit, log_times, rcond=None)[0]

                scalability_results['scaling_coefficients'][method_name] = scaling_coeff
                scalability_results['complexity_estimates'][method_name] = f"O(n^{scaling_coeff:.2f})"

        return scalability_results

    def generate_convergence_plots(self,
                                 comparison_result: ComparisonResult,
                                 output_dir: str = "plots") -> List[str]:
        """
        Generate comprehensive convergence analysis plots.

        Args:
            comparison_result: Results from compare_multiple_solvers
            output_dir: Directory to save plots

        Returns:
            List of generated plot filenames
        """
        Path(output_dir).mkdir(exist_ok=True)
        plot_files = []

        # 1. Residual convergence plot
        plt.figure(figsize=(12, 8))
        for metrics in comparison_result.convergence_metrics:
            if metrics.residual_history:
                iterations = range(len(metrics.residual_history))
                plt.semilogy(iterations, metrics.residual_history,
                           label=f"{metrics.method_name} ({metrics.iterations} iter)",
                           marker='o', markersize=3)

        plt.xlabel('Iteration')
        plt.ylabel('Residual Norm (log scale)')
        plt.title('Convergence History Comparison')
        plt.legend()
        plt.grid(True, alpha=0.3)
        convergence_plot = f"{output_dir}/convergence_history.png"
        plt.savefig(convergence_plot, dpi=300, bbox_inches='tight')
        plt.close()
        plot_files.append(convergence_plot)

        # 2. Performance comparison
        fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(15, 6))

        methods = [m.method_name for m in comparison_result.convergence_metrics]
        times = [m.solve_time for m in comparison_result.convergence_metrics]
        accuracies = [-np.log10(max(m.final_residual, 1e-16)) for m in comparison_result.convergence_metrics]

        ax1.bar(methods, times)
        ax1.set_ylabel('Solve Time (seconds)')
        ax1.set_title('Solve Time Comparison')
        ax1.tick_params(axis='x', rotation=45)

        ax2.bar(methods, accuracies)
        ax2.set_ylabel('Accuracy (-log₁₀(residual))')
        ax2.set_title('Accuracy Comparison')
        ax2.tick_params(axis='x', rotation=45)

        plt.tight_layout()
        performance_plot = f"{output_dir}/performance_comparison.png"
        plt.savefig(performance_plot, dpi=300, bbox_inches='tight')
        plt.close()
        plot_files.append(performance_plot)

        # 3. Efficiency analysis (accuracy per time)
        plt.figure(figsize=(10, 6))
        efficiency_scores = [m.efficiency_score for m in comparison_result.convergence_metrics]
        colors = ['green' if m.converged else 'red' for m in comparison_result.convergence_metrics]

        bars = plt.bar(methods, efficiency_scores, color=colors, alpha=0.7)
        plt.ylabel('Efficiency Score (accuracy/time)')
        plt.title('Solver Efficiency Comparison')
        plt.tick_params(axis='x', rotation=45)
        plt.grid(True, alpha=0.3)

        # Add convergence status legend
        from matplotlib.patches import Patch
        legend_elements = [Patch(facecolor='green', alpha=0.7, label='Converged'),
                          Patch(facecolor='red', alpha=0.7, label='Failed')]
        plt.legend(handles=legend_elements)

        efficiency_plot = f"{output_dir}/efficiency_comparison.png"
        plt.savefig(efficiency_plot, dpi=300, bbox_inches='tight')
        plt.close()
        plot_files.append(efficiency_plot)

        return plot_files

    def export_detailed_report(self,
                             comparison_result: ComparisonResult,
                             output_file: str = "convergence_analysis_report.json") -> str:
        """
        Export detailed convergence analysis report.

        Args:
            comparison_result: Results from comparison analysis
            output_file: Output filename

        Returns:
            Path to generated report file
        """
        # Convert results to serializable format
        report_data = {
            'matrix_properties': comparison_result.matrix_properties,
            'convergence_metrics': [asdict(m) for m in comparison_result.convergence_metrics],
            'best_method': comparison_result.best_method,
            'speedup_factors': comparison_result.speedup_factors,
            'accuracy_comparison': comparison_result.accuracy_comparison,
            'stability_analysis': comparison_result.stability_analysis,
            'analysis_metadata': {
                'tolerance': self.tolerance,
                'max_iterations': self.max_iterations,
                'timestamp': time.strftime('%Y-%m-%d %H:%M:%S')
            }
        }

        with open(output_file, 'w') as f:
            json.dump(report_data, f, indent=2, default=str)

        return output_file

    def _analyze_convergence_rate(self, residual_history: List[float]) -> Tuple[float, Optional[float], Optional[float]]:
        """Analyze convergence rate from residual history."""
        if len(residual_history) < 3:
            return 0.0, None, None

        # Remove zeros and infinities
        valid_residuals = [r for r in residual_history if r > 1e-16 and r != float('inf')]
        if len(valid_residuals) < 3:
            return 0.0, None, None

        # Compute convergence factor
        ratios = []
        for i in range(1, len(valid_residuals)):
            if valid_residuals[i-1] > 1e-16:
                ratios.append(valid_residuals[i] / valid_residuals[i-1])

        convergence_rate = np.mean(ratios) if ratios else 1.0

        # Linear convergence rate analysis
        log_residuals = np.log(valid_residuals)
        if len(log_residuals) >= 3:
            iterations = np.arange(len(log_residuals))
            linear_fit = np.polyfit(iterations, log_residuals, 1)
            linear_rate = -linear_fit[0]  # Negative slope indicates convergence
        else:
            linear_rate = None

        # Superlinear factor (check if residual reduction accelerates)
        superlinear_factor = None
        if len(ratios) >= 3:
            ratio_trend = np.polyfit(range(len(ratios)), ratios, 1)[0]
            if ratio_trend < -1e-3:  # Ratios are decreasing significantly
                superlinear_factor = -ratio_trend

        return convergence_rate, linear_rate, superlinear_factor

    def _count_stagnation(self, residual_history: List[float], threshold: float = 1e-3) -> int:
        """Count periods of stagnation in convergence."""
        if len(residual_history) < 2:
            return 0

        stagnation_count = 0
        for i in range(1, len(residual_history)):
            if residual_history[i-1] > 0:
                relative_change = abs(residual_history[i] - residual_history[i-1]) / residual_history[i-1]
                if relative_change < threshold:
                    stagnation_count += 1

        return stagnation_count

    def _compute_oscillation_index(self, residual_history: List[float]) -> float:
        """Compute index measuring oscillatory behavior."""
        if len(residual_history) < 3:
            return 0.0

        # Count direction changes in residual
        direction_changes = 0
        for i in range(2, len(residual_history)):
            prev_trend = residual_history[i-1] - residual_history[i-2]
            curr_trend = residual_history[i] - residual_history[i-1]
            if prev_trend * curr_trend < 0:  # Sign change
                direction_changes += 1

        return direction_changes / max(len(residual_history) - 2, 1)

    def _analyze_matrix_properties(self, A: np.ndarray) -> Dict[str, Any]:
        """Analyze matrix properties relevant for convergence."""
        n = A.shape[0]
        props = {
            'size': n,
            'is_sparse': issparse(A),
            'density': 1.0 if not issparse(A) else A.nnz / (n * n)
        }

        if not issparse(A) or n < 1000:
            A_dense = A.toarray() if issparse(A) else A

            # Basic properties
            props.update({
                'condition_number': np.linalg.cond(A_dense),
                'determinant': np.linalg.det(A_dense),
                'trace': np.trace(A_dense),
                'frobenius_norm': np.linalg.norm(A_dense, 'fro'),
                'is_symmetric': np.allclose(A_dense, A_dense.T)
            })

            # Diagonal dominance
            diag = np.abs(np.diag(A_dense))
            off_diag_sum = np.sum(np.abs(A_dense), axis=1) - diag
            props['diagonal_dominance'] = {
                'strict': np.all(diag > off_diag_sum),
                'weak': np.all(diag >= off_diag_sum),
                'ratio': np.min(diag / (off_diag_sum + 1e-12))
            }

            # Eigenvalue analysis (for smaller matrices)
            if n <= 200:
                try:
                    eigenvals = np.linalg.eigvals(A_dense)
                    props['eigenvalue_analysis'] = {
                        'min_eigenvalue': float(np.min(np.real(eigenvals))),
                        'max_eigenvalue': float(np.max(np.real(eigenvals))),
                        'spectral_radius': float(np.max(np.abs(eigenvals))),
                        'is_positive_definite': np.all(np.real(eigenvals) > 1e-10)
                    }
                except:
                    props['eigenvalue_analysis'] = 'computation_failed'

        return props

    def _analyze_stability(self, convergence_results: List[ConvergenceMetrics]) -> Dict[str, Any]:
        """Analyze stability characteristics across methods."""
        stability = {
            'convergence_rate_variance': {},
            'robustness_score': {},
            'oscillation_severity': {},
            'stagnation_severity': {}
        }

        for metrics in convergence_results:
            method = metrics.method_name

            # Convergence rate stability
            if len(metrics.residual_history) > 1:
                ratios = []
                for i in range(1, len(metrics.residual_history)):
                    if metrics.residual_history[i-1] > 1e-16:
                        ratios.append(metrics.residual_history[i] / metrics.residual_history[i-1])

                stability['convergence_rate_variance'][method] = np.var(ratios) if ratios else 0.0
            else:
                stability['convergence_rate_variance'][method] = float('inf')

            # Robustness score (combines convergence success and efficiency)
            if metrics.converged:
                robustness = metrics.efficiency_score * (1 - metrics.oscillation_index)
            else:
                robustness = 0.0
            stability['robustness_score'][method] = robustness

            # Oscillation and stagnation severity
            stability['oscillation_severity'][method] = metrics.oscillation_index
            stability['stagnation_severity'][method] = metrics.stagnation_count / max(metrics.iterations, 1)

        return stability

    def _get_memory_usage(self) -> Optional[float]:
        """Get current memory usage in MB."""
        try:
            import psutil
            process = psutil.Process()
            return process.memory_info().rss / 1024 / 1024  # MB
        except ImportError:
            return None

# Utility functions for creating solver functions compatible with the analyzer

def create_traditional_solver_function(solver_class, method_name: str, **kwargs):
    """Create a solver function compatible with ConvergenceAnalyzer."""
    def solver_function(A: np.ndarray, b: np.ndarray) -> Dict[str, Any]:
        solver = solver_class()

        if hasattr(solver, 'benchmark_all_methods'):
            results = solver.benchmark_all_methods(A, b)
            if method_name in results:
                return results[method_name]

        # Fallback for individual methods
        if hasattr(solver, method_name.replace('numpy_', '').replace('scipy_', '')):
            method = getattr(solver, method_name.replace('numpy_', '').replace('scipy_', ''))
            return method(A, b, **kwargs)

        return {'success': False, 'error': f'Method {method_name} not found'}

    return solver_function

def create_sublinear_solver_function(solver_class, method_name: str, **kwargs):
    """Create a sublinear solver function compatible with ConvergenceAnalyzer."""
    def solver_function(A: np.ndarray, b: np.ndarray) -> Dict[str, Any]:
        solver = solver_class()

        if method_name == 'neumann':
            return solver.solve_neumann(A, b, **kwargs)
        elif method_name == 'random_walk':
            return solver.solve_random_walk(A, b, **kwargs)
        elif method_name in ['forward_push', 'backward_push', 'bidirectional']:
            return solver.solve_push_methods(A, b, method_name.replace('_', '-'), **kwargs)

        return {'success': False, 'error': f'Unknown sublinear method: {method_name}'}

    return solver_function

if __name__ == "__main__":
    # Example usage and testing
    from matrix_generators import ADDMatrixGenerator
    from traditional_solvers import TraditionalSolvers
    from sublinear_solvers import SublinearSolvers

    print("Testing Convergence Analysis Framework")
    print("=" * 50)

    # Create test matrix
    generator = ADDMatrixGenerator()
    A, props = generator.tridiagonal_add(100, dominance_factor=2.0)
    b = np.ones(100)

    # Initialize analyzer
    analyzer = ConvergenceAnalyzer(tolerance=1e-10)

    # Create solver functions
    traditional = TraditionalSolvers()
    sublinear = SublinearSolvers()

    solvers = {
        'NumPy Solve': create_traditional_solver_function(TraditionalSolvers, 'numpy_solve'),
        'SciPy GMRES': create_traditional_solver_function(TraditionalSolvers, 'scipy_iter_gmres'),
        'Sublinear Neumann': create_sublinear_solver_function(SublinearSolvers, 'neumann'),
        'Sublinear Random Walk': create_sublinear_solver_function(SublinearSolvers, 'random_walk'),
        'Sublinear Bidirectional': create_sublinear_solver_function(SublinearSolvers, 'bidirectional')
    }

    # Run comparison
    print("Running convergence comparison...")
    results = analyzer.compare_multiple_solvers(solvers, A, b)

    print(f"\nBest performing method: {results.best_method}")
    print("\nSpeedup factors:")
    for method, speedup in results.speedup_factors.items():
        print(f"  {method}: {speedup:.2f}x")

    print("\nConvergence status:")
    for metrics in results.convergence_metrics:
        status = "✓" if metrics.converged else "✗"
        print(f"  {status} {metrics.method_name}: {metrics.iterations} iter, "
              f"{metrics.final_residual:.2e} residual, {metrics.solve_time:.3f}s")

    # Generate plots and report
    plot_files = analyzer.generate_convergence_plots(results, "test_plots")
    report_file = analyzer.export_detailed_report(results, "test_convergence_report.json")

    print(f"\nGenerated {len(plot_files)} plots and report: {report_file}")