#!/usr/bin/env python3
"""
Social Influence Propagation Models

This module implements various influence propagation models:
- Independent Cascade Model
- Linear Threshold Model
- Friedkin-Johnsen Opinion Dynamics
- Information Diffusion via Matrix Powers
- Sublinear approximations using linear systems

Author: Social Network Analysis Agent
"""

import numpy as np
import networkx as nx
import time
from typing import Dict, List, Tuple, Set, Optional
import random
from collections import defaultdict, deque


class IndependentCascadeModel:
    """Independent Cascade Model for influence propagation."""

    def __init__(self, graph: nx.Graph, influence_probabilities: Optional[Dict] = None):
        """
        Initialize Independent Cascade Model.

        Args:
            graph: Social network graph
            influence_probabilities: Edge influence probabilities (default: 0.1)
        """
        self.graph = graph
        self.influence_probs = influence_probabilities or {}

        # Set default influence probability
        for u, v in graph.edges():
            if (u, v) not in self.influence_probs:
                self.influence_probs[(u, v)] = 0.1

    def simulate_cascade(self, seed_nodes: List[int], num_simulations: int = 1000) -> Dict:
        """
        Simulate influence cascade from seed nodes.

        Returns:
            Dictionary with influence statistics
        """
        total_influenced = []
        influence_counts = defaultdict(int)

        for sim in range(num_simulations):
            influenced = set(seed_nodes)
            newly_influenced = set(seed_nodes)

            while newly_influenced:
                next_influenced = set()

                for node in newly_influenced:
                    for neighbor in self.graph.neighbors(node):
                        if neighbor not in influenced:
                            # Check if influence occurs
                            prob = self.influence_probs.get((node, neighbor), 0.1)
                            if random.random() < prob:
                                next_influenced.add(neighbor)
                                influenced.add(neighbor)

                newly_influenced = next_influenced

            # Record results
            total_influenced.append(len(influenced))
            for node in influenced:
                influence_counts[node] += 1

        # Calculate statistics
        avg_influenced = np.mean(total_influenced)
        std_influenced = np.std(total_influenced)

        # Node influence probabilities
        node_influence_probs = {node: count / num_simulations
                               for node, count in influence_counts.items()}

        return {
            'average_influenced': avg_influenced,
            'std_influenced': std_influenced,
            'total_influenced_distribution': total_influenced,
            'node_influence_probabilities': node_influence_probs,
            'max_influenced': max(total_influenced),
            'min_influenced': min(total_influenced)
        }

    def expected_influence_linear(self, seed_nodes: List[int]) -> Dict[int, float]:
        """
        Compute expected influence using linear algebraic approximation.

        Expected influence ≈ (I - P)^(-1) * seed_vector
        where P is the influence probability matrix.
        """
        nodes = list(self.graph.nodes())
        n_nodes = len(nodes)
        node_to_idx = {node: idx for idx, node in enumerate(nodes)}

        # Create influence probability matrix
        P = np.zeros((n_nodes, n_nodes))
        for u, v in self.graph.edges():
            prob = self.influence_probs.get((u, v), 0.1)
            P[node_to_idx[u], node_to_idx[v]] = prob
            # For undirected graphs, add reverse direction
            if not self.graph.is_directed():
                P[node_to_idx[v], node_to_idx[u]] = prob

        # Create seed vector
        seed_vector = np.zeros(n_nodes)
        for seed in seed_nodes:
            if seed in node_to_idx:
                seed_vector[node_to_idx[seed]] = 1.0

        # Solve (I - P)x = seed_vector
        I = np.eye(n_nodes)
        try:
            system_matrix = I - P
            expected_influence = np.linalg.solve(system_matrix, seed_vector)
        except np.linalg.LinAlgError:
            # Use pseudoinverse if singular
            expected_influence = np.linalg.pinv(I - P) @ seed_vector

        # Convert back to node dictionary
        return {nodes[i]: expected_influence[i] for i in range(n_nodes)}


