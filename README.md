# Sublinear Time Solver

![A sparse matrix with a small neighborhood highlighted. The solver can focus on the entries that matter.](docs/assets/solver-hero.svg)

[![npm version](https://img.shields.io/npm/v/sublinear-time-solver.svg)](https://www.npmjs.com/package/sublinear-time-solver)
[![npm downloads](https://img.shields.io/npm/dm/sublinear-time-solver.svg)](https://www.npmjs.com/package/sublinear-time-solver)
[![Rust crate](https://img.shields.io/crates/v/sublinear.svg)](https://crates.io/crates/sublinear)

Solve systems of equations, estimate individual values, and rank nodes in a graph. Use the command line, the JavaScript API, the Rust crate, or an MCP client.

The idea is simple. If a problem has many variables but each one depends on only a few others, the solver can sometimes answer a question by visiting a small neighborhood. Think of a sensor network where one reading changes and you want to check nearby readings first.

[Quick start](#quick-start) · [How it works](#how-it-works) · [MCP](#use-it-with-an-mcp-client) · [Benchmarks](#performance)

## Quick start

With Node.js and npm, paste this into a terminal in a scratch directory. The matrix uses coordinate format (COO), which lists the value, row, and column of each nonzero entry.

```bash
cat > matrix.json <<'JSON'
{
  "rows": 2, "cols": 2, "format": "coo",
  "values": [4, -1, -1, 4],
  "rowIndices": [0, 0, 1, 1],
  "colIndices": [0, 1, 0, 1]
}
JSON
printf '%s\n' '[3,3]' > vector.json
npx -y sublinear-time-solver solve --matrix matrix.json --vector vector.json \
  --method forward-push --output solution.json
cat solution.json
```

The two values in `solution` should be close to `[1, 1]`, and `metadata.converged` should be `true`. Run `npx -y sublinear-time-solver --help` to see commands for inspecting matrices, generating test inputs, and running PageRank.

## How it works

![A changed row lights up, the solver follows its nearby dependencies, and the highlighted entries become a small set of updates.](docs/assets/local-update.svg)

A sparse matrix stores only the relationships that exist. A local solve starts with the row you care about, follows nearby dependencies, and returns a value or a small set of updates. The cost depends on how large that neighborhood gets and how much accuracy you need. A full solve returns every value, so its output grows with the size of the system.

| If you need | Start with |
| --- | --- |
| Every value in a system of equations | CLI or MCP `solve` |
| An estimate for one entry | MCP `estimateEntry` |
| A small set of updates after a local change | The [Rust event driven example](examples/event_driven_anomaly.rs) on the development branch |
| An importance score for each node in a graph | CLI `pagerank` or MCP `pageRank` |

The local path works best when rows have few neighbors, the diagonal terms dominate, and the change stays local. Inspect your matrix with `analyze` and check the residual when you need to know how closely a solution satisfies the equations.

## Use it with an MCP client

Add this to your client's MCP server configuration. It starts the server over standard input and output.

```json
{
  "mcpServers": {
    "sublinear-solver": {
      "command": "npx",
      "args": ["-y", "sublinear-time-solver", "serve"]
    }
  }
}
```

Try asking your client to list the solver tools, then pass the COO matrix and vector from the quick start to `solve`. Clients can also inspect complexity metadata before choosing a tool. The [MCP server source](src/mcp/server.ts) has the tool schemas, and [ADR 001](docs/adr/ADR-001-complexity-as-architecture.md) explains the complexity budget.

## Use it in code

| Environment | Start here |
| --- | --- |
| JavaScript and TypeScript | Run `npm install sublinear-time-solver` and import `SublinearSolver`. See the [solver implementation](src/core/solver.ts) and [input and result types](src/core/types.ts). |
| Rust | Run `cargo add sublinear` and use the `sublinear_solver` library. See the [crate documentation](https://docs.rs/sublinear/). |
| Development branch | Clone this repository and run `cargo run --release --example event_driven_anomaly` to see which entries change after a sensor event. |

## Performance

![In one sparse matrix benchmark, the full solve took 66.7 microseconds at 256 rows and 258 microseconds at 1,024 rows. The local update took 2.28 and 2.32 milliseconds, respectively.](docs/assets/benchmark-scaling.svg)

On this benchmark, quadrupling the number of rows took the full solve from **66.7 µs to 258 µs** and the local update from **2.28 ms to 2.32 ms**. The full solve was faster at both sizes. The local update grew more slowly because the changed neighborhood stayed small. See the [benchmark setup and results](BENCHMARK.md).

## More resources

[Changelog and upgrade notes](CHANGELOG.md) · [Security policy](SECURITY.md) · [Issue tracker](https://github.com/ruvnet/sublinear-time-solver/issues) · [License](LICENSE)

The repository also includes [temporal lead solving](crates/temporal-lead-solver/), a [nanosecond scheduler](https://crates.io/crates/nanosecond-scheduler), and experimental [reasoning](crates/psycho-symbolic-reasoner/) and [consciousness exploration](src/consciousness-explorer/) modules.
