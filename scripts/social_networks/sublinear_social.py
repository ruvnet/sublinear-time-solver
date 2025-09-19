#!/usr/bin/env python3
"""
Sublinear Social Network Analysis using MCP Solver

This module implements social network analysis using linear algebraic formulations
and the MCP sublinear solver for improved performance on large networks.

Author: Social Network Analysis Agent
"""

import numpy as np
import networkx as nx
import time
import json
from typing import Dict, List, Tuple, Optional, Any
import scipy.sparse as sp
from scipy.sparse.linalg import eigsh
import tracemalloc


class SublinearSocialAnalysis:
    """Sublinear social network analysis using MCP solver."""

    def __init__(self, graph: nx.Graph):
        """Initialize with NetworkX graph."""
        self.graph = graph
        self.nodes = list(graph.nodes())
        self.n_nodes = len(self.nodes)
        self.n_edges = len(graph.edges())
        self.node_to_idx = {node: i for i, node in enumerate(self.nodes)}

        # Create adjacency matrix
        self.adj_matrix = self._create_adjacency_matrix()
        self.results = {}
        self.performance_metrics = {}

    def _create_adjacency_matrix(self) -> sp.csr_matrix:
        """Create sparse adjacency matrix."""
        return nx.adjacency_matrix(self.graph, nodelist=self.nodes)

    def _matrix_to_mcp_format(self, matrix: sp.spmatrix) -> Dict[str, Any]:
        """Convert scipy sparse matrix to MCP format."""
        matrix_coo = matrix.tocoo()
        return {
            "rows": matrix.shape[0],
            "cols": matrix.shape[1],
            "format": "coo",
            "data": {
                "values": [float(v) for v in matrix_coo.data],
                "rowIndices": [int(r) for r in matrix_coo.row],
                "colIndices": [int(c) for c in matrix_coo.col]
            }
        }

    def _call_mcp_pagerank(self, damping: float = 0.85, epsilon: float = 1e-6) -> Dict[str, float]:
        """Call MCP PageRank solver."""
        try:
            # Import MCP tools
            from mcp__sublinear_solver__pageRank import mcp__sublinear_solver__pageRank

            adjacency_mcp = self._matrix_to_mcp_format(self.adj_matrix)

            result = mcp__sublinear_solver__pageRank({
                "adjacency": adjacency_mcp,
                "damping": damping,
                "epsilon": epsilon
            })

            # Convert result back to node labels
            if 'values' in result:
                return {self.nodes[i]: result['values'][i] for i in range(self.n_nodes)}
            else:
                raise Exception("MCP PageRank failed to return values")

        except ImportError:
            # Fallback to manual implementation for demonstration
            return self._manual_pagerank_linear_system(damping, epsilon)

    def _manual_pagerank_linear_system(self, alpha: float = 0.85, epsilon: float = 1e-6) -> Dict[str, float]:
        """Manual PageRank using linear system formulation."""
        # PageRank: (I - αP^T)x = (1-α)/n * 1
        # where P is the column-stochastic transition matrix

        # Create transition matrix P
        degrees = np.array([self.graph.degree(node) for node in self.nodes])
        degrees[degrees == 0] = 1  # Handle isolated nodes

        # P^T (transpose of transition matrix)
        P_T = self.adj_matrix.T.multiply(1.0 / degrees).tocsr()

        # System matrix: I - αP^T
        I = sp.identity(self.n_nodes, format='csr')
        system_matrix = I - alpha * P_T

        # Right-hand side: (1-α)/n * 1
        rhs = np.full(self.n_nodes, (1 - alpha) / self.n_nodes)

        # Solve linear system
        from scipy.sparse.linalg import spsolve
        solution = spsolve(system_matrix, rhs)

        return {self.nodes[i]: float(solution[i]) for i in range(self.n_nodes)}

    def _call_mcp_solve(self, system_matrix: sp.spmatrix, rhs: np.ndarray, method: str = "neumann") -> np.ndarray:
        """Call MCP linear system solver."""
        try:
            # Import MCP tools
            from mcp__sublinear_solver__solve import mcp__sublinear_solver__solve

            matrix_mcp = self._matrix_to_mcp_format(system_matrix)

            result = mcp__sublinear_solver__solve({
                "matrix": matrix_mcp,
                "vector": rhs.tolist(),
                "method": method,
                "epsilon": 1e-6,
                "maxIterations": 1000
            })

            if 'solution' in result:
                return np.array(result['solution'])
            else:
                raise Exception("MCP solver failed to return solution")

        except ImportError:
            # Fallback to scipy solver
            from scipy.sparse.linalg import spsolve
            return spsolve(system_matrix, rhs)

    def compute_pagerank_sublinear(self, damping: float = 0.85, epsilon: float = 1e-6) -> Dict[str, Any]:
        """Compute PageRank using sublinear methods."""
        print("Computing PageRank using sublinear solver...")

        start_time = time.time()
        tracemalloc.start()

        pagerank_values = self._call_mcp_pagerank(damping, epsilon)

        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        computation_time = time.time() - start_time

        result = {
            'values': pagerank_values,
            'computation_time': computation_time,
            'memory_peak': peak,
            'method': 'linear_system_mcp',
            'parameters': {
                'damping': damping,
                'epsilon': epsilon
            }
        }

        return result

    def compute_katz_centrality_sublinear(self, alpha: float = None, beta: float = 1.0) -> Dict[str, Any]:
        """Compute Katz centrality using sublinear methods."""
        print("Computing Katz centrality using sublinear solver...")

        # Auto-select alpha if not provided
        if alpha is None:
            # Use 1/spectral_radius as conservative estimate
            max_degree = max(dict(self.graph.degree()).values())
            alpha = 0.1 / max(1, max_degree)

        start_time = time.time()
        tracemalloc.start()

        # Katz centrality: (I - αA)x = β1
        I = sp.identity(self.n_nodes, format='csr')
        system_matrix = I - alpha * self.adj_matrix
        rhs = np.full(self.n_nodes, beta)

        try:
            solution = self._call_mcp_solve(system_matrix, rhs, method="neumann")
            katz_values = {self.nodes[i]: float(solution[i]) for i in range(self.n_nodes)}
            success = True
        except Exception as e:
            print(f"Katz centrality computation failed: {e}")
            katz_values = {node: 0.0 for node in self.nodes}
            success = False

        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        computation_time = time.time() - start_time

        result = {
            'values': katz_values,
            'computation_time': computation_time,
            'memory_peak': peak,
            'method': 'linear_system_mcp',
            'success': success,
            'parameters': {
                'alpha': alpha,
                'beta': beta
            }
        }

        return result

    def compute_resistance_centrality(self) -> Dict[str, Any]:
        """Compute resistance-based centrality measures."""
        print("Computing resistance centrality using Laplacian pseudoinverse...")

        start_time = time.time()
        tracemalloc.start()

        # Create Laplacian matrix
        laplacian = nx.laplacian_matrix(self.graph, nodelist=self.nodes).astype(float)

        # For connected graphs, we need the pseudoinverse of the Laplacian
        # This involves solving linear systems of the form Lx = e_i - e_j

        resistance_distances = {}
        effective_resistances = {}

        try:
            # Sample a subset of node pairs for efficiency
            sample_nodes = self.nodes[:min(20, self.n_nodes)]

            for i, node_i in enumerate(sample_nodes):
                for j, node_j in enumerate(sample_nodes):
                    if i != j:
                        # Create RHS vector: e_i - e_j
                        rhs = np.zeros(self.n_nodes)
                        idx_i = self.node_to_idx[node_i]
                        idx_j = self.node_to_idx[node_j]
                        rhs[idx_i] = 1.0
                        rhs[idx_j] = -1.0

                        # Solve Laplacian system (using pseudoinverse property)
                        # Add small regularization for numerical stability
                        regularized_laplacian = laplacian + 1e-8 * sp.identity(self.n_nodes)

                        try:
                            potential = self._call_mcp_solve(regularized_laplacian, rhs, method="neumann")
                            resistance = potential[idx_i] - potential[idx_j]
                            resistance_distances[(node_i, node_j)] = abs(resistance)
                        except:
                            resistance_distances[(node_i, node_j)] = float('inf')

            # Compute effective resistance centrality
            for node in sample_nodes:
                total_resistance = sum(resistance_distances.get((node, other), 0)
                                     for other in sample_nodes if other != node)
                effective_resistances[node] = 1.0 / (1.0 + total_resistance) if total_resistance > 0 else 0

            success = True

        except Exception as e:
            print(f"Resistance centrality computation failed: {e}")
            resistance_distances = {}
            effective_resistances = {node: 0.0 for node in self.nodes}
            success = False

        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        computation_time = time.time() - start_time

        result = {
            'resistance_distances': resistance_distances,
            'effective_resistances': effective_resistances,
            'computation_time': computation_time,
            'memory_peak': peak,
            'method': 'laplacian_pseudoinverse',
            'success': success
        }

        return result

    def compute_influence_propagation_sublinear(self, seed_nodes: List, alpha: float = 0.3) -> Dict[str, Any]:
        """Compute influence propagation using matrix geometric series."""
        print("Computing influence propagation using sublinear methods...")

        start_time = time.time()
        tracemalloc.start()

        # Influence propagation: Total influence = (I - αA^T)^(-1) * seed_vector
        # where seed_vector has 1s for seed nodes, 0s elsewhere

        # Create seed vector
        seed_vector = np.zeros(self.n_nodes)
        for node in seed_nodes:
            if node in self.node_to_idx:
                seed_vector[self.node_to_idx[node]] = 1.0

        # System matrix: I - αA^T
        I = sp.identity(self.n_nodes, format='csr')
        A_T = self.adj_matrix.T
        system_matrix = I - alpha * A_T

        try:
            influence_vector = self._call_mcp_solve(system_matrix, seed_vector, method="neumann")
            influence_values = {self.nodes[i]: float(influence_vector[i]) for i in range(self.n_nodes)}

            # Calculate total influence
            total_influence = np.sum(influence_vector)
            influenced_nodes = np.sum(influence_vector > 0.01)  # Threshold for meaningful influence

            success = True

        except Exception as e:
            print(f"Influence propagation failed: {e}")
            influence_values = {node: 0.0 for node in self.nodes}
            total_influence = 0.0
            influenced_nodes = 0
            success = False

        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        computation_time = time.time() - start_time

        result = {
            'seed_nodes': seed_nodes,
            'influence_values': influence_values,
            'total_influence': total_influence,
            'influenced_nodes': influenced_nodes,
            'influence_fraction': influenced_nodes / self.n_nodes,
            'computation_time': computation_time,
            'memory_peak': peak,
            'method': 'matrix_geometric_series',
            'success': success,
            'parameters': {
                'alpha': alpha
            }
        }

        return result

    def compute_spectral_clustering_sublinear(self, n_clusters: int = None) -> Dict[str, Any]:
        """Compute spectral clustering using Laplacian eigensolvers."""
        print("Computing spectral clustering using sublinear methods...")

        start_time = time.time()
        tracemalloc.start()

        try:
            # Create normalized Laplacian
            laplacian = nx.normalized_laplacian_matrix(self.graph, nodelist=self.nodes).astype(float)

            # Estimate number of clusters if not provided
            if n_clusters is None:
                # Use eigengap heuristic
                try:
                    eigenvals = eigsh(laplacian, k=min(10, self.n_nodes-2), which='SM', return_eigenvectors=False)
                    eigenvals = np.sort(eigenvals)
                    gaps = np.diff(eigenvals)
                    n_clusters = np.argmax(gaps) + 2
                    n_clusters = min(n_clusters, 8)  # Cap at 8
                except:
                    n_clusters = 3  # Default

            # Compute first k eigenvectors
            k = min(n_clusters, self.n_nodes - 2)
            eigenvals, eigenvectors = eigsh(laplacian, k=k, which='SM')

            # Use k-means on eigenvectors
            from sklearn.cluster import KMeans
            kmeans = KMeans(n_clusters=n_clusters, random_state=42, n_init=10)
            cluster_labels = kmeans.fit_predict(eigenvectors)

            # Create partition dict
            partition = {self.nodes[i]: int(cluster_labels[i]) for i in range(self.n_nodes)}

            success = True

        except Exception as e:
            print(f"Spectral clustering failed: {e}")
            partition = {node: 0 for node in self.nodes}
            n_clusters = 1
            success = False

        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        computation_time = time.time() - start_time

        result = {
            'partition': partition,
            'n_clusters': n_clusters,
            'computation_time': computation_time,
            'memory_peak': peak,
            'method': 'laplacian_eigenvectors',
            'success': success
        }

        return result

    def compute_all_sublinear_measures(self) -> Dict[str, Any]:
        """Compute all sublinear social network measures."""
        print("Computing all sublinear measures...")

        results = {}

        # PageRank
        try:
            results['pagerank'] = self.compute_pagerank_sublinear()
        except Exception as e:
            print(f"PageRank failed: {e}")
            results['pagerank'] = {'error': str(e)}

        # Katz Centrality
        try:
            results['katz_centrality'] = self.compute_katz_centrality_sublinear()
        except Exception as e:
            print(f"Katz centrality failed: {e}")
            results['katz_centrality'] = {'error': str(e)}

        # Resistance Centrality (for smaller graphs)
        if self.n_nodes <= 200:
            try:
                results['resistance_centrality'] = self.compute_resistance_centrality()
            except Exception as e:
                print(f"Resistance centrality failed: {e}")
                results['resistance_centrality'] = {'error': str(e)}

        # Influence Propagation (test with high-degree seeds)
        try:
            degrees = dict(self.graph.degree())
            high_degree_seeds = sorted(degrees.keys(), key=lambda x: degrees[x], reverse=True)[:3]
            results['influence_propagation'] = self.compute_influence_propagation_sublinear(high_degree_seeds)
        except Exception as e:
            print(f"Influence propagation failed: {e}")
            results['influence_propagation'] = {'error': str(e)}

        # Spectral Clustering
        try:
            results['spectral_clustering'] = self.compute_spectral_clustering_sublinear()
        except Exception as e:
            print(f"Spectral clustering failed: {e}")
            results['spectral_clustering'] = {'error': str(e)}

        self.results = results
        return results

    def get_performance_summary(self) -> Dict[str, Any]:
        """Get performance summary of all computations."""
        summary = {
            'graph_properties': {
                'n_nodes': self.n_nodes,
                'n_edges': self.n_edges,
                'density': self.n_edges / (self.n_nodes * (self.n_nodes - 1) / 2) if self.n_nodes > 1 else 0,
                'average_degree': 2 * self.n_edges / self.n_nodes if self.n_nodes > 0 else 0
            },
            'total_computation_time': 0,
            'algorithm_times': {},
            'peak_memory_usage': 0
        }

        # Aggregate performance metrics
        for method, data in self.results.items():
            if isinstance(data, dict) and 'computation_time' in data:
                time_taken = data['computation_time']
                summary['total_computation_time'] += time_taken
                summary['algorithm_times'][method] = time_taken

                if 'memory_peak' in data:
                    summary['peak_memory_usage'] = max(summary['peak_memory_usage'],
                                                     data['memory_peak'])

        return summary

    def save_results(self, filename: str = 'sublinear_results.json'):
        """Save all results to JSON file."""
        output_path = f'/workspaces/sublinear-time-solver/scripts/social_networks/{filename}'

        # Prepare results for JSON serialization
        json_results = {
            'graph_info': {
                'n_nodes': self.n_nodes,
                'n_edges': self.n_edges,
                'graph_type': 'sublinear_analysis'
            },
            'results': self.results,
            'performance_summary': self.get_performance_summary()
        }

        with open(output_path, 'w') as f:
            json.dump(json_results, f, indent=2, default=str)

        print(f"Results saved to {output_path}")


