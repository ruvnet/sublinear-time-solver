# Performance Analysis Suite for Sublinear-Time Solver

This directory contains a comprehensive performance analysis and validation infrastructure for the sublinear-time-solver-mcp project.

## 🎯 Overview

The Performance Analysis Suite provides tools for:
- **Unified Benchmarking** across all solver domains
- **Scalability Testing** up to extreme problem sizes
- **Memory Profiling** with detailed usage analysis
- **Mathematical Accuracy Validation** with ground truth comparison
- **Real-time Performance Monitoring** with live dashboards

## 📁 Suite Components

### 1. `unified_benchmark.py` - Master Benchmarking Orchestrator
**Purpose**: Coordinates comprehensive performance testing across all domains

**Features**:
- Cross-domain benchmark execution (linear systems, PageRank, etc.)
- Standardized result format and reporting
- Time complexity analysis with curve fitting
- Automated scalability assessment
- Integration with all MCP solver tools

**Usage**:
```bash
# Run complete benchmark suite
python unified_benchmark.py --domain all --output-dir results

# Test specific domain
python unified_benchmark.py --domain linear --quick

# Custom output location
python unified_benchmark.py --output-dir custom_results
```

**Outputs**:
- `raw_results.json` - All benchmark data
- `scalability_analysis.json` - Complexity analysis
- `benchmark_summary.md` - Human-readable report

### 2. `scalability_tests.py` - Advanced Scalability Analysis
**Purpose**: Tests performance scaling to extreme problem sizes

**Features**:
- Progressive size testing (100 → 100,000+ elements)
- Memory limit detection and handling
- Complexity curve fitting (O(log n), O(n), O(n²))
- Performance regression detection
- Visual scaling plots generation

**Usage**:
```bash
# Full scalability analysis
python scalability_tests.py --max-matrix-size 50000 --max-graph-nodes 100000

# Quick scaling test
python scalability_tests.py --quick

# Generate performance plots
python scalability_tests.py --output-dir scaling_results
```

**Outputs**:
- `scalability_raw_results.json` - Detailed test results
- `complexity_analysis.json` - Mathematical complexity analysis
- `scalability_report.md` - Comprehensive findings
- `*_scalability.png` - Performance visualization plots

### 3. `memory_profiler.py` - Comprehensive Memory Analysis
**Purpose**: Detailed memory usage profiling and leak detection

**Features**:
- Real-time memory tracking during execution
- Memory leak detection algorithms
- Cross-domain memory efficiency analysis
- Peak memory usage monitoring
- Memory scaling pattern analysis

**Usage**:
```bash
# Full memory profiling
python memory_profiler.py --max-matrix-size 10000 --profile-sparse

# Matrix operations only
python memory_profiler.py --max-matrix-size 5000

# Include sparse matrix profiling
python memory_profiler.py --profile-sparse
```

**Outputs**:
- `memory_profiles.json` - Detailed profiling data
- `memory_analysis_report.md` - Analysis and recommendations
- Memory efficiency metrics and leak detection results

### 4. `accuracy_validator.py` - Mathematical Correctness Verification
**Purpose**: Validates solver accuracy against ground truth solutions

**Features**:
- Ground truth generation for known problems
- Multiple accuracy metrics (relative/absolute error, residual norms)
- Condition number analysis for numerical stability
- Convergence validation across problem types
- Statistical accuracy pattern analysis

**Usage**:
```bash
# Complete accuracy validation
python accuracy_validator.py --domain all --tolerance standard

# Linear systems only
python accuracy_validator.py --domain linear --tolerance tight

# PageRank validation
python accuracy_validator.py --domain pagerank
```

**Outputs**:
- `accuracy_results.json` - Detailed validation results
- `accuracy_validation_report.md` - Analysis and recommendations
- Mathematical correctness verification across domains

### 5. `performance_dashboard.py` - Real-time Monitoring
**Purpose**: Live performance monitoring with GUI and CLI interfaces

**Features**:
- Real-time performance visualization
- System resource monitoring
- Cross-domain performance tracking
- Export capabilities for long-term analysis
- Both graphical and command-line interfaces

**Usage**:
```bash
# GUI dashboard (requires matplotlib/tkinter)
python performance_dashboard.py --mode gui

# Command-line monitoring
python performance_dashboard.py --mode cli --duration 300

# Export monitoring data
python performance_dashboard.py --output-dir monitoring_data
```

**Outputs**:
- Real-time performance plots and statistics
- `dashboard_metrics_*.json` - Exported monitoring data
- Live system resource usage tracking

## 🚀 Quick Start Guide

