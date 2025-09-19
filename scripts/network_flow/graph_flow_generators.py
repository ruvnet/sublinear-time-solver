#!/usr/bin/env python3
"""
Realistic Network Topology Generators

Generates various types of network topologies for flow algorithm testing,
including grid networks, scale-free networks, road networks, and supply chains.

Authors: Network Flow Comparison Agent
Created: 2025-09-19
"""

import numpy as np
import networkx as nx
from typing import Dict, List, Tuple, Optional, Union
import random
import math
from dataclasses import dataclass
from collections import defaultdict
import json


@dataclass
class NetworkMetrics:
    """Container for network topology metrics"""
    nodes: int
    edges: int
    avg_degree: float
    diameter: Optional[int]
    clustering_coefficient: float
    density: float
    is_connected: bool
    is_directed: bool
    max_flow_bound: Optional[float] = None
    sparsity: float = 0.0


class NetworkTopologyGenerator:
    """Generator for various network topologies"""
    
    def __init__(self, seed: Optional[int] = None):
        if seed is not None:
            random.seed(seed)
            np.random.seed(seed)
    
    def generate_grid_network(self, 
                             rows: int, 
                             cols: int,
                             directed: bool = True,
                             capacity_range: Tuple[int, int] = (1, 10),
                             cost_range: Tuple[int, int] = (1, 5),
                             bottleneck_probability: float = 0.1) -> nx.DiGraph:
        """Generate grid network with optional bottlenecks"""
        if directed:
            G = nx.grid_2d_graph(rows, cols, create_using=nx.DiGraph())
        else:
            G = nx.grid_2d_graph(rows, cols)
            
        # Add reverse edges for undirected-like behavior in directed case
        if directed:
            edges_to_add = []
            for u, v in list(G.edges()):
                edges_to_add.append((v, u))
            G.add_edges_from(edges_to_add)
        
        # Add random capacities and costs
        for u, v in G.edges():
            if random.random() < bottleneck_probability:
                # Create bottleneck
                capacity = max(1, capacity_range[0] // 2)
                cost = cost_range[1] * 2
            else:
                capacity = random.randint(*capacity_range)
                cost = random.randint(*cost_range)
            
            G[u][v]['capacity'] = capacity
            G[u][v]['weight'] = cost
            G[u][v]['resistance'] = cost * 0.1  # For electrical analysis
        
        # Relabel nodes to integers
        mapping = {node: i for i, node in enumerate(G.nodes())}
        G = nx.relabel_nodes(G, mapping)
        
        return G
    
    def generate_scale_free_network(self,
                                   n_nodes: int,
                                   attachment_parameter: int = 3,
                                   directed: bool = True,
                                   capacity_range: Tuple[int, int] = (1, 15),
                                   cost_range: Tuple[int, int] = (1, 8)) -> nx.DiGraph:
        """Generate scale-free network using preferential attachment"""
        if directed:
            G = nx.barabasi_albert_graph(n_nodes, attachment_parameter, create_using=nx.DiGraph())
        else:
            G = nx.barabasi_albert_graph(n_nodes, attachment_parameter)
        
        # Add capacities and costs based on node degrees
        for u, v in G.edges():
            # Higher degree nodes get higher capacities
            degree_factor = (G.degree(u) + G.degree(v)) / (2 * attachment_parameter)
            
            capacity = int(capacity_range[0] + (capacity_range[1] - capacity_range[0]) * min(degree_factor, 1.0))
            cost = random.randint(*cost_range)
            
            G[u][v]['capacity'] = capacity
            G[u][v]['weight'] = cost
            G[u][v]['resistance'] = cost * 0.1
        
        return G
    
    def generate_small_world_network(self,
                                    n_nodes: int,
                                    k_neighbors: int = 4,
                                    rewire_probability: float = 0.3,
                                    directed: bool = True,
                                    capacity_range: Tuple[int, int] = (1, 10),
                                    cost_range: Tuple[int, int] = (1, 5)) -> nx.DiGraph:
        """Generate small-world network (Watts-Strogatz model)"""
        if directed:
            G = nx.watts_strogatz_graph(n_nodes, k_neighbors, rewire_probability)
            G = G.to_directed()
        else:
            G = nx.watts_strogatz_graph(n_nodes, k_neighbors, rewire_probability)
        
        # Add random capacities and costs
        for u, v in G.edges():
            G[u][v]['capacity'] = random.randint(*capacity_range)
            G[u][v]['weight'] = random.randint(*cost_range)
            G[u][v]['resistance'] = G[u][v]['weight'] * 0.1
        
        return G
    
    def generate_road_network(self,
                             n_intersections: int,
                             highway_probability: float = 0.15,
                             local_road_probability: float = 0.4,
                             directed: bool = True) -> nx.DiGraph:
        """Generate realistic road network topology"""
        # Start with random geometric graph
        pos = {i: (random.random(), random.random()) for i in range(n_intersections)}
        
        if directed:
            G = nx.DiGraph()
        else:
            G = nx.Graph()
        
        G.add_nodes_from(range(n_intersections))
        
        # Add highways (long-distance, high-capacity connections)
        for i in range(n_intersections):
            for j in range(i + 1, n_intersections):
                distance = math.sqrt((pos[i][0] - pos[j][0])**2 + (pos[i][1] - pos[j][1])**2)
                
                if distance < 0.3 and random.random() < highway_probability:
                    # Highway connection
                    capacity = random.randint(50, 100)  # High capacity
                    travel_time = max(1, int(distance * 10))  # Cost based on distance
                    
                    G.add_edge(i, j, capacity=capacity, weight=travel_time, 
                              road_type='highway', resistance=travel_time * 0.01)
                    
                    if directed:
                        G.add_edge(j, i, capacity=capacity, weight=travel_time,
                                  road_type='highway', resistance=travel_time * 0.01)
                
                elif distance < 0.2 and random.random() < local_road_probability:
                    # Local road connection
                    capacity = random.randint(10, 30)  # Lower capacity
                    travel_time = max(1, int(distance * 20))
                    
                    G.add_edge(i, j, capacity=capacity, weight=travel_time,
                              road_type='local', resistance=travel_time * 0.05)
                    
                    if directed:
                        G.add_edge(j, i, capacity=capacity, weight=travel_time,
                                  road_type='local', resistance=travel_time * 0.05)
        
        # Ensure connectivity by adding minimum spanning tree
        if not nx.is_connected(G.to_undirected()):
            mst_edges = nx.minimum_spanning_edges(nx.complete_graph(n_intersections), 
                                                 weight=lambda u, v, d: math.sqrt((pos[u][0] - pos[v][0])**2 + (pos[u][1] - pos[v][1])**2))
            
            for u, v in mst_edges:
                if not G.has_edge(u, v):
                    distance = math.sqrt((pos[u][0] - pos[v][0])**2 + (pos[u][1] - pos[v][1])**2)
                    capacity = random.randint(5, 15)
                    travel_time = max(1, int(distance * 25))
                    
                    G.add_edge(u, v, capacity=capacity, weight=travel_time,
                              road_type='connector', resistance=travel_time * 0.1)
                    
                    if directed:
                        G.add_edge(v, u, capacity=capacity, weight=travel_time,
                                  road_type='connector', resistance=travel_time * 0.1)
        
        return G
    
    def generate_supply_chain_network(self,
                                     n_suppliers: int = 5,
                                     n_warehouses: int = 8,
                                     n_retailers: int = 12,
                                     supply_capacity_range: Tuple[int, int] = (20, 50),
                                     warehouse_capacity_range: Tuple[int, int] = (100, 200),
                                     retail_demand_range: Tuple[int, int] = (5, 25)) -> nx.DiGraph:
        """Generate multi-level supply chain network"""
        G = nx.DiGraph()
        
        # Create node layers
        suppliers = list(range(n_suppliers))
        warehouses = list(range(n_suppliers, n_suppliers + n_warehouses))
        retailers = list(range(n_suppliers + n_warehouses, n_suppliers + n_warehouses + n_retailers))
        
        # Add all nodes
        G.add_nodes_from(suppliers, node_type='supplier')
        G.add_nodes_from(warehouses, node_type='warehouse')
        G.add_nodes_from(retailers, node_type='retailer')
        
        # Supplier to warehouse connections
        for supplier in suppliers:
            # Each supplier connects to 2-4 warehouses
            connected_warehouses = random.sample(warehouses, random.randint(2, min(4, len(warehouses))))
            for warehouse in connected_warehouses:
                capacity = random.randint(*supply_capacity_range)
                cost = random.randint(2, 8)
                G.add_edge(supplier, warehouse, capacity=capacity, weight=cost,
                          connection_type='supply', resistance=cost * 0.1)
        
        # Warehouse to retailer connections
        for warehouse in warehouses:
            # Each warehouse connects to 3-6 retailers
            connected_retailers = random.sample(retailers, random.randint(3, min(6, len(retailers))))
            for retailer in connected_retailers:
                capacity = random.randint(10, 40)
                cost = random.randint(1, 6)
                G.add_edge(warehouse, retailer, capacity=capacity, weight=cost,
                          connection_type='distribution', resistance=cost * 0.1)
        
        # Warehouse to warehouse connections (redistribution)
        for i, warehouse1 in enumerate(warehouses):
            for warehouse2 in warehouses[i+1:]:
                if random.random() < 0.3:  # 30% chance of inter-warehouse connection
                    capacity = random.randint(*warehouse_capacity_range)
                    cost = random.randint(3, 10)
                    G.add_edge(warehouse1, warehouse2, capacity=capacity, weight=cost,
                              connection_type='redistribution', resistance=cost * 0.1)
                    G.add_edge(warehouse2, warehouse1, capacity=capacity, weight=cost,
                              connection_type='redistribution', resistance=cost * 0.1)
        
        return G
    
    def generate_electrical_circuit(self,
                                   circuit_type: str = "grid",
                                   n_nodes: int = 20,
                                   resistance_range: Tuple[float, float] = (0.1, 10.0)) -> nx.Graph:
        """Generate electrical circuit topology"""
        if circuit_type == "grid":
            # Grid-like circuit
            side = int(math.sqrt(n_nodes))
            G = nx.grid_2d_graph(side, side)
            
            # Relabel nodes to integers
            mapping = {node: i for i, node in enumerate(G.nodes())}
            G = nx.relabel_nodes(G, mapping)
            
        elif circuit_type == "ladder":
            # Ladder circuit
            G = nx.Graph()
            for i in range(n_nodes // 2):
                G.add_edge(2*i, 2*i + 1)  # Rungs
                if i < n_nodes // 2 - 1:
                    G.add_edge(2*i, 2*i + 2)      # Left rail
                    G.add_edge(2*i + 1, 2*i + 3)  # Right rail
        
        elif circuit_type == "bridge":
            # Bridge circuit (Wheatstone bridge-like)
            G = nx.Graph()
            G.add_edges_from([(0, 1), (0, 2), (1, 3), (2, 3), (1, 2)])  # Basic bridge
            
            # Add more nodes randomly
            for i in range(5, n_nodes):
                # Connect to 1-3 existing nodes
                existing_nodes = random.sample(list(G.nodes()), random.randint(1, min(3, len(G.nodes()))))
                for node in existing_nodes:
                    G.add_edge(i, node)
        
        else:  # random
            G = nx.erdos_renyi_graph(n_nodes, 0.3)
        
        # Add resistance values
        for u, v in G.edges():
            resistance = random.uniform(*resistance_range)
            G[u][v]['resistance'] = resistance
            G[u][v]['weight'] = resistance  # For compatibility
            G[u][v]['capacity'] = 1.0 / resistance  # Conductance as capacity
        
        return G
    
    def generate_hierarchical_network(self,
                                     levels: int = 3,
                                     nodes_per_level: List[int] = None,
                                     inter_level_connectivity: float = 0.8,
                                     intra_level_connectivity: float = 0.3) -> nx.DiGraph:
        """Generate hierarchical network topology"""
        if nodes_per_level is None:
            nodes_per_level = [5, 10, 20][:levels]
        
        G = nx.DiGraph()
        
        # Create levels
        level_nodes = []
        node_id = 0
        
        for level in range(levels):
            level_node_list = list(range(node_id, node_id + nodes_per_level[level]))
            level_nodes.append(level_node_list)
            
            # Add nodes with level attribute
            for node in level_node_list:
                G.add_node(node, level=level)
            
            node_id += nodes_per_level[level]
        
        # Add intra-level connections
        for level, nodes in enumerate(level_nodes):
            for i, node1 in enumerate(nodes):
                for node2 in nodes[i+1:]:
                    if random.random() < intra_level_connectivity:
                        capacity = random.randint(5, 15)
                        cost = random.randint(1, 5)
                        G.add_edge(node1, node2, capacity=capacity, weight=cost,
                                  connection_type='intra_level', resistance=cost * 0.1)
                        G.add_edge(node2, node1, capacity=capacity, weight=cost,
                                  connection_type='intra_level', resistance=cost * 0.1)
        
        # Add inter-level connections (top-down)
        for level in range(levels - 1):
            upper_nodes = level_nodes[level]
            lower_nodes = level_nodes[level + 1]
            
            for upper_node in upper_nodes:
                # Each upper node connects to multiple lower nodes
                n_connections = random.randint(2, min(4, len(lower_nodes)))
                connected_lower = random.sample(lower_nodes, n_connections)
                
                for lower_node in connected_lower:
                    if random.random() < inter_level_connectivity:
                        capacity = random.randint(10, 25)
                        cost = random.randint(2, 8)
                        G.add_edge(upper_node, lower_node, capacity=capacity, weight=cost,
                                  connection_type='inter_level', resistance=cost * 0.1)
        
        return G
    
    def calculate_network_metrics(self, G: Union[nx.Graph, nx.DiGraph]) -> NetworkMetrics:
        """Calculate comprehensive network metrics"""
        is_directed = G.is_directed()
        n_nodes = G.number_of_nodes()
        n_edges = G.number_of_edges()
        
        # Basic metrics
        if n_nodes > 1:
            avg_degree = 2 * n_edges / n_nodes if not is_directed else n_edges / n_nodes
            density = nx.density(G)
        else:
            avg_degree = 0
            density = 0
        
        # Connectivity
        if is_directed:
            is_connected = nx.is_weakly_connected(G)
        else:
            is_connected = nx.is_connected(G)
        
        # Diameter (only for connected graphs)
        diameter = None
        if is_connected and n_nodes > 1:
            try:
                if is_directed:
                    diameter = nx.diameter(G.to_undirected())
                else:
                    diameter = nx.diameter(G)
            except:
                diameter = None
        
        # Clustering coefficient
        try:
            if is_directed:
                clustering_coefficient = nx.average_clustering(G.to_undirected())
            else:
                clustering_coefficient = nx.average_clustering(G)
        except:
            clustering_coefficient = 0.0
        
        # Sparsity
        max_edges = n_nodes * (n_nodes - 1)
        if not is_directed:
            max_edges //= 2
        sparsity = 1.0 - (n_edges / max_edges) if max_edges > 0 else 0.0
        
        # Theoretical max flow bound (sum of minimum cuts)
        max_flow_bound = None
        if 'capacity' in next(iter(G.edges(data=True)), [None, None, {}])[2]:
            total_capacity = sum(data.get('capacity', 0) for _, _, data in G.edges(data=True))
            max_flow_bound = total_capacity
        
        return NetworkMetrics(
            nodes=n_nodes,
            edges=n_edges,
            avg_degree=avg_degree,
            diameter=diameter,
            clustering_coefficient=clustering_coefficient,
            density=density,
            is_connected=is_connected,
            is_directed=is_directed,
            max_flow_bound=max_flow_bound,
            sparsity=sparsity
        )
    
    def generate_test_suite(self, output_dir: str = "/workspaces/sublinear-time-solver/scripts/network_flow/test_networks/") -> Dict[str, nx.Graph]:
        """Generate comprehensive test suite of networks"""
        import os
        os.makedirs(output_dir, exist_ok=True)
        
        networks = {}
        
        # Small networks for detailed analysis
        networks['small_grid'] = self.generate_grid_network(4, 4, directed=True)
        networks['small_scale_free'] = self.generate_scale_free_network(15, 2, directed=True)
        networks['small_road'] = self.generate_road_network(12, highway_probability=0.2)
        
        # Medium networks for performance comparison
        networks['medium_grid'] = self.generate_grid_network(8, 8, directed=True)
        networks['medium_scale_free'] = self.generate_scale_free_network(50, 3, directed=True)
        networks['medium_supply_chain'] = self.generate_supply_chain_network(8, 12, 20)
        
        # Large sparse networks for sublinear advantage
        networks['large_grid'] = self.generate_grid_network(15, 15, directed=True)
        networks['large_scale_free'] = self.generate_scale_free_network(200, 4, directed=True)
        networks['large_hierarchical'] = self.generate_hierarchical_network(4, [10, 20, 40, 80])
        
        # Electrical circuits
        networks['electrical_grid'] = self.generate_electrical_circuit("grid", 25)
        networks['electrical_ladder'] = self.generate_electrical_circuit("ladder", 20)
        networks['electrical_bridge'] = self.generate_electrical_circuit("bridge", 30)
        
        # Export networks and calculate metrics
        metrics_summary = {}
        for name, network in networks.items():
            # Save network
            nx.write_gml(network, os.path.join(output_dir, f"{name}.gml"))
            
            # Calculate and save metrics
            metrics = self.calculate_network_metrics(network)
            metrics_summary[name] = {
                'nodes': metrics.nodes,
                'edges': metrics.edges,
                'avg_degree': metrics.avg_degree,
                'diameter': metrics.diameter,
                'clustering_coefficient': metrics.clustering_coefficient,
                'density': metrics.density,
                'is_connected': metrics.is_connected,
                'is_directed': metrics.is_directed,
                'sparsity': metrics.sparsity
            }
        
        # Save metrics summary
        with open(os.path.join(output_dir, "network_metrics.json"), 'w') as f:
            json.dump(metrics_summary, f, indent=2)
        
        return networks


if __name__ == "__main__":
    # Example usage and testing
    print("Network Topology Generator Demo")
    print("=" * 40)
    
    generator = NetworkTopologyGenerator(seed=42)
    
    # Generate different network types
    print("\nGenerating test networks...")
    
    # Grid network
    grid = generator.generate_grid_network(5, 5)
    grid_metrics = generator.calculate_network_metrics(grid)
    print(f"Grid Network: {grid_metrics.nodes} nodes, {grid_metrics.edges} edges, sparsity: {grid_metrics.sparsity:.3f}")
    
    # Scale-free network
    scale_free = generator.generate_scale_free_network(25, 3)
    sf_metrics = generator.calculate_network_metrics(scale_free)
    print(f"Scale-Free Network: {sf_metrics.nodes} nodes, {sf_metrics.edges} edges, sparsity: {sf_metrics.sparsity:.3f}")
    
    # Road network
    road = generator.generate_road_network(20)
    road_metrics = generator.calculate_network_metrics(road)
    print(f"Road Network: {road_metrics.nodes} nodes, {road_metrics.edges} edges, sparsity: {road_metrics.sparsity:.3f}")
    
    # Supply chain
    supply_chain = generator.generate_supply_chain_network(5, 8, 12)
    sc_metrics = generator.calculate_network_metrics(supply_chain)
    print(f"Supply Chain: {sc_metrics.nodes} nodes, {sc_metrics.edges} edges, sparsity: {sc_metrics.sparsity:.3f}")
    
    # Electrical circuit
    circuit = generator.generate_electrical_circuit("grid", 16)
    circuit_metrics = generator.calculate_network_metrics(circuit)
    print(f"Electrical Circuit: {circuit_metrics.nodes} nodes, {circuit_metrics.edges} edges, sparsity: {circuit_metrics.sparsity:.3f}")
    
    # Generate full test suite
    print("\nGenerating complete test suite...")
    networks = generator.generate_test_suite()
    print(f"Generated {len(networks)} test networks")
    
    print("\nTest networks saved to /workspaces/sublinear-time-solver/scripts/network_flow/test_networks/")
