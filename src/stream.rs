//! Streaming event iterator over the SubLinear orchestrator.
//!
//! ADR-001's primitives (closure, single-entry Neumann, contrastive
//! orchestrator, coherence gate, witness, budget) compose into a
//! per-event call pattern. Real callers — RuView agents watching
//! sensor / log / metric streams, Cognitum reflex loops over a
//! learning system, Ruflo inner-loop planners — process an *iterator*
//! of events, not a single one.
//!
//! This module lifts the single-event call into a stdlib `Iterator`
//! adapter: feed it an event stream, get back an iterator of
//! `(event_idx, Vec<AnomalyRow>, latency_us)` tuples. Native
//! composition with `.filter()`, `.take()`, `.collect()`, etc.
//!
//! Each yielded event:
//!
//!   1. Probes the [`crate::coherence::delta_below_solve_threshold`]
//!      skip gate first. If the gate trips, yields an empty `Vec`
//!      and a near-zero latency — the "no event, no work" path.
//!   2. Otherwise calls
//!      [`crate::contrastive_solve_on_change_sublinear_auto`] to
//!      compute the SubLinear top-k anomalies for the event.
//!   3. Optionally consumes a slot from a caller-supplied
//!      [`crate::PlanBudget`]; if the budget refuses, the iterator
//!      ends.
//!
//! The whole pipeline stays SubLinear per event, with the gate +
//! budget short-circuits letting callers cap end-to-end cost.

use crate::budget::PlanBudget;
use crate::complexity::{Complexity, ComplexityClass};
use crate::contrastive::{contrastive_solve_on_change_sublinear_auto, AnomalyRow};
use crate::error::Result;
use crate::matrix::Matrix;
use crate::types::Precision;
use crate::SparseDelta;
use alloc::vec::Vec;
use core::time::Duration;

/// One processed event: the original index in the input stream,
/// the top-k anomalies (empty if the gate skipped or budget refused),
/// the wall-time spent on this event, and a status flag.
#[derive(Debug, Clone)]
pub struct ProcessedEvent {
    /// Position of this event in the input iterator.
    pub event_idx: usize,
    /// Top-k anomalies. Empty if `status != Solved`.
    pub anomalies: Vec<AnomalyRow>,
    /// Wall-time spent on this event.
    pub latency: Duration,
    /// What happened.
    pub status: EventStatus,
}

/// What the orchestrator did with a single event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventStatus {
    /// The SubLinear orchestrator ran and returned a top-k.
    Solved,
    /// The coherence gate decided the delta was too small to warrant
    /// a solve. `latency` is the gate-probe cost only.
    Skipped,
    /// The caller-supplied [`PlanBudget`] refused this event's
    /// op-class. Iterator terminates after yielding this status.
    BudgetRefused,
    /// The orchestrator returned an error. `latency` is partial.
    Errored,
}

/// Op marker for [`event_stream_iter`]. Per-event class is the
/// orchestrator's — SubLinear on a strict-DD matrix.
#[derive(Debug, Clone, Copy, Default)]
pub struct EventStreamOp;

impl Complexity for EventStreamOp {
    const CLASS: ComplexityClass = ComplexityClass::SubLinear;
    const DETAIL: &'static str =
        "Per-event class is the auto-tuned SubLinear orchestrator's. The iterator wrapper \
         adds O(1) gate probe + optional O(1) budget consume per event; aggregate cost is \
         O(events · per_event_class).";
}

/// Configuration for the event-stream iterator.
#[derive(Debug, Clone)]
pub struct EventStreamConfig {
    /// Audit tolerance passed to the auto-tuned orchestrator.
    pub tolerance: Precision,
    /// Top-k to extract per event.
    pub k: usize,
    /// Optional skip-gate tolerance. If `Some`, the coherence-gated
    /// `delta_below_solve_threshold` check runs first and the
    /// orchestrator is skipped when the delta is below threshold.
    pub skip_threshold: Option<Precision>,
    /// Cached coherence + min-diag for the skip gate. Required iff
    /// `skip_threshold.is_some()`.
    pub cached_coherence: Option<Precision>,
    /// Cached min |diag(A)|. Required iff `skip_threshold.is_some()`.
    pub cached_min_diag: Option<Precision>,
}

impl Default for EventStreamConfig {
    fn default() -> Self {
        Self {
            tolerance: 1e-8,
            k: 3,
            skip_threshold: None,
            cached_coherence: None,
            cached_min_diag: None,
        }
    }
}

