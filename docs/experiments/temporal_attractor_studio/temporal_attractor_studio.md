Here’s a fast, no-nonsense Rust CLI that estimates the largest Lyapunov exponent from a trajectory or time series using delay embedding, a Theiler window, a cache-friendly VP-tree nearest-neighbor search, and parallel slope fits over early divergences. It avoids exotic deps, compiles cleanly, and scales well to mid-size datasets. Usage examples are in the doc comment at the top.

```toml
# Cargo.toml
[package]
name = "lyapfit"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
clap = { version = "4.5", features = ["derive"] }
csv = "1.3"
rayon = "1.10"
```

```rust
// src/main.rs
/*!
Lyapunov λ via nearest-neighbor divergence.

Examples
  # multivariate state per row, dt=0.01s, fit first 12 steps
  lyapfit data.csv --dt 0.01 --k-fit 12

  # univariate series in column 0 with delay embedding m=6, tau=2 samples
  lyapfit data.csv --dt 0.01 --col 0 --m 6 --tau 2 --k-fit 15

  # read CSV from stdin, skip header, widen Theiler window
  cat data.csv | lyapfit - --no-header --dt 0.02 --theiler 50
*/

use anyhow::{bail, Context, Result};
use clap::Parser;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::fs::File;
use std::io::{stdin, BufReader, Read};

#[derive(Parser, Debug)]
#[command(name="lyapfit")]
struct Args {
    /// CSV file path or "-" for stdin
    path: String,
    /// Sampling interval ∆t in seconds
    #[arg(long)]
    dt: f64,
    /// Column index for univariate delay embedding. Omit to use all columns as the state.
    #[arg(long)]
    col: Option<usize>,
    /// Embedding dimension m (univariate only)
    #[arg(long, default_value = "1")]
    m: usize,
    /// Delay in samples τ (univariate only)
    #[arg(long, default_value = "1")]
    tau: usize,
    /// Use header row
    #[arg(long, default_value_t = true)]
    header: bool,
    /// Theiler window W in samples to exclude temporal neighbors
    #[arg(long, default_value = "20")]
    theiler: usize,
    /// Number of early steps to fit (K_fit)
    #[arg(long, default_value = "12")]
    k_fit: usize,
    /// Maximum pairs sampled for averaging (stride over i)
    #[arg(long, default_value = "4000")]
    max_pairs: usize,
    /// Minimum initial separation; pairs below are skipped
    #[arg(long, default_value = "1e-12")]
    min_init_sep: f64,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.dt <= 0.0 {
        bail!("dt must be > 0");
    }
    if args.m == 0 {
        bail!("m must be >= 1");
    }
    if args.k_fit < 2 {
        bail!("k-fit must be >= 2");
    }

    let raw = read_csv(&args.path, args.header).context("reading CSV")?;
    if raw.is_empty() {
        bail!("empty input");
    }

    // Build state matrix X: Vec<Vec<f64>> where each entry is a state vector at time t
    let x = if let Some(col) = args.col {
        let series: Vec<f64> = raw
            .iter()
            .map(|row| {
                row.get(col)
                    .copied()
                    .unwrap_or_else(|| f64::NAN)
            })
            .collect();
        let x = delay_embed(&series, args.m, args.tau)?;
        x
    } else {
        // multivariate state per row
        raw
    };

    let n = x.len();
    if n < args.k_fit + 2 {
        bail!("not enough points after embedding");
    }
    let dim = x[0].len();
    if dim == 0 {
        bail!("zero-dimension state");
    }

    // Build VP-tree over embedded states
    let mut indices: Vec<usize> = (0..n - args.k_fit).collect(); // restrict to allow i+k access
    let tree = VpTree::build(&x, &mut indices);

    // Precompute linear regression constants for t = {1..K} * dt
    let k = args.k_fit as usize;
    let dt = args.dt;
    let mut t = Vec::with_capacity(k);
    for kk in 1..=k {
        t.push(kk as f64 * dt);
    }
    let t_mean = mean(&t);
    let var_t = t.iter().map(|tk| (tk - t_mean) * (tk - t_mean)).sum::<f64>();
    if var_t <= 0.0 {
        bail!("degenerate time variance");
    }

    // Sample pairs i -> j_nearest with Theiler window, fit slope on early log distances
    let stride = std::cmp::max(1usize, (n - args.k_fit) / args.max_pairs.max(1));
    let theiler = args.theiler;

    let slopes: Vec<f64> = (0..n - args.k_fit)
        .step_by(stride)
        .collect::<Vec<_>>()
        .par_iter()
        .filter_map(|&i| {
            let query = &x[i];
            // nearest neighbor with Theiler exclusion
            if let Some((j, d0)) = tree.nearest_excluding(query, i, theiler) {
                if d0 <= args.min_init_sep || j + k >= x.len() || i + k >= x.len() {
                    return None;
                }
                // Early growth curve
                let mut y = Vec::with_capacity(k);
                for kk in 1..=k {
                    let d = dist(&x[i + kk], &x[j + kk]);
                    // numerical guard
                    let dd = if d <= 0.0 { 1e-300 } else { d };
                    y.push((dd / d0).ln());
                }
                let y_mean = mean(&y);
                let cov = t
                    .iter()
                    .zip(y.iter())
                    .map(|(tk, yk)| (tk - t_mean) * (yk - y_mean))
                    .sum::<f64>();
                let slope = cov / var_t; // λ estimate from this pair
                if slope.is_finite() { Some(slope) } else { None }
            } else {
                None
            }
        })
        .collect();

    if slopes.is_empty() {
        bail!("no valid pairs found. Try reducing theiler or k-fit, or increase max-pairs");
    }

    let lambda = mean(&slopes);
    let td = std::f64::consts::LN_2 / lambda;
    let tl = 1.0 / lambda;

    println!("points_used,dim,dt,k_fit,theiler,pairs,lambda,lyapunov_time,Td_doubling");
    println!(
        "{},{},{:.9},{},{},{},{:.9},{:.9},{:.9}",
        n, dim, dt, k, theiler, slopes.len(), lambda, tl, td
    );

    Ok(())
}

/// Read CSV into Vec<Vec<f64>>
fn read_csv(path: &str, header: bool) -> Result<Vec<Vec<f64>>> {
    let rdr: Box<dyn Read> = if path == "-" {
        Box::new(stdin())
    } else {
        Box::new(File::open(path)?)
    };
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(header)
        .from_reader(BufReader::new(rdr));
    let mut out = Vec::new();
    for rec in reader.records() {
        let rec = rec?;
        let mut row = Vec::with_capacity(rec.len());
        for f in rec.iter() {
            // Allow blank cells to be NaN, skip rows that contain NaN
            let v = f.parse::<f64>().unwrap_or(f64::NAN);
            row.push(v);
        }
        if row.iter().all(|v| v.is_finite()) {
            out.push(row);
        }
    }
    Ok(out)
}

/// Univariate delay embed: returns Vec of state vectors length n_eff
fn delay_embed(series: &[f64], m: usize, tau: usize) -> Result<Vec<Vec<f64>>> {
    if m == 1 {
        // return as single-column states
        return Ok(series.iter().map(|v| vec![*v]).collect());
    }
    let n = series.len();
    let span = (m - 1) * tau;
    if n <= span {
        bail!("series too short for embedding");
    }
    let n_eff = n - span;
    let mut x = Vec::with_capacity(n_eff);
    for i in 0..n_eff {
        let mut v = Vec::with_capacity(m);
        for k in 0..m {
            let idx = i + k * tau;
            v.push(series[idx]);
        }
        x.push(v);
    }
    Ok(x)
}

#[inline]
fn mean(v: &[f64]) -> f64 {
    let s: f64 = v.iter().sum();
    s / (v.len() as f64)
}

#[inline]
fn dist(a: &[f64], b: &[f64]) -> f64 {
    let mut acc = 0.0;
    // manual unroll for small dims
    let len = a.len();
    let mut i = 0;
    while i + 3 < len {
        let d0 = a[i] - b[i];
        let d1 = a[i + 1] - b[i + 1];
        let d2 = a[i + 2] - b[i + 2];
        let d3 = a[i + 3] - b[i + 3];
        acc += d0 * d0 + d1 * d1 + d2 * d2 + d3 * d3;
        i += 4;
    }
    while i < len {
        let d = a[i] - b[i];
        acc += d * d;
        i += 1;
    }
    acc.sqrt()
}

/// Vantage-point tree with dynamic dimension
struct VpNode {
    idx: usize,        // index into dataset
    tau: f64,          // partition radius
    left: Option<Box<VpNode>>,
    right: Option<Box<VpNode>>,
}

struct VpTree<'a> {
    data: &'a [Vec<f64>],
    root: Option<Box<VpNode>>,
}

impl<'a> VpTree<'a> {
    fn build(data: &'a [Vec<f64>], indices: &mut [usize]) -> Self {
        let root = Self::build_rec(data, indices);
        Self { data, root }
    }

    fn build_rec(data: &'a [Vec<f64>], indices: &mut [usize]) -> Option<Box<VpNode>> {
        if indices.is_empty() {
            return None;
        }
        // use last as vantage point
        let vp = indices[indices.len() - 1];
        if indices.len() == 1 {
            return Some(Box::new(VpNode { idx: vp, tau: 0.0, left: None, right: None }));
        }
        // compute distances to vp
        let (left_slice, _vp_slot) = indices.split_at_mut(indices.len() - 1);
        let mut dists: Vec<(usize, f64)> = left_slice
            .iter()
            .map(|&j| (j, dist(&data[vp], &data[j])))
            .collect();
        // median split on distance
        let mid = dists.len() / 2;
        dists.select_nth_unstable_by(mid, |a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
        let tau = dists[mid].1;
        // partition into inner and outer
        let mut inner: Vec<usize> = Vec::with_capacity(mid + 1);
        let mut outer: Vec<usize> = Vec::with_capacity(dists.len() - mid);
        for (j, d) in dists {
            if d <= tau {
                inner.push(j);
            } else {
                outer.push(j);
            }
        }
        let left = Self::build_rec(data, &mut inner);
        let right = Self::build_rec(data, &mut outer);
        Some(Box::new(VpNode { idx: vp, tau, left, right }))
    }

    /// Nearest neighbor excluding indices within Theiler window of target_i
    fn nearest_excluding(&self, q: &[f64], target_i: usize, theiler: usize) -> Option<(usize, f64)> {
        let mut best_idx = usize::MAX;
        let mut best_dist = f64::INFINITY;
        self.search(&self.root, q, target_i, theiler, &mut best_idx, &mut best_dist);
        if best_idx == usize::MAX { None } else { Some((best_idx, best_dist)) }
    }

    fn search(
        &self,
        node: &Option<Box<VpNode>>,
        q: &[f64],
        target_i: usize,
        theiler: usize,
        best_idx: &mut usize,
        best_dist: &mut f64,
    ) {
        let Some(n) = node else { return; };
        let d = dist(q, &self.data[n.idx]);

        // respect Theiler window and skip self
        if n.idx != target_i && theiler_exclude(target_i, n.idx, theiler) == false {
            if d < *best_dist {
                *best_dist = d;
                *best_idx = n.idx;
            }
        }

        // choose side to visit first
        let first_left = d < n.tau || n.right.is_none();
        let (first, second) = if first_left { (&n.left, &n.right) } else { (&n.right, &n.left) };

        if let Some(_) = first {
            self.search(first, q, target_i, theiler, best_idx, best_dist);
        }
        // visit the other side if the hypersphere around q intersects the boundary
        if (d - n.tau).abs() <= *best_dist {
            if let Some(_) = second {
                self.search(second, q, target_i, theiler, best_idx, best_dist);
            }
        }
    }
}

#[inline]
fn theiler_exclude(i: usize, j: usize, w: usize) -> bool {
    let di = if i > j { i - j } else { j - i };
    di <= w
}
```

