#!/usr/bin/env python3
"""
Traditional Network Flow Algorithms

Implements classical flow algorithms using NetworkX and OR-Tools for comparison
with sublinear solvers. Includes maximum flow, minimum cost flow, and multi-commodity
flow problems.

Authors: Network Flow Comparison Agent
Created: 2025-09-19
"""

import time
import numpy as np
import networkx as nx
from typing import Dict, List, Tuple, Optional, Union
from dataclasses import dataclass
from collections import defaultdict
import json
import warnings

try:
    from ortools.graph import pywrapgraph
    ORTOOLS_AVAILABLE = True
except ImportError:
    ORTOOLS_AVAILABLE = False
    warnings.warn("OR-Tools not available. Some algorithms will be unavailable.")


@dataclass
class FlowResult:
    """Container for flow algorithm results"""
    algorithm: str
    max_flow_value: Optional[float] = None
    min_cost: Optional[float] = None
    flow_dict: Optional[Dict] = None
    execution_time: float = 0.0
    iterations: int = 0
    memory_usage: float = 0.0
    convergence_info: Optional[Dict] = None
    

class TraditionalFlowSolver:
    """Traditional network flow algorithms implementation"""
    
    def __init__(self):
        self.results_history = []
        
    def maximum_flow_ford_fulkerson(self, 
                                   G: nx.DiGraph, 
                                   source: int, 
                                   sink: int,
                                   capacity_attr: str = 'capacity') -> FlowResult:
        """Solve maximum flow using Ford-Fulkerson (NetworkX implementation)"""
        start_time = time.time()
        
        try:
            flow_value, flow_dict = nx.maximum_flow(G, source, sink, capacity=capacity_attr)
            
            result = FlowResult(
                algorithm="Ford-Fulkerson (NetworkX)",
                max_flow_value=flow_value,
                flow_dict=flow_dict,
                execution_time=time.time() - start_time
            )
            
        except Exception as e:
            result = FlowResult(
                algorithm="Ford-Fulkerson (NetworkX)",
                execution_time=time.time() - start_time,
                convergence_info={"error": str(e)}
            )
            
        self.results_history.append(result)
        return result
    
    def maximum_flow_dinic(self, 
                          G: nx.DiGraph, 
                          source: int, 
                          sink: int,
                          capacity_attr: str = 'capacity') -> FlowResult:
        """Solve maximum flow using Dinic's algorithm (NetworkX)"""
        start_time = time.time()
        
        try:
            # NetworkX uses Dinic's algorithm by default for maximum_flow
            flow_value, flow_dict = nx.maximum_flow(G, source, sink, 
                                                   capacity=capacity_attr,
                                                   flow_func=nx.algorithms.flow.dinitz)
            
            result = FlowResult(
                algorithm="Dinic's Algorithm (NetworkX)",
                max_flow_value=flow_value,
                flow_dict=flow_dict,
                execution_time=time.time() - start_time
            )
            
        except Exception as e:
            result = FlowResult(
                algorithm="Dinic's Algorithm (NetworkX)",
                execution_time=time.time() - start_time,
                convergence_info={"error": str(e)}
            )
            
        self.results_history.append(result)
        return result
    
    def minimum_cost_flow_networkx(self, 
                                  G: nx.DiGraph,
                                  demand_dict: Dict[int, float],
                                  capacity_attr: str = 'capacity',
                                  weight_attr: str = 'weight') -> FlowResult:
        """Solve minimum cost flow using NetworkX"""
        start_time = time.time()
        
        try:
            # Set node demands
            for node, demand in demand_dict.items():
                G.nodes[node]['demand'] = demand
                
            flow_dict = nx.min_cost_flow(G, capacity=capacity_attr, weight=weight_attr)
            
            # Calculate total cost
            total_cost = 0
            for u in flow_dict:
                for v in flow_dict[u]:
                    if flow_dict[u][v] > 0:
                        total_cost += flow_dict[u][v] * G[u][v].get(weight_attr, 1)
            
            result = FlowResult(
                algorithm="Min Cost Flow (NetworkX)",
                min_cost=total_cost,
                flow_dict=flow_dict,
                execution_time=time.time() - start_time
            )
            
        except Exception as e:
            result = FlowResult(
                algorithm="Min Cost Flow (NetworkX)",
                execution_time=time.time() - start_time,
                convergence_info={"error": str(e)}
            )
            
        self.results_history.append(result)
        return result
    
    def minimum_cost_flow_ortools(self, 
                                 G: nx.DiGraph,
                                 demand_dict: Dict[int, float],
                                 capacity_attr: str = 'capacity',
                                 weight_attr: str = 'weight') -> FlowResult:
        """Solve minimum cost flow using OR-Tools"""
        if not ORTOOLS_AVAILABLE:
            return FlowResult(
                algorithm="Min Cost Flow (OR-Tools)",
                convergence_info={"error": "OR-Tools not available"}
            )
            
        start_time = time.time()
        
        try:
            # Create OR-Tools solver
            min_cost_flow = pywrapgraph.SimpleMinCostFlow()
            
            # Node mapping
            node_to_index = {node: i for i, node in enumerate(G.nodes())}
            index_to_node = {i: node for node, i in node_to_index.items()}
            
            # Add arcs
            for u, v, data in G.edges(data=True):
                capacity = data.get(capacity_attr, float('inf'))
                cost = data.get(weight_attr, 1)
                
                if capacity != float('inf'):
                    min_cost_flow.AddArcWithCapacityAndUnitCost(
                        node_to_index[u], node_to_index[v], int(capacity), int(cost)
                    )
            
            # Set supplies/demands
            for node, demand in demand_dict.items():
                min_cost_flow.SetNodeSupply(node_to_index[node], int(-demand))
            
            # Solve
            status = min_cost_flow.Solve()
            
            if status == min_cost_flow.OPTIMAL:
                # Extract solution
                flow_dict = defaultdict(lambda: defaultdict(int))
                total_cost = min_cost_flow.OptimalCost()
                
                for i in range(min_cost_flow.NumArcs()):
                    if min_cost_flow.Flow(i) > 0:
                        tail = index_to_node[min_cost_flow.Tail(i)]
                        head = index_to_node[min_cost_flow.Head(i)]
                        flow_dict[tail][head] = min_cost_flow.Flow(i)
                
                result = FlowResult(
                    algorithm="Min Cost Flow (OR-Tools)",
                    min_cost=total_cost,
                    flow_dict=dict(flow_dict),
                    execution_time=time.time() - start_time
                )
            else:
                result = FlowResult(
                    algorithm="Min Cost Flow (OR-Tools)",
                    execution_time=time.time() - start_time,
                    convergence_info={"error": f"Solver status: {status}"}
                )
                
        except Exception as e:
            result = FlowResult(
                algorithm="Min Cost Flow (OR-Tools)",
                execution_time=time.time() - start_time,
                convergence_info={"error": str(e)}
            )
            
        self.results_history.append(result)
        return result
    
    def multi_commodity_flow(self, 
                           G: nx.DiGraph,
                           commodities: List[Dict],
                           capacity_attr: str = 'capacity') -> FlowResult:
        """Solve multi-commodity flow problem
        
        Args:
            commodities: List of dicts with 'source', 'sink', 'demand' keys
        """
        start_time = time.time()
        
        try:
            # Simple implementation using successive shortest paths
            total_flow_dict = defaultdict(lambda: defaultdict(float))
            total_cost = 0
            
            for i, commodity in enumerate(commodities):
                source = commodity['source']
                sink = commodity['sink']
                demand = commodity['demand']
                
                # Create temporary graph with remaining capacities
                temp_G = G.copy()
                for u, v in temp_G.edges():
                    current_flow = total_flow_dict[u][v]
                    remaining_capacity = temp_G[u][v][capacity_attr] - current_flow
                    temp_G[u][v][capacity_attr] = max(0, remaining_capacity)
                
                # Try to route this commodity
                try:
                    flow_value, flow_dict = nx.maximum_flow(temp_G, source, sink, 
                                                           capacity=capacity_attr)
                    
                    if flow_value >= demand:
                        # Scale flow to exact demand
                        scale_factor = demand / flow_value if flow_value > 0 else 0
                        for u in flow_dict:
                            for v in flow_dict[u]:
                                if flow_dict[u][v] > 0:
                                    scaled_flow = flow_dict[u][v] * scale_factor
                                    total_flow_dict[u][v] += scaled_flow
                    else:
                        # Partial routing
                        for u in flow_dict:
                            for v in flow_dict[u]:
                                if flow_dict[u][v] > 0:
                                    total_flow_dict[u][v] += flow_dict[u][v]
                                    
                except nx.NetworkXError:
                    continue
            
            result = FlowResult(
                algorithm="Multi-Commodity Flow (Greedy)",
                flow_dict=dict(total_flow_dict),
                execution_time=time.time() - start_time
            )
            
        except Exception as e:
            result = FlowResult(
                algorithm="Multi-Commodity Flow (Greedy)",
                execution_time=time.time() - start_time,
                convergence_info={"error": str(e)}
            )
            
        self.results_history.append(result)
        return result
    
    def edge_disjoint_paths(self, 
                           G: nx.Graph,
                           source: int,
                           target: int) -> FlowResult:
        """Find maximum number of edge-disjoint paths"""
        start_time = time.time()
        
        try:
            paths = list(nx.edge_disjoint_paths(G, source, target))
            
            result = FlowResult(
                algorithm="Edge Disjoint Paths (NetworkX)",
                max_flow_value=len(paths),
                flow_dict={'paths': [list(path) for path in paths]},
                execution_time=time.time() - start_time
            )
            
        except Exception as e:
            result = FlowResult(
                algorithm="Edge Disjoint Paths (NetworkX)",
                execution_time=time.time() - start_time,
                convergence_info={"error": str(e)}
            )
            
        self.results_history.append(result)
        return result
    
    def node_connectivity(self, G: nx.Graph, source: int, target: int) -> FlowResult:
        """Compute node connectivity between source and target"""
        start_time = time.time()
        
        try:
            connectivity = nx.node_connectivity(G, source, target)
            
            result = FlowResult(
                algorithm="Node Connectivity (NetworkX)",
                max_flow_value=connectivity,
                execution_time=time.time() - start_time
            )
            
        except Exception as e:
            result = FlowResult(
                algorithm="Node Connectivity (NetworkX)",
                execution_time=time.time() - start_time,
                convergence_info={"error": str(e)}
            )
            
        self.results_history.append(result)
        return result
    
    def benchmark_all_algorithms(self, 
                                G: nx.DiGraph,
                                source: int,
                                sink: int,
                                demand_dict: Optional[Dict[int, float]] = None,
                                commodities: Optional[List[Dict]] = None) -> Dict[str, FlowResult]:
        """Run all available algorithms for comparison"""
        results = {}
        
        # Maximum flow algorithms
        results['ford_fulkerson'] = self.maximum_flow_ford_fulkerson(G, source, sink)
        results['dinic'] = self.maximum_flow_dinic(G, source, sink)
        
        # Minimum cost flow (if demands provided)
        if demand_dict:
            results['min_cost_networkx'] = self.minimum_cost_flow_networkx(G, demand_dict)
            if ORTOOLS_AVAILABLE:
                results['min_cost_ortools'] = self.minimum_cost_flow_ortools(G, demand_dict)
        
        # Multi-commodity flow (if commodities provided)
        if commodities:
            results['multi_commodity'] = self.multi_commodity_flow(G, commodities)
        
        # Edge disjoint paths (convert to undirected)
        if not G.is_directed() or nx.is_weakly_connected(G):
            undirected_G = G.to_undirected() if G.is_directed() else G
            results['edge_disjoint'] = self.edge_disjoint_paths(undirected_G, source, sink)
            results['node_connectivity'] = self.node_connectivity(undirected_G, source, sink)
        
        return results
    
    def export_results(self, filename: str) -> None:
        """Export results history to JSON file"""
        export_data = []
        for result in self.results_history:
            # Convert result to JSON-serializable format
            result_dict = {
                'algorithm': result.algorithm,
                'max_flow_value': result.max_flow_value,
                'min_cost': result.min_cost,
                'execution_time': result.execution_time,
                'iterations': result.iterations,
                'memory_usage': result.memory_usage,
                'convergence_info': result.convergence_info,
                'flow_dict_size': len(result.flow_dict) if result.flow_dict else 0
            }
            export_data.append(result_dict)
        
        with open(filename, 'w') as f:
            json.dump(export_data, f, indent=2)
    
    def clear_history(self) -> None:
        """Clear results history"""
        self.results_history = []


