#!/usr/bin/env python3
"""
Opinion Dynamics and Friedkin-Johnsen Model Implementation

This module implements the Friedkin-Johnsen opinion dynamics model and other
consensus mechanisms using both traditional simulation and sublinear linear algebraic methods.

The Friedkin-Johnsen model captures how opinions evolve in social networks when
individuals have both social influence and individual susceptibility to change.

Author: Social Network Analysis Agent
"""

import numpy as np
import networkx as nx
import time
import json
from typing import Dict, List, Tuple, Optional, Any, Union
import scipy.sparse as sp
from scipy.sparse.linalg import spsolve
import matplotlib.pyplot as plt
import seaborn as sns
import tracemalloc


class FriedkinJohnsenModel:
    """
    Friedkin-Johnsen opinion dynamics model implementation.

    The model is defined by:
    x = Λ * W * x + (I - Λ) * s

    Where:
    - x: final opinion vector
    - Λ: diagonal matrix of susceptibilities (how much individuals change)
    - W: row-normalized adjacency matrix (social influence network)
    - s: initial opinion vector (stubborn opinions)

    At equilibrium: x = (I - Λ*W)^(-1) * (I - Λ) * s
    """

    def __init__(self, graph: nx.Graph, susceptibilities: Union[float, Dict, np.ndarray] = 0.5):
        """
        Initialize Friedkin-Johnsen model.

        Args:
            graph: Social network as NetworkX graph
            susceptibilities: Individual susceptibilities to change
                            - float: uniform susceptibility for all nodes
                            - dict: {node: susceptibility} mapping
                            - array: susceptibilities in node order
        """
        self.graph = graph
        self.nodes = list(graph.nodes())
        self.n_nodes = len(self.nodes)
        self.node_to_idx = {node: i for i, node in enumerate(self.nodes)}

        # Set up susceptibilities
        self.susceptibilities = self._setup_susceptibilities(susceptibilities)

        # Create network matrices
        self.adj_matrix = nx.adjacency_matrix(self.graph, nodelist=self.nodes)
        self.influence_matrix = self._create_influence_matrix()

        self.results = {}

    def _setup_susceptibilities(self, susceptibilities: Union[float, Dict, np.ndarray]) -> np.ndarray:
        """Set up susceptibility vector."""
        if isinstance(susceptibilities, float):
            return np.full(self.n_nodes, susceptibilities)
        elif isinstance(susceptibilities, dict):
            return np.array([susceptibilities.get(node, 0.5) for node in self.nodes])
        elif isinstance(susceptibilities, (list, np.ndarray)):
            return np.array(susceptibilities)
        else:
            raise ValueError("Susceptibilities must be float, dict, or array")

    def _create_influence_matrix(self) -> sp.csr_matrix:
        """Create row-normalized influence matrix W."""
        # Get degree sequence
        degrees = np.array([self.graph.degree(node) for node in self.nodes])

        # Handle isolated nodes
        degrees[degrees == 0] = 1

        # Create row-normalized matrix
        influence_matrix = self.adj_matrix.multiply(1.0 / degrees[:, np.newaxis])
        return influence_matrix.tocsr()

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

    def compute_equilibrium_opinions_sublinear(self, initial_opinions: Union[Dict, np.ndarray],
                                             method: str = "neumann") -> Dict[str, Any]:
        """
        Compute equilibrium opinions using sublinear solver.

        Solves: (I - Λ*W) * x = (I - Λ) * s
        """
        print("Computing Friedkin-Johnsen equilibrium using sublinear solver...")

        start_time = time.time()
        tracemalloc.start()

        # Setup initial opinions vector
        if isinstance(initial_opinions, dict):
            s = np.array([initial_opinions.get(node, 0.0) for node in self.nodes])
        else:
            s = np.array(initial_opinions)

        # Create susceptibility matrix Λ
        Lambda = sp.diags(self.susceptibilities, format='csr')

        # System matrix: I - Λ*W
        I = sp.identity(self.n_nodes, format='csr')
        system_matrix = I - Lambda @ self.influence_matrix

        # Right-hand side: (I - Λ) * s
        rhs = (I - Lambda) @ s

        try:
            # Solve using MCP
            equilibrium_opinions = self._call_mcp_solve(system_matrix, rhs, method)

            # Convert to dict
            opinion_dict = {self.nodes[i]: float(equilibrium_opinions[i]) for i in range(self.n_nodes)}

            # Calculate convergence metrics
            opinion_change = np.linalg.norm(equilibrium_opinions - s)
            max_change = np.max(np.abs(equilibrium_opinions - s))

            success = True

        except Exception as e:
            print(f"Equilibrium computation failed: {e}")
            opinion_dict = {node: 0.0 for node in self.nodes}
            equilibrium_opinions = np.zeros(self.n_nodes)
            opinion_change = 0.0
            max_change = 0.0
            success = False

        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        computation_time = time.time() - start_time

        result = {
            'initial_opinions': s.tolist(),
            'equilibrium_opinions': opinion_dict,
            'equilibrium_vector': equilibrium_opinions.tolist(),
            'susceptibilities': self.susceptibilities.tolist(),
            'opinion_change_norm': float(opinion_change),
            'max_opinion_change': float(max_change),
            'computation_time': computation_time,
            'memory_peak': peak,
            'method': f'linear_system_{method}',
            'success': success
        }

        return result

    def simulate_opinion_dynamics_traditional(self, initial_opinions: Union[Dict, np.ndarray],
                                            max_iterations: int = 1000,
                                            tolerance: float = 1e-6) -> Dict[str, Any]:
        """
        Simulate opinion dynamics using traditional iterative method.

        Iterates: x^(t+1) = Λ * W * x^(t) + (I - Λ) * s
        """
        print("Simulating Friedkin-Johnsen dynamics using traditional iteration...")

        start_time = time.time()
        tracemalloc.start()

        # Setup initial opinions
        if isinstance(initial_opinions, dict):
            s = np.array([initial_opinions.get(node, 0.0) for node in self.nodes])
        else:
            s = np.array(initial_opinions)

        # Initialize opinions
        x = s.copy()
        history = [x.copy()]

        # Create susceptibility matrices
        Lambda = sp.diags(self.susceptibilities, format='csr')
        I = sp.identity(self.n_nodes, format='csr')

        # Precompute matrices
        Lambda_W = Lambda @ self.influence_matrix
        I_minus_Lambda_s = (I - Lambda) @ s

        # Iterate until convergence
        for iteration in range(max_iterations):
            x_new = Lambda_W @ x + I_minus_Lambda_s

            # Check convergence
            change = np.linalg.norm(x_new - x)
            x = x_new
            history.append(x.copy())

            if change < tolerance:
                break

        # Convert final opinions to dict
        final_opinions = {self.nodes[i]: float(x[i]) for i in range(self.n_nodes)}

        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        computation_time = time.time() - start_time

        result = {
            'initial_opinions': s.tolist(),
            'final_opinions': final_opinions,
            'final_vector': x.tolist(),
            'susceptibilities': self.susceptibilities.tolist(),
            'iterations': iteration + 1,
            'converged': iteration < max_iterations - 1,
            'final_change': float(change),
            'opinion_history': [h.tolist() for h in history],
            'computation_time': computation_time,
            'memory_peak': peak,
            'method': 'iterative_simulation'
        }

        return result

    def analyze_opinion_consensus(self, scenarios: List[Dict]) -> Dict[str, Any]:
        """
        Analyze opinion consensus under different scenarios.

        Args:
            scenarios: List of scenario dicts with 'initial_opinions' and optional 'susceptibilities'
        """
        print("Analyzing opinion consensus across scenarios...")

        results = {}

        for i, scenario in enumerate(scenarios):
            scenario_name = scenario.get('name', f'scenario_{i+1}')
            print(f"  Running {scenario_name}...")

            # Update susceptibilities if provided
            if 'susceptibilities' in scenario:
                old_susceptibilities = self.susceptibilities.copy()
                self.susceptibilities = self._setup_susceptibilities(scenario['susceptibilities'])

            # Run both methods
            try:
                # Sublinear method
                sublinear_result = self.compute_equilibrium_opinions_sublinear(
                    scenario['initial_opinions'], method="neumann"
                )

                # Traditional method
                traditional_result = self.simulate_opinion_dynamics_traditional(
                    scenario['initial_opinions']
                )

                # Compare results
                if sublinear_result['success'] and traditional_result['converged']:
                    sub_opinions = np.array(sublinear_result['equilibrium_vector'])
                    trad_opinions = np.array(traditional_result['final_vector'])

                    mse = np.mean((sub_opinions - trad_opinions) ** 2)
                    max_diff = np.max(np.abs(sub_opinions - trad_opinions))
                    correlation = np.corrcoef(sub_opinions, trad_opinions)[0, 1]

                    comparison = {
                        'mse': float(mse),
                        'max_difference': float(max_diff),
                        'correlation': float(correlation),
                        'speedup': traditional_result['computation_time'] / sublinear_result['computation_time']
                    }
                else:
                    comparison = {'error': 'One or both methods failed'}

                results[scenario_name] = {
                    'scenario': scenario,
                    'sublinear_result': sublinear_result,
                    'traditional_result': traditional_result,
                    'comparison': comparison
                }

            except Exception as e:
                print(f"    Failed: {e}")
                results[scenario_name] = {'error': str(e)}

            # Restore susceptibilities if changed
            if 'susceptibilities' in scenario:
                self.susceptibilities = old_susceptibilities

        return results

    def analyze_polarization_dynamics(self, polarized_opinions: Dict[str, float],
                                    susceptibility_range: Tuple[float, float] = (0.1, 0.9)) -> Dict[str, Any]:
        """
        Analyze how susceptibilities affect polarization and consensus.

        Args:
            polarized_opinions: Initial polarized opinions
            susceptibility_range: Range of susceptibilities to test
        """
        print("Analyzing polarization dynamics...")

        start_time = time.time()

        # Test different uniform susceptibility levels
        susceptibility_levels = np.linspace(susceptibility_range[0], susceptibility_range[1], 10)

        results = {
            'susceptibility_levels': susceptibility_levels.tolist(),
            'consensus_measures': [],
            'polarization_measures': [],
            'computation_times': []
        }

        original_susceptibilities = self.susceptibilities.copy()

        for susceptibility in susceptibility_levels:
            print(f"  Testing susceptibility = {susceptibility:.2f}")

            # Set uniform susceptibility
            self.susceptibilities = np.full(self.n_nodes, susceptibility)

            try:
                # Compute equilibrium
                result = self.compute_equilibrium_opinions_sublinear(polarized_opinions)

                if result['success']:
                    opinions = np.array(result['equilibrium_vector'])

                    # Consensus measure: 1 - variance of opinions
                    consensus = 1.0 - np.var(opinions)

                    # Polarization measure: bimodality coefficient
                    mean_op = np.mean(opinions)
                    std_op = np.std(opinions)
                    skew = np.mean(((opinions - mean_op) / std_op) ** 3) if std_op > 0 else 0
                    kurt = np.mean(((opinions - mean_op) / std_op) ** 4) if std_op > 0 else 0
                    polarization = (skew**2 + 1) / (kurt + 3 * (self.n_nodes - 1)**2 / ((self.n_nodes - 2) * (self.n_nodes - 3)))

                    results['consensus_measures'].append(float(consensus))
                    results['polarization_measures'].append(float(polarization))
                    results['computation_times'].append(result['computation_time'])
                else:
                    results['consensus_measures'].append(None)
                    results['polarization_measures'].append(None)
                    results['computation_times'].append(None)

            except Exception as e:
                print(f"    Failed: {e}")
                results['consensus_measures'].append(None)
                results['polarization_measures'].append(None)
                results['computation_times'].append(None)

        # Restore original susceptibilities
        self.susceptibilities = original_susceptibilities

        total_time = time.time() - start_time
        results['total_analysis_time'] = total_time

        return results

    def save_results(self, results: Dict[str, Any], filename: str = 'opinion_dynamics_results.json'):
        """Save results to JSON file."""
        output_path = f'/workspaces/sublinear-time-solver/scripts/social_networks/{filename}'

        # Add metadata
        output_data = {
            'graph_info': {
                'n_nodes': self.n_nodes,
                'n_edges': len(self.graph.edges()),
                'average_degree': 2 * len(self.graph.edges()) / self.n_nodes if self.n_nodes > 0 else 0
            },
            'model_parameters': {
                'susceptibilities': self.susceptibilities.tolist(),
                'model_type': 'friedkin_johnsen'
            },
            'results': results
        }

        with open(output_path, 'w') as f:
            json.dump(output_data, f, indent=2, default=str)

        print(f"Results saved to {output_path}")