/// Process an iterator of `(SparseDelta, b_new)` events through the
/// SubLinear orchestrator, yielding [`ProcessedEvent`]s. The optional
/// `budget` argument tracks cumulative consumption across the stream;
/// when it refuses, the iterator yields one final `BudgetRefused`
/// event and ends.
///
/// `matrix` and `prev_solution` are borrowed for the lifetime of the
/// iterator — the same baseline state applies to every event in the
/// stream (matching the "stable baseline + sparse events" idiom RuView
/// agents use).
///
/// # Examples
///
/// ```rust,no_run
/// # use sublinear_solver::{Matrix, SparseDelta, PlanBudget};
/// # use sublinear_solver::complexity::ComplexityClass;
/// # use sublinear_solver::stream::{event_stream_iter, EventStreamConfig};
/// # fn demo<M: Matrix>(matrix: &M, prev: &[f64],
/// #                  events: Vec<(SparseDelta, Vec<f64>)>) {
/// let mut budget = PlanBudget::new(ComplexityClass::SubLinear, 100);
/// let cfg = EventStreamConfig { tolerance: 1e-8, k: 3, ..Default::default() };
///
/// for e in event_stream_iter(matrix, prev, events.into_iter(), &cfg, Some(&mut budget)) {
///     println!("event {}: {} anomalies in {:?}", e.event_idx, e.anomalies.len(), e.latency);
/// }
/// # }
/// ```
pub fn event_stream_iter<'a, I>(
    matrix: &'a dyn Matrix,
    prev_solution: &'a [Precision],
    events: I,
    config: &'a EventStreamConfig,
    budget: Option<&'a mut PlanBudget>,
) -> EventStreamIter<'a, I>
where
    I: Iterator<Item = (SparseDelta, Vec<Precision>)>,
{
    EventStreamIter {
        matrix,
        prev_solution,
        events,
        config,
        budget,
        idx: 0,
        ended: false,
    }
}

/// Iterator returned by [`event_stream_iter`].
pub struct EventStreamIter<'a, I>
where
    I: Iterator<Item = (SparseDelta, Vec<Precision>)>,
{
    matrix: &'a dyn Matrix,
    prev_solution: &'a [Precision],
    events: I,
    config: &'a EventStreamConfig,
    budget: Option<&'a mut PlanBudget>,
    idx: usize,
    ended: bool,
}

impl<'a, I> Iterator for EventStreamIter<'a, I>
where
    I: Iterator<Item = (SparseDelta, Vec<Precision>)>,
{
    type Item = ProcessedEvent;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ended {
            return None;
        }
        let (delta, b_new) = self.events.next()?;
        let idx = self.idx;
        self.idx += 1;

        let start = std_time_instant_now();

        // (1) Skip gate (optional).
        if let (Some(skip), Some(coh), Some(md)) = (
            self.config.skip_threshold,
            self.config.cached_coherence,
            self.config.cached_min_diag,
        ) {
            if crate::coherence::delta_below_solve_threshold(coh, md, &delta.values, skip) {
                let latency = std_time_instant_elapsed(start);
                return Some(ProcessedEvent {
                    event_idx: idx,
                    anomalies: Vec::new(),
                    latency,
                    status: EventStatus::Skipped,
                });
            }
        }

        // (2) Budget gate (optional).
        if let Some(budget) = self.budget.as_deref_mut() {
            if budget.try_consume(ComplexityClass::SubLinear).is_err() {
                self.ended = true;
                let latency = std_time_instant_elapsed(start);
                return Some(ProcessedEvent {
                    event_idx: idx,
                    anomalies: Vec::new(),
                    latency,
                    status: EventStatus::BudgetRefused,
                });
            }
        }

        // (3) SubLinear orchestrator.
        let result: Result<Vec<AnomalyRow>> = contrastive_solve_on_change_sublinear_auto(
            self.matrix,
            self.prev_solution,
            &b_new,
            &delta,
            self.config.tolerance,
            self.config.k,
        );
        let latency = std_time_instant_elapsed(start);
        match result {
            Ok(anomalies) => Some(ProcessedEvent {
                event_idx: idx,
                anomalies,
                latency,
                status: EventStatus::Solved,
            }),
            Err(_e) => Some(ProcessedEvent {
                event_idx: idx,
                anomalies: Vec::new(),
                latency,
                status: EventStatus::Errored,
            }),
        }
    }
}

// Wall-clock helpers. Gated to keep `no_std` builds compiling (the
// iterator still works without timing — it just records Duration::ZERO).

#[cfg(feature = "std")]
fn std_time_instant_now() -> std::time::Instant {
    std::time::Instant::now()
}

#[cfg(feature = "std")]
fn std_time_instant_elapsed(start: std::time::Instant) -> Duration {
    start.elapsed()
}

#[cfg(not(feature = "std"))]
fn std_time_instant_now() {
    // No-op stand-in.
}

