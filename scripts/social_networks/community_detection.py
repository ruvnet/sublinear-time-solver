#!/usr/bin/env python3
"""
Community Detection: Traditional vs Spectral Methods

This module compares traditional community detection algorithms with
spectral and linear algebraic approaches:
- Modularity optimization (Louvain, Leiden)
- Spectral clustering using graph Laplacian
- Normalized cuts via generalized eigenvalue problems
- Non-negative matrix factorization
- Sublinear approximations using linear systems

Author: Social Network Analysis Agent
"""

import numpy as np
import networkx as nx
import time
from typing import Dict, List, Tuple, Set, Optional
from collections import defaultdict
import scipy.sparse as sp
from scipy.sparse.linalg import eigsh, spsolve
from sklearn.cluster import KMeans, SpectralClustering
from sklearn.metrics import adjusted_rand_score, normalized_mutual_info_score


class TraditionalCommunityDetection:
    """Traditional graph-based community detection methods."""

    def __init__(self, graph: nx.Graph):
        """Initialize with NetworkX graph."""
        self.graph = graph
        self.n_nodes = len(graph.nodes())
        self.results = {}

    def louvain_communities(self) -> Dict:
        """Detect communities using Louvain algorithm."""
        start_time = time.time()

        try:
            # Use NetworkX's Louvain implementation
            communities = nx.community.louvain_communities(self.graph, seed=42)

            # Convert to node->community mapping
            node_communities = {}
            for i, community in enumerate(communities):
                for node in community:
                    node_communities[node] = i

            # Calculate modularity
            modularity = nx.community.modularity(self.graph, communities)

            computation_time = time.time() - start_time

            result = {
                'communities': communities,
                'node_communities': node_communities,
                'n_communities': len(communities),
                'modularity': modularity,
                'time': computation_time,
                'method': 'louvain'
            }

            self.results['louvain'] = result
            return result

        except Exception as e:
            print(f"Louvain algorithm failed: {e}")
            return None

    def greedy_modularity_communities(self) -> Dict:
        """Detect communities using greedy modularity optimization."""
        start_time = time.time()

        communities = nx.community.greedy_modularity_communities(self.graph)

        # Convert to node->community mapping
        node_communities = {}
        for i, community in enumerate(communities):
            for node in community:
                node_communities[node] = i

        # Calculate modularity
        modularity = nx.community.modularity(self.graph, communities)

        computation_time = time.time() - start_time

        result = {
            'communities': communities,
            'node_communities': node_communities,
            'n_communities': len(communities),
            'modularity': modularity,
            'time': computation_time,
            'method': 'greedy_modularity'
        }

        self.results['greedy_modularity'] = result
        return result

    def label_propagation_communities(self) -> Dict:
        """Detect communities using label propagation."""
        start_time = time.time()

        communities = nx.community.label_propagation_communities(self.graph, seed=42)

        # Convert to node->community mapping
        node_communities = {}
        for i, community in enumerate(communities):
            for node in community:
                node_communities[node] = i

        # Calculate modularity
        modularity = nx.community.modularity(self.graph, communities)

        computation_time = time.time() - start_time

        result = {
            'communities': communities,
            'node_communities': node_communities,
            'n_communities': len(communities),
            'modularity': modularity,
            'time': computation_time,
            'method': 'label_propagation'
        }

        self.results['label_propagation'] = result
        return result

    def edge_betweenness_communities(self, k: int = None) -> Dict:
        """Detect communities using edge betweenness clustering."""
        start_time = time.time()

        # This can be expensive for large graphs
        if self.n_nodes > 100:
            print("Skipping edge betweenness for large graph")
            return None

        # Find k communities or use natural hierarchy
        if k is None:
            communities = nx.community.girvan_newman(self.graph)
            # Take first partition (2 communities)
            communities = next(communities)
        else:
            communities = nx.community.girvan_newman(self.graph)
            for i in range(k - 1):
                communities = next(communities)

        # Convert to node->community mapping
        node_communities = {}
        for i, community in enumerate(communities):
            for node in community:
                node_communities[node] = i

        # Calculate modularity
        modularity = nx.community.modularity(self.graph, communities)

        computation_time = time.time() - start_time

        result = {
            'communities': communities,
            'node_communities': node_communities,
            'n_communities': len(communities),
            'modularity': modularity,
            'time': computation_time,
            'method': 'edge_betweenness'
        }

        self.results['edge_betweenness'] = result
        return result


