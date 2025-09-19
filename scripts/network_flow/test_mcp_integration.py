"""
Test MCP Solver Integration with Flow Conservation Matrices
============================================================

Test script to validate that the sublinear-solver-mcp can handle
flow conservation matrices and demonstrate the linear algebraic approach
to network flow problems.
"""

import numpy as np
import json
from typing import Dict, List, Tuple
import sys
import os

# Add parent directory to path for imports
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from flow_generators import NetworkSuite
from sublinear_flows import SublinearFlowSolver


def create_flow_conservation_matrix(network_data: Dict) -> Tuple[np.ndarray, np.ndarray]:
    """
    Create flow conservation constraint matrix for network.
    Returns (A, b) where Ax = b represents flow conservation.
    """
    n_nodes = network_data['n_nodes']
    n_edges = len(network_data['edges'])

    # Create node-edge incidence matrix
    A = np.zeros((n_nodes, n_edges))

    for edge_idx, (u, v) in enumerate(network_data['edges']):
        A[u, edge_idx] = 1   # Flow out of u
        A[v, edge_idx] = -1  # Flow into v

    # Set up demand vector (source = +flow, sink = -flow)
    b = np.zeros(n_nodes)
    source = network_data['source']
    sink = network_data['sink']

    target_flow = 10.0  # Test with target flow of 10
    b[source] = target_flow
    b[sink] = -target_flow

    return A, b


def create_capacity_matrix(network_data: Dict) -> np.ndarray:
    """Create diagonal matrix of edge capacities."""
    n_edges = len(network_data['edges'])
    capacities = np.zeros(n_edges)

    for edge_idx, (u, v) in enumerate(network_data['edges']):
        capacities[edge_idx] = network_data['capacities'][(u, v)]

    return np.diag(capacities)


def test_mcp_solver_with_flow_matrix():
    """Test MCP solver with flow conservation matrices."""
    print("Testing MCP Solver Integration with Flow Matrices")
    print("=" * 60)

    # Generate a small test network
    suite = NetworkSuite(seed=42)
    networks = suite.generate_test_suite('small')

    # Test with a simple grid network
    test_network = networks['2d_grid']
    source = test_network['source']
    sink = test_network['sink']

    print(f"Test Network: 2D Grid")
    print(f"  Nodes: {test_network['n_nodes']}")
    print(f"  Edges: {len(test_network['edges'])}")
    print(f"  Source: {source}, Sink: {sink}")

    # Create flow conservation matrix
    A, b = create_flow_conservation_matrix(test_network)

    print(f"\nFlow Conservation Matrix:")
    print(f"  Shape: {A.shape}")
    print(f"  Rank: {np.linalg.matrix_rank(A)}")
    print(f"  Condition number: {np.linalg.cond(A @ A.T):.2e}")

    # Test direct MCP solver interface (mock)
    print(f"\nTesting MCP Interface Format:")

    # Convert to sparse COO format as expected by MCP
    row_indices, col_indices = np.nonzero(A)
    values = A[row_indices, col_indices]

    matrix_data = {
        "rows": A.shape[0],
        "cols": A.shape[1],
        "format": "coo",
        "data": {
            "values": values.tolist(),
            "rowIndices": row_indices.tolist(),
            "colIndices": col_indices.tolist()
        }
    }

    print(f"  COO format - nnz: {len(values)}")
    print(f"  Matrix density: {len(values) / (A.shape[0] * A.shape[1]):.3f}")

    # Test with augmented system for capacity constraints
    print(f"\nCreating Augmented System with Capacity Constraints:")

    # Create regularized system that's always solvable
    # [A^T A + λI] x = A^T b (normal equations with regularization)
    ATA = A.T @ A
    regularization = 0.01 * np.eye(ATA.shape[0])
    system_matrix = ATA + regularization
    rhs = A.T @ b

    print(f"  Augmented matrix shape: {system_matrix.shape}")
    print(f"  Condition number: {np.linalg.cond(system_matrix):.2e}")
    print(f"  Is positive definite: {np.all(np.linalg.eigvals(system_matrix) > 0)}")

    # Test diagonal dominance
    diag_elements = np.diag(system_matrix)
    off_diag_sums = np.sum(np.abs(system_matrix), axis=1) - np.abs(diag_elements)
    diag_dominant = np.all(np.abs(diag_elements) >= off_diag_sums)

    print(f"  Diagonally dominant: {diag_dominant}")
    if not diag_dominant:
        # Make it diagonally dominant
        min_diagonal = np.max(off_diag_sums)
        system_matrix += (min_diagonal * 1.1) * np.eye(system_matrix.shape[0])
        print(f"  Made diagonally dominant with additional regularization")

    # Test the SublinearFlowSolver approach
    print(f"\nTesting SublinearFlowSolver:")

    solver = SublinearFlowSolver(test_network['n_nodes'])
    for u, v in test_network['edges']:
        capacity = test_network['capacities'][(u, v)]
        cost = test_network['costs'][(u, v)]
        solver.add_edge(u, v, capacity, cost)

    result = solver.solve_max_flow_as_linear_system(source, sink)

    print(f"  Max flow found: {result['max_flow']:.3f}")
    print(f"  Iterations: {result['iterations']}")
    print(f"  Success: {result['max_flow'] > 0}")

    # Compare with numpy direct solve
    print(f"\nComparing with Direct Linear Algebra:")

    try:
        # Solve regularized system directly
        flows_direct = np.linalg.solve(system_matrix, rhs)

        # Project back to check conservation
        conservation_error = np.linalg.norm(A @ flows_direct - b)

        print(f"  Direct solve completed")
        print(f"  Conservation error: {conservation_error:.2e}")
        print(f"  Flow vector norm: {np.linalg.norm(flows_direct):.3f}")

        # Check capacity constraints
        capacity_violations = 0
        for edge_idx, (u, v) in enumerate(test_network['edges']):
            capacity = test_network['capacities'][(u, v)]
            flow = flows_direct[edge_idx] if edge_idx < len(flows_direct) else 0
            if flow > capacity:
                capacity_violations += 1

        print(f"  Capacity violations: {capacity_violations}")

    except Exception as e:
        print(f"  Direct solve failed: {e}")

    return {
        'network_size': test_network['n_nodes'],
        'matrix_shape': A.shape,
        'condition_number': np.linalg.cond(system_matrix),
        'diagonally_dominant': diag_dominant,
        'sublinear_flow': result['max_flow'],
        'sublinear_iterations': result['iterations']
    }


