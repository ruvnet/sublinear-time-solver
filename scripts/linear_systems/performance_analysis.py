"""
Performance Analysis and Visualization Tools
Analyzes benchmark results and creates visualizations comparing solver performance.
"""

import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import json
from pathlib import Path
from typing import Dict, Any, List, Optional, Tuple
import warnings

# Suppress warnings for cleaner output
warnings.filterwarnings('ignore')

class PerformanceAnalyzer:
    """
    Analyze and visualize linear solver benchmark results.
    """

    def __init__(self, results_dir: str = "benchmark_results"):
        """
        Initialize performance analyzer.

        Args:
            results_dir: Directory containing benchmark results
        """
        self.results_dir = Path(results_dir)
        self.results_dir.mkdir(exist_ok=True)

        # Set up plotting style
        plt.style.use('default')
        sns.set_palette("husl")

    def load_results(self, filename: str) -> Dict[str, Any]:
        """
        Load benchmark results from JSON file.

        Args:
            filename: Path to results file

        Returns:
            Loaded results dictionary
        """
        filepath = self.results_dir / filename if not Path(filename).is_absolute() else Path(filename)

        with open(filepath, 'r') as f:
            results = json.load(f)

        return results

    def extract_performance_data(self, results: Dict[str, Any]) -> pd.DataFrame:
        """
        Extract performance data into a structured DataFrame.

        Args:
            results: Benchmark results dictionary

        Returns:
            DataFrame with performance metrics
        """
        data_rows = []

        def extract_recursive(data, context):
            """Recursively extract solver results."""
            if isinstance(data, dict):
                # Check if this is a solver result
                if all(key in data for key in ["time", "method"]):
                    row = {
                        "matrix_type": context.get("matrix_type", "unknown"),
                        "matrix_size": context.get("size", 0),
                        "solver_category": context.get("solver_category", "unknown"),
                        "method": data["method"],
                        "time": data["time"],
                        "success": data.get("success", False),
                        "residual": data.get("residual", float('inf')),
                        "iterations": data.get("iterations", 0),
                        "memory_efficient": data.get("memory_efficient", False)
                    }

                    # Add matrix properties if available
                    if "matrix_properties" in context:
                        props = context["matrix_properties"]
                        row.update({
                            "condition_number": props.get("condition_number", None),
                            "is_symmetric": props.get("is_symmetric", None),
                            "density": props.get("density", None)
                        })

                    data_rows.append(row)

                else:
                    # Recurse into nested structure
                    for key, value in data.items():
                        new_context = context.copy()

                        # Update context based on key
                        if key in ["traditional", "iterative", "sublinear"]:
                            new_context["solver_category"] = key
                        elif key.startswith("n_"):
                            new_context["size"] = int(key.split("_")[1])
                        elif "matrix_name" in data:
                            new_context["matrix_type"] = data["matrix_name"]
                        elif "matrix_properties" in data:
                            new_context["matrix_properties"] = data["matrix_properties"]

                        extract_recursive(value, new_context)

            elif isinstance(data, list):
                for item in data:
                    extract_recursive(item, context)

        # Extract from different sections
        sections = ["scaling_analysis", "matrix_comparison", "condition_analysis"]
        for section in sections:
            if section in results:
                extract_recursive(results[section], {"section": section})

        return pd.DataFrame(data_rows)

    def create_scaling_plots(self, df: pd.DataFrame, save_dir: Optional[str] = None) -> List[str]:
        """
        Create scaling analysis plots.

        Args:
            df: Performance data DataFrame
            save_dir: Directory to save plots

        Returns:
            List of saved plot filenames
        """
        if save_dir is None:
            save_dir = self.results_dir / "plots"
        save_dir = Path(save_dir)
        save_dir.mkdir(exist_ok=True)

        saved_plots = []

        # Filter successful results
        df_success = df[df["success"] == True].copy()

        if df_success.empty:
            print("No successful results to plot")
            return saved_plots

        # Plot 1: Time vs Matrix Size by Solver Category
        plt.figure(figsize=(12, 8))

        for solver_cat in df_success["solver_category"].unique():
            cat_data = df_success[df_success["solver_category"] == solver_cat]

            if not cat_data.empty:
                # Aggregate by size (take median time for each size)
                size_times = cat_data.groupby("matrix_size")["time"].agg(["median", "min", "max"]).reset_index()

                plt.plot(size_times["matrix_size"], size_times["median"],
                        marker='o', label=f"{solver_cat.title()} (median)",
                        linewidth=2, markersize=6)

                # Add error bars showing min-max range
                plt.fill_between(size_times["matrix_size"], size_times["min"], size_times["max"],
                               alpha=0.2)

        plt.xlabel("Matrix Size (n)")
        plt.ylabel("Solution Time (seconds)")
        plt.title("Scaling Performance: Time vs Matrix Size")
        plt.legend()
        plt.grid(True, alpha=0.3)
        plt.yscale('log')
        plt.xscale('log')

        plot_file = save_dir / "scaling_time_vs_size.png"
        plt.tight_layout()
        plt.savefig(plot_file, dpi=300, bbox_inches='tight')
        plt.close()
        saved_plots.append(str(plot_file))

        # Plot 2: Performance by Matrix Type
        if "matrix_type" in df_success.columns:
            matrix_types = df_success["matrix_type"].unique()

            if len(matrix_types) > 1:
                plt.figure(figsize=(14, 8))

                # Create box plot for each matrix type
                solver_categories = df_success["solver_category"].unique()
                n_cats = len(solver_categories)
                n_types = len(matrix_types)

                for i, matrix_type in enumerate(matrix_types):
                    type_data = df_success[df_success["matrix_type"] == matrix_type]

                    for j, solver_cat in enumerate(solver_categories):
                        cat_data = type_data[type_data["solver_category"] == solver_cat]

                        if not cat_data.empty:
                            x_pos = i * (n_cats + 1) + j
                            times = cat_data["time"].values

                            plt.boxplot(times, positions=[x_pos], widths=0.6,
                                      patch_artist=True, labels=[f"{solver_cat}"])

                # Customize plot
                plt.xlabel("Matrix Type")
                plt.ylabel("Solution Time (seconds)")
                plt.title("Performance by Matrix Type and Solver Category")
                plt.yscale('log')

                # Set x-axis labels
                x_positions = [i * (n_cats + 1) + (n_cats - 1) / 2 for i in range(n_types)]
                plt.xticks(x_positions, matrix_types, rotation=45)

                plot_file = save_dir / "performance_by_matrix_type.png"
                plt.tight_layout()
                plt.savefig(plot_file, dpi=300, bbox_inches='tight')
                plt.close()
                saved_plots.append(str(plot_file))

        # Plot 3: Speedup Analysis
        if len(df_success["solver_category"].unique()) > 1:
            plt.figure(figsize=(12, 8))

            # Calculate speedup relative to traditional methods
            speedup_data = []

            for size in df_success["matrix_size"].unique():
                size_data = df_success[df_success["matrix_size"] == size]

                # Get baseline (traditional methods)
                traditional_data = size_data[size_data["solver_category"] == "traditional"]
                if not traditional_data.empty:
                    baseline_time = traditional_data["time"].median()

                    # Calculate speedup for other categories
                    for solver_cat in ["iterative", "sublinear"]:
                        cat_data = size_data[size_data["solver_category"] == solver_cat]
                        if not cat_data.empty:
                            cat_time = cat_data["time"].median()
                            speedup = baseline_time / cat_time

                            speedup_data.append({
                                "size": size,
                                "solver_category": solver_cat,
                                "speedup": speedup
                            })

            if speedup_data:
                speedup_df = pd.DataFrame(speedup_data)

                for solver_cat in speedup_df["solver_category"].unique():
                    cat_data = speedup_df[speedup_df["solver_category"] == solver_cat]
                    plt.plot(cat_data["size"], cat_data["speedup"],
                           marker='o', label=f"{solver_cat.title()} vs Traditional",
                           linewidth=2, markersize=6)

                plt.xlabel("Matrix Size (n)")
                plt.ylabel("Speedup Factor")
                plt.title("Speedup Analysis (Relative to Traditional Methods)")
                plt.legend()
                plt.grid(True, alpha=0.3)
                plt.xscale('log')
                plt.axhline(y=1, color='red', linestyle='--', alpha=0.5, label='Baseline')

                plot_file = save_dir / "speedup_analysis.png"
                plt.tight_layout()
                plt.savefig(plot_file, dpi=300, bbox_inches='tight')
                plt.close()
                saved_plots.append(str(plot_file))

        return saved_plots

    def create_accuracy_plots(self, df: pd.DataFrame, save_dir: Optional[str] = None) -> List[str]:
        """
        Create accuracy analysis plots.

        Args:
            df: Performance data DataFrame
            save_dir: Directory to save plots

        Returns:
            List of saved plot filenames
        """
        if save_dir is None:
            save_dir = self.results_dir / "plots"
        save_dir = Path(save_dir)
        save_dir.mkdir(exist_ok=True)

        saved_plots = []

        # Filter for finite residuals
        df_finite = df[np.isfinite(df["residual"])].copy()

        if df_finite.empty:
            print("No finite residual data to plot")
            return saved_plots

        # Plot 1: Residual vs Matrix Size
        plt.figure(figsize=(12, 8))

        for solver_cat in df_finite["solver_category"].unique():
            cat_data = df_finite[df_finite["solver_category"] == solver_cat]

            if not cat_data.empty:
                # Aggregate by size
                size_residuals = cat_data.groupby("matrix_size")["residual"].median().reset_index()

                plt.plot(size_residuals["matrix_size"], size_residuals["residual"],
                        marker='o', label=f"{solver_cat.title()}",
                        linewidth=2, markersize=6)

        plt.xlabel("Matrix Size (n)")
        plt.ylabel("Final Residual ||Ax - b||")
        plt.title("Solution Accuracy vs Matrix Size")
        plt.legend()
        plt.grid(True, alpha=0.3)
        plt.yscale('log')
        plt.xscale('log')

        plot_file = save_dir / "accuracy_vs_size.png"
        plt.tight_layout()
        plt.savefig(plot_file, dpi=300, bbox_inches='tight')
        plt.close()
        saved_plots.append(str(plot_file))

        # Plot 2: Time vs Accuracy Trade-off
        plt.figure(figsize=(12, 8))

        colors = ['blue', 'red', 'green', 'orange', 'purple']
        for i, solver_cat in enumerate(df_finite["solver_category"].unique()):
            cat_data = df_finite[df_finite["solver_category"] == solver_cat]

            if not cat_data.empty:
                plt.scatter(cat_data["time"], cat_data["residual"],
                          alpha=0.6, label=f"{solver_cat.title()}",
                          color=colors[i % len(colors)], s=50)

        plt.xlabel("Solution Time (seconds)")
        plt.ylabel("Final Residual ||Ax - b||")
        plt.title("Time vs Accuracy Trade-off")
        plt.legend()
        plt.grid(True, alpha=0.3)
        plt.yscale('log')
        plt.xscale('log')

        plot_file = save_dir / "time_vs_accuracy.png"
        plt.tight_layout()
        plt.savefig(plot_file, dpi=300, bbox_inches='tight')
        plt.close()
        saved_plots.append(str(plot_file))

        return saved_plots

    def generate_performance_heatmap(self, df: pd.DataFrame, save_dir: Optional[str] = None) -> str:
        """
        Generate performance heatmap showing relative performance across methods and matrix types.

        Args:
            df: Performance data DataFrame
            save_dir: Directory to save plot

        Returns:
            Path to saved heatmap
        """
        if save_dir is None:
            save_dir = self.results_dir / "plots"
        save_dir = Path(save_dir)
        save_dir.mkdir(exist_ok=True)

        # Filter successful results
        df_success = df[df["success"] == True].copy()

        if df_success.empty:
            print("No successful results for heatmap")
            return ""

        # Create pivot table with median times
        if "matrix_type" in df_success.columns:
            pivot_data = df_success.groupby(["matrix_type", "method"])["time"].median().unstack(fill_value=np.nan)

            # Create heatmap
            plt.figure(figsize=(16, 10))
            sns.heatmap(pivot_data, annot=True, fmt='.4f', cmap='RdYlBu_r',
                       cbar_kws={'label': 'Solution Time (seconds)'})

            plt.title("Performance Heatmap: Median Solution Time by Method and Matrix Type")
            plt.xlabel("Solution Method")
            plt.ylabel("Matrix Type")
            plt.xticks(rotation=45, ha='right')
            plt.yticks(rotation=0)

            plot_file = save_dir / "performance_heatmap.png"
            plt.tight_layout()
            plt.savefig(plot_file, dpi=300, bbox_inches='tight')
            plt.close()

            return str(plot_file)

        return ""

    def analyze_sublinear_advantage(self, df: pd.DataFrame) -> Dict[str, Any]:
        """
        Analyze where sublinear methods show advantage.

        Args:
            df: Performance data DataFrame

        Returns:
            Analysis results dictionary
        """
        analysis = {
            "overall_summary": {},
            "by_matrix_type": {},
            "by_size": {},
            "recommendations": []
        }

        # Filter successful results
        df_success = df[df["success"] == True].copy()

        if df_success.empty:
            return {"error": "No successful results to analyze"}

        # Overall performance comparison
        category_stats = df_success.groupby("solver_category")["time"].agg(["mean", "median", "std", "count"])
        analysis["overall_summary"] = category_stats.to_dict()

        # Compare sublinear vs others
        sublinear_data = df_success[df_success["solver_category"] == "sublinear"]
        traditional_data = df_success[df_success["solver_category"] == "traditional"]

        if not sublinear_data.empty and not traditional_data.empty:
            sub_median = sublinear_data["time"].median()
            trad_median = traditional_data["time"].median()
            speedup = trad_median / sub_median

            analysis["overall_summary"]["sublinear_vs_traditional_speedup"] = speedup

        # Analysis by matrix type
        if "matrix_type" in df_success.columns:
            for matrix_type in df_success["matrix_type"].unique():
                type_data = df_success[df_success["matrix_type"] == matrix_type]
                type_analysis = {}

                # Performance by solver category for this matrix type
                cat_performance = type_data.groupby("solver_category")["time"].median().to_dict()
                type_analysis["median_times"] = cat_performance

                # Determine best performer
                if cat_performance:
                    best_category = min(cat_performance, key=cat_performance.get)
                    type_analysis["best_performer"] = best_category
                    type_analysis["best_time"] = cat_performance[best_category]

                    # Calculate advantage
                    if "sublinear" in cat_performance and best_category != "sublinear":
                        advantage = cat_performance["sublinear"] / cat_performance[best_category]
                        type_analysis["sublinear_disadvantage"] = advantage
                    elif "sublinear" in cat_performance:
                        # Sublinear is best, calculate advantage over second best
                        sorted_cats = sorted(cat_performance.items(), key=lambda x: x[1])
                        if len(sorted_cats) > 1:
                            second_best = sorted_cats[1][1]
                            advantage = second_best / cat_performance["sublinear"]
                            type_analysis["sublinear_advantage"] = advantage

                analysis["by_matrix_type"][matrix_type] = type_analysis

        # Analysis by size
        for size in df_success["matrix_size"].unique():
            size_data = df_success[df_success["matrix_size"] == size]
            size_analysis = {}

            cat_performance = size_data.groupby("solver_category")["time"].median().to_dict()
            size_analysis["median_times"] = cat_performance

            if cat_performance:
                best_category = min(cat_performance, key=cat_performance.get)
                size_analysis["best_performer"] = best_category

            analysis["by_size"][f"size_{size}"] = size_analysis

        # Generate recommendations
        recommendations = []

        # Check where sublinear methods excel
        sublinear_best_types = []
        for matrix_type, data in analysis["by_matrix_type"].items():
            if data.get("best_performer") == "sublinear":
                sublinear_best_types.append(matrix_type)

        if sublinear_best_types:
            recommendations.append(f"Sublinear methods are best for: {', '.join(sublinear_best_types)}")

        # Check diagonal dominance correlation
        dd_performance = {}
        for matrix_type, data in analysis["by_matrix_type"].items():
            if "dd" in matrix_type.lower() or "diagonal" in matrix_type.lower():
                if "sublinear_advantage" in data:
                    dd_performance[matrix_type] = data["sublinear_advantage"]

        if dd_performance:
            avg_advantage = np.mean(list(dd_performance.values()))
            if avg_advantage > 1.5:
                recommendations.append(f"Sublinear methods show {avg_advantage:.1f}x average speedup on diagonally dominant matrices")

        analysis["recommendations"] = recommendations

        return analysis

    def create_comprehensive_report(self, results_file: str) -> str:
        """
        Create comprehensive analysis report with plots and statistics.

        Args:
            results_file: Path to benchmark results JSON file

        Returns:
            Path to generated report
        """
        print("Creating comprehensive performance analysis report...")

        # Load results
        results = self.load_results(results_file)

        # Extract performance data
        df = self.extract_performance_data(results)

        if df.empty:
            print("No performance data found in results")
            return ""

        # Create plots directory
        plots_dir = self.results_dir / "plots"
        plots_dir.mkdir(exist_ok=True)

        # Generate all plots
        scaling_plots = self.create_scaling_plots(df, plots_dir)
        accuracy_plots = self.create_accuracy_plots(df, plots_dir)
        heatmap_plot = self.generate_performance_heatmap(df, plots_dir)

        # Generate analysis
        sublinear_analysis = self.analyze_sublinear_advantage(df)

        # Create comprehensive report
        report_lines = []
        report_lines.append("=" * 80)
        report_lines.append("COMPREHENSIVE LINEAR SOLVER PERFORMANCE ANALYSIS")
        report_lines.append("=" * 80)
        report_lines.append("")

        # Data summary
        report_lines.append("DATA SUMMARY")
        report_lines.append("-" * 20)
        report_lines.append(f"Total test cases: {len(df)}")
        report_lines.append(f"Successful cases: {len(df[df['success'] == True])}")
        report_lines.append(f"Success rate: {len(df[df['success'] == True]) / len(df) * 100:.1f}%")
        report_lines.append(f"Matrix sizes tested: {sorted(df['matrix_size'].unique())}")
        report_lines.append(f"Solver categories: {list(df['solver_category'].unique())}")
        report_lines.append("")

        # Performance summary
        report_lines.append("PERFORMANCE SUMMARY")
        report_lines.append("-" * 30)

        if "overall_summary" in sublinear_analysis:
            for category, stats in sublinear_analysis["overall_summary"].items():
                if isinstance(stats, dict) and "median" in stats:
                    report_lines.append(f"{category.title()}: {stats['median']:.4f}s median, {stats['count']} tests")

        report_lines.append("")

        # Sublinear analysis
        if "recommendations" in sublinear_analysis:
            report_lines.append("SUBLINEAR METHOD ANALYSIS")
            report_lines.append("-" * 40)
            for rec in sublinear_analysis["recommendations"]:
                report_lines.append(f"• {rec}")
            report_lines.append("")

        # Matrix type analysis
        if "by_matrix_type" in sublinear_analysis:
            report_lines.append("PERFORMANCE BY MATRIX TYPE")
            report_lines.append("-" * 40)

            for matrix_type, data in sublinear_analysis["by_matrix_type"].items():
                report_lines.append(f"\n{matrix_type.upper()}:")
                if "best_performer" in data:
                    report_lines.append(f"  Best performer: {data['best_performer']}")
                if "median_times" in data:
                    for cat, time_val in data["median_times"].items():
                        report_lines.append(f"  {cat}: {time_val:.4f}s")

        # Generated files
        report_lines.append("\n\nGENERATED FILES")
        report_lines.append("-" * 20)
        all_plots = scaling_plots + accuracy_plots
        if heatmap_plot:
            all_plots.append(heatmap_plot)

        for plot in all_plots:
            report_lines.append(f"• {Path(plot).name}")

        report_text = "\n".join(report_lines)

        # Save report
        timestamp = Path(results_file).stem
        report_file = self.results_dir / f"comprehensive_analysis_{timestamp}.txt"

        with open(report_file, 'w') as f:
            f.write(report_text)

        print(f"Comprehensive report saved to: {report_file}")
        print(f"Plots saved to: {plots_dir}")

        return str(report_file)

def main():
    """Main function for command-line usage."""
    import argparse

    parser = argparse.ArgumentParser(description="Performance Analysis for Linear Solver Benchmarks")
    parser.add_argument("results_file", help="Path to benchmark results JSON file")
    parser.add_argument("--output-dir", default="benchmark_results", help="Output directory")

    args = parser.parse_args()

    analyzer = PerformanceAnalyzer(args.output_dir)
    report_file = analyzer.create_comprehensive_report(args.results_file)

    print(f"\nAnalysis complete. Report: {report_file}")

if __name__ == "__main__":
    main()