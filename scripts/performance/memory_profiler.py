#!/usr/bin/env python3
"""
Advanced Memory Profiling for Sublinear-Time Solver
Comprehensive memory usage analysis across all algorithms and domains
"""

import time
import psutil
import numpy as np
import json
import sys
import gc
import tracemalloc
from pathlib import Path
from typing import Dict, List, Tuple, Optional, Any, Callable
from dataclasses import dataclass, asdict
import logging
from contextlib import contextmanager
import threading
import queue
from collections import defaultdict

# Add project root to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

@dataclass
class MemorySnapshot:
    """Single memory measurement snapshot"""
    timestamp: float
    current_memory_mb: float
    peak_memory_mb: float
    system_memory_percent: float
    gc_objects: int
    operation: str = ""
    problem_size: int = 0

@dataclass
class MemoryProfile:
    """Complete memory profile for an operation"""
    operation_name: str
    algorithm: str
    domain: str
    problem_size: int
    duration_seconds: float
    snapshots: List[MemorySnapshot]
    peak_memory_mb: float
    final_memory_mb: float
    memory_efficiency: float  # MB per problem unit
    gc_collections: int
    memory_leaks_detected: bool
    error_message: str = ""

class MemoryProfiler:
    """Advanced memory profiling and analysis"""

    def __init__(self, output_dir: str = "memory_profiles"):
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True)
        self.logger = self._setup_logging()
        self.profiles: List[MemoryProfile] = []
        self._monitoring_active = False
        self._snapshot_queue = queue.Queue()

    def _setup_logging(self) -> logging.Logger:
        logger = logging.getLogger("MemoryProfiler")
        logger.setLevel(logging.INFO)

        handler = logging.StreamHandler()
        formatter = logging.Formatter('%(asctime)s - %(levelname)s - %(message)s')
        handler.setFormatter(formatter)
        logger.addHandler(handler)

        return logger

    @contextmanager
    def profile_memory(self, operation_name: str, algorithm: str, domain: str,
                      problem_size: int):
        """Context manager for memory profiling"""
        self.logger.info(f"Starting memory profile: {operation_name}")

        # Start monitoring
        snapshots = []
        start_time = time.time()
        gc_start = len(gc.get_objects())

        # Enable detailed tracing
        tracemalloc.start()

        # Get initial snapshot
        initial_snapshot = self._take_snapshot(operation_name + "_start", problem_size)
        snapshots.append(initial_snapshot)

        # Start background monitoring
        monitoring_thread = threading.Thread(
            target=self._background_monitor,
            args=(operation_name, problem_size, snapshots),
            daemon=True
        )
        self._monitoring_active = True
        monitoring_thread.start()

        try:
            yield self

        except Exception as e:
            self.logger.error(f"Error during profiled operation: {str(e)}")
            raise

        finally:
            # Stop monitoring
            self._monitoring_active = False
            monitoring_thread.join(timeout=1.0)

            # Final snapshot
            final_snapshot = self._take_snapshot(operation_name + "_end", problem_size)
            snapshots.append(final_snapshot)

            # Get peak memory
            current, peak = tracemalloc.get_traced_memory()
            tracemalloc.stop()

            # Calculate metrics
            duration = time.time() - start_time
            gc_end = len(gc.get_objects())
            gc_collections = gc_end - gc_start

            # Detect memory leaks (simple heuristic)
            memory_leak = final_snapshot.current_memory_mb > initial_snapshot.current_memory_mb * 1.1

            # Calculate efficiency
            efficiency = peak / (1024 * 1024) / problem_size if problem_size > 0 else 0

            # Create profile
            profile = MemoryProfile(
                operation_name=operation_name,
                algorithm=algorithm,
                domain=domain,
                problem_size=problem_size,
                duration_seconds=duration,
                snapshots=snapshots,
                peak_memory_mb=peak / (1024 * 1024),
                final_memory_mb=final_snapshot.current_memory_mb,
                memory_efficiency=efficiency,
                gc_collections=gc_collections,
                memory_leaks_detected=memory_leak
            )

            self.profiles.append(profile)
            self.logger.info(f"Memory profile completed: {operation_name} "
                           f"(Peak: {peak / (1024 * 1024):.1f} MB)")

    def _take_snapshot(self, operation: str, problem_size: int) -> MemorySnapshot:
        """Take a memory snapshot"""
        process = psutil.Process()
        memory_info = process.memory_info()

        current_memory = memory_info.rss / (1024 * 1024)  # MB
        memory_percent = process.memory_percent()

        # Get tracemalloc info if available
        peak_memory = current_memory
        try:
            current_traced, peak_traced = tracemalloc.get_traced_memory()
            peak_memory = max(peak_memory, peak_traced / (1024 * 1024))
        except:
            pass

        return MemorySnapshot(
            timestamp=time.time(),
            current_memory_mb=current_memory,
            peak_memory_mb=peak_memory,
            system_memory_percent=memory_percent,
            gc_objects=len(gc.get_objects()),
            operation=operation,
            problem_size=problem_size
        )

    def _background_monitor(self, operation: str, problem_size: int,
                          snapshots: List[MemorySnapshot]):
        """Background thread for continuous memory monitoring"""
        while self._monitoring_active:
            snapshot = self._take_snapshot(f"{operation}_monitor", problem_size)
            snapshots.append(snapshot)
            time.sleep(0.1)  # Sample every 100ms

    def profile_matrix_operations(self, max_size: int = 10000) -> None:
        """Profile memory usage for matrix operations"""
        self.logger.info("Profiling matrix operations memory usage...")

        sizes = [100, 500, 1000, 2000, 5000]
        if max_size > 5000:
            sizes.extend([7500, 10000])

        methods = ["neumann", "random-walk", "forward-push"]

        for size in sizes:
            for method in methods:
                if size * size * 8 > psutil.virtual_memory().available * 0.5:
                    self.logger.warning(f"Skipping {size}x{size} - insufficient memory")
                    continue

                try:
                    with self.profile_memory(
                        f"matrix_solve_{method}",
                        method,
                        "linear_systems",
                        size * size
                    ):
                        self._simulate_matrix_solve(size, method)

                except Exception as e:
                    self.logger.error(f"Matrix profiling failed for {size}x{size}: {str(e)}")

                # Force garbage collection between tests
                gc.collect()

    def _simulate_matrix_solve(self, size: int, method: str) -> None:
        """Simulate matrix solving with memory tracking"""
        self.logger.debug(f"Simulating {method} solve for {size}x{size} matrix")

        # Generate matrix
        np.random.seed(42)
        A = np.random.rand(size, size)
        A = A + A.T  # Symmetric
        A = A + size * np.eye(size)  # Diagonally dominant
        b = np.random.rand(size)

        # Simulate solving process
        if method == "neumann":
            # Neumann series: x = b + Mb + M²b + ...
            M = np.eye(size) - A / np.linalg.norm(A)
            x = b.copy()
            for i in range(10):  # 10 iterations
                x = x + M @ x
                time.sleep(0.01)  # Simulate computation time

        elif method == "random-walk":
            # Simulate random walk on matrix
            x = np.zeros(size)
            for i in range(100):  # Multiple walks
                walk_result = np.random.rand(size)
                x += walk_result
                time.sleep(0.001)

        else:  # forward-push or other
            # Simple iterative method
            x = b.copy()
            for i in range(20):
                x = x + 0.1 * (b - A @ x)
                time.sleep(0.005)

        # Clean up local variables
        del A, b, x

    def profile_graph_operations(self, max_nodes: int = 20000) -> None:
        """Profile memory usage for graph operations"""
        self.logger.info("Profiling graph operations memory usage...")

        sizes = [100, 500, 1000, 2000, 5000, 10000]
        if max_nodes > 10000:
            sizes.extend([15000, 20000])

        for size in sizes:
            # Estimate memory requirement for adjacency matrix
            estimated_memory_gb = (size * size * 8) / (1024**3)
            available_memory_gb = psutil.virtual_memory().available / (1024**3)

            if estimated_memory_gb > available_memory_gb * 0.5:
                self.logger.warning(f"Skipping {size} nodes - insufficient memory")
                continue

            try:
                with self.profile_memory(
                    "graph_pagerank",
                    "pagerank",
                    "graph_algorithms",
                    size
                ):
                    self._simulate_pagerank(size)

            except Exception as e:
                self.logger.error(f"Graph profiling failed for {size} nodes: {str(e)}")

            # Force cleanup
            gc.collect()

    def _simulate_pagerank(self, num_nodes: int) -> None:
        """Simulate PageRank with memory tracking"""
        self.logger.debug(f"Simulating PageRank for {num_nodes} nodes")

        # Generate adjacency matrix
        np.random.seed(42)
        adjacency = np.zeros((num_nodes, num_nodes))

        # Add edges (sparse graph)
        num_edges = int(num_nodes * np.log(num_nodes))
        for _ in range(num_edges):
            i, j = np.random.randint(0, num_nodes, 2)
            if i != j:
                adjacency[i, j] = 1

        # Normalize
        row_sums = adjacency.sum(axis=1)
        row_sums[row_sums == 0] = 1  # Avoid division by zero
        adjacency = adjacency / row_sums[:, np.newaxis]

        # PageRank iteration
        damping = 0.85
        pagerank = np.ones(num_nodes) / num_nodes

        for iteration in range(50):
            prev_pagerank = pagerank.copy()
            pagerank = (1 - damping) / num_nodes + damping * adjacency.T @ pagerank

            # Check convergence
            if np.linalg.norm(pagerank - prev_pagerank) < 1e-6:
                break

            time.sleep(0.01)  # Simulate computation

        # Clean up
        del adjacency, pagerank

    def profile_sparse_operations(self) -> None:
        """Profile sparse matrix operations"""
        self.logger.info("Profiling sparse matrix operations...")

        try:
            from scipy.sparse import random, csr_matrix

            sizes = [1000, 5000, 10000, 20000]
            densities = [0.01, 0.05, 0.1]  # 1%, 5%, 10% non-zero

            for size in sizes:
                for density in densities:
                    try:
                        with self.profile_memory(
                            f"sparse_solve_density_{int(density*100)}",
                            "sparse_iterative",
                            "sparse_systems",
                            int(size * size * density)
                        ):
                            self._simulate_sparse_solve(size, density)

                    except Exception as e:
                        self.logger.error(f"Sparse profiling failed: {str(e)}")

                    gc.collect()

        except ImportError:
            self.logger.warning("SciPy not available for sparse matrix profiling")

    def _simulate_sparse_solve(self, size: int, density: float) -> None:
        """Simulate sparse matrix solving"""
        try:
            from scipy.sparse import random, csr_matrix
            import scipy.sparse.linalg as spla

            # Generate sparse matrix
            A = random(size, size, density=density, format='csr')
            A = A + A.T  # Make symmetric
            A.setdiag(A.diagonal() + size)  # Make diagonally dominant

            b = np.random.rand(size)

            # Solve using iterative method
            x, info = spla.cg(A, b, maxiter=100)

            del A, b, x

        except ImportError:
            # Fallback without scipy
            self.logger.debug("Using numpy fallback for sparse simulation")
            time.sleep(0.5)  # Simulate work

    def analyze_memory_patterns(self) -> Dict[str, Any]:
        """Analyze memory usage patterns across all profiles"""
        self.logger.info("Analyzing memory patterns...")

        analysis = {
            "efficiency_analysis": {},
            "scaling_analysis": {},
            "leak_detection": {},
            "peak_memory_analysis": {},
            "recommendations": []
        }

        # Group profiles by domain and algorithm
        domain_profiles = defaultdict(lambda: defaultdict(list))
        for profile in self.profiles:
            domain_profiles[profile.domain][profile.algorithm].append(profile)

        # Efficiency analysis
        for domain, algorithms in domain_profiles.items():
            analysis["efficiency_analysis"][domain] = {}
            for algorithm, profiles in algorithms.items():
                efficiencies = [p.memory_efficiency for p in profiles if p.memory_efficiency > 0]
                if efficiencies:
                    analysis["efficiency_analysis"][domain][algorithm] = {
                        "avg_efficiency_mb_per_unit": np.mean(efficiencies),
                        "min_efficiency": np.min(efficiencies),
                        "max_efficiency": np.max(efficiencies),
                        "efficiency_std": np.std(efficiencies)
                    }

        # Scaling analysis
        for domain, algorithms in domain_profiles.items():
            analysis["scaling_analysis"][domain] = {}
            for algorithm, profiles in algorithms.items():
                if len(profiles) >= 3:
                    # Sort by problem size
                    profiles.sort(key=lambda x: x.problem_size)

                    sizes = [p.problem_size for p in profiles]
                    peak_memories = [p.peak_memory_mb for p in profiles]

                    # Calculate memory scaling
                    if len(sizes) >= 2:
                        memory_scaling = peak_memories[-1] / peak_memories[0] if peak_memories[0] > 0 else float('inf')
                        size_scaling = sizes[-1] / sizes[0] if sizes[0] > 0 else float('inf')

                        analysis["scaling_analysis"][domain][algorithm] = {
                            "memory_scaling_factor": memory_scaling,
                            "size_scaling_factor": size_scaling,
                            "memory_growth_rate": memory_scaling / size_scaling if size_scaling > 0 else float('inf'),
                            "size_range": f"{sizes[0]}-{sizes[-1]}",
                            "memory_range_mb": f"{peak_memories[0]:.1f}-{peak_memories[-1]:.1f}"
                        }

        # Leak detection
        leaky_profiles = [p for p in self.profiles if p.memory_leaks_detected]
        analysis["leak_detection"] = {
            "total_profiles": len(self.profiles),
            "profiles_with_leaks": len(leaky_profiles),
            "leak_percentage": len(leaky_profiles) / len(self.profiles) * 100 if self.profiles else 0,
            "leaky_operations": [p.operation_name for p in leaky_profiles]
        }

        # Peak memory analysis
        all_peaks = [p.peak_memory_mb for p in self.profiles]
        if all_peaks:
            analysis["peak_memory_analysis"] = {
                "max_peak_memory_mb": np.max(all_peaks),
                "avg_peak_memory_mb": np.mean(all_peaks),
                "median_peak_memory_mb": np.median(all_peaks),
                "memory_variance": np.var(all_peaks),
                "system_memory_gb": psutil.virtual_memory().total / (1024**3)
            }

        # Generate recommendations
        analysis["recommendations"] = self._generate_memory_recommendations(analysis)

        return analysis

    def _generate_memory_recommendations(self, analysis: Dict[str, Any]) -> List[str]:
        """Generate memory optimization recommendations"""
        recommendations = []

        # Check for memory leaks
        leak_data = analysis.get("leak_detection", {})
        if leak_data.get("leak_percentage", 0) > 10:
            recommendations.append("🚨 HIGH: Memory leaks detected in multiple operations - implement proper cleanup")

        # Check efficiency
        efficiency_data = analysis.get("efficiency_analysis", {})
        for domain, algorithms in efficiency_data.items():
            for algo, stats in algorithms.items():
                avg_eff = stats.get("avg_efficiency_mb_per_unit", 0)
                if avg_eff > 1.0:  # More than 1MB per unit is concerning
                    recommendations.append(f"⚠️ MEDIUM: {algo} in {domain} uses {avg_eff:.2f} MB per unit - consider optimization")

        # Check scaling
        scaling_data = analysis.get("scaling_analysis", {})
        for domain, algorithms in scaling_data.items():
            for algo, stats in algorithms.items():
                growth_rate = stats.get("memory_growth_rate", 1)
                if growth_rate > 2.0:  # Memory grows faster than problem size
                    recommendations.append(f"⚠️ MEDIUM: {algo} in {domain} shows superlinear memory growth ({growth_rate:.1f}x)")

        # Check peak memory
        peak_data = analysis.get("peak_memory_analysis", {})
        system_memory_gb = peak_data.get("system_memory_gb", 0)
        max_peak_gb = peak_data.get("max_peak_memory_mb", 0) / 1024

        if max_peak_gb > system_memory_gb * 0.5:
            recommendations.append(f"🚨 HIGH: Peak memory usage ({max_peak_gb:.1f} GB) exceeds 50% of system memory")

        if not recommendations:
            recommendations.append("✅ No major memory issues detected")

        return recommendations

    def generate_memory_report(self) -> None:
        """Generate comprehensive memory analysis report"""
        self.logger.info("Generating memory analysis report...")

        analysis = self.analyze_memory_patterns()

        report_file = self.output_dir / "memory_analysis_report.md"
        with open(report_file, 'w') as f:
            f.write("# Memory Usage Analysis Report\n\n")
            f.write(f"Generated: {time.strftime('%Y-%m-%d %H:%M:%S')}\n\n")

            # Executive Summary
            f.write("## Executive Summary\n\n")
            f.write(f"- **Total profiles analyzed**: {len(self.profiles)}\n")

            peak_data = analysis.get("peak_memory_analysis", {})
            if peak_data:
                f.write(f"- **Maximum peak memory**: {peak_data.get('max_peak_memory_mb', 0):.1f} MB\n")
                f.write(f"- **Average peak memory**: {peak_data.get('avg_peak_memory_mb', 0):.1f} MB\n")
                f.write(f"- **System memory**: {peak_data.get('system_memory_gb', 0):.1f} GB\n")

            leak_data = analysis.get("leak_detection", {})
            if leak_data:
                f.write(f"- **Memory leaks detected**: {leak_data.get('profiles_with_leaks', 0)} profiles ({leak_data.get('leak_percentage', 0):.1f}%)\n")

            # Recommendations
            f.write("\n## Key Recommendations\n\n")
            for rec in analysis.get("recommendations", []):
                f.write(f"- {rec}\n")

            # Efficiency Analysis
            f.write("\n## Memory Efficiency Analysis\n\n")
            efficiency_data = analysis.get("efficiency_analysis", {})
            for domain, algorithms in efficiency_data.items():
                f.write(f"### {domain.title()}\n\n")
                for algo, stats in algorithms.items():
                    f.write(f"**{algo}**:\n")
                    f.write(f"- Average efficiency: {stats.get('avg_efficiency_mb_per_unit', 0):.4f} MB per unit\n")
                    f.write(f"- Efficiency range: {stats.get('min_efficiency', 0):.4f} - {stats.get('max_efficiency', 0):.4f} MB per unit\n")
                    f.write(f"- Standard deviation: {stats.get('efficiency_std', 0):.4f}\n\n")

            # Scaling Analysis
            f.write("## Memory Scaling Analysis\n\n")
            scaling_data = analysis.get("scaling_analysis", {})
            for domain, algorithms in scaling_data.items():
                f.write(f"### {domain.title()}\n\n")
                for algo, stats in algorithms.items():
                    f.write(f"**{algo}**:\n")
                    f.write(f"- Problem size range: {stats.get('size_range', 'N/A')}\n")
                    f.write(f"- Memory range: {stats.get('memory_range_mb', 'N/A')} MB\n")
                    f.write(f"- Memory scaling factor: {stats.get('memory_scaling_factor', 'N/A'):.2f}x\n")
                    f.write(f"- Memory growth rate: {stats.get('memory_growth_rate', 'N/A'):.2f}x problem size\n\n")

            # Detailed Profiles
            f.write("## Detailed Memory Profiles\n\n")
            for profile in self.profiles:
                f.write(f"### {profile.operation_name}\n")
                f.write(f"- **Algorithm**: {profile.algorithm}\n")
                f.write(f"- **Domain**: {profile.domain}\n")
                f.write(f"- **Problem size**: {profile.problem_size:,}\n")
                f.write(f"- **Duration**: {profile.duration_seconds:.3f} seconds\n")
                f.write(f"- **Peak memory**: {profile.peak_memory_mb:.1f} MB\n")
                f.write(f"- **Memory efficiency**: {profile.memory_efficiency:.6f} MB per unit\n")
                f.write(f"- **Memory leak detected**: {'Yes' if profile.memory_leaks_detected else 'No'}\n")
                f.write(f"- **Snapshots taken**: {len(profile.snapshots)}\n\n")

        self.logger.info(f"Memory report generated: {report_file}")

    def save_profiles(self) -> None:
        """Save all memory profiles to JSON"""
        profiles_file = self.output_dir / "memory_profiles.json"

        # Convert profiles to dictionaries
        profiles_data = []
        for profile in self.profiles:
            profile_dict = asdict(profile)
            profiles_data.append(profile_dict)

        with open(profiles_file, 'w') as f:
            json.dump(profiles_data, f, indent=2)

        self.logger.info(f"Memory profiles saved: {profiles_file}")

    def run_comprehensive_memory_analysis(self) -> None:
        """Run complete memory profiling suite"""
        self.logger.info("Starting comprehensive memory analysis...")

        try:
            # Profile matrix operations
            self.profile_matrix_operations()

            # Profile graph operations
            self.profile_graph_operations()

            # Profile sparse operations
            self.profile_sparse_operations()

            # Save profiles
            self.save_profiles()

            # Generate analysis report
            self.generate_memory_report()

            self.logger.info("Memory analysis completed successfully")

        except Exception as e:
            self.logger.error(f"Memory analysis failed: {str(e)}")
            raise

def main():
    """Main entry point"""
    import argparse

    parser = argparse.ArgumentParser(description="Memory Profiling Suite")
    parser.add_argument("--max-matrix-size", type=int, default=5000,
                       help="Maximum matrix size to profile")
    parser.add_argument("--max-graph-nodes", type=int, default=10000,
                       help="Maximum graph nodes to profile")
    parser.add_argument("--output-dir", default="memory_profiles",
                       help="Output directory")
    parser.add_argument("--profile-sparse", action="store_true",
                       help="Include sparse matrix profiling")

    args = parser.parse_args()

    profiler = MemoryProfiler(args.output_dir)

    # Run specific profiling
    profiler.profile_matrix_operations(args.max_matrix_size)
    profiler.profile_graph_operations(args.max_graph_nodes)

    if args.profile_sparse:
        profiler.profile_sparse_operations()

    # Generate reports
    profiler.save_profiles()
    profiler.generate_memory_report()

    print(f"Memory profiling completed. Results in {args.output_dir}")

if __name__ == "__main__":
    main()