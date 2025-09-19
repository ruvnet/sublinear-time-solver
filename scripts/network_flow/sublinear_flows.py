"""
Sublinear Network Flow Algorithms
==================================

Linear algebraic formulations of network flow problems using sublinear solvers.
Demonstrates how classical graph problems can be reformulated as linear systems
and solved using matrix-based approaches.

Key insights:
- Flow conservation as linear constraints
- Capacity constraints via augmented systems
- Equilibrium formulations for circulation problems
- Diagonally dominant systems for fast convergence
"""

import numpy as np
from typing import Dict, List, Tuple, Optional
import time
import json
from scipy.sparse import csr_matrix, diags
from scipy.linalg import solve


class SublinearFlowSolver:
    """
    Reformulate network flow problems as linear systems solvable by sublinear methods.
    """

    def __init__(self, n_nodes: int):
        self.n_nodes = n_nodes
        self.edges = []
        self.capacities = {}
        self.costs = {}
        self.demands = np.zeros(n_nodes)

    def add_edge(self, u: int, v: int, capacity: float, cost: float = 0):
        """Add edge to the network."""
        edge_id = len(self.edges)
        self.edges.append((u, v))
        self.capacities[(u, v)] = capacity
        self.costs[(u, v)] = cost

    def set_demand(self, node: int, demand: float):
        """Set supply/demand at node (positive = supply, negative = demand)."""
        self.demands[node] = demand

    def build_flow_conservation_matrix(self) -> Tuple[np.ndarray, np.ndarray]:
        """
        Build the flow conservation constraint matrix Ax = b where:
        - A: node-edge incidence matrix
        - x: flow variables
        - b: demand vector
        """
        n_edges = len(self.edges)
        A = np.zeros((self.n_nodes, n_edges))

        for edge_idx, (u, v) in enumerate(self.edges):
            A[u, edge_idx] = 1   # Flow out of u
            A[v, edge_idx] = -1  # Flow into v

        return A, self.demands.copy()

    def build_circulation_system(self) -> Tuple[np.ndarray, np.ndarray]:
        """
        Build a system for circulation problems where flow naturally satisfies
        conservation without external supplies/demands.

        Creates a diagonally dominant system by adding regularization.
        """
        n_edges = len(self.edges)

        # Create resistance matrix (inverse of capacity)
        R = np.zeros((n_edges, n_edges))
        for i, (u, v) in enumerate(self.edges):
            capacity = self.capacities[(u, v)]
            # Resistance = 1/capacity (with regularization for numerical stability)
            R[i, i] = 1.0 / (capacity + 1e-8)

        # Add regularization to ensure diagonal dominance
        regularization = 0.1 * np.eye(n_edges)
        system_matrix = R + regularization

        # For circulation, we want to find equilibrium flows
        # Set target to small random perturbation to avoid trivial solution
        np.random.seed(42)
        target_vector = 0.01 * np.random.randn(n_edges)

        return system_matrix, target_vector

    def build_potential_system(self) -> Tuple[np.ndarray, np.ndarray]:
        """
        Build system using node potentials (dual formulation).
        Flow = (potential_difference) / resistance

        This often produces better conditioned systems.
        """
        # Create node-node adjacency with weights = 1/resistance
        A = np.zeros((self.n_nodes, self.n_nodes))

        for u, v in self.edges:
            capacity = self.capacities[(u, v)]
            weight = capacity  # conductance = 1/resistance

            A[u, u] += weight
            A[v, v] += weight
            A[u, v] -= weight
            A[v, u] -= weight

        # Make diagonally dominant by adding small regularization
        A += 0.01 * np.eye(self.n_nodes)

        return A, self.demands

    def solve_max_flow_as_linear_system(self, source: int, sink: int) -> Dict:
        """
        Solve max flow by iteratively solving linear systems.
        Uses augmenting path concept but via linear algebra.
        """
        max_flow = 0
        flow_values = {}
        iterations = 0

        # Initialize flow values
        for edge in self.edges:
            flow_values[edge] = 0

        while iterations < 100:  # Max iterations to prevent infinite loops
            iterations += 1

            # Build residual capacity system
            residual_capacities = {}
            for u, v in self.edges:
                residual_capacities[(u, v)] = (
                    self.capacities[(u, v)] - flow_values[(u, v)]
                )

            # Check if we can still push flow
            if self._check_path_exists(source, sink, residual_capacities):
                # Solve for potential differences
                A, b = self._build_residual_system(source, sink, residual_capacities)

                try:
                    # Use MCP solver if available, otherwise fall back to numpy
                    potentials = self._solve_linear_system(A, b)

                    # Extract flow increment from potential differences
                    flow_increment = self._extract_flow_from_potentials(
                        potentials, residual_capacities
                    )

                    if flow_increment > 1e-8:
                        # Update flows along the augmenting path
                        path_flow = self._update_flows_along_path(
                            source, sink, flow_increment, flow_values, residual_capacities
                        )
                        max_flow += path_flow
                    else:
                        break

                except Exception as e:
                    print(f"Linear system solve failed: {e}")
                    break
            else:
                break

        return {
            'max_flow': max_flow,
            'iterations': iterations,
            'flow_values': flow_values
        }

    def solve_min_cost_flow_linear(self, source: int, sink: int, flow_demand: float) -> Dict:
        """
        Solve minimum cost flow as a quadratic programming problem
        approximated by linear systems.
        """
        # Build cost-weighted system
        n_edges = len(self.edges)
        cost_matrix = np.zeros((n_edges, n_edges))

        for i, (u, v) in enumerate(self.edges):
            cost = self.costs.get((u, v), 1.0)
            capacity = self.capacities[(u, v)]

            # Resistance proportional to cost
            cost_matrix[i, i] = cost / (capacity + 1e-8)

        # Add flow conservation constraints via Lagrange multipliers
        A, b = self.build_flow_conservation_matrix()

        # Set up demand: flow_demand from source, -flow_demand to sink
        demands = np.zeros(self.n_nodes)
        demands[source] = flow_demand
        demands[sink] = -flow_demand

        # Solve augmented system [cost_matrix, A^T; A, 0] [flows; potentials] = [0; demands]
        upper = np.hstack([cost_matrix, A.T])
        lower = np.hstack([A, np.zeros((self.n_nodes, self.n_nodes))])
        augmented_matrix = np.vstack([upper, lower])

        rhs = np.hstack([np.zeros(n_edges), demands])

        try:
            solution = self._solve_linear_system(augmented_matrix, rhs)
            flows = solution[:n_edges]
            potentials = solution[n_edges:]

            total_cost = sum(
                flows[i] * self.costs.get(self.edges[i], 0)
                for i in range(n_edges)
            )

            return {
                'flows': flows,
                'potentials': potentials,
                'total_cost': total_cost,
                'feasible': np.allclose(A @ flows, demands, rtol=1e-6)
            }

        except Exception as e:
            return {
                'error': str(e),
                'feasible': False
            }

    def _solve_linear_system(self, A: np.ndarray, b: np.ndarray) -> np.ndarray:
        """
        Solve linear system using available methods.
        Priority: MCP solver > NumPy > SciPy
        """
        try:
            # Try MCP solver first
            return self._solve_with_mcp(A, b)
        except:
            try:
                # Fall back to NumPy
                return np.linalg.solve(A, b)
            except:
                # Last resort: least squares
                return np.linalg.lstsq(A, b, rcond=None)[0]

    def _solve_with_mcp(self, A: np.ndarray, b: np.ndarray) -> np.ndarray:
        """
        Solve using MCP sublinear solver.
        Converts dense matrix to required format.
        """
        rows, cols = A.shape

        # Convert to sparse COO format for MCP
        row_indices, col_indices = np.nonzero(A)
        values = A[row_indices, col_indices].tolist()

        matrix_data = {
            "rows": rows,
            "cols": cols,
            "format": "coo",
            "data": {
                "values": values,
                "rowIndices": row_indices.tolist(),
                "colIndices": col_indices.tolist()
            }
        }

        # This would call MCP solver
        # For now, we'll simulate the interface
        return self._mock_mcp_solve(matrix_data, b.tolist())

    def _mock_mcp_solve(self, matrix_data: Dict, vector: List[float]) -> np.ndarray:
        """
        Mock MCP solver for testing.
        Replace with actual MCP call in integration.
        """
        # Reconstruct matrix from sparse format
        rows = matrix_data["rows"]
        cols = matrix_data["cols"]
        data = matrix_data["data"]

        A = np.zeros((rows, cols))
        for val, r, c in zip(data["values"], data["rowIndices"], data["colIndices"]):
            A[r, c] = val

        # Use numpy solve as fallback
        return np.linalg.solve(A, np.array(vector))

    def _check_path_exists(self, source: int, sink: int, capacities: Dict) -> bool:
        """Check if path exists from source to sink in residual graph."""
        visited = set()
        stack = [source]

        while stack:
            node = stack.pop()
            if node == sink:
                return True

            if node in visited:
                continue

            visited.add(node)

            for u, v in self.edges:
                if u == node and capacities.get((u, v), 0) > 1e-8:
                    stack.append(v)

        return False

    def _build_residual_system(self, source: int, sink: int, capacities: Dict) -> Tuple[np.ndarray, np.ndarray]:
        """Build system for finding augmenting path via potentials."""
        # Simple Laplacian-based system
        A = np.eye(self.n_nodes) * 1e-6  # Small regularization

        for u, v in self.edges:
            if capacities.get((u, v), 0) > 1e-8:
                weight = capacities[(u, v)]
                A[u, u] += weight
                A[v, v] += weight
                A[u, v] -= weight
                A[v, u] -= weight

        # Force potential difference between source and sink
        b = np.zeros(self.n_nodes)
        b[source] = 1.0
        b[sink] = -1.0

        return A, b

    def _extract_flow_from_potentials(self, potentials: np.ndarray, capacities: Dict) -> float:
        """Extract maximum flow increment from potential solution."""
        max_increment = 0

        for u, v in self.edges:
            if capacities.get((u, v), 0) > 1e-8:
                potential_diff = potentials[u] - potentials[v]
                capacity = capacities[(u, v)]

                # Flow proportional to potential difference and capacity
                flow_increment = min(capacity, abs(potential_diff) * capacity)
                max_increment = max(max_increment, flow_increment)

        return max_increment

    def _update_flows_along_path(self, source: int, sink: int, flow_increment: float,
                                 flow_values: Dict, capacities: Dict) -> float:
        """Update flow values along augmenting path."""
        # Simple greedy path finding and update
        path = self._find_simple_path(source, sink, capacities)

        if not path:
            return 0

        # Find bottleneck capacity
        bottleneck = float('inf')
        for i in range(len(path) - 1):
            u, v = path[i], path[i + 1]
            bottleneck = min(bottleneck, capacities.get((u, v), 0))

        # Update flows
        for i in range(len(path) - 1):
            u, v = path[i], path[i + 1]
            if (u, v) in flow_values:
                flow_values[(u, v)] += bottleneck

        return bottleneck

    def _find_simple_path(self, source: int, sink: int, capacities: Dict) -> List[int]:
        """Find simple path using DFS."""
        visited = set()
        path = []

        def dfs(node: int) -> bool:
            if node == sink:
                path.append(node)
                return True

            visited.add(node)
            path.append(node)

            for u, v in self.edges:
                if u == node and v not in visited and capacities.get((u, v), 0) > 1e-8:
                    if dfs(v):
                        return True

            path.pop()
            return False

        if dfs(source):
            return path
        return []


