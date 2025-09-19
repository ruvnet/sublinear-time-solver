#!/usr/bin/env python3
"""
Sublinear Network Flow Solver

Implements network flow problems as linear systems and solves them using
sublinear-time MCP solvers. Demonstrates advantages for large sparse networks.

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
import subprocess
import tempfile
import os
from scipy.sparse import csr_matrix, coo_matrix
from scipy.sparse.linalg import spsolve
import warnings


@dataclass
class SublinearFlowResult:
    """Container for sublinear flow algorithm results"""
    algorithm: str
    max_flow_value: Optional[float] = None
    min_cost: Optional[float] = None
    flow_dict: Optional[Dict] = None
    potentials: Optional[np.ndarray] = None
    execution_time: float = 0.0
    iterations: int = 0
    mcp_solver_time: float = 0.0
    matrix_setup_time: float = 0.0
    convergence_info: Optional[Dict] = None
    matrix_properties: Optional[Dict] = None
    sublinear_advantage: Optional[Dict] = None


class SublinearFlowSolver:
    """Network flow solver using sublinear-time linear system solvers"""
    
    def __init__(self, mcp_server_path: Optional[str] = None):
        self.results_history = []
        self.mcp_server_path = mcp_server_path or "npx sublinear-solver-mcp"
        
    def _call_mcp_solver(self, matrix_data: Dict, vector: np.ndarray, 
                        method: str = "neumann", epsilon: float = 1e-6) -> Dict:
        """Call MCP solver with matrix and vector data"""
        start_time = time.time()
        
        try:
            # Create temporary input file
            with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
                input_data = {
                    "method": "solve",
                    "params": {
                        "matrix": matrix_data,
                        "vector": vector.tolist(),
                        "method": method,
                        "epsilon": epsilon,
                        "maxIterations": 1000
                    }
                }
                json.dump(input_data, f)
                input_file = f.name
            
            # Call MCP solver
            result = subprocess.run(
                ["node", "-e", f"""
                const fs = require('fs');
                const {{ solve }} = require('/workspaces/sublinear-time-solver/server/index.js');
                
                const input = JSON.parse(fs.readFileSync('{input_file}', 'utf8'));
                const result = solve(input.params);
                console.log(JSON.stringify(result));
                """],
                capture_output=True,
                text=True,
                timeout=300
            )
            
            if result.returncode == 0:
                solver_result = json.loads(result.stdout.strip())
                solver_result['mcp_time'] = time.time() - start_time
                return solver_result
            else:
                return {
                    'error': f"MCP solver failed: {result.stderr}",
                    'mcp_time': time.time() - start_time
                }
                
        except Exception as e:
            return {
                'error': f"MCP solver exception: {str(e)}",
                'mcp_time': time.time() - start_time
            }
        finally:
            # Clean up temporary file
            try:
                os.unlink(input_file)
            except:
                pass
    
    def _analyze_matrix_properties(self, matrix_data: Dict) -> Dict:
        """Analyze matrix properties for sublinear solver optimization"""
        try:
            # Call MCP matrix analyzer
            with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
                input_data = {
                    "method": "analyzeMatrix",
                    "params": {
                        "matrix": matrix_data,
                        "checkDominance": True,
                        "checkSymmetry": True,
                        "estimateCondition": True
                    }
                }
                json.dump(input_data, f)
                input_file = f.name
            
            result = subprocess.run(
                ["node", "-e", f"""
                const fs = require('fs');
                const {{ analyzeMatrix }} = require('/workspaces/sublinear-time-solver/server/index.js');
                
                const input = JSON.parse(fs.readFileSync('{input_file}', 'utf8'));
                const result = analyzeMatrix(input.params);
                console.log(JSON.stringify(result));
                """],
                capture_output=True,
                text=True,
                timeout=60
            )
            
            if result.returncode == 0:
                return json.loads(result.stdout.strip())
            else:
                return {"error": f"Matrix analysis failed: {result.stderr}"}
                
        except Exception as e:
            return {"error": f"Matrix analysis exception: {str(e)}"}
        finally:
            try:
                os.unlink(input_file)
            except:
                pass
    
    def _graph_to_flow_matrix(self, G: nx.DiGraph, 
                             flow_type: str = "max_flow",
                             source: Optional[int] = None,
                             sink: Optional[int] = None,
                             demand_dict: Optional[Dict[int, float]] = None) -> Tuple[Dict, np.ndarray, Dict]:
        """Convert network flow problem to linear system Ax = b"""
        setup_start = time.time()
        
        nodes = list(G.nodes())
        edges = list(G.edges())
        n_nodes = len(nodes)
        n_edges = len(edges)
        
        node_to_idx = {node: i for i, node in enumerate(nodes)}
        edge_to_idx = {edge: i for i, edge in enumerate(edges)}
        
        if flow_type == "max_flow":
            # For maximum flow: flow conservation constraints + capacity constraints
            # Variables: edge flows + auxiliary flow variable
            n_vars = n_edges + 1  # edge flows + total flow
            n_constraints = n_nodes  # flow conservation at each node
            
            # Flow conservation matrix (A)
            row_indices = []
            col_indices = []
            values = []
            
            for i, node in enumerate(nodes):
                # Outgoing edges (positive contribution)
                for j, (u, v) in enumerate(edges):
                    if u == node:
                        row_indices.append(i)
                        col_indices.append(j)
                        values.append(1.0)
                    elif v == node:
                        row_indices.append(i)
                        col_indices.append(j)
                        values.append(-1.0)
                
                # Source produces flow, sink consumes flow
                if node == source:
                    row_indices.append(i)
                    col_indices.append(n_edges)  # auxiliary flow variable
                    values.append(-1.0)
                elif node == sink:
                    row_indices.append(i)
                    col_indices.append(n_edges)
                    values.append(1.0)
            
            # Right-hand side (b) - all zeros for flow conservation
            b = np.zeros(n_constraints)
            
        elif flow_type == "min_cost_flow":
            # For minimum cost flow: flow conservation constraints
            n_vars = n_edges
            n_constraints = n_nodes
            
            row_indices = []
            col_indices = []
            values = []
            
            for i, node in enumerate(nodes):
                for j, (u, v) in enumerate(edges):
                    if u == node:
                        row_indices.append(i)
                        col_indices.append(j)
                        values.append(1.0)
                    elif v == node:
                        row_indices.append(i)
                        col_indices.append(j)
                        values.append(-1.0)
            
            # Right-hand side from demand_dict
            b = np.zeros(n_constraints)
            if demand_dict:
                for node, demand in demand_dict.items():
                    if node in node_to_idx:
                        b[node_to_idx[node]] = -demand  # negative for supply/demand convention
        
        else:
            raise ValueError(f"Unknown flow type: {flow_type}")
        
        # Convert to sparse matrix format for MCP
        matrix_data = {
            "rows": n_constraints,
            "cols": n_vars,
            "format": "coo",
            "data": {
                "values": values,
                "rowIndices": row_indices,
                "colIndices": col_indices
            }
        }
        
        metadata = {
            "nodes": nodes,
            "edges": edges,
            "node_to_idx": node_to_idx,
            "edge_to_idx": edge_to_idx,
            "flow_type": flow_type,
            "setup_time": time.time() - setup_start
        }
        
        return matrix_data, b, metadata
    
    def maximum_flow_sublinear(self, 
                              G: nx.DiGraph,
                              source: int,
                              sink: int,
                              capacity_attr: str = 'capacity',
                              method: str = "neumann") -> SublinearFlowResult:
        """Solve maximum flow using sublinear solver"""
        start_time = time.time()
        
        try:
            # Convert to linear system
            matrix_data, b, metadata = self._graph_to_flow_matrix(
                G, "max_flow", source, sink
            )
            
            # Analyze matrix properties
            matrix_props = self._analyze_matrix_properties(matrix_data)
            
            # Solve using MCP
            solver_result = self._call_mcp_solver(matrix_data, b, method)
            
            if 'error' in solver_result:
                return SublinearFlowResult(
                    algorithm=f"Sublinear Max Flow ({method})",
                    execution_time=time.time() - start_time,
                    matrix_setup_time=metadata['setup_time'],
                    mcp_solver_time=solver_result.get('mcp_time', 0),
                    convergence_info={"error": solver_result['error']},
                    matrix_properties=matrix_props
                )
            
            # Extract flow solution
            solution = np.array(solver_result.get('solution', []))
            if len(solution) > len(metadata['edges']):
                max_flow_value = solution[-1]  # auxiliary flow variable
                edge_flows = solution[:-1]
            else:
                edge_flows = solution
                max_flow_value = np.sum([edge_flows[i] for i, (u, v) in enumerate(metadata['edges']) if u == source])
            
            # Convert to flow dictionary
            flow_dict = defaultdict(lambda: defaultdict(float))
            for i, (u, v) in enumerate(metadata['edges']):
                if i < len(edge_flows) and edge_flows[i] > 1e-10:
                    flow_dict[u][v] = edge_flows[i]
            
            result = SublinearFlowResult(
                algorithm=f"Sublinear Max Flow ({method})",
                max_flow_value=max_flow_value,
                flow_dict=dict(flow_dict),
                execution_time=time.time() - start_time,
                iterations=solver_result.get('iterations', 0),
                matrix_setup_time=metadata['setup_time'],
                mcp_solver_time=solver_result.get('mcp_time', 0),
                convergence_info=solver_result.get('convergence', {}),
                matrix_properties=matrix_props
            )
            
        except Exception as e:
            result = SublinearFlowResult(
                algorithm=f"Sublinear Max Flow ({method})",
                execution_time=time.time() - start_time,
                convergence_info={"error": str(e)}
            )
        
        self.results_history.append(result)
        return result
    
    def minimum_cost_flow_sublinear(self,
                                   G: nx.DiGraph,
                                   demand_dict: Dict[int, float],
                                   capacity_attr: str = 'capacity',
                                   weight_attr: str = 'weight',
                                   method: str = "neumann") -> SublinearFlowResult:
        """Solve minimum cost flow using sublinear solver"""
        start_time = time.time()
        
        try:
            # Convert to linear system
            matrix_data, b, metadata = self._graph_to_flow_matrix(
                G, "min_cost_flow", demand_dict=demand_dict
            )
            
            # Analyze matrix properties
            matrix_props = self._analyze_matrix_properties(matrix_data)
            
            # Solve using MCP
            solver_result = self._call_mcp_solver(matrix_data, b, method)
            
            if 'error' in solver_result:
                return SublinearFlowResult(
                    algorithm=f"Sublinear Min Cost Flow ({method})",
                    execution_time=time.time() - start_time,
                    matrix_setup_time=metadata['setup_time'],
                    mcp_solver_time=solver_result.get('mcp_time', 0),
                    convergence_info={"error": solver_result['error']},
                    matrix_properties=matrix_props
                )
            
            # Extract flow solution
            solution = np.array(solver_result.get('solution', []))
            edge_flows = solution
            
            # Calculate total cost
            total_cost = 0
            flow_dict = defaultdict(lambda: defaultdict(float))
            
            for i, (u, v) in enumerate(metadata['edges']):
                if i < len(edge_flows) and abs(edge_flows[i]) > 1e-10:
                    flow_value = edge_flows[i]
                    flow_dict[u][v] = flow_value
                    cost = G[u][v].get(weight_attr, 1)
                    total_cost += flow_value * cost
            
            result = SublinearFlowResult(
                algorithm=f"Sublinear Min Cost Flow ({method})",
                min_cost=total_cost,
                flow_dict=dict(flow_dict),
                execution_time=time.time() - start_time,
                iterations=solver_result.get('iterations', 0),
                matrix_setup_time=metadata['setup_time'],
                mcp_solver_time=solver_result.get('mcp_time', 0),
                convergence_info=solver_result.get('convergence', {}),
                matrix_properties=matrix_props
            )
            
        except Exception as e:
            result = SublinearFlowResult(
                algorithm=f"Sublinear Min Cost Flow ({method})",
                execution_time=time.time() - start_time,
                convergence_info={"error": str(e)}
            )
        
        self.results_history.append(result)
        return result
    
    def electrical_network_analysis(self,
                                   G: nx.Graph,
                                   voltage_sources: Dict[int, float],
                                   resistance_attr: str = 'resistance') -> SublinearFlowResult:
        """Analyze electrical network using sublinear solver
        
        Solves the system: (G^T * C * G) * V = I
        where G is incidence matrix, C is conductance matrix, V is voltage, I is current
        """
        start_time = time.time()
        
        try:
            nodes = list(G.nodes())
            edges = list(G.edges())
            n_nodes = len(nodes)
            n_edges = len(edges)
            
            node_to_idx = {node: i for i, node in enumerate(nodes)}
            
            # Build incidence matrix G (nodes x edges)
            row_indices = []
            col_indices = []
            values = []
            
            for j, (u, v) in enumerate(edges):
                # Arbitrary orientation: u -> v
                row_indices.extend([node_to_idx[u], node_to_idx[v]])
                col_indices.extend([j, j])
                values.extend([1.0, -1.0])
            
            # Conductance matrix C (diagonal)
            conductances = []
            for u, v in edges:
                resistance = G[u][v].get(resistance_attr, 1.0)
                conductance = 1.0 / resistance if resistance > 0 else 1.0
                conductances.append(conductance)
            
            # Build Laplacian matrix L = G^T * C * G
            laplacian_rows = []
            laplacian_cols = []
            laplacian_vals = []
            
            for i in range(n_nodes):
                for j in range(n_nodes):
                    if i == j:
                        # Diagonal: sum of conductances of incident edges
                        total_conductance = 0
                        for k, (u, v) in enumerate(edges):
                            if u == nodes[i] or v == nodes[i]:
                                total_conductance += conductances[k]
                        if total_conductance > 0:
                            laplacian_rows.append(i)
                            laplacian_cols.append(j)
                            laplacian_vals.append(total_conductance)
                    else:
                        # Off-diagonal: negative conductance if connected
                        edge_conductance = 0
                        for k, (u, v) in enumerate(edges):
                            if (u == nodes[i] and v == nodes[j]) or (u == nodes[j] and v == nodes[i]):
                                edge_conductance = conductances[k]
                                break
                        if edge_conductance > 0:
                            laplacian_rows.append(i)
                            laplacian_cols.append(j)
                            laplacian_vals.append(-edge_conductance)
            
            # Handle voltage sources (current injection)
            current_vector = np.zeros(n_nodes)
            for node, voltage in voltage_sources.items():
                if node in node_to_idx:
                    # Set high conductance to ground for voltage source
                    idx = node_to_idx[node]
                    high_conductance = 1e6
                    laplacian_rows.append(idx)
                    laplacian_cols.append(idx)
                    laplacian_vals.append(high_conductance)
                    current_vector[idx] = voltage * high_conductance
            
            # Create matrix data for MCP
            matrix_data = {
                "rows": n_nodes,
                "cols": n_nodes,
                "format": "coo",
                "data": {
                    "values": laplacian_vals,
                    "rowIndices": laplacian_rows,
                    "colIndices": laplacian_cols
                }
            }
            
            # Analyze matrix properties
            matrix_props = self._analyze_matrix_properties(matrix_data)
            
            # Solve using MCP
            solver_result = self._call_mcp_solver(matrix_data, current_vector, "neumann")
            
            if 'error' in solver_result:
                return SublinearFlowResult(
                    algorithm="Electrical Network Analysis",
                    execution_time=time.time() - start_time,
                    mcp_solver_time=solver_result.get('mcp_time', 0),
                    convergence_info={"error": solver_result['error']},
                    matrix_properties=matrix_props
                )
            
            # Extract voltage solution
            voltages = np.array(solver_result.get('solution', []))
            
            # Calculate currents through edges
            currents = {}
            for k, (u, v) in enumerate(edges):
                if node_to_idx[u] < len(voltages) and node_to_idx[v] < len(voltages):
                    voltage_diff = voltages[node_to_idx[u]] - voltages[node_to_idx[v]]
                    current = voltage_diff * conductances[k]
                    currents[(u, v)] = current
            
            result = SublinearFlowResult(
                algorithm="Electrical Network Analysis",
                potentials=voltages,
                flow_dict={'currents': currents, 'voltages': {nodes[i]: voltages[i] for i in range(len(voltages))}},
                execution_time=time.time() - start_time,
                iterations=solver_result.get('iterations', 0),
                mcp_solver_time=solver_result.get('mcp_time', 0),
                convergence_info=solver_result.get('convergence', {}),
                matrix_properties=matrix_props
            )
            
        except Exception as e:
            result = SublinearFlowResult(
                algorithm="Electrical Network Analysis",
                execution_time=time.time() - start_time,
                convergence_info={"error": str(e)}
            )
        
        self.results_history.append(result)
        return result
    
    def benchmark_sublinear_methods(self,
                                   G: nx.DiGraph,
                                   source: int,
                                   sink: int,
                                   demand_dict: Optional[Dict[int, float]] = None) -> Dict[str, SublinearFlowResult]:
        """Benchmark different sublinear methods"""
        results = {}
        methods = ["neumann", "random-walk", "forward-push", "backward-push"]
        
        for method in methods:
            # Maximum flow
            results[f'max_flow_{method}'] = self.maximum_flow_sublinear(
                G, source, sink, method=method
            )
            
            # Minimum cost flow (if demands provided)
            if demand_dict:
                results[f'min_cost_{method}'] = self.minimum_cost_flow_sublinear(
                    G, demand_dict, method=method
                )
        
        return results
    
    def analyze_sublinear_advantage(self, traditional_results: Dict, sublinear_results: Dict) -> Dict:
        """Analyze when sublinear methods have advantages"""
        analysis = {
            'speed_improvements': {},
            'accuracy_comparison': {},
            'scalability_analysis': {},
            'method_recommendations': {}
        }
        
        for alg_name in traditional_results:
            trad_result = traditional_results[alg_name]
            
            # Find corresponding sublinear result
            for sub_name in sublinear_results:
                sub_result = sublinear_results[sub_name]
                
                if (trad_result.max_flow_value is not None and 
                    sub_result.max_flow_value is not None):
                    
                    # Speed comparison
                    speedup = trad_result.execution_time / sub_result.execution_time
                    analysis['speed_improvements'][f'{alg_name}_vs_{sub_name}'] = {
                        'speedup': speedup,
                        'traditional_time': trad_result.execution_time,
                        'sublinear_time': sub_result.execution_time
                    }
                    
                    # Accuracy comparison
                    accuracy = abs(trad_result.max_flow_value - sub_result.max_flow_value) / max(trad_result.max_flow_value, 1e-10)
                    analysis['accuracy_comparison'][f'{alg_name}_vs_{sub_name}'] = {
                        'relative_error': accuracy,
                        'traditional_value': trad_result.max_flow_value,
                        'sublinear_value': sub_result.max_flow_value
                    }
        
        return analysis
    
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
                'mcp_solver_time': result.mcp_solver_time,
                'matrix_setup_time': result.matrix_setup_time,
                'convergence_info': result.convergence_info,
                'matrix_properties': result.matrix_properties,
                'flow_dict_size': len(result.flow_dict) if result.flow_dict else 0
            }
            export_data.append(result_dict)
        
        with open(filename, 'w') as f:
            json.dump(export_data, f, indent=2)
    
    def clear_history(self) -> None:
        """Clear results history"""
        self.results_history = []


if __name__ == "__main__":
    # Example usage and testing
    print("Sublinear Flow Algorithms Demo")
    print("=" * 40)
    
    # Create sample network
    G = nx.erdos_renyi_graph(10, 0.3, directed=True)
    
    # Add random capacities and costs
    for u, v in G.edges():
        G[u][v]['capacity'] = np.random.randint(1, 10)
        G[u][v]['weight'] = np.random.randint(1, 5)
    
    source, sink = 0, 9
    
    # Ensure connectivity
    if not nx.has_path(G, source, sink):
        path_nodes = list(range(source, sink + 1))
        for i in range(len(path_nodes) - 1):
            if not G.has_edge(path_nodes[i], path_nodes[i + 1]):
                G.add_edge(path_nodes[i], path_nodes[i + 1], capacity=5, weight=1)
    
    print(f"Network: {G.number_of_nodes()} nodes, {G.number_of_edges()} edges")
    print(f"Source: {source}, Sink: {sink}")
    
    # Initialize solver
    solver = SublinearFlowSolver()
    
    # Test sublinear maximum flow
    print("\nSublinear Maximum Flow Results:")
    result = solver.maximum_flow_sublinear(G, source, sink)
    if result.max_flow_value is not None:
        print(f"Max Flow: {result.max_flow_value:.4f} (time: {result.execution_time:.4f}s)")
        print(f"MCP Solver Time: {result.mcp_solver_time:.4f}s")
        print(f"Iterations: {result.iterations}")
    else:
        print(f"Failed: {result.convergence_info}")
    
    # Test sublinear minimum cost flow
    print("\nSublinear Minimum Cost Flow Results:")
    demand_dict = {source: -5, sink: 5}
    result = solver.minimum_cost_flow_sublinear(G, demand_dict)
    if result.min_cost is not None:
        print(f"Min Cost: {result.min_cost:.4f} (time: {result.execution_time:.4f}s)")
        print(f"MCP Solver Time: {result.mcp_solver_time:.4f}s")
    else:
        print(f"Failed: {result.convergence_info}")
    
    # Export results
    solver.export_results("/workspaces/sublinear-time-solver/scripts/network_flow/sublinear_results.json")
    print("\nResults exported to sublinear_results.json")
