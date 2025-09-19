#!/usr/bin/env python3
"""
Mathematical Accuracy Validation for Sublinear-Time Solver
Comprehensive correctness verification across all domains
"""

import numpy as np
import json
import sys
import time
from pathlib import Path
from typing import Dict, List, Tuple, Optional, Any, Callable
from dataclasses import dataclass, asdict
import logging
from scipy.linalg import norm
import warnings

# Add project root to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

@dataclass
class AccuracyResult:
    """Results from accuracy validation"""
    test_name: str
    domain: str
    algorithm: str
    problem_size: int
    ground_truth_available: bool
    relative_error: Optional[float] = None
    absolute_error: Optional[float] = None
    residual_norm: Optional[float] = None
    convergence_achieved: bool = False
    numerical_rank: Optional[int] = None
    condition_number: Optional[float] = None
    spectral_properties: Dict[str, float] = None
    success: bool = False
    error_message: str = ""
    execution_time: float = 0.0

    def __post_init__(self):
        if self.spectral_properties is None:
            self.spectral_properties = {}

class AccuracyValidator:
    """Comprehensive mathematical accuracy validation"""

    def __init__(self, output_dir: str = "accuracy_validation"):
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True)
        self.logger = self._setup_logging()
        self.results: List[AccuracyResult] = []

        # Numerical tolerances
        self.tolerances = {
            "tight": 1e-12,
            "standard": 1e-8,
            "relaxed": 1e-4,
            "very_relaxed": 1e-2
        }

    def _setup_logging(self) -> logging.Logger:
        logger = logging.getLogger("AccuracyValidator")
        logger.setLevel(logging.INFO)

        handler = logging.StreamHandler()
        formatter = logging.Formatter('%(asctime)s - %(levelname)s - %(message)s')
        handler.setFormatter(formatter)
        logger.addHandler(handler)

        return logger

    def validate_linear_systems(self) -> List[AccuracyResult]:
        """Validate linear system solving accuracy"""
        self.logger.info("Validating linear systems accuracy...")

        test_cases = [
            # Well-conditioned systems
            {"size": 100, "condition": "well", "method": "neumann"},
            {"size": 500, "condition": "well", "method": "random-walk"},
            {"size": 1000, "condition": "well", "method": "forward-push"},

            # Moderately conditioned systems
            {"size": 100, "condition": "moderate", "method": "neumann"},
            {"size": 500, "condition": "moderate", "method": "random-walk"},

            # Ill-conditioned systems (challenging)
            {"size": 50, "condition": "ill", "method": "neumann"},
            {"size": 100, "condition": "ill", "method": "random-walk"},

            # Special structure matrices
            {"size": 200, "condition": "symmetric", "method": "neumann"},
            {"size": 300, "condition": "sparse", "method": "forward-push"},
        ]

        results = []
        for case in test_cases:
            try:
                result = self._validate_single_linear_system(
                    case["size"], case["condition"], case["method"]
                )
                results.append(result)
            except Exception as e:
                self.logger.error(f"Linear system validation failed: {str(e)}")

        return results

    def _validate_single_linear_system(self, size: int, condition_type: str,
                                     method: str) -> AccuracyResult:
        """Validate a single linear system configuration"""
        start_time = time.time()

        try:
            # Generate test matrix based on condition type
            A, b, true_solution = self._generate_test_linear_system(size, condition_type)

            # Calculate matrix properties
            cond_num = np.linalg.cond(A)
            rank = np.linalg.matrix_rank(A)

            # Solve using the specified method (simulated for now)
            computed_solution = self._solve_linear_system_mcp(A, b, method)

            execution_time = time.time() - start_time

            # Calculate accuracy metrics
            if true_solution is not None:
                absolute_error = norm(computed_solution - true_solution)
                relative_error = absolute_error / norm(true_solution) if norm(true_solution) > 0 else absolute_error
            else:
                absolute_error = None
                relative_error = None

            # Calculate residual
            residual = b - A @ computed_solution
            residual_norm = norm(residual) / norm(b) if norm(b) > 0 else norm(residual)

            # Determine if convergence was achieved
            convergence = residual_norm < self.tolerances["standard"]

            # Spectral properties
            eigenvalues = np.linalg.eigvals(A)
            spectral_props = {
                "min_eigenvalue": float(np.min(eigenvalues.real)),
                "max_eigenvalue": float(np.max(eigenvalues.real)),
                "spectral_radius": float(np.max(np.abs(eigenvalues))),
                "eigenvalue_spread": float(np.max(eigenvalues.real) - np.min(eigenvalues.real))
            }

            return AccuracyResult(
                test_name=f"linear_system_{condition_type}_{size}x{size}",
                domain="linear_systems",
                algorithm=method,
                problem_size=size * size,
                ground_truth_available=true_solution is not None,
                relative_error=relative_error,
                absolute_error=absolute_error,
                residual_norm=residual_norm,
                convergence_achieved=convergence,
                numerical_rank=rank,
                condition_number=cond_num,
                spectral_properties=spectral_props,
                success=True,
                execution_time=execution_time
            )

        except Exception as e:
            execution_time = time.time() - start_time
            return AccuracyResult(
                test_name=f"linear_system_{condition_type}_{size}x{size}",
                domain="linear_systems",
                algorithm=method,
                problem_size=size * size,
                ground_truth_available=False,
                success=False,
                error_message=str(e),
                execution_time=execution_time
            )

    def _generate_test_linear_system(self, size: int, condition_type: str) -> Tuple[np.ndarray, np.ndarray, Optional[np.ndarray]]:
        """Generate test linear systems with known properties"""
        np.random.seed(42)  # Reproducible

        if condition_type == "well":
            # Well-conditioned diagonally dominant matrix
            A = np.random.randn(size, size)
            A = A + A.T  # Symmetric
            A = A + (size + 10) * np.eye(size)  # Strong diagonal dominance
            true_solution = np.random.randn(size)
            b = A @ true_solution

        elif condition_type == "moderate":
            # Moderately conditioned matrix
            A = np.random.randn(size, size)
            A = A + A.T
            A = A + 5 * np.eye(size)  # Moderate diagonal dominance
            true_solution = np.random.randn(size)
            b = A @ true_solution

        elif condition_type == "ill":
            # Ill-conditioned matrix (high condition number)
            U, _, Vt = np.linalg.svd(np.random.randn(size, size))
            singular_values = np.logspace(-8, 0, size)  # Wide range of singular values
            A = U @ np.diag(singular_values) @ Vt
            A = A + A.T  # Make symmetric
            A = A + 0.1 * np.eye(size)  # Ensure positive definiteness
            true_solution = np.random.randn(size)
            b = A @ true_solution

        elif condition_type == "symmetric":
            # Symmetric positive definite matrix
            G = np.random.randn(size, size)
            A = G @ G.T + np.eye(size)  # Always positive definite
            true_solution = np.random.randn(size)
            b = A @ true_solution

        elif condition_type == "sparse":
            # Sparse matrix (tridiagonal structure)
            A = np.zeros((size, size))
            np.fill_diagonal(A, 4)  # Main diagonal
            np.fill_diagonal(A[1:], -1)  # Super diagonal
            np.fill_diagonal(A[:, 1:], -1)  # Sub diagonal
            true_solution = np.random.randn(size)
            b = A @ true_solution

        else:
            raise ValueError(f"Unknown condition type: {condition_type}")

        return A, b, true_solution

    def _solve_linear_system_mcp(self, A: np.ndarray, b: np.ndarray, method: str) -> np.ndarray:
        """Solve linear system using MCP solver (simulated)"""
        # TODO: Replace with actual MCP tool call
        # For now, use different numerical methods to simulate different MCP methods

        if method == "neumann":
            # Neumann series approximation
            # x = (I - (I - A))^{-1} b ≈ sum_{k=0}^n (I-A)^k b
            M = np.eye(len(A)) - A / np.linalg.norm(A, 'fro')
            x = b.copy()
            for i in range(10):  # 10 iterations
                x = x + M @ x
            return x

        elif method == "random-walk":
            # Simulate random walk solution
            # Use conjugate gradient as approximation
            try:
                from scipy.sparse.linalg import cg
                solution, info = cg(A, b, maxiter=100)
                return solution
            except ImportError:
                # Fallback to direct solve
                return np.linalg.solve(A, b)

        elif method == "forward-push":
            # Simulate forward-push algorithm
            # Use iterative refinement
            x = np.linalg.solve(A, b)  # Initial guess
            for i in range(5):  # Refinement iterations
                residual = b - A @ x
                correction = np.linalg.solve(A, residual)
                x = x + correction
            return x

        else:
            # Default to direct solve
            return np.linalg.solve(A, b)

    def validate_pagerank(self) -> List[AccuracyResult]:
        """Validate PageRank accuracy"""
        self.logger.info("Validating PageRank accuracy...")

        test_cases = [
            # Simple graphs with known PageRank
            {"nodes": 10, "graph_type": "complete", "damping": 0.85},
            {"nodes": 20, "graph_type": "star", "damping": 0.85},
            {"nodes": 50, "graph_type": "chain", "damping": 0.85},
            {"nodes": 100, "graph_type": "random", "damping": 0.85},

            # Different damping factors
            {"nodes": 50, "graph_type": "random", "damping": 0.5},
            {"nodes": 50, "graph_type": "random", "damping": 0.95},

            # Larger graphs
            {"nodes": 500, "graph_type": "scale_free", "damping": 0.85},
            {"nodes": 1000, "graph_type": "small_world", "damping": 0.85},
        ]

        results = []
        for case in test_cases:
            try:
                result = self._validate_single_pagerank(
                    case["nodes"], case["graph_type"], case["damping"]
                )
                results.append(result)
            except Exception as e:
                self.logger.error(f"PageRank validation failed: {str(e)}")

        return results

    def _validate_single_pagerank(self, num_nodes: int, graph_type: str,
                                damping: float) -> AccuracyResult:
        """Validate a single PageRank configuration"""
        start_time = time.time()

        try:
            # Generate test graph
            adjacency, true_pagerank = self._generate_test_graph(num_nodes, graph_type, damping)

            # Calculate graph properties
            spectral_props = self._analyze_graph_spectral_properties(adjacency)

            # Compute PageRank using MCP solver (simulated)
            computed_pagerank = self._solve_pagerank_mcp(adjacency, damping)

            execution_time = time.time() - start_time

            # Calculate accuracy metrics
            if true_pagerank is not None:
                absolute_error = norm(computed_pagerank - true_pagerank)
                relative_error = absolute_error / norm(true_pagerank)
            else:
                # Use power iteration as reference
                reference_pagerank = self._power_iteration_pagerank(adjacency, damping)
                absolute_error = norm(computed_pagerank - reference_pagerank)
                relative_error = absolute_error / norm(reference_pagerank)

            # Check convergence (PageRank vector should sum to 1)
            sum_error = abs(np.sum(computed_pagerank) - 1.0)
            convergence = sum_error < self.tolerances["standard"]

            return AccuracyResult(
                test_name=f"pagerank_{graph_type}_{num_nodes}_nodes",
                domain="pagerank",
                algorithm="pagerank",
                problem_size=num_nodes,
                ground_truth_available=true_pagerank is not None,
                relative_error=relative_error,
                absolute_error=absolute_error,
                residual_norm=sum_error,  # Use sum error as residual
                convergence_achieved=convergence,
                spectral_properties=spectral_props,
                success=True,
                execution_time=execution_time
            )

        except Exception as e:
            execution_time = time.time() - start_time
            return AccuracyResult(
                test_name=f"pagerank_{graph_type}_{num_nodes}_nodes",
                domain="pagerank",
                algorithm="pagerank",
                problem_size=num_nodes,
                ground_truth_available=False,
                success=False,
                error_message=str(e),
                execution_time=execution_time
            )

    def _generate_test_graph(self, num_nodes: int, graph_type: str,
                           damping: float) -> Tuple[np.ndarray, Optional[np.ndarray]]:
        """Generate test graphs with known PageRank values"""
        np.random.seed(42)

        if graph_type == "complete":
            # Complete graph - all nodes have equal PageRank
            adjacency = np.ones((num_nodes, num_nodes)) - np.eye(num_nodes)
            adjacency = adjacency / (num_nodes - 1)  # Normalize
            true_pagerank = np.ones(num_nodes) / num_nodes

        elif graph_type == "star":
            # Star graph - center has high PageRank
            adjacency = np.zeros((num_nodes, num_nodes))
            # All nodes point to center (node 0)
            adjacency[1:, 0] = 1
            # Center points to all others
            adjacency[0, 1:] = 1 / (num_nodes - 1)

            # Calculate true PageRank analytically for star graph
            center_pr = (1 - damping + damping * (num_nodes - 1)) / num_nodes
            leaf_pr = (1 - damping + damping * center_pr / (num_nodes - 1)) / num_nodes
            true_pagerank = np.full(num_nodes, leaf_pr)
            true_pagerank[0] = center_pr
            # Normalize
            true_pagerank = true_pagerank / np.sum(true_pagerank)

        elif graph_type == "chain":
            # Chain graph - ends have different PageRank than middle
            adjacency = np.zeros((num_nodes, num_nodes))
            for i in range(num_nodes - 1):
                adjacency[i, i + 1] = 1  # Forward edges
                adjacency[i + 1, i] = 1  # Backward edges
            # Normalize rows
            row_sums = adjacency.sum(axis=1)
            adjacency = adjacency / row_sums[:, np.newaxis]
            true_pagerank = None  # No analytical solution

        elif graph_type == "random":
            # Random graph
            adjacency = np.random.rand(num_nodes, num_nodes)
            adjacency = (adjacency > 0.8).astype(float)  # Sparse
            np.fill_diagonal(adjacency, 0)  # No self-loops
            # Normalize
            row_sums = adjacency.sum(axis=1)
            row_sums[row_sums == 0] = 1  # Avoid division by zero
            adjacency = adjacency / row_sums[:, np.newaxis]
            true_pagerank = None

        elif graph_type == "scale_free":
            # Scale-free network (preferential attachment)
            adjacency = np.zeros((num_nodes, num_nodes))
            for i in range(2, num_nodes):
                # Connect to existing nodes with probability proportional to degree
                degrees = adjacency.sum(axis=0) + 1  # Add 1 to avoid zero degrees
                probs = degrees / degrees.sum()
                targets = np.random.choice(i, size=min(2, i), replace=False, p=probs[:i])
                adjacency[i, targets] = 1
                adjacency[targets, i] = 1

            # Normalize
            row_sums = adjacency.sum(axis=1)
            row_sums[row_sums == 0] = 1
            adjacency = adjacency / row_sums[:, np.newaxis]
            true_pagerank = None

        elif graph_type == "small_world":
            # Small world network (Watts-Strogatz model)
            adjacency = np.zeros((num_nodes, num_nodes))
            k = 4  # Each node connects to k nearest neighbors

            # Ring lattice
            for i in range(num_nodes):
                for j in range(1, k//2 + 1):
                    adjacency[i, (i + j) % num_nodes] = 1
                    adjacency[i, (i - j) % num_nodes] = 1

            # Random rewiring with probability 0.1
            for i in range(num_nodes):
                for j in range(num_nodes):
                    if adjacency[i, j] == 1 and np.random.random() < 0.1:
                        adjacency[i, j] = 0
                        new_target = np.random.randint(0, num_nodes)
                        adjacency[i, new_target] = 1

            # Normalize
            row_sums = adjacency.sum(axis=1)
            row_sums[row_sums == 0] = 1
            adjacency = adjacency / row_sums[:, np.newaxis]
            true_pagerank = None

        else:
            raise ValueError(f"Unknown graph type: {graph_type}")

        return adjacency, true_pagerank

    def _analyze_graph_spectral_properties(self, adjacency: np.ndarray) -> Dict[str, float]:
        """Analyze spectral properties of graph adjacency matrix"""
        try:
            eigenvalues = np.linalg.eigvals(adjacency)

            return {
                "spectral_radius": float(np.max(np.abs(eigenvalues))),
                "second_largest_eigenvalue": float(np.sort(np.abs(eigenvalues))[-2]) if len(eigenvalues) > 1 else 0.0,
                "spectral_gap": float(np.sort(np.abs(eigenvalues))[-1] - np.sort(np.abs(eigenvalues))[-2]) if len(eigenvalues) > 1 else 0.0,
                "num_connected_components": self._estimate_connected_components(adjacency)
            }
        except Exception:
            return {"error": "spectral_analysis_failed"}

    def _estimate_connected_components(self, adjacency: np.ndarray) -> float:
        """Estimate number of connected components from spectrum"""
        try:
            # Number of zero eigenvalues approximates number of components
            eigenvalues = np.linalg.eigvals(adjacency + adjacency.T)  # Symmetrize
            zero_eigenvalues = np.sum(np.abs(eigenvalues) < 1e-10)
            return float(zero_eigenvalues)
        except Exception:
            return 1.0

    def _solve_pagerank_mcp(self, adjacency: np.ndarray, damping: float) -> np.ndarray:
        """Solve PageRank using MCP solver (simulated)"""
        # TODO: Replace with actual MCP tool call
        # For now, use power iteration
        return self._power_iteration_pagerank(adjacency, damping)

    def _power_iteration_pagerank(self, adjacency: np.ndarray, damping: float,
                                max_iter: int = 100, tol: float = 1e-8) -> np.ndarray:
        """Reference PageRank implementation using power iteration"""
        num_nodes = adjacency.shape[0]
        pagerank = np.ones(num_nodes) / num_nodes

        for iteration in range(max_iter):
            prev_pagerank = pagerank.copy()

            # PageRank update
            pagerank = (1 - damping) / num_nodes + damping * adjacency.T @ pagerank

            # Check convergence
            if norm(pagerank - prev_pagerank) < tol:
                break

        return pagerank

    def analyze_accuracy_patterns(self) -> Dict[str, Any]:
        """Analyze accuracy patterns across all validation results"""
        self.logger.info("Analyzing accuracy patterns...")

        analysis = {
            "overall_statistics": {},
            "domain_analysis": {},
            "algorithm_comparison": {},
            "error_distribution": {},
            "convergence_analysis": {},
            "recommendations": []
        }

        if not self.results:
            return analysis

        # Overall statistics
        successful_results = [r for r in self.results if r.success]
        analysis["overall_statistics"] = {
            "total_tests": len(self.results),
            "successful_tests": len(successful_results),
            "success_rate": len(successful_results) / len(self.results) * 100
        }

        # Error statistics for successful results with ground truth
        ground_truth_results = [r for r in successful_results if r.ground_truth_available and r.relative_error is not None]
        if ground_truth_results:
            relative_errors = [r.relative_error for r in ground_truth_results]
            analysis["overall_statistics"].update({
                "avg_relative_error": np.mean(relative_errors),
                "median_relative_error": np.median(relative_errors),
                "max_relative_error": np.max(relative_errors),
                "min_relative_error": np.min(relative_errors),
                "std_relative_error": np.std(relative_errors)
            })

        # Domain analysis
        domains = {}
        for result in successful_results:
            if result.domain not in domains:
                domains[result.domain] = []
            domains[result.domain].append(result)

        for domain, domain_results in domains.items():
            domain_ground_truth = [r for r in domain_results if r.ground_truth_available and r.relative_error is not None]

            domain_stats = {
                "total_tests": len(domain_results),
                "convergence_rate": sum(1 for r in domain_results if r.convergence_achieved) / len(domain_results) * 100
            }

            if domain_ground_truth:
                errors = [r.relative_error for r in domain_ground_truth]
                domain_stats.update({
                    "avg_relative_error": np.mean(errors),
                    "max_relative_error": np.max(errors),
                    "error_std": np.std(errors)
                })

            analysis["domain_analysis"][domain] = domain_stats

        # Algorithm comparison
        algorithms = {}
        for result in successful_results:
            if result.algorithm not in algorithms:
                algorithms[result.algorithm] = []
            algorithms[result.algorithm].append(result)

        for algorithm, algo_results in algorithms.items():
            algo_ground_truth = [r for r in algo_results if r.ground_truth_available and r.relative_error is not None]

            algo_stats = {
                "total_tests": len(algo_results),
                "avg_execution_time": np.mean([r.execution_time for r in algo_results]),
                "convergence_rate": sum(1 for r in algo_results if r.convergence_achieved) / len(algo_results) * 100
            }

            if algo_ground_truth:
                errors = [r.relative_error for r in algo_ground_truth]
                algo_stats.update({
                    "avg_relative_error": np.mean(errors),
                    "error_range": f"{np.min(errors):.2e} - {np.max(errors):.2e}"
                })

            analysis["algorithm_comparison"][algorithm] = algo_stats

        # Error distribution analysis
        if ground_truth_results:
            errors = [r.relative_error for r in ground_truth_results]

            # Categorize errors
            excellent = sum(1 for e in errors if e < 1e-10)
            very_good = sum(1 for e in errors if 1e-10 <= e < 1e-8)
            good = sum(1 for e in errors if 1e-8 <= e < 1e-6)
            acceptable = sum(1 for e in errors if 1e-6 <= e < 1e-4)
            poor = sum(1 for e in errors if e >= 1e-4)

            analysis["error_distribution"] = {
                "excellent_lt_1e-10": excellent,
                "very_good_1e-10_to_1e-8": very_good,
                "good_1e-8_to_1e-6": good,
                "acceptable_1e-6_to_1e-4": acceptable,
                "poor_gte_1e-4": poor
            }

        # Convergence analysis
        convergent_results = [r for r in successful_results if r.convergence_achieved]
        analysis["convergence_analysis"] = {
            "total_convergent": len(convergent_results),
            "convergence_rate": len(convergent_results) / len(successful_results) * 100 if successful_results else 0
        }

        # Generate recommendations
        analysis["recommendations"] = self._generate_accuracy_recommendations(analysis)

        return analysis

    def _generate_accuracy_recommendations(self, analysis: Dict[str, Any]) -> List[str]:
        """Generate recommendations based on accuracy analysis"""
        recommendations = []

        overall_stats = analysis.get("overall_statistics", {})

        # Check success rate
        success_rate = overall_stats.get("success_rate", 0)
        if success_rate < 90:
            recommendations.append(f"🚨 LOW SUCCESS RATE: Only {success_rate:.1f}% of tests succeeded - investigate failures")

        # Check convergence rate
        convergence_data = analysis.get("convergence_analysis", {})
        convergence_rate = convergence_data.get("convergence_rate", 0)
        if convergence_rate < 80:
            recommendations.append(f"⚠️ CONVERGENCE ISSUES: Only {convergence_rate:.1f}% of tests converged")

        # Check accuracy
        avg_error = overall_stats.get("avg_relative_error", 0)
        if avg_error:
            if avg_error > 1e-4:
                recommendations.append(f"🚨 HIGH ERROR: Average relative error {avg_error:.2e} exceeds acceptable threshold")
            elif avg_error > 1e-6:
                recommendations.append(f"⚠️ MODERATE ERROR: Average relative error {avg_error:.2e} could be improved")
            else:
                recommendations.append(f"✅ GOOD ACCURACY: Average relative error {avg_error:.2e} is acceptable")

        # Domain-specific recommendations
        domain_analysis = analysis.get("domain_analysis", {})
        for domain, stats in domain_analysis.items():
            domain_error = stats.get("avg_relative_error", 0)
            if domain_error and domain_error > 1e-4:
                recommendations.append(f"⚠️ {domain.upper()}: High error rate - review algorithm implementation")

        # Algorithm comparison
        algo_analysis = analysis.get("algorithm_comparison", {})
        if len(algo_analysis) > 1:
            # Find best and worst algorithms
            algo_errors = {algo: stats.get("avg_relative_error", float('inf'))
                          for algo, stats in algo_analysis.items()}
            best_algo = min(algo_errors, key=algo_errors.get)
            worst_algo = max(algo_errors, key=algo_errors.get)

            if algo_errors[best_algo] < float('inf') and algo_errors[worst_algo] < float('inf'):
                recommendations.append(f"📊 BEST ALGORITHM: {best_algo} (error: {algo_errors[best_algo]:.2e})")
                if algo_errors[worst_algo] / algo_errors[best_algo] > 10:
                    recommendations.append(f"⚠️ WORST ALGORITHM: {worst_algo} needs improvement")

        if not recommendations:
            recommendations.append("✅ No major accuracy issues detected")

        return recommendations

    def generate_accuracy_report(self) -> None:
        """Generate comprehensive accuracy validation report"""
        self.logger.info("Generating accuracy validation report...")

        analysis = self.analyze_accuracy_patterns()

        report_file = self.output_dir / "accuracy_validation_report.md"
        with open(report_file, 'w') as f:
            f.write("# Mathematical Accuracy Validation Report\n\n")
            f.write(f"Generated: {time.strftime('%Y-%m-%d %H:%M:%S')}\n\n")

            # Executive Summary
            f.write("## Executive Summary\n\n")
            overall_stats = analysis.get("overall_statistics", {})
            f.write(f"- **Total validation tests**: {overall_stats.get('total_tests', 0)}\n")
            f.write(f"- **Successful tests**: {overall_stats.get('successful_tests', 0)} ({overall_stats.get('success_rate', 0):.1f}%)\n")

            if 'avg_relative_error' in overall_stats:
                f.write(f"- **Average relative error**: {overall_stats['avg_relative_error']:.2e}\n")
                f.write(f"- **Maximum relative error**: {overall_stats['max_relative_error']:.2e}\n")

            convergence_data = analysis.get("convergence_analysis", {})
            f.write(f"- **Convergence rate**: {convergence_data.get('convergence_rate', 0):.1f}%\n")

            # Key Recommendations
            f.write("\n## Key Recommendations\n\n")
            for rec in analysis.get("recommendations", []):
                f.write(f"- {rec}\n")

            # Domain Analysis
            f.write("\n## Domain Analysis\n\n")
            domain_analysis = analysis.get("domain_analysis", {})
            for domain, stats in domain_analysis.items():
                f.write(f"### {domain.title()}\n")
                f.write(f"- Tests: {stats.get('total_tests', 0)}\n")
                f.write(f"- Convergence rate: {stats.get('convergence_rate', 0):.1f}%\n")
                if 'avg_relative_error' in stats:
                    f.write(f"- Average relative error: {stats['avg_relative_error']:.2e}\n")
                    f.write(f"- Maximum relative error: {stats['max_relative_error']:.2e}\n")
                f.write("\n")

            # Algorithm Comparison
            f.write("## Algorithm Comparison\n\n")
            algo_analysis = analysis.get("algorithm_comparison", {})
            for algorithm, stats in algo_analysis.items():
                f.write(f"### {algorithm}\n")
                f.write(f"- Tests: {stats.get('total_tests', 0)}\n")
                f.write(f"- Average execution time: {stats.get('avg_execution_time', 0):.4f}s\n")
                f.write(f"- Convergence rate: {stats.get('convergence_rate', 0):.1f}%\n")
                if 'avg_relative_error' in stats:
                    f.write(f"- Average relative error: {stats['avg_relative_error']:.2e}\n")
                    f.write(f"- Error range: {stats.get('error_range', 'N/A')}\n")
                f.write("\n")

            # Error Distribution
            f.write("## Error Distribution\n\n")
            error_dist = analysis.get("error_distribution", {})
            if error_dist:
                f.write("| Error Range | Count | Quality |\n")
                f.write("|-------------|-------|----------|\n")
                f.write(f"| < 1e-10 | {error_dist.get('excellent_lt_1e-10', 0)} | Excellent |\n")
                f.write(f"| 1e-10 to 1e-8 | {error_dist.get('very_good_1e-10_to_1e-8', 0)} | Very Good |\n")
                f.write(f"| 1e-8 to 1e-6 | {error_dist.get('good_1e-8_to_1e-6', 0)} | Good |\n")
                f.write(f"| 1e-6 to 1e-4 | {error_dist.get('acceptable_1e-6_to_1e-4', 0)} | Acceptable |\n")
                f.write(f"| ≥ 1e-4 | {error_dist.get('poor_gte_1e-4', 0)} | Poor |\n")

            # Detailed Results
            f.write("\n## Detailed Test Results\n\n")
            for result in self.results:
                f.write(f"### {result.test_name}\n")
                f.write(f"- **Domain**: {result.domain}\n")
                f.write(f"- **Algorithm**: {result.algorithm}\n")
                f.write(f"- **Problem size**: {result.problem_size:,}\n")
                f.write(f"- **Execution time**: {result.execution_time:.4f}s\n")
                f.write(f"- **Success**: {'Yes' if result.success else 'No'}\n")

                if result.success:
                    if result.relative_error is not None:
                        f.write(f"- **Relative error**: {result.relative_error:.2e}\n")
                    if result.residual_norm is not None:
                        f.write(f"- **Residual norm**: {result.residual_norm:.2e}\n")
                    f.write(f"- **Convergence**: {'Yes' if result.convergence_achieved else 'No'}\n")
                    if result.condition_number is not None:
                        f.write(f"- **Condition number**: {result.condition_number:.2e}\n")
                else:
                    f.write(f"- **Error**: {result.error_message}\n")

                f.write("\n")

        self.logger.info(f"Accuracy report generated: {report_file}")

    def save_results(self) -> None:
        """Save validation results to JSON"""
        results_file = self.output_dir / "accuracy_results.json"

        results_data = [asdict(result) for result in self.results]

        with open(results_file, 'w') as f:
            json.dump(results_data, f, indent=2)

        self.logger.info(f"Accuracy results saved: {results_file}")

    def run_comprehensive_validation(self) -> None:
        """Run complete accuracy validation suite"""
        self.logger.info("Starting comprehensive accuracy validation...")

        try:
            # Validate linear systems
            linear_results = self.validate_linear_systems()
            self.results.extend(linear_results)
            self.logger.info(f"Linear systems validation: {len(linear_results)} tests")

            # Validate PageRank
            pagerank_results = self.validate_pagerank()
            self.results.extend(pagerank_results)
            self.logger.info(f"PageRank validation: {len(pagerank_results)} tests")

            # Save results
            self.save_results()

            # Generate comprehensive report
            self.generate_accuracy_report()

            self.logger.info("Accuracy validation completed successfully")

        except Exception as e:
            self.logger.error(f"Accuracy validation failed: {str(e)}")
            raise

def main():
    """Main entry point"""
    import argparse

    parser = argparse.ArgumentParser(description="Mathematical Accuracy Validation")
    parser.add_argument("--domain", choices=["linear", "pagerank", "all"],
                       default="all", help="Which domain to validate")
    parser.add_argument("--output-dir", default="accuracy_validation",
                       help="Output directory")
    parser.add_argument("--tolerance", choices=["tight", "standard", "relaxed"],
                       default="standard", help="Tolerance level for validation")

    args = parser.parse_args()

    validator = AccuracyValidator(args.output_dir)

    if args.domain == "all":
        validator.run_comprehensive_validation()
    elif args.domain == "linear":
        results = validator.validate_linear_systems()
        validator.results.extend(results)
        validator.save_results()
        validator.generate_accuracy_report()
    elif args.domain == "pagerank":
        results = validator.validate_pagerank()
        validator.results.extend(results)
        validator.save_results()
        validator.generate_accuracy_report()

    print(f"Accuracy validation completed. Results in {args.output_dir}")

if __name__ == "__main__":
    main()