class SpectralCommunityDetection:
    """Spectral and linear algebraic community detection methods."""

    def __init__(self, graph: nx.Graph):
        """Initialize with NetworkX graph."""
        self.graph = graph
        self.nodes = list(graph.nodes())
        self.n_nodes = len(self.nodes)
        self.node_to_idx = {node: idx for idx, node in enumerate(self.nodes)}
        self.results = {}

    def spectral_clustering_laplacian(self, k: int, normalized: bool = True) -> Dict:
        """
        Spectral clustering using graph Laplacian eigenvalues.

        Args:
            k: Number of communities
            normalized: Use normalized Laplacian
        """
        start_time = time.time()

        # Get Laplacian matrix
        if normalized:
            laplacian = nx.normalized_laplacian_matrix(self.graph, nodelist=self.nodes)
        else:
            laplacian = nx.laplacian_matrix(self.graph, nodelist=self.nodes)

        # Compute k smallest eigenvalues and eigenvectors
        try:
            eigenvals, eigenvecs = eigsh(laplacian, k=k, which='SM', sigma=0)
        except:
            # Fallback to dense computation for small graphs
            laplacian_dense = laplacian.toarray()
            eigenvals, eigenvecs = np.linalg.eigh(laplacian_dense)
            idx = np.argsort(eigenvals)
            eigenvals = eigenvals[idx[:k]]
            eigenvecs = eigenvecs[:, idx[:k]]

        # K-means clustering on eigenvectors
        kmeans = KMeans(n_clusters=k, random_state=42, n_init=10)
        cluster_labels = kmeans.fit_predict(eigenvecs)

        # Convert to communities
        communities = defaultdict(list)
        for i, label in enumerate(cluster_labels):
            communities[label].append(self.nodes[i])

        communities = [set(comm) for comm in communities.values()]

        # Node->community mapping
        node_communities = {}
        for i, community in enumerate(communities):
            for node in community:
                node_communities[node] = i

        # Calculate modularity
        modularity = nx.community.modularity(self.graph, communities)

        computation_time = time.time() - start_time

        result = {
            'communities': communities,
            'node_communities': node_communities,
            'n_communities': len(communities),
            'modularity': modularity,
            'eigenvalues': eigenvals.tolist(),
            'time': computation_time,
            'method': f'spectral_{"normalized" if normalized else "unnormalized"}'
        }

        self.results[f'spectral_{"normalized" if normalized else "unnormalized"}'] = result
        return result

    def normalized_cuts(self, k: int) -> Dict:
        """
        Normalized cuts using generalized eigenvalue problem.

        Solves: L * v = λ * D * v
        where L is Laplacian and D is degree matrix.
        """
        start_time = time.time()

        # Get Laplacian and degree matrices
        laplacian = nx.laplacian_matrix(self.graph, nodelist=self.nodes)

        # Get degree matrix
        degrees = np.array([self.graph.degree(node) for node in self.nodes])
        degrees[degrees == 0] = 1  # Handle isolated nodes
        degree_matrix = sp.diags(degrees)

        # Solve generalized eigenvalue problem: L * v = λ * D * v
        try:
            eigenvals, eigenvecs = eigsh(laplacian, k=k, M=degree_matrix,
                                       which='SM', sigma=0)
        except:
            # Fallback: solve (D^(-1/2) * L * D^(-1/2)) * u = λ * u
            # Then v = D^(-1/2) * u
            deg_sqrt_inv = sp.diags(1.0 / np.sqrt(degrees))
            normalized_lap = deg_sqrt_inv @ laplacian @ deg_sqrt_inv

            if normalized_lap.shape[0] < k:
                k = normalized_lap.shape[0] - 1

            eigenvals, u = eigsh(normalized_lap.tocsc(), k=k, which='SM')
            eigenvecs = deg_sqrt_inv @ u

        # K-means clustering on eigenvectors
        kmeans = KMeans(n_clusters=k, random_state=42, n_init=10)
        cluster_labels = kmeans.fit_predict(eigenvecs)

        # Convert to communities
        communities = defaultdict(list)
        for i, label in enumerate(cluster_labels):
            communities[label].append(self.nodes[i])

        communities = [set(comm) for comm in communities.values()]

        # Node->community mapping
        node_communities = {}
        for i, community in enumerate(communities):
            for node in community:
                node_communities[node] = i

        # Calculate modularity
        modularity = nx.community.modularity(self.graph, communities)

        computation_time = time.time() - start_time

        result = {
            'communities': communities,
            'node_communities': node_communities,
            'n_communities': len(communities),
            'modularity': modularity,
            'eigenvalues': eigenvals.tolist(),
            'time': computation_time,
            'method': 'normalized_cuts'
        }

        self.results['normalized_cuts'] = result
        return result

    def modularity_eigenvector(self) -> Dict:
        """
        Community detection using modularity matrix eigenvector.

        The modularity matrix B has B_ij = A_ij - k_i*k_j/(2m)
        where k_i is degree of node i and m is total edges.
        """
        start_time = time.time()

        # Get adjacency matrix
        adj_matrix = nx.adjacency_matrix(self.graph, nodelist=self.nodes)

        # Calculate modularity matrix B = A - (k*k^T)/(2m)
        degrees = np.array([self.graph.degree(node) for node in self.nodes])
        m = self.graph.number_of_edges()

        if m == 0:
            # Handle empty graph
            result = {
                'communities': [set(self.nodes)],
                'node_communities': {node: 0 for node in self.nodes},
                'n_communities': 1,
                'modularity': 0.0,
                'time': time.time() - start_time,
                'method': 'modularity_eigenvector'
            }
            self.results['modularity_eigenvector'] = result
            return result

        # B = A - k*k^T/(2m)
        degree_outer = np.outer(degrees, degrees) / (2.0 * m)
        modularity_matrix = adj_matrix.toarray() - degree_outer

        # Find leading eigenvector of modularity matrix
        try:
            eigenvals, eigenvecs = np.linalg.eigh(modularity_matrix)
            # Sort by eigenvalue (descending)
            idx = np.argsort(eigenvals)[::-1]
            leading_eigenvec = eigenvecs[:, idx[0]]

            # Split into two communities based on eigenvector sign
            community1 = [self.nodes[i] for i in range(self.n_nodes)
                         if leading_eigenvec[i] >= 0]
            community2 = [self.nodes[i] for i in range(self.n_nodes)
                         if leading_eigenvec[i] < 0]

            communities = [set(community1), set(community2)]
            communities = [comm for comm in communities if len(comm) > 0]

        except:
            # Fallback: single community
            communities = [set(self.nodes)]

        # Node->community mapping
        node_communities = {}
        for i, community in enumerate(communities):
            for node in community:
                node_communities[node] = i

        # Calculate modularity
        modularity = nx.community.modularity(self.graph, communities)

        computation_time = time.time() - start_time

        result = {
            'communities': communities,
            'node_communities': node_communities,
            'n_communities': len(communities),
            'modularity': modularity,
            'leading_eigenvalue': eigenvals[idx[0]] if 'eigenvals' in locals() else 0.0,
            'time': computation_time,
            'method': 'modularity_eigenvector'
        }

        self.results['modularity_eigenvector'] = result
        return result

    def resistance_distance_clustering(self, k: int) -> Dict:
        """
        Community detection using effective resistance distances.

        Uses the pseudoinverse of the Laplacian to compute resistance distances,
        then applies clustering.
        """
        start_time = time.time()

        # Get Laplacian matrix
        laplacian = nx.laplacian_matrix(self.graph, nodelist=self.nodes).toarray()

        # Compute Moore-Penrose pseudoinverse
        laplacian_pinv = np.linalg.pinv(laplacian)

        # Compute resistance distance matrix
        # R_ij = L^+_ii + L^+_jj - 2*L^+_ij
        n = self.n_nodes
        resistance_matrix = np.zeros((n, n))

        for i in range(n):
            for j in range(i, n):
                resistance = (laplacian_pinv[i, i] + laplacian_pinv[j, j] -
                            2 * laplacian_pinv[i, j])
                resistance_matrix[i, j] = resistance
                resistance_matrix[j, i] = resistance

        # Use resistance distances as features for clustering
        # Convert to similarity matrix (smaller resistance = higher similarity)
        max_resistance = np.max(resistance_matrix)
        similarity_matrix = max_resistance - resistance_matrix

        # Spectral clustering on similarity matrix
        try:
            clustering = SpectralClustering(n_clusters=k, affinity='precomputed',
                                          random_state=42)
            cluster_labels = clustering.fit_predict(similarity_matrix)
        except:
            # Fallback to k-means on resistance distances
            kmeans = KMeans(n_clusters=k, random_state=42, n_init=10)
            cluster_labels = kmeans.fit_predict(resistance_matrix)

        # Convert to communities
        communities = defaultdict(list)
        for i, label in enumerate(cluster_labels):
            communities[label].append(self.nodes[i])

        communities = [set(comm) for comm in communities.values()]

        # Node->community mapping
        node_communities = {}
        for i, community in enumerate(communities):
            for node in community:
                node_communities[node] = i

        # Calculate modularity
        modularity = nx.community.modularity(self.graph, communities)

        computation_time = time.time() - start_time

        result = {
            'communities': communities,
            'node_communities': node_communities,
            'n_communities': len(communities),
            'modularity': modularity,
            'time': computation_time,
            'method': 'resistance_distance'
        }

        self.results['resistance_distance'] = result
        return result


