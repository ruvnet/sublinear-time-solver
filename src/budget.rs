//! `PlanBudget` — cumulative complexity-class + operation-count
//! accumulator for chained solves.
//!
//! ADR-001 roadmap item #4's phase-2 MCP gate refuses **one** solve
//! that exceeds the caller's `max_complexity_class`. That's necessary
//! but not sufficient: an agent that issues 10 000 SubLinear solves in
//! a tight loop can still blow a real-world J/decision budget without
//! ever tripping a single per-call gate.
//!
//! This module closes the gap with a budget accumulator that lives
//! across the chain. Callers `try_consume(class)` once per solve;
//! the budget refuses the call if either:
//!
//! 1. `class` exceeds `max_class` (per-call class gate, same semantics
//!    as the existing MCP `enforceComplexityBudget`), or
//! 2. the remaining operation count has dropped to zero.
//!
//! The "operation count" abstraction is *number of solves*, not
//! number of inner-loop iterations — the unit a budget-aware agent
//! actually reasons about. A 100-event reflex loop with a 100-op
//! budget runs to completion; a 10 000-event one trips the budget
//! at iteration 100 and short-circuits gracefully.
//!
//! ## Composition
//!
//! Pair this with the per-call class gate that's already wired into
//! the MCP solve handlers. The MCP layer enforces *budget per call*;
//! `PlanBudget` enforces *budget per plan*.
//!
//! ## Why not `Rc<RefCell<...>>`?
//!
//! The accumulator is intentionally `&mut self` — borrow-checker
//! catches accidental shared mutation. Callers running planners on
//! multiple threads should `Arc<Mutex<PlanBudget>>` or partition the
//! budget per worker. The single-threaded planning case (the
//! common one) stays zero-cost.

#![allow(missing_docs)] // PlanBudget module — full docs TBD

use crate::complexity::ComplexityClass;
use core::fmt;

/// Cumulative budget for a chain of solves.
///
/// Tracks both the strictest per-call class allowance (`max_class`)
/// and a count of remaining ops (`remaining_ops`). Each
/// [`try_consume`] either succeeds (decrementing `remaining_ops`,
/// updating `worst_seen`) or returns a [`BudgetExhausted`] error
/// the caller can use to short-circuit the chain.
#[derive(Debug, Clone)]
pub struct PlanBudget {
    /// Strictest per-call class allowed. A `try_consume(class)` with
    /// `class > max_class` is rejected regardless of how many ops
    /// remain.
    max_class: ComplexityClass,
    /// Remaining operation slots. Decremented per successful consume.
    remaining_ops: usize,
    /// Worst class consumed so far (highest rank). Useful for the
    /// planner to report what bound *was* actually exercised.
    worst_seen: Option<ComplexityClass>,
}

impl PlanBudget {
    /// Construct a fresh budget with the given per-call class ceiling
    /// and op-count allowance.
    pub fn new(max_class: ComplexityClass, max_ops: usize) -> Self {
        Self {
            max_class,
            remaining_ops: max_ops,
            worst_seen: None,
        }
    }

    /// Attempt to consume one slot for an operation of the given
    /// `class`. Returns `Ok(())` on success and decrements the
    /// remaining-op count; returns `Err(BudgetExhausted)` if the
    /// class exceeds the budget or no ops remain.
    pub fn try_consume(&mut self, class: ComplexityClass) -> Result<(), BudgetExhausted> {
        if class > self.max_class {
            return Err(BudgetExhausted::ClassExceeded {
                attempted: class,
                ceiling: self.max_class,
            });
        }
        if self.remaining_ops == 0 {
            return Err(BudgetExhausted::OpsExhausted {
                worst_seen: self.worst_seen,
            });
        }
        self.remaining_ops -= 1;
        self.worst_seen = Some(match self.worst_seen {
            Some(prev) if prev >= class => prev,
            _ => class,
        });
        Ok(())
    }

    /// Number of op-slots still available.
    pub const fn remaining_ops(&self) -> usize {
        self.remaining_ops
    }

    /// Strictest per-call class allowed.
    pub const fn max_class(&self) -> ComplexityClass {
        self.max_class
    }

    /// Highest class consumed so far across the chain, or `None`
    /// if no ops have been consumed yet.
    pub const fn worst_seen(&self) -> Option<ComplexityClass> {
        self.worst_seen
    }

    /// True iff the budget can still accept *at least* a `Logarithmic`
    /// op — i.e. there are ops remaining (since `Logarithmic` is
    /// the cheapest class and always within any sensible `max_class`).
    /// Useful for "should I even try the next iteration?" loop guards.
    pub const fn has_capacity(&self) -> bool {
        self.remaining_ops > 0
    }
}

/// Reason a `try_consume` was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetExhausted {
    /// The attempted op's class is stronger than the budget's `max_class`
    /// ceiling. The op was *not* counted against the remaining ops.
    ClassExceeded {
        attempted: ComplexityClass,
        ceiling: ComplexityClass,
    },
    /// The remaining op-count dropped to zero. `worst_seen` reports
    /// the highest class consumed before the budget ran out.
    OpsExhausted { worst_seen: Option<ComplexityClass> },
}

