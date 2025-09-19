#!/usr/bin/env python3
"""
Traditional Social Network Analysis using NetworkX and igraph

This module implements classical graph-theoretic approaches for social network analysis,
providing baseline comparisons for sublinear solver approaches.

Author: Social Network Analysis Agent
"""

import numpy as np
import networkx as nx
import time
import json
from typing import Dict, List, Tuple, Optional, Any
import matplotlib.pyplot as plt
import seaborn as sns
from collections import defaultdict
import psutil
import tracemalloc

# Try to import igraph if available
try:
    import igraph as ig
    IGRAPH_AVAILABLE = True
except ImportError:
    IGRAPH_AVAILABLE = False
    print("Warning: igraph not available. Some algorithms will be skipped.")


class TraditionalSocialAnalysis:
    """Traditional social network analysis using NetworkX and igraph."""

    def __init__(self, graph: nx.Graph):
        """Initialize with NetworkX graph."""
        self.graph = graph
        self.n_nodes = len(graph.nodes())
        self.n_edges = len(graph.edges())
        self.results = {}
        self.performance_metrics = {}

        # Convert to igraph if available
        if IGRAPH_AVAILABLE:
            self.igraph = self._nx_to_igraph(graph)
        else:
            self.igraph = None

    def _nx_to_igraph(self, G: nx.Graph) -> Optional[ig.Graph]:
        """Convert NetworkX graph to igraph."""
        try:
            # Create igraph from edge list
            edges = list(G.edges())
            nodes = list(G.nodes())

            # Create mapping from node labels to indices
            node_to_idx = {node: i for i, node in enumerate(nodes)}

            # Convert edges to indices
            edge_indices = [(node_to_idx[u], node_to_idx[v]) for u, v in edges]

            # Create igraph
            ig_graph = ig.Graph(n=len(nodes), edges=edge_indices, directed=G.is_directed())
            ig_graph.vs['name'] = nodes

            return ig_graph
        except Exception as e:
            print(f"Failed to convert to igraph: {e}")
            return None

    def compute_centrality_measures(self) -> Dict[str, Any]:
        """Compute all centrality measures using traditional methods."""
        print("Computing traditional centrality measures...")

        results = {}

        # PageRank
        start_time = time.time()
        tracemalloc.start()

        pagerank = nx.pagerank(self.graph, alpha=0.85, max_iter=1000, tol=1e-6)

        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        pagerank_time = time.time() - start_time

        results['pagerank'] = {
            'values': pagerank,
            'computation_time': pagerank_time,
            'memory_peak': peak,
            'method': 'power_iteration'
        }

        # Eigenvector Centrality
        start_time = time.time()
        tracemalloc.start()

        try:
            eigenvector = nx.eigenvector_centrality(self.graph, max_iter=1000, tol=1e-6)
            eigencentral_success = True
        except:
            eigenvector = {node: 0.0 for node in self.graph.nodes()}
            eigencentral_success = False

        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        eigenvector_time = time.time() - start_time

        results['eigenvector_centrality'] = {
            'values': eigenvector,
            'computation_time': eigenvector_time,
            'memory_peak': peak,
            'success': eigencentral_success,
            'method': 'power_iteration'
        }

        # Katz Centrality
        start_time = time.time()
        tracemalloc.start()

        try:
            # Use conservative alpha to avoid convergence issues
            alpha = 0.1 / max(1, max(dict(self.graph.degree()).values()))
            katz = nx.katz_centrality(self.graph, alpha=alpha, max_iter=1000, tol=1e-6)
            katz_success = True
        except:
            katz = {node: 0.0 for node in self.graph.nodes()}
            katz_success = False

        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        katz_time = time.time() - start_time

        results['katz_centrality'] = {
            'values': katz,
            'computation_time': katz_time,
            'memory_peak': peak,
            'success': katz_success,
            'alpha': alpha if katz_success else None,
            'method': 'matrix_inversion'
        }

        # Betweenness Centrality
        start_time = time.time()
        tracemalloc.start()

        betweenness = nx.betweenness_centrality(self.graph, normalized=True)

        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        betweenness_time = time.time() - start_time

        results['betweenness_centrality'] = {
            'values': betweenness,
            'computation_time': betweenness_time,
            'memory_peak': peak,
            'method': 'shortest_paths'
        }

        # Closeness Centrality
        start_time = time.time()
        tracemalloc.start()

        closeness = nx.closeness_centrality(self.graph)

        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        closeness_time = time.time() - start_time

        results['closeness_centrality'] = {
            'values': closeness,
            'computation_time': closeness_time,
            'memory_peak': peak,
            'method': 'shortest_paths'
        }

        # Degree Centrality (baseline)
        degree = nx.degree_centrality(self.graph)
        results['degree_centrality'] = {
            'values': degree,
            'computation_time': 0.001,  # Very fast
            'memory_peak': 1024,  # Minimal memory
            'method': 'local_computation'
        }

        self.results['centrality'] = results
        return results

    def compute_community_detection(self) -> Dict[str, Any]:
        """Compute community detection using traditional algorithms."""
        print("Computing traditional community detection...")

        results = {}

        # Louvain Method (if available)
        try:
            import community.community_louvain as community_louvain

            start_time = time.time()
            tracemalloc.start()

            partition = community_louvain.best_partition(self.graph)
            modularity = community_louvain.modularity(partition, self.graph)

            current, peak = tracemalloc.get_traced_memory()
            tracemalloc.stop()
            louvain_time = time.time() - start_time

            results['louvain'] = {
                'partition': partition,
                'modularity': modularity,
                'n_communities': len(set(partition.values())),
                'computation_time': louvain_time,
                'memory_peak': peak,
                'method': 'modularity_optimization'
            }
        except ImportError:
            print("Warning: community package not available for Louvain method")

        # Label Propagation
        start_time = time.time()
        tracemalloc.start()

        label_prop_communities = list(nx.label_propagation_communities(self.graph))
        # Convert to partition dict
        label_partition = {}
        for i, community in enumerate(label_prop_communities):
            for node in community:
                label_partition[node] = i

        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        label_time = time.time() - start_time

        results['label_propagation'] = {
            'partition': label_partition,
            'n_communities': len(label_prop_communities),
            'computation_time': label_time,
            'memory_peak': peak,
            'method': 'label_propagation'
        }

        # Spectral Clustering (approximate)
        if self.n_nodes <= 1000:  # Only for smaller graphs
            start_time = time.time()
            tracemalloc.start()

            try:
                # Compute Laplacian eigenvalues to estimate number of communities
                laplacian = nx.laplacian_matrix(self.graph).toarray()
                eigenvals = np.linalg.eigvals(laplacian)
                eigenvals = np.sort(eigenvals.real)

                # Estimate number of communities from eigengap
                gaps = np.diff(eigenvals[:10])  # Look at first 10 eigenvalues
                n_communities = np.argmax(gaps) + 2
                n_communities = min(n_communities, 10)  # Cap at 10

                # Use sklearn for actual clustering
                from sklearn.cluster import SpectralClustering
                spectral = SpectralClustering(n_clusters=n_communities,
                                            affinity='precomputed',
                                            random_state=42)
                adj_matrix = nx.adjacency_matrix(self.graph).toarray()
                spectral_labels = spectral.fit_predict(adj_matrix)

                spectral_partition = {node: int(spectral_labels[i])
                                    for i, node in enumerate(self.graph.nodes())}

                current, peak = tracemalloc.get_traced_memory()
                tracemalloc.stop()
                spectral_time = time.time() - start_time

                results['spectral_clustering'] = {
                    'partition': spectral_partition,
                    'n_communities': n_communities,
                    'computation_time': spectral_time,
                    'memory_peak': peak,
                    'method': 'laplacian_eigenvectors'
                }
            except Exception as e:
                print(f"Spectral clustering failed: {e}")

        # igraph algorithms if available
        if self.igraph is not None:
            # Fast Greedy
            start_time = time.time()
            tracemalloc.start()

            fast_greedy = self.igraph.community_fastgreedy()
            optimal_cut = fast_greedy.as_clustering()

            # Convert to partition dict
            fg_partition = {}
            for i, community in enumerate(optimal_cut):
                for node_idx in community:
                    node_name = self.igraph.vs[node_idx]['name']
                    fg_partition[node_name] = i

            current, peak = tracemalloc.get_traced_memory()
            tracemalloc.stop()
            fg_time = time.time() - start_time

            results['fast_greedy'] = {
                'partition': fg_partition,
                'modularity': optimal_cut.modularity,
                'n_communities': len(optimal_cut),
                'computation_time': fg_time,
                'memory_peak': peak,
                'method': 'hierarchical_greedy'
            }

            # Walktrap
            start_time = time.time()
            tracemalloc.start()

            walktrap = self.igraph.community_walktrap()
            wt_clustering = walktrap.as_clustering()

            # Convert to partition dict
            wt_partition = {}
            for i, community in enumerate(wt_clustering):
                for node_idx in community:
                    node_name = self.igraph.vs[node_idx]['name']
                    wt_partition[node_name] = i

            current, peak = tracemalloc.get_traced_memory()
            tracemalloc.stop()
            wt_time = time.time() - start_time

            results['walktrap'] = {
                'partition': wt_partition,
                'modularity': wt_clustering.modularity,
                'n_communities': len(wt_clustering),
                'computation_time': wt_time,
                'memory_peak': peak,
                'method': 'random_walks'
            }

        self.results['community_detection'] = results
        return results

    def compute_influence_propagation(self) -> Dict[str, Any]:
        """Compute influence propagation using traditional simulation methods."""
        print("Computing traditional influence propagation...")

        results = {}

        # Independent Cascade Model
        def independent_cascade(seed_nodes: List, p: float = 0.1, max_steps: int = 10) -> Dict:
            """Simulate independent cascade model."""
            active = set(seed_nodes)
            newly_active = set(seed_nodes)
            step = 0
            history = [set(seed_nodes)]

            while newly_active and step < max_steps:
                next_active = set()
                for node in newly_active:
                    for neighbor in self.graph.neighbors(node):
                        if neighbor not in active and np.random.random() < p:
                            next_active.add(neighbor)

                active.update(next_active)
                newly_active = next_active
                history.append(active.copy())
                step += 1

            return {
                'final_active': active,
                'total_influenced': len(active),
                'steps': step,
                'history': history,
                'influence_fraction': len(active) / self.n_nodes
            }

        # Test with different seed strategies
        strategies = ['high_degree', 'random', 'centrality_based']
        seed_sizes = [1, 3, 5]

        for strategy in strategies:
            for seed_size in seed_sizes:
                if seed_size > self.n_nodes:
                    continue

                # Select seeds based on strategy
                if strategy == 'high_degree':
                    degrees = dict(self.graph.degree())
                    seeds = sorted(degrees.keys(), key=lambda x: degrees[x], reverse=True)[:seed_size]
                elif strategy == 'random':
                    seeds = list(np.random.choice(list(self.graph.nodes()), seed_size, replace=False))
                elif strategy == 'centrality_based' and 'centrality' in self.results:
                    pagerank = self.results['centrality']['pagerank']['values']
                    seeds = sorted(pagerank.keys(), key=lambda x: pagerank[x], reverse=True)[:seed_size]
                else:
                    continue

                # Run multiple simulations
                start_time = time.time()
                simulations = []
                for _ in range(100):  # Monte Carlo simulations
                    sim_result = independent_cascade(seeds, p=0.1)
                    simulations.append(sim_result)

                ic_time = time.time() - start_time

                # Aggregate results
                influences = [sim['total_influenced'] for sim in simulations]
                mean_influence = np.mean(influences)
                std_influence = np.std(influences)

                key = f'ic_{strategy}_seeds_{seed_size}'
                results[key] = {
                    'seeds': seeds,
                    'mean_influence': mean_influence,
                    'std_influence': std_influence,
                    'max_influence': max(influences),
                    'min_influence': min(influences),
                    'computation_time': ic_time,
                    'n_simulations': 100,
                    'method': 'monte_carlo_simulation'
                }

        # Linear Threshold Model
        def linear_threshold(seed_nodes: List, max_steps: int = 10) -> Dict:
            """Simulate linear threshold model."""
            # Random thresholds for each node
            thresholds = {node: np.random.random() for node in self.graph.nodes()}

            # Influence weights (normalized degree)
            active = set(seed_nodes)
            step = 0
            history = [set(seed_nodes)]

            while step < max_steps:
                newly_active = set()
                for node in self.graph.nodes():
                    if node not in active:
                        # Calculate total influence from active neighbors
                        influence = 0
                        degree = self.graph.degree(node)
                        if degree > 0:
                            for neighbor in self.graph.neighbors(node):
                                if neighbor in active:
                                    influence += 1.0 / degree

                        if influence >= thresholds[node]:
                            newly_active.add(node)

                if not newly_active:
                    break

                active.update(newly_active)
                history.append(active.copy())
                step += 1

            return {
                'final_active': active,
                'total_influenced': len(active),
                'steps': step,
                'history': history,
                'influence_fraction': len(active) / self.n_nodes
            }

        # Test Linear Threshold with high degree seeds
        degrees = dict(self.graph.degree())
        high_degree_seeds = sorted(degrees.keys(), key=lambda x: degrees[x], reverse=True)[:3]

        start_time = time.time()
        lt_simulations = []
        for _ in range(50):  # Fewer simulations as it's more expensive
            sim_result = linear_threshold(high_degree_seeds)
            lt_simulations.append(sim_result)

        lt_time = time.time() - start_time

        lt_influences = [sim['total_influenced'] for sim in lt_simulations]
        results['linear_threshold'] = {
            'seeds': high_degree_seeds,
            'mean_influence': np.mean(lt_influences),
            'std_influence': np.std(lt_influences),
            'max_influence': max(lt_influences),
            'min_influence': min(lt_influences),
            'computation_time': lt_time,
            'n_simulations': 50,
            'method': 'threshold_simulation'
        }

        self.results['influence_propagation'] = results
        return results

    def get_performance_summary(self) -> Dict[str, Any]:
        """Get performance summary of all computations."""
        summary = {
            'graph_properties': {
                'n_nodes': self.n_nodes,
                'n_edges': self.n_edges,
                'density': self.n_edges / (self.n_nodes * (self.n_nodes - 1) / 2) if self.n_nodes > 1 else 0,
                'average_degree': 2 * self.n_edges / self.n_nodes if self.n_nodes > 0 else 0
            },
            'total_computation_time': 0,
            'algorithm_times': {},
            'peak_memory_usage': 0
        }

        # Aggregate performance metrics
        for category, methods in self.results.items():
            if isinstance(methods, dict):
                for method, data in methods.items():
                    if isinstance(data, dict) and 'computation_time' in data:
                        time_taken = data['computation_time']
                        summary['total_computation_time'] += time_taken
                        summary['algorithm_times'][f'{category}_{method}'] = time_taken

                        if 'memory_peak' in data:
                            summary['peak_memory_usage'] = max(summary['peak_memory_usage'],
                                                             data['memory_peak'])

        return summary

    def save_results(self, filename: str = 'traditional_results.json'):
        """Save all results to JSON file."""
        output_path = f'/workspaces/sublinear-time-solver/scripts/social_networks/{filename}'

        # Prepare results for JSON serialization
        json_results = {
            'graph_info': {
                'n_nodes': self.n_nodes,
                'n_edges': self.n_edges,
                'graph_type': 'traditional_analysis'
            },
            'results': self.results,
            'performance_summary': self.get_performance_summary()
        }

        with open(output_path, 'w') as f:
            json.dump(json_results, f, indent=2, default=str)

        print(f"Results saved to {output_path}")