### 1. Run Complete Analysis Suite
```bash
# Install dependencies (if needed)
pip install numpy scipy matplotlib psutil

# Run unified benchmark
python unified_benchmark.py

# Run scalability analysis
python scalability_tests.py --quick

# Profile memory usage
python memory_profiler.py

# Validate accuracy
python accuracy_validator.py

# Start monitoring dashboard
python performance_dashboard.py --mode gui
```

### 2. Integration with MCP Tools
The suite is designed to work with sublinear-solver-mcp tools:

```python
# Example integration in unified_benchmark.py
import mcp_sublinear_solver

# Solve linear system using MCP
result = mcp_sublinear_solver.solve({
    "matrix": matrix_data,
    "vector": vector_data,
    "method": "neumann",
    "epsilon": 1e-6
})
```

### 3. Automated Testing Pipeline
```bash
#!/bin/bash
# Run complete validation pipeline

echo "Starting performance analysis pipeline..."

# 1. Unified benchmarks
python unified_benchmark.py --domain all --output-dir pipeline_results/benchmarks

# 2. Scalability testing
python scalability_tests.py --output-dir pipeline_results/scalability

# 3. Memory profiling
python memory_profiler.py --output-dir pipeline_results/memory

# 4. Accuracy validation
python accuracy_validator.py --domain all --output-dir pipeline_results/accuracy

echo "Pipeline completed. Results in pipeline_results/"
```

## 📊 Result Interpretation

### Benchmark Results
- **Execution Time**: Look for sublinear scaling (O(log n))
- **Memory Usage**: Should scale efficiently with problem size
- **Success Rate**: Target >95% for production readiness

### Scalability Analysis
- **Time Complexity**: Confirm sublinear claims (O(log n) preferred)
- **Memory Scaling**: Linear or sublinear memory growth
- **Breaking Points**: Maximum problem sizes before failure

### Accuracy Validation
- **Relative Error**: Target <1e-6 for good accuracy
- **Convergence Rate**: >90% convergence expected
- **Numerical Stability**: Low condition number sensitivity

### Memory Profiling
- **Peak Usage**: Reasonable for problem size
- **Leak Detection**: Zero memory leaks for production
- **Efficiency**: MB per problem unit should be minimal

## 🎯 Performance Targets

### Sublinear-Time Claims Validation
- **Linear Systems**: O(log n) time complexity target
- **PageRank**: O(log n) convergence iterations
- **Memory Usage**: O(n) or better space complexity
- **Accuracy**: Relative error <1e-6 for well-conditioned problems

### Production Readiness Metrics
- **Success Rate**: >95%
- **Performance Consistency**: <20% variance in execution time
- **Memory Efficiency**: <1MB per 1000 problem units
- **Scalability**: Handle 100k+ problem sizes

## 🔧 Customization and Extension

### Adding New Domains
1. Create domain-specific test generator
2. Add to `unified_benchmark.py` domain list
3. Implement MCP tool integration
4. Add accuracy validation methods

### Custom Metrics
```python
# Example: Add new metric to BenchmarkResult
@dataclass
class BenchmarkResult:
    # ... existing fields ...
    custom_metric: float = 0.0

# Update analysis functions accordingly
```

### Integration with CI/CD
```yaml
# Example GitHub Actions integration
- name: Run Performance Analysis
  run: |
    python scripts/performance/unified_benchmark.py --quick
    python scripts/performance/accuracy_validator.py
```

## 📈 Continuous Monitoring

### Automated Performance Regression Detection
```python
# Example regression detection
def detect_regression(current_results, baseline_results):
    """Detect performance regressions"""
    for metric in ['execution_time', 'memory_usage']:
        if current_results[metric] > baseline_results[metric] * 1.2:
            alert(f"Performance regression in {metric}")
```

### Long-term Trend Analysis
- Set up scheduled benchmark runs
- Track performance trends over time
- Alert on degradation patterns
- Maintain performance history database

## 🤝 Contributing

To add new performance tests:
1. Follow the standardized result format (`BenchmarkResult`, `MemoryProfile`, etc.)
2. Integrate with existing analysis pipelines
3. Add comprehensive documentation
4. Include example usage and expected outputs

## 📞 Support

For issues with the performance analysis suite:
1. Check the individual script help: `python script_name.py --help`
2. Review the generated reports for diagnostic information
3. Enable debug logging for detailed execution traces
4. Ensure all dependencies are properly installed

---

**Note**: This suite is designed to provide comprehensive validation of sublinear-time solver claims across all domains. Results should be used to verify algorithmic complexity, ensure numerical accuracy, and validate production readiness.