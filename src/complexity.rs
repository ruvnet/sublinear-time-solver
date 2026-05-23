//! Complexity-class declarations for every public solver / sampler / analyser.
//!
//! See [ADR-001: Complexity as Architecture](../../docs/adr/ADR-001-complexity-as-architecture.md)
//! for the strategic context. The short version: every public algorithm in this
//! crate carries an explicit worst-case complexity class as a compile-time
//! associated constant, so callers can refuse anything that exceeds their
//! per-subsystem budget without reading the source.
//!
//! ```rust,no_run
//! use sublinear_solver::complexity::{Complexity, ComplexityClass};
//! use sublinear_solver::OptimizedConjugateGradientSolver;
//!
//! // Compile-time check: the optimised CG solver is linear-in-nonzeros per iter.
//! const _: () = assert!(
//!     matches!(<OptimizedConjugateGradientSolver as Complexity>::CLASS,
//!              ComplexityClass::Linear)
//! );
//! ```
//!
//! Runtime introspection works the same way via the `ComplexityIntrospect`
//! trait, which is object-safe so a `dyn ComplexityIntrospect` query works on
//! any solver:
//!
//! ```rust,no_run
//! # use sublinear_solver::complexity::{ComplexityIntrospect, ComplexityClass};
//! fn budget_check(solver: &dyn ComplexityIntrospect, max: ComplexityClass) -> bool {
//!     solver.complexity_class() <= max
//! }
//! ```

#![allow(missing_docs)] // Complexity module — full docs TBD

use core::cmp::Ordering;

/// The twelve-tier complexity taxonomy from the directive in
/// [ADR-001](../../docs/adr/ADR-001-complexity-as-architecture.md).
///
/// Ordering is by asymptotic growth: `Logarithmic` is "easiest" / "cheapest",
/// `DoubleExponential` is "hardest" / "most expensive". `PartialOrd` /
/// `Ord` lift this into a usable budget comparison — `c <= max_budget`
/// means *c is at most as expensive as the budget*, so a caller with a
/// `SubLinear` budget will accept `Logarithmic` and `PolyLogarithmic` and
/// reject anything stronger.
///
/// The single-parameter `Polynomial(degree)` is ranked by its degree;
/// `Polynomial(2)` precedes `Polynomial(3)` etc. Adaptive solvers that
/// degrade on hard inputs can use `Adaptive { default, worst }` so the
/// caller sees both bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComplexityClass {
    /// `O(log n)` — binary search, HNSW layer traversal, sublinear-Neumann
    /// single-entry query on a DD system.
    Logarithmic,
    /// `O((log n)^k)` — spectral sparsifiers, dynamic connectivity, live
    /// graph repair. The "polylog" tier.
    PolyLogarithmic,
    /// `O(n^c), c < 1` — approximate nearest neighbour, sparse attention,
    /// event-driven activation, anomaly detection.
    SubLinear,
    /// `O(n)` — one-pass streaming, ingest, WAL replay, sensor scan.
    Linear,
    /// `O(n log n)` — sorting, indexing, ANN build, graph compression.
    /// The "practical sweet spot" for offline preprocessing.
    QuasiLinear,
    /// `O(n^{2-ε})` — sparsified mincut, sub-quadratic graph algorithms.
    SubQuadratic,
    /// `O(n^k)` — classical polynomial algorithms (matrix multiply for k=3,
    /// dense linear solves for k=2.37+, etc.). Degree of the polynomial is
    /// carried so callers can prefer lower-degree solvers.
    Polynomial(u8),
    /// Worse than any fixed polynomial but better than exponential.
    /// Combinatorial enumeration with strong pruning lives here.
    SuperPolynomial,
    /// `2^{O(n^c)}, c < 1` — SAT solvers, advanced cryptography, optimisation.
    SubExponential,
    /// `O(2^n)` — brute-force search, exhaustive planning, unrestricted
    /// combinatorics. Catastrophic at any non-trivial scale.
    Exponential,
    /// `O(n!)` — permutation search, exhaustive TSP. "Heat death of the
    /// universe territory" at n ≥ 20.
    Factorial,
    /// `O(2^{2^n})` — formal systems, symbolic explosion. Theoretical only.
    DoubleExponential,

    /// Adaptive — a solver that runs at `default` on typical inputs but
    /// can degrade to `worst` on adversarial inputs. Callers should budget
    /// against `worst` to be safe.
    Adaptive {
        default: &'static ComplexityClass,
        worst: &'static ComplexityClass,
    },
}

