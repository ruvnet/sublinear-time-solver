#!/usr/bin/env python3
"""
Traffic Flow Simulation and Optimization

Implements traffic flow modeling for road networks using flow conservation
principles. Compares traditional traffic assignment algorithms with
sublinear solver approaches.

Authors: Network Flow Comparison Agent
Created: 2025-09-19
"""

import time
import numpy as np
import networkx as nx
from typing import Dict, List, Tuple, Optional, Union, Any
from dataclasses import dataclass
from collections import defaultdict
import json
import math
from scipy.optimize import minimize
import warnings

try:
    from sublinear_flow import SublinearFlowSolver, SublinearFlowResult
except ImportError:
    class SublinearFlowSolver:
        def minimum_cost_flow_sublinear(self, *args, **kwargs):
            return {"error": "SublinearFlowSolver not available"}
    
    class SublinearFlowResult:
        pass


@dataclass
class TrafficFlowResult:
    """Container for traffic flow optimization results"""
    algorithm: str
    link_flows: Optional[Dict[Tuple[int, int], float]] = None
    travel_times: Optional[Dict[Tuple[int, int], float]] = None
    total_travel_time: Optional[float] = None
    total_delay: Optional[float] = None
    convergence_achieved: bool = False
    execution_time: float = 0.0
    iterations: int = 0
    convergence_info: Optional[Dict] = None
    network_performance: Optional[Dict] = None


