"""
Traditional Network Flow Algorithms
=====================================

Implementation of classical combinatorial algorithms for network flow problems:
- Ford-Fulkerson with DFS
- Edmonds-Karp (Ford-Fulkerson with BFS)
- Push-Relabel (FIFO variant)
- Minimum Cost Flow (Successive Shortest Path)

These serve as baseline comparisons for sublinear approaches.
"""

import numpy as np
from collections import defaultdict, deque
import heapq
from typing import Dict, List, Tuple, Optional, Set
import time


class FlowNetwork:
    """Represents a flow network with capacity constraints."""

    def __init__(self, n_nodes: int):
        self.n_nodes = n_nodes
        self.capacity = defaultdict(lambda: defaultdict(int))
        self.flow = defaultdict(lambda: defaultdict(int))
        self.cost = defaultdict(lambda: defaultdict(float))
        self.adj = defaultdict(list)

    def add_edge(self, u: int, v: int, cap: int, cost: float = 0):
        """Add edge with capacity and optional cost."""
        self.capacity[u][v] += cap
        self.cost[u][v] = cost
        if v not in self.adj[u]:
            self.adj[u].append(v)
        if u not in self.adj[v]:
            self.adj[v].append(u)

    def residual_capacity(self, u: int, v: int) -> int:
        """Get residual capacity of edge u->v."""
        return self.capacity[u][v] - self.flow[u][v]


class FordFulkersonDFS:
    """Ford-Fulkerson algorithm using DFS to find augmenting paths."""

    def __init__(self, network: FlowNetwork):
        self.network = network
        self.visited = set()

    def dfs_path(self, source: int, sink: int, min_flow: int = float('inf')) -> int:
        """Find augmenting path using DFS."""
        if source == sink:
            return min_flow

        self.visited.add(source)

        for neighbor in self.network.adj[source]:
            residual = self.network.residual_capacity(source, neighbor)
            if neighbor not in self.visited and residual > 0:
                flow = self.dfs_path(neighbor, sink, min(min_flow, residual))
                if flow > 0:
                    # Update flow
                    self.network.flow[source][neighbor] += flow
                    self.network.flow[neighbor][source] -= flow
                    return flow

        return 0

    def max_flow(self, source: int, sink: int) -> int:
        """Compute maximum flow from source to sink."""
        total_flow = 0

        while True:
            self.visited.clear()
            path_flow = self.dfs_path(source, sink)
            if path_flow == 0:
                break
            total_flow += path_flow

        return total_flow


class EdmondsKarp:
    """Edmonds-Karp algorithm (Ford-Fulkerson with BFS)."""

    def __init__(self, network: FlowNetwork):
        self.network = network

    def bfs_path(self, source: int, sink: int) -> Tuple[int, List[int]]:
        """Find shortest augmenting path using BFS."""
        parent = [-1] * self.network.n_nodes
        visited = [False] * self.network.n_nodes
        queue = deque([source])
        visited[source] = True

        while queue:
            u = queue.popleft()

            for v in self.network.adj[u]:
                if not visited[v] and self.network.residual_capacity(u, v) > 0:
                    visited[v] = True
                    parent[v] = u
                    queue.append(v)
                    if v == sink:
                        # Reconstruct path and find bottleneck
                        path = []
                        min_flow = float('inf')
                        curr = sink

                        while curr != source:
                            prev = parent[curr]
                            path.append((prev, curr))
                            min_flow = min(min_flow, self.network.residual_capacity(prev, curr))
                            curr = prev

                        return min_flow, path[::-1]

        return 0, []

    def max_flow(self, source: int, sink: int) -> int:
        """Compute maximum flow using Edmonds-Karp."""
        total_flow = 0

        while True:
            path_flow, path = self.bfs_path(source, sink)
            if path_flow == 0:
                break

            # Update flows along the path
            for u, v in path:
                self.network.flow[u][v] += path_flow
                self.network.flow[v][u] -= path_flow

            total_flow += path_flow

        return total_flow


