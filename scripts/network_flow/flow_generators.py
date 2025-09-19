"""
Network Flow Graph Generators
==============================

Generate various network topologies for testing flow algorithms:
- Grid networks (2D/3D meshes)
- Random sparse graphs
- Scale-free networks (Barabási-Albert)
- Hierarchical networks
- Small-world networks (Watts-Strogatz)
- Real-world inspired topologies

Each generator creates networks with realistic capacity and cost distributions.
"""

import numpy as np
import random
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
import math


@dataclass
class NetworkProperties:
    """Properties of generated network."""
    n_nodes: int
    n_edges: int
    density: float
    avg_capacity: float
    avg_cost: float
    topology_type: str
    parameters: Dict


class NetworkGenerator:
    """Base class for network generators."""

    def __init__(self, seed: int = 42):
        self.seed = seed
        np.random.seed(seed)
        random.seed(seed)

    def generate_capacity(self, base_capacity: float = 10.0) -> float:
        """Generate realistic capacity with log-normal distribution."""
        return max(1, np.random.lognormal(np.log(base_capacity), 0.5))

    def generate_cost(self, base_cost: float = 1.0) -> float:
        """Generate realistic cost with exponential distribution."""
        return max(0.1, np.random.exponential(base_cost))

    def add_source_sink(self, edges: List[Tuple], capacities: Dict, costs: Dict,
                       n_nodes: int, source_capacity: float = 100.0) -> Tuple[int, int]:
        """Add dedicated source and sink nodes with high capacity connections."""
        source = n_nodes
        sink = n_nodes + 1

        # Connect source to multiple nodes
        source_connections = min(5, n_nodes // 2)
        for _ in range(source_connections):
            target = np.random.randint(0, n_nodes)
            edge = (source, target)
            edges.append(edge)
            capacities[edge] = self.generate_capacity(source_capacity)
            costs[edge] = self.generate_cost(0.1)

        # Connect multiple nodes to sink
        sink_connections = min(5, n_nodes // 2)
        for _ in range(sink_connections):
            origin = np.random.randint(0, n_nodes)
            edge = (origin, sink)
            edges.append(edge)
            capacities[edge] = self.generate_capacity(source_capacity)
            costs[edge] = self.generate_cost(0.1)

        return source, sink


class GridNetworkGenerator(NetworkGenerator):
    """Generate 2D and 3D grid networks."""

    def generate_2d_grid(self, width: int, height: int,
                        diagonal_connections: bool = False) -> Dict:
        """Generate 2D grid network."""
        n_nodes = width * height
        edges = []
        capacities = {}
        costs = {}

        def node_id(x: int, y: int) -> int:
            return y * width + x

        # Add horizontal and vertical connections
        for y in range(height):
            for x in range(width):
                current = node_id(x, y)

                # Right connection
                if x < width - 1:
                    right = node_id(x + 1, y)
                    edge = (current, right)
                    edges.append(edge)
                    capacities[edge] = self.generate_capacity(15.0)
                    costs[edge] = self.generate_cost(1.0)

                # Down connection
                if y < height - 1:
                    down = node_id(x, y + 1)
                    edge = (current, down)
                    edges.append(edge)
                    capacities[edge] = self.generate_capacity(15.0)
                    costs[edge] = self.generate_cost(1.0)

                # Diagonal connections (if enabled)
                if diagonal_connections:
                    if x < width - 1 and y < height - 1:
                        diagonal = node_id(x + 1, y + 1)
                        edge = (current, diagonal)
                        edges.append(edge)
                        capacities[edge] = self.generate_capacity(8.0)
                        costs[edge] = self.generate_cost(1.4)  # Higher cost for diagonal

        source, sink = self.add_source_sink(edges, capacities, costs, n_nodes)

        return {
            'n_nodes': n_nodes + 2,
            'edges': edges,
            'capacities': capacities,
            'costs': costs,
            'source': source,
            'sink': sink,
            'properties': NetworkProperties(
                n_nodes=n_nodes + 2,
                n_edges=len(edges),
                density=len(edges) / (n_nodes * (n_nodes - 1) / 2),
                avg_capacity=np.mean(list(capacities.values())),
                avg_cost=np.mean(list(costs.values())),
                topology_type='2d_grid',
                parameters={'width': width, 'height': height, 'diagonal': diagonal_connections}
            )
        }

    def generate_3d_grid(self, width: int, height: int, depth: int) -> Dict:
        """Generate 3D grid network."""
        n_nodes = width * height * depth
        edges = []
        capacities = {}
        costs = {}

        def node_id(x: int, y: int, z: int) -> int:
            return z * (width * height) + y * width + x

        # Add connections in all 3 dimensions
        for z in range(depth):
            for y in range(height):
                for x in range(width):
                    current = node_id(x, y, z)

                    # X direction (right)
                    if x < width - 1:
                        right = node_id(x + 1, y, z)
                        edge = (current, right)
                        edges.append(edge)
                        capacities[edge] = self.generate_capacity(12.0)
                        costs[edge] = self.generate_cost(1.0)

                    # Y direction (down)
                    if y < height - 1:
                        down = node_id(x, y + 1, z)
                        edge = (current, down)
                        edges.append(edge)
                        capacities[edge] = self.generate_capacity(12.0)
                        costs[edge] = self.generate_cost(1.0)

                    # Z direction (forward)
                    if z < depth - 1:
                        forward = node_id(x, y, z + 1)
                        edge = (current, forward)
                        edges.append(edge)
                        capacities[edge] = self.generate_capacity(12.0)
                        costs[edge] = self.generate_cost(1.0)

        source, sink = self.add_source_sink(edges, capacities, costs, n_nodes)

        return {
            'n_nodes': n_nodes + 2,
            'edges': edges,
            'capacities': capacities,
            'costs': costs,
            'source': source,
            'sink': sink,
            'properties': NetworkProperties(
                n_nodes=n_nodes + 2,
                n_edges=len(edges),
                density=len(edges) / (n_nodes * (n_nodes - 1) / 2),
                avg_capacity=np.mean(list(capacities.values())),
                avg_cost=np.mean(list(costs.values())),
                topology_type='3d_grid',
                parameters={'width': width, 'height': height, 'depth': depth}
            )
        }


class RandomNetworkGenerator(NetworkGenerator):
    """Generate random networks with various properties."""

    def generate_erdos_renyi(self, n_nodes: int, edge_probability: float) -> Dict:
        """Generate Erdős-Rényi random graph."""
        edges = []
        capacities = {}
        costs = {}

        for u in range(n_nodes):
            for v in range(u + 1, n_nodes):
                if np.random.random() < edge_probability:
                    edge = (u, v)
                    edges.append(edge)
                    capacities[edge] = self.generate_capacity(10.0)
                    costs[edge] = self.generate_cost(1.0)

        source, sink = self.add_source_sink(edges, capacities, costs, n_nodes)

        return {
            'n_nodes': n_nodes + 2,
            'edges': edges,
            'capacities': capacities,
            'costs': costs,
            'source': source,
            'sink': sink,
            'properties': NetworkProperties(
                n_nodes=n_nodes + 2,
                n_edges=len(edges),
                density=len(edges) / (n_nodes * (n_nodes - 1) / 2),
                avg_capacity=np.mean(list(capacities.values())),
                avg_cost=np.mean(list(costs.values())),
                topology_type='erdos_renyi',
                parameters={'edge_probability': edge_probability}
            )
        }

    def generate_sparse_random(self, n_nodes: int, avg_degree: float) -> Dict:
        """Generate sparse random graph with specified average degree."""
        target_edges = int(n_nodes * avg_degree / 2)
        edges = []
        capacities = {}
        costs = {}
        edge_set = set()

        while len(edges) < target_edges:
            u = np.random.randint(0, n_nodes)
            v = np.random.randint(0, n_nodes)

            if u != v and (u, v) not in edge_set and (v, u) not in edge_set:
                edge = (min(u, v), max(u, v))
                edges.append(edge)
                edge_set.add(edge)
                capacities[edge] = self.generate_capacity(8.0)
                costs[edge] = self.generate_cost(1.2)

        source, sink = self.add_source_sink(edges, capacities, costs, n_nodes)

        return {
            'n_nodes': n_nodes + 2,
            'edges': edges,
            'capacities': capacities,
            'costs': costs,
            'source': source,
            'sink': sink,
            'properties': NetworkProperties(
                n_nodes=n_nodes + 2,
                n_edges=len(edges),
                density=len(edges) / (n_nodes * (n_nodes - 1) / 2),
                avg_capacity=np.mean(list(capacities.values())),
                avg_cost=np.mean(list(costs.values())),
                topology_type='sparse_random',
                parameters={'avg_degree': avg_degree}
            )
        }


class ScaleFreeNetworkGenerator(NetworkGenerator):
    """Generate scale-free networks using Barabási-Albert model."""

    def generate_barabasi_albert(self, n_nodes: int, m: int) -> Dict:
        """
        Generate Barabási-Albert scale-free network.
        m: number of edges to attach from a new node to existing nodes
        """
        if m >= n_nodes:
            m = n_nodes - 1

        edges = []
        capacities = {}
        costs = {}

        # Start with complete graph of m+1 nodes
        for u in range(m + 1):
            for v in range(u + 1, m + 1):
                edge = (u, v)
                edges.append(edge)
                capacities[edge] = self.generate_capacity(15.0)  # Higher capacity for early edges
                costs[edge] = self.generate_cost(0.8)

        # Add remaining nodes with preferential attachment
        degrees = [len([e for e in edges if u in e]) for u in range(m + 1)]

        for new_node in range(m + 1, n_nodes):
            # Calculate attachment probabilities
            total_degree = sum(degrees)
            probabilities = [deg / total_degree for deg in degrees]

            # Select m nodes to connect to (without replacement)
            targets = np.random.choice(
                new_node, size=min(m, new_node), replace=False, p=probabilities
            )

            for target in targets:
                edge = (target, new_node)
                edges.append(edge)
                # Hub nodes get higher capacity
                hub_bonus = 1.0 + degrees[target] / 10.0
                capacities[edge] = self.generate_capacity(10.0 * hub_bonus)
                costs[edge] = self.generate_cost(1.0 / hub_bonus)  # Lower cost for hubs

            # Update degrees
            degrees.append(m)
            for target in targets:
                degrees[target] += 1

        source, sink = self.add_source_sink(edges, capacities, costs, n_nodes)

        return {
            'n_nodes': n_nodes + 2,
            'edges': edges,
            'capacities': capacities,
            'costs': costs,
            'source': source,
            'sink': sink,
            'properties': NetworkProperties(
                n_nodes=n_nodes + 2,
                n_edges=len(edges),
                density=len(edges) / (n_nodes * (n_nodes - 1) / 2),
                avg_capacity=np.mean(list(capacities.values())),
                avg_cost=np.mean(list(costs.values())),
                topology_type='barabasi_albert',
                parameters={'m': m}
            )
        }


class HierarchicalNetworkGenerator(NetworkGenerator):
    """Generate hierarchical networks with multiple levels."""

    def generate_tree_hierarchy(self, levels: int, branching_factor: int) -> Dict:
        """Generate tree-like hierarchical network."""
        nodes_per_level = [branching_factor ** i for i in range(levels)]
        total_nodes = sum(nodes_per_level)

        edges = []
        capacities = {}
        costs = {}

        node_id = 0
        level_start = [0]

        # Create hierarchical connections
        for level in range(levels - 1):
            level_nodes = nodes_per_level[level]
            next_level_nodes = nodes_per_level[level + 1]

            next_level_start = level_start[-1] + level_nodes
            level_start.append(next_level_start)

            # Connect each node in current level to branching_factor nodes in next level
            for i in range(level_nodes):
                current_node = level_start[level] + i

                for j in range(branching_factor):
                    if next_level_start + i * branching_factor + j < total_nodes:
                        child_node = next_level_start + i * branching_factor + j
                        edge = (current_node, child_node)
                        edges.append(edge)

                        # Capacity decreases with depth, cost increases
                        capacity_factor = 2.0 ** (levels - level - 1)
                        cost_factor = 1.0 + level * 0.5

                        capacities[edge] = self.generate_capacity(15.0 * capacity_factor)
                        costs[edge] = self.generate_cost(cost_factor)

        # Add some cross-connections within levels
        for level in range(levels):
            level_nodes = nodes_per_level[level]
            level_start_idx = level_start[level]

            # Add random connections within level
            cross_connections = max(1, level_nodes // 4)
            for _ in range(cross_connections):
                u = level_start_idx + np.random.randint(0, level_nodes)
                v = level_start_idx + np.random.randint(0, level_nodes)

                if u != v:
                    edge = (min(u, v), max(u, v))
                    if edge not in capacities:
                        edges.append(edge)
                        capacities[edge] = self.generate_capacity(5.0)
                        costs[edge] = self.generate_cost(2.0)

        source, sink = self.add_source_sink(edges, capacities, costs, total_nodes)

        return {
            'n_nodes': total_nodes + 2,
            'edges': edges,
            'capacities': capacities,
            'costs': costs,
            'source': source,
            'sink': sink,
            'properties': NetworkProperties(
                n_nodes=total_nodes + 2,
                n_edges=len(edges),
                density=len(edges) / (total_nodes * (total_nodes - 1) / 2),
                avg_capacity=np.mean(list(capacities.values())),
                avg_cost=np.mean(list(costs.values())),
                topology_type='tree_hierarchy',
                parameters={'levels': levels, 'branching_factor': branching_factor}
            )
        }


class SmallWorldNetworkGenerator(NetworkGenerator):
    """Generate small-world networks using Watts-Strogatz model."""

    def generate_watts_strogatz(self, n_nodes: int, k: int, p: float) -> Dict:
        """
        Generate Watts-Strogatz small-world network.
        k: each node connected to k nearest neighbors in ring topology
        p: probability of rewiring each edge
        """
        if k >= n_nodes:
            k = n_nodes - 1

        edges = []
        capacities = {}
        costs = {}

        # Start with regular ring lattice
        for i in range(n_nodes):
            for j in range(1, k // 2 + 1):
                neighbor = (i + j) % n_nodes
                edge = (min(i, neighbor), max(i, neighbor))

                if edge not in capacities:
                    edges.append(edge)
                    capacities[edge] = self.generate_capacity(12.0)
                    costs[edge] = self.generate_cost(1.0)

        # Rewire edges with probability p
        original_edges = edges.copy()
        for edge in original_edges:
            if np.random.random() < p:
                u, v = edge

                # Remove original edge
                del capacities[edge]
                edges.remove(edge)

                # Add new random edge
                new_target = np.random.randint(0, n_nodes)
                while new_target == u or (min(u, new_target), max(u, new_target)) in capacities:
                    new_target = np.random.randint(0, n_nodes)

                new_edge = (min(u, new_target), max(u, new_target))
                edges.append(new_edge)
                capacities[new_edge] = self.generate_capacity(8.0)  # Rewired edges have different capacity
                costs[new_edge] = self.generate_cost(1.5)  # Higher cost for long-range connections

        source, sink = self.add_source_sink(edges, capacities, costs, n_nodes)

        return {
            'n_nodes': n_nodes + 2,
            'edges': edges,
            'capacities': capacities,
            'costs': costs,
            'source': source,
            'sink': sink,
            'properties': NetworkProperties(
                n_nodes=n_nodes + 2,
                n_edges=len(edges),
                density=len(edges) / (n_nodes * (n_nodes - 1) / 2),
                avg_capacity=np.mean(list(capacities.values())),
                avg_cost=np.mean(list(costs.values())),
                topology_type='watts_strogatz',
                parameters={'k': k, 'p': p}
            )
        }


class NetworkSuite:
    """Generate comprehensive suite of test networks."""

    def __init__(self, seed: int = 42):
        self.seed = seed
        self.generators = {
            'grid': GridNetworkGenerator(seed),
            'random': RandomNetworkGenerator(seed),
            'scale_free': ScaleFreeNetworkGenerator(seed),
            'hierarchical': HierarchicalNetworkGenerator(seed),
            'small_world': SmallWorldNetworkGenerator(seed)
        }

    def generate_test_suite(self, size_category: str = 'medium') -> Dict[str, Dict]:
        """Generate comprehensive test suite for specified size category."""

        size_configs = {
            'small': {'nodes': 20, 'grid_size': (4, 5), 'levels': 3},
            'medium': {'nodes': 50, 'grid_size': (7, 7), 'levels': 4},
            'large': {'nodes': 100, 'grid_size': (10, 10), 'levels': 5},
            'xlarge': {'nodes': 200, 'grid_size': (14, 14), 'levels': 6}
        }

        config = size_configs.get(size_category, size_configs['medium'])
        n_nodes = config['nodes']
        grid_w, grid_h = config['grid_size']
        levels = config['levels']

        suite = {}

        # 2D Grid
        suite['2d_grid'] = self.generators['grid'].generate_2d_grid(grid_w, grid_h)

        # 3D Grid (smaller for computational efficiency)
        cube_size = int(n_nodes ** (1/3)) + 1
        suite['3d_grid'] = self.generators['grid'].generate_3d_grid(cube_size, cube_size, cube_size)

        # Random graphs
        suite['erdos_renyi_sparse'] = self.generators['random'].generate_erdos_renyi(n_nodes, 0.1)
        suite['erdos_renyi_dense'] = self.generators['random'].generate_erdos_renyi(n_nodes, 0.3)
        suite['sparse_random'] = self.generators['random'].generate_sparse_random(n_nodes, 4.0)

        # Scale-free
        suite['barabasi_albert'] = self.generators['scale_free'].generate_barabasi_albert(n_nodes, 3)

        # Hierarchical
        suite['tree_hierarchy'] = self.generators['hierarchical'].generate_tree_hierarchy(levels, 3)

        # Small-world
        suite['small_world'] = self.generators['small_world'].generate_watts_strogatz(n_nodes, 6, 0.3)

        return suite

    def save_networks_to_files(self, networks: Dict[str, Dict], output_dir: str):
        """Save generated networks to files for later use."""
        import json
        import os

        os.makedirs(output_dir, exist_ok=True)

        for name, network in networks.items():
            # Convert numpy types to JSON-serializable types
            json_network = {
                'n_nodes': int(network['n_nodes']),
                'edges': [(int(u), int(v)) for u, v in network['edges']],
                'capacities': {f"{u},{v}": float(cap) for (u, v), cap in network['capacities'].items()},
                'costs': {f"{u},{v}": float(cost) for (u, v), cost in network['costs'].items()},
                'source': int(network['source']),
                'sink': int(network['sink']),
                'properties': {
                    'n_nodes': int(network['properties'].n_nodes),
                    'n_edges': int(network['properties'].n_edges),
                    'density': float(network['properties'].density),
                    'avg_capacity': float(network['properties'].avg_capacity),
                    'avg_cost': float(network['properties'].avg_cost),
                    'topology_type': network['properties'].topology_type,
                    'parameters': network['properties'].parameters
                }
            }

            filename = os.path.join(output_dir, f"{name}.json")
            with open(filename, 'w') as f:
                json.dump(json_network, f, indent=2)

        print(f"Saved {len(networks)} networks to {output_dir}")


if __name__ == "__main__":
    # Example usage and testing
    print("Network Flow Graph Generators")
    print("=" * 40)

    suite = NetworkSuite(seed=42)

    # Generate test networks
    networks = suite.generate_test_suite('small')

    print(f"Generated {len(networks)} test networks:")
    for name, network in networks.items():
        props = network['properties']
        print(f"  {name:20}: {props.n_nodes:3d} nodes, {props.n_edges:3d} edges, "
              f"density={props.density:.3f}")

    # Test a specific generator
    print(f"\nTesting Grid Generator:")
    grid_gen = GridNetworkGenerator(seed=42)
    grid_2d = grid_gen.generate_2d_grid(5, 4, diagonal_connections=True)
    print(f"  2D Grid (5x4): {grid_2d['n_nodes']} nodes, {len(grid_2d['edges'])} edges")

    grid_3d = grid_gen.generate_3d_grid(3, 3, 3)
    print(f"  3D Grid (3x3x3): {grid_3d['n_nodes']} nodes, {len(grid_3d['edges'])} edges")

    # Test scale-free generator
    print(f"\nTesting Scale-Free Generator:")
    sf_gen = ScaleFreeNetworkGenerator(seed=42)
    sf_network = sf_gen.generate_barabasi_albert(30, 2)
    print(f"  Barabási-Albert (n=30, m=2): {sf_network['n_nodes']} nodes, {len(sf_network['edges'])} edges")

    # Test small-world generator
    print(f"\nTesting Small-World Generator:")
    sw_gen = SmallWorldNetworkGenerator(seed=42)
    sw_network = sw_gen.generate_watts_strogatz(20, 4, 0.3)
    print(f"  Watts-Strogatz (n=20, k=4, p=0.3): {sw_network['n_nodes']} nodes, {len(sw_network['edges'])} edges")