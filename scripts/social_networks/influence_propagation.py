#!/usr/bin/env python3
"""
Influence Propagation and Information Cascade Modeling

This module implements various influence propagation models including Independent Cascade,
Linear Threshold, and continuous influence models using both simulation and linear algebraic
approaches with sublinear solvers.

Author: Social Network Analysis Agent
"""

import numpy as np
import networkx as nx
import time
import json
from typing import Dict, List, Tuple, Optional, Any, Set
import scipy.sparse as sp
from scipy.sparse.linalg import spsolve
import matplotlib.pyplot as plt
import seaborn as sns
import tracemalloc
from collections import defaultdict, deque


class InfluencePropagationAnalyzer:
    """Comprehensive influence propagation analysis using multiple models."""

    def __init__(self, graph: nx.Graph):
        """Initialize with NetworkX graph."""
        self.graph = graph
        self.nodes = list(graph.nodes())
        self.n_nodes = len(self.nodes)
        self.n_edges = len(graph.edges())
        self.node_to_idx = {node: i for i, node in enumerate(self.nodes)}

        # Create adjacency matrix
        self.adj_matrix = nx.adjacency_matrix(self.graph, nodelist=self.nodes)

        self.results = {}

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
            return spsolve(system_matrix, rhs)

    def independent_cascade_simulation(self, seed_nodes: List,
                                     propagation_probability: float = 0.1,
                                     max_steps: int = 10,
                                     n_simulations: int = 1000) -> Dict[str, Any]:
        """
        Simulate Independent Cascade Model using Monte Carlo.

        In IC model, each newly activated node gets one chance to activate
        each inactive neighbor with probability p.
        """
        print(f"Running Independent Cascade simulation (p={propagation_probability})...")

        start_time = time.time()
        tracemalloc.start()

        simulation_results = []

        for sim in range(n_simulations):
            # Initialize active set
            active = set(seed_nodes)
            newly_active = set(seed_nodes)
            step = 0
            activation_history = [set(seed_nodes)]

            while newly_active and step < max_steps:
                next_active = set()

                for node in newly_active:
                    for neighbor in self.graph.neighbors(node):
                        if neighbor not in active and np.random.random() < propagation_probability:
                            next_active.add(neighbor)

                active.update(next_active)
                newly_active = next_active
                activation_history.append(active.copy())
                step += 1

            simulation_results.append({
                'final_active': active,
                'total_influenced': len(active),
                'steps': step,
                'activation_history': activation_history
            })

        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        computation_time = time.time() - start_time

        # Aggregate statistics
        influences = [result['total_influenced'] for result in simulation_results]
        steps = [result['steps'] for result in simulation_results]

        # Calculate activation probabilities for each node
        activation_counts = defaultdict(int)
        for result in simulation_results:
            for node in result['final_active']:
                activation_counts[node] += 1

        activation_probabilities = {
            node: activation_counts[node] / n_simulations
            for node in self.nodes
        }

        result = {
            'seed_nodes': seed_nodes,
            'propagation_probability': propagation_probability,
            'mean_influence': np.mean(influences),
            'std_influence': np.std(influences),
            'median_influence': np.median(influences),
            'max_influence': max(influences),
            'min_influence': min(influences),
            'mean_steps': np.mean(steps),
            'activation_probabilities': activation_probabilities,
            'influence_distribution': influences,
            'computation_time': computation_time,
            'memory_peak': peak,
            'n_simulations': n_simulations,
            'method': 'monte_carlo_ic'
        }

        return result

    def linear_threshold_simulation(self, seed_nodes: List,
                                  thresholds: Optional[Dict] = None,
                                  max_steps: int = 10,
                                  n_simulations: int = 500) -> Dict[str, Any]:
        """
        Simulate Linear Threshold Model using Monte Carlo.

        In LT model, each node has a threshold and becomes active when
        the weighted sum of active neighbors exceeds the threshold.
        """
        print("Running Linear Threshold simulation...")

        start_time = time.time()
        tracemalloc.start()

        simulation_results = []

        for sim in range(n_simulations):
            # Generate random thresholds if not provided
            if thresholds is None:
                node_thresholds = {node: np.random.uniform(0.1, 0.9) for node in self.nodes}
            else:
                node_thresholds = thresholds

            # Initialize
            active = set(seed_nodes)
            step = 0
            activation_history = [set(seed_nodes)]

            while step < max_steps:
                newly_active = set()

                for node in self.nodes:
                    if node not in active:
                        # Calculate influence from active neighbors
                        influence = 0.0
                        degree = self.graph.degree(node)

                        if degree > 0:
                            for neighbor in self.graph.neighbors(node):
                                if neighbor in active:
                                    # Weight by inverse degree (uniform influence)
                                    influence += 1.0 / degree

                        # Check if threshold is exceeded
                        if influence >= node_thresholds[node]:
                            newly_active.add(node)

                if not newly_active:
                    break

                active.update(newly_active)
                activation_history.append(active.copy())
                step += 1

            simulation_results.append({
                'final_active': active,
                'total_influenced': len(active),
                'steps': step,
                'thresholds': node_thresholds,
                'activation_history': activation_history
            })

        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        computation_time = time.time() - start_time

        # Aggregate statistics
        influences = [result['total_influenced'] for result in simulation_results]
        steps = [result['steps'] for result in simulation_results]

        # Calculate activation probabilities
        activation_counts = defaultdict(int)
        for result in simulation_results:
            for node in result['final_active']:
                activation_counts[node] += 1

        activation_probabilities = {
            node: activation_counts[node] / n_simulations
            for node in self.nodes
        }

        result = {
            'seed_nodes': seed_nodes,
            'mean_influence': np.mean(influences),
            'std_influence': np.std(influences),
            'median_influence': np.median(influences),
            'max_influence': max(influences),
            'min_influence': min(influences),
            'mean_steps': np.mean(steps),
            'activation_probabilities': activation_probabilities,
            'influence_distribution': influences,
            'computation_time': computation_time,
            'memory_peak': peak,
            'n_simulations': n_simulations,
            'method': 'monte_carlo_lt'
        }

        return result

    def continuous_influence_sublinear(self, seed_nodes: List,
                                     decay_factor: float = 0.3,
                                     method: str = "neumann") -> Dict[str, Any]:
        """
        Compute continuous influence using linear algebraic formulation.

        Solves: Total Influence = (I - αA^T)^(-1) * seed_vector
        where α is the decay factor and A^T is the transpose adjacency matrix.
        """
        print(f"Computing continuous influence using sublinear solver (α={decay_factor})...")

        start_time = time.time()
        tracemalloc.start()

        # Create seed vector
        seed_vector = np.zeros(self.n_nodes)
        for node in seed_nodes:
            if node in self.node_to_idx:
                seed_vector[self.node_to_idx[node]] = 1.0

        # System matrix: I - αA^T
        I = sp.identity(self.n_nodes, format='csr')
        A_T = self.adj_matrix.T.tocsr()
        system_matrix = I - decay_factor * A_T

        try:
            # Solve using MCP
            influence_vector = self._call_mcp_solve(system_matrix, seed_vector, method)

            # Convert to dict
            influence_values = {self.nodes[i]: float(influence_vector[i]) for i in range(self.n_nodes)}

            # Calculate metrics
            total_influence = np.sum(influence_vector)
            influenced_nodes = np.sum(influence_vector > 0.01)  # Threshold for meaningful influence
            max_influence = np.max(influence_vector)
            influence_variance = np.var(influence_vector)

            success = True

        except Exception as e:
            print(f"Continuous influence computation failed: {e}")
            influence_values = {node: 0.0 for node in self.nodes}
            total_influence = 0.0
            influenced_nodes = 0
            max_influence = 0.0
            influence_variance = 0.0
            success = False

        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        computation_time = time.time() - start_time

        result = {
            'seed_nodes': seed_nodes,
            'decay_factor': decay_factor,
            'influence_values': influence_values,
            'total_influence': float(total_influence),
            'influenced_nodes': int(influenced_nodes),
            'max_influence': float(max_influence),
            'influence_variance': float(influence_variance),
            'influence_efficiency': float(total_influence / len(seed_nodes)) if seed_nodes else 0.0,
            'computation_time': computation_time,
            'memory_peak': peak,
            'method': f'linear_system_{method}',
            'success': success
        }

        return result

    def multi_step_influence_analysis(self, seed_nodes: List,
                                    max_steps: int = 5,
                                    decay_factor: float = 0.3) -> Dict[str, Any]:
        """
        Analyze multi-step influence propagation using matrix powers.

        Computes influence at each step k using (αA^T)^k for k = 1, 2, ..., max_steps
        """
        print(f"Computing multi-step influence analysis (k=1 to {max_steps})...")

        start_time = time.time()
        tracemalloc.start()

        # Create seed vector
        seed_vector = np.zeros(self.n_nodes)
        for node in seed_nodes:
            if node in self.node_to_idx:
                seed_vector[self.node_to_idx[node]] = 1.0

        # Adjacency matrix with decay
        A_T_scaled = decay_factor * self.adj_matrix.T.tocsr()

        step_influences = {}
        cumulative_influence = seed_vector.copy()

        try:
            # Current influence matrix (starts as identity)
            current_matrix = sp.identity(self.n_nodes, format='csr')

            for step in range(1, max_steps + 1):
                # Multiply by A^T to get next step
                current_matrix = current_matrix @ A_T_scaled

                # Compute influence at this step
                step_influence = current_matrix @ seed_vector

                # Store results
                step_influences[step] = {
                    'influence_vector': step_influence.tolist(),
                    'total_influence': float(np.sum(step_influence)),
                    'max_influence': float(np.max(step_influence)),
                    'influenced_nodes': int(np.sum(step_influence > 0.01))
                }

                # Update cumulative influence
                cumulative_influence += step_influence

            # Calculate final metrics
            final_influence_values = {self.nodes[i]: float(cumulative_influence[i]) for i in range(self.n_nodes)}

            success = True

        except Exception as e:
            print(f"Multi-step influence analysis failed: {e}")
            step_influences = {}
            final_influence_values = {node: 0.0 for node in self.nodes}
            success = False

        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        computation_time = time.time() - start_time

        result = {
            'seed_nodes': seed_nodes,
            'decay_factor': decay_factor,
            'max_steps': max_steps,
            'step_influences': step_influences,
            'cumulative_influence': final_influence_values,
            'total_cumulative_influence': float(np.sum(cumulative_influence)),
            'computation_time': computation_time,
            'memory_peak': peak,
            'method': 'matrix_powers',
            'success': success
        }

        return result

    def influence_maximization_analysis(self, budget: int = 3,
                                      methods: List[str] = ['degree', 'pagerank', 'random']) -> Dict[str, Any]:
        """
        Compare different seed selection strategies for influence maximization.

        Args:
            budget: Number of seed nodes to select
            methods: List of seed selection methods to compare
        """
        print(f"Running influence maximization analysis (budget={budget})...")

        results = {}

        # Precompute node rankings for different methods
        rankings = {}

        if 'degree' in methods:
            degree_centrality = dict(self.graph.degree())
            rankings['degree'] = sorted(degree_centrality.keys(),
                                      key=lambda x: degree_centrality[x], reverse=True)

        if 'pagerank' in methods:
            pagerank = nx.pagerank(self.graph, alpha=0.85)
            rankings['pagerank'] = sorted(pagerank.keys(),
                                        key=lambda x: pagerank[x], reverse=True)

        if 'betweenness' in methods:
            betweenness = nx.betweenness_centrality(self.graph)
            rankings['betweenness'] = sorted(betweenness.keys(),
                                           key=lambda x: betweenness[x], reverse=True)

        if 'closeness' in methods:
            closeness = nx.closeness_centrality(self.graph)
            rankings['closeness'] = sorted(closeness.keys(),
                                         key=lambda x: closeness[x], reverse=True)

        if 'random' in methods:
            rankings['random'] = list(np.random.choice(self.nodes, self.n_nodes, replace=False))

        # Test each method
        for method in methods:
            print(f"  Testing {method} seed selection...")

            if method in rankings:
                seeds = rankings[method][:budget]
            else:
                continue

            # Test with multiple influence models
            method_results = {}

            try:
                # Independent Cascade
                ic_result = self.independent_cascade_simulation(
                    seeds, propagation_probability=0.1, n_simulations=200
                )
                method_results['independent_cascade'] = {
                    'mean_influence': ic_result['mean_influence'],
                    'std_influence': ic_result['std_influence'],
                    'computation_time': ic_result['computation_time']
                }

                # Linear Threshold
                lt_result = self.linear_threshold_simulation(
                    seeds, n_simulations=100
                )
                method_results['linear_threshold'] = {
                    'mean_influence': lt_result['mean_influence'],
                    'std_influence': lt_result['std_influence'],
                    'computation_time': lt_result['computation_time']
                }

                # Continuous Influence
                cont_result = self.continuous_influence_sublinear(
                    seeds, decay_factor=0.3
                )
                method_results['continuous_influence'] = {
                    'total_influence': cont_result['total_influence'],
                    'influenced_nodes': cont_result['influenced_nodes'],
                    'computation_time': cont_result['computation_time']
                }

                results[method] = {
                    'seeds': seeds,
                    'results': method_results
                }

            except Exception as e:
                print(f"    {method} failed: {e}")
                results[method] = {'error': str(e)}

        return results

    def viral_cascade_analysis(self, seed_nodes: List,
                             cascade_threshold: float = 0.05,
                             viral_threshold: float = 0.1) -> Dict[str, Any]:
        """
        Analyze viral cascade potential and cascade size distribution.

        Args:
            seed_nodes: Initial seed nodes
            cascade_threshold: Minimum influence to be considered part of cascade
            viral_threshold: Threshold for considering cascade as "viral"
        """
        print("Analyzing viral cascade potential...")

        start_time = time.time()

        results = {}

        # Test different propagation probabilities
        probabilities = [0.01, 0.02, 0.05, 0.1, 0.15, 0.2, 0.3]

        for prob in probabilities:
            print(f"  Testing p={prob}...")

            # Run IC simulation
            ic_result = self.independent_cascade_simulation(
                seed_nodes, propagation_probability=prob, n_simulations=500
            )

            # Analyze cascade sizes
            cascade_sizes = ic_result['influence_distribution']
            viral_cascades = [size for size in cascade_sizes if size / self.n_nodes >= viral_threshold]

            results[f'p_{prob}'] = {
                'propagation_probability': prob,
                'mean_cascade_size': ic_result['mean_influence'],
                'cascade_size_distribution': cascade_sizes,
                'viral_cascade_probability': len(viral_cascades) / len(cascade_sizes),
                'largest_cascade': max(cascade_sizes),
                'cascade_variance': ic_result['std_influence'] ** 2
            }

        # Find critical probability (where viral cascades become likely)
        viral_probs = [results[f'p_{p}']['viral_cascade_probability'] for p in probabilities]
        critical_prob_idx = next((i for i, vp in enumerate(viral_probs) if vp >= 0.1), len(probabilities) - 1)
        critical_probability = probabilities[critical_prob_idx]

        total_time = time.time() - start_time

        summary = {
            'seed_nodes': seed_nodes,
            'cascade_threshold': cascade_threshold,
            'viral_threshold': viral_threshold,
            'critical_probability': critical_probability,
            'probability_results': results,
            'total_analysis_time': total_time
        }

        return summary

    def save_results(self, results: Dict[str, Any], filename: str = 'influence_propagation_results.json'):
        """Save results to JSON file."""
        output_path = f'/workspaces/sublinear-time-solver/scripts/social_networks/{filename}'

        # Add metadata
        output_data = {
            'graph_info': {
                'n_nodes': self.n_nodes,
                'n_edges': self.n_edges,
                'average_degree': 2 * self.n_edges / self.n_nodes if self.n_nodes > 0 else 0,
                'density': self.n_edges / (self.n_nodes * (self.n_nodes - 1) / 2) if self.n_nodes > 1 else 0
            },
            'analysis_type': 'influence_propagation',
            'results': results
        }

        with open(output_path, 'w') as f:
            json.dump(output_data, f, indent=2, default=str)

        print(f"Results saved to {output_path}")


