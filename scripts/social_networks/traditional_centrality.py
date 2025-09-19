#!/usr/bin/env python3
"""
Traditional Social Network Centrality Analysis using NetworkX

This module implements standard graph-theoretic centrality measures:
- PageRank centrality
- Betweenness centrality
- Closeness centrality
- Eigenvector centrality
- Katz centrality

Author: Social Network Analysis Agent
"""

import numpy as np
import networkx as nx
import time
from typing import Dict, List, Tuple, Optional
import matplotlib.pyplot as plt
from collections import defaultdict


class TraditionalCentrality:
    """Traditional graph-based centrality computations using NetworkX."""

    def __init__(self, graph: nx.Graph):
        """Initialize with a NetworkX graph."""
        self.graph = graph
        self.n_nodes = len(graph.nodes())
        self.results = {}

    def compute_pagerank(self, alpha: float = 0.85, max_iter: int = 1000,
                        tol: float = 1e-6) -> Dict[int, float]:
        """Compute PageRank centrality using NetworkX power iteration."""
        start_time = time.time()

        pagerank = nx.pagerank(self.graph, alpha=alpha, max_iter=max_iter, tol=tol)

        computation_time = time.time() - start_time
        self.results['pagerank'] = {
            'values': pagerank,
            'time': computation_time,
            'method': 'networkx_power_iteration'
        }

        return pagerank

    def compute_eigenvector_centrality(self, max_iter: int = 1000,
                                     tol: float = 1e-6) -> Dict[int, float]:
        """Compute eigenvector centrality using NetworkX."""
        start_time = time.time()

        try:
            eigen_centrality = nx.eigenvector_centrality(
                self.graph, max_iter=max_iter, tol=tol
            )
        except nx.PowerIterationFailedConvergence:
            # Use numpy method as fallback
            eigen_centrality = nx.eigenvector_centrality_numpy(self.graph)

        computation_time = time.time() - start_time
        self.results['eigenvector'] = {
            'values': eigen_centrality,
            'time': computation_time,
            'method': 'networkx_power_iteration'
        }

        return eigen_centrality

    def compute_katz_centrality(self, alpha: float = 0.1, beta: float = 1.0,
                               max_iter: int = 1000, tol: float = 1e-6) -> Dict[int, float]:
        """Compute Katz centrality using NetworkX."""
        start_time = time.time()

        # Auto-adjust alpha if it's too large
        try:
            katz = nx.katz_centrality(self.graph, alpha=alpha, beta=beta,
                                    max_iter=max_iter, tol=tol)
        except nx.PowerIterationFailedConvergence:
            # Use smaller alpha
            alpha = 0.05
            katz = nx.katz_centrality(self.graph, alpha=alpha, beta=beta,
                                    max_iter=max_iter, tol=tol)

        computation_time = time.time() - start_time
        self.results['katz'] = {
            'values': katz,
            'time': computation_time,
            'method': 'networkx_power_iteration',
            'alpha': alpha
        }

        return katz

    def compute_betweenness_centrality(self, normalized: bool = True) -> Dict[int, float]:
        """Compute betweenness centrality using NetworkX."""
        start_time = time.time()

        betweenness = nx.betweenness_centrality(self.graph, normalized=normalized)

        computation_time = time.time() - start_time
        self.results['betweenness'] = {
            'values': betweenness,
            'time': computation_time,
            'method': 'networkx_shortest_paths'
        }

        return betweenness

    def compute_closeness_centrality(self, normalized: bool = True) -> Dict[int, float]:
        """Compute closeness centrality using NetworkX."""
        start_time = time.time()

        closeness = nx.closeness_centrality(self.graph)

        computation_time = time.time() - start_time
        self.results['closeness'] = {
            'values': closeness,
            'time': computation_time,
            'method': 'networkx_shortest_paths'
        }

        return closeness

    def compute_all_centralities(self) -> Dict[str, Dict]:
        """Compute all centrality measures and return comprehensive results."""
        print(f"Computing traditional centralities for graph with {self.n_nodes} nodes...")

        # Compute all centrality measures
        self.compute_pagerank()
        self.compute_eigenvector_centrality()
        self.compute_katz_centrality()
        self.compute_betweenness_centrality()
        self.compute_closeness_centrality()

        return self.results

    def get_top_nodes(self, centrality_type: str, k: int = 10) -> List[Tuple[int, float]]:
        """Get top k nodes by centrality measure."""
        if centrality_type not in self.results:
            raise ValueError(f"Centrality type {centrality_type} not computed yet")

        values = self.results[centrality_type]['values']
        return sorted(values.items(), key=lambda x: x[1], reverse=True)[:k]

    def compare_centralities(self) -> Dict[str, float]:
        """Compare correlation between different centrality measures."""
        correlations = {}
        centrality_types = list(self.results.keys())

        for i, cent1 in enumerate(centrality_types):
            for cent2 in centrality_types[i+1:]:
                values1 = np.array(list(self.results[cent1]['values'].values()))
                values2 = np.array(list(self.results[cent2]['values'].values()))

                correlation = np.corrcoef(values1, values2)[0, 1]
                correlations[f"{cent1}_vs_{cent2}"] = correlation

        return correlations

    def get_performance_summary(self) -> Dict[str, float]:
        """Get computation time summary for all centrality measures."""
        return {cent_type: result['time']
                for cent_type, result in self.results.items()}


