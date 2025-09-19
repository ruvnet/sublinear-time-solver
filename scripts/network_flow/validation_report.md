# Network Flow Algorithm Validation Report

## Summary

- Total tests: 19
- Passed tests: 11
- Success rate: 57.9%
- Failed tests: algorithm_consistency, algorithm_consistency, algorithm_consistency, algorithm_consistency, algorithm_consistency, algorithm_consistency, algorithm_consistency, algorithm_consistency

## Edge Case Tests

- **single_node_edge_case**: ✅ PASS
  - Single node max flow: 0.000000

- **disconnected_network_edge_case**: ✅ PASS
  - Disconnected network max flow: 0.000000

- **zero_capacity_edge_case**: ✅ PASS
  - Zero capacity bottleneck max flow: 0.000000

## Network-Specific Tests

### 2d_grid

- **algorithm_consistency**: ❌ FAIL
  - Flow values: {'Ford-Fulkerson': 225, 'Edmonds-Karp': 225, 'Push-Relabel': 225, 'Sublinear': 129.50755553809813}, Max difference: 95.492444
  - Error magnitude: 9.55e+01

- **numerical_stability**: ✅ PASS
  - Base std: 0.000000, Perturbed std: 0.000000, Stability ratio: 0.000000
  - Error magnitude: 1.45e-09

### 3d_grid

- **algorithm_consistency**: ❌ FAIL
  - Flow values: {'Ford-Fulkerson': 225, 'Edmonds-Karp': 225, 'Push-Relabel': 225, 'Sublinear': 227.58378596754477}, Max difference: 2.583786
  - Error magnitude: 2.58e+00

- **numerical_stability**: ✅ PASS
  - Base std: 0.000000, Perturbed std: 0.000001, Stability ratio: 0.000000
  - Error magnitude: 2.62e-09

### erdos_renyi_sparse

- **algorithm_consistency**: ❌ FAIL
  - Flow values: {'Ford-Fulkerson': 11, 'Edmonds-Karp': 11, 'Push-Relabel': 11, 'Sublinear': 11.212960638296119}, Max difference: 0.212961
  - Error magnitude: 2.13e-01

- **numerical_stability**: ✅ PASS
  - Base std: 0.000000, Perturbed std: 0.000000, Stability ratio: 0.000000
  - Error magnitude: 4.22e-09

### erdos_renyi_dense

- **algorithm_consistency**: ❌ FAIL
  - Flow values: {'Ford-Fulkerson': 199, 'Edmonds-Karp': 199, 'Push-Relabel': 199, 'Sublinear': 190.7318202550042}, Max difference: 8.268180
  - Error magnitude: 8.27e+00

- **numerical_stability**: ✅ PASS
  - Base std: 0.000000, Perturbed std: 0.000000, Stability ratio: 0.000000
  - Error magnitude: 2.52e-09

### sparse_random

- **algorithm_consistency**: ❌ FAIL
  - Flow values: {'Ford-Fulkerson': 182, 'Edmonds-Karp': 182, 'Push-Relabel': 182, 'Sublinear': 184.8568369454918}, Max difference: 2.856837
  - Error magnitude: 2.86e+00

- **numerical_stability**: ✅ PASS
  - Base std: 0.000000, Perturbed std: 0.000000, Stability ratio: 0.000000
  - Error magnitude: 1.98e-09

### barabasi_albert

- **algorithm_consistency**: ❌ FAIL
  - Flow values: {'Ford-Fulkerson': 175, 'Edmonds-Karp': 175, 'Push-Relabel': 175, 'Sublinear': 176.59133727636748}, Max difference: 1.591337
  - Error magnitude: 1.59e+00

- **numerical_stability**: ✅ PASS
  - Base std: 0.000000, Perturbed std: 0.000000, Stability ratio: 0.000000
  - Error magnitude: 2.40e-09

### tree_hierarchy

- **algorithm_consistency**: ❌ FAIL
  - Flow values: {'Ford-Fulkerson': 136, 'Edmonds-Karp': 136, 'Push-Relabel': 136, 'Sublinear': 137.55021629151094}, Max difference: 1.550216
  - Error magnitude: 1.55e+00

- **numerical_stability**: ✅ PASS
  - Base std: 0.000000, Perturbed std: 0.000000, Stability ratio: 0.000000
  - Error magnitude: 1.75e-09

### small_world

- **algorithm_consistency**: ❌ FAIL
  - Flow values: {'Ford-Fulkerson': 136, 'Edmonds-Karp': 136, 'Push-Relabel': 136, 'Sublinear': 118.95709242259053}, Max difference: 17.042908
  - Error magnitude: 1.70e+01

- **numerical_stability**: ✅ PASS
  - Base std: 0.000000, Perturbed std: 0.000000, Stability ratio: 0.000000
  - Error magnitude: 1.86e-09