class TrafficFlowSimulator:
    """Traffic flow simulation and optimization"""
    
    def __init__(self):
        self.results_history = []
        self.sublinear_solver = SublinearFlowSolver()
    
    def create_bpr_travel_time_function(self, 
                                       free_flow_time: float,
                                       capacity: float,
                                       alpha: float = 0.15,
                                       beta: float = 4.0):
        """Bureau of Public Roads (BPR) travel time function
        
        t(x) = t0 * (1 + alpha * (x/c)^beta)
        where t0 = free flow time, x = flow, c = capacity
        """
        def travel_time(flow):
            if capacity <= 0:
                return float('inf')
            volume_capacity_ratio = max(0, flow / capacity)
            return free_flow_time * (1 + alpha * (volume_capacity_ratio ** beta))
        
        def derivative(flow):
            if capacity <= 0:
                return float('inf')
            volume_capacity_ratio = max(0, flow / capacity)
            return free_flow_time * alpha * beta * (volume_capacity_ratio ** (beta - 1)) / capacity
        
        return travel_time, derivative
    
    def user_equilibrium_frank_wolfe(self,
                                    G: nx.DiGraph,
                                    od_demands: Dict[Tuple[int, int], float],
                                    capacity_attr: str = 'capacity',
                                    free_flow_time_attr: str = 'weight',
                                    max_iterations: int = 100,
                                    convergence_tolerance: float = 1e-4) -> TrafficFlowResult:
        """Solve user equilibrium using Frank-Wolfe algorithm"""
        start_time = time.time()
        
        try:
            # Initialize
            links = list(G.edges())
            link_flows = {link: 0.0 for link in links}
            
            # Create travel time functions for each link
            travel_time_functions = {}
            for u, v, data in G.edges(data=True):
                capacity = data.get(capacity_attr, 1.0)
                free_flow_time = data.get(free_flow_time_attr, 1.0)
                travel_time_func, _ = self.create_bpr_travel_time_function(free_flow_time, capacity)
                travel_time_functions[(u, v)] = travel_time_func
            
            iteration = 0
            converged = False
            
            while iteration < max_iterations and not converged:
                iteration += 1
                
                # Update link travel times
                current_travel_times = {}
                for link, flow in link_flows.items():
                    current_travel_times[link] = travel_time_functions[link](flow)
                
                # Update edge weights for shortest path calculation
                for (u, v), travel_time in current_travel_times.items():
                    G[u][v]['current_weight'] = travel_time
                
                # All-or-nothing assignment (shortest paths for all OD pairs)
                auxiliary_flows = {link: 0.0 for link in links}
                
                for (origin, destination), demand in od_demands.items():
                    if origin != destination and demand > 0:
                        try:
                            # Find shortest path using current travel times
                            path = nx.shortest_path(G, origin, destination, weight='current_weight')
                            
                            # Add demand to path links
                            for i in range(len(path) - 1):
                                link = (path[i], path[i + 1])
                                if link in auxiliary_flows:
                                    auxiliary_flows[link] += demand
                        except nx.NetworkXNoPath:
                            continue
                
                # Line search to find optimal step size
                def objective(alpha):
                    total_cost = 0
                    for link in links:
                        combined_flow = (1 - alpha) * link_flows[link] + alpha * auxiliary_flows[link]
                        # Integral of travel time function (for user equilibrium)
                        travel_time_func = travel_time_functions[link]
                        # Approximate integral using trapezoidal rule
                        if combined_flow > 0:
                            steps = 10
                            flow_step = combined_flow / steps
                            integral = 0
                            for i in range(steps):
                                flow_val = i * flow_step
                                integral += travel_time_func(flow_val) * flow_step
                            total_cost += integral
                    return total_cost
                
                # Simple line search
                best_alpha = 0
                best_cost = objective(0)
                
                for alpha in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]:
                    cost = objective(alpha)
                    if cost < best_cost:
                        best_cost = cost
                        best_alpha = alpha
                
                # Update flows
                old_flows = link_flows.copy()
                for link in links:
                    link_flows[link] = (1 - best_alpha) * link_flows[link] + best_alpha * auxiliary_flows[link]
                
                # Check convergence
                max_flow_change = max(abs(link_flows[link] - old_flows[link]) for link in links)
                if max_flow_change < convergence_tolerance:
                    converged = True
            
            # Calculate final travel times and total travel time
            final_travel_times = {}
            total_travel_time = 0
            
            for link, flow in link_flows.items():
                travel_time = travel_time_functions[link](flow)
                final_travel_times[link] = travel_time
                total_travel_time += flow * travel_time
            
            result = TrafficFlowResult(
                algorithm="User Equilibrium (Frank-Wolfe)",
                link_flows=link_flows,
                travel_times=final_travel_times,
                total_travel_time=total_travel_time,
                convergence_achieved=converged,
                execution_time=time.time() - start_time,
                iterations=iteration
            )
            
        except Exception as e:
            result = TrafficFlowResult(
                algorithm="User Equilibrium (Frank-Wolfe)",
                execution_time=time.time() - start_time,
                convergence_info={"error": str(e)}
            )
        
        self.results_history.append(result)
        return result
    
    def system_optimal_assignment(self,
                                 G: nx.DiGraph,
                                 od_demands: Dict[Tuple[int, int], float],
                                 capacity_attr: str = 'capacity',
                                 free_flow_time_attr: str = 'weight') -> TrafficFlowResult:
        """Solve system optimal assignment (minimize total travel time)"""
        start_time = time.time()
        
        try:
            # Convert to minimum cost flow problem
            # Create expanded network with OD demands
            expanded_G = G.copy()
            
            # Add source and sink nodes for each OD pair
            node_id = max(G.nodes()) + 1
            od_sources = {}
            od_sinks = {}
            
            for (origin, destination), demand in od_demands.items():
                if demand > 0:
                    source_node = node_id
                    sink_node = node_id + 1
                    node_id += 2
                    
                    # Connect source to origin with unlimited capacity and zero cost
                    expanded_G.add_edge(source_node, origin, capacity=demand, weight=0)
                    # Connect destination to sink with unlimited capacity and zero cost
                    expanded_G.add_edge(destination, sink_node, capacity=demand, weight=0)
                    
                    od_sources[(origin, destination)] = source_node
                    od_sinks[(origin, destination)] = sink_node
            
            # Set up demand dictionary for min cost flow
            demand_dict = {}
            for (origin, destination), demand in od_demands.items():
                if demand > 0:
                    source_node = od_sources[(origin, destination)]
                    sink_node = od_sinks[(origin, destination)]
                    demand_dict[source_node] = -demand  # Supply
                    demand_dict[sink_node] = demand     # Demand
            
            # Use sublinear solver for system optimal
            sublinear_result = self.sublinear_solver.minimum_cost_flow_sublinear(
                expanded_G, demand_dict, capacity_attr, free_flow_time_attr
            )
            
            if hasattr(sublinear_result, 'flow_dict') and sublinear_result.flow_dict:
                # Extract original network flows
                link_flows = {}
                for u, v in G.edges():
                    if u in sublinear_result.flow_dict and v in sublinear_result.flow_dict[u]:
                        link_flows[(u, v)] = sublinear_result.flow_dict[u][v]
                    else:
                        link_flows[(u, v)] = 0.0
                
                # Calculate travel times using BPR function
                travel_times = {}
                total_travel_time = 0
                
                for (u, v), flow in link_flows.items():
                    capacity = G[u][v].get(capacity_attr, 1.0)
                    free_flow_time = G[u][v].get(free_flow_time_attr, 1.0)
                    travel_time_func, _ = self.create_bpr_travel_time_function(free_flow_time, capacity)
                    
                    travel_time = travel_time_func(flow)
                    travel_times[(u, v)] = travel_time
                    total_travel_time += flow * travel_time
                
                result = TrafficFlowResult(
                    algorithm="System Optimal (Sublinear)",
                    link_flows=link_flows,
                    travel_times=travel_times,
                    total_travel_time=total_travel_time,
                    convergence_achieved=True,
                    execution_time=time.time() - start_time,
                    iterations=getattr(sublinear_result, 'iterations', 0),
                    convergence_info=getattr(sublinear_result, 'convergence_info', {})
                )
            else:
                result = TrafficFlowResult(
                    algorithm="System Optimal (Sublinear)",
                    execution_time=time.time() - start_time,
                    convergence_info={"error": "Sublinear solver failed"}
                )
            
        except Exception as e:
            result = TrafficFlowResult(
                algorithm="System Optimal (Sublinear)",
                execution_time=time.time() - start_time,
                convergence_info={"error": str(e)}
            )
        
        self.results_history.append(result)
        return result
    
    def incremental_assignment(self,
                              G: nx.DiGraph,
                              od_demands: Dict[Tuple[int, int], float],
                              capacity_attr: str = 'capacity',
                              free_flow_time_attr: str = 'weight',
                              num_increments: int = 10) -> TrafficFlowResult:
        """Incremental traffic assignment"""
        start_time = time.time()
        
        try:
            links = list(G.edges())
            link_flows = {link: 0.0 for link in links}
            
            # Create travel time functions
            travel_time_functions = {}
            for u, v, data in G.edges(data=True):
                capacity = data.get(capacity_attr, 1.0)
                free_flow_time = data.get(free_flow_time_attr, 1.0)
                travel_time_func, _ = self.create_bpr_travel_time_function(free_flow_time, capacity)
                travel_time_functions[(u, v)] = travel_time_func
            
            # Assign demand in increments
            increment_size = 1.0 / num_increments
            
            for increment in range(num_increments):
                # Update link travel times
                for (u, v), flow in link_flows.items():
                    current_travel_time = travel_time_functions[(u, v)](flow)
                    G[u][v]['current_weight'] = current_travel_time
                
                # Assign incremental demand using shortest paths
                for (origin, destination), total_demand in od_demands.items():
                    incremental_demand = total_demand * increment_size
                    
                    if origin != destination and incremental_demand > 0:
                        try:
                            path = nx.shortest_path(G, origin, destination, weight='current_weight')
                            
                            # Add incremental demand to path links
                            for i in range(len(path) - 1):
                                link = (path[i], path[i + 1])
                                if link in link_flows:
                                    link_flows[link] += incremental_demand
                        except nx.NetworkXNoPath:
                            continue
            
            # Calculate final travel times and total travel time
            travel_times = {}
            total_travel_time = 0
            
            for link, flow in link_flows.items():
                travel_time = travel_time_functions[link](flow)
                travel_times[link] = travel_time
                total_travel_time += flow * travel_time
            
            result = TrafficFlowResult(
                algorithm="Incremental Assignment",
                link_flows=link_flows,
                travel_times=travel_times,
                total_travel_time=total_travel_time,
                convergence_achieved=True,
                execution_time=time.time() - start_time,
                iterations=num_increments
            )
            
        except Exception as e:
            result = TrafficFlowResult(
                algorithm="Incremental Assignment",
                execution_time=time.time() - start_time,
                convergence_info={"error": str(e)}
            )
        
        self.results_history.append(result)
        return result
    
    def capacity_constrained_assignment(self,
                                       G: nx.DiGraph,
                                       od_demands: Dict[Tuple[int, int], float],
                                       capacity_attr: str = 'capacity',
                                       free_flow_time_attr: str = 'weight') -> TrafficFlowResult:
        """Traffic assignment with strict capacity constraints"""
        start_time = time.time()
        
        try:
            # Convert to multi-commodity flow with capacity constraints
            # Create separate commodity for each OD pair
            commodities = []
            for (origin, destination), demand in od_demands.items():
                if demand > 0:
                    commodities.append({
                        'source': origin,
                        'sink': destination,
                        'demand': demand
                    })
            
            # Use sublinear multi-commodity flow (if available)
            # For now, use a simple greedy approach
            link_flows = {link: 0.0 for link in G.edges()}
            unassigned_demands = od_demands.copy()
            
            # Iteratively assign demands respecting capacity constraints
            max_iterations = 100
            iteration = 0
            
            while unassigned_demands and iteration < max_iterations:
                iteration += 1
                progress_made = False
                
                for (origin, destination), remaining_demand in list(unassigned_demands.items()):
                    if remaining_demand <= 0:
                        del unassigned_demands[(origin, destination)]
                        continue
                    
                    try:
                        # Find path with sufficient residual capacity
                        path = None
                        
                        # Check all simple paths (limited to avoid exponential time)
                        simple_paths = list(nx.all_simple_paths(G, origin, destination, cutoff=10))
                        
                        for candidate_path in simple_paths[:5]:  # Limit to first 5 paths
                            # Check if path has sufficient capacity
                            min_residual_capacity = float('inf')
                            
                            for i in range(len(candidate_path) - 1):
                                link = (candidate_path[i], candidate_path[i + 1])
                                if link in G.edges():
                                    capacity = G[link[0]][link[1]].get(capacity_attr, float('inf'))
                                    current_flow = link_flows.get(link, 0)
                                    residual_capacity = capacity - current_flow
                                    min_residual_capacity = min(min_residual_capacity, residual_capacity)
                            
                            if min_residual_capacity > 0:
                                path = candidate_path
                                
                                # Assign as much demand as possible
                                assignable_demand = min(remaining_demand, min_residual_capacity)
                                
                                for i in range(len(path) - 1):
                                    link = (path[i], path[i + 1])
                                    link_flows[link] += assignable_demand
                                
                                unassigned_demands[(origin, destination)] -= assignable_demand
                                progress_made = True
                                break
                    
                    except nx.NetworkXNoPath:
                        del unassigned_demands[(origin, destination)]
                
                if not progress_made:
                    break
            
            # Calculate travel times
            travel_times = {}
            total_travel_time = 0
            
            for (u, v), flow in link_flows.items():
                capacity = G[u][v].get(capacity_attr, 1.0)
                free_flow_time = G[u][v].get(free_flow_time_attr, 1.0)
                travel_time_func, _ = self.create_bpr_travel_time_function(free_flow_time, capacity)
                
                travel_time = travel_time_func(flow)
                travel_times[(u, v)] = travel_time
                total_travel_time += flow * travel_time
            
            # Calculate assignment success rate
            total_original_demand = sum(od_demands.values())
            total_unassigned = sum(unassigned_demands.values())
            assignment_rate = 1.0 - (total_unassigned / total_original_demand) if total_original_demand > 0 else 1.0
            
            result = TrafficFlowResult(
                algorithm="Capacity Constrained Assignment",
                link_flows=link_flows,
                travel_times=travel_times,
                total_travel_time=total_travel_time,
                convergence_achieved=len(unassigned_demands) == 0,
                execution_time=time.time() - start_time,
                iterations=iteration,
                network_performance={'assignment_rate': assignment_rate, 'unassigned_demand': total_unassigned}
            )
            
        except Exception as e:
            result = TrafficFlowResult(
                algorithm="Capacity Constrained Assignment",
                execution_time=time.time() - start_time,
                convergence_info={"error": str(e)}
            )
        
        self.results_history.append(result)
        return result
    
    def analyze_network_performance(self,
                                   G: nx.DiGraph,
                                   result: TrafficFlowResult,
                                   capacity_attr: str = 'capacity') -> Dict[str, Any]:
        """Analyze traffic network performance metrics"""
        performance = {
            'total_flow': 0,
            'avg_volume_capacity_ratio': 0,
            'max_volume_capacity_ratio': 0,
            'congested_links': 0,
            'congestion_threshold': 0.8,
            'network_efficiency': 0,
            'travel_time_reliability': 0
        }
        
        if not result.link_flows:
            return performance
        
        volume_capacity_ratios = []
        total_flow = 0
        congested_links = 0
        
        for (u, v), flow in result.link_flows.items():
            total_flow += flow
            
            if G.has_edge(u, v):
                capacity = G[u][v].get(capacity_attr, 1.0)
                vc_ratio = flow / capacity if capacity > 0 else 0
                volume_capacity_ratios.append(vc_ratio)
                
                if vc_ratio > performance['congestion_threshold']:
                    congested_links += 1
        
        if volume_capacity_ratios:
            performance['total_flow'] = total_flow
            performance['avg_volume_capacity_ratio'] = np.mean(volume_capacity_ratios)
            performance['max_volume_capacity_ratio'] = max(volume_capacity_ratios)
            performance['congested_links'] = congested_links
            performance['congestion_rate'] = congested_links / len(volume_capacity_ratios)
        
        # Network efficiency (inverse of average travel time)
        if result.total_travel_time and total_flow > 0:
            avg_travel_time = result.total_travel_time / total_flow
            performance['avg_travel_time'] = avg_travel_time
            performance['network_efficiency'] = 1.0 / avg_travel_time
        
        return performance
    
    def benchmark_traffic_algorithms(self,
                                    G: nx.DiGraph,
                                    od_demands: Dict[Tuple[int, int], float],
                                    capacity_attr: str = 'capacity',
                                    free_flow_time_attr: str = 'weight') -> Dict[str, TrafficFlowResult]:
        """Benchmark all traffic assignment algorithms"""
        results = {}
        
        # User equilibrium
        results['user_equilibrium'] = self.user_equilibrium_frank_wolfe(
            G, od_demands, capacity_attr, free_flow_time_attr
        )
        
        # System optimal
        results['system_optimal'] = self.system_optimal_assignment(
            G, od_demands, capacity_attr, free_flow_time_attr
        )
        
        # Incremental assignment
        results['incremental'] = self.incremental_assignment(
            G, od_demands, capacity_attr, free_flow_time_attr
        )
        
        # Capacity constrained
        results['capacity_constrained'] = self.capacity_constrained_assignment(
            G, od_demands, capacity_attr, free_flow_time_attr
        )
        
        return results
    
    def export_results(self, filename: str) -> None:
        """Export results history to JSON file"""
        export_data = []
        for result in self.results_history:
            result_dict = {
                'algorithm': result.algorithm,
                'total_travel_time': result.total_travel_time,
                'total_delay': result.total_delay,
                'convergence_achieved': result.convergence_achieved,
                'execution_time': result.execution_time,
                'iterations': result.iterations,
                'convergence_info': result.convergence_info,
                'network_performance': result.network_performance,
                'num_links': len(result.link_flows) if result.link_flows else 0
            }
            export_data.append(result_dict)
        
        with open(filename, 'w') as f:
            json.dump(export_data, f, indent=2)
    
    def clear_history(self) -> None:
        """Clear results history"""
        self.results_history = []