class PushRelabel:
    """Push-Relabel algorithm with FIFO selection rule."""

    def __init__(self, network: FlowNetwork):
        self.network = network
        self.n = network.n_nodes
        self.excess = [0] * self.n
        self.height = [0] * self.n
        self.active = set()
        self.queue = deque()

    def initialize_preflow(self, source: int):
        """Initialize preflow from source."""
        self.height[source] = self.n

        for v in self.network.adj[source]:
            capacity = self.network.capacity[source][v]
            if capacity > 0:
                self.network.flow[source][v] = capacity
                self.network.flow[v][source] = -capacity
                self.excess[v] = capacity
                self.excess[source] -= capacity

                if v != source and v not in self.active:
                    self.active.add(v)
                    self.queue.append(v)

    def push(self, u: int, v: int):
        """Push flow from u to v."""
        push_flow = min(self.excess[u], self.network.residual_capacity(u, v))

        self.network.flow[u][v] += push_flow
        self.network.flow[v][u] -= push_flow
        self.excess[u] -= push_flow
        self.excess[v] += push_flow

        if v not in self.active and self.excess[v] > 0:
            self.active.add(v)
            self.queue.append(v)

    def relabel(self, u: int):
        """Relabel node u to enable pushing."""
        min_height = float('inf')

        for v in self.network.adj[u]:
            if self.network.residual_capacity(u, v) > 0:
                min_height = min(min_height, self.height[v])

        if min_height < float('inf'):
            self.height[u] = min_height + 1

    def discharge(self, u: int):
        """Discharge excess flow from node u."""
        while self.excess[u] > 0:
            # Try to push to admissible neighbors
            pushed = False

            for v in self.network.adj[u]:
                if (self.network.residual_capacity(u, v) > 0 and
                    self.height[u] == self.height[v] + 1):
                    self.push(u, v)
                    pushed = True
                    if self.excess[u] == 0:
                        break

            if not pushed:
                self.relabel(u)

    def max_flow(self, source: int, sink: int) -> int:
        """Compute maximum flow using Push-Relabel."""
        self.initialize_preflow(source)

        while self.queue:
            u = self.queue.popleft()
            if u in self.active and u != source and u != sink:
                self.discharge(u)
                if self.excess[u] == 0:
                    self.active.remove(u)

        return -self.excess[source]


class MinCostFlow:
    """Minimum Cost Flow using Successive Shortest Path algorithm."""

    def __init__(self, network: FlowNetwork):
        self.network = network
        self.n = network.n_nodes

    def shortest_path_bellman_ford(self, source: int, sink: int) -> Tuple[float, List[int]]:
        """Find shortest path in residual graph with costs."""
        dist = [float('inf')] * self.n
        parent = [-1] * self.n
        dist[source] = 0

        # Bellman-Ford algorithm
        for _ in range(self.n - 1):
            for u in range(self.n):
                if dist[u] == float('inf'):
                    continue

                for v in self.network.adj[u]:
                    if self.network.residual_capacity(u, v) > 0:
                        edge_cost = self.network.cost[u][v]
                        if dist[u] + edge_cost < dist[v]:
                            dist[v] = dist[u] + edge_cost
                            parent[v] = u

        if dist[sink] == float('inf'):
            return float('inf'), []

        # Reconstruct path
        path = []
        curr = sink
        while curr != source:
            path.append(curr)
            curr = parent[curr]
        path.append(source)

        return dist[sink], path[::-1]

    def min_cost_max_flow(self, source: int, sink: int) -> Tuple[int, float]:
        """Compute minimum cost maximum flow."""
        total_flow = 0
        total_cost = 0.0

        while True:
            cost, path = self.shortest_path_bellman_ford(source, sink)
            if cost == float('inf'):
                break

            # Find bottleneck capacity along path
            flow_amount = float('inf')
            for i in range(len(path) - 1):
                u, v = path[i], path[i + 1]
                flow_amount = min(flow_amount, self.network.residual_capacity(u, v))

            if flow_amount == 0:
                break

            # Update flows and costs
            for i in range(len(path) - 1):
                u, v = path[i], path[i + 1]
                self.network.flow[u][v] += flow_amount
                self.network.flow[v][u] -= flow_amount
                total_cost += flow_amount * self.network.cost[u][v]

            total_flow += flow_amount

        return total_flow, total_cost


