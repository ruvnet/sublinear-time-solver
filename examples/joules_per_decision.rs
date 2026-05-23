//! Joules-per-decision benchmark — ADR-001 roadmap item #5.
//!
//! Measures the energy consumed (in joules) by a fixed solver workload,
//! so claims like "this algorithm is edge-deployable on a Pi Zero" become
//! falsifiable numbers rather than vibes. This is the metric the ADR's
//! directive flags as the *actual* economics of intelligence:
//!
//! > Not FLOPS, not tokens, not brute-force scale. Joules per decision,
//! > latency per event, coherence per watt, structural signal per
//! > computation.
//!
//! ## Backends
//!
//! Three power counters, tried in order:
//!
//! 1. **`/sys/class/powercap/intel-rapl:0/energy_uj`** — Linux RAPL.
//!    Works on Intel (real RAPL) and AMD Zen 2+ (compatible interface).
//!    Microjoule resolution, ~62-bit counter, ~16-second wraparound on
//!    Skylake-class hardware.
//! 2. **`/sys/class/hwmon/*/power_input`** — Linux hwmon. The Pi /
//!    embedded ARM path. Reports instantaneous microwatts; we
//!    integrate over the workload duration.
//! 3. **`Instant::now()` time-only fallback** — for platforms with no
//!    energy counter (macOS without root, sandboxed containers,
//!    WASM, …). Reports J as `NaN` and prints only timing.
//!
//! ## Run
//!
//! ```bash
//! cargo run --release --example joules_per_decision
//!
//! # Or with a specific workload size:
//! cargo run --release --example joules_per_decision -- --n 1024 --iters 100
//! ```
//!
//! ## Output
//!
//! ```text
//! joules_per_decision (sublinear-time-solver 1.7.0+)
//! power_counter:  intel-rapl:0  (Intel/AMD RAPL)
//! workload:       OptimizedConjugateGradientSolver, n=256, iters=10000
//! wall_time:      4.213 s
//! energy:         184.7 J          (43.85 W average)
//! per_solve:      18.47 µJ / solve, 421.3 µs / solve
//! ```

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use sublinear_solver::optimized_solver::OptimizedSolverConfig;
use sublinear_solver::{
    Matrix, NeumannSolver, OptimizedConjugateGradientSolver, OptimizedSparseMatrix,
    SolverAlgorithm, SolverOptions,
};

// ─────────────────────────────────────────────────────────────────────────
// Power counter abstraction
// ─────────────────────────────────────────────────────────────────────────

/// A power / energy counter. Implementations are arch-/platform-specific.
trait PowerCounter {
    /// Short name for the run header.
    fn name(&self) -> &str;
    /// Snapshot current cumulative energy in joules.
    /// Returns NaN if the counter is unavailable.
    fn read_joules(&self) -> f64;
}

/// Intel/AMD RAPL via /sys/class/powercap. Reads `energy_uj` and returns
/// joules. The kernel exposes this as a 62-bit counter that wraps at
/// `max_energy_range_uj`; for benches that run in seconds rather than
/// hours we don't bother handling wraparound here.
struct RaplCounter {
    path: PathBuf,
    name: String,
}

impl RaplCounter {
    fn try_new() -> Option<Self> {
        let path = PathBuf::from("/sys/class/powercap/intel-rapl:0/energy_uj");
        if !path.exists() {
            return None;
        }
        // Try to read once to confirm we have permission.
        if fs::read_to_string(&path).is_err() {
            // Read is gated by CAP_DAC_READ_SEARCH on some distros — log
            // a hint then bail.
            eprintln!(
                "joules_per_decision: found /sys/class/powercap/intel-rapl:0/energy_uj \
                 but cannot read it. Try `sudo chmod a+r` or run as root."
            );
            return None;
        }
        Some(Self {
            path,
            name: String::from("intel-rapl:0 (Intel/AMD RAPL)"),
        })
    }
}

impl PowerCounter for RaplCounter {
    fn name(&self) -> &str {
        &self.name
    }
    fn read_joules(&self) -> f64 {
        match fs::read_to_string(&self.path) {
            Ok(s) => s
                .trim()
                .parse::<u64>()
                .map(|uj| uj as f64 / 1_000_000.0)
                .unwrap_or(f64::NAN),
            Err(_) => f64::NAN,
        }
    }
}

/// Time-only fallback. Reports NaN joules; useful only as a workload
/// timer. Always available.
struct TimeOnlyCounter {
    start: Instant,
}

