# PageRank Comparison Suite

This directory contains a comprehensive PageRank algorithm comparison system that validates the sublinear-solver-mcp against traditional Python implementations.

## Overview

The PageRank Comparison Suite provides:
- **Traditional Implementations**: NetworkX, SciPy, Power Iteration, and Sparse Matrix approaches
- **Sublinear Implementation**: MCP sublinear-solver tools integration
- **Comprehensive Benchmarking**: Performance and accuracy validation across multiple graph types
- **Automated Testing**: Graph generation and systematic comparison framework

## Files Description

### Core Implementation Files

- **`traditional_pagerank.py`** - Traditional PageRank implementations
  - NetworkX built-in PageRank
  - SciPy eigenvalue-based approach
  - Power iteration method
  - Sparse matrix optimization

- **`sublinear_pagerank.py`** - Sublinear PageRank via MCP tools
  - Dense and sparse matrix format support
  - MCP tool integration simulation
  - Performance measurement framework

- **`actual_mcp_pagerank.py`** - Real MCP tool integration
  - Direct MCP sublinear-solver calls
  - True sublinear algorithm implementation
  - Performance comparison utilities

### Testing and Benchmarking

- **`generate_test_graphs.py`** - Test graph generation
  - Small dense graphs (10-100 nodes)
  - Large sparse graphs (1000-10000 nodes)
  - Power-law distribution graphs (web-like)
  - Social network graphs with communities
  - Directed acyclic graphs (DAGs)
  - Grid graphs with random edges
  - Complete and star graphs

- **`benchmark_pagerank.py`** - Comprehensive benchmarking framework
  - Multi-method performance comparison
  - Accuracy validation against baselines
  - Statistical analysis and reporting
  - Visualization generation

- **`test_mcp_integration.py`** - MCP integration validation
  - Direct MCP tool testing
  - Accuracy comparison with NetworkX
  - Integration verification

## Graph Categories

### 1. Small Dense Graphs (10-100 nodes)
- **Purpose**: Test correctness and baseline performance
- **Characteristics**: High edge density (30-40%)
- **Use Cases**: Algorithm validation, quick testing

### 2. Large Sparse Graphs (1000-10000 nodes)
- **Purpose**: Evaluate scalability and sublinear advantages
- **Characteristics**: Low edge density, realistic structure
- **Use Cases**: Performance benchmarking, memory efficiency testing

### 3. Power-Law Graphs (200-1000 nodes)
- **Purpose**: Web-like graph simulation
- **Characteristics**: Scale-free network with preferential attachment
- **Use Cases**: Web search, social media analysis

### 4. Social Network Graphs (300-1000 nodes)
- **Purpose**: Community structure testing
- **Characteristics**: Multiple communities with hub nodes
- **Use Cases**: Social media analysis, influence ranking

### 5. Directed Acyclic Graphs (100-400 nodes)
- **Purpose**: Hierarchical structure analysis
- **Characteristics**: No cycles, layered structure
- **Use Cases**: Citation networks, dependency graphs

### 6. Grid Graphs (10x10 to 20x20)
- **Purpose**: Regular structure with spatial properties
- **Characteristics**: 2D grid with random long-distance edges
- **Use Cases**: Geographic networks, mesh analysis

## Usage

### Basic Testing

```bash
# Generate test graphs
python generate_test_graphs.py

# Test individual implementations
python traditional_pagerank.py
python sublinear_pagerank.py
python actual_mcp_pagerank.py

# Test MCP integration
python test_mcp_integration.py
```

### Comprehensive Benchmarking

```bash
# Run full benchmark suite
python benchmark_pagerank.py

# Benchmark specific categories
python benchmark_pagerank.py --categories small_dense large_sparse

# Limit graphs per category
python benchmark_pagerank.py --max-per-category 3

# Custom output directory
python benchmark_pagerank.py --output-dir /path/to/results
```

### MCP Tool Integration

The actual MCP PageRank tool can be called directly:

