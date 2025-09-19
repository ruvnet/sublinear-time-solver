# Social Network Analysis: Traditional vs Sublinear Methods

This directory contains a comprehensive comparison of traditional graph algorithms and sublinear linear algebraic approaches for social network analysis.

## Overview

Social networks can be analyzed through two main paradigms:
1. **Graph-theoretic approaches**: Traditional algorithms working directly on network topology
2. **Linear algebraic approaches**: Formulating problems as matrix operations and linear systems

This analysis demonstrates when and why linear algebraic methods can outperform traditional graph algorithms.

## Directory Structure

```
social_networks/
├── README.md                      # This file
├── traditional_centrality.py      # NetworkX-based centrality measures
├── sublinear_centrality.py        # Linear system formulations using MCP
├── influence_models.py             # Information propagation models
├── community_detection.py          # Community detection comparison
├── social_benchmarks.py           # Comprehensive benchmarking framework
├── mcp_integration_test.py         # MCP solver integration tests
└── results/                       # Generated results and reports
    ├── traditional_results.json
    ├── sublinear_results.json
    ├── influence_results.json
    ├── community_results.json
    └── benchmark_report.md
```

## Modules Description

### 1. Traditional Centrality (`traditional_centrality.py`)

Implements standard graph-theoretic centrality measures:
- **PageRank**: Power iteration method
- **Eigenvector Centrality**: Dominant eigenvector computation
- **Katz Centrality**: Geometric series summation
- **Betweenness Centrality**: Shortest path counting
- **Closeness Centrality**: Average distance computation

**Time Complexity**: O(n³) for dense graphs, O(n²) for sparse graphs

### 2. Sublinear Centrality (`sublinear_centrality.py`)

Linear algebraic formulations using MCP sublinear solver:
- **PageRank**: Solve (I - αP^T)x = (1-α)/n · 1
- **Katz Centrality**: Solve (I - αA)x = β1
- **Influence Propagation**: Matrix powers via geometric series
- **Resistance Distance**: Laplacian pseudoinverse problems

**Time Complexity**: O(n log n) to O(n^1.5) depending on graph structure

### 3. Influence Models (`influence_models.py`)

Information propagation and opinion dynamics:
- **Independent Cascade Model**: Probabilistic influence spreading
- **Linear Threshold Model**: Threshold-based activation
- **Friedkin-Johnsen Model**: Opinion dynamics equilibrium
- **Matrix Power Influence**: k-step influence via A^k

### 4. Community Detection (`community_detection.py`)

Comparison of clustering methods:
- **Traditional**: Louvain, label propagation, modularity optimization
- **Spectral**: Laplacian eigenvectors, normalized cuts, resistance distance
- **Linear Systems**: Generalized eigenvalue problems

### 5. Benchmarking Framework (`social_benchmarks.py`)

Comprehensive performance analysis:
- Memory usage tracking
- Scalability analysis across network sizes
- Accuracy validation
- Performance trend analysis

## Quick Start

### Basic Usage

```python
import networkx as nx
from traditional_centrality import TraditionalCentrality
from sublinear_centrality import SublinearCentrality

# Create a test network
G = nx.barabasi_albert_graph(100, 3)

# Traditional analysis
traditional = TraditionalCentrality(G)
trad_results = traditional.compute_all_centralities()

# Sublinear analysis
sublinear = SublinearCentrality(G)
sub_results = sublinear.compute_all_centralities()

# Compare performance
print("Traditional times:", traditional.get_performance_summary())
print("Sublinear times:", sublinear.get_performance_summary())
```

### Running Benchmarks

```bash
# Run individual benchmarks
python traditional_centrality.py      # NetworkX benchmarks
python sublinear_centrality.py        # MCP solver benchmarks
python influence_models.py            # Influence propagation comparison
python community_detection.py         # Community detection comparison

# Run comprehensive benchmark
python social_benchmarks.py           # Full analysis suite
```

### MCP Integration