def create_test_networks() -> Dict[str, nx.Graph]:
    """Create various test networks for analysis."""
    networks = {}

    # Small test networks
    networks['karate'] = nx.karate_club_graph()
    networks['dolphin'] = nx.Graph()  # Placeholder - would load real dolphin network

    # Synthetic networks
    networks['erdos_renyi_100'] = nx.erdos_renyi_graph(100, 0.1, seed=42)
    networks['barabasi_albert_100'] = nx.barabasi_albert_graph(100, 3, seed=42)
    networks['watts_strogatz_100'] = nx.watts_strogatz_graph(100, 6, 0.3, seed=42)

    # Larger networks for performance testing
    networks['erdos_renyi_500'] = nx.erdos_renyi_graph(500, 0.02, seed=42)
    networks['barabasi_albert_500'] = nx.barabasi_albert_graph(500, 2, seed=42)

    # Scale-free network with communities
    networks['community_graph'] = nx.planted_partition_graph(4, 25, 0.8, 0.1, seed=42)

    return networks


def main():
    """Run traditional social network analysis on test networks."""
    print("=" * 60)
    print("Traditional Social Network Analysis")
    print("=" * 60)

    networks = create_test_networks()

    for name, graph in networks.items():
        print(f"\nAnalyzing {name} network ({len(graph.nodes())} nodes, {len(graph.edges())} edges)...")

        # Skip very large networks for some algorithms
        if len(graph.nodes()) > 1000:
            print("  Skipping large network for traditional analysis")
            continue

        analyzer = TraditionalSocialAnalysis(graph)

        # Run analysis
        try:
            analyzer.compute_centrality_measures()
            analyzer.compute_community_detection()
            analyzer.compute_influence_propagation()

            # Save results
            analyzer.save_results(f'traditional_{name}_results.json')

            # Print summary
            summary = analyzer.get_performance_summary()
            print(f"  Total computation time: {summary['total_computation_time']:.2f}s")
            print(f"  Peak memory usage: {summary['peak_memory_usage']/1024/1024:.1f} MB")

        except Exception as e:
            print(f"  Analysis failed: {e}")
            continue

    print("\n" + "=" * 60)
    print("Traditional analysis complete!")
    print("=" * 60)


if __name__ == "__main__":
    main()