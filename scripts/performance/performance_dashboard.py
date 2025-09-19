#!/usr/bin/env python3
"""
Real-time Performance Monitoring Dashboard for Sublinear-Time Solver
Live visualization and monitoring of solver performance across all domains
"""

import time
import json
import sys
import threading
import queue
import numpy as np
from pathlib import Path
from typing import Dict, List, Tuple, Optional, Any
from dataclasses import dataclass, asdict
import logging
from datetime import datetime, timedelta
import psutil

# Dashboard dependencies
try:
    import matplotlib.pyplot as plt
    import matplotlib.animation as animation
    from matplotlib.backends.backend_tkagg import FigureCanvasTkAgg
    import tkinter as tk
    from tkinter import ttk, messagebox
    DASHBOARD_AVAILABLE = True
except ImportError:
    DASHBOARD_AVAILABLE = False

# Add project root to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

@dataclass
class PerformanceMetric:
    """Single performance measurement"""
    timestamp: float
    domain: str
    algorithm: str
    problem_size: int
    execution_time: float
    memory_usage_mb: float
    cpu_percent: float
    convergence_iterations: Optional[int] = None
    accuracy_error: Optional[float] = None
    success: bool = True

@dataclass
class SystemMetric:
    """System-wide performance measurement"""
    timestamp: float
    cpu_percent: float
    memory_percent: float
    memory_available_gb: float
    active_processes: int
    solver_processes: int

class PerformanceCollector:
    """Collects performance metrics from various sources"""

    def __init__(self):
        self.metrics_queue = queue.Queue()
        self.system_queue = queue.Queue()
        self.logger = self._setup_logging()
        self._collecting = False

    def _setup_logging(self) -> logging.Logger:
        logger = logging.getLogger("PerformanceCollector")
        logger.setLevel(logging.INFO)
        handler = logging.StreamHandler()
        formatter = logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s')
        handler.setFormatter(formatter)
        logger.addHandler(handler)
        return logger

    def start_collection(self):
        """Start collecting performance metrics"""
        self._collecting = True

        # Start system metrics collection
        system_thread = threading.Thread(target=self._collect_system_metrics, daemon=True)
        system_thread.start()

        # Start solver metrics monitoring
        solver_thread = threading.Thread(target=self._monitor_solver_processes, daemon=True)
        solver_thread.start()

        self.logger.info("Performance collection started")

    def stop_collection(self):
        """Stop collecting metrics"""
        self._collecting = False
        self.logger.info("Performance collection stopped")

    def _collect_system_metrics(self):
        """Collect system-wide performance metrics"""
        while self._collecting:
            try:
                cpu_percent = psutil.cpu_percent(interval=1)
                memory = psutil.virtual_memory()
                processes = list(psutil.process_iter(['name']))

                # Count solver-related processes
                solver_processes = sum(1 for p in processes
                                     if any(keyword in p.info['name'].lower()
                                           for keyword in ['solver', 'python', 'benchmark']))

                metric = SystemMetric(
                    timestamp=time.time(),
                    cpu_percent=cpu_percent,
                    memory_percent=memory.percent,
                    memory_available_gb=memory.available / (1024**3),
                    active_processes=len(processes),
                    solver_processes=solver_processes
                )

                self.system_queue.put(metric)

            except Exception as e:
                self.logger.error(f"System metrics collection error: {str(e)}")

            time.sleep(1)  # Collect every second

    def _monitor_solver_processes(self):
        """Monitor solver-specific processes"""
        while self._collecting:
            try:
                # Look for solver result files and parse them
                self._check_result_files()

            except Exception as e:
                self.logger.error(f"Solver monitoring error: {str(e)}")

            time.sleep(5)  # Check every 5 seconds

    def _check_result_files(self):
        """Check for new solver result files"""
        # Look for benchmark result files
        result_dirs = [
            Path("benchmark_results"),
            Path("scalability_results"),
            Path("memory_profiles"),
            Path("accuracy_validation")
        ]

        for result_dir in result_dirs:
            if result_dir.exists():
                # Look for recent JSON files
                for json_file in result_dir.glob("*.json"):
                    try:
                        # Check if file was modified recently (last 10 seconds)
                        if time.time() - json_file.stat().st_mtime < 10:
                            self._parse_result_file(json_file)
                    except Exception as e:
                        self.logger.debug(f"Error parsing {json_file}: {str(e)}")

    def _parse_result_file(self, file_path: Path):
        """Parse solver result file and extract metrics"""
        try:
            with open(file_path, 'r') as f:
                data = json.load(f)

            # Handle different result file formats
            if isinstance(data, list):
                for item in data:
                    metric = self._extract_metric_from_item(item)
                    if metric:
                        self.metrics_queue.put(metric)

        except Exception as e:
            self.logger.debug(f"Failed to parse {file_path}: {str(e)}")

    def _extract_metric_from_item(self, item: Dict[str, Any]) -> Optional[PerformanceMetric]:
        """Extract performance metric from result item"""
        try:
            # Try to extract common fields
            domain = item.get('domain', 'unknown')
            algorithm = item.get('algorithm', item.get('method_used', 'unknown'))
            problem_size = item.get('problem_size', item.get('matrix_size', item.get('graph_nodes', 0)))
            execution_time = item.get('execution_time', 0)
            memory_usage = item.get('peak_memory_mb', item.get('memory_usage', 0))
            success = item.get('success', True)

            return PerformanceMetric(
                timestamp=time.time(),
                domain=domain,
                algorithm=algorithm,
                problem_size=problem_size,
                execution_time=execution_time,
                memory_usage_mb=memory_usage,
                cpu_percent=0,  # Not available from file
                success=success
            )

        except Exception:
            return None

    def add_metric(self, metric: PerformanceMetric):
        """Manually add a performance metric"""
        self.metrics_queue.put(metric)