class LinearThresholdModel:
    """Linear Threshold Model for influence propagation."""

    def __init__(self, graph: nx.Graph, thresholds: Optional[Dict] = None,
                 edge_weights: Optional[Dict] = None):
        """
        Initialize Linear Threshold Model.

        Args:
            graph: Social network graph
            thresholds: Node activation thresholds (default: random [0.1, 0.5])
            edge_weights: Edge influence weights (default: uniform)
        """
        self.graph = graph
        self.thresholds = thresholds or {}
        self.edge_weights = edge_weights or {}

        # Set default thresholds
        for node in graph.nodes():
            if node not in self.thresholds:
                self.thresholds[node] = random.uniform(0.1, 0.5)

        # Normalize edge weights so each node's incoming weights sum to ≤ 1
        self._normalize_edge_weights()

    def _normalize_edge_weights(self):
        """Normalize edge weights to satisfy threshold model constraints."""
        for node in self.graph.nodes():
            neighbors = list(self.graph.neighbors(node))
            if not neighbors:
                continue

            # Set uniform weights if not specified
            total_weight = 0
            for neighbor in neighbors:
                if (neighbor, node) not in self.edge_weights:
                    self.edge_weights[(neighbor, node)] = 1.0 / len(neighbors)
                total_weight += self.edge_weights[(neighbor, node)]

            # Normalize to ensure sum ≤ 1
            if total_weight > 1.0:
                for neighbor in neighbors:
                    self.edge_weights[(neighbor, node)] /= total_weight

    def simulate_threshold_cascade(self, seed_nodes: List[int],
                                 num_simulations: int = 1000) -> Dict:
        """Simulate Linear Threshold Model cascade."""
        total_influenced = []
        influence_counts = defaultdict(int)

        for sim in range(num_simulations):
            influenced = set(seed_nodes)

            while True:
                newly_influenced = set()

                for node in self.graph.nodes():
                    if node in influenced:
                        continue

                    # Calculate total influence from neighbors
                    total_influence = 0
                    for neighbor in self.graph.neighbors(node):
                        if neighbor in influenced:
                            weight = self.edge_weights.get((neighbor, node), 0)
                            total_influence += weight

                    # Check if threshold is exceeded
                    if total_influence >= self.thresholds[node]:
                        newly_influenced.add(node)

                if not newly_influenced:
                    break

                influenced.update(newly_influenced)

            # Record results
            total_influenced.append(len(influenced))
            for node in influenced:
                influence_counts[node] += 1

        # Calculate statistics
        avg_influenced = np.mean(total_influenced)
        std_influenced = np.std(total_influenced)

        node_influence_probs = {node: count / num_simulations
                               for node, count in influence_counts.items()}

        return {
            'average_influenced': avg_influenced,
            'std_influenced': std_influenced,
            'total_influenced_distribution': total_influenced,
            'node_influence_probabilities': node_influence_probs,
            'max_influenced': max(total_influenced),
            'min_influenced': min(total_influenced)
        }


class FriedkinJohnsenModel:
    """Friedkin-Johnsen Opinion Dynamics Model."""

    def __init__(self, graph: nx.Graph, susceptibility: Optional[Dict] = None):
        """
        Initialize Friedkin-Johnsen model.

        Args:
            graph: Social network graph
            susceptibility: Node susceptibility to influence [0,1] (default: 0.5)
        """
        self.graph = graph
        self.susceptibility = susceptibility or {}

        # Set default susceptibility
        for node in graph.nodes():
            if node not in self.susceptibility:
                self.susceptibility[node] = 0.5

    def compute_opinion_equilibrium(self, initial_opinions: Dict[int, float]) -> Dict[int, float]:
        """
        Compute opinion equilibrium using linear system:
        (I - α*W)x = (1-α)*s + α*W*s

        where:
        - α is susceptibility matrix
        - W is normalized adjacency matrix
        - s is initial opinions
        """
        nodes = list(self.graph.nodes())
        n_nodes = len(nodes)
        node_to_idx = {node: idx for idx, node in enumerate(nodes)}

        # Create normalized adjacency matrix (row-stochastic)
        adj = nx.adjacency_matrix(self.graph, nodelist=nodes).toarray()
        degrees = adj.sum(axis=1)
        degrees[degrees == 0] = 1  # Handle isolated nodes
        W = adj / degrees.reshape(-1, 1)

        # Create susceptibility diagonal matrix
        alpha_diag = np.array([self.susceptibility[node] for node in nodes])
        Alpha = np.diag(alpha_diag)

        # Create initial opinion vector
        s = np.array([initial_opinions.get(node, 0.0) for node in nodes])

        # Solve: (I - Alpha * W) * x = (I - Alpha) * s
        I = np.eye(n_nodes)
        system_matrix = I - Alpha @ W
        rhs = (I - Alpha) @ s

        try:
            equilibrium = np.linalg.solve(system_matrix, rhs)
        except np.linalg.LinAlgError:
            equilibrium = np.linalg.lstsq(system_matrix, rhs, rcond=None)[0]

        return {nodes[i]: equilibrium[i] for i in range(n_nodes)}

    def simulate_opinion_dynamics(self, initial_opinions: Dict[int, float],
                                max_iterations: int = 100,
                                tolerance: float = 1e-6) -> Dict:
        """Simulate opinion dynamics iteratively."""
        nodes = list(self.graph.nodes())
        current_opinions = {node: initial_opinions.get(node, 0.0) for node in nodes}
        history = [current_opinions.copy()]

        for iteration in range(max_iterations):
            new_opinions = {}

            for node in nodes:
                # Get neighbor opinions
                neighbor_opinions = [current_opinions[neighbor]
                                   for neighbor in self.graph.neighbors(node)]

                if neighbor_opinions:
                    avg_neighbor_opinion = np.mean(neighbor_opinions)
                else:
                    avg_neighbor_opinion = 0.0

                # Update opinion: α * neighbor_avg + (1-α) * initial
                alpha = self.susceptibility[node]
                initial = initial_opinions.get(node, 0.0)
                new_opinions[node] = (alpha * avg_neighbor_opinion +
                                    (1 - alpha) * initial)

            # Check convergence
            max_change = max(abs(new_opinions[node] - current_opinions[node])
                           for node in nodes)

            current_opinions = new_opinions
            history.append(current_opinions.copy())

            if max_change < tolerance:
                break

        return {
            'final_opinions': current_opinions,
            'history': history,
            'iterations': len(history) - 1,
            'converged': max_change < tolerance
        }