class VoterModel:
    """
    Simple voter model for comparison with Friedkin-Johnsen.

    In the voter model, each node adopts the opinion of a random neighbor
    at each time step.
    """

    def __init__(self, graph: nx.Graph):
        self.graph = graph
        self.nodes = list(graph.nodes())
        self.n_nodes = len(self.nodes)

    def simulate_voter_dynamics(self, initial_opinions: Dict[str, int],
                              max_iterations: int = 1000) -> Dict[str, Any]:
        """
        Simulate voter model dynamics.

        Args:
            initial_opinions: Dict mapping nodes to binary opinions (0 or 1)
            max_iterations: Maximum simulation steps
        """
        print("Simulating voter model dynamics...")

        start_time = time.time()

        # Initialize opinions
        opinions = {node: initial_opinions.get(node, 0) for node in self.nodes}
        history = [opinions.copy()]

        # Simulate dynamics
        for iteration in range(max_iterations):
            new_opinions = opinions.copy()

            # Update each node
            for node in self.nodes:
                neighbors = list(self.graph.neighbors(node))
                if neighbors:
                    # Choose random neighbor and adopt their opinion
                    random_neighbor = np.random.choice(neighbors)
                    new_opinions[node] = opinions[random_neighbor]

            opinions = new_opinions
            history.append(opinions.copy())

            # Check for consensus
            unique_opinions = set(opinions.values())
            if len(unique_opinions) == 1:
                break

        computation_time = time.time() - start_time

        # Calculate final statistics
        final_counts = {op: sum(1 for v in opinions.values() if v == op)
                       for op in set(opinions.values())}

        result = {
            'initial_opinions': initial_opinions,
            'final_opinions': opinions,
            'iterations': iteration + 1,
            'converged_to_consensus': len(set(opinions.values())) == 1,
            'final_opinion_counts': final_counts,
            'opinion_history': history,
            'computation_time': computation_time,
            'method': 'voter_model_simulation'
        }

        return result