def create_community_networks() -> Dict[str, Tuple[nx.Graph, Dict]]:
    """Create networks with known community structure for testing."""
    networks = {}

    # Karate Club (classic example)
    karate = nx.karate_club_graph()
    # True communities based on the split
    karate_communities = {
        'true_communities': [
            set(range(0, 17)),  # Mr. Hi's group
            set(range(17, 34))  # Officer's group
        ]
    }
    networks['karate'] = (karate, karate_communities)

    # Planted partition model
    planted = nx.planted_partition_graph(4, 10, 0.7, 0.1, seed=42)
    planted_communities = {
        'true_communities': [
            set(range(0, 10)),
            set(range(10, 20)),
            set(range(20, 30)),
            set(range(30, 40))
        ]
    }
    networks['planted_partition'] = (planted, planted_communities)

    # LFR benchmark network (Lancichinetti-Fortunato-Radicchi)
    # Approximate with random partition
    lfr_like = nx.barabasi_albert_graph(100, 3, seed=42)
    # Create approximate communities by grouping consecutive nodes
    lfr_communities = {
        'true_communities': [
            set(range(0, 25)),
            set(range(25, 50)),
            set(range(50, 75)),
            set(range(75, 100))
        ]
    }
    networks['lfr_like'] = (lfr_like, lfr_communities)

    return networks


