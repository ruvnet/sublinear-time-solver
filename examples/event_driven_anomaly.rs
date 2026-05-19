//! Event-driven anomaly detection — the canonical RuView / Cognitum
//! inner-loop demo for the ADR-001 SubLinear primitives.
//!
//! ADR-001 thesis: *intelligence is sparse, event-driven, sub-linear,
//! coherence-gated activation — NOT brute-force token streams.*
//!
//! This example concretises that thesis on a sensor-network-style
//! workload:
//!
//!   1. Build a 256-node diagonally-dominant adjacency network.
//!   2. Compute the "baseline" solution `x_prev = A⁻¹ · b_prev` once.
//!   3. Stream 5 single-sensor events (sparse RHS deltas).
//!   4. For each event:
//!        a. Compute the bounded-depth closure of the delta's support.
//!        b. Solve only the closure entries via the sublinear path
//!           (`solve_on_change_sublinear`).
//!        c. Run contrastive top-k anomaly detection scoped to the
//!           closure (`contrastive_solve_on_change_sublinear`).
//!        d. Log closure size, per-event latency, and the top-3
//!           anomalies. Nothing outside the closure is ever touched.
//!
//! Real applications would forward each event's top-k to an agent's
//! attention queue; the wake-on-event discipline is exactly what
//! prevents the recursive-planning blowup the ADR flags.
//!
//! ## Run
//!
//! ```bash
//! cargo run --release --example event_driven_anomaly
//! ```
//!
//! ## Why this is the *canonical* inner loop
//!
//! RuView agents see sensor / log / metric streams. Each new event is a
//! sparse RHS delta against a stable baseline state. The system never
//! needs to recompute the full `n`-dimensional state — only the rows
//! that *could* have changed under the delta's bounded-depth influence.
//! That's exactly what `closure_indices` + `solve_on_change_sublinear`
//! delivers. The orchestrator returns control in `O(|closure| · log(1/ε))`
//! regardless of `n`, so the system's wake-cycle latency is bounded by
//! the *information content of the event*, not the size of the world.

use std::time::Instant;
use sublinear_solver::{
    contrastive_solve_on_change_sublinear, solve_on_change_sublinear, AnomalyRow, Matrix,
    NeumannSolver, SolverAlgorithm, SolverOptions, SparseDelta, SparseMatrix,
};

/// Build a strictly diagonally-dominant `n × n` "ring-stencil" matrix:
/// `a[i,i] = 5`, plus ±1 at four nearest neighbours with wrap. Models a
/// dense-enough sensor network that the closure isn't trivial.
fn build_network(n: usize) -> SparseMatrix {
    let mut t = Vec::with_capacity(n * 5);
    for i in 0..n {
        t.push((i, i, 5.0_f64));
        t.push((i, (i + 1) % n, 1.0));
        t.push((i, (i + 2) % n, 1.0));
        t.push((i, (i + n - 1) % n, -1.0));
        t.push((i, (i + n - 2) % n, -1.0));
    }
    SparseMatrix::from_triplets(t, n, n).expect("build_network")
}

fn main() {
    println!("event_driven_anomaly (sublinear-time-solver — ADR-001 demo)\n");

    let n = 256usize;
    let matrix = build_network(n);
    let b_prev: Vec<f64> = (0..n).map(|i| (i as f64) + 1.0).collect();

    println!(
        "matrix:       {n}x{n} strict-DD ring-stencil, nnz ≈ {}",
        matrix.nnz()
    );

    // ── Baseline solve. One-shot Linear cost; pays for itself across
    //    all subsequent events. ──
    let solver = NeumannSolver::new(128, 1e-12);
    let opts_tight = SolverOptions {
        max_iterations: 500,
        tolerance: 1e-10,
        ..SolverOptions::default()
    };
    let t_baseline = Instant::now();
    let prev_solution = solver
        .solve(&matrix, &b_prev, &opts_tight)
        .expect("baseline solve")
        .solution;
    let baseline_us = t_baseline.elapsed().as_micros();
    println!("baseline:     {baseline_us} µs (full Neumann solve, one-shot)\n");

    // ── Event stream. Each entry is (event_label, sensor_index, delta_value). ──
    let events: &[(&str, usize, f64)] = &[
        ("sensor #42 spike   ", 42, 1.50),
        ("sensor #117 drift  ", 117, -0.40),
        ("sensor #200 spike  ", 200, 2.10),
        ("sensor #7 dropout  ", 7, -1.80),
        ("sensor #155 outlier", 155, 3.25),
    ];

    println!("event stream (closure_depth=4, max_terms=24, tolerance=1e-8):");
    println!(
        "{:<22} {:>10} {:>12} {:>14} {:>14}",
        "event", "closure", "latency_us", "top_anomaly", "score"
    );
    println!("{}", "─".repeat(74));

    let closure_depth = 4usize;
    let max_terms = 24usize;
    let tolerance = 1e-8_f64;
    let top_k = 3usize;

    for (label, idx, dv) in events {
        let delta = SparseDelta::new(vec![*idx], vec![*dv]).expect("delta");
        let mut b_new = b_prev.clone();
        delta.apply_to(&mut b_new).expect("apply");

        // (1) Sparse delta-solve over the closure only.
        let t = Instant::now();
        let sparse_entries = solve_on_change_sublinear(
            &matrix,
            &prev_solution,
            &b_new,
            &delta,
            closure_depth,
            max_terms,
            tolerance,
        )
        .expect("sublinear delta-solve");
        let sparse_us = t.elapsed().as_micros();

        // (2) Contrastive top-k anomaly detection, same closure scope.
        let top: Vec<AnomalyRow> = contrastive_solve_on_change_sublinear(
            &matrix,
            &prev_solution,
            &b_new,
            &delta,
            closure_depth,
            max_terms,
            tolerance,
            top_k,
        )
        .expect("sublinear contrastive solve");

        let closure_n = sparse_entries.len();
        let top_row = top.first().map(|r| r.row.to_string()).unwrap_or_default();
        let top_score = top.first().map(|r| r.anomaly).unwrap_or(0.0);

        println!(
            "{:<22} {:>10} {:>12} {:>14} {:>14.4}",
            label, closure_n, sparse_us, top_row, top_score
        );
    }

    println!();
    println!("Architecture summary:");
    println!(
        "  baseline    Linear     {:>5} µs    full n-vector solve, one-shot",
        baseline_us
    );
    println!(
        "  per-event   SubLinear  closure=17 rows    independent of n=256"
    );
    println!();
    println!("Per-event latency is *bounded by closure size*, not n. Doubling n");
    println!("(or growing the state space to 10⁴+ rows) leaves the per-event");
    println!("closure size and cost essentially unchanged — the architectural");
    println!("win that lets RuView / Cognitum sustain change-driven loops over");
    println!("large state spaces without burning the J/decision budget. See");
    println!("docs/adr/ADR-001-complexity-as-architecture.md.");
}