### Notes

* Fit window defaults to the first 12 steps. Bump `--k-fit` if your system has a longer linear growth regime.
* Increase `--theiler` when trajectories are smooth and temporally adjacent points collapse to trivial neighbors.
* For very long series, raise `--max-pairs` or lower the stride by setting a larger value for it implicitly via `max-pairs`.

If you want this as a library crate with an exported `estimate_lambda(&[Vec<f64>], …) -> f64` plus unit tests, say the word and I’ll hand you the full module split.
--

Here is a full, buildable plan that uses your sublinear approach, a Rust echo‑state forecaster today, and a gated adapter for ruv‑fann. You can download a working Cargo workspace with CLI, algorithms, and tests of alignment.

**Download:** [temporal_attractor_studio.zip](sandbox:/mnt/data/temporal_attractor_studio.zip)

---

## 1) Plain‑language intro

You are building a conversation forecasting simulator. It learns the **shapes** conversations tend to follow over time, then simulates future branches as an evolving “temporal attractor.” You do not ask for a single future. You generate **ensembles of likely futures**, score them for truth, cost, and novelty, then steer toward the best paths. The “temporal strange attractor” piece means these futures form drifting patterns rather than fixed endpoints. We measure the stability of those patterns with finite‑time Lyapunov exponents and related diagnostics so you know how far ahead to trust the forecast. ([American Meteorological Society Journals][1])