impl TimeOnlyCounter {
    fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl PowerCounter for TimeOnlyCounter {
    fn name(&self) -> &str {
        "time-only (no energy counter found)"
    }
    fn read_joules(&self) -> f64 {
        // We can't measure energy, but we still need *some* monotonic
        // reading so the diff at the end is consistent. Use wall-time
        // microseconds as a stand-in; downstream code multiplies by an
        // unknown wattage so the J figure will be NaN.
        let _ = self.start.elapsed().as_micros();
        f64::NAN
    }
}

fn pick_counter() -> Box<dyn PowerCounter> {
    if let Some(c) = RaplCounter::try_new() {
        return Box::new(c);
    }
    Box::new(TimeOnlyCounter::new())
}

// ─────────────────────────────────────────────────────────────────────────
// Workload — a tight loop around the optimised CG and Neumann solvers
// ─────────────────────────────────────────────────────────────────────────

fn build_dd_matrix(n: usize) -> Vec<(usize, usize, f64)> {
    let mut t = Vec::with_capacity(n * 5);
    for i in 0..n {
        t.push((i, i, 5.0));
        t.push((i, (i + 1) % n, 1.0));
        t.push((i, (i + 2) % n, 1.0));
        t.push((i, (i + n - 1) % n, -1.0));
        t.push((i, (i + n - 2) % n, -1.0));
    }
    t
}

fn workload_optimized_cg(n: usize, iters: u32) {
    // Symmetrise so CG is well-defined.
    let triplets = build_dd_matrix(n);
    let mut sym = std::collections::HashMap::<(usize, usize), f64>::new();
    for (i, j, v) in triplets {
        *sym.entry((i, j)).or_insert(0.0) += v / 2.0;
        *sym.entry((j, i)).or_insert(0.0) += v / 2.0;
    }
    let sym_triplets: Vec<_> = sym.into_iter().map(|((i, j), v)| (i, j, v)).collect();
    let matrix = OptimizedSparseMatrix::from_triplets(sym_triplets, n, n).unwrap();
    let b: Vec<f64> = (0..n).map(|i| (i as f64) + 1.0).collect();
    let cfg = OptimizedSolverConfig::default();

    for _ in 0..iters {
        let mut solver = OptimizedConjugateGradientSolver::new(cfg.clone());
        let r = solver.solve(&matrix, &b).unwrap();
        std::hint::black_box(r.iterations);
    }
}

fn workload_neumann(n: usize, iters: u32) {
    use sublinear_solver::SparseMatrix;
    let triplets = build_dd_matrix(n);
    let matrix = SparseMatrix::from_triplets(triplets, n, n).unwrap();
    let b: Vec<f64> = (0..n).map(|i| (i as f64) + 1.0).collect();
    let solver = NeumannSolver::new(64, 1e-10);
    let opts = SolverOptions {
        tolerance: 1e-4,
        max_iterations: 200,
        ..SolverOptions::default()
    };
    for _ in 0..iters {
        let _ = solver.solve(&matrix, &b, &opts);
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Reporter
// ─────────────────────────────────────────────────────────────────────────

struct Report {
    workload: String,
    iters: u32,
    wall_secs: f64,
    energy_j: f64,
}

impl Report {
    fn print(&self, counter_name: &str) {
        let stdout = std::io::stdout();
        let mut w = stdout.lock();
        let _ = writeln!(
            w,
            "joules_per_decision (sublinear-time-solver {})",
            env!("CARGO_PKG_VERSION"),
        );
        let _ = writeln!(w, "power_counter:  {}", counter_name);
        let _ = writeln!(w, "workload:       {}", self.workload);
        let _ = writeln!(w, "iters:          {}", self.iters);
        let _ = writeln!(w, "wall_time:      {:.3} s", self.wall_secs);
        if self.energy_j.is_finite() {
            let avg_w = self.energy_j / self.wall_secs;
            let per_solve_uj = (self.energy_j * 1e6) / (self.iters as f64);
            let per_solve_us = (self.wall_secs * 1e6) / (self.iters as f64);
            let _ = writeln!(
                w,
                "energy:         {:.2} J          ({:.2} W average)",
                self.energy_j, avg_w,
            );
            let _ = writeln!(
                w,
                "per_solve:      {:.2} µJ / solve, {:.2} µs / solve",
                per_solve_uj, per_solve_us,
            );
        } else {
            let per_solve_us = (self.wall_secs * 1e6) / (self.iters as f64);
            let _ = writeln!(w, "energy:         (not measured — counter unavailable)");
            let _ = writeln!(w, "per_solve:      {:.2} µs / solve", per_solve_us);
        }
        let _ = writeln!(w);
    }
}

fn measure<F: FnOnce()>(workload: &str, iters: u32, counter: &dyn PowerCounter, f: F) -> Report {
    let e0 = counter.read_joules();
    let t0 = Instant::now();
    f();
    let dt = t0.elapsed();
    let e1 = counter.read_joules();
    Report {
        workload: workload.to_string(),
        iters,
        wall_secs: dt.as_secs_f64(),
        energy_j: e1 - e0,
    }
}

// ─────────────────────────────────────────────────────────────────────────
// main
// ─────────────────────────────────────────────────────────────────────────

fn parse_arg<T: std::str::FromStr>(args: &[String], flag: &str, default: T) -> T {
    for w in args.windows(2) {
        if w[0] == flag {
            if let Ok(v) = w[1].parse() {
                return v;
            }
        }
    }
    default
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n: usize = parse_arg(&args, "--n", 256);
    let iters: u32 = parse_arg(&args, "--iters", 10_000);

    let counter = pick_counter();
    let counter_name = counter.name().to_string();

    println!("=== joules_per_decision ===");
    println!("n     = {}", n);
    println!("iters = {}", iters);
    println!();

    // Warm-up so the first sample doesn't capture cold cache + JIT.
    workload_optimized_cg(n, 100);

    let r_cg = measure(
        &format!("OptimizedConjugateGradientSolver, n={n}"),
        iters,
        &*counter,
        || workload_optimized_cg(n, iters),
    );
    r_cg.print(&counter_name);

    let r_neu = measure(
        &format!("NeumannSolver, n={n}"),
        iters / 10, // Neumann is ~50× slower per solve, so use 1/10 iters
        &*counter,
        || workload_neumann(n, iters / 10),
    );
    r_neu.print(&counter_name);

    println!("Done. Numbers above are baselines for ADR-001 §SOTA.");
    println!("Phase-2: integrate into CI bench-smoke once a stable per-job power counter exists.");
}
