# Network Flow Algorithm Benchmark Report

Generated: 2025-09-19 21:30:19

## Summary

- Total benchmark runs: 64
- Successful runs: 64
- Algorithms tested: ford_fulkerson, edmonds_karp, push_relabel, sublinear_max_flow
- Network types tested: 2d_grid, 3d_grid, erdos_renyi, sparse_random, barabasi_albert, tree_hierarchy, watts_strogatz

## Performance by Algorithm

### ford_fulkerson
- Average runtime: 0.005139 seconds
- Runtime std deviation: 0.013918 seconds
- Average peak memory: 112.23 MB
- Number of runs: 16

### edmonds_karp
- Average runtime: 0.000823 seconds
- Runtime std deviation: 0.001895 seconds
- Average peak memory: 112.23 MB
- Number of runs: 16

### push_relabel
- Average runtime: 0.014946 seconds
- Runtime std deviation: 0.020612 seconds
- Average peak memory: 112.23 MB
- Number of runs: 16

### sublinear_max_flow
- Average runtime: 0.037088 seconds
- Runtime std deviation: 0.061457 seconds
- Average peak memory: 112.30 MB
- Number of runs: 16

## Performance by Network Type

### 2d_grid
- Average runtime: 0.010343 seconds
- Average max flow: 123.78
- Number of runs: 8

### 3d_grid
- Average runtime: 0.008722 seconds
- Average max flow: 130.01
- Number of runs: 8

### erdos_renyi
- Average runtime: 0.024738 seconds
- Average max flow: 104.87
- Number of runs: 16

### sparse_random
- Average runtime: 0.001352 seconds
- Average max flow: 130.50
- Number of runs: 8

### barabasi_albert
- Average runtime: 0.017494 seconds
- Average max flow: 133.55
- Number of runs: 8

### tree_hierarchy
- Average runtime: 0.001359 seconds
- Average max flow: 104.81
- Number of runs: 8

### watts_strogatz
- Average runtime: 0.027246 seconds
- Average max flow: 119.78
- Number of runs: 8

## Key Findings

- **Fastest algorithm**: edmonds_karp (avg: 0.000823s)
- **Most memory efficient**: ford_fulkerson (avg: 112.23 MB)

### Scalability Analysis
- **ford_fulkerson**: weak positive correlation between network size and runtime (r=0.334)
- **edmonds_karp**: weak positive correlation between network size and runtime (r=0.289)
- **push_relabel**: moderate positive correlation between network size and runtime (r=0.749)
- **sublinear_max_flow**: weak positive correlation between network size and runtime (r=0.314)
