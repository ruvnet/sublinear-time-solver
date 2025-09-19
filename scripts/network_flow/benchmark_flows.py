"""
Network Flow Benchmarking Framework
====================================

Comprehensive benchmarking framework comparing traditional and sublinear flow algorithms
across different network topologies and sizes. Includes:

- Performance metrics collection
- Memory usage analysis
- Scalability testing
- Solution quality validation
- Statistical analysis and reporting
"""

import numpy as np
import time
import psutil
import gc
import statistics
from typing import Dict, List, Tuple, Optional, Any
from dataclasses import dataclass, asdict
import json
import matplotlib.pyplot as plt
import pandas as pd
from pathlib import Path

# Import our flow implementations
from traditional_flows import (
    FlowNetwork, FordFulkersonDFS, EdmondsKarp, PushRelabel, MinCostFlow
)
from sublinear_flows import SublinearFlowSolver
from flow_generators import NetworkSuite


@dataclass
class BenchmarkResult:
    """Result of a single benchmark run."""
    algorithm: str
    network_type: str
    network_size: int
    runtime: float
    memory_peak: float
    memory_avg: float
    max_flow: float
    total_cost: float
    iterations: int
    success: bool
    error_message: Optional[str] = None


@dataclass
class NetworkMetrics:
    """Metrics describing network properties."""
    n_nodes: int
    n_edges: int
    density: float
    avg_degree: float
    max_degree: int
    diameter: Optional[int]
    clustering_coefficient: float


class MemoryMonitor:
    """Monitor memory usage during algorithm execution."""

    def __init__(self):
        self.process = psutil.Process()
        self.peak_memory = 0
        self.memory_samples = []

    def start(self):
        """Start monitoring memory."""
        self.peak_memory = 0
        self.memory_samples = []
        gc.collect()  # Clean up before measurement

    def sample(self):
        """Take a memory sample."""
        memory_mb = self.process.memory_info().rss / (1024 * 1024)
        self.memory_samples.append(memory_mb)
        self.peak_memory = max(self.peak_memory, memory_mb)

    def get_stats(self) -> Tuple[float, float]:
        """Get peak and average memory usage in MB."""
        avg_memory = statistics.mean(self.memory_samples) if self.memory_samples else 0
        return self.peak_memory, avg_memory


