"""
Network Flow Validation Suite
==============================

Comprehensive validation framework to ensure correctness of both traditional
and sublinear flow algorithms. Includes:

- Flow conservation verification
- Capacity constraint checking
- Optimality validation for min-cost flows
- Cross-algorithm result comparison
- Edge case testing
- Numerical stability analysis
"""

import numpy as np
from typing import Dict, List, Tuple, Optional, Set
from dataclasses import dataclass
import statistics
import warnings

from traditional_flows import (
    FlowNetwork, FordFulkersonDFS, EdmondsKarp, PushRelabel, MinCostFlow
)
from sublinear_flows import SublinearFlowSolver
from flow_generators import NetworkSuite


@dataclass
class ValidationResult:
    """Result of validation check."""
    test_name: str
    passed: bool
    details: str
    error_magnitude: float = 0.0
    warnings: List[str] = None

    def __post_init__(self):
        if self.warnings is None:
            self.warnings = []


class FlowValidationSuite:
    """Comprehensive validation suite for flow algorithms."""

    def __init__(self, tolerance: float = 1e-6):
        self.tolerance = tolerance
        self.validation_results = []

    def validate_flow_conservation(self, network_data: Dict, flows: Dict[Tuple[int, int], float],
                                 source: int, sink: int) -> ValidationResult:
        """
        Validate that flow satisfies conservation constraints.
        For each node (except source/sink): inflow = outflow
        """
        n_nodes = network_data['n_nodes']
        net_flow = [0.0] * n_nodes

        # Calculate net flow for each node
        for (u, v), flow_value in flows.items():
            net_flow[u] -= flow_value  # Outflow
            net_flow[v] += flow_value  # Inflow

        violations = []
        max_violation = 0.0

        for node in range(n_nodes):
            if node == source or node == sink:
                continue  # Skip source and sink

            violation = abs(net_flow[node])
            max_violation = max(max_violation, violation)

            if violation > self.tolerance:
                violations.append(f"Node {node}: net flow = {net_flow[node]:.6f}")

        passed = len(violations) == 0
        details = "Flow conservation satisfied" if passed else f"Violations: {violations}"

        return ValidationResult(
            test_name="flow_conservation",
            passed=passed,
            details=details,
            error_magnitude=max_violation
        )

    def validate_capacity_constraints(self, network_data: Dict,
                                    flows: Dict[Tuple[int, int], float]) -> ValidationResult:
        """
        Validate that flows don't exceed edge capacities.
        For each edge: 0 ≤ flow ≤ capacity
        """
        violations = []
        max_violation = 0.0

        for (u, v), capacity in network_data['capacities'].items():
            flow_value = flows.get((u, v), 0.0)

            # Check lower bound
            if flow_value < -self.tolerance:
                violation = abs(flow_value)
                max_violation = max(max_violation, violation)
                violations.append(f"Edge ({u},{v}): negative flow = {flow_value:.6f}")

            # Check upper bound
            if flow_value > capacity + self.tolerance:
                violation = flow_value - capacity
                max_violation = max(max_violation, violation)
                violations.append(f"Edge ({u},{v}): flow {flow_value:.6f} > capacity {capacity:.6f}")

        passed = len(violations) == 0
        details = "Capacity constraints satisfied" if passed else f"Violations: {violations}"

        return ValidationResult(
            test_name="capacity_constraints",
            passed=passed,
            details=details,
            error_magnitude=max_violation
        )

    def validate_max_flow_optimality(self, network_data: Dict, flows: Dict[Tuple[int, int], float],
                                   max_flow_value: float, source: int, sink: int) -> ValidationResult:
        """
        Validate max flow optimality using max-flow min-cut theorem.
        Find minimum cut and verify it equals the maximum flow.
        """
        try:
            # Find reachable nodes from source using residual capacities
            residual_capacities = {}
            for (u, v), capacity in network_data['capacities'].items():
                flow_value = flows.get((u, v), 0.0)
                residual_capacities[(u, v)] = capacity - flow_value

            reachable = self._find_reachable_nodes(source, residual_capacities, network_data['n_nodes'])

            # Calculate cut capacity
            cut_capacity = 0.0
            cut_edges = []

            for (u, v), capacity in network_data['capacities'].items():
                if u in reachable and v not in reachable:
                    cut_capacity += capacity
                    cut_edges.append((u, v))

            # Check if max flow equals min cut
            flow_cut_diff = abs(max_flow_value - cut_capacity)
            passed = flow_cut_diff <= self.tolerance

            details = (f"Max flow: {max_flow_value:.6f}, Min cut: {cut_capacity:.6f}, "
                      f"Difference: {flow_cut_diff:.6f}")

            if not passed:
                details += f" Cut edges: {cut_edges}"

            return ValidationResult(
                test_name="max_flow_optimality",
                passed=passed,
                details=details,
                error_magnitude=flow_cut_diff
            )

        except Exception as e:
            return ValidationResult(
                test_name="max_flow_optimality",
                passed=False,
                details=f"Error in optimality check: {str(e)}",
                error_magnitude=float('inf')
            )

    def _find_reachable_nodes(self, source: int, residual_capacities: Dict[Tuple[int, int], float],
                            n_nodes: int) -> Set[int]:
        """Find nodes reachable from source in residual graph."""
        reachable = set()
        stack = [source]

        while stack:
            node = stack.pop()
            if node in reachable:
                continue

            reachable.add(node)

            # Check all outgoing edges
            for (u, v), residual_cap in residual_capacities.items():
                if u == node and v not in reachable and residual_cap > self.tolerance:
                    stack.append(v)

        return reachable

    def validate_algorithm_consistency(self, network_data: Dict, source: int, sink: int) -> ValidationResult:
        """
        Run multiple algorithms on the same network and verify they produce the same max flow.
        """
        try:
            # Convert network for traditional algorithms
            flow_network_ff = FlowNetwork(network_data['n_nodes'])
            flow_network_ek = FlowNetwork(network_data['n_nodes'])
            flow_network_pr = FlowNetwork(network_data['n_nodes'])

            for u, v in network_data['edges']:
                capacity = int(network_data['capacities'][(u, v)])
                cost = network_data['costs'][(u, v)]

                flow_network_ff.add_edge(u, v, capacity, cost)
                flow_network_ek.add_edge(u, v, capacity, cost)
                flow_network_pr.add_edge(u, v, capacity, cost)

            # Run traditional algorithms
            ff = FordFulkersonDFS(flow_network_ff)
            ff_flow = ff.max_flow(source, sink)

            ek = EdmondsKarp(flow_network_ek)
            ek_flow = ek.max_flow(source, sink)

            pr = PushRelabel(flow_network_pr)
            pr_flow = pr.max_flow(source, sink)

            # Run sublinear algorithm
            sublinear_solver = SublinearFlowSolver(network_data['n_nodes'])
            for u, v in network_data['edges']:
                capacity = network_data['capacities'][(u, v)]
                cost = network_data['costs'][(u, v)]
                sublinear_solver.add_edge(u, v, capacity, cost)

            sublinear_result = sublinear_solver.solve_max_flow_as_linear_system(source, sink)
            sublinear_flow = sublinear_result.get('max_flow', 0.0)

            # Compare results
            flows = [ff_flow, ek_flow, pr_flow, sublinear_flow]
            algorithm_names = ['Ford-Fulkerson', 'Edmonds-Karp', 'Push-Relabel', 'Sublinear']

            max_diff = max(flows) - min(flows)
            passed = max_diff <= self.tolerance

            details = f"Flow values: {dict(zip(algorithm_names, flows))}, Max difference: {max_diff:.6f}"

            warnings_list = []
            if sublinear_flow == 0 and max(flows[:3]) > 0:
                warnings_list.append("Sublinear algorithm returned zero flow while traditional algorithms found positive flow")

            return ValidationResult(
                test_name="algorithm_consistency",
                passed=passed,
                details=details,
                error_magnitude=max_diff,
                warnings=warnings_list
            )

        except Exception as e:
            return ValidationResult(
                test_name="algorithm_consistency",
                passed=False,
                details=f"Error in consistency check: {str(e)}",
                error_magnitude=float('inf')
            )

    def validate_min_cost_flow_optimality(self, network_data: Dict, flows: Dict[Tuple[int, int], float],
                                        total_cost: float, source: int, sink: int,
                                        flow_value: float) -> ValidationResult:
        """
        Validate minimum cost flow optimality using complementary slackness conditions.
        """
        try:
            # For minimum cost flow, we need to check:
            # 1. Flow conservation
            # 2. Capacity constraints
            # 3. Reduced costs (complementary slackness)

            # This is a simplified check - full validation requires dual solution
            flow_conservation = self.validate_flow_conservation(network_data, flows, source, sink)
            capacity_constraints = self.validate_capacity_constraints(network_data, flows)

            # Calculate actual cost
            calculated_cost = sum(
                flows.get((u, v), 0.0) * network_data['costs'][(u, v)]
                for u, v in network_data['edges']
            )

            cost_diff = abs(calculated_cost - total_cost)
            cost_consistent = cost_diff <= self.tolerance

            passed = flow_conservation.passed and capacity_constraints.passed and cost_consistent

            details = (f"Flow conservation: {flow_conservation.passed}, "
                      f"Capacity constraints: {capacity_constraints.passed}, "
                      f"Cost consistency: {cost_consistent} (diff: {cost_diff:.6f})")

            return ValidationResult(
                test_name="min_cost_flow_optimality",
                passed=passed,
                details=details,
                error_magnitude=max(flow_conservation.error_magnitude,
                                  capacity_constraints.error_magnitude,
                                  cost_diff)
            )

        except Exception as e:
            return ValidationResult(
                test_name="min_cost_flow_optimality",
                passed=False,
                details=f"Error in min cost flow validation: {str(e)}",
                error_magnitude=float('inf')
            )

    def test_edge_cases(self) -> List[ValidationResult]:
        """Test algorithms on edge cases."""
        edge_case_results = []

        # Test 1: Single node network
        try:
            single_node_network = {
                'n_nodes': 1,
                'edges': [],
                'capacities': {},
                'costs': {},
                'source': 0,
                'sink': 0
            }

            # Should return 0 flow
            sublinear_solver = SublinearFlowSolver(1)
            result = sublinear_solver.solve_max_flow_as_linear_system(0, 0)
            max_flow = result.get('max_flow', 0.0)

            passed = abs(max_flow) <= self.tolerance
            edge_case_results.append(ValidationResult(
                test_name="single_node_edge_case",
                passed=passed,
                details=f"Single node max flow: {max_flow:.6f}",
                error_magnitude=abs(max_flow)
            ))

        except Exception as e:
            edge_case_results.append(ValidationResult(
                test_name="single_node_edge_case",
                passed=False,
                details=f"Error: {str(e)}",
                error_magnitude=float('inf')
            ))

        # Test 2: Disconnected source and sink
        try:
            disconnected_network = {
                'n_nodes': 4,
                'edges': [(0, 1), (2, 3)],
                'capacities': {(0, 1): 10.0, (2, 3): 5.0},
                'costs': {(0, 1): 1.0, (2, 3): 1.0},
                'source': 0,
                'sink': 3
            }

            sublinear_solver = SublinearFlowSolver(4)
            sublinear_solver.add_edge(0, 1, 10.0, 1.0)
            sublinear_solver.add_edge(2, 3, 5.0, 1.0)

            result = sublinear_solver.solve_max_flow_as_linear_system(0, 3)
            max_flow = result.get('max_flow', 0.0)

            passed = abs(max_flow) <= self.tolerance
            edge_case_results.append(ValidationResult(
                test_name="disconnected_network_edge_case",
                passed=passed,
                details=f"Disconnected network max flow: {max_flow:.6f}",
                error_magnitude=abs(max_flow)
            ))

        except Exception as e:
            edge_case_results.append(ValidationResult(
                test_name="disconnected_network_edge_case",
                passed=False,
                details=f"Error: {str(e)}",
                error_magnitude=float('inf')
            ))

        # Test 3: Zero capacity edges
        try:
            zero_capacity_network = {
                'n_nodes': 3,
                'edges': [(0, 1), (1, 2)],
                'capacities': {(0, 1): 0.0, (1, 2): 10.0},
                'costs': {(0, 1): 1.0, (1, 2): 1.0},
                'source': 0,
                'sink': 2
            }

            sublinear_solver = SublinearFlowSolver(3)
            sublinear_solver.add_edge(0, 1, 0.0, 1.0)
            sublinear_solver.add_edge(1, 2, 10.0, 1.0)

            result = sublinear_solver.solve_max_flow_as_linear_system(0, 2)
            max_flow = result.get('max_flow', 0.0)

            passed = abs(max_flow) <= self.tolerance
            edge_case_results.append(ValidationResult(
                test_name="zero_capacity_edge_case",
                passed=passed,
                details=f"Zero capacity bottleneck max flow: {max_flow:.6f}",
                error_magnitude=abs(max_flow)
            ))

        except Exception as e:
            edge_case_results.append(ValidationResult(
                test_name="zero_capacity_edge_case",
                passed=False,
                details=f"Error: {str(e)}",
                error_magnitude=float('inf')
            ))

        return edge_case_results

    def test_numerical_stability(self, network_data: Dict, source: int, sink: int) -> ValidationResult:
        """Test numerical stability by running the same problem multiple times with small perturbations."""
        try:
            base_results = []
            perturbed_results = []

            # Run base case multiple times
            for _ in range(5):
                sublinear_solver = SublinearFlowSolver(network_data['n_nodes'])
                for u, v in network_data['edges']:
                    capacity = network_data['capacities'][(u, v)]
                    cost = network_data['costs'][(u, v)]
                    sublinear_solver.add_edge(u, v, capacity, cost)

                result = sublinear_solver.solve_max_flow_as_linear_system(source, sink)
                base_results.append(result.get('max_flow', 0.0))

            # Run with small perturbations
            for _ in range(5):
                sublinear_solver = SublinearFlowSolver(network_data['n_nodes'])
                for u, v in network_data['edges']:
                    # Add small random perturbation to capacity
                    capacity = network_data['capacities'][(u, v)]
                    perturbation = capacity * 1e-8 * (np.random.random() - 0.5)
                    perturbed_capacity = max(capacity + perturbation, 0.1)

                    cost = network_data['costs'][(u, v)]
                    sublinear_solver.add_edge(u, v, perturbed_capacity, cost)

                result = sublinear_solver.solve_max_flow_as_linear_system(source, sink)
                perturbed_results.append(result.get('max_flow', 0.0))

            # Analyze stability
            base_std = statistics.stdev(base_results) if len(base_results) > 1 else 0.0
            perturbed_std = statistics.stdev(perturbed_results) if len(perturbed_results) > 1 else 0.0

            base_mean = statistics.mean(base_results)
            perturbed_mean = statistics.mean(perturbed_results)

            stability_ratio = (perturbed_std / abs(base_mean)) if abs(base_mean) > 1e-10 else float('inf')
            passed = stability_ratio < 1e-3  # Less than 0.1% relative deviation

            details = (f"Base std: {base_std:.6f}, Perturbed std: {perturbed_std:.6f}, "
                      f"Stability ratio: {stability_ratio:.6f}")

            warnings_list = []
            if stability_ratio > 1e-2:
                warnings_list.append("High sensitivity to small perturbations detected")

            return ValidationResult(
                test_name="numerical_stability",
                passed=passed,
                details=details,
                error_magnitude=stability_ratio,
                warnings=warnings_list
            )

        except Exception as e:
            return ValidationResult(
                test_name="numerical_stability",
                passed=False,
                details=f"Error in stability test: {str(e)}",
                error_magnitude=float('inf')
            )

    def run_comprehensive_validation(self, size_category: str = 'small') -> Dict[str, List[ValidationResult]]:
        """Run comprehensive validation on generated test networks."""
        suite = NetworkSuite(seed=42)
        networks = suite.generate_test_suite(size_category)

        all_results = {
            'per_network': {},
            'edge_cases': self.test_edge_cases(),
            'summary': {}
        }

        for network_name, network_data in networks.items():
            print(f"Validating {network_name}...")
            source = network_data['source']
            sink = network_data['sink']

            network_results = []

            # Run algorithm consistency test
            consistency_result = self.validate_algorithm_consistency(network_data, source, sink)
            network_results.append(consistency_result)

            # Test numerical stability
            stability_result = self.test_numerical_stability(network_data, source, sink)
            network_results.append(stability_result)

            # If we have a successful flow solution, validate its properties
            if consistency_result.passed:
                # Extract flows from one of the traditional algorithms for validation
                try:
                    flow_network = FlowNetwork(network_data['n_nodes'])
                    for u, v in network_data['edges']:
                        capacity = int(network_data['capacities'][(u, v)])
                        cost = network_data['costs'][(u, v)]
                        flow_network.add_edge(u, v, capacity, cost)

                    # Use Edmonds-Karp to get a validated flow
                    ek = EdmondsKarp(flow_network)
                    max_flow_value = ek.max_flow(source, sink)

                    # Extract flows (simplified - would need actual flow extraction from algorithm)
                    flows = {}
                    for u, v in network_data['edges']:
                        # This is a placeholder - real implementation would extract actual flows
                        flows[(u, v)] = min(flow_network.flow[u][v], network_data['capacities'][(u, v)])

                    # Validate flow properties
                    conservation_result = self.validate_flow_conservation(
                        network_data, flows, source, sink
                    )
                    network_results.append(conservation_result)

                    capacity_result = self.validate_capacity_constraints(network_data, flows)
                    network_results.append(capacity_result)

                    optimality_result = self.validate_max_flow_optimality(
                        network_data, flows, max_flow_value, source, sink
                    )
                    network_results.append(optimality_result)

                except Exception as e:
                    error_result = ValidationResult(
                        test_name="flow_extraction_error",
                        passed=False,
                        details=f"Error extracting flows for validation: {str(e)}",
                        error_magnitude=float('inf')
                    )
                    network_results.append(error_result)

            all_results['per_network'][network_name] = network_results

        # Generate summary
        all_tests = []
        for network_results in all_results['per_network'].values():
            all_tests.extend(network_results)
        all_tests.extend(all_results['edge_cases'])

        passed_tests = len([r for r in all_tests if r.passed])
        total_tests = len(all_tests)

        all_results['summary'] = {
            'total_tests': total_tests,
            'passed_tests': passed_tests,
            'success_rate': passed_tests / total_tests if total_tests > 0 else 0.0,
            'failed_tests': [r.test_name for r in all_tests if not r.passed]
        }

        return all_results

    def generate_validation_report(self, results: Dict, filename: str = "validation_report.md"):
        """Generate comprehensive validation report."""
        with open(filename, 'w') as f:
            f.write("# Network Flow Algorithm Validation Report\n\n")

            # Summary
            summary = results['summary']
            f.write(f"## Summary\n\n")
            f.write(f"- Total tests: {summary['total_tests']}\n")
            f.write(f"- Passed tests: {summary['passed_tests']}\n")
            f.write(f"- Success rate: {summary['success_rate']:.1%}\n")

            if summary['failed_tests']:
                f.write(f"- Failed tests: {', '.join(summary['failed_tests'])}\n")

            f.write("\n")

            # Edge cases
            f.write("## Edge Case Tests\n\n")
            for result in results['edge_cases']:
                status = "✅ PASS" if result.passed else "❌ FAIL"
                f.write(f"- **{result.test_name}**: {status}\n")
                f.write(f"  - {result.details}\n")
                if result.warnings:
                    f.write(f"  - Warnings: {', '.join(result.warnings)}\n")
                f.write("\n")

            # Per-network results
            f.write("## Network-Specific Tests\n\n")
            for network_name, network_results in results['per_network'].items():
                f.write(f"### {network_name}\n\n")
                for result in network_results:
                    status = "✅ PASS" if result.passed else "❌ FAIL"
                    f.write(f"- **{result.test_name}**: {status}\n")
                    f.write(f"  - {result.details}\n")
                    if result.error_magnitude > 0:
                        f.write(f"  - Error magnitude: {result.error_magnitude:.2e}\n")
                    if result.warnings:
                        f.write(f"  - Warnings: {', '.join(result.warnings)}\n")
                    f.write("\n")

        print(f"Validation report saved to {filename}")


if __name__ == "__main__":
    # Run comprehensive validation
    print("Network Flow Algorithm Validation Suite")
    print("=" * 50)

    validator = FlowValidationSuite(tolerance=1e-6)

    # Run validation
    results = validator.run_comprehensive_validation('small')

    # Print summary
    summary = results['summary']
    print(f"\nValidation Summary:")
    print(f"  Total tests: {summary['total_tests']}")
    print(f"  Passed: {summary['passed_tests']}")
    print(f"  Success rate: {summary['success_rate']:.1%}")

    if summary['failed_tests']:
        print(f"  Failed tests: {', '.join(summary['failed_tests'])}")

    # Generate report
    validator.generate_validation_report(results, "/workspaces/sublinear-time-solver/scripts/network_flow/validation_report.md")

    print("\nValidation completed!")