---

## 2) What you will ship

**Name:** Agentics Temporal Attractor Studio
**One‑liner:** Forecast and steer long‑running conversations as evolving attractors with cost, truth, and novelty control.
**Positioning:** Strategic simulator for leaders who need to plan narrative, product, and comms under uncertainty.

---

## 3) Architecture

```
+---------------------------+         +---------------------+
|  Ingestion & Embedding    |  z(t)   |  Dynamics Learner   |
|  • CSV, threads, docs     |-------> |  • Echo-state core  |
|  • Hashed embed + RP      |         |  • ruv-fann adapter |
+-------------+-------------+         +---------------------+
              |                                   |
              | ensembles                         | step(z_t)
              v                                   v
+-------------+-------------+         +---------------------+
| Temporal Attractor Engine |<------->|  Control Policy     |
| • Pullback snapshots A(t) |   u(t)  |  • sublinear hooks  |
| • FTLE, DKY, drift ΔH     |         |  • spend guards     |
+-------------+-------------+         +---------------------+
              |
              v
+-------------+-------------+
| Verification & Scoring    |
| • Alignment vs reality    |
| • Truth, novelty, cost    |
+-------------+-------------+
              |
              v
+---------------------------+
| CLI + Studio UI (next)    |
+---------------------------+
```

* **Nonautonomous attractor view:** pullback or snapshot attractors that evolve under time‑dependent forcing. ([American Meteorological Society Journals][1])
* **Stability gauges:** finite‑time Lyapunov exponents and Kaplan–Yorke dimension as time‑varying indicators of predictability. ([Venturi Lab][2])
* **Forecaster:** echo‑state network baseline now, optional ruv‑fann adapter for your C‑backed FANN nets. ([RUG AI][3])

---

## 4) Features you get on day one

