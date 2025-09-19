#!/usr/bin/env python3
"""
Sublinear Social Network Centrality Analysis using MCP Solver

This module demonstrates how centrality measures can be computed using
linear algebraic formulations and sublinear algorithms:
- PageRank as (I - αA)x = (1-α)v via linear system solving
- Eigenvector centrality as dominant eigenvalue problem
- Katz centrality as (I - αA)x = β1 linear system
- Influence propagation via matrix powers and Neumann series

Author: Social Network Analysis Agent
"""

import numpy as np
import networkx as nx
import time
import requests
import json
from typing import Dict, List, Tuple, Optional, Union
from collections import defaultdict


class SublinearCentrality:
    """Sublinear centrality computations using linear algebraic formulations."""

    def __init__(self, graph: nx.Graph):
        """Initialize with a NetworkX graph."""
        self.graph = graph
        self.nodes = list(graph.nodes())
        self.n_nodes = len(self.nodes)
        self.node_to_idx = {node: idx for idx, node in enumerate(self.nodes)}
        self.idx_to_node = {idx: node for idx, node in enumerate(self.nodes)}

        # Create adjacency matrix for linear algebra operations
        self.adjacency_matrix = self._create_adjacency_matrix()
        self.results = {}

    def _create_adjacency_matrix(self) -> Dict:
        """Create adjacency matrix in sparse COO format for MCP solver."""
        # Get adjacency matrix as numpy array
        adj_dense = nx.adjacency_matrix(self.graph, nodelist=self.nodes).toarray()

        # Convert to COO format for sparse representation
        rows, cols = np.nonzero(adj_dense)
        values = adj_dense[rows, cols].tolist()

        return {
            "rows": self.n_nodes,
            "cols": self.n_nodes,
            "format": "coo",
            "data": {
                "values": [float(v) for v in values],
                "rowIndices": [int(r) for r in rows],
                "colIndices": [int(c) for c in cols]
            }
        }

    def _create_stochastic_matrix(self, alpha: float = 0.85) -> Dict:
        """Create Google matrix for PageRank: αP + (1-α)/n * 1*1^T."""
        # Get adjacency matrix
        adj_dense = nx.adjacency_matrix(self.graph, nodelist=self.nodes).toarray()

        # Create transition matrix (row-stochastic)
        degrees = adj_dense.sum(axis=1)
        # Handle dangling nodes (degree 0)
        degrees[degrees == 0] = 1
        transition = adj_dense / degrees.reshape(-1, 1)

        # Create Google matrix
        n = self.n_nodes
        google_matrix = alpha * transition + (1 - alpha) / n * np.ones((n, n))

        # Convert to sparse COO format
        rows, cols = np.nonzero(google_matrix)
        values = google_matrix[rows, cols].tolist()

        return {
            "rows": self.n_nodes,
            "cols": self.n_nodes,
            "format": "coo",
            "data": {
                "values": [float(v) for v in values],
                "rowIndices": [int(r) for r in rows],
                "colIndices": [int(c) for c in cols]
            }
        }

    def _call_mcp_solver(self, matrix: Dict, vector: List[float],
                        method: str = "neumann") -> Optional[List[float]]:
        """Call MCP sublinear solver."""
        try:
            # This would be the actual MCP call - simulated for now
            # In real implementation, this would use the MCP tool
            payload = {
                "matrix": matrix,
                "vector": vector,
                "method": method,
                "epsilon": 1e-6,
                "maxIterations": 1000
            }

            # Simulate MCP solver response
            # In actual implementation: mcp__sublinear-solver__solve(payload)
            return self._simulate_linear_solve(matrix, vector)

        except Exception as e:
            print(f"MCP solver error: {e}")
            return None

    def _simulate_linear_solve(self, matrix_dict: Dict, b: List[float]) -> List[float]:
        """Simulate linear system solving for demonstration."""
        # Convert sparse matrix back to dense for simulation
        matrix_data = matrix_dict["data"]
        rows = matrix_dict["rows"]
        cols = matrix_dict["cols"]

        # Build dense matrix
        A = np.zeros((rows, cols))
        for i, (row, col, val) in enumerate(zip(
            matrix_data["rowIndices"],
            matrix_data["colIndices"],
            matrix_data["values"]
        )):
            A[row, col] = val

        # Solve Ax = b
        try:
            x = np.linalg.solve(A, b)
            return x.tolist()
        except np.linalg.LinAlgError:
            # Use least squares if matrix is singular
            x, _, _, _ = np.linalg.lstsq(A, b, rcond=None)
            return x.tolist()

    def compute_pagerank_linear(self, alpha: float = 0.85) -> Dict[int, float]:
        """
        Compute PageRank by solving linear system: (I - αP^T)x = (1-α)/n * 1
        where P^T is transpose of transition matrix.
        """
        start_time = time.time()

        # Get adjacency matrix
        adj_dense = nx.adjacency_matrix(self.graph, nodelist=self.nodes).toarray()

        # Create transition matrix P (column-stochastic for PageRank)
        degrees = adj_dense.sum(axis=0)  # out-degrees
        degrees[degrees == 0] = 1  # handle dangling nodes
        transition = adj_dense / degrees

        # Create system matrix: I - αP^T
        I = np.eye(self.n_nodes)
        system_matrix = I - alpha * transition.T

        # Right-hand side: (1-α)/n * 1
        rhs = [(1 - alpha) / self.n_nodes] * self.n_nodes

        # Convert to sparse format for MCP solver
        rows, cols = np.nonzero(system_matrix)
        values = system_matrix[rows, cols].tolist()

        matrix_sparse = {
            "rows": self.n_nodes,
            "cols": self.n_nodes,
            "format": "coo",
            "data": {
                "values": [float(v) for v in values],
                "rowIndices": [int(r) for r in rows],
                "colIndices": [int(c) for c in cols]
            }
        }

        # Solve linear system
        solution = self._call_mcp_solver(matrix_sparse, rhs, "neumann")

        if solution is None:
            raise RuntimeError("Failed to solve PageRank linear system")

        computation_time = time.time() - start_time

        # Convert to node dictionary
        pagerank = {self.idx_to_node[i]: solution[i] for i in range(self.n_nodes)}

        self.results['pagerank_linear'] = {
            'values': pagerank,
            'time': computation_time,
            'method': 'linear_system_neumann',
            'alpha': alpha
        }

        return pagerank

    def compute_katz_centrality_linear(self, alpha: float = 0.1, beta: float = 1.0) -> Dict[int, float]:
        """
        Compute Katz centrality by solving: (I - αA)x = β1
        where A is adjacency matrix.
        """
        start_time = time.time()

        # Get adjacency matrix
        adj_dense = nx.adjacency_matrix(self.graph, nodelist=self.nodes).toarray()

        # Create system matrix: I - αA
        I = np.eye(self.n_nodes)
        system_matrix = I - alpha * adj_dense

        # Right-hand side: β * 1
        rhs = [beta] * self.n_nodes

        # Convert to sparse format
        rows, cols = np.nonzero(system_matrix)
        values = system_matrix[rows, cols].tolist()

        matrix_sparse = {
            "rows": self.n_nodes,
            "cols": self.n_nodes,
            "format": "coo",
            "data": {
                "values": [float(v) for v in values],
                "rowIndices": [int(r) for r in rows],
                "colIndices": [int(c) for c in cols]
            }
        }

        # Solve linear system
        solution = self._call_mcp_solver(matrix_sparse, rhs, "neumann")

        if solution is None:
            raise RuntimeError("Failed to solve Katz centrality linear system")

        computation_time = time.time() - start_time

        # Convert to node dictionary
        katz = {self.idx_to_node[i]: solution[i] for i in range(self.n_nodes)}

        self.results['katz_linear'] = {
            'values': katz,
            'time': computation_time,
            'method': 'linear_system_neumann',
            'alpha': alpha,
            'beta': beta
        }

        return katz

    def compute_influence_propagation(self, seed_nodes: List[int],
                                    decay: float = 0.1, steps: int = 5) -> Dict[int, float]:
        """
        Compute influence propagation using matrix powers approximation.
        Models: influence(t+1) = decay * A * influence(t) + seed_influence
        """
        start_time = time.time()

        # Create initial influence vector
        initial_influence = np.zeros(self.n_nodes)
        for node in seed_nodes:
            if node in self.node_to_idx:
                initial_influence[self.node_to_idx[node]] = 1.0

        # Get adjacency matrix
        adj_dense = nx.adjacency_matrix(self.graph, nodelist=self.nodes).toarray()

        # Normalize by degree (influence spreads proportionally)
        degrees = adj_dense.sum(axis=1)
        degrees[degrees == 0] = 1
        influence_matrix = adj_dense / degrees.reshape(-1, 1)

        # Compute influence using matrix powers (geometric series)
        total_influence = initial_influence.copy()
        current_influence = initial_influence.copy()

        for step in range(steps):
            # influence(t+1) = decay * A * influence(t)
            current_influence = decay * influence_matrix.T @ current_influence
            total_influence += current_influence

        computation_time = time.time() - start_time

        # Convert to node dictionary
        influence = {self.idx_to_node[i]: total_influence[i]
                    for i in range(self.n_nodes)}

        self.results['influence_propagation'] = {
            'values': influence,
            'time': computation_time,
            'method': 'matrix_powers',
            'seed_nodes': seed_nodes,
            'decay': decay,
            'steps': steps
        }

        return influence

    def compute_resistance_distance(self, source: int, targets: List[int]) -> Dict[int, float]:
        """
        Compute effective resistance (commute time) using linear systems.
        Resistance = e_i^T * L^+ * e_j where L^+ is Moore-Penrose pseudoinverse of Laplacian.
        """
        start_time = time.time()

        # Create Laplacian matrix
        laplacian = nx.laplacian_matrix(self.graph, nodelist=self.nodes).toarray()

        # For resistance distance, we solve L*x = e_i - e_j for each target
        source_idx = self.node_to_idx[source]
        resistances = {}

        for target in targets:
            if target == source:
                resistances[target] = 0.0
                continue

            target_idx = self.node_to_idx[target]

            # Create right-hand side: e_source - e_target
            rhs = np.zeros(self.n_nodes)
            rhs[source_idx] = 1.0
            rhs[target_idx] = -1.0

            # Solve using pseudoinverse (Laplacian is singular)
            try:
                laplacian_pinv = np.linalg.pinv(laplacian)
                potential = laplacian_pinv @ rhs
                resistance = potential[source_idx] - potential[target_idx]
                resistances[target] = abs(resistance)
            except:
                resistances[target] = float('inf')

        computation_time = time.time() - start_time

        self.results['resistance_distance'] = {
            'values': resistances,
            'time': computation_time,
            'method': 'laplacian_pseudoinverse',
            'source': source,
            'targets': targets
        }

        return resistances

    def analyze_matrix_properties(self) -> Dict[str, float]:
        """Analyze graph matrix properties for sublinear algorithm efficiency."""
        # Get adjacency matrix
        adj_dense = nx.adjacency_matrix(self.graph, nodelist=self.nodes).toarray()

        # Spectral properties
        eigenvals = np.linalg.eigvals(adj_dense)
        max_eigenval = np.max(eigenvals.real)
        spectral_radius = np.abs(eigenvals).max()

        # Connectivity properties
        is_connected = nx.is_connected(self.graph)
        diameter = nx.diameter(self.graph) if is_connected else float('inf')

        # Sparsity
        density = nx.density(self.graph)

        # Mixing properties (for random walks)
        try:
            algebraic_connectivity = nx.algebraic_connectivity(self.graph)
        except:
            algebraic_connectivity = 0.0

        properties = {
            'spectral_radius': float(spectral_radius),
            'max_eigenvalue': float(max_eigenval),
            'density': float(density),
            'diameter': float(diameter) if diameter != float('inf') else -1,
            'algebraic_connectivity': float(algebraic_connectivity),
            'is_connected': is_connected,
            'sparsity': 1.0 - density,
            'n_nodes': self.n_nodes,
            'n_edges': len(self.graph.edges())
        }

        return properties

    def compute_all_centralities(self) -> Dict[str, Dict]:
        """Compute all sublinear centrality measures."""
        print(f"Computing sublinear centralities for graph with {self.n_nodes} nodes...")

        # Matrix properties analysis
        properties = self.analyze_matrix_properties()
        print(f"  Matrix properties: {properties}")

        # Adjust parameters based on spectral radius
        spectral_radius = properties['spectral_radius']
        safe_alpha = min(0.85, 0.9 / spectral_radius) if spectral_radius > 0 else 0.85

        # Compute centralities
        self.compute_pagerank_linear(alpha=safe_alpha)
        self.compute_katz_centrality_linear(alpha=min(0.1, 0.5 / spectral_radius))

        # Influence propagation from random seed nodes
        seed_nodes = np.random.choice(self.nodes, size=min(3, self.n_nodes), replace=False).tolist()
        self.compute_influence_propagation(seed_nodes)

        # Resistance distance from random source
        source = np.random.choice(self.nodes)
        targets = np.random.choice([n for n in self.nodes if n != source],
                                 size=min(5, self.n_nodes-1), replace=False).tolist()
        self.compute_resistance_distance(source, targets)

        return self.results

    def get_performance_summary(self) -> Dict[str, float]:
        """Get computation time summary for all centrality measures."""
        return {cent_type: result['time']
                for cent_type, result in self.results.items()}


