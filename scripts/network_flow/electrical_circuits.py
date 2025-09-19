#!/usr/bin/env python3
"""
Electrical Circuit Analysis Tools

Implements electrical network analysis using both traditional methods and
sublinear solvers. Includes current flow, effective resistance, and circuit
optimization calculations.

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
from scipy.sparse import csr_matrix, diags
from scipy.sparse.linalg import spsolve, inv
from scipy.linalg import pinv
import warnings

try:
    from sublinear_flow import SublinearFlowSolver, SublinearFlowResult
except ImportError:
    # Handle case where sublinear_flow module is not available
    class SublinearFlowSolver:
        def electrical_network_analysis(self, *args, **kwargs):
            return {"error": "SublinearFlowSolver not available"}
    
    class SublinearFlowResult:
        pass


@dataclass
class CircuitAnalysisResult:
    """Container for circuit analysis results"""
    algorithm: str
    node_voltages: Optional[Dict[int, float]] = None
    edge_currents: Optional[Dict[Tuple[int, int], float]] = None
    total_power: Optional[float] = None
    effective_resistance: Optional[Dict[Tuple[int, int], float]] = None
    execution_time: float = 0.0
    convergence_info: Optional[Dict] = None
    matrix_properties: Optional[Dict] = None
    

class ElectricalCircuitAnalyzer:
    """Electrical circuit analysis using various methods"""
    
    def __init__(self):
        self.results_history = []
        self.sublinear_solver = SublinearFlowSolver()
    
    def build_conductance_matrix(self, G: nx.Graph, resistance_attr: str = 'resistance') -> Tuple[np.ndarray, Dict[int, int]]:
        """Build conductance (Laplacian) matrix for circuit analysis"""
        nodes = list(G.nodes())
        n_nodes = len(nodes)
        node_to_idx = {node: i for i, node in enumerate(nodes)}
        
        # Initialize conductance matrix
        L = np.zeros((n_nodes, n_nodes))
        
        for u, v, data in G.edges(data=True):
            resistance = data.get(resistance_attr, 1.0)
            conductance = 1.0 / resistance if resistance > 0 else 1.0
            
            i, j = node_to_idx[u], node_to_idx[v]
            
            # Off-diagonal elements (negative conductance)
            L[i][j] -= conductance
            L[j][i] -= conductance
            
            # Diagonal elements (sum of conductances)
            L[i][i] += conductance
            L[j][j] += conductance
        
        return L, node_to_idx
    
    def solve_dc_circuit_numpy(self, 
                              G: nx.Graph,
                              voltage_sources: Dict[int, float],
                              current_sources: Dict[int, float] = None,
                              resistance_attr: str = 'resistance') -> CircuitAnalysisResult:
        """Solve DC circuit using direct NumPy linear algebra"""
        start_time = time.time()
        
        try:
            if current_sources is None:
                current_sources = {}
            
            # Build conductance matrix
            L, node_to_idx = self.build_conductance_matrix(G, resistance_attr)
            n_nodes = len(node_to_idx)
            
            # Build current injection vector
            I = np.zeros(n_nodes)
            for node, current in current_sources.items():
                if node in node_to_idx:
                    I[node_to_idx[node]] = current
            
            # Handle voltage sources by modifying matrix
            # For each voltage source, set high conductance to "ground"
            for node, voltage in voltage_sources.items():
                if node in node_to_idx:
                    idx = node_to_idx[node]
                    # Clear the row and set diagonal to large value
                    L[idx, :] = 0
                    L[idx, idx] = 1e12
                    I[idx] = voltage * 1e12
            
            # Solve the system L * V = I
            if np.linalg.det(L) != 0:
                voltages = np.linalg.solve(L, I)
            else:
                # Use pseudoinverse for singular matrices
                voltages = pinv(L) @ I
            
            # Calculate edge currents using Ohm's law
            edge_currents = {}
            for u, v, data in G.edges(data=True):
                resistance = data.get(resistance_attr, 1.0)
                if u in node_to_idx and v in node_to_idx:
                    u_idx, v_idx = node_to_idx[u], node_to_idx[v]
                    voltage_diff = voltages[u_idx] - voltages[v_idx]
                    current = voltage_diff / resistance if resistance > 0 else 0
                    edge_currents[(u, v)] = current
            
            # Calculate total power
            total_power = 0
            for (u, v), current in edge_currents.items():
                resistance = G[u][v].get(resistance_attr, 1.0)
                power = current**2 * resistance
                total_power += abs(power)
            
            # Convert voltages to dictionary
            node_voltages = {node: voltages[idx] for node, idx in node_to_idx.items()}
            
            result = CircuitAnalysisResult(
                algorithm="DC Circuit (NumPy)",
                node_voltages=node_voltages,
                edge_currents=edge_currents,
                total_power=total_power,
                execution_time=time.time() - start_time
            )
            
        except Exception as e:
            result = CircuitAnalysisResult(
                algorithm="DC Circuit (NumPy)",
                execution_time=time.time() - start_time,
                convergence_info={"error": str(e)}
            )
        
        self.results_history.append(result)
        return result
    
    def solve_dc_circuit_sparse(self,
                               G: nx.Graph,
                               voltage_sources: Dict[int, float],
                               current_sources: Dict[int, float] = None,
                               resistance_attr: str = 'resistance') -> CircuitAnalysisResult:
        """Solve DC circuit using sparse matrix methods"""
        start_time = time.time()
        
        try:
            if current_sources is None:
                current_sources = {}
            
            nodes = list(G.nodes())
            n_nodes = len(nodes)
            node_to_idx = {node: i for i, node in enumerate(nodes)}
            
            # Build sparse conductance matrix
            row_indices = []
            col_indices = []
            data = []
            
            # Off-diagonal elements
            for u, v, edge_data in G.edges(data=True):
                resistance = edge_data.get(resistance_attr, 1.0)
                conductance = 1.0 / resistance if resistance > 0 else 1.0
                
                i, j = node_to_idx[u], node_to_idx[v]
                
                # Off-diagonal terms (negative)
                row_indices.extend([i, j])
                col_indices.extend([j, i])
                data.extend([-conductance, -conductance])
            
            # Diagonal elements (sum of conductances)
            for node in nodes:
                i = node_to_idx[node]
                total_conductance = 0
                
                for neighbor in G.neighbors(node):
                    resistance = G[node][neighbor].get(resistance_attr, 1.0)
                    conductance = 1.0 / resistance if resistance > 0 else 1.0
                    total_conductance += conductance
                
                if total_conductance > 0:
                    row_indices.append(i)
                    col_indices.append(i)
                    data.append(total_conductance)
            
            # Create sparse matrix
            L = csr_matrix((data, (row_indices, col_indices)), shape=(n_nodes, n_nodes))
            
            # Build current injection vector
            I = np.zeros(n_nodes)
            for node, current in current_sources.items():
                if node in node_to_idx:
                    I[node_to_idx[node]] = current
            
            # Handle voltage sources
            for node, voltage in voltage_sources.items():
                if node in node_to_idx:
                    idx = node_to_idx[node]
                    # Modify matrix for voltage constraint
                    L[idx, :] = 0
                    L[idx, idx] = 1e12
                    I[idx] = voltage * 1e12
            
            # Solve sparse system
            voltages = spsolve(L, I)
            
            # Calculate edge currents
            edge_currents = {}
            for u, v, edge_data in G.edges(data=True):
                resistance = edge_data.get(resistance_attr, 1.0)
                if u in node_to_idx and v in node_to_idx:
                    u_idx, v_idx = node_to_idx[u], node_to_idx[v]
                    voltage_diff = voltages[u_idx] - voltages[v_idx]
                    current = voltage_diff / resistance if resistance > 0 else 0
                    edge_currents[(u, v)] = current
            
            # Calculate total power
            total_power = sum(
                current**2 * G[u][v].get(resistance_attr, 1.0)
                for (u, v), current in edge_currents.items()
            )
            
            # Convert voltages to dictionary
            node_voltages = {node: voltages[idx] for node, idx in node_to_idx.items()}
            
            result = CircuitAnalysisResult(
                algorithm="DC Circuit (Sparse)",
                node_voltages=node_voltages,
                edge_currents=edge_currents,
                total_power=total_power,
                execution_time=time.time() - start_time
            )
            
        except Exception as e:
            result = CircuitAnalysisResult(
                algorithm="DC Circuit (Sparse)",
                execution_time=time.time() - start_time,
                convergence_info={"error": str(e)}
            )
        
        self.results_history.append(result)
        return result
    
    def solve_dc_circuit_sublinear(self,
                                  G: nx.Graph,
                                  voltage_sources: Dict[int, float],
                                  current_sources: Dict[int, float] = None,
                                  resistance_attr: str = 'resistance') -> CircuitAnalysisResult:
        """Solve DC circuit using sublinear methods"""
        start_time = time.time()
        
        try:
            # Use sublinear electrical network analysis
            sublinear_result = self.sublinear_solver.electrical_network_analysis(
                G, voltage_sources, resistance_attr
            )
            
            if hasattr(sublinear_result, 'flow_dict') and sublinear_result.flow_dict:
                # Extract results from sublinear solver
                currents = sublinear_result.flow_dict.get('currents', {})
                voltages = sublinear_result.flow_dict.get('voltages', {})
                
                # Calculate total power
                total_power = 0
                for (u, v), current in currents.items():
                    if G.has_edge(u, v):
                        resistance = G[u][v].get(resistance_attr, 1.0)
                        power = current**2 * resistance
                        total_power += abs(power)
                
                result = CircuitAnalysisResult(
                    algorithm="DC Circuit (Sublinear)",
                    node_voltages=voltages,
                    edge_currents=currents,
                    total_power=total_power,
                    execution_time=time.time() - start_time,
                    convergence_info=sublinear_result.convergence_info,
                    matrix_properties=sublinear_result.matrix_properties
                )
            else:
                result = CircuitAnalysisResult(
                    algorithm="DC Circuit (Sublinear)",
                    execution_time=time.time() - start_time,
                    convergence_info={"error": "Sublinear solver failed"}
                )
            
        except Exception as e:
            result = CircuitAnalysisResult(
                algorithm="DC Circuit (Sublinear)",
                execution_time=time.time() - start_time,
                convergence_info={"error": str(e)}
            )
        
        self.results_history.append(result)
        return result
    
    def calculate_effective_resistance(self,
                                     G: nx.Graph,
                                     node_pairs: List[Tuple[int, int]],
                                     resistance_attr: str = 'resistance',
                                     method: str = 'matrix_inverse') -> CircuitAnalysisResult:
        """Calculate effective resistance between node pairs"""
        start_time = time.time()
        
        try:
            L, node_to_idx = self.build_conductance_matrix(G, resistance_attr)
            n_nodes = len(node_to_idx)
            
            effective_resistances = {}
            
            if method == 'matrix_inverse':
                # Use Moore-Penrose pseudoinverse for singular Laplacian
                L_pinv = pinv(L)
                
                for node1, node2 in node_pairs:
                    if node1 in node_to_idx and node2 in node_to_idx:
                        i, j = node_to_idx[node1], node_to_idx[node2]
                        
                        # Effective resistance = L_pinv[i,i] + L_pinv[j,j] - 2*L_pinv[i,j]
                        r_eff = L_pinv[i, i] + L_pinv[j, j] - 2 * L_pinv[i, j]
                        effective_resistances[(node1, node2)] = r_eff
            
            elif method == 'current_injection':
                # Calculate by injecting unit current and measuring voltage
                for node1, node2 in node_pairs:
                    if node1 in node_to_idx and node2 in node_to_idx:
                        # Inject +1 current at node1, -1 at node2
                        I = np.zeros(n_nodes)
                        I[node_to_idx[node1]] = 1.0
                        I[node_to_idx[node2]] = -1.0
                        
                        # Ground one node (set voltage to 0)
                        L_modified = L.copy()
                        L_modified[0, :] = 0
                        L_modified[0, 0] = 1
                        I[0] = 0
                        
                        # Solve for voltages
                        voltages = np.linalg.solve(L_modified, I)
                        
                        # Effective resistance is voltage difference
                        v1 = voltages[node_to_idx[node1]]
                        v2 = voltages[node_to_idx[node2]]
                        r_eff = abs(v1 - v2)
                        effective_resistances[(node1, node2)] = r_eff
            
            result = CircuitAnalysisResult(
                algorithm=f"Effective Resistance ({method})",
                effective_resistance=effective_resistances,
                execution_time=time.time() - start_time
            )
            
        except Exception as e:
            result = CircuitAnalysisResult(
                algorithm=f"Effective Resistance ({method})",
                execution_time=time.time() - start_time,
                convergence_info={"error": str(e)}
            )
        
        self.results_history.append(result)
        return result
    
    def analyze_circuit_properties(self, 
                                  G: nx.Graph,
                                  resistance_attr: str = 'resistance') -> Dict[str, Any]:
        """Analyze electrical properties of the circuit"""
        properties = {
            'nodes': G.number_of_nodes(),
            'edges': G.number_of_edges(),
            'is_connected': nx.is_connected(G),
            'avg_resistance': 0,
            'total_conductance': 0,
            'max_resistance': 0,
            'min_resistance': float('inf'),
            'resistance_distribution': {},
            'node_degrees': {},
            'circuit_complexity': 0
        }
        
        resistances = []
        total_conductance = 0
        
        for u, v, data in G.edges(data=True):
            resistance = data.get(resistance_attr, 1.0)
            resistances.append(resistance)
            total_conductance += 1.0 / resistance if resistance > 0 else 0
            
            properties['max_resistance'] = max(properties['max_resistance'], resistance)
            properties['min_resistance'] = min(properties['min_resistance'], resistance)
        
        if resistances:
            properties['avg_resistance'] = np.mean(resistances)
            properties['std_resistance'] = np.std(resistances)
            properties['total_conductance'] = total_conductance
        
        # Node degree distribution
        degrees = dict(G.degree())
        properties['node_degrees'] = degrees
        properties['avg_degree'] = np.mean(list(degrees.values()))
        properties['max_degree'] = max(degrees.values())
        
        # Circuit complexity (based on cycles and connectivity)
        if nx.is_connected(G):
            properties['diameter'] = nx.diameter(G)
            properties['radius'] = nx.radius(G)
            properties['circuit_complexity'] = (G.number_of_edges() - G.number_of_nodes() + 1) / G.number_of_nodes()
        
        return properties
    
    def benchmark_circuit_solvers(self,
                                 G: nx.Graph,
                                 voltage_sources: Dict[int, float],
                                 current_sources: Dict[int, float] = None,
                                 resistance_attr: str = 'resistance') -> Dict[str, CircuitAnalysisResult]:
        """Benchmark all available circuit solving methods"""
        results = {}
        
        # Traditional methods
        results['numpy'] = self.solve_dc_circuit_numpy(G, voltage_sources, current_sources, resistance_attr)
        results['sparse'] = self.solve_dc_circuit_sparse(G, voltage_sources, current_sources, resistance_attr)
        
        # Sublinear method
        results['sublinear'] = self.solve_dc_circuit_sublinear(G, voltage_sources, current_sources, resistance_attr)
        
        return results
    
    def validate_circuit_solution(self,
                                 G: nx.Graph,
                                 result: CircuitAnalysisResult,
                                 voltage_sources: Dict[int, float],
                                 current_sources: Dict[int, float] = None,
                                 resistance_attr: str = 'resistance',
                                 tolerance: float = 1e-6) -> Dict[str, Any]:
        """Validate circuit solution using Kirchhoff's laws"""
        validation = {
            'kcl_violations': [],  # Kirchhoff's Current Law
            'voltage_source_errors': [],  # Voltage source constraint errors
            'power_balance': None,
            'max_kcl_error': 0,
            'is_valid': True
        }
        
        if not result.node_voltages or not result.edge_currents:
            validation['is_valid'] = False
            validation['error'] = "Missing solution data"
            return validation
        
        # Check Kirchhoff's Current Law at each node
        for node in G.nodes():
            current_sum = 0
            
            # Sum currents from edges
            for neighbor in G.neighbors(node):
                if (node, neighbor) in result.edge_currents:
                    current_sum += result.edge_currents[(node, neighbor)]
                elif (neighbor, node) in result.edge_currents:
                    current_sum -= result.edge_currents[(neighbor, node)]
            
            # Add external current sources
            if current_sources and node in current_sources:
                current_sum += current_sources[node]
            
            # Check if current sum is approximately zero
            if abs(current_sum) > tolerance:
                validation['kcl_violations'].append({
                    'node': node,
                    'current_error': current_sum
                })
                validation['max_kcl_error'] = max(validation['max_kcl_error'], abs(current_sum))
        
        # Check voltage source constraints
        for node, expected_voltage in voltage_sources.items():
            if node in result.node_voltages:
                actual_voltage = result.node_voltages[node]
                error = abs(actual_voltage - expected_voltage)
                
                if error > tolerance:
                    validation['voltage_source_errors'].append({
                        'node': node,
                        'expected': expected_voltage,
                        'actual': actual_voltage,
                        'error': error
                    })
        
        # Check if solution is valid
        if validation['kcl_violations'] or validation['voltage_source_errors']:
            validation['is_valid'] = False
        
        return validation
    
    def export_results(self, filename: str) -> None:
        """Export results history to JSON file"""
        export_data = []
        for result in self.results_history:
            # Convert result to JSON-serializable format
            result_dict = {
                'algorithm': result.algorithm,
                'execution_time': result.execution_time,
                'total_power': result.total_power,
                'convergence_info': result.convergence_info,
                'matrix_properties': result.matrix_properties,
                'num_nodes': len(result.node_voltages) if result.node_voltages else 0,
                'num_edges': len(result.edge_currents) if result.edge_currents else 0
            }
            export_data.append(result_dict)
        
        with open(filename, 'w') as f:
            json.dump(export_data, f, indent=2)
    
    def clear_history(self) -> None:
        """Clear results history"""
        self.results_history = []