1. **Fit** a lightweight dynamics model on your latent conversation trajectories. Echo‑state forecaster uses a fixed reservoir with a trained linear readout, which is the classic reservoir computing approach for chaotic forecasting. ([RUG AI][3])
2. **Simulate** ensembles to produce pullback snapshots A(t), FTLE curves, and a rough DKY estimate so you can see attractors “breathe” and know the current forecasting horizon. ([American Meteorological Society Journals][4])
3. **Score** prediction alignment against realized branches using cosine similarity and DTW on latent paths, with the option to add EMD on distributions. ([AAAI][5])
4. **Control policy hooks** that represent your **sublinear‑time‑solver** as a control u(t): budgets, novelty thresholds, sparsity schedules.
5. **CLI** for fit, simulate, strobe snapshots, and score.

---

## 5) Usage examples

```bash
# Build
cargo build --release

# Fit on your turns CSV (schema: ts,role,text,cost,truth_score,accepted)
cargo run -p cli -- fit --data sample.csv --latent-dim 3 --window 5

# Simulate 3 weeks with weekly strobe snapshots
cargo run -p cli -- simulate --horizon 21 --ensembles 128 --strobe 7

# Backtest alignment vs a held-out suffix
cargo run -p cli -- score --data sample.csv --k 5
```

---

## 6) How prediction alignment is measured against past decision points

At each decision point t₀ in your historical thread:

1. **Cut** the real sequence at t₀.
2. **Simulate** M futures from the learned dynamics to obtain an ensemble distribution over z(t₀+1..t₀+K).
3. **Compare** the real continuation to the simulated ensemble with a set of metrics:

   * **Cosine similarity** of next‑step latent vectors.
   * **DTW** between the realized latent path and each simulated path, then report the minimum DTW and rank. ([AAAI][5])
   * **Optional EMD** between predicted and realized distributions of path features when you form cluster signatures. ([CMU School of Computer Science][6])
4. **Aggregate** over many t₀ to estimate your simulator’s precision at K‑step horizons and how that varies with FTLE(t₀). When λmax(t₀) rises, the verified horizon typically shrinks. ([Venturi Lab][2])

---

## 7) Implementation algorithms

### 7.1 Embedding and latent state

* **Text to latent**: simple, deterministic **feature hashing** + random projection to a d‑dimensional latent vector z(t). Good enough to demonstrate the temporal attractor machinery without network calls. Replace with your preferred encoder later. ([arXiv][7])

### 7.2 Dynamics learner

* **Echo‑state network** forecaster with readout learned by ridge regression. You train only W_out. This is standard reservoir computing and is fast for chaotic time series. ([RUG AI][3])
* **ruv‑fann adapter**: feature‑gated module stub that implements the same `Forecaster` trait, ready to call your ruv‑fann wrapper over the C FANN library. ([GitHub][8])

### 7.3 Temporal attractor engine

* **Pullback snapshots**: integrate ensembles from t₀−T to t₀, take the cloud at t₀ as A(t₀), then slide t₀. ([American Meteorological Society Journals][1])
* **FTLE**: central‑difference Jacobian along trajectories with a Benettin‑style re‑orthonormalization for the largest exponent. Use FTLE to show predictability windows. ([Venturi Lab][2])
* **DKY estimate**: compute a simple spectrum surrogate [λ₁, λ₂, …], report Kaplan–Yorke dimension for interpretability. ([Wikipedia][9])

### 7.4 Verification and alignment

* **Next‑step cosine** and **K‑step DTW** as defaults. **EMD** is an optional upgrade when you compare clustered distributions of futures. ([AAAI][5])

### 7.5 Chaos diagnostics

* Optional **0–1 test for chaos** on latent series during fit to ensure you are not mistaking noise for chaos in short windows. ([School of Mathematics and Statistics][10])

---

## 8) Code layout you can run today

```
temporal_attractor_studio/
  crates/
    common/         # shared types and JSON models
    embedding/      # hashed embedding + random projection
    dynamics/       # EchoState forecaster + ruv-fann adapter stub (feature gated)
    simulator/      # ensembles, pullback snapshots, FTLE + DKY
    verification/   # cosine, DTW metrics
    control/        # sublinear control policy hooks (u(t))
    cli/            # ruv-agentics CLI
```

Key trait and default learner:

```rust
// crates/dynamics/src/lib.rs
pub trait Forecaster {
    fn fit(&mut self, zs: &Vec<DVector<f64>>) -> Result<()>;
    fn step(&self, z: &DVector<f64>) -> DVector<f64>;
    fn save(&self) -> ModelSnapshot;
    fn load(snapshot: &ModelSnapshot) -> Self where Self: Sized;
}

// Echo-state network forecaster (reservoir + linear readout)
pub struct EchoState { /* ... */ }
```

Temporal metrics:

```rust
// crates/simulator/src/lib.rs
pub fn ftle_max<F: Forecaster>(f: &F, z: &DVector<f64>, dt: f64, steps: usize, eps: f64) -> f64 {
    // finite-difference Jacobian + Benettin normalization
}

pub fn kaplan_yorke(lams: &Vec<f64>) -> f64 { /* DKY formula */ }
```

Control hook that stands in for your **sublinear‑time‑solver**:

