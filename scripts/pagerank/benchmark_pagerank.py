#!/usr/bin/env python3
"""
Comprehensive PageRank benchmarking framework.
Compares traditional Python implementations vs sublinear-solver MCP tools.
"""

import json
import time
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from typing import Dict, List, Tuple, Optional
import os
import sys
from pathlib import Path
import argparse
import traceback
from datetime import datetime

# Import our implementations
from traditional_pagerank import TraditionalPageRank, compare_pagerank_methods
from sublinear_pagerank import SublinearPageRank, MCPPageRankActual
from generate_test_graphs import GraphGenerator


class PageRankBenchmark:
    """Comprehensive PageRank benchmarking suite."""

    def __init__(self, output_dir: str = "/workspaces/sublinear-time-solver/scripts/pagerank/results"):
        self.output_dir = output_dir
        os.makedirs(output_dir, exist_ok=True)

        # Initialize PageRank implementations
        self.traditional_pr = TraditionalPageRank()
        self.sublinear_pr = SublinearPageRank()
        self.mcp_pr = MCPPageRankActual()

        # Benchmark results storage
        self.results = {
            'timestamp': datetime.now().isoformat(),
            'benchmarks': [],
            'summary': {},
            'errors': []
        }

    def load_test_graphs(self, test_graphs_file: str) -> Dict:
        """Load test graphs from JSON file."""
        try:
            with open(test_graphs_file, 'r') as f:
                return json.load(f)
        except FileNotFoundError:
            print(f"Test graphs file not found: {test_graphs_file}")
            print("Generating test graphs...")
            from generate_test_graphs import generate_test_suite
            test_graphs, _ = generate_test_suite()
            return test_graphs
        except Exception as e:
            print(f"Error loading test graphs: {e}")
            return {}

    def benchmark_single_graph(self, graph_name: str, graph_data: Dict) -> Dict:
        """Benchmark all PageRank implementations on a single graph."""
        print(f"\nBenchmarking graph: {graph_name}")
        print(f"  Nodes: {graph_data['num_nodes']}, Edges: {graph_data['num_edges']}")
        print(f"  Category: {graph_data['category']}, Density: {graph_data['density']:.4f}")

        adjacency_matrix = np.array(graph_data['adjacency_matrix'])
        benchmark_result = {
            'graph_name': graph_name,
            'graph_info': {
                'num_nodes': graph_data['num_nodes'],
                'num_edges': graph_data['num_edges'],
                'density': graph_data['density'],
                'category': graph_data['category']
            },
            'results': {},
            'accuracy_comparison': {},
            'performance_analysis': {}
        }

        # Traditional methods
        print("  Running traditional methods...")
        try:
            import networkx as nx
            nx_graph = nx.from_numpy_array(adjacency_matrix, create_using=nx.DiGraph())
            traditional_results = compare_pagerank_methods(adjacency_matrix, nx_graph)

            for method, result in traditional_results.items():
                if 'error' not in result:
                    benchmark_result['results'][method] = {
                        'metrics': result['metrics'],
                        'success': True
                    }
                    # Store scores for accuracy comparison
                    if isinstance(result['scores'], dict):
                        # NetworkX format
                        scores_array = np.array([result['scores'][i] for i in sorted(result['scores'].keys())])
                    else:
                        # NumPy array format
                        scores_array = result['scores']
                    benchmark_result['accuracy_comparison'][method] = scores_array.tolist()
                else:
                    benchmark_result['results'][method] = {
                        'error': result['error'],
                        'success': False
                    }
                    print(f"    {method} failed: {result['error']}")

        except Exception as e:
            error_msg = f"Traditional methods failed: {str(e)}"
            print(f"  {error_msg}")
            self.results['errors'].append({
                'graph': graph_name,
                'method': 'traditional',
                'error': error_msg
            })

        # Sublinear methods
        print("  Running sublinear methods...")
        try:
            sublinear_results = self.sublinear_pr.compare_formats(adjacency_matrix)

            for method, result in sublinear_results.items():
                if 'error' not in result:
                    benchmark_result['results'][method] = {
                        'metrics': result['metrics'],
                        'success': True
                    }
                    # Store scores for accuracy comparison
                    scores_array = np.array(result['result']['pageRankVector'])
                    benchmark_result['accuracy_comparison'][method] = scores_array.tolist()
                else:
                    benchmark_result['results'][method] = {
                        'error': result['error'],
                        'success': False
                    }
                    print(f"    {method} failed: {result['error']}")

        except Exception as e:
            error_msg = f"Sublinear methods failed: {str(e)}"
            print(f"  {error_msg}")
            self.results['errors'].append({
                'graph': graph_name,
                'method': 'sublinear',
                'error': error_msg
            })

        # MCP actual method
        print("  Running MCP actual method...")
        try:
            mcp_result, mcp_metrics = self.mcp_pr.pagerank(adjacency_matrix)
            benchmark_result['results']['mcp_actual'] = {
                'metrics': mcp_metrics,
                'success': True
            }
            # Store scores for accuracy comparison
            scores_array = np.array(mcp_result['pageRankVector'])
            benchmark_result['accuracy_comparison']['mcp_actual'] = scores_array.tolist()

        except Exception as e:
            error_msg = f"MCP actual method failed: {str(e)}"
            print(f"  {error_msg}")
            benchmark_result['results']['mcp_actual'] = {
                'error': error_msg,
                'success': False
            }
            self.results['errors'].append({
                'graph': graph_name,
                'method': 'mcp_actual',
                'error': error_msg
            })

        # Analyze accuracy and performance
        self._analyze_benchmark_result(benchmark_result)

        return benchmark_result

    def _analyze_benchmark_result(self, benchmark_result: Dict):
        """Analyze accuracy and performance for a single benchmark result."""
        accuracy_data = benchmark_result['accuracy_comparison']
        results_data = benchmark_result['results']

        # Accuracy analysis
        if len(accuracy_data) >= 2:
            methods = list(accuracy_data.keys())
            reference_method = 'networkx' if 'networkx' in methods else methods[0]
            reference_scores = np.array(accuracy_data[reference_method])

            accuracy_analysis = {
                'reference_method': reference_method,
                'method_comparisons': {}
            }

            for method, scores in accuracy_data.items():
                if method != reference_method:
                    scores_array = np.array(scores)
                    if len(scores_array) == len(reference_scores):
                        # Calculate various accuracy metrics
                        mse = np.mean((scores_array - reference_scores) ** 2)
                        mae = np.mean(np.abs(scores_array - reference_scores))
                        max_error = np.max(np.abs(scores_array - reference_scores))
                        correlation = np.corrcoef(scores_array, reference_scores)[0, 1]

                        # Rank correlation (Spearman)
                        from scipy.stats import spearmanr
                        rank_correlation, _ = spearmanr(scores_array, reference_scores)

                        accuracy_analysis['method_comparisons'][method] = {
                            'mse': float(mse),
                            'mae': float(mae),
                            'max_error': float(max_error),
                            'correlation': float(correlation),
                            'rank_correlation': float(rank_correlation)
                        }

            benchmark_result['accuracy_comparison'] = accuracy_analysis

        # Performance analysis
        performance_analysis = {
            'fastest_method': None,
            'most_memory_efficient': None,
            'method_rankings': {},
            'sublinear_advantage': {}
        }

        successful_methods = {method: data for method, data in results_data.items()
                            if data.get('success', False)}

        if successful_methods:
            # Find fastest method
            execution_times = {method: data['metrics']['execution_time']
                             for method, data in successful_methods.items()}
            fastest_method = min(execution_times, key=execution_times.get)
            performance_analysis['fastest_method'] = {
                'method': fastest_method,
                'time': execution_times[fastest_method]
            }

            # Find most memory efficient method
            memory_usage = {method: data['metrics'].get('memory_usage_mb', float('inf'))
                          for method, data in successful_methods.items()}
            most_efficient = min(memory_usage, key=memory_usage.get)
            performance_analysis['most_memory_efficient'] = {
                'method': most_efficient,
                'memory_mb': memory_usage[most_efficient]
            }

            # Analyze sublinear advantage
            traditional_methods = ['networkx', 'scipy_eigenvalue', 'power_iteration', 'sparse_matrix']
            sublinear_methods = ['mcp_dense', 'mcp_sparse', 'mcp_actual']

            traditional_times = [execution_times[m] for m in traditional_methods if m in execution_times]
            sublinear_times = [execution_times[m] for m in sublinear_methods if m in execution_times]

            if traditional_times and sublinear_times:
                avg_traditional_time = np.mean(traditional_times)
                avg_sublinear_time = np.mean(sublinear_times)
                speedup = avg_traditional_time / avg_sublinear_time

                performance_analysis['sublinear_advantage'] = {
                    'avg_traditional_time': float(avg_traditional_time),
                    'avg_sublinear_time': float(avg_sublinear_time),
                    'speedup_factor': float(speedup),
                    'sublinear_wins': speedup > 1.0
                }

        benchmark_result['performance_analysis'] = performance_analysis

    def run_benchmark_suite(self, test_graphs_file: str = None, categories: List[str] = None,
                          max_graphs_per_category: int = None) -> Dict:
        """Run complete benchmark suite."""
        if test_graphs_file is None:
            test_graphs_file = "/workspaces/sublinear-time-solver/scripts/pagerank/test_graphs/test_graphs.json"

        print("Loading test graphs...")
        test_graphs = self.load_test_graphs(test_graphs_file)

        if not test_graphs:
            print("No test graphs available!")
            return self.results

        # Filter by categories if specified
        if categories:
            filtered_graphs = {name: data for name, data in test_graphs.items()
                             if data['category'] in categories}
            test_graphs = filtered_graphs

        # Limit graphs per category if specified
        if max_graphs_per_category:
            category_counts = {}
            filtered_graphs = {}
            for name, data in test_graphs.items():
                category = data['category']
                if category not in category_counts:
                    category_counts[category] = 0
                if category_counts[category] < max_graphs_per_category:
                    filtered_graphs[name] = data
                    category_counts[category] += 1
            test_graphs = filtered_graphs

        print(f"Running benchmarks on {len(test_graphs)} graphs...")

        # Run benchmarks
        for graph_name, graph_data in test_graphs.items():
            try:
                benchmark_result = self.benchmark_single_graph(graph_name, graph_data)
                self.results['benchmarks'].append(benchmark_result)
            except Exception as e:
                error_msg = f"Benchmark failed for graph {graph_name}: {str(e)}"
                print(f"ERROR: {error_msg}")
                self.results['errors'].append({
                    'graph': graph_name,
                    'method': 'benchmark',
                    'error': error_msg,
                    'traceback': traceback.format_exc()
                })

        # Generate summary
        self._generate_summary()

        # Save results
        self._save_results()

        return self.results

    def _generate_summary(self):
        """Generate benchmark summary statistics."""
        benchmarks = self.results['benchmarks']
        if not benchmarks:
            return

        summary = {
            'total_graphs': len(benchmarks),
            'categories': {},
            'method_performance': {},
            'accuracy_summary': {},
            'sublinear_analysis': {}
        }

        # Category analysis
        category_data = {}
        for benchmark in benchmarks:
            category = benchmark['graph_info']['category']
            if category not in category_data:
                category_data[category] = []
            category_data[category].append(benchmark)

        for category, benchmarks_list in category_data.items():
            summary['categories'][category] = {
                'count': len(benchmarks_list),
                'avg_nodes': np.mean([b['graph_info']['num_nodes'] for b in benchmarks_list]),
                'avg_edges': np.mean([b['graph_info']['num_edges'] for b in benchmarks_list]),
                'avg_density': np.mean([b['graph_info']['density'] for b in benchmarks_list])
            }

        # Method performance analysis
        all_methods = set()
        for benchmark in benchmarks:
            all_methods.update(benchmark['results'].keys())

        for method in all_methods:
            method_benchmarks = [b for b in benchmarks
                               if method in b['results'] and b['results'][method].get('success', False)]

            if method_benchmarks:
                execution_times = [b['results'][method]['metrics']['execution_time']
                                 for b in method_benchmarks]
                memory_usage = [b['results'][method]['metrics'].get('memory_usage_mb', 0)
                              for b in method_benchmarks]

                summary['method_performance'][method] = {
                    'success_rate': len(method_benchmarks) / len(benchmarks),
                    'avg_execution_time': np.mean(execution_times),
                    'std_execution_time': np.std(execution_times),
                    'avg_memory_usage': np.mean(memory_usage),
                    'std_memory_usage': np.std(memory_usage),
                    'benchmark_count': len(method_benchmarks)
                }

        # Sublinear analysis
        traditional_methods = ['networkx', 'scipy_eigenvalue', 'power_iteration', 'sparse_matrix']
        sublinear_methods = ['mcp_dense', 'mcp_sparse', 'mcp_actual']

        sublinear_wins = 0
        valid_comparisons = 0

        for benchmark in benchmarks:
            perf_analysis = benchmark.get('performance_analysis', {})
            if 'sublinear_advantage' in perf_analysis:
                valid_comparisons += 1
                if perf_analysis['sublinear_advantage'].get('sublinear_wins', False):
                    sublinear_wins += 1

        if valid_comparisons > 0:
            summary['sublinear_analysis'] = {
                'win_rate': sublinear_wins / valid_comparisons,
                'valid_comparisons': valid_comparisons,
                'sublinear_wins': sublinear_wins
            }

        self.results['summary'] = summary

    def _convert_numpy_types(self, obj):
        """Convert numpy types to native Python types for JSON serialization."""
        if isinstance(obj, dict):
            return {key: self._convert_numpy_types(value) for key, value in obj.items()}
        elif isinstance(obj, list):
            return [self._convert_numpy_types(item) for item in obj]
        elif isinstance(obj, np.ndarray):
            return obj.tolist()
        elif isinstance(obj, np.integer):
            return int(obj)
        elif isinstance(obj, np.floating):
            return float(obj)
        elif isinstance(obj, np.bool_):
            return bool(obj)
        else:
            return obj

    def _save_results(self):
        """Save benchmark results to files."""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

        # Save detailed results (with numpy compatibility)
        results_file = os.path.join(self.output_dir, f"benchmark_results_{timestamp}.json")
        with open(results_file, 'w') as f:
            # Convert numpy types to native Python types for JSON serialization
            json_results = self._convert_numpy_types(self.results)
            json.dump(json_results, f, indent=2)

        print(f"\nBenchmark results saved to: {results_file}")

        # Save summary CSV
        if self.results['benchmarks']:
            self._save_csv_summary(timestamp)

        # Generate plots
        try:
            self._generate_plots(timestamp)
        except Exception as e:
            print(f"Plot generation failed: {e}")

    def _save_csv_summary(self, timestamp: str):
        """Save summary as CSV for easy analysis."""
        benchmarks = self.results['benchmarks']
        data_rows = []

        for benchmark in benchmarks:
            base_row = {
                'graph_name': benchmark['graph_name'],
                'category': benchmark['graph_info']['category'],
                'num_nodes': benchmark['graph_info']['num_nodes'],
                'num_edges': benchmark['graph_info']['num_edges'],
                'density': benchmark['graph_info']['density']
            }

            for method, result in benchmark['results'].items():
                if result.get('success', False):
                    row = base_row.copy()
                    row.update({
                        'method': method,
                        'execution_time': result['metrics']['execution_time'],
                        'memory_usage_mb': result['metrics'].get('memory_usage_mb', 0),
                        'peak_memory_mb': result['metrics'].get('peak_memory_mb', 0)
                    })
                    data_rows.append(row)

        if data_rows:
            df = pd.DataFrame(data_rows)
            csv_file = os.path.join(self.output_dir, f"benchmark_summary_{timestamp}.csv")
            df.to_csv(csv_file, index=False)
            print(f"CSV summary saved to: {csv_file}")

    def _generate_plots(self, timestamp: str):
        """Generate visualization plots."""
        plt.style.use('default')
        sns.set_palette("husl")

        benchmarks = self.results['benchmarks']
        if not benchmarks:
            return

        # Prepare data
        data_rows = []
        for benchmark in benchmarks:
            for method, result in benchmark['results'].items():
                if result.get('success', False):
                    data_rows.append({
                        'graph_name': benchmark['graph_name'],
                        'category': benchmark['graph_info']['category'],
                        'num_nodes': benchmark['graph_info']['num_nodes'],
                        'method': method,
                        'execution_time': result['metrics']['execution_time'],
                        'memory_usage': result['metrics'].get('memory_usage_mb', 0)
                    })

        if not data_rows:
            return

        df = pd.DataFrame(data_rows)

        # Create figure with subplots
        fig, axes = plt.subplots(2, 2, figsize=(15, 12))
        fig.suptitle('PageRank Benchmark Results', fontsize=16)

        # 1. Execution time by method
        sns.boxplot(data=df, x='method', y='execution_time', ax=axes[0, 0])
        axes[0, 0].set_title('Execution Time by Method')
        axes[0, 0].set_ylabel('Time (seconds)')
        axes[0, 0].tick_params(axis='x', rotation=45)

        # 2. Memory usage by method
        sns.boxplot(data=df, x='method', y='memory_usage', ax=axes[0, 1])
        axes[0, 1].set_title('Memory Usage by Method')
        axes[0, 1].set_ylabel('Memory (MB)')
        axes[0, 1].tick_params(axis='x', rotation=45)

        # 3. Performance vs graph size
        sns.scatterplot(data=df, x='num_nodes', y='execution_time', hue='method', ax=axes[1, 0])
        axes[1, 0].set_title('Performance vs Graph Size')
        axes[1, 0].set_xlabel('Number of Nodes')
        axes[1, 0].set_ylabel('Execution Time (seconds)')
        axes[1, 0].legend(bbox_to_anchor=(1.05, 1), loc='upper left')

        # 4. Performance by category
        sns.boxplot(data=df, x='category', y='execution_time', ax=axes[1, 1])
        axes[1, 1].set_title('Performance by Graph Category')
        axes[1, 1].set_ylabel('Execution Time (seconds)')
        axes[1, 1].tick_params(axis='x', rotation=45)

        plt.tight_layout()
        plot_file = os.path.join(self.output_dir, f"benchmark_plots_{timestamp}.png")
        plt.savefig(plot_file, dpi=300, bbox_inches='tight')
        plt.close()

        print(f"Plots saved to: {plot_file}")