def create_test_networks() -> Dict[str, nx.Graph]:
    """Create test networks for influence propagation analysis."""
    networks = {}

    # Small test networks
    networks['karate'] = nx.karate_club_graph()

    # Synthetic networks with different properties
    networks['erdos_renyi'] = nx.erdos_renyi_graph(100, 0.05, seed=42)
    networks['barabasi_albert'] = nx.barabasi_albert_graph(100, 3, seed=42)
    networks['watts_strogatz'] = nx.watts_strogatz_graph(100, 6, 0.3, seed=42)

    # Networks with community structure
    networks['community'] = nx.planted_partition_graph(4, 25, 0.7, 0.05, seed=42)

    return networks


def main():
    """Run comprehensive influence propagation analysis."""
    print("=" * 60)
    print("Influence Propagation and Cascade Analysis")
    print("=" * 60)

    # Create test networks
    networks = create_test_networks()

    for network_name, graph in networks.items():
        print(f"\nAnalyzing {network_name} network ({len(graph.nodes())} nodes, {len(graph.edges())} edges)...")

        analyzer = InfluencePropagationAnalyzer(graph)

        # Select seed nodes (top 3 by degree)
        degrees = dict(graph.degree())
        seed_nodes = sorted(degrees.keys(), key=lambda x: degrees[x], reverse=True)[:3]

        print(f"  Seed nodes: {seed_nodes}")

        try:
            # 1. Basic influence models comparison
            print("  Running basic influence models...")

            ic_result = analyzer.independent_cascade_simulation(seed_nodes, n_simulations=500)
            lt_result = analyzer.linear_threshold_simulation(seed_nodes, n_simulations=200)
            cont_result = analyzer.continuous_influence_sublinear(seed_nodes)

            basic_results = {
                'independent_cascade': ic_result,
                'linear_threshold': lt_result,
                'continuous_influence': cont_result
            }

            # 2. Multi-step analysis
            print("  Running multi-step analysis...")
            multistep_result = analyzer.multi_step_influence_analysis(seed_nodes, max_steps=5)

            # 3. Influence maximization
            print("  Running influence maximization analysis...")
            maximization_result = analyzer.influence_maximization_analysis(
                budget=3, methods=['degree', 'pagerank', 'random']
            )

            # 4. Viral cascade analysis (only for smaller networks)
            if len(graph.nodes()) <= 100:
                print("  Running viral cascade analysis...")
                viral_result = analyzer.viral_cascade_analysis(seed_nodes)
            else:
                viral_result = {'skipped': 'Network too large for viral analysis'}

            # Combine all results
            all_results = {
                'basic_models': basic_results,
                'multi_step_analysis': multistep_result,
                'influence_maximization': maximization_result,
                'viral_cascade_analysis': viral_result
            }

            # Save results
            analyzer.save_results(all_results, f'influence_{network_name}_results.json')

            # Print summary
            print(f"  IC mean influence: {ic_result['mean_influence']:.1f} nodes")
            print(f"  LT mean influence: {lt_result['mean_influence']:.1f} nodes")
            if cont_result['success']:
                print(f"  Continuous influence: {cont_result['total_influence']:.2f}")

        except Exception as e:
            print(f"  Analysis failed: {e}")

    print("\n" + "=" * 60)
    print("Influence propagation analysis complete!")
    print("=" * 60)


if __name__ == "__main__":
    main()