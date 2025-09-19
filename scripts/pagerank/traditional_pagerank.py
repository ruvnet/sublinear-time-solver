#!/usr/bin/env python3
"""
Traditional PageRank implementations using NetworkX and SciPy.
Provides baseline comparison for sublinear solver validation.
"""

import time
import numpy as np
import networkx as nx
from scipy.sparse import csr_matrix
from scipy.sparse.linalg import eigs
import psutil
import os
from typing import Dict, List, Tuple, Optional
import tracemalloc


class TraditionalPageRank:
    """Traditional PageRank implementations for comparison."""

    def __init__(self, damping: float = 0.85, max_iter: int = 1000, tol: float = 1e-6):
        self.damping = damping
        self.max_iter = max_iter
        self.tol = tol

    def networkx_pagerank(self, graph: nx.Graph) -> Tuple[Dict, Dict]:
        """Compute PageRank using NetworkX built-in function."""
        tracemalloc.start()
        start_time = time.perf_counter()
        start_memory = psutil.Process(os.getpid()).memory_info().rss / 1024 / 1024

        # Convert to directed graph if needed
        if not graph.is_directed():
            graph = graph.to_directed()

        pagerank_scores = nx.pagerank(
            graph,
            alpha=self.damping,
            max_iter=self.max_iter,
            tol=self.tol
        )

        end_time = time.perf_counter()
        end_memory = psutil.Process(os.getpid()).memory_info().rss / 1024 / 1024
        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()

        metrics = {
            'execution_time': end_time - start_time,
            'memory_usage_mb': end_memory - start_memory,
            'peak_memory_mb': peak / 1024 / 1024,
            'convergence_iterations': self.max_iter,  # NetworkX doesn't expose this
            'method': 'NetworkX'
        }

        return pagerank_scores, metrics

    def scipy_pagerank(self, adjacency_matrix: np.ndarray) -> Tuple[np.ndarray, Dict]:
        """Compute PageRank using SciPy eigenvalue approach."""
        tracemalloc.start()
        start_time = time.perf_counter()
        start_memory = psutil.Process(os.getpid()).memory_info().rss / 1024 / 1024

        n = adjacency_matrix.shape[0]

        # Create transition matrix
        out_degrees = np.sum(adjacency_matrix, axis=1)
        # Handle dangling nodes (nodes with no outgoing edges)
        out_degrees[out_degrees == 0] = 1

        # Normalize to create stochastic matrix
        transition_matrix = adjacency_matrix / out_degrees[:, np.newaxis]

        # Create Google matrix: G = α*P + (1-α)/n * ones
        google_matrix = (self.damping * transition_matrix +
                        (1 - self.damping) / n * np.ones((n, n)))

        # Find dominant eigenvector
        eigenvalues, eigenvectors = eigs(google_matrix.T, k=1, which='LM')
        pagerank_vector = np.real(eigenvectors[:, 0])

        # Normalize to sum to 1
        pagerank_vector = np.abs(pagerank_vector)
        pagerank_vector = pagerank_vector / np.sum(pagerank_vector)

        end_time = time.perf_counter()
        end_memory = psutil.Process(os.getpid()).memory_info().rss / 1024 / 1024
        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()

        metrics = {
            'execution_time': end_time - start_time,
            'memory_usage_mb': end_memory - start_memory,
            'peak_memory_mb': peak / 1024 / 1024,
            'convergence_iterations': 'N/A (eigenvalue)',
            'method': 'SciPy Eigenvalue'
        }

        return pagerank_vector, metrics

    def power_iteration_pagerank(self, adjacency_matrix: np.ndarray) -> Tuple[np.ndarray, Dict]:
        """Compute PageRank using power iteration method."""
        tracemalloc.start()
        start_time = time.perf_counter()
        start_memory = psutil.Process(os.getpid()).memory_info().rss / 1024 / 1024

        n = adjacency_matrix.shape[0]

        # Create transition matrix
        out_degrees = np.sum(adjacency_matrix, axis=1)
        # Handle dangling nodes
        dangling_nodes = out_degrees == 0
        out_degrees[dangling_nodes] = 1

        # Normalize
        transition_matrix = adjacency_matrix / out_degrees[:, np.newaxis]

        # Initialize PageRank vector
        pagerank_vector = np.ones(n) / n

        iterations = 0
        for i in range(self.max_iter):
            old_pagerank = pagerank_vector.copy()

            # Power iteration step
            pagerank_vector = (self.damping * transition_matrix.T @ pagerank_vector +
                             (1 - self.damping) / n * np.ones(n))

            # Handle dangling nodes
            dangling_sum = np.sum(old_pagerank[dangling_nodes])
            pagerank_vector += self.damping * dangling_sum / n * np.ones(n)

            # Check convergence
            if np.linalg.norm(pagerank_vector - old_pagerank, 1) < self.tol:
                iterations = i + 1
                break
        else:
            iterations = self.max_iter

        end_time = time.perf_counter()
        end_memory = psutil.Process(os.getpid()).memory_info().rss / 1024 / 1024
        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()

        metrics = {
            'execution_time': end_time - start_time,
            'memory_usage_mb': end_memory - start_memory,
            'peak_memory_mb': peak / 1024 / 1024,
            'convergence_iterations': iterations,
            'method': 'Power Iteration'
        }

        return pagerank_vector, metrics

    def sparse_pagerank(self, adjacency_matrix: np.ndarray) -> Tuple[np.ndarray, Dict]:
        """Compute PageRank using sparse matrix operations."""
        tracemalloc.start()
        start_time = time.perf_counter()
        start_memory = psutil.Process(os.getpid()).memory_info().rss / 1024 / 1024

        n = adjacency_matrix.shape[0]

        # Convert to sparse matrix
        sparse_adj = csr_matrix(adjacency_matrix)

        # Create transition matrix
        out_degrees = np.array(sparse_adj.sum(axis=1)).flatten()
        dangling_nodes = out_degrees == 0
        out_degrees[dangling_nodes] = 1

        # Create sparse transition matrix
        row_sums = csr_matrix((1.0 / out_degrees, (range(n), range(n))), shape=(n, n))
        transition_matrix = sparse_adj @ row_sums

        # Initialize PageRank vector
        pagerank_vector = np.ones(n) / n

        iterations = 0
        for i in range(self.max_iter):
            old_pagerank = pagerank_vector.copy()

            # Sparse power iteration
            pagerank_vector = (self.damping * transition_matrix.T @ pagerank_vector +
                             (1 - self.damping) / n * np.ones(n))

            # Handle dangling nodes
            dangling_sum = np.sum(old_pagerank[dangling_nodes])
            pagerank_vector += self.damping * dangling_sum / n * np.ones(n)

            # Check convergence
            if np.linalg.norm(pagerank_vector - old_pagerank, 1) < self.tol:
                iterations = i + 1
                break
        else:
            iterations = self.max_iter

        end_time = time.perf_counter()
        end_memory = psutil.Process(os.getpid()).memory_info().rss / 1024 / 1024
        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()

        metrics = {
            'execution_time': end_time - start_time,
            'memory_usage_mb': end_memory - start_memory,
            'peak_memory_mb': peak / 1024 / 1024,
            'convergence_iterations': iterations,
            'method': 'Sparse Matrix'
        }

        return pagerank_vector, metrics