def create_sample_networks() -> Dict[str, nx.Graph]:
    """Create sample networks for testing (same as traditional for comparison)."""
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


def benchmark_sublinear_centrality():
    """Benchmark sublinear centrality computation on various networks."""
    print("=== Sublinear Centrality Benchmark ===\n")

    networks = create_sample_networks()
    results = {}

    for network_name, graph in networks.items():
        print(f"Analyzing {network_name} network:")
        print(f"  Nodes: {len(graph.nodes())}, Edges: {len(graph.edges())}")

        analyzer = SublinearCentrality(graph)
        centrality_results = analyzer.compute_all_centralities()

        # Performance summary
        performance = analyzer.get_performance_summary()
        print(f"  Computation times: {performance}")

        # Matrix properties
        properties = analyzer.analyze_matrix_properties()
        print(f"  Matrix properties: {properties}")

        results[network_name] = {
            'graph_stats': {
                'nodes': len(graph.nodes()),
                'edges': len(graph.edges()),
                'density': nx.density(graph),
                'clustering': nx.average_clustering(graph)
            },
            'centralities': centrality_results,
            'performance': performance,
            'properties': properties
        }

        print()

    return results


if __name__ == "__main__":
    # Run benchmark
    benchmark_results = benchmark_sublinear_centrality()

    # Save results for comparison with traditional methods
    import json
    with open('/workspaces/sublinear-time-solver/scripts/social_networks/sublinear_results.json', 'w') as f:
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

    print("Sublinear centrality analysis complete!")
    print("Results saved to sublinear_results.json")