def create_test_traffic_networks() -> Dict[str, Tuple[nx.DiGraph, Dict[Tuple[int, int], float]]]:
    """Create test traffic networks with OD demands"""
    networks = {}
    
    # Simple corridor network
    G1 = nx.path_graph(5, create_using=nx.DiGraph)
    for u, v in G1.edges():
        G1[u][v]['capacity'] = 1000
        G1[u][v]['weight'] = 1.0  # Free flow time
    
    # Add reverse direction
    G1.add_edges_from([(v, u, {'capacity': 1000, 'weight': 1.0}) for u, v in list(G1.edges())])
    
    od_demands1 = {(0, 4): 800, (4, 0): 600, (1, 3): 400}
    networks['corridor'] = (G1, od_demands1)
    
    # Grid network
    G2 = nx.grid_2d_graph(4, 4, create_using=nx.DiGraph)
    
    # Convert to integer nodes
    mapping = {node: i for i, node in enumerate(G2.nodes())}
    G2 = nx.relabel_nodes(G2, mapping)
    
    # Add reverse edges and attributes
    edges_to_add = []
    for u, v in list(G2.edges()):
        capacity = np.random.randint(500, 1500)
        travel_time = np.random.uniform(0.5, 2.0)
        
        G2[u][v]['capacity'] = capacity
        G2[u][v]['weight'] = travel_time
        
        edges_to_add.append((v, u, {'capacity': capacity, 'weight': travel_time}))
    
    G2.add_edges_from(edges_to_add)
    
    # Random OD demands
    od_demands2 = {}
    nodes = list(G2.nodes())
    for _ in range(8):
        origin = np.random.choice(nodes)
        destination = np.random.choice(nodes)
        if origin != destination:
            demand = np.random.randint(100, 500)
            od_demands2[(origin, destination)] = demand
    
    networks['grid'] = (G2, od_demands2)
    
    return networks


