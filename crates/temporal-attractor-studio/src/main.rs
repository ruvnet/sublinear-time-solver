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
use std::fs::File;
use std::io::{stdin, BufReader, Read};
use temporal_attractor_studio::{VpTree, delay_embed, dist, mean, theiler_exclude};

#[derive(Parser, Debug)]
#[command(name="lyapfit")]
#[command(about="Estimate largest Lyapunov exponent via nearest-neighbor divergence")]
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

    // Validate parameters
    if args.dt <= 0.0 {
        bail!("dt must be > 0");
    }
    if args.m == 0 {
        bail!("m must be >= 1");
    }
    if args.k_fit < 2 {
        bail!("k-fit must be >= 2");
    }

    // Read CSV data
    let raw = read_csv(&args.path, args.header).context("reading CSV")?;
    if raw.is_empty() {
        bail!("empty input");
    }

    // Build state matrix X: Vec<Vec<f64>> where each entry is a state vector at time t
    let x = if let Some(col) = args.col {
        // Univariate series - extract column and perform delay embedding
        let series: Vec<f64> = raw
            .iter()
            .map(|row| {
                row.get(col)
                    .copied()
                    .unwrap_or_else(|| f64::NAN)
            })
            .collect();
        delay_embed(&series, args.m, args.tau).context("delay embedding")?
    } else {
        // Multivariate state per row
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
    let k = args.k_fit;
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
        let mut row = Vec::new();
        for field in rec.iter() {
            match field.parse::<f64>() {
                Ok(val) => row.push(val),
                Err(_) => {
                    // Skip non-numeric fields or use NaN
                    row.push(f64::NAN);
                }
            }
        }
        if !row.is_empty() {
            out.push(row);
        }
    }

    Ok(out)
}