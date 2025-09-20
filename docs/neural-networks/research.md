Here is a clear, copyable plan you can implement and A-B test against a traditional micro-net. It is stack aligned, measurable, and fast to prove.

# 0) Objective and hypothesis

* Goal: cut end-to-end loop time and improve stability for short-horizon prediction while holding accuracy.
* Hypothesis: adding a tiny prior, solver-based gating, and graph-aware sampling reduces latency and error tails vs a traditional micro-net of the same size.

# 1) Two systems to compare

**System A: Traditional micro-net**

* Residual GRU or TCN, hidden 32 or 64, fp32 train, int8 infer.
* Input window 128 ms. Predict t+500 ms.

**System B: Temporal micro-net with solver**

* Same net size and window.
* Baseline prior: constant velocity or Kalman filter.
* Sublinear solver gate: `solve_projection(t, eps, budget)` returns `{estimate, cert.error, work}`.
* Active selection: `pagerank_local` on kNN of embeddings to pick high-value samples.

# 2) Data and splits

* Use at least 2 domains to avoid overfitting conclusions:

  1. Human cursor or touch trajectories at 1 to 2 kHz.
  2. IMU pose stream or price micro-ticks.
* Preprocess: resample to fixed rate, z-score per feature, create sliding windows.
* Temporal split: first 70 percent train, next 15 percent val, final 15 percent test. No shuffling across time.

# 3) Model configs

```yaml
common:
  horizon_ms: 500
  window_ms: 128
  sample_rate_hz: 2000
  features: [x, y, vx, vy]   # adjust per domain
  quantize: int8
  optimizer: adam
  lr: 1e-3
  batch: 256
  epochs: 15

A_traditional:
  model: micro_gru
  hidden: 32

B_temporal_solver:
  model: micro_gru
  hidden: 32
  prior: kalman
  solver_gate:
    eps: 0.02
    budget: 200000
  active_selection:
    k: 15
    eps: 0.03
```

# 4) Training protocol

* Both A and B train on the same batches for the first 2 epochs.
* From epoch 3, B switches to active selection:

  1. Embed last layer features on train set.
  2. Build kNN graph.
  3. Score with `pagerank_local(query=recent_errors)`.
  4. Sample top K windows for the next epoch.
* Loss: MSE at horizon plus smoothness penalty on velocity. Same loss for A and B.
* Early stopping on val MSE at 500 ms.

# 5) Inference loop design

**A**
`y_hat = net(x_window)`

**B**

```
y_prior = kalman_step(state)
y_resid = net(x_window)
y_hat = y_prior + y_resid
cert = sublinear.solve_projection(t=jacobian_row, eps=0.02, budget=2e5)
if cert.error > 0.02: y_hat = hold_last_safe()
```

# 6) Deployment targets and budgets

* CPU only, single core. Pin thread and lock memory.
* Latency budget per tick:

  * Ingest 0.10 ms
  * Prior 0.10 ms
  * Net 0.30 ms
  * Gate 0.20 ms
  * Actuation 0.10 ms
* Total target p99.9 ≤ 0.90 ms.

# 7) Metrics and logging

* Accuracy: MSE\_500ms, MAE\_500ms, p90 and p99 absolute error.
* Stability: missed\_deadlines per 2 minutes at 2 kHz.
* Latency: p50 and p99.9 for predict call.
* Reliability: gate pass\_rate, average `cert.error`, replan count.
* Cost: tokens saved if used in RAG pruning, or CPU seconds per hour.
* Log as CSV: `ts, wall_us, mse_500, p99_ae_500, passed, cert_error, work`.

# 8) Statistical test

* For each domain, compute paired differences of MSE\_500ms and p99 latency over N runs with different seeds.
* Use paired t-test on MSE and a Mann-Whitney on latency tails.
* Report mean, 95 percent CI, and effect size.

# 9) Ablations

* Remove prior in B to isolate solver benefit.
* Keep prior, remove solver gate.
* Vary `eps` in {0.01, 0.02, 0.05}.
* Hidden size {16, 32, 64} to show compute vs accuracy curve.

# 10) Success criteria

* B reduces p99.9 latency by at least 20 percent with no loss in MSE, or
* B keeps latency equal and reduces p99 error by at least 15 percent, and
* Gate pass\_rate ≥ 90 percent with average `cert.error` ≤ 0.02.

# 11) Stack wiring

**Claude-Flow**

* Training workflow: embed, build kNN, `sublinear.pagerank_local`, train step, validate, log.
* Verification workflow: after each predict, call `solve_projection` and enforce the gate.

**Flow-Nexus**

* `/mcp__flow-nexus__workflow_create name="temporal-train"`
* `/mcp__flow-nexus__workflow_execute name="temporal-train" args='{"horizon_ms":500,"window_ms":128,"hidden":32}'`
* Use stream logs for `{wall_us, cert_error, pass_rate}`.

**ruv-fann**

* Export int8 weights for the residual.
* SIMD matvecs for the hot path.

# 12) Repro scripts

```bash
# Train A and B
python train.py --exp A_traditional.yaml
python train.py --exp B_temporal_solver.yaml

# Evaluate
python eval.py --exp A_traditional.yaml --out results_A.csv
python eval.py --exp B_temporal_solver.yaml --out results_B.csv

# Compare
python compare.py --a results_A.csv --b results_B.csv --report ab_report.md
```

# 13) Reporting template

* One table per domain with Accuracy, Latency, Stability.
* One plot of error CDFs and one of latency histograms.
* A short narrative: where B wins, where it ties, failure cases, and knobs that move the curve.

