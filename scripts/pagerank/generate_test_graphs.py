#!/usr/bin/env python3
"""
Generate various types of test graphs for PageRank comparison.
Creates different graph categories to test algorithm performance.
"""

import numpy as np
import networkx as nx
import random
from typing import Tuple, List, Dict
import json
import os


class GraphGenerator:
    """Generate different types of graphs for PageRank testing."""

    @staticmethod
    def small_dense_graph(n: int = 50, density: float = 0.3, seed: int = 42) -> Tuple[np.ndarray, nx.Graph]:
        """Generate small dense graph."""
        np.random.seed(seed)
        random.seed(seed)

        # Create random dense graph
        graph = nx.erdos_renyi_graph(n, density, directed=True, seed=seed)

        # Ensure strong connectivity by adding a cycle
        for i in range(n):
            graph.add_edge(i, (i + 1) % n)

        adjacency_matrix = nx.adjacency_matrix(graph).toarray()
        return adjacency_matrix, graph

    @staticmethod
    def large_sparse_graph(n: int = 5000, avg_degree: int = 5, seed: int = 42) -> Tuple[np.ndarray, nx.Graph]:
        """Generate large sparse graph."""
        np.random.seed(seed)
        random.seed(seed)

        # Calculate edge probability for desired average degree
        p = avg_degree / (n - 1)

        # Create sparse random graph
        graph = nx.erdos_renyi_graph(n, p, directed=True, seed=seed)

        # Add some structure to make it more realistic
        # Add preferential attachment component
        ba_graph = nx.barabasi_albert_graph(n // 4, 3, seed=seed)
        ba_directed = ba_graph.to_directed()

        # Merge graphs
        for edge in ba_directed.edges():
            if edge[0] < n and edge[1] < n:
                graph.add_edge(edge[0], edge[1])

        adjacency_matrix = nx.adjacency_matrix(graph).toarray()
        return adjacency_matrix, graph

    @staticmethod
    def power_law_graph(n: int = 1000, alpha: float = 2.5, seed: int = 42) -> Tuple[np.ndarray, nx.Graph]:
        """Generate web-like graph with power-law degree distribution."""
        np.random.seed(seed)
        random.seed(seed)

        # Create scale-free network using preferential attachment
        m = max(1, int(np.log(n)))  # Number of edges to attach from a new node
        graph = nx.barabasi_albert_graph(n, m, seed=seed).to_directed()

        # Add some randomness to make it more web-like
        num_random_edges = n // 10
        nodes = list(graph.nodes())

        for _ in range(num_random_edges):
            u, v = random.sample(nodes, 2)
            if u != v:
                graph.add_edge(u, v)

        adjacency_matrix = nx.adjacency_matrix(graph).toarray()
        return adjacency_matrix, graph

    @staticmethod
    def social_network_graph(n: int = 800, communities: int = 5, seed: int = 42) -> Tuple[np.ndarray, nx.Graph]:
        """Generate social network-like graph with community structure."""
        np.random.seed(seed)
        random.seed(seed)

        # Create graph with community structure
        sizes = [n // communities] * communities
        if sum(sizes) < n:
            sizes[-1] += n - sum(sizes)

        # Higher probability within communities, lower between
        p_in = 0.3   # Probability of edge within community
        p_out = 0.05  # Probability of edge between communities

        graph = nx.stochastic_block_model(sizes, [[p_in if i == j else p_out for j in range(communities)] for i in range(communities)], directed=True, seed=seed)

        # Add some influential nodes (hubs)
        num_hubs = max(1, n // 50)
        hub_nodes = random.sample(list(graph.nodes()), num_hubs)

        for hub in hub_nodes:
            # Connect hub to many random nodes
            targets = random.sample([node for node in graph.nodes() if node != hub], min(n // 10, n - 1))
            for target in targets:
                graph.add_edge(hub, target)
                # Some bidirectional connections
                if random.random() < 0.3:
                    graph.add_edge(target, hub)

        adjacency_matrix = nx.adjacency_matrix(graph).toarray()
        return adjacency_matrix, graph

    @staticmethod
    def dag_graph(n: int = 200, layers: int = 5, seed: int = 42) -> Tuple[np.ndarray, nx.Graph]:
        """Generate directed acyclic graph (DAG)."""
        np.random.seed(seed)
        random.seed(seed)

        graph = nx.DiGraph()
        graph.add_nodes_from(range(n))

        # Organize nodes into layers
        nodes_per_layer = n // layers
        node_layers = []

        for i in range(layers):
            start_idx = i * nodes_per_layer
            end_idx = start_idx + nodes_per_layer if i < layers - 1 else n
            node_layers.append(list(range(start_idx, end_idx)))

        # Add edges only from earlier layers to later layers
        for i in range(layers - 1):
            current_layer = node_layers[i]
            next_layers = node_layers[i + 1:]

            for node in current_layer:
                # Connect to nodes in subsequent layers
                for j, target_layer in enumerate(next_layers):
                    # Probability decreases with distance
                    prob = 0.3 / (j + 1)
                    targets = [target for target in target_layer if random.random() < prob]

                    for target in targets:
                        graph.add_edge(node, target)

        # Ensure connectivity
        for i in range(layers - 1):
            current_layer = node_layers[i]
            next_layer = node_layers[i + 1]

            # Ensure at least one connection from each layer to the next
            for node in current_layer:
                if not any(target in next_layer for target in graph.successors(node)):
                    target = random.choice(next_layer)
                    graph.add_edge(node, target)

        adjacency_matrix = nx.adjacency_matrix(graph).toarray()
        return adjacency_matrix, graph

    @staticmethod
    def grid_graph(width: int = 20, height: int = 20, add_random_edges: bool = True, seed: int = 42) -> Tuple[np.ndarray, nx.Graph]:
        """Generate grid-like graph with optional random edges."""
        np.random.seed(seed)
        random.seed(seed)

        # Create 2D grid graph
        graph = nx.grid_2d_graph(width, height, create_using=nx.DiGraph())

        # Convert node labels from (x,y) tuples to integers
        mapping = {node: i for i, node in enumerate(graph.nodes())}
        graph = nx.relabel_nodes(graph, mapping)

        # Make edges bidirectional for grid
        edges_to_add = []
        for u, v in graph.edges():
            edges_to_add.append((v, u))
        graph.add_edges_from(edges_to_add)

        # Add some random long-distance edges if requested
        if add_random_edges:
            n = len(graph.nodes())
            num_random_edges = n // 20
            nodes = list(graph.nodes())

            for _ in range(num_random_edges):
                u, v = random.sample(nodes, 2)
                if u != v and not graph.has_edge(u, v):
                    graph.add_edge(u, v)

        adjacency_matrix = nx.adjacency_matrix(graph).toarray()
        return adjacency_matrix, graph

    @staticmethod
    def complete_graph(n: int = 30) -> Tuple[np.ndarray, nx.Graph]:
        """Generate complete directed graph."""
        graph = nx.complete_graph(n, create_using=nx.DiGraph())
        adjacency_matrix = nx.adjacency_matrix(graph).toarray()
        return adjacency_matrix, graph

    @staticmethod
    def star_graph(n: int = 100, centers: int = 3, seed: int = 42) -> Tuple[np.ndarray, nx.Graph]:
        """Generate star-like graph with multiple centers."""
        np.random.seed(seed)
        random.seed(seed)

        graph = nx.DiGraph()
        graph.add_nodes_from(range(n))

        # Select center nodes
        center_nodes = random.sample(list(range(n)), centers)
        peripheral_nodes = [node for node in range(n) if node not in center_nodes]

        # Connect each peripheral node to random centers
        for node in peripheral_nodes:
            # Connect to at least one center
            target_centers = random.sample(center_nodes, random.randint(1, min(centers, 3)))
            for center in target_centers:
                graph.add_edge(node, center)
                # Sometimes bidirectional
                if random.random() < 0.4:
                    graph.add_edge(center, node)

        # Connect centers to each other
        for i, center1 in enumerate(center_nodes):
            for center2 in center_nodes[i + 1:]:
                if random.random() < 0.8:
                    graph.add_edge(center1, center2)
                    graph.add_edge(center2, center1)

        adjacency_matrix = nx.adjacency_matrix(graph).toarray()
        return adjacency_matrix, graph


def generate_test_suite(output_dir: str = "/workspaces/sublinear-time-solver/scripts/pagerank/test_graphs"):
    """Generate complete test suite of graphs."""
    os.makedirs(output_dir, exist_ok=True)

    generator = GraphGenerator()
    test_graphs = {}

    print("Generating test graphs...")

    # Small dense graphs
    print("1. Small dense graphs...")
    for i, n in enumerate([10, 30, 50, 100]):
        adj_matrix, graph = generator.small_dense_graph(n, density=0.4, seed=42 + i)
        test_graphs[f"small_dense_{n}"] = {
            "adjacency_matrix": adj_matrix.tolist(),
            "num_nodes": int(n),
            "num_edges": int(graph.number_of_edges()),
            "density": float(nx.density(graph)),
            "category": "small_dense"
        }

    # Large sparse graphs
    print("2. Large sparse graphs...")
    for i, n in enumerate([500, 1000, 2000, 5000]):
        adj_matrix, graph = generator.large_sparse_graph(n, avg_degree=5, seed=42 + i)
        test_graphs[f"large_sparse_{n}"] = {
            "adjacency_matrix": adj_matrix.tolist(),
            "num_nodes": int(n),
            "num_edges": int(graph.number_of_edges()),
            "density": float(nx.density(graph)),
            "category": "large_sparse"
        }

    # Power-law graphs
    print("3. Power-law graphs...")
    for i, n in enumerate([200, 500, 1000]):
        adj_matrix, graph = generator.power_law_graph(n, alpha=2.5, seed=42 + i)
        test_graphs[f"power_law_{n}"] = {
            "adjacency_matrix": adj_matrix.tolist(),
            "num_nodes": int(n),
            "num_edges": int(graph.number_of_edges()),
            "density": float(nx.density(graph)),
            "category": "power_law"
        }

    # Social network graphs
    print("4. Social network graphs...")
    for i, n in enumerate([300, 600, 1000]):
        adj_matrix, graph = generator.social_network_graph(n, communities=5, seed=42 + i)
        test_graphs[f"social_network_{n}"] = {
            "adjacency_matrix": adj_matrix.tolist(),
            "num_nodes": int(n),
            "num_edges": int(graph.number_of_edges()),
            "density": float(nx.density(graph)),
            "category": "social_network"
        }

    # DAG graphs
    print("5. DAG graphs...")
    for i, n in enumerate([100, 200, 400]):
        adj_matrix, graph = generator.dag_graph(n, layers=5, seed=42 + i)
        test_graphs[f"dag_{n}"] = {
            "adjacency_matrix": adj_matrix.tolist(),
            "num_nodes": int(n),
            "num_edges": int(graph.number_of_edges()),
            "density": float(nx.density(graph)),
            "category": "dag"
        }

    # Grid graphs
    print("6. Grid graphs...")
    for i, size in enumerate([10, 15, 20]):
        adj_matrix, graph = generator.grid_graph(size, size, add_random_edges=True, seed=42 + i)
        test_graphs[f"grid_{size}x{size}"] = {
            "adjacency_matrix": adj_matrix.tolist(),
            "num_nodes": int(graph.number_of_nodes()),
            "num_edges": int(graph.number_of_edges()),
            "density": float(nx.density(graph)),
            "category": "grid"
        }

    # Special case graphs
    print("7. Special case graphs...")

    # Complete graph
    adj_matrix, graph = generator.complete_graph(25)
    test_graphs["complete_25"] = {
        "adjacency_matrix": adj_matrix.tolist(),
        "num_nodes": int(graph.number_of_nodes()),
        "num_edges": int(graph.number_of_edges()),
        "density": float(nx.density(graph)),
        "category": "complete"
    }

    # Star graph
    adj_matrix, graph = generator.star_graph(100, centers=3, seed=42)
    test_graphs["star_100_3centers"] = {
        "adjacency_matrix": adj_matrix.tolist(),
        "num_nodes": int(graph.number_of_nodes()),
        "num_edges": int(graph.number_of_edges()),
        "density": float(nx.density(graph)),
        "category": "star"
    }

    # Save test graphs
    output_file = os.path.join(output_dir, "test_graphs.json")
    with open(output_file, 'w') as f:
        json.dump(test_graphs, f, indent=2)

    print(f"\nGenerated {len(test_graphs)} test graphs")
    print(f"Saved to: {output_file}")

    # Generate summary
    summary = {}
    for category in ["small_dense", "large_sparse", "power_law", "social_network", "dag", "grid", "complete", "star"]:
        category_graphs = [name for name, data in test_graphs.items() if data["category"] == category]
        if category_graphs:
            summary[category] = {
                "count": len(category_graphs),
                "graphs": category_graphs,
                "size_range": [
                    min(test_graphs[g]["num_nodes"] for g in category_graphs),
                    max(test_graphs[g]["num_nodes"] for g in category_graphs)
                ]
            }

    summary_file = os.path.join(output_dir, "test_summary.json")
    with open(summary_file, 'w') as f:
        json.dump(summary, f, indent=2)

    print(f"Test summary saved to: {summary_file}")

    return test_graphs, summary


if __name__ == "__main__":
    generate_test_suite()