def test_different_network_types():
    """Test MCP integration across different network topologies."""
    print("\n" + "=" * 60)
    print("Testing Different Network Topologies")
    print("=" * 60)

    suite = NetworkSuite(seed=42)
    networks = suite.generate_test_suite('small')

    results = {}

    for network_name, network_data in networks.items():
        print(f"\nTesting {network_name}:")

        try:
            # Create matrices
            A, b = create_flow_conservation_matrix(network_data)

            # Check matrix properties
            rank = np.linalg.matrix_rank(A)
            if A.shape[0] > 0 and A.shape[1] > 0:
                cond_num = np.linalg.cond(A @ A.T + 1e-6 * np.eye(A.shape[0]))
            else:
                cond_num = float('inf')

            # Test sublinear solver
            solver = SublinearFlowSolver(network_data['n_nodes'])
            for u, v in network_data['edges']:
                capacity = network_data['capacities'][(u, v)]
                cost = network_data['costs'][(u, v)]
                solver.add_edge(u, v, capacity, cost)

            result = solver.solve_max_flow_as_linear_system(
                network_data['source'], network_data['sink']
            )

            results[network_name] = {
                'nodes': network_data['n_nodes'],
                'edges': len(network_data['edges']),
                'matrix_rank': rank,
                'condition_number': cond_num,
                'max_flow': result['max_flow'],
                'iterations': result['iterations'],
                'success': result['max_flow'] > 0
            }

            print(f"  Nodes: {network_data['n_nodes']}, Edges: {len(network_data['edges'])}")
            print(f"  Matrix rank: {rank}")
            print(f"  Condition number: {cond_num:.2e}")
            print(f"  Max flow: {result['max_flow']:.3f}")
            print(f"  Success: {result['max_flow'] > 0}")

        except Exception as e:
            print(f"  Error: {e}")
            results[network_name] = {'error': str(e)}

    return results