# 0) Objective and hypothesis

* Goal: cut end-to-end loop time and improve stability for short-horizon prediction while holding accuracy.
* Hypothesis: adding a tiny prior, solver-based gating, and graph-aware sampling reduces latency and error tails vs a traditional micro-net of the same size.

# 1) Two systems to compare

**System A: Traditional micro-net**

* Residual GRU or TCN, hidden 32 or 64, fp32 train, int8 infer.
* Input window 128 ms. Predict t+500 ms.

**System B: Temporal micro-net with solver**

* Same net size and window.
* Baseline prior: constant velocity or Kalman filter.
* Sublinear solver gate: `solve_projection(t, eps, budget)` returns `{estimate, cert.error, work}`.
* Active selection: `pagerank_local` on kNN of embeddings to pick high-value samples.

# 2) Data and splits

* Use at least 2 domains to avoid overfitting conclusions:

  1. Human cursor or touch trajectories at 1 to 2 kHz.
  2. IMU pose stream or price micro-ticks.
* Preprocess: resample to fixed rate, z-score per feature, create sliding windows.
* Temporal split: first 70 percent train, next 15 percent val, final 15 percent test. No shuffling across time.

# 3) Model configs

```yaml
common:
  horizon_ms: 500
  window_ms: 128
  sample_rate_hz: 2000
  features: [x, y, vx, vy]   # adjust per domain
  quantize: int8
  optimizer: adam
  lr: 1e-3
  batch: 256
  epochs: 15

A_traditional:
  model: micro_gru
  hidden: 32

B_temporal_solver:
  model: micro_gru
  hidden: 32
  prior: kalman
  solver_gate:
    eps: 0.02
    budget: 200000
  active_selection:
    k: 15
    eps: 0.03
```

# 4) Training protocol

* Both A and B train on the same batches for the first 2 epochs.
* From epoch 3, B switches to active selection:

  1. Embed last layer features on train set.
  2. Build kNN graph.
  3. Score with `pagerank_local(query=recent_errors)`.
  4. Sample top K windows for the next epoch.
* Loss: MSE at horizon plus smoothness penalty on velocity. Same loss for A and B.
* Early stopping on val MSE at 500 ms.

# 5) Inference loop design

**A**
`y_hat = net(x_window)`

**B**

```
y_prior = kalman_step(state)
y_resid = net(x_window)
y_hat = y_prior + y_resid
cert = sublinear.solve_projection(t=jacobian_row, eps=0.02, budget=2e5)
if cert.error > 0.02: y_hat = hold_last_safe()
```

# 6) Deployment targets and budgets

* CPU only, single core. Pin thread and lock memory.
* Latency budget per tick:

  * Ingest 0.10 ms
  * Prior 0.10 ms
  * Net 0.30 ms
  * Gate 0.20 ms
  * Actuation 0.10 ms
* Total target p99.9 ≤ 0.90 ms.

# 7) Metrics and logging

* Accuracy: MSE\_500ms, MAE\_500ms, p90 and p99 absolute error.
* Stability: missed\_deadlines per 2 minutes at 2 kHz.
* Latency: p50 and p99.9 for predict call.
* Reliability: gate pass\_rate, average `cert.error`, replan count.
* Cost: tokens saved if used in RAG pruning, or CPU seconds per hour.
* Log as CSV: `ts, wall_us, mse_500, p99_ae_500, passed, cert_error, work`.

# 8) Statistical test

* For each domain, compute paired differences of MSE\_500ms and p99 latency over N runs with different seeds.
* Use paired t-test on MSE and a Mann-Whitney on latency tails.
* Report mean, 95 percent CI, and effect size.

# 9) Ablations

* Remove prior in B to isolate solver benefit.
* Keep prior, remove solver gate.
* Vary `eps` in {0.01, 0.02, 0.05}.
* Hidden size {16, 32, 64} to show compute vs accuracy curve.

# 10) Success criteria

* B reduces p99.9 latency by at least 20 percent with no loss in MSE, or
* B keeps latency equal and reduces p99 error by at least 15 percent, and
* Gate pass\_rate ≥ 90 percent with average `cert.error` ≤ 0.02.

# 11) Stack wiring

**Claude-Flow**

* Training workflow: embed, build kNN, `sublinear.pagerank_local`, train step, validate, log.
* Verification workflow: after each predict, call `solve_projection` and enforce the gate.

**Flow-Nexus**

* `/mcp__flow-nexus__workflow_create name="temporal-train"`
* `/mcp__flow-nexus__workflow_execute name="temporal-train" args='{"horizon_ms":500,"window_ms":128,"hidden":32}'`
* Use stream logs for `{wall_us, cert_error, pass_rate}`.

**ruv-fann**

* Export int8 weights for the residual.
* SIMD matvecs for the hot path.

# 12) Repro scripts

```bash
# Train A and B
python train.py --exp A_traditional.yaml
python train.py --exp B_temporal_solver.yaml

# Evaluate
python eval.py --exp A_traditional.yaml --out results_A.csv
python eval.py --exp B_temporal_solver.yaml --out results_B.csv

# Compare
python compare.py --a results_A.csv --b results_B.csv --report ab_report.md
```

# 13) Reporting template

* One table per domain with Accuracy, Latency, Stability.
* One plot of error CDFs and one of latency histograms.
* A short narrative: where B wins, where it ties, failure cases, and knobs that move the curve.

If you want, I can generate the YAMLs, the CSV schemas, and a minimal PyTorch plus Rust stub so you can run the first A-B tonight. In the end, the win is simple: decide sooner, prove it, and move.