```python
# Using the MCP sublinear-solver PageRank tool
result = mcp__sublinear_solver__pageRank(
    adjacency={
        "rows": 4,
        "cols": 4,
        "format": "dense",
        "data": [[0, 1, 1, 0], [1, 0, 1, 1], [1, 1, 0, 1], [0, 1, 1, 0]]
    },
    damping=0.85,
    epsilon=1e-06,
    maxIterations=1000
)
```

## Performance Metrics

### Execution Time
- Wall-clock time for PageRank computation
- Excludes graph loading and result formatting
- Measured using high-precision timers

### Memory Usage
- Peak memory consumption during computation
- Baseline and peak memory tracking
- Memory efficiency comparison

### Convergence Analysis
- Number of iterations to convergence
- Convergence rate comparison
- Early stopping effectiveness

### Accuracy Validation
- Mean Squared Error (MSE) vs baseline
- Maximum absolute error
- Pearson correlation coefficient
- Spearman rank correlation

## Expected Results

### Sublinear Advantages

The sublinear-solver should demonstrate advantages in:

1. **Large Sparse Graphs**: Significant speedup for graphs with >1000 nodes
2. **Memory Efficiency**: Lower memory footprint for large graphs
3. **Convergence Rate**: Faster convergence through advanced sampling
4. **Scalability**: Better performance scaling with graph size

### When Traditional Methods Win

Traditional methods may be faster for:

1. **Very Small Graphs**: Overhead dominates for <50 nodes
2. **Very Dense Graphs**: Matrix operations are already efficient
3. **High Precision Requirements**: When ε < 1e-8 is needed

## Output Files

### Results Directory Structure
```
results/
├── benchmark_results_YYYYMMDD_HHMMSS.json    # Detailed results
├── benchmark_summary_YYYYMMDD_HHMMSS.csv     # Summary statistics
├── benchmark_plots_YYYYMMDD_HHMMSS.png       # Performance visualizations
└── mcp_integration_test.json                 # MCP validation results
```

### Test Graphs Directory
```
test_graphs/
├── test_graphs.json                          # All generated graphs
└── test_summary.json                         # Graph statistics summary
```

## Dependencies

```bash
# Required packages
pip install numpy networkx scipy pandas matplotlib seaborn psutil

# Optional for enhanced testing
pip install scikit-learn  # For additional graph generators
```

## Example Results Interpretation

### Performance Summary
```
Method Performance Summary:
  networkx:
    Success rate: 100.00%
    Avg execution time: 0.0234s
    Avg memory usage: 2.34 MB

  mcp_sublinear:
    Success rate: 100.00%
    Avg execution time: 0.0089s
    Avg memory usage: 1.78 MB

Sublinear Advantage:
  Win rate: 87.50%
  Speedup factor: 2.63x
```

### Accuracy Analysis
```
Accuracy vs NetworkX baseline:
  mcp_sublinear:
    MSE: 1.23e-08
    Max error: 3.45e-06
    Correlation: 0.9999
    Rank correlation: 0.9998
```

## Troubleshooting

### Common Issues

1. **MCP Tool Not Available**
   - Ensure MCP server is running
   - Check tool registration
   - Verify network connectivity

2. **Memory Errors on Large Graphs**
   - Reduce graph sizes in test generation
   - Use sparse matrix formats
   - Increase system memory

3. **Convergence Issues**
   - Adjust epsilon tolerance
   - Increase max iterations
   - Check graph connectivity

### Debug Mode

Enable verbose logging:
```python
import logging
logging.basicConfig(level=logging.DEBUG)
```

## Integration with Main Project

This PageRank comparison suite integrates with the main sublinear-time-solver project:

1. **Validation**: Confirms MCP tool correctness
2. **Benchmarking**: Demonstrates performance advantages
3. **Documentation**: Provides usage examples
4. **Testing**: Automated validation pipeline

## Future Enhancements

1. **Additional Graph Types**: Temporal, multilayer, weighted graphs
2. **More Baselines**: GraphX, igraph, SNAP implementations
3. **Advanced Metrics**: Time complexity analysis, memory profiling
4. **Interactive Visualization**: Real-time performance dashboard
5. **Cloud Integration**: Distributed benchmarking across systems