def create_sample_networks() -> Dict[str, nx.Graph]:
    """Create sample networks for testing."""
    networks = {}

    # Small Watts-Strogatz small-world network
    networks['small_world'] = nx.watts_strogatz_graph(100, 6, 0.3, seed=42)

    # Barabási-Albert scale-free network
    networks['scale_free'] = nx.barabasi_albert_graph(100, 3, seed=42)

    # Random geometric graph (social proximity)
    networks['geometric'] = nx.random_geometric_graph(100, 0.15, seed=42)

    # Complete graph (fully connected)
    networks['complete'] = nx.complete_graph(20)

    # Path graph (linear chain)
    networks['path'] = nx.path_graph(50)

    return networks


def benchmark_traditional_centrality():
    """Benchmark traditional centrality computation on various networks."""
    print("=== Traditional Centrality Benchmark ===\n")

    networks = create_sample_networks()
    results = {}

    for network_name, graph in networks.items():
        print(f"Analyzing {network_name} network:")
        print(f"  Nodes: {len(graph.nodes())}, Edges: {len(graph.edges())}")

        analyzer = TraditionalCentrality(graph)
        centrality_results = analyzer.compute_all_centralities()

        # Performance summary
        performance = analyzer.get_performance_summary()
        print(f"  Computation times: {performance}")

        # Top nodes by PageRank
        top_pagerank = analyzer.get_top_nodes('pagerank', 5)
        print(f"  Top 5 PageRank nodes: {top_pagerank}")

        # Centrality correlations
        correlations = analyzer.compare_centralities()
        print(f"  Centrality correlations: {correlations}")

        results[network_name] = {
            'graph_stats': {
                'nodes': len(graph.nodes()),
                'edges': len(graph.edges()),
                'density': nx.density(graph),
                'clustering': nx.average_clustering(graph)
            },
            'centralities': centrality_results,
            'performance': performance,
            'correlations': correlations
        }

        print()

    return results


if __name__ == "__main__":
    # Run benchmark
    benchmark_results = benchmark_traditional_centrality()

    # Save results for comparison with sublinear methods
    import json
    with open('/workspaces/sublinear-time-solver/scripts/social_networks/traditional_results.json', 'w') as f:
        # Convert numpy types for JSON serialization
        def convert_types(obj):
            if isinstance(obj, np.integer):
                return int(obj)
            elif isinstance(obj, np.floating):
                return float(obj)
            elif isinstance(obj, np.ndarray):
                return obj.tolist()
            elif isinstance(obj, dict):
                return {k: convert_types(v) for k, v in obj.items()}
            elif isinstance(obj, list):
                return [convert_types(item) for item in obj]
            return obj

        json.dump(convert_types(benchmark_results), f, indent=2)

    print("Traditional centrality analysis complete!")
    print("Results saved to traditional_results.json")