if __name__ == "__main__":
    # Example usage and testing
    print("Traffic Flow Simulation Demo")
    print("=" * 40)
    
    simulator = TrafficFlowSimulator()
    
    # Create test networks
    networks = create_test_traffic_networks()
    
    for name, (network, od_demands) in networks.items():
        print(f"\nTesting {name} network:")
        print(f"Nodes: {network.number_of_nodes()}, Edges: {network.number_of_edges()}")
        print(f"OD pairs: {len(od_demands)}, Total demand: {sum(od_demands.values())}")
        
        # Benchmark all algorithms
        results = simulator.benchmark_traffic_algorithms(network, od_demands)
        
        print("\nAlgorithm Performance:")
        for algorithm, result in results.items():
            if result.total_travel_time is not None:
                print(f"{algorithm}:")
                print(f"  Total travel time: {result.total_travel_time:.2f}")
                print(f"  Execution time: {result.execution_time:.4f} s")
                print(f"  Convergence: {'Yes' if result.convergence_achieved else 'No'}")
                print(f"  Iterations: {result.iterations}")
                
                # Analyze performance
                performance = simulator.analyze_network_performance(network, result)
                print(f"  Avg V/C ratio: {performance['avg_volume_capacity_ratio']:.3f}")
                print(f"  Congested links: {performance['congested_links']}")
            else:
                print(f"{algorithm}: Failed - {result.convergence_info}")
    
    # Export results
    simulator.export_results("/workspaces/sublinear-time-solver/scripts/network_flow/traffic_results.json")
    print("\nResults exported to traffic_results.json")