impl ComplexityClass {
    /// Rank suitable for ordering. Lower = cheaper. `Adaptive` ranks by its
    /// `worst` bound so callers comparing against a budget see the safe
    /// upper bound.
    pub const fn rank(&self) -> u16 {
        match self {
            ComplexityClass::Logarithmic => 100,
            ComplexityClass::PolyLogarithmic => 200,
            ComplexityClass::SubLinear => 300,
            ComplexityClass::Linear => 400,
            ComplexityClass::QuasiLinear => 500,
            ComplexityClass::SubQuadratic => 600,
            // Polynomial: 700 + degree, so Polynomial(2) = 702, Polynomial(3) = 703, …
            ComplexityClass::Polynomial(degree) => 700u16 + *degree as u16,
            ComplexityClass::SuperPolynomial => 800,
            ComplexityClass::SubExponential => 900,
            ComplexityClass::Exponential => 1000,
            ComplexityClass::Factorial => 1100,
            ComplexityClass::DoubleExponential => 1200,
            ComplexityClass::Adaptive { worst, .. } => worst.rank(),
        }
    }

    /// Short human-readable label suitable for log lines and MCP tool schemas.
    pub const fn short_label(&self) -> &'static str {
        match self {
            ComplexityClass::Logarithmic => "O(log n)",
            ComplexityClass::PolyLogarithmic => "O((log n)^k)",
            ComplexityClass::SubLinear => "O(n^c), c<1",
            ComplexityClass::Linear => "O(n)",
            ComplexityClass::QuasiLinear => "O(n log n)",
            ComplexityClass::SubQuadratic => "O(n^{2-ε})",
            ComplexityClass::Polynomial(2) => "O(n^2)",
            ComplexityClass::Polynomial(3) => "O(n^3)",
            ComplexityClass::Polynomial(_) => "O(n^k)",
            ComplexityClass::SuperPolynomial => "superpoly",
            ComplexityClass::SubExponential => "subexp",
            ComplexityClass::Exponential => "O(2^n)",
            ComplexityClass::Factorial => "O(n!)",
            ComplexityClass::DoubleExponential => "O(2^{2^n})",
            ComplexityClass::Adaptive { .. } => "adaptive",
        }
    }

    /// True if this class is acceptable for a real-time / edge / always-on hot
    /// path. Currently: anything strictly cheaper than `Linear`. Linear itself
    /// is conditional (acceptable for one-pass streaming, not for per-query
    /// work) — callers needing nuance should match directly.
    pub const fn is_edge_safe(&self) -> bool {
        self.rank() < ComplexityClass::Linear.rank()
    }
}

impl PartialOrd for ComplexityClass {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ComplexityClass {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}

/// Compile-time complexity-class declaration. Every public solver / sampler /
/// analyser in this crate implements this trait, exposing its worst-case
/// class as a `const` so callers can `match` on it at compile time.
///
/// ```rust,no_run
/// # use sublinear_solver::complexity::{Complexity, ComplexityClass};
/// fn pick_solver<S: Complexity>() {
///     match S::CLASS {
///         ComplexityClass::Logarithmic | ComplexityClass::SubLinear => {
///             // use it — edge-safe
///         }
///         _ => {
///             // fall back to something cheaper or refuse
///         }
///     }
/// }
/// ```
pub trait Complexity {
    /// Worst-case complexity class on a single-query call. For iterative
    /// solvers this is the per-iter cost; the iteration count is bounded by
    /// other configuration (max_iterations, tolerance, ef_construction).
    const CLASS: ComplexityClass;

    /// Optional human-readable detail for documentation / MCP tool schemas.
    /// Defaults to the short label of `CLASS`. Override when there's a
    /// non-obvious constant or k-bound.
    const DETAIL: &'static str = "";
}

/// Object-safe runtime introspection. Defaulting `impl<T: Complexity>` so any
/// type with a static `Complexity` impl gets `ComplexityIntrospect` for free,
/// and a `dyn ComplexityIntrospect` query works on solver trait objects.
///
/// ```rust,no_run
/// # use sublinear_solver::complexity::{ComplexityIntrospect, ComplexityClass};
/// fn within_budget(s: &dyn ComplexityIntrospect, max: ComplexityClass) -> bool {
///     s.complexity_class() <= max
/// }
/// ```
pub trait ComplexityIntrospect {
    fn complexity_class(&self) -> ComplexityClass;
    fn complexity_detail(&self) -> &'static str {
        ""
    }
}