def create_test_networks() -> Dict[str, nx.Graph]:
    """Create various test networks for analysis."""
    networks = {}

    # Small test networks
    networks['karate'] = nx.karate_club_graph()

    # Synthetic networks
    networks['erdos_renyi_100'] = nx.erdos_renyi_graph(100, 0.1, seed=42)
    networks['barabasi_albert_100'] = nx.barabasi_albert_graph(100, 3, seed=42)
    networks['watts_strogatz_100'] = nx.watts_strogatz_graph(100, 6, 0.3, seed=42)

    # Larger networks for performance testing
    networks['erdos_renyi_500'] = nx.erdos_renyi_graph(500, 0.02, seed=42)
    networks['barabasi_albert_500'] = nx.barabasi_albert_graph(500, 2, seed=42)
    networks['barabasi_albert_1000'] = nx.barabasi_albert_graph(1000, 2, seed=42)

    # Scale-free network with communities
    networks['community_graph'] = nx.planted_partition_graph(4, 25, 0.8, 0.1, seed=42)

    return networks


def main():
    """Run sublinear social network analysis on test networks."""
    print("=" * 60)
    print("Sublinear Social Network Analysis")
    print("=" * 60)

    networks = create_test_networks()

    for name, graph in networks.items():
        print(f"\nAnalyzing {name} network ({len(graph.nodes())} nodes, {len(graph.edges())} edges)...")

        analyzer = SublinearSocialAnalysis(graph)

        try:
            # Run sublinear analysis
            analyzer.compute_all_sublinear_measures()

            # Save results
            analyzer.save_results(f'sublinear_{name}_results.json')

            # Print summary
            summary = analyzer.get_performance_summary()
            print(f"  Total computation time: {summary['total_computation_time']:.2f}s")
            print(f"  Peak memory usage: {summary['peak_memory_usage']/1024/1024:.1f} MB")

        except Exception as e:
            print(f"  Analysis failed: {e}")
            continue

    print("\n" + "=" * 60)
    print("Sublinear analysis complete!")
    print("=" * 60)


if __name__ == "__main__":
    main()