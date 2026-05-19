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
    contrastive_solve_on_change_sublinear_auto, delta_below_solve_threshold,
    solve_on_change_sublinear_auto, AnomalyRow, Matrix, NeumannSolver, SolverAlgorithm,
    SolverOptions, SparseDelta, SparseMatrix,
};
use sublinear_solver::coherence::coherence_score;

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
    println!("baseline:     {baseline_us} µs (full Neumann solve, one-shot)");

    // ── Cache (coherence, min_diag) once for the coherence-gated event
    //    filter (PR #34). Each event probes delta_below_solve_threshold
    //    in O(|delta|) before paying the closure + per-entry-Neumann cost.
    //    This is the "no event, no work" gate from ADR-001. ──
    let coherence = coherence_score(&matrix);
    let min_diag = (0..matrix.rows())
        .map(|i| matrix.get(i, i).unwrap_or(0.0).abs())
        .filter(|x| *x > 0.0)
        .fold(f64::INFINITY, |a, b| if a < b { a } else { b });
    // ── PR #38: orchestrators are now fully auto-tuned. We still cache
    //    (coherence, min_diag) here for the O(|δ|) skip gate (PR #34) —
    //    that's not auto, by design (cheaper to probe than recompute). ──
    println!(
        "gate cache:   coherence={coherence:.3}, min_diag={min_diag:.3} (skip=1e-6, tol=1e-8)\n"
    );

    // ── Event stream. Each entry is (event_label, sensor_index, delta_value).
    //    Includes a "noise" event whose delta is tiny enough that the
    //    coherence gate skips the solve entirely. ──
    let events: &[(&str, usize, f64)] = &[
        ("sensor #42 spike   ", 42, 1.50),
        ("sensor #117 drift  ", 117, -0.40),
        ("sensor #88  noise  ", 88, 1.0e-12),  // ← gated-out
        ("sensor #200 spike  ", 200, 2.10),
        ("sensor #7 dropout  ", 7, -1.80),
        ("sensor #155 outlier", 155, 3.25),
    ];

    println!("event stream (auto-tuned closure + max_terms, tolerance=1e-8, skip=1e-6):");
    println!(
        "{:<22} {:>10} {:>12} {:>14} {:>14}",
        "event", "closure", "latency_us", "top_anomaly", "score"
    );
    println!("{}", "─".repeat(74));

    let tolerance = 1e-8_f64;
    let skip_threshold = 1.0e-6_f64;
    let top_k = 3usize;

    for (label, idx, dv) in events {
        let delta = SparseDelta::new(vec![*idx], vec![*dv]).expect("delta");

        // ── Coherence-gated early exit. O(|delta|) — independent of n. ──
        let t_total = Instant::now();
        if delta_below_solve_threshold(coherence, min_diag, &delta.values, skip_threshold) {
            let skip_us = t_total.elapsed().as_micros();
            println!(
                "{:<22} {:>10} {:>12} {:>14} {:>14}",
                label, "—", skip_us, "skipped", "—"
            );
            continue;
        }

        let mut b_new = b_prev.clone();
        delta.apply_to(&mut b_new).expect("apply");

        // (1) Sparse delta-solve over the closure only — auto-tuned.
        //     The orchestrator computes coherence + picks
        //     closure_depth + max_terms internally; caller only supplies
        //     the tolerance contract.
        let sparse_entries =
            solve_on_change_sublinear_auto(&matrix, &prev_solution, &b_new, &delta, tolerance)
                .expect("auto-tuned delta-solve");

        // (2) Contrastive top-k anomaly detection — auto-tuned sibling.
        let top: Vec<AnomalyRow> = contrastive_solve_on_change_sublinear_auto(
            &matrix,
            &prev_solution,
            &b_new,
            &delta,
            tolerance,
            top_k,
        )
        .expect("auto-tuned contrastive solve");

        let total_us = t_total.elapsed().as_micros();
        let closure_n = sparse_entries.len();
        let top_row = top.first().map(|r| r.row.to_string()).unwrap_or_default();
        let top_score = top.first().map(|r| r.anomaly).unwrap_or(0.0);

        println!(
            "{:<22} {:>10} {:>12} {:>14} {:>14.4}",
            label, closure_n, total_us, top_row, top_score
        );
    }

    println!();
    println!("Architecture summary:");
    println!(
        "  baseline       Linear      {:>5} µs    full n-vector solve, one-shot",
        baseline_us
    );
    println!(
        "  coherence gate O(|δ|)        ~0 µs    skip tiny deltas before any solve"
    );
    println!(
        "  per-event      SubLinear   auto-tuned closure_depth+max_terms from coherence={coherence:.3}"
    );
    println!();
    println!("The orchestrators are now magic-number-free: pass tolerance, get top-k.");
    println!("Closure depth + Neumann terms are picked from the Neumann-envelope");
    println!("bound (PRs #37 + #38) — provably sufficient, never over-budget. On");
    println!("this low-coherence (c=0.2) test matrix, that math correctly demands a");
    println!("wide closure to reach 1e-8 tolerance. Higher-coherence matrices");
    println!("(c≥0.5) auto-pick tighter closures, pulling per-event cost down by");
    println!("orders of magnitude. The coherence gate short-circuits tiny / noisy");
    println!("events in O(|δ|) before any closure computation runs — the 'no");
    println!("event, no work' discipline of ADR-001 in action. See");
    println!("docs/adr/ADR-001-complexity-as-architecture.md.");
}