class PerformanceDashboard:
    """Real-time performance monitoring dashboard"""

    def __init__(self, collector: PerformanceCollector):
        if not DASHBOARD_AVAILABLE:
            raise ImportError("Dashboard dependencies not available. Install matplotlib and tkinter.")

        self.collector = collector
        self.logger = self._setup_logging()

        # Data storage
        self.metrics_history: List[PerformanceMetric] = []
        self.system_history: List[SystemMetric] = []
        self.max_history = 1000  # Keep last 1000 data points

        # GUI components
        self.root = None
        self.fig = None
        self.axes = {}
        self.lines = {}

        # Animation
        self.animation = None
        self.update_interval = 1000  # 1 second

    def _setup_logging(self) -> logging.Logger:
        logger = logging.getLogger("PerformanceDashboard")
        logger.setLevel(logging.INFO)
        handler = logging.StreamHandler()
        formatter = logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s')
        handler.setFormatter(formatter)
        logger.addHandler(handler)
        return logger

    def create_gui(self):
        """Create the GUI dashboard"""
        self.root = tk.Tk()
        self.root.title("Sublinear-Time Solver Performance Dashboard")
        self.root.geometry("1200x800")

        # Create main frame
        main_frame = ttk.Frame(self.root)
        main_frame.pack(fill=tk.BOTH, expand=True, padx=5, pady=5)

        # Control panel
        control_frame = ttk.LabelFrame(main_frame, text="Controls")
        control_frame.pack(fill=tk.X, pady=(0, 5))

        ttk.Button(control_frame, text="Start Monitoring",
                  command=self.start_monitoring).pack(side=tk.LEFT, padx=5, pady=5)
        ttk.Button(control_frame, text="Stop Monitoring",
                  command=self.stop_monitoring).pack(side=tk.LEFT, padx=5, pady=5)
        ttk.Button(control_frame, text="Export Data",
                  command=self.export_data).pack(side=tk.LEFT, padx=5, pady=5)
        ttk.Button(control_frame, text="Clear Data",
                  command=self.clear_data).pack(side=tk.LEFT, padx=5, pady=5)

        # Status frame
        status_frame = ttk.LabelFrame(main_frame, text="Status")
        status_frame.pack(fill=tk.X, pady=(0, 5))

        self.status_vars = {
            'metrics_count': tk.StringVar(value="Metrics: 0"),
            'last_update': tk.StringVar(value="Last Update: Never"),
            'system_status': tk.StringVar(value="System: OK")
        }

        for i, (key, var) in enumerate(self.status_vars.items()):
            ttk.Label(status_frame, textvariable=var).grid(row=0, column=i, padx=10, pady=5)

        # Create matplotlib figure
        self.fig, self.axes = plt.subplots(2, 3, figsize=(12, 8))
        self.fig.suptitle('Sublinear-Time Solver Performance Dashboard')

        # Setup subplots
        self._setup_plots()

        # Embed matplotlib in tkinter
        canvas_frame = ttk.Frame(main_frame)
        canvas_frame.pack(fill=tk.BOTH, expand=True)

        canvas = FigureCanvasTkAgg(self.fig, canvas_frame)
        canvas.draw()
        canvas.get_tk_widget().pack(fill=tk.BOTH, expand=True)

        # Statistics panel
        stats_frame = ttk.LabelFrame(main_frame, text="Live Statistics")
        stats_frame.pack(fill=tk.X, pady=(5, 0))

        self.stats_text = tk.Text(stats_frame, height=6, wrap=tk.WORD)
        scrollbar = ttk.Scrollbar(stats_frame, orient=tk.VERTICAL, command=self.stats_text.yview)
        self.stats_text.configure(yscrollcommand=scrollbar.set)

        self.stats_text.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        scrollbar.pack(side=tk.RIGHT, fill=tk.Y)

    def _setup_plots(self):
        """Setup dashboard plots"""
        # Execution time plot
        ax1 = self.axes[0, 0]
        ax1.set_title('Execution Time by Algorithm')
        ax1.set_xlabel('Time')
        ax1.set_ylabel('Execution Time (s)')
        ax1.grid(True, alpha=0.3)

        # Memory usage plot
        ax2 = self.axes[0, 1]
        ax2.set_title('Memory Usage')
        ax2.set_xlabel('Time')
        ax2.set_ylabel('Memory (MB)')
        ax2.grid(True, alpha=0.3)

        # Problem size vs performance
        ax3 = self.axes[0, 2]
        ax3.set_title('Performance vs Problem Size')
        ax3.set_xlabel('Problem Size')
        ax3.set_ylabel('Execution Time (s)')
        ax3.set_xscale('log')
        ax3.set_yscale('log')
        ax3.grid(True, alpha=0.3)

        # System CPU usage
        ax4 = self.axes[1, 0]
        ax4.set_title('System CPU Usage')
        ax4.set_xlabel('Time')
        ax4.set_ylabel('CPU %')
        ax4.set_ylim(0, 100)
        ax4.grid(True, alpha=0.3)

        # System memory usage
        ax5 = self.axes[1, 1]
        ax5.set_title('System Memory Usage')
        ax5.set_xlabel('Time')
        ax5.set_ylabel('Memory %')
        ax5.set_ylim(0, 100)
        ax5.grid(True, alpha=0.3)

        # Success rate
        ax6 = self.axes[1, 2]
        ax6.set_title('Success Rate by Domain')
        ax6.set_xlabel('Domain')
        ax6.set_ylabel('Success Rate %')
        ax6.set_ylim(0, 100)

        plt.tight_layout()

    def start_monitoring(self):
        """Start performance monitoring"""
        self.collector.start_collection()

        # Start animation
        self.animation = animation.FuncAnimation(
            self.fig, self._update_plots, interval=self.update_interval, blit=False
        )

        self.logger.info("Dashboard monitoring started")

    def stop_monitoring(self):
        """Stop performance monitoring"""
        self.collector.stop_collection()

        if self.animation:
            self.animation.event_source.stop()

        self.logger.info("Dashboard monitoring stopped")

    def _update_plots(self, frame):
        """Update dashboard plots with new data"""
        # Collect new metrics
        self._collect_new_data()

        # Update plots
        self._update_execution_time_plot()
        self._update_memory_plot()
        self._update_performance_vs_size_plot()
        self._update_system_cpu_plot()
        self._update_system_memory_plot()
        self._update_success_rate_plot()

        # Update status
        self._update_status()

        # Update statistics
        self._update_statistics()

        return []

    def _collect_new_data(self):
        """Collect new data from queues"""
        # Collect performance metrics
        while not self.collector.metrics_queue.empty():
            try:
                metric = self.collector.metrics_queue.get_nowait()
                self.metrics_history.append(metric)
            except queue.Empty:
                break

        # Collect system metrics
        while not self.collector.system_queue.empty():
            try:
                metric = self.collector.system_queue.get_nowait()
                self.system_history.append(metric)
            except queue.Empty:
                break

        # Trim history
        if len(self.metrics_history) > self.max_history:
            self.metrics_history = self.metrics_history[-self.max_history:]

        if len(self.system_history) > self.max_history:
            self.system_history = self.system_history[-self.max_history:]

    def _update_execution_time_plot(self):
        """Update execution time plot"""
        ax = self.axes[0, 0]
        ax.clear()
        ax.set_title('Execution Time by Algorithm')
        ax.set_xlabel('Time')
        ax.set_ylabel('Execution Time (s)')
        ax.grid(True, alpha=0.3)

        if not self.metrics_history:
            return

        # Group by algorithm
        algorithms = {}
        for metric in self.metrics_history[-100:]:  # Last 100 points
            if metric.algorithm not in algorithms:
                algorithms[metric.algorithm] = {'times': [], 'exec_times': []}

            algorithms[metric.algorithm]['times'].append(
                datetime.fromtimestamp(metric.timestamp)
            )
            algorithms[metric.algorithm]['exec_times'].append(metric.execution_time)

        # Plot each algorithm
        colors = plt.cm.tab10(np.linspace(0, 1, len(algorithms)))
        for i, (algo, data) in enumerate(algorithms.items()):
            if data['times']:
                ax.plot(data['times'], data['exec_times'], 'o-',
                       color=colors[i], label=algo, markersize=3)

        ax.legend()
        ax.tick_params(axis='x', rotation=45)

    def _update_memory_plot(self):
        """Update memory usage plot"""
        ax = self.axes[0, 1]
        ax.clear()
        ax.set_title('Memory Usage')
        ax.set_xlabel('Time')
        ax.set_ylabel('Memory (MB)')
        ax.grid(True, alpha=0.3)

        if not self.metrics_history:
            return

        times = [datetime.fromtimestamp(m.timestamp) for m in self.metrics_history[-100:]]
        memory = [m.memory_usage_mb for m in self.metrics_history[-100:]]

        if times and memory:
            ax.plot(times, memory, 'b-', linewidth=2)

        ax.tick_params(axis='x', rotation=45)

    def _update_performance_vs_size_plot(self):
        """Update performance vs problem size plot"""
        ax = self.axes[0, 2]
        ax.clear()
        ax.set_title('Performance vs Problem Size')
        ax.set_xlabel('Problem Size')
        ax.set_ylabel('Execution Time (s)')
        ax.set_xscale('log')
        ax.set_yscale('log')
        ax.grid(True, alpha=0.3)

        if not self.metrics_history:
            return

        # Group by domain
        domains = {}
        for metric in self.metrics_history:
            if metric.problem_size > 0:  # Valid problem size
                if metric.domain not in domains:
                    domains[metric.domain] = {'sizes': [], 'times': []}

                domains[metric.domain]['sizes'].append(metric.problem_size)
                domains[metric.domain]['times'].append(metric.execution_time)

        # Plot each domain
        colors = plt.cm.tab10(np.linspace(0, 1, len(domains)))
        for i, (domain, data) in enumerate(domains.items()):
            if data['sizes']:
                ax.scatter(data['sizes'], data['times'],
                          color=colors[i], label=domain, alpha=0.7)

        ax.legend()

    def _update_system_cpu_plot(self):
        """Update system CPU usage plot"""
        ax = self.axes[1, 0]
        ax.clear()
        ax.set_title('System CPU Usage')
        ax.set_xlabel('Time')
        ax.set_ylabel('CPU %')
        ax.set_ylim(0, 100)
        ax.grid(True, alpha=0.3)

        if not self.system_history:
            return

        times = [datetime.fromtimestamp(m.timestamp) for m in self.system_history[-100:]]
        cpu = [m.cpu_percent for m in self.system_history[-100:]]

        if times and cpu:
            ax.plot(times, cpu, 'r-', linewidth=2)
            ax.fill_between(times, cpu, alpha=0.3, color='red')

        ax.tick_params(axis='x', rotation=45)

    def _update_system_memory_plot(self):
        """Update system memory usage plot"""
        ax = self.axes[1, 1]
        ax.clear()
        ax.set_title('System Memory Usage')
        ax.set_xlabel('Time')
        ax.set_ylabel('Memory %')
        ax.set_ylim(0, 100)
        ax.grid(True, alpha=0.3)

        if not self.system_history:
            return

        times = [datetime.fromtimestamp(m.timestamp) for m in self.system_history[-100:]]
        memory = [m.memory_percent for m in self.system_history[-100:]]

        if times and memory:
            ax.plot(times, memory, 'g-', linewidth=2)
            ax.fill_between(times, memory, alpha=0.3, color='green')

        ax.tick_params(axis='x', rotation=45)

    def _update_success_rate_plot(self):
        """Update success rate by domain plot"""
        ax = self.axes[1, 2]
        ax.clear()
        ax.set_title('Success Rate by Domain')
        ax.set_xlabel('Domain')
        ax.set_ylabel('Success Rate %')
        ax.set_ylim(0, 100)

        if not self.metrics_history:
            return

        # Calculate success rates by domain
        domain_stats = {}
        for metric in self.metrics_history:
            if metric.domain not in domain_stats:
                domain_stats[metric.domain] = {'total': 0, 'success': 0}

            domain_stats[metric.domain]['total'] += 1
            if metric.success:
                domain_stats[metric.domain]['success'] += 1

        domains = list(domain_stats.keys())
        success_rates = [
            (stats['success'] / stats['total']) * 100 if stats['total'] > 0 else 0
            for stats in domain_stats.values()
        ]

        if domains and success_rates:
            bars = ax.bar(domains, success_rates, color='skyblue', alpha=0.7)

            # Add value labels on bars
            for bar, rate in zip(bars, success_rates):
                height = bar.get_height()
                ax.text(bar.get_x() + bar.get_width()/2., height + 1,
                       f'{rate:.1f}%', ha='center', va='bottom')

        ax.tick_params(axis='x', rotation=45)

    def _update_status(self):
        """Update status information"""
        self.status_vars['metrics_count'].set(f"Metrics: {len(self.metrics_history)}")

        if self.metrics_history:
            last_time = datetime.fromtimestamp(self.metrics_history[-1].timestamp)
            self.status_vars['last_update'].set(f"Last Update: {last_time.strftime('%H:%M:%S')}")

        # System status
        if self.system_history:
            latest_system = self.system_history[-1]
            if latest_system.cpu_percent > 90 or latest_system.memory_percent > 90:
                self.status_vars['system_status'].set("System: HIGH LOAD")
            else:
                self.status_vars['system_status'].set("System: OK")

    def _update_statistics(self):
        """Update live statistics panel"""
        self.stats_text.delete(1.0, tk.END)

        if not self.metrics_history:
            self.stats_text.insert(tk.END, "No data available yet...\n")
            return

        # Calculate statistics
        recent_metrics = self.metrics_history[-50:]  # Last 50 metrics

        # Execution time stats
        exec_times = [m.execution_time for m in recent_metrics if m.execution_time > 0]
        if exec_times:
            avg_exec_time = np.mean(exec_times)
            max_exec_time = np.max(exec_times)
            min_exec_time = np.min(exec_times)

            self.stats_text.insert(tk.END, f"Execution Time Statistics (last {len(exec_times)} tests):\n")
            self.stats_text.insert(tk.END, f"  Average: {avg_exec_time:.4f}s\n")
            self.stats_text.insert(tk.END, f"  Range: {min_exec_time:.4f}s - {max_exec_time:.4f}s\n\n")

        # Memory stats
        memory_usage = [m.memory_usage_mb for m in recent_metrics if m.memory_usage_mb > 0]
        if memory_usage:
            avg_memory = np.mean(memory_usage)
            max_memory = np.max(memory_usage)

            self.stats_text.insert(tk.END, f"Memory Usage Statistics:\n")
            self.stats_text.insert(tk.END, f"  Average: {avg_memory:.1f} MB\n")
            self.stats_text.insert(tk.END, f"  Peak: {max_memory:.1f} MB\n\n")

        # Success rate
        total_tests = len(recent_metrics)
        successful_tests = sum(1 for m in recent_metrics if m.success)
        success_rate = (successful_tests / total_tests) * 100 if total_tests > 0 else 0

        self.stats_text.insert(tk.END, f"Success Rate: {success_rate:.1f}% ({successful_tests}/{total_tests})\n\n")

        # Domain breakdown
        domains = {}
        for metric in recent_metrics:
            if metric.domain not in domains:
                domains[metric.domain] = 0
            domains[metric.domain] += 1

        if domains:
            self.stats_text.insert(tk.END, "Domain Distribution:\n")
            for domain, count in sorted(domains.items()):
                percentage = (count / total_tests) * 100
                self.stats_text.insert(tk.END, f"  {domain}: {count} ({percentage:.1f}%)\n")

    def export_data(self):
        """Export collected data to files"""
        try:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

            # Export performance metrics
            metrics_file = Path(f"dashboard_metrics_{timestamp}.json")
            with open(metrics_file, 'w') as f:
                json.dump([asdict(m) for m in self.metrics_history], f, indent=2)

            # Export system metrics
            system_file = Path(f"dashboard_system_{timestamp}.json")
            with open(system_file, 'w') as f:
                json.dump([asdict(m) for m in self.system_history], f, indent=2)

            messagebox.showinfo("Export Success",
                               f"Data exported to:\n{metrics_file}\n{system_file}")

        except Exception as e:
            messagebox.showerror("Export Error", f"Failed to export data:\n{str(e)}")

    def clear_data(self):
        """Clear all collected data"""
        if messagebox.askyesno("Clear Data", "Are you sure you want to clear all data?"):
            self.metrics_history.clear()
            self.system_history.clear()
            self.logger.info("Dashboard data cleared")

    def run(self):
        """Run the dashboard"""
        if not self.root:
            self.create_gui()

        self.logger.info("Starting performance dashboard...")
        self.root.mainloop()

