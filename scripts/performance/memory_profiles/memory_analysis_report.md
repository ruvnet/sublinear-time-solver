# Memory Usage Analysis Report

Generated: 2025-09-19 21:36:03

## Executive Summary

- **Total profiles analyzed**: 21
- **Maximum peak memory**: 1526.4 MB
- **Average peak memory**: 171.8 MB
- **System memory**: 7.8 GB
- **Memory leaks detected**: 2 profiles (9.5%)

## Key Recommendations

- ⚠️ MEDIUM: pagerank in graph_algorithms shows superlinear memory growth (51.8x)

## Memory Efficiency Analysis

### Linear_Systems

**neumann**:
- Average efficiency: 0.0000 MB per unit
- Efficiency range: 0.0000 - 0.0001 MB per unit
- Standard deviation: 0.0000

**random-walk**:
- Average efficiency: 0.0000 MB per unit
- Efficiency range: 0.0000 - 0.0000 MB per unit
- Standard deviation: 0.0000

**forward-push**:
- Average efficiency: 0.0000 MB per unit
- Efficiency range: 0.0000 - 0.0000 MB per unit
- Standard deviation: 0.0000

### Graph_Algorithms

**pagerank**:
- Average efficiency: 0.0477 MB per unit
- Efficiency range: 0.0029 - 0.1526 MB per unit
- Standard deviation: 0.0529

## Memory Scaling Analysis

### Linear_Systems

**neumann**:
- Problem size range: 10000-25000000
- Memory range: 1.1-572.5 MB
- Memory scaling factor: 514.34x
- Memory growth rate: 0.21x problem size

**random-walk**:
- Problem size range: 10000-25000000
- Memory range: 0.3-381.7 MB
- Memory scaling factor: 1301.58x
- Memory growth rate: 0.52x problem size

**forward-push**:
- Problem size range: 10000-25000000
- Memory range: 0.3-381.7 MB
- Memory scaling factor: 1305.09x
- Memory growth rate: 0.52x problem size

### Graph_Algorithms

**pagerank**:
- Problem size range: 100-10000
- Memory range: 0.3-1526.4 MB
- Memory scaling factor: 5175.34x
- Memory growth rate: 51.75x problem size

## Detailed Memory Profiles

### matrix_solve_neumann
- **Algorithm**: neumann
- **Domain**: linear_systems
- **Problem size**: 10,000
- **Duration**: 0.216 seconds
- **Peak memory**: 1.1 MB
- **Memory efficiency**: 0.000111 MB per unit
- **Memory leak detected**: Yes
- **Snapshots taken**: 4

### matrix_solve_random-walk
- **Algorithm**: random-walk
- **Domain**: linear_systems
- **Problem size**: 10,000
- **Duration**: 0.213 seconds
- **Peak memory**: 0.3 MB
- **Memory efficiency**: 0.000029 MB per unit
- **Memory leak detected**: No
- **Snapshots taken**: 4

### matrix_solve_forward-push
- **Algorithm**: forward-push
- **Domain**: linear_systems
- **Problem size**: 10,000
- **Duration**: 0.217 seconds
- **Peak memory**: 0.3 MB
- **Memory efficiency**: 0.000029 MB per unit
- **Memory leak detected**: No
- **Snapshots taken**: 4

### matrix_solve_neumann
- **Algorithm**: neumann
- **Domain**: linear_systems
- **Problem size**: 250,000
- **Duration**: 0.210 seconds
- **Peak memory**: 5.7 MB
- **Memory efficiency**: 0.000023 MB per unit
- **Memory leak detected**: No
- **Snapshots taken**: 4

### matrix_solve_random-walk
- **Algorithm**: random-walk
- **Domain**: linear_systems
- **Problem size**: 250,000
- **Duration**: 0.228 seconds
- **Peak memory**: 3.9 MB
- **Memory efficiency**: 0.000016 MB per unit
- **Memory leak detected**: No
- **Snapshots taken**: 4

### matrix_solve_forward-push
- **Algorithm**: forward-push
- **Domain**: linear_systems
- **Problem size**: 250,000
- **Duration**: 0.229 seconds
- **Peak memory**: 3.9 MB
- **Memory efficiency**: 0.000016 MB per unit
- **Memory leak detected**: No
- **Snapshots taken**: 4

### matrix_solve_neumann
- **Algorithm**: neumann
- **Domain**: linear_systems
- **Problem size**: 1,000,000
- **Duration**: 0.325 seconds
- **Peak memory**: 22.9 MB
- **Memory efficiency**: 0.000023 MB per unit
- **Memory leak detected**: No
- **Snapshots taken**: 5