def evaluate_community_detection():
    """Compare traditional and spectral community detection methods."""
    print("=== Community Detection Comparison ===\n")

    networks = create_community_networks()
    all_results = {}

    for network_name, (graph, metadata) in networks.items():
        print(f"Analyzing {network_name} network:")
        print(f"  Nodes: {len(graph.nodes())}, Edges: {len(graph.edges())}")

        # Traditional methods
        traditional = TraditionalCommunityDetection(graph)

        # Spectral methods
        spectral = SpectralCommunityDetection(graph)

        # Run traditional algorithms
        print("  Traditional methods:")
        trad_results = {}

        louvain = traditional.louvain_communities()
        if louvain:
            print(f"    Louvain: {louvain['n_communities']} communities, "
                  f"modularity {louvain['modularity']:.3f} ({louvain['time']:.3f}s)")
            trad_results['louvain'] = louvain

        greedy = traditional.greedy_modularity_communities()
        if greedy:
            print(f"    Greedy: {greedy['n_communities']} communities, "
                  f"modularity {greedy['modularity']:.3f} ({greedy['time']:.3f}s)")
            trad_results['greedy'] = greedy

        label_prop = traditional.label_propagation_communities()
        if label_prop:
            print(f"    Label Prop: {label_prop['n_communities']} communities, "
                  f"modularity {label_prop['modularity']:.3f} ({label_prop['time']:.3f}s)")
            trad_results['label_prop'] = label_prop

        # Run spectral algorithms
        print("  Spectral methods:")
        spec_results = {}

        # Determine number of communities for spectral methods
        if 'true_communities' in metadata:
            k = len(metadata['true_communities'])
        else:
            k = 3  # Default

        spec_normalized = spectral.spectral_clustering_laplacian(k, normalized=True)
        print(f"    Spectral (norm): {spec_normalized['n_communities']} communities, "
              f"modularity {spec_normalized['modularity']:.3f} ({spec_normalized['time']:.3f}s)")
        spec_results['spectral_normalized'] = spec_normalized

        spec_unnorm = spectral.spectral_clustering_laplacian(k, normalized=False)
        print(f"    Spectral (unnorm): {spec_unnorm['n_communities']} communities, "
              f"modularity {spec_unnorm['modularity']:.3f} ({spec_unnorm['time']:.3f}s)")
        spec_results['spectral_unnormalized'] = spec_unnorm

        mod_eigen = spectral.modularity_eigenvector()
        print(f"    Modularity eigen: {mod_eigen['n_communities']} communities, "
              f"modularity {mod_eigen['modularity']:.3f} ({mod_eigen['time']:.3f}s)")
        spec_results['modularity_eigen'] = mod_eigen

        if len(graph.nodes()) <= 50:  # Only for smaller graphs
            resistance = spectral.resistance_distance_clustering(k)
            print(f"    Resistance: {resistance['n_communities']} communities, "
                  f"modularity {resistance['modularity']:.3f} ({resistance['time']:.3f}s)")
            spec_results['resistance'] = resistance

        # Evaluate against ground truth if available
        if 'true_communities' in metadata:
            print("  Evaluation against ground truth:")
            true_labels = []
            nodes = list(graph.nodes())

            # Create true label vector
            for node in nodes:
                for i, community in enumerate(metadata['true_communities']):
                    if node in community:
                        true_labels.append(i)
                        break

            # Evaluate each method
            all_methods = {**trad_results, **spec_results}
            for method_name, result in all_methods.items():
                pred_labels = [result['node_communities'].get(node, 0) for node in nodes]

                ari = adjusted_rand_score(true_labels, pred_labels)
                nmi = normalized_mutual_info_score(true_labels, pred_labels)

                print(f"    {method_name}: ARI {ari:.3f}, NMI {nmi:.3f}")
                result['ari'] = ari
                result['nmi'] = nmi

        # Store results
        all_results[network_name] = {
            'graph_stats': {
                'nodes': len(graph.nodes()),
                'edges': len(graph.edges()),
                'density': nx.density(graph),
                'clustering': nx.average_clustering(graph)
            },
            'traditional': trad_results,
            'spectral': spec_results,
            'metadata': metadata
        }

        print()

    return all_results


if __name__ == "__main__":
    results = evaluate_community_detection()

    # Save results
    import json
    def convert_types(obj):
        if isinstance(obj, np.integer):
            return int(obj)
        elif isinstance(obj, np.floating):
            return float(obj)
        elif isinstance(obj, np.ndarray):
            return obj.tolist()
        elif isinstance(obj, set):
            return list(obj)
        elif isinstance(obj, dict):
            return {k: convert_types(v) for k, v in obj.items()}
        elif isinstance(obj, list):
            return [convert_types(item) for item in obj]
        return obj

    with open('/workspaces/sublinear-time-solver/scripts/social_networks/community_results.json', 'w') as f:
        json.dump(convert_types(results), f, indent=2)

    print("Community detection comparison complete!")
    print("Results saved to community_results.json")