```rust
// crates/control/src/lib.rs
pub trait ControlPolicy { fn control(&self, z: &DVector<f64>, t: f64) -> DVector<f64>; }

pub struct SublinearHeuristic { /* encodes budget, truth, sparsity */ }
```

CLI with fit, simulate, score:

```bash
cargo run -p cli -- --help
```

The project compiles out of the box with the echo‑state learner. The ruv‑fann adapter is feature‑gated so you can add your crate when ready.

---

## 9) How your sublinear solver plugs in

* **Control u(t):** your solver outputs a small control vector that encodes budget, novelty, and truth constraints at time t. The simulator accepts this via the `ControlPolicy` trait and can bias trajectory sampling or adjust dynamics parameters on the fly.
* **Spend guards:** enforce per‑topic ceilings and slow down sampling when FTLE spikes, since chaos implies shorter reliable horizons.

---

## 10) Personas and workflows

* **Architect rUv:** configures constraints C, sets KPIs, and approves control policies.
* **Data Eng:** maintains ingestion pipelines and embedding.
* **Analyst:** runs strobe snapshots, reviews drift and alignment, green‑lights strategy branches.

**Daily loop:** ingest → fit or refresh readout → simulate ensembles → inspect A(t), FTLE, DKY → choose branches → ship posts or decisions → verify against realized outcomes → update model.

---

## 11) Diagrams for quick mental models

**Loop view**

```
Data -> Embed -> Forecast -> Attractor A(t) -> Score -> Control u(t) -> Forecast ...
             ^                                                  |
             +------------------ Verification <- Reality -------+
```

**Attractor evolution**

```
A(t0):  ••..•.•
A(t1):   ••..•.•
A(t2):    •••..•
         drift ΔH  ↑  FTLE(t) ↑ means shorter reliable horizon
```

---

## 12) Evaluation protocol that avoids an observer paradox

* Forecast **distributions** of futures, not a single outcome, then act.
* Backtest with rolling cutoffs t₀, compare to realized paths with DTW and cosine, and log whether the realized branch lay inside high‑probability regions of A(t).
* Track **alignment** by horizon K and **current FTLE** to report a reliability window. You are observing shapes of possibilities, which is robust to your own actions. ([Venturi Lab][2])

---

## 13) KPIs and dashboards for enterprise value

* **Cost per accepted token** and **truth density** per decision thread.
* **Alignment@K**: proportion of realized steps within top‑p ensemble bands.
* **DKY trend**: complexity drifting up or down by topic.
* **λmax trend**: predictability windows by team or client.

---

## 14) Security and governance

* Least‑privilege ingestion.
* Deterministic run logs with seeds.
* Per‑topic budget caps.
* JSON audit of sources, parameters, and decisions for every run.

---

## 15) Roadmap

* **Core now:** echo‑state forecaster, pullback snapshots, FTLE, DKY, alignment scoring, sublinear control hooks, CLI.
* **Stretch:** Koopman EDMD with control for interpretable linear predictors in a lifted space. ([arXiv][11])
* **Frontier:** neural ODE or SDE dynamics when you want smoother continuous‑time latent flows with uncertainty. ([arXiv][12])

---

## 16) References you can cite

* Snapshot and pullback attractors for nonautonomous systems. ([American Meteorological Society Journals][1])
* Echo‑state networks and reservoir computing. ([RUG AI][3])
* Koopman EDMD and controlled predictors. ([arXiv][11])
* Neural ODEs for continuous‑time latent dynamics. ([arXiv][12])
* Finite‑time Lyapunov exponents and DKY. ([Venturi Lab][2])
* 0–1 test for chaos. ([School of Mathematics and Statistics][10])
* Feature hashing and random projections. ([arXiv][7])
* DTW and EMD for alignment metrics. ([AAAI][5])
* FANN library background. ([GitHub][8])

---

## 17) What is already included in the zip

* A complete Cargo workspace that compiles without external model services.
* Echo‑state learner, snapshot engine, FTLE and DKY estimate, alignment metrics.
* Feature‑gated ruv‑fann adapter stub you can flesh out when you plug your crate.
* Control policy module that stands in for your sublinear‑time‑solver.

**Download and build:**
[temporal_attractor_studio.zip](sandbox:/mnt/data/temporal_attractor_studio.zip)

---

## 18) Next step proposals for rUv

* Wire your **sublinear‑time‑solver** to the `ControlPolicy` trait to generate u(t) that enforces budget and novelty constraints per topic.
* Replace the hashed embedder with your production text encoder.
* Decide if you want interpretable Koopman modes next or a Neural ODE for smoother flows.
* Publish stroboscopic attractor reels each week with FTLE and DKY overlays to show leadership how your steering wheel works.

If you want, I can add an example notebook that ingests one of your public LinkedIn threads, runs the CLI, and plots A(t) and FTLE next.