impl fmt::Display for BudgetExhausted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BudgetExhausted::ClassExceeded { attempted, ceiling } => write!(
                f,
                "PlanBudget: attempted class {} exceeds ceiling {}",
                attempted.short_label(),
                ceiling.short_label(),
            ),
            BudgetExhausted::OpsExhausted { worst_seen } => match worst_seen {
                Some(c) => write!(
                    f,
                    "PlanBudget: ops exhausted (worst seen: {})",
                    c.short_label()
                ),
                None => write!(f, "PlanBudget: ops exhausted (none consumed)"),
            },
        }
    }
}

#[cfg(any(feature = "std", test))]
impl std::error::Error for BudgetExhausted {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::complexity::ComplexityClass;

    #[test]
    fn fresh_budget_has_capacity_and_no_worst_seen() {
        let b = PlanBudget::new(ComplexityClass::SubLinear, 10);
        assert_eq!(b.remaining_ops(), 10);
        assert!(b.has_capacity());
        assert_eq!(b.worst_seen(), None);
    }

    #[test]
    fn consume_within_class_and_count_succeeds() {
        let mut b = PlanBudget::new(ComplexityClass::SubLinear, 3);
        assert!(b.try_consume(ComplexityClass::Logarithmic).is_ok());
        assert!(b.try_consume(ComplexityClass::SubLinear).is_ok());
        assert_eq!(b.remaining_ops(), 1);
        assert_eq!(b.worst_seen(), Some(ComplexityClass::SubLinear));
    }

    #[test]
    fn consume_above_ceiling_refuses_without_counting() {
        let mut b = PlanBudget::new(ComplexityClass::SubLinear, 5);
        let err = b.try_consume(ComplexityClass::Linear).unwrap_err();
        match err {
            BudgetExhausted::ClassExceeded { attempted, ceiling } => {
                assert_eq!(attempted, ComplexityClass::Linear);
                assert_eq!(ceiling, ComplexityClass::SubLinear);
            }
            _ => panic!("expected ClassExceeded"),
        }
        // Refusal must NOT decrement the remaining-op count.
        assert_eq!(b.remaining_ops(), 5);
        assert_eq!(b.worst_seen(), None);
    }

    #[test]
    fn consume_after_ops_exhausted_refuses() {
        let mut b = PlanBudget::new(ComplexityClass::SubLinear, 2);
        assert!(b.try_consume(ComplexityClass::Logarithmic).is_ok());
        assert!(b.try_consume(ComplexityClass::SubLinear).is_ok());
        assert!(!b.has_capacity());

        let err = b.try_consume(ComplexityClass::Logarithmic).unwrap_err();
        match err {
            BudgetExhausted::OpsExhausted { worst_seen } => {
                assert_eq!(worst_seen, Some(ComplexityClass::SubLinear));
            }
            _ => panic!("expected OpsExhausted"),
        }
    }

    #[test]
    fn worst_seen_is_monotone() {
        let mut b = PlanBudget::new(ComplexityClass::Linear, 10);
        b.try_consume(ComplexityClass::SubLinear).unwrap();
        assert_eq!(b.worst_seen(), Some(ComplexityClass::SubLinear));

        // A later cheaper op must not lower worst_seen.
        b.try_consume(ComplexityClass::Logarithmic).unwrap();
        assert_eq!(b.worst_seen(), Some(ComplexityClass::SubLinear));

        // A later more-expensive op must raise it.
        b.try_consume(ComplexityClass::Linear).unwrap();
        assert_eq!(b.worst_seen(), Some(ComplexityClass::Linear));
    }

    #[test]
    fn ops_exhausted_carries_worst_seen() {
        let mut b = PlanBudget::new(ComplexityClass::Linear, 1);
        b.try_consume(ComplexityClass::Linear).unwrap();
        let err = b.try_consume(ComplexityClass::Logarithmic).unwrap_err();
        match err {
            BudgetExhausted::OpsExhausted { worst_seen } => {
                assert_eq!(worst_seen, Some(ComplexityClass::Linear));
            }
            _ => panic!("expected OpsExhausted"),
        }
    }

    #[test]
    fn zero_ops_budget_refuses_first_consume() {
        let mut b = PlanBudget::new(ComplexityClass::Linear, 0);
        assert!(!b.has_capacity());
        let err = b.try_consume(ComplexityClass::Logarithmic).unwrap_err();
        assert!(matches!(
            err,
            BudgetExhausted::OpsExhausted { worst_seen: None }
        ));
    }

    #[test]
    fn budget_exhausted_display_strings() {
        // ComplexityClass::short_label() gives big-O notation,
        // e.g. "O(n)" for Linear and "O(n^c), c<1" for SubLinear.
        let class_err = BudgetExhausted::ClassExceeded {
            attempted: ComplexityClass::Linear,
            ceiling: ComplexityClass::SubLinear,
        };
        let s = format!("{class_err}");
        assert!(s.contains("O(n)"), "expected O(n) in {s:?}");
        assert!(s.contains("c<1"), "expected c<1 in {s:?}");
        assert!(s.contains("exceeds"));

        let ops_err = BudgetExhausted::OpsExhausted { worst_seen: None };
        let s = format!("{ops_err}");
        assert!(s.contains("exhausted"));
    }
}