class CommandLineDashboard:
    """Command-line version of performance monitoring"""

    def __init__(self, collector: PerformanceCollector):
        self.collector = collector
        self.logger = self._setup_logging()
        self.running = False

    def _setup_logging(self) -> logging.Logger:
        logger = logging.getLogger("CLIDashboard")
        logger.setLevel(logging.INFO)
        handler = logging.StreamHandler()
        formatter = logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s')
        handler.setFormatter(formatter)
        logger.addHandler(handler)
        return logger

    def run(self, duration: int = 60):
        """Run command-line monitoring for specified duration"""
        self.logger.info(f"Starting command-line monitoring for {duration} seconds...")

        self.collector.start_collection()
        self.running = True

        start_time = time.time()
        metrics_count = 0

        try:
            while self.running and (time.time() - start_time) < duration:
                # Collect metrics
                new_metrics = []
                while not self.collector.metrics_queue.empty():
                    try:
                        metric = self.collector.metrics_queue.get_nowait()
                        new_metrics.append(metric)
                        metrics_count += 1
                    except queue.Empty:
                        break

                # Print summary every 10 seconds
                if int(time.time() - start_time) % 10 == 0:
                    self._print_summary(metrics_count, new_metrics)

                time.sleep(1)

        except KeyboardInterrupt:
            self.logger.info("Monitoring interrupted by user")

        finally:
            self.collector.stop_collection()
            self.running = False
            self.logger.info(f"Monitoring completed. Total metrics collected: {metrics_count}")

    def _print_summary(self, total_metrics: int, recent_metrics: List[PerformanceMetric]):
        """Print monitoring summary"""
        print(f"\n--- Performance Summary ---")
        print(f"Total metrics collected: {total_metrics}")
        print(f"Recent metrics: {len(recent_metrics)}")

        if recent_metrics:
            avg_time = np.mean([m.execution_time for m in recent_metrics])
            avg_memory = np.mean([m.memory_usage_mb for m in recent_metrics if m.memory_usage_mb > 0])
            success_rate = sum(1 for m in recent_metrics if m.success) / len(recent_metrics) * 100

            print(f"Average execution time: {avg_time:.4f}s")
            print(f"Average memory usage: {avg_memory:.1f} MB")
            print(f"Success rate: {success_rate:.1f}%")

        # System info
        try:
            system_metric = self.collector.system_queue.get_nowait()
            print(f"System CPU: {system_metric.cpu_percent:.1f}%")
            print(f"System Memory: {system_metric.memory_percent:.1f}%")
        except queue.Empty:
            pass

        print("--- End Summary ---\n")

def main():
    """Main entry point"""
    import argparse

    parser = argparse.ArgumentParser(description="Performance Monitoring Dashboard")
    parser.add_argument("--mode", choices=["gui", "cli"], default="gui",
                       help="Dashboard mode")
    parser.add_argument("--duration", type=int, default=300,
                       help="Monitoring duration for CLI mode (seconds)")
    parser.add_argument("--output-dir", default="dashboard_output",
                       help="Output directory for exported data")

    args = parser.parse_args()

    # Create output directory
    Path(args.output_dir).mkdir(exist_ok=True)

    # Create collector
    collector = PerformanceCollector()

    if args.mode == "gui":
        if not DASHBOARD_AVAILABLE:
            print("GUI mode requires matplotlib and tkinter. Install with:")
            print("pip install matplotlib tkinter")
            print("Falling back to CLI mode...")
            args.mode = "cli"
        else:
            dashboard = PerformanceDashboard(collector)
            dashboard.run()

    if args.mode == "cli":
        dashboard = CommandLineDashboard(collector)
        dashboard.run(args.duration)

if __name__ == "__main__":
    main()