class MatrixPowerInfluence:
    """Influence propagation using matrix powers and geometric series."""

    def __init__(self, graph: nx.Graph):
        """Initialize with graph."""
        self.graph = graph
        self.nodes = list(graph.nodes())
        self.n_nodes = len(self.nodes)
        self.node_to_idx = {node: idx for idx, node in enumerate(self.nodes)}

    def compute_k_step_influence(self, seed_nodes: List[int], k: int,
                               decay: float = 0.8) -> Dict[int, float]:
        """
        Compute k-step influence using matrix powers.

        Influence after k steps: Σ(t=0 to k) decay^t * A^t * seed_vector
        """
        # Create adjacency matrix
        adj = nx.adjacency_matrix(self.graph, nodelist=self.nodes).toarray()

        # Normalize by out-degree
        degrees = adj.sum(axis=1)
        degrees[degrees == 0] = 1
        transition = adj / degrees.reshape(-1, 1)

        # Create seed vector
        seed_vector = np.zeros(self.n_nodes)
        for seed in seed_nodes:
            if seed in self.node_to_idx:
                seed_vector[self.node_to_idx[seed]] = 1.0

        # Compute influence using matrix powers
        total_influence = seed_vector.copy()
        current_vector = seed_vector.copy()

        for step in range(1, k + 1):
            current_vector = transition.T @ current_vector
            total_influence += (decay ** step) * current_vector

        return {self.nodes[i]: total_influence[i] for i in range(self.n_nodes)}

    def compute_infinite_influence(self, seed_nodes: List[int],
                                 decay: float = 0.8) -> Dict[int, float]:
        """
        Compute infinite-step influence using geometric series.

        Total influence: (I - decay*A^T)^(-1) * seed_vector
        """
        # Create adjacency matrix
        adj = nx.adjacency_matrix(self.graph, nodelist=self.nodes).toarray()

        # Normalize by out-degree
        degrees = adj.sum(axis=1)
        degrees[degrees == 0] = 1
        transition = adj / degrees.reshape(-1, 1)

        # Create seed vector
        seed_vector = np.zeros(self.n_nodes)
        for seed in seed_nodes:
            if seed in self.node_to_idx:
                seed_vector[self.node_to_idx[seed]] = 1.0

        # Solve (I - decay*A^T) * x = seed_vector
        I = np.eye(self.n_nodes)
        system_matrix = I - decay * transition.T

        try:
            total_influence = np.linalg.solve(system_matrix, seed_vector)
        except np.linalg.LinAlgError:
            total_influence = np.linalg.lstsq(system_matrix, seed_vector, rcond=None)[0]

        return {self.nodes[i]: total_influence[i] for i in range(self.n_nodes)}