def create_test_scenarios() -> List[Dict[str, Any]]:
    """Create test scenarios for opinion dynamics analysis."""
    scenarios = [
        {
            'name': 'polarized_extreme',
            'initial_opinions': {i: 1.0 if i < 17 else -1.0 for i in range(34)},  # Karate club split
            'susceptibilities': 0.7
        },
        {
            'name': 'polarized_moderate',
            'initial_opinions': {i: 0.5 if i < 17 else -0.5 for i in range(34)},
            'susceptibilities': 0.5
        },
        {
            'name': 'random_opinions',
            'initial_opinions': {i: np.random.normal(0, 0.5) for i in range(34)},
            'susceptibilities': 0.3
        },
        {
            'name': 'single_influencer',
            'initial_opinions': {0: 1.0, **{i: 0.0 for i in range(1, 34)}},
            'susceptibilities': 0.8
        },
        {
            'name': 'heterogeneous_susceptibility',
            'initial_opinions': {i: 1.0 if i < 17 else -1.0 for i in range(34)},
            'susceptibilities': {i: 0.9 if i in [0, 33] else 0.3 for i in range(34)}  # Leaders less susceptible
        }
    ]

    return scenarios


def main():
    """Run opinion dynamics analysis."""
    print("=" * 60)
    print("Opinion Dynamics Analysis")
    print("=" * 60)

    # Test on Karate Club graph
    G = nx.karate_club_graph()
    print(f"Testing on Karate Club graph ({len(G.nodes())} nodes, {len(G.edges())} edges)")

    # Initialize Friedkin-Johnsen model
    fj_model = FriedkinJohnsenModel(G, susceptibilities=0.5)

    # Create test scenarios
    scenarios = create_test_scenarios()

    # Run consensus analysis
    print("\n1. Running consensus analysis...")
    consensus_results = fj_model.analyze_opinion_consensus(scenarios)
    fj_model.save_results(consensus_results, 'fj_consensus_results.json')

    # Run polarization analysis
    print("\n2. Running polarization analysis...")
    polarized_opinions = {i: 1.0 if i < 17 else -1.0 for i in range(34)}
    polarization_results = fj_model.analyze_polarization_dynamics(polarized_opinions)
    fj_model.save_results(polarization_results, 'fj_polarization_results.json')

    # Compare with voter model
    print("\n3. Comparing with voter model...")
    voter_model = VoterModel(G)
    binary_opinions = {i: 1 if i < 17 else 0 for i in range(34)}
    voter_results = voter_model.simulate_voter_dynamics(binary_opinions)

    # Save voter model results
    output_path = '/workspaces/sublinear-time-solver/scripts/social_networks/voter_model_results.json'
    with open(output_path, 'w') as f:
        json.dump(voter_results, f, indent=2, default=str)

    print("\n" + "=" * 60)
    print("Opinion dynamics analysis complete!")
    print("Results saved:")
    print("- fj_consensus_results.json")
    print("- fj_polarization_results.json")
    print("- voter_model_results.json")
    print("=" * 60)


if __name__ == "__main__":
    main()