class FlowBenchmarkSuite:
    """Comprehensive benchmarking suite for flow algorithms."""

    def __init__(self, output_dir: str = "benchmark_results"):
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True)
        self.memory_monitor = MemoryMonitor()
        self.results = []

    def convert_network_to_traditional(self, network_data: Dict) -> FlowNetwork:
        """Convert generated network to traditional FlowNetwork format."""
        flow_network = FlowNetwork(network_data['n_nodes'])

        for u, v in network_data['edges']:
            capacity = network_data['capacities'][(u, v)]
            cost = network_data['costs'][(u, v)]
            flow_network.add_edge(u, v, int(capacity), cost)

        return flow_network

    def convert_network_to_sublinear(self, network_data: Dict) -> SublinearFlowSolver:
        """Convert generated network to sublinear solver format."""
        solver = SublinearFlowSolver(network_data['n_nodes'])

        for u, v in network_data['edges']:
            capacity = network_data['capacities'][(u, v)]
            cost = network_data['costs'][(u, v)]
            solver.add_edge(u, v, capacity, cost)

        return solver

    def calculate_network_metrics(self, network_data: Dict) -> NetworkMetrics:
        """Calculate network topology metrics."""
        n_nodes = network_data['n_nodes']
        n_edges = len(network_data['edges'])
        density = 2 * n_edges / (n_nodes * (n_nodes - 1)) if n_nodes > 1 else 0

        # Calculate degree distribution
        degrees = [0] * n_nodes
        for u, v in network_data['edges']:
            degrees[u] += 1
            degrees[v] += 1

        avg_degree = statistics.mean(degrees) if degrees else 0
        max_degree = max(degrees) if degrees else 0

        # Estimate clustering coefficient (simplified)
        clustering = self._estimate_clustering(network_data['edges'], n_nodes)

        return NetworkMetrics(
            n_nodes=n_nodes,
            n_edges=n_edges,
            density=density,
            avg_degree=avg_degree,
            max_degree=max_degree,
            diameter=None,  # Expensive to compute
            clustering_coefficient=clustering
        )

    def _estimate_clustering(self, edges: List[Tuple], n_nodes: int) -> float:
        """Estimate clustering coefficient."""
        if n_nodes < 3:
            return 0.0

        # Build adjacency list
        adj = {i: set() for i in range(n_nodes)}
        for u, v in edges:
            adj[u].add(v)
            adj[v].add(u)

        total_clustering = 0
        nodes_with_neighbors = 0

        for node in range(n_nodes):
            neighbors = adj[node]
            if len(neighbors) < 2:
                continue

            # Count triangles
            triangles = 0
            possible_triangles = len(neighbors) * (len(neighbors) - 1) // 2

            for n1 in neighbors:
                for n2 in neighbors:
                    if n1 < n2 and n2 in adj[n1]:
                        triangles += 1

            if possible_triangles > 0:
                total_clustering += triangles / possible_triangles
                nodes_with_neighbors += 1

        return total_clustering / nodes_with_neighbors if nodes_with_neighbors > 0 else 0

    def benchmark_traditional_algorithm(self, algorithm_name: str, network_data: Dict,
                                      source: int, sink: int) -> BenchmarkResult:
        """Benchmark a traditional flow algorithm."""
        try:
            # Convert network
            flow_network = self.convert_network_to_traditional(network_data)

            # Start monitoring
            self.memory_monitor.start()
            start_time = time.perf_counter()

            # Run algorithm
            if algorithm_name == 'ford_fulkerson':
                algorithm = FordFulkersonDFS(flow_network)
                max_flow = algorithm.max_flow(source, sink)
                iterations = 0  # Ford-Fulkerson doesn't track iterations

            elif algorithm_name == 'edmonds_karp':
                algorithm = EdmondsKarp(flow_network)
                max_flow = algorithm.max_flow(source, sink)
                iterations = 0

            elif algorithm_name == 'push_relabel':
                algorithm = PushRelabel(flow_network)
                max_flow = algorithm.max_flow(source, sink)
                iterations = 0

            elif algorithm_name == 'min_cost_flow':
                algorithm = MinCostFlow(flow_network)
                max_flow, total_cost = algorithm.min_cost_max_flow(source, sink)

            else:
                raise ValueError(f"Unknown algorithm: {algorithm_name}")

            # Stop timing
            end_time = time.perf_counter()
            runtime = end_time - start_time

            # Get memory stats
            self.memory_monitor.sample()
            peak_memory, avg_memory = self.memory_monitor.get_stats()

            return BenchmarkResult(
                algorithm=algorithm_name,
                network_type=network_data['properties'].topology_type,
                network_size=network_data['n_nodes'],
                runtime=runtime,
                memory_peak=peak_memory,
                memory_avg=avg_memory,
                max_flow=max_flow,
                total_cost=getattr(algorithm, 'total_cost', 0.0),
                iterations=iterations,
                success=True
            )

        except Exception as e:
            return BenchmarkResult(
                algorithm=algorithm_name,
                network_type=network_data['properties'].topology_type,
                network_size=network_data['n_nodes'],
                runtime=0.0,
                memory_peak=0.0,
                memory_avg=0.0,
                max_flow=0.0,
                total_cost=0.0,
                iterations=0,
                success=False,
                error_message=str(e)
            )

    def benchmark_sublinear_algorithm(self, network_data: Dict, source: int, sink: int) -> BenchmarkResult:
        """Benchmark sublinear flow algorithm."""
        try:
            # Convert network
            solver = self.convert_network_to_sublinear(network_data)

            # Start monitoring
            self.memory_monitor.start()
            start_time = time.perf_counter()

            # Run sublinear max flow
            result = solver.solve_max_flow_as_linear_system(source, sink)

            # Stop timing
            end_time = time.perf_counter()
            runtime = end_time - start_time

            # Get memory stats
            self.memory_monitor.sample()
            peak_memory, avg_memory = self.memory_monitor.get_stats()

            return BenchmarkResult(
                algorithm='sublinear_max_flow',
                network_type=network_data['properties'].topology_type,
                network_size=network_data['n_nodes'],
                runtime=runtime,
                memory_peak=peak_memory,
                memory_avg=avg_memory,
                max_flow=result.get('max_flow', 0.0),
                total_cost=0.0,
                iterations=result.get('iterations', 0),
                success=True
            )

        except Exception as e:
            return BenchmarkResult(
                algorithm='sublinear_max_flow',
                network_type=network_data['properties'].topology_type,
                network_size=network_data['n_nodes'],
                runtime=0.0,
                memory_peak=0.0,
                memory_avg=0.0,
                max_flow=0.0,
                total_cost=0.0,
                iterations=0,
                success=False,
                error_message=str(e)
            )

    def run_comprehensive_benchmark(self, size_categories: List[str] = None) -> List[BenchmarkResult]:
        """Run comprehensive benchmark across all algorithms and network types."""
        if size_categories is None:
            size_categories = ['small', 'medium']

        algorithms = ['ford_fulkerson', 'edmonds_karp', 'push_relabel', 'sublinear_max_flow']
        all_results = []

        suite = NetworkSuite(seed=42)

        for size_category in size_categories:
            print(f"\nBenchmarking {size_category} networks...")

            # Generate test networks
            networks = suite.generate_test_suite(size_category)

            for network_name, network_data in networks.items():
                source = network_data['source']
                sink = network_data['sink']

                print(f"  Testing {network_name} ({network_data['n_nodes']} nodes)...")

                # Calculate network metrics
                metrics = self.calculate_network_metrics(network_data)

                for algorithm in algorithms:
                    print(f"    Running {algorithm}...")

                    if algorithm == 'sublinear_max_flow':
                        result = self.benchmark_sublinear_algorithm(network_data, source, sink)
                    else:
                        result = self.benchmark_traditional_algorithm(algorithm, network_data, source, sink)

                    all_results.append(result)

                    if result.success:
                        print(f"      ✓ Flow: {result.max_flow:.1f}, Time: {result.runtime:.4f}s")
                    else:
                        print(f"      ✗ Error: {result.error_message}")

        self.results.extend(all_results)
        return all_results

    def analyze_results(self) -> Dict[str, Any]:
        """Analyze benchmark results and generate statistics."""
        if not self.results:
            return {}

        # Convert to DataFrame for analysis
        df = pd.DataFrame([asdict(r) for r in self.results if r.success])

        if df.empty:
            return {"error": "No successful benchmark results"}

        analysis = {
            'summary': {
                'total_runs': len(self.results),
                'successful_runs': len(df),
                'algorithms_tested': df['algorithm'].unique().tolist(),
                'network_types_tested': df['network_type'].unique().tolist()
            },
            'performance_by_algorithm': {},
            'performance_by_network_type': {},
            'scalability_analysis': {},
            'memory_analysis': {}
        }

        # Performance by algorithm
        for algorithm in df['algorithm'].unique():
            alg_data = df[df['algorithm'] == algorithm]
            analysis['performance_by_algorithm'][algorithm] = {
                'avg_runtime': float(alg_data['runtime'].mean()),
                'std_runtime': float(alg_data['runtime'].std()),
                'avg_memory': float(alg_data['memory_peak'].mean()),
                'runs': len(alg_data)
            }

        # Performance by network type
        for network_type in df['network_type'].unique():
            net_data = df[df['network_type'] == network_type]
            analysis['performance_by_network_type'][network_type] = {
                'avg_runtime': float(net_data['runtime'].mean()),
                'avg_flow': float(net_data['max_flow'].mean()),
                'runs': len(net_data)
            }

        # Scalability analysis
        for algorithm in df['algorithm'].unique():
            alg_data = df[df['algorithm'] == algorithm]
            if len(alg_data) > 1:
                # Simple correlation between network size and runtime
                correlation = alg_data['network_size'].corr(alg_data['runtime'])
                analysis['scalability_analysis'][algorithm] = {
                    'size_runtime_correlation': float(correlation) if not pd.isna(correlation) else 0.0
                }

        return analysis

    def generate_visualizations(self):
        """Generate benchmark visualization plots."""
        if not self.results:
            print("No results to visualize")
            return

        df = pd.DataFrame([asdict(r) for r in self.results if r.success])
        if df.empty:
            print("No successful results to visualize")
            return

        # Performance comparison by algorithm
        plt.figure(figsize=(12, 8))

        # Runtime comparison
        plt.subplot(2, 2, 1)
        runtime_by_alg = df.groupby('algorithm')['runtime'].mean()
        runtime_by_alg.plot(kind='bar')
        plt.title('Average Runtime by Algorithm')
        plt.ylabel('Runtime (seconds)')
        plt.xticks(rotation=45)

        # Memory usage comparison
        plt.subplot(2, 2, 2)
        memory_by_alg = df.groupby('algorithm')['memory_peak'].mean()
        memory_by_alg.plot(kind='bar', color='orange')
        plt.title('Average Peak Memory by Algorithm')
        plt.ylabel('Memory (MB)')
        plt.xticks(rotation=45)

        # Performance by network type
        plt.subplot(2, 2, 3)
        runtime_by_net = df.groupby('network_type')['runtime'].mean()
        runtime_by_net.plot(kind='bar', color='green')
        plt.title('Average Runtime by Network Type')
        plt.ylabel('Runtime (seconds)')
        plt.xticks(rotation=45)

        # Scalability plot
        plt.subplot(2, 2, 4)
        for algorithm in df['algorithm'].unique():
            alg_data = df[df['algorithm'] == algorithm]
            plt.scatter(alg_data['network_size'], alg_data['runtime'],
                       label=algorithm, alpha=0.7)
        plt.xlabel('Network Size (nodes)')
        plt.ylabel('Runtime (seconds)')
        plt.title('Scalability Comparison')
        plt.legend()
        plt.yscale('log')

        plt.tight_layout()
        plt.savefig(self.output_dir / 'benchmark_visualization.png', dpi=300, bbox_inches='tight')
        plt.close()

        print(f"Visualization saved to {self.output_dir / 'benchmark_visualization.png'}")

    def save_results(self, filename: str = "benchmark_results.json"):
        """Save benchmark results to file."""
        output_file = self.output_dir / filename

        # Convert results to JSON-serializable format
        json_results = []
        for result in self.results:
            json_results.append(asdict(result))

        with open(output_file, 'w') as f:
            json.dump({
                'benchmark_metadata': {
                    'timestamp': time.time(),
                    'total_runs': len(self.results),
                    'successful_runs': len([r for r in self.results if r.success])
                },
                'results': json_results
            }, f, indent=2)

        print(f"Results saved to {output_file}")

    def generate_report(self, filename: str = "benchmark_report.md"):
        """Generate comprehensive benchmark report."""
        analysis = self.analyze_results()
        if not analysis:
            print("No results to report")
            return

        report_file = self.output_dir / filename

        with open(report_file, 'w') as f:
            f.write("# Network Flow Algorithm Benchmark Report\n\n")
            f.write(f"Generated: {time.strftime('%Y-%m-%d %H:%M:%S')}\n\n")

            # Summary
            f.write("## Summary\n\n")
            summary = analysis['summary']
            f.write(f"- Total benchmark runs: {summary['total_runs']}\n")
            f.write(f"- Successful runs: {summary['successful_runs']}\n")
            f.write(f"- Algorithms tested: {', '.join(summary['algorithms_tested'])}\n")
            f.write(f"- Network types tested: {', '.join(summary['network_types_tested'])}\n\n")

            # Performance by algorithm
            f.write("## Performance by Algorithm\n\n")
            for alg, stats in analysis['performance_by_algorithm'].items():
                f.write(f"### {alg}\n")
                f.write(f"- Average runtime: {stats['avg_runtime']:.6f} seconds\n")
                f.write(f"- Runtime std deviation: {stats['std_runtime']:.6f} seconds\n")
                f.write(f"- Average peak memory: {stats['avg_memory']:.2f} MB\n")
                f.write(f"- Number of runs: {stats['runs']}\n\n")

            # Network type analysis
            f.write("## Performance by Network Type\n\n")
            for net_type, stats in analysis['performance_by_network_type'].items():
                f.write(f"### {net_type}\n")
                f.write(f"- Average runtime: {stats['avg_runtime']:.6f} seconds\n")
                f.write(f"- Average max flow: {stats['avg_flow']:.2f}\n")
                f.write(f"- Number of runs: {stats['runs']}\n\n")

            # Conclusions
            f.write("## Key Findings\n\n")

            # Find best performing algorithm
            perf_data = analysis['performance_by_algorithm']
            if perf_data:
                best_runtime = min(perf_data.items(), key=lambda x: x[1]['avg_runtime'])
                best_memory = min(perf_data.items(), key=lambda x: x[1]['avg_memory'])

                f.write(f"- **Fastest algorithm**: {best_runtime[0]} "
                       f"(avg: {best_runtime[1]['avg_runtime']:.6f}s)\n")
                f.write(f"- **Most memory efficient**: {best_memory[0]} "
                       f"(avg: {best_memory[1]['avg_memory']:.2f} MB)\n\n")

            # Scalability insights
            if 'scalability_analysis' in analysis:
                f.write("### Scalability Analysis\n")
                for alg, scaling in analysis['scalability_analysis'].items():
                    corr = scaling['size_runtime_correlation']
                    if corr > 0.8:
                        trend = "strong positive correlation"
                    elif corr > 0.5:
                        trend = "moderate positive correlation"
                    elif corr > 0.2:
                        trend = "weak positive correlation"
                    else:
                        trend = "no clear correlation"

                    f.write(f"- **{alg}**: {trend} between network size and runtime "
                           f"(r={corr:.3f})\n")

        print(f"Report generated: {report_file}")


if __name__ == "__main__":
    # Run comprehensive benchmark
    print("Network Flow Algorithm Benchmark Suite")
    print("=" * 50)

    benchmark = FlowBenchmarkSuite()

    # Run benchmarks
    results = benchmark.run_comprehensive_benchmark(['small', 'medium'])

    # Analyze and report
    analysis = benchmark.analyze_results()
    print(f"\nBenchmark completed. {len(results)} runs executed.")

    # Generate outputs
    benchmark.save_results()
    benchmark.generate_report()

    try:
        benchmark.generate_visualizations()
    except ImportError:
        print("Matplotlib not available - skipping visualizations")

    print(f"\nResults saved to: {benchmark.output_dir}")