def compare_pagerank_methods(adjacency_matrix: np.ndarray, graph: Optional[nx.Graph] = None) -> Dict:
    """Compare all traditional PageRank methods on the same graph."""
    pagerank = TraditionalPageRank()
    results = {}

    # NetworkX method (requires NetworkX graph)
    if graph is not None:
        try:
            nx_scores, nx_metrics = pagerank.networkx_pagerank(graph)
            results['networkx'] = {
                'scores': nx_scores,
                'metrics': nx_metrics
            }
        except Exception as e:
            results['networkx'] = {'error': str(e)}

    # SciPy eigenvalue method
    try:
        scipy_scores, scipy_metrics = pagerank.scipy_pagerank(adjacency_matrix)
        results['scipy_eigenvalue'] = {
            'scores': scipy_scores,
            'metrics': scipy_metrics
        }
    except Exception as e:
        results['scipy_eigenvalue'] = {'error': str(e)}

    # Power iteration method
    try:
        power_scores, power_metrics = pagerank.power_iteration_pagerank(adjacency_matrix)
        results['power_iteration'] = {
            'scores': power_scores,
            'metrics': power_metrics
        }
    except Exception as e:
        results['power_iteration'] = {'error': str(e)}

    # Sparse matrix method
    try:
        sparse_scores, sparse_metrics = pagerank.sparse_pagerank(adjacency_matrix)
        results['sparse_matrix'] = {
            'scores': sparse_scores,
            'metrics': sparse_metrics
        }
    except Exception as e:
        results['sparse_matrix'] = {'error': str(e)}

    return results


if __name__ == "__main__":
    # Simple test case
    test_adj = np.array([
        [0, 1, 1],
        [1, 0, 1],
        [1, 1, 0]
    ])

    test_graph = nx.from_numpy_array(test_adj)
    results = compare_pagerank_methods(test_adj, test_graph)

    print("Traditional PageRank Comparison Results:")
    print("=" * 50)

    for method, result in results.items():
        if 'error' in result:
            print(f"\n{method}: ERROR - {result['error']}")
        else:
            print(f"\n{method}:")
            print(f"  Execution time: {result['metrics']['execution_time']:.6f}s")
            print(f"  Memory usage: {result['metrics']['memory_usage_mb']:.2f} MB")
            print(f"  Peak memory: {result['metrics']['peak_memory_mb']:.2f} MB")
            print(f"  Iterations: {result['metrics']['convergence_iterations']}")
            if isinstance(result['scores'], dict):
                print(f"  Top nodes: {sorted(result['scores'].items(), key=lambda x: x[1], reverse=True)[:3]}")
            else:
                top_indices = np.argsort(result['scores'])[-3:][::-1]
                print(f"  Top nodes: {[(i, result['scores'][i]) for i in top_indices]}")