def compare_influence_models():
    """Compare different influence propagation models."""
    print("=== Influence Models Comparison ===\n")

    # Create test network
    G = nx.barabasi_albert_graph(50, 3, seed=42)
    seed_nodes = [0, 10, 20]  # High-degree nodes as seeds

    results = {}

    # Independent Cascade Model
    print("1. Independent Cascade Model")
    ic_model = IndependentCascadeModel(G)
    start_time = time.time()
    ic_results = ic_model.simulate_cascade(seed_nodes, num_simulations=500)
    ic_time = time.time() - start_time

    # Linear approximation
    start_time = time.time()
    ic_linear = ic_model.expected_influence_linear(seed_nodes)
    ic_linear_time = time.time() - start_time

    print(f"  Simulation: {ic_results['average_influenced']:.2f} ± {ic_results['std_influenced']:.2f} nodes ({ic_time:.3f}s)")
    print(f"  Linear approx: {sum(ic_linear.values()):.2f} total influence ({ic_linear_time:.3f}s)")

    # Linear Threshold Model
    print("\n2. Linear Threshold Model")
    lt_model = LinearThresholdModel(G)
    start_time = time.time()
    lt_results = lt_model.simulate_threshold_cascade(seed_nodes, num_simulations=500)
    lt_time = time.time() - start_time

    print(f"  Simulation: {lt_results['average_influenced']:.2f} ± {lt_results['std_influenced']:.2f} nodes ({lt_time:.3f}s)")

    # Friedkin-Johnsen Opinion Dynamics
    print("\n3. Friedkin-Johnsen Opinion Dynamics")
    fj_model = FriedkinJohnsenModel(G)
    initial_opinions = {node: 1.0 if node in seed_nodes else 0.0
                       for node in G.nodes()}

    start_time = time.time()
    fj_equilibrium = fj_model.compute_opinion_equilibrium(initial_opinions)
    fj_linear_time = time.time() - start_time

    start_time = time.time()
    fj_simulation = fj_model.simulate_opinion_dynamics(initial_opinions)
    fj_sim_time = time.time() - start_time

    equilibrium_sum = sum(fj_equilibrium.values())
    simulation_sum = sum(fj_simulation['final_opinions'].values())

    print(f"  Linear solution: {equilibrium_sum:.3f} total opinion ({fj_linear_time:.3f}s)")
    print(f"  Simulation: {simulation_sum:.3f} total opinion, {fj_simulation['iterations']} iterations ({fj_sim_time:.3f}s)")

    # Matrix Power Influence
    print("\n4. Matrix Power Influence")
    mp_model = MatrixPowerInfluence(G)

    start_time = time.time()
    mp_k_step = mp_model.compute_k_step_influence(seed_nodes, k=5, decay=0.8)
    mp_k_time = time.time() - start_time

    start_time = time.time()
    mp_infinite = mp_model.compute_infinite_influence(seed_nodes, decay=0.8)
    mp_inf_time = time.time() - start_time

    print(f"  5-step influence: {sum(mp_k_step.values()):.3f} total ({mp_k_time:.3f}s)")
    print(f"  Infinite influence: {sum(mp_infinite.values()):.3f} total ({mp_inf_time:.3f}s)")

    # Summary
    print("\n=== Performance Summary ===")
    print(f"Independent Cascade - Simulation: {ic_time:.3f}s, Linear: {ic_linear_time:.3f}s (speedup: {ic_time/ic_linear_time:.1f}x)")
    print(f"Opinion Dynamics - Simulation: {fj_sim_time:.3f}s, Linear: {fj_linear_time:.3f}s (speedup: {fj_sim_time/fj_linear_time:.1f}x)")
    print(f"Matrix Power - k-step: {mp_k_time:.3f}s, Infinite: {mp_inf_time:.3f}s")

    return {
        'independent_cascade': {
            'simulation': ic_results,
            'linear': ic_linear,
            'times': {'simulation': ic_time, 'linear': ic_linear_time}
        },
        'linear_threshold': {
            'simulation': lt_results,
            'time': lt_time
        },
        'opinion_dynamics': {
            'equilibrium': fj_equilibrium,
            'simulation': fj_simulation,
            'times': {'simulation': fj_sim_time, 'linear': fj_linear_time}
        },
        'matrix_power': {
            'k_step': mp_k_step,
            'infinite': mp_infinite,
            'times': {'k_step': mp_k_time, 'infinite': mp_inf_time}
        }
    }


if __name__ == "__main__":
    results = compare_influence_models()

    # Save results
    import json
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

    with open('/workspaces/sublinear-time-solver/scripts/social_networks/influence_results.json', 'w') as f:
        json.dump(convert_types(results), f, indent=2)

    print("\nInfluence models comparison complete!")
    print("Results saved to influence_results.json")