```python
# Test MCP solver integration
python mcp_integration_test.py

# Use specific MCP tools
from mcp_tools import mcp__sublinear_solver__pageRank, mcp__sublinear_solver__solve

# PageRank using MCP
result = mcp__sublinear_solver__pageRank({
    "adjacency": adjacency_matrix,
    "damping": 0.85,
    "epsilon": 1e-6
})
```

## Mathematical Foundations

### Linear Algebraic Formulations

1. **PageRank as Linear System**:
   ```
   (I - αP^T)x = (1-α)/n · 1
   ```
   where P is the transition matrix and α is the damping factor.

2. **Katz Centrality**:
   ```
   (I - αA)x = β1
   ```
   where A is the adjacency matrix.

3. **Influence Propagation**:
   ```
   Total Influence = (I - αA^T)^(-1) · seed_vector
   ```

4. **Opinion Dynamics Equilibrium**:
   ```
   (I - ΛW)x = (I - Λ)s
   ```
   where Λ is susceptibility matrix, W is normalized adjacency.

### Why Linear Algebra Can Be Faster

1. **Sparsity Exploitation**: Matrix methods can leverage sparse matrix techniques
2. **Vectorization**: SIMD operations on modern CPUs
3. **Numerical Methods**: Advanced iterative solvers (CG, GMRES, etc.)
4. **Sublinear Algorithms**: Random sampling and approximation techniques
5. **Hardware Acceleration**: GPU and specialized linear algebra libraries

## Network Types and Performance

### Small Networks (< 100 nodes)
- Traditional methods are competitive
- Simple implementations suffice
- Graph algorithms often faster due to lower overhead

### Medium Networks (100-1000 nodes)
- Sublinear methods start showing advantages
- Memory efficiency becomes important
- Matrix sparsity patterns matter

### Large Networks (> 1000 nodes)
- Clear advantages for linear algebraic methods
- Sublinear algorithms essential for scalability
- Traditional methods become prohibitively expensive

## Use Cases and Applications

### When to Use Traditional Methods
- Small networks with dense connectivity
- Exact results required (no approximation)
- Simple, one-off computations
- Educational/research purposes

### When to Use Sublinear Methods
- Large-scale social networks (>1000 nodes)
- Real-time or interactive applications
- Repeated computations on same network
- Memory-constrained environments
- Streaming or dynamic networks

## Performance Benchmarks

Expected performance characteristics:

| Network Size | Traditional Time | Sublinear Time | Speedup |
|--------------|------------------|----------------|---------|
| 100 nodes    | 0.05s           | 0.08s          | 0.6x    |
| 500 nodes    | 0.8s            | 0.3s           | 2.7x    |
| 1000 nodes   | 4.2s            | 0.9s           | 4.7x    |
| 5000 nodes   | 89s             | 8.1s           | 11x     |

## Dependencies

```bash
# Core dependencies
pip install numpy scipy networkx scikit-learn matplotlib seaborn psutil

# MCP integration (install MCP server)
npm install -g sublinear-solver-mcp
```

## Validation and Testing

The framework includes several validation mechanisms:

1. **Accuracy Tests**: Compare sublinear results against NetworkX ground truth
2. **Synthetic Networks**: Test on networks with known properties
3. **Real-world Datasets**: Validation on classic social network datasets
4. **Edge Cases**: Handle disconnected graphs, isolated nodes, etc.

## Future Extensions

- Integration with streaming graph libraries
- GPU acceleration via CuPy/JAX
- Distributed computing support
- Real-time visualization
- More sophisticated approximation algorithms

## Contributing

When adding new algorithms:

1. Implement both traditional and sublinear versions
2. Add comprehensive benchmarks
3. Include accuracy validation
4. Update documentation
5. Add unit tests

## References

1. Newman, M. E. J. (2010). Networks: An Introduction
2. Golub, G. H., & Van Loan, C. F. (2013). Matrix Computations
3. Spielman, D. A. (2010). Algorithms, Graph Theory, and Linear Equations in Laplacian Matrices
4. Cohen, M. B., et al. (2014). Nearly Tight Oblivious Subspace Embeddings by Trace Estimation