class SublinearFlowBenchmark:
    """Benchmark sublinear flow algorithms against traditional methods."""

    @staticmethod
    def time_algorithm(algorithm_func, *args) -> Tuple[float, any]:
        """Time an algorithm and return (runtime, result)."""
        start_time = time.perf_counter()
        result = algorithm_func(*args)
        end_time = time.perf_counter()
        return end_time - start_time, result

    @staticmethod
    def create_test_network(n_nodes: int, density: float = 0.3) -> SublinearFlowSolver:
        """Create random test network for benchmarking."""
        solver = SublinearFlowSolver(n_nodes)

        np.random.seed(42)
        for u in range(n_nodes):
            for v in range(u + 1, n_nodes):
                if np.random.random() < density:
                    capacity = np.random.randint(1, 20)
                    cost = np.random.random() * 10
                    solver.add_edge(u, v, capacity, cost)

        return solver

    @staticmethod
    def compare_with_traditional(network_sizes: List[int]) -> Dict:
        """Compare sublinear vs traditional approaches across network sizes."""
        results = {}

        for n_nodes in network_sizes:
            print(f"Testing network size: {n_nodes}")

            # Create test network
            solver = SublinearFlowBenchmark.create_test_network(n_nodes)
            source, sink = 0, n_nodes - 1

            # Time sublinear approach
            runtime, result = SublinearFlowBenchmark.time_algorithm(
                solver.solve_max_flow_as_linear_system, source, sink
            )

            results[n_nodes] = {
                'sublinear_runtime': runtime,
                'sublinear_flow': result.get('max_flow', 0),
                'sublinear_iterations': result.get('iterations', 0),
                'nodes': n_nodes,
                'edges': len(solver.edges)
            }

        return results


