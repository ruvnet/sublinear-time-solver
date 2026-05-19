//! Streaming event-stream processing via the SubLinear pipeline.
//!
//! Companion to `examples/event_driven_anomaly.rs`. The earlier
//! example called the per-event primitives directly; this one uses
//! the [`event_stream_iter`] iterator adapter (PR #45) and composes
//! it with stdlib's `.filter()` / `.take()` / `.collect()` — the
//! native Rust pattern RuView / Cognitum / Ruflo would adopt.
//!
//! Demonstrates:
//!   - One event stream → one iterator chain → reports.
//!   - `.filter(EventStatus::Solved)` cleanly skips noise + budget-
//!     refused entries.
//!   - `PlanBudget` cap on cumulative ops — iterator ends gracefully
//!     when budget exhausted.
//!   - Per-event latency tracked by the iterator.
//!
//! Run:
//!
//! ```bash
//! cargo run --release --example event_stream_processing
//! ```

use sublinear_solver::budget::PlanBudget;
use sublinear_solver::coherence::coherence_score;
use sublinear_solver::complexity::ComplexityClass;
use sublinear_solver::stream::{event_stream_iter, EventStatus, EventStreamConfig};
use sublinear_solver::{
    Matrix, NeumannSolver, SolverAlgorithm, SolverOptions, SparseDelta, SparseMatrix,
};

fn build_strong_ring(n: usize) -> SparseMatrix {
    let mut t = Vec::new();
    for i in 0..n {
        t.push((i, i, 10.0_f64));
        t.push((i, (i + 1) % n, 0.5));
        t.push((i, (i + n - 1) % n, -0.5));
    }
    SparseMatrix::from_triplets(t, n, n).unwrap()
}

fn main() {
    println!("event_stream_processing (sublinear-time-solver — ADR-001 streaming demo)\n");

    let n = 64usize;
    let matrix = build_strong_ring(n);
    let b_prev: Vec<f64> = (0..n).map(|i| (i as f64) + 1.0).collect();

    // ── Baseline solve, one-shot. ──
    let solver = NeumannSolver::new(64, 1e-12);
    let opts = SolverOptions {
        max_iterations: 500,
        tolerance: 1e-10,
        ..SolverOptions::default()
    };
    let prev_solution = solver.solve(&matrix, &b_prev, &opts).unwrap().solution;

    // ── Cache (coherence, min_diag) for the skip gate. ──
    let coh = coherence_score(&matrix);
    let min_diag = (0..n)
        .map(|i| matrix.get(i, i).unwrap_or(0.0).abs())
        .filter(|x| *x > 0.0)
        .fold(f64::INFINITY, |a, b| if a < b { a } else { b });

    println!(
        "matrix=64x64 strong-DD ring | coherence={coh:.3}, min_diag={min_diag:.3}"
    );

    // ── Build an iterator of 10 events (mix of real + noise). ──
    let events: Vec<(SparseDelta, Vec<f64>)> = [
        ( 7, 0.50),    // real
        (13, 1e-12),   // noise → gate skips
        (22, -0.75),   // real
        (31, 1e-13),   // noise → gate skips
        (40, 1.20),    // real
        (51, -0.30),   // real
        (58, 1e-14),   // noise → gate skips
        ( 3, 0.65),    // real (budget limit hits here at max=4)
        (19, -1.10),   // would be budget-refused
        (44, 0.95),    // would be budget-refused
    ]
    .iter()
    .map(|(idx, dv)| {
        let d = SparseDelta::new(vec![*idx], vec![*dv]).unwrap();
        let mut b = b_prev.clone();
        d.apply_to(&mut b).unwrap();
        (d, b)
    })
    .collect();

    // ── PlanBudget caps cumulative solves at 4. ──
    let mut budget = PlanBudget::new(ComplexityClass::SubLinear, 4);
    let cfg = EventStreamConfig {
        tolerance: 1e-8,
        k: 3,
        skip_threshold: Some(1e-6),
        cached_coherence: Some(coh),
        cached_min_diag: Some(min_diag),
    };

    // ── Iterator pipeline. ──
    println!(
        "\n  iterator chain: event_stream_iter().take_while(!budget_refused)\n"
    );
    println!(
        "{:<8} {:<10} {:>12} {:<15}",
        "event #", "status", "latency_us", "top-anomaly"
    );
    println!("{}", "─".repeat(58));

    let mut solved = 0usize;
    let mut skipped = 0usize;
    let mut refused = 0usize;
    let mut errored = 0usize;

    for processed in event_stream_iter(
        &matrix,
        &prev_solution,
        events.into_iter(),
        &cfg,
        Some(&mut budget),
    ) {
        let label = match processed.status {
            EventStatus::Solved => {
                solved += 1;
                "Solved"
            }
            EventStatus::Skipped => {
                skipped += 1;
                "Skipped"
            }
            EventStatus::BudgetRefused => {
                refused += 1;
                "BudgetRefused"
            }
            EventStatus::Errored => {
                errored += 1;
                "Errored"
            }
        };
        let top_row = processed
            .anomalies
            .first()
            .map(|a| a.row.to_string())
            .unwrap_or_else(|| "—".to_string());
        let latency_us = processed.latency.as_micros();
        println!(
            "{:<8} {:<10} {:>12} row {:<10}",
            processed.event_idx, label, latency_us, top_row
        );
    }

    println!();
    println!(
        "Summary: {solved} solved, {skipped} skipped by gate, {refused} budget-refused, {errored} errored"
    );
    println!("Budget worst-class seen: {:?}", budget.worst_seen());
    println!("Budget remaining slots: {}", budget.remaining_ops());
    println!();
    println!("Architecture:");
    println!("  iterator surface  composes with stdlib (.filter / .take / .collect)");
    println!("  per-event         SubLinear (auto-tuned closure + Neumann)");
    println!("  coherence gate    O(|δ|) skip-on-tiny-delta");
    println!("  budget gate       O(1) per try_consume, ends iterator on refuse");
}
