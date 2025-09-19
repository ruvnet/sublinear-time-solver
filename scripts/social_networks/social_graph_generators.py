#!/usr/bin/env python3
"""
Realistic Social Graph Generators

This module creates realistic social network graphs for testing sublinear solvers,
including networks with properties observed in real social media and collaboration networks.

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
import random


class SocialGraphGenerator:
    """Generator for realistic social network graphs."""

    def __init__(self, seed: int = 42):
        """Initialize with random seed for reproducibility."""
        self.seed = seed
        np.random.seed(seed)
        random.seed(seed)

    def generate_facebook_like_network(self, n_users: int = 1000,
                                     avg_degree: int = 50,
                                     clustering_coefficient: float = 0.4,
                                     community_sizes: Optional[List[int]] = None) -> nx.Graph:
        """
        Generate Facebook-like social network with high clustering and communities.

        Based on properties observed in Facebook networks:
        - High clustering coefficient (0.3-0.6)
        - Power-law degree distribution
        - Strong community structure
        - Small world properties
        """
        print(f"Generating Facebook-like network ({n_users} users, avg degree {avg_degree})...")

        if community_sizes is None:
            # Create communities of varying sizes
            n_communities = max(5, n_users // 200)
            community_sizes = []
            remaining = n_users

            for i in range(n_communities - 1):
                size = np.random.poisson(n_users // n_communities)
                size = min(size, remaining - (n_communities - i - 1))
                community_sizes.append(max(1, size))
                remaining -= size

            community_sizes.append(remaining)

        print(f"  Creating {len(community_sizes)} communities: {community_sizes}")

        # Create community structure
        communities = []
        node_id = 0

        for size in community_sizes:
            community_nodes = list(range(node_id, node_id + size))
            communities.append(community_nodes)
            node_id += size

        # Start with empty graph
        G = nx.Graph()
        G.add_nodes_from(range(n_users))

        # Add intra-community edges with high probability
        intra_prob = 0.3  # High probability within communities

        for community in communities:
            for i, node1 in enumerate(community):
                for node2 in community[i+1:]:
                    if np.random.random() < intra_prob:
                        G.add_edge(node1, node2)

        # Add inter-community edges with lower probability
        inter_prob = 0.01  # Low probability between communities

        for i, comm1 in enumerate(communities):
            for comm2 in communities[i+1:]:
                # Sample pairs to connect
                n_connections = max(1, int(len(comm1) * len(comm2) * inter_prob))

                for _ in range(n_connections):
                    node1 = np.random.choice(comm1)
                    node2 = np.random.choice(comm2)
                    G.add_edge(node1, node2)

        # Add preferential attachment to create hubs
        target_edges = (n_users * avg_degree) // 2
        current_edges = G.number_of_edges()

        if current_edges < target_edges:
            # Add more edges using preferential attachment
            additional_edges = target_edges - current_edges

            for _ in range(additional_edges):
                # Select nodes with probability proportional to degree
                degrees = dict(G.degree())
                if sum(degrees.values()) == 0:
                    continue

                # Weighted selection
                nodes = list(G.nodes())
                weights = [degrees[node] + 1 for node in nodes]  # +1 to avoid zero weights
                prob = np.array(weights) / sum(weights)

                node1, node2 = np.random.choice(nodes, size=2, replace=False, p=prob)

                if not G.has_edge(node1, node2):
                    G.add_edge(node1, node2)

        # Add node attributes
        for node in G.nodes():
            G.nodes[node]['community'] = next(i for i, comm in enumerate(communities) if node in comm)
            G.nodes[node]['type'] = 'user'

        print(f"  Created network: {G.number_of_nodes()} nodes, {G.number_of_edges()} edges")
        print(f"  Clustering coefficient: {nx.average_clustering(G):.3f}")

        return G

    def generate_twitter_like_network(self, n_users: int = 1000,
                                    n_celebrities: int = 50,
                                    avg_followers: int = 30) -> nx.DiGraph:
        """
        Generate Twitter-like directed network with celebrities and followers.

        Properties:
        - Directed edges (follows relationships)
        - Power-law degree distribution
        - Celebrities with many followers
        - Low reciprocity for celebrity edges
        """
        print(f"Generating Twitter-like network ({n_users} users, {n_celebrities} celebrities)...")

        G = nx.DiGraph()

        # Create users
        regular_users = list(range(n_celebrities, n_users))
        celebrities = list(range(n_celebrities))

        G.add_nodes_from(range(n_users))

        # Set node attributes
        for node in celebrities:
            G.nodes[node]['type'] = 'celebrity'
            G.nodes[node]['verified'] = True

        for node in regular_users:
            G.nodes[node]['type'] = 'regular'
            G.nodes[node]['verified'] = False

        # Celebrities follow each other with moderate probability
        for i, celeb1 in enumerate(celebrities):
            for celeb2 in celebrities[i+1:]:
                if np.random.random() < 0.3:
                    G.add_edge(celeb1, celeb2)
                if np.random.random() < 0.3:
                    G.add_edge(celeb2, celeb1)

        # Regular users follow celebrities with high probability
        for user in regular_users:
            n_celeb_follows = np.random.poisson(min(5, n_celebrities // 2))
            celeb_follows = np.random.choice(celebrities, size=min(n_celeb_follows, n_celebrities), replace=False)

            for celeb in celeb_follows:
                G.add_edge(user, celeb)

        # Regular users follow each other
        target_edges = (n_users * avg_followers) - G.number_of_edges()

        for _ in range(max(0, target_edges)):
            user1 = np.random.choice(regular_users)
            user2 = np.random.choice(regular_users)

            if user1 != user2 and not G.has_edge(user1, user2):
                G.add_edge(user1, user2)

        print(f"  Created network: {G.number_of_nodes()} nodes, {G.number_of_edges()} edges")

        return G

    def generate_collaboration_network(self, n_researchers: int = 500,
                                     n_papers: int = 200,
                                     avg_authors_per_paper: int = 3) -> nx.Graph:
        """
        Generate scientific collaboration network.

        Properties:
        - Bipartite structure (researchers-papers)
        - Projected to researcher collaboration
        - High clustering due to shared projects
        - Small components for different fields
        """
        print(f"Generating collaboration network ({n_researchers} researchers, {n_papers} papers)...")

        # Create bipartite graph
        B = nx.Graph()

        # Add nodes
        researchers = [f"R{i}" for i in range(n_researchers)]
        papers = [f"P{i}" for i in range(n_papers)]

        B.add_nodes_from(researchers, bipartite=0)
        B.add_nodes_from(papers, bipartite=1)

        # Assign researchers to papers
        for paper in papers:
            # Number of authors per paper (Poisson distribution)
            n_authors = max(1, np.random.poisson(avg_authors_per_paper))
            authors = np.random.choice(researchers, size=min(n_authors, n_researchers), replace=False)

            for author in authors:
                B.add_edge(author, paper)

        # Project to researcher collaboration network
        G = nx.projected_graph(B, researchers)

        # Add weights based on number of shared papers
        for edge in G.edges():
            shared_papers = len(set(B.neighbors(edge[0])) & set(B.neighbors(edge[1])))
            G.edges[edge]['weight'] = shared_papers

        # Add node attributes
        for researcher in G.nodes():
            papers_count = len(list(B.neighbors(researcher)))
            G.nodes[researcher]['papers'] = papers_count
            G.nodes[researcher]['productivity'] = 'high' if papers_count > avg_authors_per_paper else 'low'

        print(f"  Created network: {G.number_of_nodes()} nodes, {G.number_of_edges()} edges")

        return G

    def generate_hierarchical_organization(self, n_employees: int = 300,
                                         branching_factor: int = 3,
                                         cross_department_prob: float = 0.1) -> nx.DiGraph:
        """
        Generate hierarchical organizational network.

        Properties:
        - Tree-like hierarchy
        - Cross-departmental connections
        - Authority flows downward
        - Information flows upward and across
        """
        print(f"Generating organizational network ({n_employees} employees)...")

        G = nx.DiGraph()

        # Create hierarchy levels
        levels = []
        remaining = n_employees
        level = 0

        # CEO level
        levels.append([0])
        remaining -= 1
        level += 1

        # Build hierarchy
        while remaining > 0:
            prev_level = levels[-1]
            current_level = []

            for parent in prev_level:
                n_children = min(remaining, np.random.poisson(branching_factor))
                if n_children == 0:
                    continue

                children = list(range(n_employees - remaining, n_employees - remaining + n_children))
                current_level.extend(children)
                remaining -= n_children

                # Add hierarchical edges
                for child in children:
                    G.add_edge(parent, child, relationship='manages')

            if current_level:
                levels.append(current_level)
            level += 1

        # Add all nodes
        G.add_nodes_from(range(n_employees))

        # Add cross-departmental connections
        for level_nodes in levels[1:]:  # Skip CEO
            for node in level_nodes:
                # Connect to peers in other departments
                potential_peers = [n for n in level_nodes if n != node and not G.has_edge(node, n)]

                if potential_peers:
                    n_connections = np.random.binomial(len(potential_peers), cross_department_prob)
                    peers = np.random.choice(potential_peers, size=min(n_connections, len(potential_peers)), replace=False)

                    for peer in peers:
                        # Bidirectional peer relationship
                        G.add_edge(node, peer, relationship='collaborates')
                        G.add_edge(peer, node, relationship='collaborates')

        # Add node attributes
        for i, level_nodes in enumerate(levels):
            for node in level_nodes:
                G.nodes[node]['level'] = i
                G.nodes[node]['department'] = node % 5  # 5 departments
                G.nodes[node]['role'] = 'CEO' if i == 0 else f'Level_{i}'

        print(f"  Created network: {G.number_of_nodes()} nodes, {G.number_of_edges()} edges")

        return G

    def generate_online_community(self, n_users: int = 800,
                                n_topics: int = 20,
                                interest_overlap: float = 0.3) -> nx.Graph:
        """
        Generate online community network based on shared interests.

        Properties:
        - Users connected by shared interests
        - Multiple overlapping communities
        - Interest-based clustering
        - Influence based on expertise
        """
        print(f"Generating online community ({n_users} users, {n_topics} topics)...")

        G = nx.Graph()
        G.add_nodes_from(range(n_users))

        # Assign interests to users
        user_interests = {}
        topic_experts = defaultdict(list)

        for user in range(n_users):
            # Each user has 2-8 interests
            n_interests = np.random.randint(2, 9)
            interests = np.random.choice(n_topics, size=n_interests, replace=False)
            user_interests[user] = set(interests)

            # Some users are experts in their interests
            for interest in interests:
                if np.random.random() < 0.1:  # 10% chance to be expert
                    topic_experts[interest].append(user)

            G.nodes[user]['interests'] = list(interests)
            G.nodes[user]['expert_in'] = [i for i in interests if user in topic_experts[i]]

        # Connect users with overlapping interests
        for user1 in range(n_users):
            for user2 in range(user1 + 1, n_users):
                overlap = len(user_interests[user1] & user_interests[user2])
                total_interests = len(user_interests[user1] | user_interests[user2])

                if total_interests > 0:
                    overlap_ratio = overlap / total_interests

                    # Connect if overlap is significant
                    if overlap_ratio >= interest_overlap:
                        G.add_edge(user1, user2, shared_interests=overlap)

        # Add expert-follower connections
        for topic, experts in topic_experts.items():
            interested_users = [u for u in range(n_users) if topic in user_interests[u]]

            for expert in experts:
                # Expert connects to many interested users
                followers = np.random.choice(
                    [u for u in interested_users if u != expert],
                    size=min(10, len(interested_users) - 1),
                    replace=False
                )

                for follower in followers:
                    if not G.has_edge(expert, follower):
                        G.add_edge(expert, follower, relationship='expert_follower')

        print(f"  Created network: {G.number_of_nodes()} nodes, {G.number_of_edges()} edges")

        return G

    def generate_multilayer_social_network(self, n_users: int = 400) -> Dict[str, nx.Graph]:
        """
        Generate multi-layer social network with different relationship types.

        Returns:
            Dictionary with layers: 'friendship', 'professional', 'family', 'online'
        """
        print(f"Generating multi-layer social network ({n_users} users)...")

        layers = {}

        # Friendship layer (high clustering, small communities)
        friendship = nx.Graph()
        friendship.add_nodes_from(range(n_users))

        # Create friend groups
        group_size = 8
        n_groups = n_users // group_size

        for group_id in range(n_groups):
            start = group_id * group_size
            end = min(start + group_size, n_users)
            group_members = list(range(start, end))

            # Densely connect within group
            for i, member1 in enumerate(group_members):
                for member2 in group_members[i+1:]:
                    if np.random.random() < 0.6:
                        friendship.add_edge(member1, member2)

        # Add some inter-group friendships
        for _ in range(n_users // 10):
            user1, user2 = np.random.choice(n_users, size=2, replace=False)
            if not friendship.has_edge(user1, user2):
                friendship.add_edge(user1, user2)

        layers['friendship'] = friendship

        # Professional layer (based on workplace/industry)
        professional = nx.Graph()
        professional.add_nodes_from(range(n_users))

        # Create workplaces
        n_workplaces = n_users // 20
        for workplace in range(n_workplaces):
            workplace_size = np.random.poisson(20)
            employees = np.random.choice(n_users, size=min(workplace_size, n_users), replace=False)

            for i, emp1 in enumerate(employees):
                for emp2 in employees[i+1:]:
                    if np.random.random() < 0.3:
                        professional.add_edge(emp1, emp2)

        layers['professional'] = professional

        # Family layer (small cliques)
        family = nx.Graph()
        family.add_nodes_from(range(n_users))

        family_size = 4
        n_families = n_users // family_size

        for family_id in range(n_families):
            start = family_id * family_size
            end = min(start + family_size, n_users)
            family_members = list(range(start, end))

            # Fully connect family
            for i, member1 in enumerate(family_members):
                for member2 in family_members[i+1:]:
                    family.add_edge(member1, member2)

        layers['family'] = family

        # Online layer (interest-based, long-range connections)
        online = self.generate_online_community(n_users, n_topics=15, interest_overlap=0.2)
        layers['online'] = online

        print(f"  Created {len(layers)} layers")
        for layer_name, layer_graph in layers.items():
            print(f"    {layer_name}: {layer_graph.number_of_edges()} edges")

        return layers

    def analyze_network_properties(self, graph: nx.Graph, name: str = "network") -> Dict[str, Any]:
        """Analyze structural properties of generated network."""
        print(f"Analyzing {name} properties...")

        properties = {
            'name': name,
            'n_nodes': graph.number_of_nodes(),
            'n_edges': graph.number_of_edges(),
            'is_directed': graph.is_directed(),
            'density': nx.density(graph),
            'average_degree': 2 * graph.number_of_edges() / graph.number_of_nodes() if graph.number_of_nodes() > 0 else 0
        }

        # Additional properties for undirected graphs
        if not graph.is_directed() and graph.number_of_nodes() > 0:
            try:
                properties['average_clustering'] = nx.average_clustering(graph)
            except:
                properties['average_clustering'] = 0

            try:
                if nx.is_connected(graph):
                    properties['diameter'] = nx.diameter(graph)
                    properties['average_path_length'] = nx.average_shortest_path_length(graph)
                else:
                    largest_cc = max(nx.connected_components(graph), key=len)
                    subgraph = graph.subgraph(largest_cc)
                    properties['diameter'] = nx.diameter(subgraph)
                    properties['average_path_length'] = nx.average_shortest_path_length(subgraph)
                    properties['n_components'] = nx.number_connected_components(graph)
            except:
                properties['diameter'] = None
                properties['average_path_length'] = None

        # Degree distribution
        degrees = [d for n, d in graph.degree()]
        properties['degree_stats'] = {
            'mean': np.mean(degrees),
            'std': np.std(degrees),
            'min': min(degrees) if degrees else 0,
            'max': max(degrees) if degrees else 0,
            'median': np.median(degrees)
        }

        return properties

    def save_network(self, graph: nx.Graph, filename: str, format: str = 'gml'):
        """Save network to file."""
        output_path = f'/workspaces/sublinear-time-solver/scripts/social_networks/{filename}.{format}'

        if format == 'gml':
            nx.write_gml(graph, output_path)
        elif format == 'graphml':
            nx.write_graphml(graph, output_path)
        elif format == 'edgelist':
            nx.write_edgelist(graph, output_path)
        else:
            raise ValueError(f"Unsupported format: {format}")

        print(f"Network saved to {output_path}")


def main():
    """Generate and analyze realistic social networks."""
    print("=" * 60)
    print("Social Graph Generation and Analysis")
    print("=" * 60)

    generator = SocialGraphGenerator(seed=42)

    # Generate different types of networks
    networks = {}

    print("\n1. Generating Facebook-like network...")
    networks['facebook'] = generator.generate_facebook_like_network(n_users=500, avg_degree=40)

    print("\n2. Generating Twitter-like network...")
    networks['twitter'] = generator.generate_twitter_like_network(n_users=400, n_celebrities=20)

    print("\n3. Generating collaboration network...")
    networks['collaboration'] = generator.generate_collaboration_network(n_researchers=300, n_papers=150)

    print("\n4. Generating organizational network...")
    networks['organization'] = generator.generate_hierarchical_organization(n_employees=200)

    print("\n5. Generating online community...")
    networks['community'] = generator.generate_online_community(n_users=300, n_topics=15)

    print("\n6. Generating multi-layer network...")
    multilayer = generator.generate_multilayer_social_network(n_users=200)

    # Analyze all networks
    print("\n" + "=" * 40)
    print("NETWORK ANALYSIS")
    print("=" * 40)

    all_analyses = {}

    for name, graph in networks.items():
        print(f"\n{name.upper()} NETWORK:")
        analysis = generator.analyze_network_properties(graph, name)
        all_analyses[name] = analysis

        print(f"  Nodes: {analysis['n_nodes']}, Edges: {analysis['n_edges']}")
        print(f"  Density: {analysis['density']:.4f}")
        print(f"  Average degree: {analysis['average_degree']:.2f}")

        if 'average_clustering' in analysis:
            print(f"  Clustering: {analysis['average_clustering']:.3f}")

        if 'diameter' in analysis and analysis['diameter']:
            print(f"  Diameter: {analysis['diameter']}")

        # Save network
        try:
            generator.save_network(graph, f'generated_{name}', format='gml')
        except:
            print(f"  Warning: Could not save {name} network")

    # Analyze multi-layer networks
    print(f"\nMULTI-LAYER NETWORK:")
    multilayer_analyses = {}
    for layer_name, layer_graph in multilayer.items():
        analysis = generator.analyze_network_properties(layer_graph, f"multilayer_{layer_name}")
        multilayer_analyses[layer_name] = analysis
        print(f"  {layer_name}: {analysis['n_nodes']} nodes, {analysis['n_edges']} edges")

    all_analyses['multilayer'] = multilayer_analyses

    # Save analysis results
    output_path = '/workspaces/sublinear-time-solver/scripts/social_networks/generated_networks_analysis.json'
    with open(output_path, 'w') as f:
        json.dump(all_analyses, f, indent=2, default=str)

    print(f"\nAnalysis results saved to generated_networks_analysis.json")

    print("\n" + "=" * 60)
    print("Social graph generation complete!")
    print("=" * 60)


if __name__ == "__main__":
    main()