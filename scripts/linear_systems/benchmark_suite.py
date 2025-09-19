"""
Comprehensive Benchmark Suite for Linear System Solvers
Compares traditional, iterative, and sublinear methods across various matrix types.
"""

import numpy as np
import time
import json
import pandas as pd
import warnings
from typing import Dict, Any, List, Optional, Callable
from pathlib import Path
import traceback

from traditional_solvers import TraditionalSolvers, analyze_matrix_properties
from iterative_solvers import IterativeSolvers
from sublinear_solvers import SublinearSolvers
from matrix_generators import MatrixGenerators, generate_random_rhs, matrix_properties_summary

class LinearSystemBenchmark:
    """
    Comprehensive benchmark suite for comparing linear system solvers.
    """

    def __init__(self, output_dir: str = "benchmark_results", mcp_client=None, random_seed: int = 42):
        """
        Initialize benchmark suite.

        Args:
            output_dir: Directory to save results
            mcp_client: MCP client for sublinear solver access
            random_seed: Random seed for reproducibility
        """
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True)

        # Initialize solver instances
        self.traditional = TraditionalSolvers()
        self.iterative = IterativeSolvers()
        self.sublinear = SublinearSolvers(mcp_client)
        self.matrix_gen = MatrixGenerators(random_seed)

        # Benchmark configuration
        self.tolerance = 1e-6
        self.max_iterations = 1000
        self.random_seed = random_seed

        # Results storage
        self.results = []
        self.timing_data = []
        self.convergence_data = []

    def run_single_benchmark(self, matrix_name: str, A, b: np.ndarray,
                           solvers: List[str] = None) -> Dict[str, Any]:
        """
        Run benchmark on a single matrix-vector pair.

        Args:
            matrix_name: Descriptive name for the matrix
            A: Coefficient matrix
            b: Right-hand side vector
            solvers: List of solver categories to test

        Returns:
            Dictionary with all solver results
        """
        if solvers is None:
            solvers = ["traditional", "iterative", "sublinear"]

        print(f"\nBenchmarking: {matrix_name} (size: {A.shape[0]})")

        # Analyze matrix properties
        matrix_props = matrix_properties_summary(A)
        matrix_analysis = analyze_matrix_properties(A)

        benchmark_result = {
            "matrix_name": matrix_name,
            "matrix_properties": matrix_props,
            "matrix_analysis": matrix_analysis,
            "results": {},
            "timestamp": time.time(),
            "tolerance": self.tolerance,
            "max_iterations": self.max_iterations
        }

        # Traditional direct/sparse methods
        if "traditional" in solvers:
            print("  Running traditional solvers...")
            try:
                traditional_results = self.traditional.benchmark_all_methods(A, b)
                benchmark_result["results"]["traditional"] = traditional_results
            except Exception as e:
                print(f"    Traditional solvers failed: {e}")
                benchmark_result["results"]["traditional"] = {"error": str(e)}

        # Iterative methods
        if "iterative" in solvers:
            print("  Running iterative solvers...")
            try:
                iterative_results = self.iterative.benchmark_all_iterative(
                    A, b, max_iter=self.max_iterations, tol=self.tolerance
                )
                benchmark_result["results"]["iterative"] = iterative_results
            except Exception as e:
                print(f"    Iterative solvers failed: {e}")
                benchmark_result["results"]["iterative"] = {"error": str(e)}

        # Sublinear methods
        if "sublinear" in solvers:
            print("  Running sublinear solvers...")
            try:
                sublinear_results = self.sublinear.benchmark_all_sublinear(
                    A, b, epsilon=self.tolerance, max_iterations=self.max_iterations
                )
                benchmark_result["results"]["sublinear"] = sublinear_results
            except Exception as e:
                print(f"    Sublinear solvers failed: {e}")
                benchmark_result["results"]["sublinear"] = {"error": str(e)}

        return benchmark_result

    def run_scaling_analysis(self, matrix_type: str = "dd_asymmetric",
                           sizes: List[int] = None,
                           solvers: List[str] = None) -> Dict[str, Any]:
        """
        Analyze how solvers scale with matrix size.

        Args:
            matrix_type: Type of matrix to generate
            sizes: List of matrix sizes to test
            solvers: List of solver categories to test

        Returns:
            Scaling analysis results
        """
        if sizes is None:
            sizes = [50, 100, 200, 500, 1000, 2000]

        if solvers is None:
            solvers = ["traditional", "iterative", "sublinear"]

        print(f"\n=== Scaling Analysis: {matrix_type} ===")

        scaling_results = {
            "matrix_type": matrix_type,
            "sizes": sizes,
            "solvers": solvers,
            "data": []
        }

        for size in sizes:
            print(f"\nTesting size: {size}")

            # Generate matrix
            if matrix_type == "dd_asymmetric":
                A = self.matrix_gen.random_asymmetric_add(size, asymmetry_level=0.7)
            elif matrix_type == "dd_symmetric":
                A = self.matrix_gen.diagonally_dominant_dense(size, symmetric=True)
            elif matrix_type == "tridiagonal":
                A = self.matrix_gen.tridiagonal_matrix(size)
            elif matrix_type == "sparse_dd":
                A = self.matrix_gen.sparse_diagonally_dominant(size, density=0.05)
            elif matrix_type == "spd":
                A = self.matrix_gen.symmetric_positive_definite(size)
            else:
                raise ValueError(f"Unknown matrix type: {matrix_type}")

            # Generate RHS
            b = generate_random_rhs(size, structure="random")

            # Run benchmark
            try:
                result = self.run_single_benchmark(f"{matrix_type}_{size}", A, b, solvers)
                scaling_results["data"].append({
                    "size": size,
                    "results": result
                })
            except Exception as e:
                print(f"  Failed for size {size}: {e}")
                scaling_results["data"].append({
                    "size": size,
                    "error": str(e)
                })

        return scaling_results

    def run_condition_number_analysis(self, base_size: int = 200,
                                    condition_numbers: List[float] = None) -> Dict[str, Any]:
        """
        Analyze solver performance vs. matrix condition number.

        Args:
            base_size: Matrix size to use
            condition_numbers: List of condition numbers to test

        Returns:
            Condition number analysis results
        """
        if condition_numbers is None:
            condition_numbers = [1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e8, 1e10]

        print(f"\n=== Condition Number Analysis (size={base_size}) ===")

        condition_results = {
            "base_size": base_size,
            "condition_numbers": condition_numbers,
            "data": []
        }

        for cond_num in condition_numbers:
            print(f"\nTesting condition number: {cond_num:.1e}")

            try:
                # Generate well-conditioned matrix first
                A = self.matrix_gen.symmetric_positive_definite(base_size, condition_number=cond_num)
                b = generate_random_rhs(base_size)

                # Verify actual condition number
                actual_cond = np.linalg.cond(A)
                print(f"  Actual condition number: {actual_cond:.2e}")

                # Run benchmark
                result = self.run_single_benchmark(f"spd_cond_{cond_num:.0e}", A, b)
                condition_results["data"].append({
                    "target_condition": cond_num,
                    "actual_condition": actual_cond,
                    "results": result
                })

            except Exception as e:
                print(f"  Failed for condition number {cond_num}: {e}")
                condition_results["data"].append({
                    "target_condition": cond_num,
                    "error": str(e)
                })

        return condition_results

    def run_matrix_type_comparison(self, size: int = 500) -> Dict[str, Any]:
        """
        Compare solver performance across different matrix types.

        Args:
            size: Matrix size to use for all tests

        Returns:
            Matrix type comparison results
        """
        print(f"\n=== Matrix Type Comparison (size={size}) ===")

        matrix_types = {
            "dd_symmetric": "Symmetric diagonally dominant",
            "dd_asymmetric": "Asymmetric diagonally dominant",
            "tridiagonal": "Tridiagonal (PDE-like)",
            "banded": "Banded matrix",
            "spd_well": "Well-conditioned SPD",
            "spd_ill": "Ill-conditioned SPD",
            "sparse_dd": "Sparse diagonally dominant"
        }

        comparison_results = {
            "size": size,
            "matrix_types": matrix_types,
            "data": {}
        }

        for matrix_key, description in matrix_types.items():
            print(f"\nTesting: {description}")

            try:
                # Generate matrix
                if matrix_key == "dd_symmetric":
                    A = self.matrix_gen.diagonally_dominant_dense(size, symmetric=True)
                elif matrix_key == "dd_asymmetric":
                    A = self.matrix_gen.random_asymmetric_add(size)
                elif matrix_key == "tridiagonal":
                    A = self.matrix_gen.tridiagonal_matrix(size)
                elif matrix_key == "banded":
                    A = self.matrix_gen.banded_matrix(size, bandwidth=size//20)
                elif matrix_key == "spd_well":
                    A = self.matrix_gen.symmetric_positive_definite(size, condition_number=100)
                elif matrix_key == "spd_ill":
                    A = self.matrix_gen.symmetric_positive_definite(size, condition_number=1e8)
                elif matrix_key == "sparse_dd":
                    A = self.matrix_gen.sparse_diagonally_dominant(size, density=0.05)

                b = generate_random_rhs(size)

                # Run benchmark
                result = self.run_single_benchmark(f"{matrix_key}_{size}", A, b)
                comparison_results["data"][matrix_key] = result

            except Exception as e:
                print(f"  Failed for {description}: {e}")
                comparison_results["data"][matrix_key] = {"error": str(e)}

        return comparison_results

    def run_comprehensive_benchmark(self, quick_mode: bool = False) -> Dict[str, Any]:
        """
        Run comprehensive benchmark suite.

        Args:
            quick_mode: If True, run smaller/faster tests

        Returns:
            Complete benchmark results
        """
        print("=" * 60)
        print("COMPREHENSIVE LINEAR SYSTEM SOLVER BENCHMARK")
        print("=" * 60)

        start_time = time.time()

        comprehensive_results = {
            "benchmark_info": {
                "timestamp": start_time,
                "tolerance": self.tolerance,
                "max_iterations": self.max_iterations,
                "random_seed": self.random_seed,
                "quick_mode": quick_mode
            },
            "scaling_analysis": {},
            "condition_analysis": {},
            "matrix_comparison": {},
            "summary_statistics": {}
        }

        # 1. Scaling analysis
        if quick_mode:
            sizes = [50, 100, 200, 500]
            matrix_types = ["dd_asymmetric", "tridiagonal"]
        else:
            sizes = [50, 100, 200, 500, 1000, 2000]
            matrix_types = ["dd_asymmetric", "dd_symmetric", "tridiagonal", "sparse_dd"]

        for matrix_type in matrix_types:
            try:
                scaling_result = self.run_scaling_analysis(matrix_type, sizes)
                comprehensive_results["scaling_analysis"][matrix_type] = scaling_result
            except Exception as e:
                print(f"Scaling analysis failed for {matrix_type}: {e}")

        # 2. Condition number analysis
        if not quick_mode:
            try:
                condition_result = self.run_condition_number_analysis()
                comprehensive_results["condition_analysis"] = condition_result
            except Exception as e:
                print(f"Condition number analysis failed: {e}")

        # 3. Matrix type comparison
        comparison_size = 200 if quick_mode else 500
        try:
            comparison_result = self.run_matrix_type_comparison(comparison_size)
            comprehensive_results["matrix_comparison"] = comparison_result
        except Exception as e:
            print(f"Matrix type comparison failed: {e}")

        # 4. Generate summary statistics
        comprehensive_results["summary_statistics"] = self._generate_summary_statistics(comprehensive_results)

        total_time = time.time() - start_time
        comprehensive_results["benchmark_info"]["total_time"] = total_time

        print(f"\n" + "=" * 60)
        print(f"BENCHMARK COMPLETED in {total_time:.2f} seconds")
        print("=" * 60)

        return comprehensive_results

    def save_results(self, results: Dict[str, Any], filename: str = None) -> str:
        """
        Save benchmark results to JSON file.

        Args:
            results: Results dictionary to save
            filename: Output filename (auto-generated if None)

        Returns:
            Path to saved file
        """
        if filename is None:
            timestamp = time.strftime("%Y%m%d_%H%M%S")
            filename = f"linear_solver_benchmark_{timestamp}.json"

        filepath = self.output_dir / filename

        # Convert numpy arrays to lists for JSON serialization
        serializable_results = self._make_json_serializable(results)

        with open(filepath, 'w') as f:
            json.dump(serializable_results, f, indent=2, default=str)

        print(f"\nResults saved to: {filepath}")
        return str(filepath)

    def generate_performance_report(self, results: Dict[str, Any]) -> str:
        """
        Generate human-readable performance report.

        Args:
            results: Benchmark results

        Returns:
            Formatted report string
        """
        report = []
        report.append("=" * 80)
        report.append("LINEAR SYSTEM SOLVER PERFORMANCE REPORT")
        report.append("=" * 80)

        # Summary
        if "benchmark_info" in results:
            info = results["benchmark_info"]
            report.append(f"Benchmark completed: {time.ctime(info['timestamp'])}")
            report.append(f"Total time: {info.get('total_time', 'unknown'):.2f} seconds")
            report.append(f"Tolerance: {info['tolerance']}")
            report.append(f"Max iterations: {info['max_iterations']}")
            report.append("")

        # Matrix type comparison summary
        if "matrix_comparison" in results and "data" in results["matrix_comparison"]:
            report.append("MATRIX TYPE PERFORMANCE SUMMARY")
            report.append("-" * 40)

            for matrix_type, data in results["matrix_comparison"]["data"].items():
                if "error" in data:
                    continue

                report.append(f"\n{matrix_type.upper()}:")

                # Extract best performers
                best_times = {}
                for solver_category, solver_results in data["results"].items():
                    if isinstance(solver_results, dict) and "error" not in solver_results:
                        for method_name, method_result in solver_results.items():
                            if isinstance(method_result, dict) and method_result.get("success", False):
                                time_taken = method_result.get("time", float('inf'))
                                best_times[f"{solver_category}_{method_name}"] = time_taken

                # Sort by time
                sorted_methods = sorted(best_times.items(), key=lambda x: x[1])

                for i, (method, time_val) in enumerate(sorted_methods[:5]):  # Top 5
                    report.append(f"  {i+1}. {method}: {time_val:.4f}s")

        # Generate summary statistics
        if "summary_statistics" in results:
            stats = results["summary_statistics"]
            report.append("\n\nSUMMARY STATISTICS")
            report.append("-" * 20)
            for key, value in stats.items():
                report.append(f"{key}: {value}")

        report_text = "\n".join(report)

        # Save report
        timestamp = time.strftime("%Y%m%d_%H%M%S")
        report_file = self.output_dir / f"performance_report_{timestamp}.txt"
        with open(report_file, 'w') as f:
            f.write(report_text)

        print(f"Performance report saved to: {report_file}")
        return report_text

    def _generate_summary_statistics(self, results: Dict[str, Any]) -> Dict[str, Any]:
        """Generate summary statistics from benchmark results."""
        stats = {}

        # Count total tests
        total_tests = 0
        successful_tests = 0

        # Collect timing data
        sublinear_times = []
        traditional_times = []
        iterative_times = []

        def extract_times_recursive(data, path=""):
            nonlocal total_tests, successful_tests, sublinear_times, traditional_times, iterative_times

            if isinstance(data, dict):
                if "time" in data and "success" in data:
                    total_tests += 1
                    if data["success"]:
                        successful_tests += 1
                        time_val = data["time"]

                        if "sublinear" in path.lower():
                            sublinear_times.append(time_val)
                        elif "traditional" in path.lower() or "numpy" in path.lower() or "scipy" in path.lower():
                            traditional_times.append(time_val)
                        elif "iterative" in path.lower() or "jacobi" in path.lower() or "gauss" in path.lower():
                            iterative_times.append(time_val)

                for key, value in data.items():
                    extract_times_recursive(value, f"{path}_{key}" if path else key)
            elif isinstance(data, list):
                for i, item in enumerate(data):
                    extract_times_recursive(item, f"{path}_{i}")

        extract_times_recursive(results)

        stats["total_tests"] = total_tests
        stats["successful_tests"] = successful_tests
        stats["success_rate"] = successful_tests / max(1, total_tests)

        if sublinear_times:
            stats["sublinear_avg_time"] = np.mean(sublinear_times)
            stats["sublinear_median_time"] = np.median(sublinear_times)

        if traditional_times:
            stats["traditional_avg_time"] = np.mean(traditional_times)
            stats["traditional_median_time"] = np.median(traditional_times)

        if iterative_times:
            stats["iterative_avg_time"] = np.mean(iterative_times)
            stats["iterative_median_time"] = np.median(iterative_times)

        # Calculate relative performance
        if sublinear_times and traditional_times:
            speedup = np.mean(traditional_times) / np.mean(sublinear_times)
            stats["sublinear_speedup_vs_traditional"] = speedup

        return stats

    def _make_json_serializable(self, obj):
        """Convert numpy arrays and other non-serializable objects to JSON-compatible format."""
        if isinstance(obj, np.ndarray):
            return obj.tolist()
        elif isinstance(obj, np.integer):
            return int(obj)
        elif isinstance(obj, np.floating):
            return float(obj)
        elif isinstance(obj, np.complex):
            return {"real": float(obj.real), "imag": float(obj.imag)}
        elif isinstance(obj, dict):
            return {key: self._make_json_serializable(value) for key, value in obj.items()}
        elif isinstance(obj, list):
            return [self._make_json_serializable(item) for item in obj]
        else:
            return obj

def main():
    """Main function for running benchmarks from command line."""
    import argparse

    parser = argparse.ArgumentParser(description="Linear System Solver Benchmark Suite")
    parser.add_argument("--quick", action="store_true", help="Run quick benchmark (smaller tests)")
    parser.add_argument("--output-dir", default="benchmark_results", help="Output directory")
    parser.add_argument("--size", type=int, help="Single matrix size to test")
    parser.add_argument("--matrix-type", help="Single matrix type to test")
    parser.add_argument("--seed", type=int, default=42, help="Random seed")

    args = parser.parse_args()

    # Initialize benchmark
    benchmark = LinearSystemBenchmark(output_dir=args.output_dir, random_seed=args.seed)

    if args.size and args.matrix_type:
        # Single test
        print(f"Running single test: {args.matrix_type}, size={args.size}")

        # Generate matrix
        if args.matrix_type == "dd_asymmetric":
            A = benchmark.matrix_gen.random_asymmetric_add(args.size)
        elif args.matrix_type == "tridiagonal":
            A = benchmark.matrix_gen.tridiagonal_matrix(args.size)
        elif args.matrix_type == "spd":
            A = benchmark.matrix_gen.symmetric_positive_definite(args.size)
        else:
            raise ValueError(f"Unknown matrix type: {args.matrix_type}")

        b = generate_random_rhs(args.size)

        result = benchmark.run_single_benchmark(f"{args.matrix_type}_{args.size}", A, b)
        filename = benchmark.save_results({"single_test": result})

    else:
        # Comprehensive benchmark
        results = benchmark.run_comprehensive_benchmark(quick_mode=args.quick)
        filename = benchmark.save_results(results)
        report = benchmark.generate_performance_report(results)
        print("\n" + report)

if __name__ == "__main__":
    main()