def create_test_circuits() -> Dict[str, Tuple[nx.Graph, Dict[int, float]]]:
    """Create test circuits for validation"""
    circuits = {}
    
    # Simple voltage divider
    G1 = nx.Graph()
    G1.add_edge(0, 1, resistance=10.0)
    G1.add_edge(1, 2, resistance=20.0)
    circuits['voltage_divider'] = (G1, {0: 10.0, 2: 0.0})
    
    # Wheatstone bridge
    G2 = nx.Graph()
    G2.add_edge(0, 1, resistance=10.0)
    G2.add_edge(0, 2, resistance=10.0)
    G2.add_edge(1, 3, resistance=20.0)
    G2.add_edge(2, 3, resistance=20.0)
    G2.add_edge(1, 2, resistance=15.0)  # Bridge resistor
    circuits['wheatstone_bridge'] = (G2, {0: 12.0, 3: 0.0})
    
    # Grid circuit
    G3 = nx.grid_2d_graph(3, 3)
    for u, v in G3.edges():
        G3[u][v]['resistance'] = np.random.uniform(1, 10)
    
    # Convert to integer nodes
    mapping = {node: i for i, node in enumerate(G3.nodes())}
    G3 = nx.relabel_nodes(G3, mapping)
    circuits['grid_circuit'] = (G3, {0: 5.0, 8: 0.0})
    
    return circuits