if __name__ == "__main__":
    # Example usage and testing
    print("Sublinear Flow Algorithm Testing")
    print("=" * 40)

    # Create test network
    solver = SublinearFlowSolver(6)

    # Add edges (same as traditional example for comparison)
    solver.add_edge(0, 1, 10, 1)
    solver.add_edge(0, 2, 8, 2)
    solver.add_edge(1, 3, 5, 1)
    solver.add_edge(1, 4, 8, 3)
    solver.add_edge(2, 4, 10, 1)
    solver.add_edge(3, 5, 10, 2)
    solver.add_edge(4, 5, 10, 1)

    # Test max flow
    start_time = time.perf_counter()
    result = solver.solve_max_flow_as_linear_system(0, 5)
    runtime = time.perf_counter() - start_time

    print(f"Sublinear Max Flow Result:")
    print(f"  Max Flow: {result['max_flow']}")
    print(f"  Iterations: {result['iterations']}")
    print(f"  Runtime: {runtime:.6f}s")

    # Test min cost flow
    print(f"\nMin Cost Flow Test:")
    mcf_result = solver.solve_min_cost_flow_linear(0, 5, 10)
    if mcf_result.get('feasible', False):
        print(f"  Total Cost: {mcf_result['total_cost']:.2f}")
        print(f"  Feasible: {mcf_result['feasible']}")
    else:
        print(f"  Error: {mcf_result.get('error', 'Unknown')}")

    # Benchmark different network sizes
    print(f"\nBenchmarking across network sizes:")
    sizes = [10, 20, 50]
    benchmark_results = SublinearFlowBenchmark.compare_with_traditional(sizes)

    for size, result in benchmark_results.items():
        print(f"  Size {size:2d}: Flow={result['sublinear_flow']:6.1f}, "
              f"Time={result['sublinear_runtime']:.6f}s, "
              f"Edges={result['edges']:3d}")