def analyze_mcp_solver_requirements():
    """Analyze what makes a good matrix for MCP solver."""
    print("\n" + "=" * 60)
    print("Analyzing MCP Solver Requirements")
    print("=" * 60)

    # Generate different matrix types and analyze their properties
    test_cases = []

    # 1. Well-conditioned diagonally dominant matrix
    n = 10
    A1 = 2 * np.eye(n) + 0.1 * np.random.randn(n, n)
    np.fill_diagonal(A1, np.sum(np.abs(A1), axis=1) + 1)  # Make diagonally dominant
    b1 = np.random.randn(n)

    test_cases.append(("Diagonally Dominant", A1, b1))

    # 2. Symmetric positive definite matrix
    B = np.random.randn(n, n)
    A2 = B.T @ B + np.eye(n)
    b2 = np.random.randn(n)

    test_cases.append(("Symmetric Positive Definite", A2, b2))

    # 3. Flow conservation matrix (rectangular)
    grid_size = 3
    n_nodes = grid_size * grid_size
    edges = []
    for i in range(grid_size):
        for j in range(grid_size):
            node = i * grid_size + j
            if j < grid_size - 1:  # Right edge
                edges.append((node, node + 1))
            if i < grid_size - 1:  # Down edge
                edges.append((node, node + grid_size))

    A3 = np.zeros((n_nodes, len(edges)))
    for edge_idx, (u, v) in enumerate(edges):
        A3[u, edge_idx] = 1
        A3[v, edge_idx] = -1

    b3 = np.zeros(n_nodes)
    b3[0] = 1  # Source
    b3[-1] = -1  # Sink

    test_cases.append(("Flow Conservation", A3, b3))

    # Analyze each case
    for name, A, b in test_cases:
        print(f"\n{name}:")
        print(f"  Shape: {A.shape}")
        print(f"  Rank: {np.linalg.matrix_rank(A)}")

        if A.shape[0] == A.shape[1]:
            # Square matrix
            eigenvals = np.linalg.eigvals(A)
            print(f"  Condition number: {np.linalg.cond(A):.2e}")
            print(f"  Determinant: {np.linalg.det(A):.2e}")
            print(f"  Min eigenvalue: {np.min(eigenvals):.2e}")
            print(f"  Max eigenvalue: {np.max(eigenvals):.2e}")

            # Check diagonal dominance
            diag_elements = np.diag(A)
            off_diag_sums = np.sum(np.abs(A), axis=1) - np.abs(diag_elements)
            diag_dominant = np.all(np.abs(diag_elements) >= off_diag_sums)
            print(f"  Diagonally dominant: {diag_dominant}")

        else:
            # Rectangular matrix - use normal equations
            AtA = A.T @ A
            print(f"  A^T A condition number: {np.linalg.cond(AtA):.2e}")
            print(f"  A^T A min eigenvalue: {np.min(np.linalg.eigvals(AtA)):.2e}")

    return test_cases


if __name__ == "__main__":
    # Run all tests
    print("MCP Solver Integration Tests")
    print("=" * 80)

    # Main test
    main_result = test_mcp_solver_with_flow_matrix()

    # Test different topologies
    topology_results = test_different_network_types()

    # Analyze solver requirements
    matrix_analysis = analyze_mcp_solver_requirements()

    # Summary
    print("\n" + "=" * 80)
    print("TEST SUMMARY")
    print("=" * 80)

    print(f"\nMain Test Results:")
    for key, value in main_result.items():
        print(f"  {key}: {value}")

    print(f"\nTopology Test Results:")
    successful_networks = [name for name, result in topology_results.items()
                          if 'success' in result and result['success']]
    print(f"  Successful networks: {len(successful_networks)}/{len(topology_results)}")
    print(f"  Success rate: {len(successful_networks)/len(topology_results):.1%}")

    if successful_networks:
        print(f"  Successful topologies: {', '.join(successful_networks)}")

    # Save results
    output_file = "/workspaces/sublinear-time-solver/scripts/network_flow/mcp_integration_results.json"
    with open(output_file, 'w') as f:
        json.dump({
            'main_test': main_result,
            'topology_tests': topology_results,
            'timestamp': __import__('time').time()
        }, f, indent=2)

    print(f"\nDetailed results saved to: {output_file}")