if __name__ == "__main__":
    # Example usage and testing
    print("Electrical Circuit Analysis Demo")
    print("=" * 40)
    
    analyzer = ElectricalCircuitAnalyzer()
    
    # Create test circuits
    circuits = create_test_circuits()
    
    for name, (circuit, voltage_sources) in circuits.items():
        print(f"\nAnalyzing {name}:")
        print(f"Nodes: {circuit.number_of_nodes()}, Edges: {circuit.number_of_edges()}")
        
        # Analyze circuit properties
        properties = analyzer.analyze_circuit_properties(circuit)
        print(f"Average resistance: {properties['avg_resistance']:.2f} Ω")
        print(f"Total conductance: {properties['total_conductance']:.4f} S")
        
        # Benchmark different solvers
        results = analyzer.benchmark_circuit_solvers(circuit, voltage_sources)
        
        for method, result in results.items():
            if result.total_power is not None:
                print(f"{method}: Power = {result.total_power:.4f} W, Time = {result.execution_time:.4f} s")
                
                # Validate solution
                validation = analyzer.validate_circuit_solution(circuit, result, voltage_sources)
                if validation['is_valid']:
                    print(f"  ✓ Solution validated")
                else:
                    print(f"  ✗ Validation failed: {len(validation['kcl_violations'])} KCL violations")
            else:
                print(f"{method}: Failed - {result.convergence_info}")
    
    # Export results
    analyzer.export_results("/workspaces/sublinear-time-solver/scripts/network_flow/electrical_results.json")
    print("\nResults exported to electrical_results.json")
