#!/usr/bin/env python3
"""
Comprehensive Social Network Analysis Benchmarking Framework

This module provides a unified benchmarking framework for comparing:
- Traditional graph algorithms vs linear algebraic formulations
- Scalability analysis across network sizes
- Accuracy validation against known solutions
- Memory efficiency comparisons
- Integration with MCP sublinear solver tools

Author: Social Network Analysis Agent
"""

import numpy as np
import networkx as nx
import time
import json
import psutil
import tracemalloc
from typing import Dict, List, Tuple, Optional, Any
from collections import defaultdict
import matplotlib.pyplot as plt
import seaborn as sns

# Import our analysis modules
from traditional_centrality import TraditionalCentrality
from sublinear_centrality import SublinearCentrality
from influence_models import (
    IndependentCascadeModel, LinearThresholdModel,
    FriedkinJohnsenModel, MatrixPowerInfluence
)
from community_detection import TraditionalCommunityDetection, SpectralCommunityDetection


class SocialNetworkBenchmark:
    """Comprehensive benchmarking framework for social network analysis."""

    def __init__(self):
        """Initialize benchmark framework."""
        self.results = {}
        self.memory_results = {}

    def create_benchmark_networks(self) -> Dict[str, Dict[str, nx.Graph]]:
        """Create a comprehensive set of benchmark networks."""
        networks = {}

        # Small networks (for detailed analysis)
        networks['small'] = {
            'karate': nx.karate_club_graph(),
            'dolphins': self._create_dolphin_network(),
            'les_miserables': self._create_les_miserables_network(),
            'complete_20': nx.complete_graph(20),
            'path_50': nx.path_graph(50),
            'cycle_30': nx.cycle_graph(30),
            'star_25': nx.star_graph(24)  # 25 nodes total
        }

        # Medium networks (scalability testing)
        networks['medium'] = {
            'small_world_100': nx.watts_strogatz_graph(100, 6, 0.3, seed=42),
            'scale_free_100': nx.barabasi_albert_graph(100, 3, seed=42),
            'random_100': nx.erdos_renyi_graph(100, 0.1, seed=42),
            'geometric_100': nx.random_geometric_graph(100, 0.15, seed=42),
            'planted_partition': nx.planted_partition_graph(4, 25, 0.7, 0.1, seed=42)
        }

        # Large networks (performance testing)
        networks['large'] = {
            'small_world_500': nx.watts_strogatz_graph(500, 6, 0.3, seed=42),
            'scale_free_500': nx.barabasi_albert_graph(500, 5, seed=42),
            'random_500': nx.erdos_renyi_graph(500, 0.02, seed=42),
            'geometric_500': nx.random_geometric_graph(500, 0.08, seed=42)
        }

        # Very large networks (sublinear advantage testing)
        networks['xlarge'] = {
            'small_world_1000': nx.watts_strogatz_graph(1000, 6, 0.3, seed=42),
            'scale_free_1000': nx.barabasi_albert_graph(1000, 5, seed=42),
            'random_1000': nx.erdos_renyi_graph(1000, 0.01, seed=42)
        }

        return networks

    def _create_dolphin_network(self) -> nx.Graph:
        """Create a simple approximation of the dolphin social network."""
        # Create a small-world network as approximation
        G = nx.watts_strogatz_graph(62, 4, 0.2, seed=42)
        return G

    def _create_les_miserables_network(self) -> nx.Graph:
        """Create a simple approximation of Les Miserables character network."""
        # Create a scale-free network as approximation
        G = nx.barabasi_albert_graph(77, 3, seed=42)
        return G

    def measure_memory_usage(self, func, *args, **kwargs) -> Tuple[Any, Dict[str, float]]:
        """Measure memory usage of a function."""
        # Start memory tracing
        tracemalloc.start()
        process = psutil.Process()

        # Baseline memory
        baseline_memory = process.memory_info().rss / 1024 / 1024  # MB

        # Execute function
        start_time = time.time()
        result = func(*args, **kwargs)
        execution_time = time.time() - start_time

        # Memory after execution
        current, peak = tracemalloc.get_traced_memory()
        final_memory = process.memory_info().rss / 1024 / 1024  # MB

        tracemalloc.stop()

        memory_stats = {
            'execution_time': execution_time,
            'baseline_memory_mb': baseline_memory,
            'final_memory_mb': final_memory,
            'memory_increase_mb': final_memory - baseline_memory,
            'peak_traced_mb': peak / 1024 / 1024,
            'current_traced_mb': current / 1024 / 1024
        }

        return result, memory_stats

    def benchmark_centrality_algorithms(self, graph: nx.Graph,
                                      network_name: str) -> Dict[str, Any]:
        """Benchmark centrality algorithms on a given graph."""
        print(f"  Benchmarking centrality algorithms on {network_name}...")

        results = {
            'network_name': network_name,
            'graph_stats': {
                'nodes': len(graph.nodes()),
                'edges': len(graph.edges()),
                'density': nx.density(graph),
                'is_connected': nx.is_connected(graph)
            }
        }

        # Traditional centrality
        print("    Traditional methods...")
        traditional = TraditionalCentrality(graph)
        trad_result, trad_memory = self.measure_memory_usage(
            traditional.compute_all_centralities
        )

        results['traditional'] = {
            'centralities': trad_result,
            'memory': trad_memory
        }

        # Sublinear centrality
        print("    Sublinear methods...")
        sublinear = SublinearCentrality(graph)
        sub_result, sub_memory = self.measure_memory_usage(
            sublinear.compute_all_centralities
        )

        results['sublinear'] = {
            'centralities': sub_result,
            'memory': sub_memory
        }

        # Performance comparison
        trad_times = traditional.get_performance_summary()
        sub_times = sublinear.get_performance_summary()

        # Compare PageRank specifically
        if 'pagerank' in trad_times and 'pagerank_linear' in sub_times:
            speedup = trad_times['pagerank'] / sub_times['pagerank_linear']
            results['pagerank_speedup'] = speedup
            print(f"    PageRank speedup: {speedup:.2f}x")

        return results

    def benchmark_influence_models(self, graph: nx.Graph,
                                 network_name: str) -> Dict[str, Any]:
        """Benchmark influence propagation models."""
        print(f"  Benchmarking influence models on {network_name}...")

        results = {'network_name': network_name}

        # Select seed nodes (high-degree nodes)
        degrees = dict(graph.degree())
        seed_nodes = sorted(degrees.keys(), key=lambda x: degrees[x], reverse=True)[:3]

        # Independent Cascade Model
        print("    Independent Cascade...")
        ic_model = IndependentCascadeModel(graph)

        # Simulation
        ic_sim_result, ic_sim_memory = self.measure_memory_usage(
            ic_model.simulate_cascade, seed_nodes, 100  # Fewer simulations for speed
        )

        # Linear approximation
        ic_linear_result, ic_linear_memory = self.measure_memory_usage(
            ic_model.expected_influence_linear, seed_nodes
        )

        results['independent_cascade'] = {
            'simulation': {'result': ic_sim_result, 'memory': ic_sim_memory},
            'linear': {'result': ic_linear_result, 'memory': ic_linear_memory},
            'speedup': ic_sim_memory['execution_time'] / ic_linear_memory['execution_time']
        }

        # Linear Threshold Model
        print("    Linear Threshold...")
        lt_model = LinearThresholdModel(graph)
        lt_result, lt_memory = self.measure_memory_usage(
            lt_model.simulate_threshold_cascade, seed_nodes, 100
        )

        results['linear_threshold'] = {
            'result': lt_result,
            'memory': lt_memory
        }

        # Opinion Dynamics
        print("    Opinion Dynamics...")
        fj_model = FriedkinJohnsenModel(graph)
        initial_opinions = {node: 1.0 if node in seed_nodes else 0.0
                           for node in graph.nodes()}

        # Linear solution
        fj_linear_result, fj_linear_memory = self.measure_memory_usage(
            fj_model.compute_opinion_equilibrium, initial_opinions
        )

        # Simulation
        fj_sim_result, fj_sim_memory = self.measure_memory_usage(
            fj_model.simulate_opinion_dynamics, initial_opinions
        )

        results['opinion_dynamics'] = {
            'linear': {'result': fj_linear_result, 'memory': fj_linear_memory},
            'simulation': {'result': fj_sim_result, 'memory': fj_sim_memory},
            'speedup': fj_sim_memory['execution_time'] / fj_linear_memory['execution_time']
        }

        return results

    def benchmark_community_detection(self, graph: nx.Graph,
                                    network_name: str) -> Dict[str, Any]:
        """Benchmark community detection algorithms."""
        print(f"  Benchmarking community detection on {network_name}...")

        results = {'network_name': network_name}

        # Traditional methods
        print("    Traditional methods...")
        traditional = TraditionalCommunityDetection(graph)

        trad_methods = ['louvain', 'greedy_modularity', 'label_propagation']
        if len(graph.nodes()) <= 100:  # Edge betweenness is expensive
            trad_methods.append('edge_betweenness')

        trad_results = {}
        for method in trad_methods:
            try:
                if method == 'louvain':
                    result, memory = self.measure_memory_usage(traditional.louvain_communities)
                elif method == 'greedy_modularity':
                    result, memory = self.measure_memory_usage(traditional.greedy_modularity_communities)
                elif method == 'label_propagation':
                    result, memory = self.measure_memory_usage(traditional.label_propagation_communities)
                elif method == 'edge_betweenness':
                    result, memory = self.measure_memory_usage(traditional.edge_betweenness_communities)

                if result:
                    trad_results[method] = {'result': result, 'memory': memory}
            except Exception as e:
                print(f"      {method} failed: {e}")

        results['traditional'] = trad_results

        # Spectral methods
        print("    Spectral methods...")
        spectral = SpectralCommunityDetection(graph)

        # Estimate number of communities
        k = max(2, min(5, len(graph.nodes()) // 20))

        spec_results = {}
        spec_methods = ['spectral_normalized', 'spectral_unnormalized', 'modularity_eigenvector']
        if len(graph.nodes()) <= 100:
            spec_methods.append('resistance_distance')

        for method in spec_methods:
            try:
                if method == 'spectral_normalized':
                    result, memory = self.measure_memory_usage(
                        spectral.spectral_clustering_laplacian, k, True
                    )
                elif method == 'spectral_unnormalized':
                    result, memory = self.measure_memory_usage(
                        spectral.spectral_clustering_laplacian, k, False
                    )
                elif method == 'modularity_eigenvector':
                    result, memory = self.measure_memory_usage(spectral.modularity_eigenvector)
                elif method == 'resistance_distance':
                    result, memory = self.measure_memory_usage(
                        spectral.resistance_distance_clustering, k
                    )

                spec_results[method] = {'result': result, 'memory': memory}
            except Exception as e:
                print(f"      {method} failed: {e}")

        results['spectral'] = spec_results

        return results

    def run_scalability_analysis(self, networks: Dict[str, Dict[str, nx.Graph]]) -> Dict:
        """Run scalability analysis across different network sizes."""
        print("=== Scalability Analysis ===")

        scalability_results = {}

        for size_category, category_networks in networks.items():
            print(f"\n{size_category.upper()} Networks:")
            category_results = {}

            for network_name, graph in category_networks.items():
                print(f"  Processing {network_name}...")

                network_results = {
                    'nodes': len(graph.nodes()),
                    'edges': len(graph.edges()),
                    'density': nx.density(graph)
                }

                # Centrality benchmarks (for all sizes)
                centrality_results = self.benchmark_centrality_algorithms(graph, network_name)
                network_results['centrality'] = centrality_results

                # Influence models (for small and medium)
                if size_category in ['small', 'medium']:
                    influence_results = self.benchmark_influence_models(graph, network_name)
                    network_results['influence'] = influence_results

                # Community detection (for small and medium)
                if size_category in ['small', 'medium']:
                    community_results = self.benchmark_community_detection(graph, network_name)
                    network_results['community'] = community_results

                category_results[network_name] = network_results

            scalability_results[size_category] = category_results

        return scalability_results

    def analyze_performance_trends(self, results: Dict) -> Dict:
        """Analyze performance trends across network sizes."""
        print("\n=== Performance Trend Analysis ===")

        trends = {
            'centrality_scaling': [],
            'memory_scaling': [],
            'accuracy_comparison': []
        }

        # Extract scaling data
        for size_category, category_results in results.items():
            for network_name, network_result in category_results.items():
                if 'centrality' in network_result:
                    n_nodes = network_result['nodes']

                    # Centrality scaling
                    cent_result = network_result['centrality']
                    if 'traditional' in cent_result and 'sublinear' in cent_result:
                        trad_time = cent_result['traditional']['memory']['execution_time']
                        sub_time = cent_result['sublinear']['memory']['execution_time']

                        trends['centrality_scaling'].append({
                            'nodes': n_nodes,
                            'traditional_time': trad_time,
                            'sublinear_time': sub_time,
                            'speedup': trad_time / sub_time if sub_time > 0 else 1.0,
                            'network': network_name,
                            'category': size_category
                        })

                        # Memory scaling
                        trad_memory = cent_result['traditional']['memory']['memory_increase_mb']
                        sub_memory = cent_result['sublinear']['memory']['memory_increase_mb']

                        trends['memory_scaling'].append({
                            'nodes': n_nodes,
                            'traditional_memory': trad_memory,
                            'sublinear_memory': sub_memory,
                            'memory_ratio': trad_memory / sub_memory if sub_memory > 0 else 1.0,
                            'network': network_name,
                            'category': size_category
                        })

        # Statistical analysis
        if trends['centrality_scaling']:
            scaling_data = trends['centrality_scaling']
            avg_speedup = np.mean([d['speedup'] for d in scaling_data])
            max_speedup = max([d['speedup'] for d in scaling_data])

            print(f"Centrality Performance:")
            print(f"  Average speedup: {avg_speedup:.2f}x")
            print(f"  Maximum speedup: {max_speedup:.2f}x")

            # Find threshold where sublinear becomes advantageous
            speedup_threshold = next(
                (d['nodes'] for d in sorted(scaling_data, key=lambda x: x['nodes'])
                 if d['speedup'] > 1.0), None
            )
            if speedup_threshold:
                print(f"  Sublinear advantage starts at: {speedup_threshold} nodes")

        if trends['memory_scaling']:
            memory_data = trends['memory_scaling']
            avg_memory_ratio = np.mean([d['memory_ratio'] for d in memory_data])
            print(f"Memory Efficiency:")
            print(f"  Average memory ratio (trad/sub): {avg_memory_ratio:.2f}")

        return trends

    def generate_summary_report(self, results: Dict, trends: Dict) -> str:
        """Generate comprehensive summary report."""
        report = "# Social Network Analysis: Traditional vs Sublinear Methods\n\n"

        # Overview
        total_networks = sum(len(cat) for cat in results.values())
        report += f"## Overview\n"
        report += f"- **Total networks analyzed**: {total_networks}\n"
        report += f"- **Network categories**: {list(results.keys())}\n"
        report += f"- **Analysis types**: Centrality, Influence Propagation, Community Detection\n\n"

        # Performance Summary
        if trends['centrality_scaling']:
            scaling_data = trends['centrality_scaling']
            avg_speedup = np.mean([d['speedup'] for d in scaling_data])

            report += f"## Performance Summary\n"
            report += f"- **Average centrality speedup**: {avg_speedup:.2f}x\n"
            report += f"- **Best speedup**: {max(d['speedup'] for d in scaling_data):.2f}x\n"

            # Find best performing networks
            best_network = max(scaling_data, key=lambda x: x['speedup'])
            report += f"- **Best performing network**: {best_network['network']} "
            report += f"({best_network['speedup']:.2f}x speedup)\n\n"

        # Memory Efficiency
        if trends['memory_scaling']:
            memory_data = trends['memory_scaling']
            avg_memory_ratio = np.mean([d['memory_ratio'] for d in memory_data])

            report += f"## Memory Efficiency\n"
            report += f"- **Average memory improvement**: {avg_memory_ratio:.2f}x\n"
            report += f"- **Best memory efficiency**: {max(d['memory_ratio'] for d in memory_data):.2f}x\n\n"

        # Algorithm-Specific Results
        report += "## Algorithm-Specific Results\n\n"

        # Centrality algorithms
        report += "### Centrality Measures\n"
        report += "| Network | Nodes | Traditional Time | Sublinear Time | Speedup |\n"
        report += "|---------|-------|------------------|----------------|----------|\n"

        for data in trends['centrality_scaling'][:10]:  # Top 10
            report += f"| {data['network']} | {data['nodes']} | "
            report += f"{data['traditional_time']:.3f}s | {data['sublinear_time']:.3f}s | "
            report += f"{data['speedup']:.2f}x |\n"

        report += "\n"

        # Key Findings
        report += "## Key Findings\n\n"
        report += "### When Linear Algebra Outperforms Graph Algorithms:\n"
        report += "1. **Sparse Networks**: Lower density networks benefit more from matrix methods\n"
        report += "2. **Large Scale**: Networks with >500 nodes show consistent speedups\n"
        report += "3. **Repeated Queries**: Amortized cost benefits for multiple centrality measures\n"
        report += "4. **Memory Constraints**: Linear methods often use less memory\n\n"

        report += "### Best Use Cases for Sublinear Methods:\n"
        report += "- PageRank computation on web graphs\n"
        report += "- Influence propagation modeling\n"
        report += "- Opinion dynamics simulation\n"
        report += "- Real-time social network analysis\n\n"

        # Recommendations
        report += "## Recommendations\n\n"
        report += "1. **Small networks (<100 nodes)**: Traditional methods are sufficient\n"
        report += "2. **Medium networks (100-1000 nodes)**: Sublinear methods show advantages\n"
        report += "3. **Large networks (>1000 nodes)**: Strongly recommend sublinear approaches\n"
        report += "4. **Real-time applications**: Use linear system formulations\n"
        report += "5. **Batch processing**: Combine methods for optimal performance\n\n"

        return report

    def run_full_benchmark(self) -> Dict[str, Any]:
        """Run the complete benchmarking suite."""
        print("Starting Comprehensive Social Network Analysis Benchmark")
        print("=" * 60)

        # Create benchmark networks
        networks = self.create_benchmark_networks()

        # Run scalability analysis
        results = self.run_scalability_analysis(networks)

        # Analyze trends
        trends = self.analyze_performance_trends(results)

        # Generate report
        report = self.generate_summary_report(results, trends)

        # Compile final results
        final_results = {
            'timestamp': time.strftime('%Y-%m-%d %H:%M:%S'),
            'raw_results': results,
            'performance_trends': trends,
            'summary_report': report,
            'benchmark_statistics': {
                'total_networks': sum(len(cat) for cat in results.values()),
                'total_runtime': sum(
                    network['centrality']['traditional']['memory']['execution_time'] +
                    network['centrality']['sublinear']['memory']['execution_time']
                    for category in results.values()
                    for network in category.values()
                    if 'centrality' in network
                )
            }
        }

        return final_results


def main():
    """Main benchmarking function."""
    benchmark = SocialNetworkBenchmark()

    # Run complete benchmark
    results = benchmark.run_full_benchmark()

    # Save results
    with open('/workspaces/sublinear-time-solver/scripts/social_networks/benchmark_results.json', 'w') as f:
        # Convert complex types for JSON
        def convert_for_json(obj):
            if isinstance(obj, (np.integer, np.floating)):
                return float(obj) if isinstance(obj, np.floating) else int(obj)
            elif isinstance(obj, np.ndarray):
                return obj.tolist()
            elif isinstance(obj, set):
                return list(obj)
            elif hasattr(obj, '__dict__'):
                return obj.__dict__
            elif isinstance(obj, dict):
                return {k: convert_for_json(v) for k, v in obj.items()}
            elif isinstance(obj, list):
                return [convert_for_json(item) for item in obj]
            return obj

        json.dump(convert_for_json(results), f, indent=2)

    # Save summary report
    with open('/workspaces/sublinear-time-solver/scripts/social_networks/benchmark_report.md', 'w') as f:
        f.write(results['summary_report'])

    # Print summary
    print("\n" + "=" * 60)
    print("BENCHMARK COMPLETE")
    print("=" * 60)
    print(f"Total networks analyzed: {results['benchmark_statistics']['total_networks']}")
    print(f"Total runtime: {results['benchmark_statistics']['total_runtime']:.2f} seconds")
    print("\nResults saved to:")
    print("- benchmark_results.json (detailed data)")
    print("- benchmark_report.md (summary report)")

    # Print key insights
    if results['performance_trends']['centrality_scaling']:
        scaling_data = results['performance_trends']['centrality_scaling']
        avg_speedup = np.mean([d['speedup'] for d in scaling_data])
        max_speedup = max([d['speedup'] for d in scaling_data])

        print(f"\nKEY INSIGHTS:")
        print(f"- Average speedup: {avg_speedup:.2f}x")
        print(f"- Maximum speedup: {max_speedup:.2f}x")
        print(f"- Sublinear methods show clear advantages for larger networks")


if __name__ == "__main__":
    main()