### matrix_solve_random-walk
- **Algorithm**: random-walk
- **Domain**: linear_systems
- **Problem size**: 1,000,000
- **Duration**: 0.215 seconds
- **Peak memory**: 15.3 MB
- **Memory efficiency**: 0.000015 MB per unit
- **Memory leak detected**: No
- **Snapshots taken**: 4

### matrix_solve_forward-push
- **Algorithm**: forward-push
- **Domain**: linear_systems
- **Problem size**: 1,000,000
- **Duration**: 0.315 seconds
- **Peak memory**: 15.3 MB
- **Memory efficiency**: 0.000015 MB per unit
- **Memory leak detected**: No
- **Snapshots taken**: 5

### matrix_solve_neumann
- **Algorithm**: neumann
- **Domain**: linear_systems
- **Problem size**: 4,000,000
- **Duration**: 0.569 seconds
- **Peak memory**: 91.8 MB
- **Memory efficiency**: 0.000023 MB per unit
- **Memory leak detected**: No
- **Snapshots taken**: 7

### matrix_solve_random-walk
- **Algorithm**: random-walk
- **Domain**: linear_systems
- **Problem size**: 4,000,000
- **Duration**: 0.323 seconds
- **Peak memory**: 61.1 MB
- **Memory efficiency**: 0.000015 MB per unit
- **Memory leak detected**: No
- **Snapshots taken**: 5

### matrix_solve_forward-push
- **Algorithm**: forward-push
- **Domain**: linear_systems
- **Problem size**: 4,000,000
- **Duration**: 0.436 seconds
- **Peak memory**: 61.1 MB
- **Memory efficiency**: 0.000015 MB per unit
- **Memory leak detected**: No
- **Snapshots taken**: 6

### matrix_solve_neumann
- **Algorithm**: neumann
- **Domain**: linear_systems
- **Problem size**: 25,000,000
- **Duration**: 2.579 seconds
- **Peak memory**: 572.5 MB
- **Memory efficiency**: 0.000023 MB per unit
- **Memory leak detected**: No
- **Snapshots taken**: 25

### matrix_solve_random-walk
- **Algorithm**: random-walk
- **Domain**: linear_systems
- **Problem size**: 25,000,000
- **Duration**: 1.512 seconds
- **Peak memory**: 381.7 MB
- **Memory efficiency**: 0.000015 MB per unit
- **Memory leak detected**: No
- **Snapshots taken**: 16

### matrix_solve_forward-push
- **Algorithm**: forward-push
- **Domain**: linear_systems
- **Problem size**: 25,000,000
- **Duration**: 1.754 seconds
- **Peak memory**: 381.7 MB
- **Memory efficiency**: 0.000015 MB per unit
- **Memory leak detected**: No
- **Snapshots taken**: 18

### graph_pagerank
- **Algorithm**: pagerank
- **Domain**: graph_algorithms
- **Problem size**: 100
- **Duration**: 0.532 seconds
- **Peak memory**: 0.3 MB
- **Memory efficiency**: 0.002949 MB per unit
- **Memory leak detected**: No
- **Snapshots taken**: 7

### graph_pagerank
- **Algorithm**: pagerank
- **Domain**: graph_algorithms
- **Problem size**: 500
- **Duration**: 0.983 seconds
- **Peak memory**: 3.9 MB
- **Memory efficiency**: 0.007773 MB per unit
- **Memory leak detected**: No
- **Snapshots taken**: 11

### graph_pagerank
- **Algorithm**: pagerank
- **Domain**: graph_algorithms
- **Problem size**: 1,000
- **Duration**: 1.980 seconds
- **Peak memory**: 15.5 MB
- **Memory efficiency**: 0.015509 MB per unit
- **Memory leak detected**: Yes
- **Snapshots taken**: 18

### graph_pagerank
- **Algorithm**: pagerank
- **Domain**: graph_algorithms
- **Problem size**: 2,000
- **Duration**: 2.973 seconds
- **Peak memory**: 61.3 MB
- **Memory efficiency**: 0.030659 MB per unit
- **Memory leak detected**: No
- **Snapshots taken**: 28

### graph_pagerank
- **Algorithm**: pagerank
- **Domain**: graph_algorithms
- **Problem size**: 5,000
- **Duration**: 7.601 seconds
- **Peak memory**: 381.9 MB
- **Memory efficiency**: 0.076371 MB per unit
- **Memory leak detected**: No
- **Snapshots taken**: 67

### graph_pagerank
- **Algorithm**: pagerank
- **Domain**: graph_algorithms
- **Problem size**: 10,000
- **Duration**: 16.999 seconds
- **Peak memory**: 1526.4 MB
- **Memory efficiency**: 0.152644 MB per unit
- **Memory leak detected**: No
- **Snapshots taken**: 139