def create_sample_network(n_nodes: int = 10, 
                         edge_probability: float = 0.3,
                         capacity_range: Tuple[int, int] = (1, 10),
                         cost_range: Tuple[int, int] = (1, 5)) -> nx.DiGraph:
    """Create a sample directed graph for testing"""
    G = nx.erdos_renyi_graph(n_nodes, edge_probability, directed=True)
    
    # Add random capacities and costs
    for u, v in G.edges():
        G[u][v]['capacity'] = np.random.randint(*capacity_range)
        G[u][v]['weight'] = np.random.randint(*cost_range)
    
    return G


if __name__ == "__main__":
    # Example usage and testing
    print("Traditional Flow Algorithms Demo")
    print("=" * 40)
    
    # Create sample network
    G = create_sample_network(8, 0.4)
    source, sink = 0, 7
    
    # Ensure source and sink are connected
    if not nx.has_path(G, source, sink):
        # Add a path
        path_nodes = list(range(source, sink + 1))
        for i in range(len(path_nodes) - 1):
            if not G.has_edge(path_nodes[i], path_nodes[i + 1]):
                G.add_edge(path_nodes[i], path_nodes[i + 1], 
                          capacity=5, weight=1)
    
    print(f"Network: {G.number_of_nodes()} nodes, {G.number_of_edges()} edges")
    print(f"Source: {source}, Sink: {sink}")
    
    # Initialize solver
    solver = TraditionalFlowSolver()
    
    # Test maximum flow
    print("\nMaximum Flow Results:")
    ff_result = solver.maximum_flow_ford_fulkerson(G, source, sink)
    print(f"Ford-Fulkerson: {ff_result.max_flow_value} (time: {ff_result.execution_time:.4f}s)")
    
    dinic_result = solver.maximum_flow_dinic(G, source, sink)
    print(f"Dinic's Algorithm: {dinic_result.max_flow_value} (time: {dinic_result.execution_time:.4f}s)")
    
    # Test minimum cost flow
    print("\nMinimum Cost Flow Results:")
    demand_dict = {source: -5, sink: 5}  # Source supplies 5, sink demands 5
    
    mcf_result = solver.minimum_cost_flow_networkx(G, demand_dict)
    if mcf_result.min_cost is not None:
        print(f"NetworkX Min Cost: {mcf_result.min_cost} (time: {mcf_result.execution_time:.4f}s)")
    else:
        print(f"NetworkX Min Cost: Failed - {mcf_result.convergence_info}")
    
    if ORTOOLS_AVAILABLE:
        ortools_result = solver.minimum_cost_flow_ortools(G, demand_dict)
        if ortools_result.min_cost is not None:
            print(f"OR-Tools Min Cost: {ortools_result.min_cost} (time: {ortools_result.execution_time:.4f}s)")
        else:
            print(f"OR-Tools Min Cost: Failed - {ortools_result.convergence_info}")
    
    # Export results
    solver.export_results("/workspaces/sublinear-time-solver/scripts/network_flow/traditional_results.json")
    print("\nResults exported to traditional_results.json")