def main():
    """Main entry point for benchmarking."""
    parser = argparse.ArgumentParser(description='PageRank Benchmark Suite')
    parser.add_argument('--categories', nargs='+', help='Graph categories to benchmark')
    parser.add_argument('--max-per-category', type=int, help='Maximum graphs per category')
    parser.add_argument('--output-dir', default='/workspaces/sublinear-time-solver/scripts/pagerank/results',
                       help='Output directory for results')
    parser.add_argument('--test-graphs', help='Path to test graphs JSON file')

    args = parser.parse_args()

    # Create benchmark instance
    benchmark = PageRankBenchmark(output_dir=args.output_dir)

    # Run benchmark suite
    print("Starting PageRank Benchmark Suite")
    print("=" * 50)

    results = benchmark.run_benchmark_suite(
        test_graphs_file=args.test_graphs,
        categories=args.categories,
        max_graphs_per_category=args.max_per_category
    )

    # Print summary
    print("\n" + "=" * 50)
    print("BENCHMARK SUMMARY")
    print("=" * 50)

    summary = results.get('summary', {})
    if summary:
        print(f"Total graphs tested: {summary.get('total_graphs', 0)}")
        print(f"Categories: {list(summary.get('categories', {}).keys())}")

        method_perf = summary.get('method_performance', {})
        if method_perf:
            print("\nMethod Performance Summary:")
            for method, perf in method_perf.items():
                print(f"  {method}:")
                print(f"    Success rate: {perf['success_rate']:.2%}")
                print(f"    Avg execution time: {perf['avg_execution_time']:.4f}s")
                print(f"    Avg memory usage: {perf['avg_memory_usage']:.2f} MB")

        sublinear_analysis = summary.get('sublinear_analysis', {})
        if sublinear_analysis:
            print(f"\nSublinear Advantage:")
            print(f"  Win rate: {sublinear_analysis['win_rate']:.2%}")
            print(f"  Comparisons: {sublinear_analysis['valid_comparisons']}")

    if results.get('errors'):
        print(f"\nErrors encountered: {len(results['errors'])}")

    print(f"\nDetailed results saved to: {args.output_dir}")


if __name__ == "__main__":
    main()