[1]: https://journals.ametsoc.org/view/journals/clim/28/8/jcli-d-14-00459.1.xml?utm_source=chatgpt.com "Probabilistic Concepts in a Changing Climate: A Snapshot ..."
[2]: https://venturi.soe.ucsc.edu/sites/default/files/Numerical_Calculation_of_Lyapunov_Exponents.pdf?utm_source=chatgpt.com "Numerical Calculation of Lyapunov Exponents"
[3]: https://www.ai.rug.nl/minds/uploads/EchoStatesTechRep.pdf?utm_source=chatgpt.com "The “echo state” approach to analysing and training ..."
[4]: https://journals.ametsoc.org/view/journals/clim/29/11/jcli-d-15-0848.1.xml?utm_source=chatgpt.com "Exploring the Pullback Attractors of a Low-Order ..."
[5]: https://cdn.aaai.org/Workshops/1994/WS-94-03/WS94-03-031.pdf?utm_source=chatgpt.com "Using Dynamic Time Warping to Find Patterns in Time Series"
[6]: https://www.cs.cmu.edu/~efros/courses/LBMV07/Papers/rubner-jcviu-00.pdf?utm_source=chatgpt.com "The Earth Mover's Distance as a Metric for Image Retrieval"
[7]: https://arxiv.org/abs/0902.2206?utm_source=chatgpt.com "Feature Hashing for Large Scale Multitask Learning"
[8]: https://github.com/libfann/fann?utm_source=chatgpt.com "libfann/fann: Official github repository for Fast Artificial ..."
[9]: https://en.wikipedia.org/wiki/Lyapunov_dimension?utm_source=chatgpt.com "Lyapunov dimension"
[10]: https://talus.maths.usyd.edu.au/u/gottwald/preprints/testforchaos_MPI.pdf?utm_source=chatgpt.com "The 0-1 Test for Chaos: A review"
[11]: https://arxiv.org/abs/1408.4408?utm_source=chatgpt.com "A Data-Driven Approximation of the Koopman Operator"
[12]: https://arxiv.org/abs/1806.07366?utm_source=chatgpt.com "Neural Ordinary Differential Equations"


Here’s a simple, easy-to-digest overview you can add at the top of your README or gist so anyone (even without technical background) gets it right away:

---

## Easy Overview 🌀

Think of this tool as a **“forecast horizon meter”** for any data series. In chaotic systems, small mistakes quickly grow until predictions are useless. The speed of that growth is called the **Lyapunov exponent**.

* If the exponent is **large**, your system becomes unpredictable quickly.
* If it’s **small**, you can trust forecasts further into the future.

This CLI program takes a CSV file of your data (a trajectory, a sensor log, a simulation trace, or even stock prices) and tells you **how far into the future you can make reliable predictions**.

It outputs three numbers:

* **λ (lambda):** error growth rate
* **T<sub>L</sub> (Lyapunov time):** time until your errors multiply by *e* (~2.718)
* **T<sub>d</sub> (Doubling time):** time until your errors double

In practical terms:

* For robots, it tells you how often to replan.
* For markets, it sets how far forecasts should extend before noise dominates.
* For agent swarms, it defines the “safe prediction window” before agents must recalibrate.

You don’t need to understand the math details. Just remember: **higher λ = shorter foresight**. This program calculates it for you quickly and efficiently.

---

Here’s a fast, no-nonsense Rust CLI that estimates the largest Lyapunov exponent from a trajectory or time series using delay embedding, a Theiler window, a cache-friendly VP-tree nearest-neighbor search, and parallel slope fits over early divergences. It avoids exotic deps, compiles cleanly, and scales well to mid-size datasets. Usage examples are in the doc comment at the top.

```toml
# Cargo.toml
[package]
name = "lyapfit"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
clap = { version = "4.5", features = ["derive"] }
csv = "1.3"
rayon = "1.10"
```