#[cfg(not(feature = "std"))]
fn std_time_instant_elapsed(_start: ()) -> Duration {
    Duration::ZERO
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::SparseMatrix;
    use crate::solver::{neumann::NeumannSolver, SolverAlgorithm, SolverOptions};

    fn build_strong_ring(n: usize) -> SparseMatrix {
        let mut t = Vec::new();
        for i in 0..n {
            t.push((i, i, 10.0_f64));
            t.push((i, (i + 1) % n, 0.5));
            t.push((i, (i + n - 1) % n, -0.5));
        }
        SparseMatrix::from_triplets(t, n, n).unwrap()
    }

    fn warmup_solve(m: &SparseMatrix, b: &[f64]) -> Vec<f64> {
        let solver = NeumannSolver::new(64, 1e-12);
        let opts = SolverOptions::default();
        solver.solve(m, b, &opts).unwrap().solution
    }

    fn build_event(n: usize, idx: usize, dv: f64) -> (SparseDelta, Vec<f64>) {
        let delta = SparseDelta::new(vec![idx], vec![dv]).unwrap();
        let mut b = (0..n).map(|i| i as f64 + 1.0).collect::<Vec<_>>();
        delta.apply_to(&mut b).unwrap();
        (delta, b)
    }

    #[test]
    fn op_class_is_sublinear() {
        const _: () = assert!(matches!(
            <EventStreamOp as Complexity>::CLASS,
            ComplexityClass::SubLinear
        ));
    }

    #[test]
    fn empty_input_yields_empty_output() {
        let n = 16;
        let m = build_strong_ring(n);
        let prev = warmup_solve(&m, &(0..n).map(|i| i as f64 + 1.0).collect::<Vec<_>>());
        let cfg = EventStreamConfig::default();
        let events: Vec<(SparseDelta, Vec<f64>)> = Vec::new();
        let out: Vec<_> = event_stream_iter(&m, &prev, events.into_iter(), &cfg, None).collect();
        assert!(out.is_empty());
    }

    #[test]
    fn yields_one_processed_event_per_input() {
        let n = 16;
        let m = build_strong_ring(n);
        let b_prev: Vec<f64> = (0..n).map(|i| i as f64 + 1.0).collect();
        let prev = warmup_solve(&m, &b_prev);
        let cfg = EventStreamConfig {
            tolerance: 1e-8,
            k: 2,
            ..Default::default()
        };
        let events = vec![
            build_event(n, 3, 0.5),
            build_event(n, 7, -0.3),
            build_event(n, 11, 1.0),
        ];
        let out: Vec<_> = event_stream_iter(&m, &prev, events.into_iter(), &cfg, None).collect();
        assert_eq!(out.len(), 3);
        for (i, e) in out.iter().enumerate() {
            assert_eq!(e.event_idx, i);
            assert_eq!(e.status, EventStatus::Solved);
            assert!(!e.anomalies.is_empty());
        }
    }

    #[test]
    fn skip_gate_short_circuits_tiny_delta() {
        // Cache coherence + min_diag so the gate can run.
        let n = 16;
        let m = build_strong_ring(n);
        let b_prev: Vec<f64> = (0..n).map(|i| i as f64 + 1.0).collect();
        let prev = warmup_solve(&m, &b_prev);
        let coh = crate::coherence::coherence_score(&m);
        let min_diag = (0..n)
            .map(|i| Matrix::get(&m, i, i).unwrap_or(0.0).abs())
            .filter(|x| *x > 0.0)
            .fold(f64::INFINITY, |a, b| if a < b { a } else { b });
        let cfg = EventStreamConfig {
            tolerance: 1e-8,
            k: 2,
            skip_threshold: Some(1e-6),
            cached_coherence: Some(coh),
            cached_min_diag: Some(min_diag),
        };
        // Tiny delta → gate should skip.
        let events = vec![build_event(n, 3, 1e-12)];
        let out: Vec<_> = event_stream_iter(&m, &prev, events.into_iter(), &cfg, None).collect();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].status, EventStatus::Skipped);
        assert!(out[0].anomalies.is_empty());
    }

    #[test]
    fn budget_terminates_stream_when_exhausted() {
        let n = 16;
        let m = build_strong_ring(n);
        let b_prev: Vec<f64> = (0..n).map(|i| i as f64 + 1.0).collect();
        let prev = warmup_solve(&m, &b_prev);
        let cfg = EventStreamConfig::default();
        let events = vec![
            build_event(n, 3, 0.5),
            build_event(n, 7, -0.3),
            build_event(n, 11, 1.0),
        ];
        let mut budget = PlanBudget::new(ComplexityClass::SubLinear, 2);
        let out: Vec<_> = event_stream_iter(
            &m,
            &prev,
            events.into_iter(),
            &cfg,
            Some(&mut budget),
        )
        .collect();
        // Two Solved + one BudgetRefused, then iterator ends (3 total).
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].status, EventStatus::Solved);
        assert_eq!(out[1].status, EventStatus::Solved);
        assert_eq!(out[2].status, EventStatus::BudgetRefused);
        assert_eq!(budget.remaining_ops(), 0);
    }
}