impl<T: Complexity> ComplexityIntrospect for T {
    fn complexity_class(&self) -> ComplexityClass {
        T::CLASS
    }
    fn complexity_detail(&self) -> &'static str {
        T::DETAIL
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Complexity impls for the headline solvers / samplers / embeddings.
//
// Each impl is one line and forces the implementer to think about which
// class their algorithm actually inhabits. CI regression-guards in
// `.github/workflows/ci.yml` will eventually diff these between PRs.
// ─────────────────────────────────────────────────────────────────────────

impl Complexity for crate::solver::neumann::NeumannSolver {
    const CLASS: ComplexityClass = ComplexityClass::Linear;
    const DETAIL: &'static str =
        "O(k · nnz(A)) per iter; k bounded by series_tolerance + max_terms (default 200).";
}

impl Complexity for crate::optimized_solver::OptimizedConjugateGradientSolver {
    const CLASS: ComplexityClass = ComplexityClass::Linear;
    const DETAIL: &'static str = "O(k · nnz(A)) per iter; k ≈ √κ(A) on SPD inputs.";
}

impl Complexity for crate::sublinear::sublinear_neumann::SublinearNeumannSolver {
    // Adaptive: O(log n) on the per-entry sublinear-guaranteed path,
    // degrades to O(n) on the base-case path when n ≤ base_case_threshold.
    const CLASS: ComplexityClass = ComplexityClass::Adaptive {
        default: &ComplexityClass::Logarithmic,
        worst: &ComplexityClass::Linear,
    };
    const DETAIL: &'static str =
        "O(log n) per single-entry query on diagonally-dominant systems via JL + recursive Neumann; \
         O(n) base case at n ≤ base_case_threshold.";
}

impl Complexity for crate::sublinear::johnson_lindenstrauss::JLEmbedding {
    const CLASS: ComplexityClass = ComplexityClass::Linear;
    const DETAIL: &'static str =
        "O(d · k) per project_vector; k = target_dim, capped at original_dim - 1.";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering_is_by_asymptotic_growth() {
        assert!(ComplexityClass::Logarithmic < ComplexityClass::PolyLogarithmic);
        assert!(ComplexityClass::PolyLogarithmic < ComplexityClass::SubLinear);
        assert!(ComplexityClass::SubLinear < ComplexityClass::Linear);
        assert!(ComplexityClass::Linear < ComplexityClass::QuasiLinear);
        assert!(ComplexityClass::QuasiLinear < ComplexityClass::SubQuadratic);
        assert!(ComplexityClass::SubQuadratic < ComplexityClass::Polynomial(2));
        assert!(ComplexityClass::Polynomial(2) < ComplexityClass::Polynomial(3));
        assert!(ComplexityClass::Polynomial(3) < ComplexityClass::SuperPolynomial);
        assert!(ComplexityClass::SuperPolynomial < ComplexityClass::SubExponential);
        assert!(ComplexityClass::SubExponential < ComplexityClass::Exponential);
        assert!(ComplexityClass::Exponential < ComplexityClass::Factorial);
        assert!(ComplexityClass::Factorial < ComplexityClass::DoubleExponential);
    }

    #[test]
    fn edge_safe_predicate_matches_adr() {
        assert!(ComplexityClass::Logarithmic.is_edge_safe());
        assert!(ComplexityClass::PolyLogarithmic.is_edge_safe());
        assert!(ComplexityClass::SubLinear.is_edge_safe());
        // Linear and above are conditional / not safe by default.
        assert!(!ComplexityClass::Linear.is_edge_safe());
        assert!(!ComplexityClass::QuasiLinear.is_edge_safe());
        assert!(!ComplexityClass::Exponential.is_edge_safe());
    }

    #[test]
    fn adaptive_ranks_by_worst_case() {
        static QUASILIN: ComplexityClass = ComplexityClass::QuasiLinear;
        static POLY3: ComplexityClass = ComplexityClass::Polynomial(3);
        let adaptive = ComplexityClass::Adaptive {
            default: &QUASILIN,
            worst: &POLY3,
        };
        // adaptive is treated as expensive-as-worst-case for budget purposes.
        assert!(adaptive > ComplexityClass::QuasiLinear);
        assert_eq!(adaptive.rank(), POLY3.rank());
    }

    #[test]
    fn short_label_format() {
        assert_eq!(ComplexityClass::Logarithmic.short_label(), "O(log n)");
        assert_eq!(ComplexityClass::Polynomial(2).short_label(), "O(n^2)");
        assert_eq!(ComplexityClass::Polynomial(3).short_label(), "O(n^3)");
        assert_eq!(ComplexityClass::Polynomial(4).short_label(), "O(n^k)");
        assert_eq!(ComplexityClass::Exponential.short_label(), "O(2^n)");
    }

    /// Compile-time / runtime parity: a value that has `Complexity` also has
    /// `ComplexityIntrospect`, and the two report the same class.
    #[test]
    fn introspect_blanket_impl_matches_const() {
        struct Dummy;
        impl Complexity for Dummy {
            const CLASS: ComplexityClass = ComplexityClass::SubLinear;
            const DETAIL: &'static str = "test dummy";
        }
        let d = Dummy;
        assert_eq!(d.complexity_class(), ComplexityClass::SubLinear);
        assert_eq!(d.complexity_detail(), "test dummy");
        // and the const path agrees:
        assert_eq!(<Dummy as Complexity>::CLASS, ComplexityClass::SubLinear);
    }
}