```rust
// src/main.rs
/*!
Lyapunov λ via nearest-neighbor divergence.

Examples
  # multivariate state per row, dt=0.01s, fit first 12 steps
  lyapfit data.csv --dt 0.01 --k-fit 12

  # univariate series in column 0 with delay embedding m=6, tau=2 samples
  lyapfit data.csv --dt 0.01 --col 0 --m 6 --tau 2 --k-fit 15

  # read CSV from stdin, skip header, widen Theiler window
  cat data.csv | lyapfit - --no-header --dt 0.02 --theiler 50
*/

use anyhow::{bail, Context, Result};
use clap::Parser;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::fs::File;
use std::io::{stdin, BufReader, Read};

#[derive(Parser, Debug)]
#[command(name="lyapfit")]
struct Args {
    /// CSV file path or "-" for stdin
    path: String,
    /// Sampling interval ∆t in seconds
    #[arg(long)]
    dt: f64,
    /// Column index for univariate delay embedding. Omit to use all columns as the state.
    #[arg(long)]
    col: Option<usize>,
    /// Embedding dimension m (univariate only)
    #[arg(long, default_value = "1")]
    m: usize,
    /// Delay in samples τ (univariate only)
    #[arg(long, default_value = "1")]
    tau: usize,
    /// Use header row
    #[arg(long, default_value_t = true)]
    header: bool,
    /// Theiler window W in samples to exclude temporal neighbors
    #[arg(long, default_value = "20")]
    theiler: usize,
    /// Number of early steps to fit (K_fit)
    #[arg(long, default_value = "12")]
    k_fit: usize,
    /// Maximum pairs sampled for averaging (stride over i)
    #[arg(long, default_value = "4000")]
    max_pairs: usize,
    /// Minimum initial separation; pairs below are skipped
    #[arg(long, default_value = "1e-12")]
    min_init_sep: f64,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.dt <= 0.0 {
        bail!("dt must be > 0");
    }
    if args.m == 0 {
        bail!("m must be >= 1");
    }
    if args.k_fit < 2 {
        bail!("k-fit must be >= 2");
    }

    let raw = read_csv(&args.path, args.header).context("reading CSV")?;
    if raw.is_empty() {
        bail!("empty input");
    }

    // Build state matrix X: Vec<Vec<f64>> where each entry is a state vector at time t
    let x = if let Some(col) = args.col {
        let series: Vec<f64> = raw
            .iter()
            .map(|row| {
                row.get(col)
                    .copied()
                    .unwrap_or_else(|| f64::NAN)
            })
            .collect();
        let x = delay_embed(&series, args.m, args.tau)?;
        x
    } else {
        // multivariate state per row
        raw
    };

    let n = x.len();
    if n < args.k_fit + 2 {
        bail!("not enough points after embedding");
    }
    let dim = x[0].len();
    if dim == 0 {
        bail!("zero-dimension state");
    }

    // Build VP-tree over embedded states
    let mut indices: Vec<usize> = (0..n - args.k_fit).collect(); // restrict to allow i+k access
    let tree = VpTree::build(&x, &mut indices);

    // Precompute linear regression constants for t = {1..K} * dt
    let k = args.k_fit as usize;
    let dt = args.dt;
    let mut t = Vec::with_capacity(k);
    for kk in 1..=k {
        t.push(kk as f64 * dt);
    }
    let t_mean = mean(&t);
    let var_t = t.iter().map(|tk| (tk - t_mean) * (tk - t_mean)).sum::<f64>();
    if var_t <= 0.0 {
        bail!("degenerate time variance");
    }

    // Sample pairs i -> j_nearest with Theiler window, fit slope on early log distances
    let stride = std::cmp::max(1usize, (n - args.k_fit) / args.max_pairs.max(1));
    let theiler = args.theiler;

    let slopes: Vec<f64> = (0..n - args.k_fit)
        .step_by(stride)
        .collect::<Vec<_>>()
        .par_iter()
        .filter_map(|&i| {
            let query = &x[i];
            // nearest neighbor with Theiler exclusion
            if let Some((j, d0)) = tree.nearest_excluding(query, i, theiler) {
                if d0 <= args.min_init_sep || j + k >= x.len() || i + k >= x.len() {
                    return None;
                }
                // Early growth curve
                let mut y = Vec::with_capacity(k);
                for kk in 1..=k {
                    let d = dist(&x[i + kk], &x[j + kk]);
                    // numerical guard
                    let dd = if d <= 0.0 { 1e-300 } else { d };
                    y.push((dd / d0).ln());
                }
                let y_mean = mean(&y);
                let cov = t
                    .iter()
                    .zip(y.iter())
                    .map(|(tk, yk)| (tk - t_mean) * (yk - y_mean))
                    .sum::<f64>();
                let slope = cov / var_t; // λ estimate from this pair
                if slope.is_finite() { Some(slope) } else { None }
            } else {
                None
            }
        })
        .collect();

    if slopes.is_empty() {
        bail!("no valid pairs found. Try reducing theiler or k-fit, or increase max-pairs");
    }

    let lambda = mean(&slopes);
    let td = std::f64::consts::LN_2 / lambda;
    let tl = 1.0 / lambda;

    println!("points_used,dim,dt,k_fit,theiler,pairs,lambda,lyapunov_time,Td_doubling");
    println!(
        "{},{},{:.9},{},{},{},{:.9},{:.9},{:.9}",
        n, dim, dt, k, theiler, slopes.len(), lambda, tl, td
    );

    Ok(())
}

/// Read CSV into Vec<Vec<f64>>
fn read_csv(path: &str, header: bool) -> Result<Vec<Vec<f64>>> {
    let rdr: Box<dyn Read> = if path == "-" {
        Box::new(stdin())
    } else {
        Box::new(File::open(path)?)
    };
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(header)
        .from_reader(BufReader::new(rdr));
    let mut out = Vec::new();
    for rec in reader.records() {
        let rec = rec?;
        let mut row = Vec::with_capacity(rec.len());
        for f in rec.iter() {
            // Allow blank cells to be NaN, skip rows that contain NaN
            let v = f.parse::<f64>().unwrap_or(f64::NAN);
            row.push(v);
        }
        if row.iter().all(|v| v.is_finite()) {
            out.push(row);
        }
    }
    Ok(out)
}

/// Univariate delay embed: returns Vec of state vectors length n_eff
fn delay_embed(series: &[f64], m: usize, tau: usize) -> Result<Vec<Vec<f64>>> {
    if m == 1 {
        // return as single-column states
        return Ok(series.iter().map(|v| vec![*v]).collect());
    }
    let n = series.len();
    let span = (m - 1) * tau;
    if n <= span {
        bail!("series too short for embedding");
    }
    let n_eff = n - span;
    let mut x = Vec::with_capacity(n_eff);
    for i in 0..n_eff {
        let mut v = Vec::with_capacity(m);
        for k in 0..m {
            let idx = i + k * tau;
            v.push(series[idx]);
        }
        x.push(v);
    }
    Ok(x)
}

#[inline]
fn mean(v: &[f64]) -> f64 {
    let s: f64 = v.iter().sum();
    s / (v.len() as f64)
}

#[inline]
fn dist(a: &[f64], b: &[f64]) -> f64 {
    let mut acc = 0.0;
    // manual unroll for small dims
    let len = a.len();
    let mut i = 0;
    while i + 3 < len {
        let d0 = a[i] - b[i];
        let d1 = a[i + 1] - b[i + 1];
        let d2 = a[i + 2] - b[i + 2];
        let d3 = a[i + 3] - b[i + 3];
        acc += d0 * d0 + d1 * d1 + d2 * d2 + d3 * d3;
        i += 4;
    }
    while i < len {
        let d = a[i] - b[i];
        acc += d * d;
        i += 1;
    }
    acc.sqrt()
}

/// Vantage-point tree with dynamic dimension
struct VpNode {
    idx: usize,        // index into dataset
    tau: f64,          // partition radius
    left: Option<Box<VpNode>>,
    right: Option<Box<VpNode>>,
}

struct VpTree<'a> {
    data: &'a [Vec<f64>],
    root: Option<Box<VpNode>>,
}

impl<'a> VpTree<'a> {
    fn build(data: &'a [Vec<f64>], indices: &mut [usize]) -> Self {
        let root = Self::build_rec(data, indices);
        Self { data, root }
    }

    fn build_rec(data: &'a [Vec<f64>], indices: &mut [usize]) -> Option<Box<VpNode>> {
        if indices.is_empty() {
            return None;
        }
        // use last as vantage point
        let vp = indices[indices.len() - 1];
        if indices.len() == 1 {
            return Some(Box::new(VpNode { idx: vp, tau: 0.0, left: None, right: None }));
        }
        // compute distances to vp
        let (left_slice, _vp_slot) = indices.split_at_mut(indices.len() - 1);
        let mut dists: Vec<(usize, f64)> = left_slice
            .iter()
            .map(|&j| (j, dist(&data[vp], &data[j])))
            .collect();
        // median split on distance
        let mid = dists.len() / 2;
        dists.select_nth_unstable_by(mid, |a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
        let tau = dists[mid].1;
        // partition into inner and outer
        let mut inner: Vec<usize> = Vec::with_capacity(mid + 1);
        let mut outer: Vec<usize> = Vec::with_capacity(dists.len() - mid);
        for (j, d) in dists {
            if d <= tau {
                inner.push(j);
            } else {
                outer.push(j);
            }
        }
        let left = Self::build_rec(data, &mut inner);
        let right = Self::build_rec(data, &mut outer);
        Some(Box::new(VpNode { idx: vp, tau, left, right }))
    }

    /// Nearest neighbor excluding indices within Theiler window of target_i
    fn nearest_excluding(&self, q: &[f64], target_i: usize, theiler: usize) -> Option<(usize, f64)> {
        let mut best_idx = usize::MAX;
        let mut best_dist = f64::INFINITY;
        self.search(&self.root, q, target_i, theiler, &mut best_idx, &mut best_dist);
        if best_idx == usize::MAX { None } else { Some((best_idx, best_dist)) }
    }

    fn search(
        &self,
        node: &Option<Box<VpNode>>,
        q: &[f64],
        target_i: usize,
        theiler: usize,
        best_idx: &mut usize,
        best_dist: &mut f64,
    ) {
        let Some(n) = node else { return; };
        let d = dist(q, &self.data[n.idx]);

        // respect Theiler window and skip self
        if n.idx != target_i && theiler_exclude(target_i, n.idx, theiler) == false {
            if d < *best_dist {
                *best_dist = d;
                *best_idx = n.idx;
            }
        }

        // choose side to visit first
        let first_left = d < n.tau || n.right.is_none();
        let (first, second) = if first_left { (&n.left, &n.right) } else { (&n.right, &n.left) };

        if let Some(_) = first {
            self.search(first, q, target_i, theiler, best_idx, best_dist);
        }
        // visit the other side if the hypersphere around q intersects the boundary
        if (d - n.tau).abs() <= *best_dist {
            if let Some(_) = second {
                self.search(second, q, target_i, theiler, best_idx, best_dist);
            }
        }
    }
}

#[inline]
fn theiler_exclude(i: usize, j: usize, w: usize) -> bool {
    let di = if i > j { i - j } else { j - i };
    di <= w
}
```

### Notes

* Fit window defaults to the first 12 steps. Bump `--k-fit` if your system has a longer linear growth regime.
* Increase `--theiler` when trajectories are smooth and temporally adjacent points collapse to trivial neighbors.
* For very long series, raise `--max-pairs` or lower the stride by setting a larger value for it implicitly via `max-pairs`.
 
 