class FlowBenchmark:
    """Benchmark suite for comparing flow algorithms."""

    @staticmethod
    def time_algorithm(algorithm_func, *args) -> Tuple[float, any]:
        """Time an algorithm and return (runtime, result)."""
        start_time = time.perf_counter()
        result = algorithm_func(*args)
        end_time = time.perf_counter()
        return end_time - start_time, result

    @staticmethod
    def compare_algorithms(network: FlowNetwork, source: int, sink: int) -> Dict:
        """Compare all max flow algorithms on the same network."""

        # Create fresh copies for each algorithm
        networks = {
            'ford_fulkerson': FlowNetwork(network.n_nodes),
            'edmonds_karp': FlowNetwork(network.n_nodes),
            'push_relabel': FlowNetwork(network.n_nodes)
        }

        # Copy network structure
        for name, net in networks.items():
            for u in range(network.n_nodes):
                for v in network.adj[u]:
                    if network.capacity[u][v] > 0:
                        net.add_edge(u, v, network.capacity[u][v], network.cost[u][v])

        results = {}

        # Ford-Fulkerson DFS
        ff = FordFulkersonDFS(networks['ford_fulkerson'])
        runtime, max_flow = FlowBenchmark.time_algorithm(ff.max_flow, source, sink)
        results['ford_fulkerson'] = {'runtime': runtime, 'max_flow': max_flow}

        # Edmonds-Karp
        ek = EdmondsKarp(networks['edmonds_karp'])
        runtime, max_flow = FlowBenchmark.time_algorithm(ek.max_flow, source, sink)
        results['edmonds_karp'] = {'runtime': runtime, 'max_flow': max_flow}

        # Push-Relabel
        pr = PushRelabel(networks['push_relabel'])
        runtime, max_flow = FlowBenchmark.time_algorithm(pr.max_flow, source, sink)
        results['push_relabel'] = {'runtime': runtime, 'max_flow': max_flow}

        return results


if __name__ == "__main__":
    # Example usage and testing
    network = FlowNetwork(6)

    # Create a simple flow network
    network.add_edge(0, 1, 10, 1)
    network.add_edge(0, 2, 8, 2)
    network.add_edge(1, 3, 5, 1)
    network.add_edge(1, 4, 8, 3)
    network.add_edge(2, 4, 10, 1)
    network.add_edge(3, 5, 10, 2)
    network.add_edge(4, 5, 10, 1)

    source, sink = 0, 5

    # Compare algorithms
    results = FlowBenchmark.compare_algorithms(network, source, sink)

    print("Traditional Flow Algorithm Comparison:")
    print("=" * 50)
    for alg_name, result in results.items():
        print(f"{alg_name:15}: Max Flow = {result['max_flow']:6}, "
              f"Runtime = {result['runtime']:.6f}s")

    # Test minimum cost flow
    mcf_network = FlowNetwork(4)
    mcf_network.add_edge(0, 1, 10, 2)
    mcf_network.add_edge(0, 2, 10, 1)
    mcf_network.add_edge(1, 3, 10, 1)
    mcf_network.add_edge(2, 3, 10, 2)

    mcf = MinCostFlow(mcf_network)
    flow, cost = mcf.min_cost_max_flow(0, 3)
    print(f"\nMin Cost Flow: Flow = {flow}, Cost = {cost}")