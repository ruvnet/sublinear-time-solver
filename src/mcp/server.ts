/**
 * MCP Server for Sublinear-Time Solver
 * Provides MCP interface to the core solver algorithms
 */

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ErrorCode,
  ListToolsRequestSchema,
  McpError,
} from '@modelcontextprotocol/sdk/types.js';

import { SublinearSolver } from '../core/solver.js';
import { MatrixOperations } from '../core/matrix.js';
import { TemporalTools } from './tools/temporal.js';
import { PsychoSymbolicTools } from './tools/psycho-symbolic.js';
import { DynamicPsychoSymbolicTools } from './tools/psycho-symbolic-dynamic.js';
import { DomainManagementTools } from './tools/domain-management.js';
import { DomainValidationTools } from './tools/domain-validation.js';
import { ConsciousnessTools } from './tools/consciousness.js';
// import { ConsciousnessEnhancedTools } from './tools/consciousness-enhanced.js';
import { EmergenceTools } from './tools/emergence-tools.js';
import { SchedulerTools } from './tools/scheduler.js';
import { CompleteWasmSublinearSolverTools as WasmSublinearSolverTools } from './tools/wasm-sublinear-complete.js';
import { TrueSublinearSolverTools } from './tools/true-sublinear-solver.js';
// SECURITY (issue #19, CWE-73): confine attacker-controlled file_path
// arguments (saveVectorToFile / vector_file) to a dedicated vector dir.
import {
  safeWriteVector,
  safeReadVector,
  resolveVectorPath,
  SafePathError,
  DEFAULT_VECTOR_DIR,
} from './safe-path.js';
import {
  Matrix,
  Vector,
  SolverConfig,
  SolveParams,
  EstimateEntryParams,
  AnalyzeMatrixParams,
  PageRankParams,
  SolverError,
  ErrorCodes
} from '../core/types.js';

export class SublinearSolverMCPServer {
  private server: Server;
  private solvers: Map<string, SublinearSolver> = new Map();
  private temporalTools: TemporalTools;
  private psychoSymbolicTools: PsychoSymbolicTools;
  private dynamicPsychoSymbolicTools: DynamicPsychoSymbolicTools;
  private domainManagementTools: DomainManagementTools;
  private domainValidationTools: DomainValidationTools;
  private consciousnessTools: ConsciousnessTools;
  // private consciousnessEnhancedTools: ConsciousnessEnhancedTools;
  private emergenceTools: EmergenceTools;
  private schedulerTools: SchedulerTools;
  private wasmSolver: WasmSublinearSolverTools;
  private trueSublinearSolver: TrueSublinearSolverTools;

  constructor() {
    this.temporalTools = new TemporalTools();
    this.psychoSymbolicTools = new PsychoSymbolicTools();
    this.domainManagementTools = new DomainManagementTools();
    // Share the same domain registry between all domain tools
    const sharedRegistry = this.domainManagementTools.getDomainRegistry();
    this.dynamicPsychoSymbolicTools = new DynamicPsychoSymbolicTools(sharedRegistry);
    this.domainValidationTools = new DomainValidationTools(sharedRegistry);
    this.consciousnessTools = new ConsciousnessTools();
    // this.consciousnessEnhancedTools = new ConsciousnessEnhancedTools();
    this.emergenceTools = new EmergenceTools();
    this.schedulerTools = new SchedulerTools();
    this.wasmSolver = new WasmSublinearSolverTools();
    this.trueSublinearSolver = new TrueSublinearSolverTools();
    this.server = new Server(
      {
        name: 'sublinear-solver',
        version: '1.0.0',
      },
      {
        capabilities: {
          tools: {},
        },
      }
    );

    this.setupToolHandlers();
    this.setupErrorHandling();
  }

  private setupToolHandlers(): void {
    this.server.setRequestHandler(ListToolsRequestSchema, async () => ({
      tools: [
        {
          name: 'solve',
          description: 'Solve a diagonally dominant linear system Mx = b',
          // ADR-001 item #4: advertise the worst-case complexity class at
          // tool-list time so clients can refuse a budget-bust call without
          // first making it. `x-complexity` is a non-standard JSON Schema
          // extension; clients that don't understand it ignore it.
          'x-complexity': {
            class: 'Linear',
            detail: 'O(k · nnz(M)) per iter; k bounded by maxIterations + epsilon.',
            edgeSafe: false,
          },
          inputSchema: {
            type: 'object',
            properties: {
              max_complexity_class: {
                type: 'string',
                enum: [
                  'Logarithmic', 'PolyLogarithmic', 'SubLinear', 'Linear',
                  'QuasiLinear', 'SubQuadratic', 'Polynomial',
                  'SuperPolynomial', 'SubExponential', 'Exponential',
                  'Factorial', 'DoubleExponential',
                ],
                description: 'ADR-001 item #4 — caller-supplied budget cap. If the chosen method\'s complexity class exceeds this, the response includes a warning. Phase-2 will harden to an error.',
              },
              matrix: {
                type: 'object',
                description: 'Matrix M in dense or sparse format',
                properties: {
                  rows: { type: 'number' },
                  cols: { type: 'number' },
                  format: { type: 'string', enum: ['dense', 'coo'] },
                  data: {
                    oneOf: [
                      { type: 'array', items: { type: 'array', items: { type: 'number' } } },
                      {
                        type: 'object',
                        properties: {
                          values: { type: 'array', items: { type: 'number' } },
                          rowIndices: { type: 'array', items: { type: 'number' } },
                          colIndices: { type: 'array', items: { type: 'number' } }
                        },
                        required: ['values', 'rowIndices', 'colIndices']
                      }
                    ]
                  }
                },
                required: ['rows', 'cols', 'format', 'data']
              },
              vector: {
                type: 'array',
                items: { type: 'number' },
                description: 'Right-hand side vector b'
              },
              method: {
                type: 'string',
                enum: ['neumann', 'random-walk', 'forward-push', 'backward-push', 'bidirectional'],
                default: 'neumann',
                description: 'Solver method to use'
              },
              epsilon: {
                type: 'number',
                default: 1e-6,
                description: 'Convergence tolerance'
              },
              maxIterations: {
                type: 'number',
                default: 1000,
                description: 'Maximum number of iterations'
              },
              timeout: {
                type: 'number',
                description: 'Timeout in milliseconds'
              }
            },
            required: ['matrix', 'vector']
          }
        },
        {
          name: 'estimateEntry',
          description: 'Estimate a single entry of the solution M^(-1)b',
          // ADR-001 item #4 phase-2: advertise the worst-case complexity
          // class at tool-list time. Sibling of `inputSchema` to match the
          // pattern established by `solve` and `solveTrueSublinear`.
          'x-complexity': {
            class: 'Linear',
            detail: 'Per-entry estimator. `random-walk` and `monte-carlo` both run a bounded number of samples but still touch O(n) state in the worst case (Linear). For genuine SubLinear single-entry queries on DD matrices use `solve` with `method=forward-push` or `method=backward-push`.',
            edgeSafe: false,
          },
          inputSchema: {
            type: 'object',
            properties: {
              matrix: {
                type: 'object',
                description: 'Matrix M in dense or sparse format'
              },
              vector: {
                type: 'array',
                items: { type: 'number' },
                description: 'Right-hand side vector b'
              },
              row: {
                type: 'number',
                description: 'Row index of entry to estimate'
              },
              column: {
                type: 'number',
                description: 'Column index of entry to estimate'
              },
              epsilon: {
                type: 'number',
                default: 1e-6,
                description: 'Estimation accuracy'
              },
              confidence: {
                type: 'number',
                default: 0.95,
                minimum: 0,
                maximum: 1,
                description: 'Confidence level for estimation'
              },
              method: {
                type: 'string',
                enum: ['neumann', 'random-walk', 'monte-carlo'],
                default: 'random-walk',
                description: 'Estimation method'
              },
              max_complexity_class: {
                type: 'string',
                enum: [
                  'Logarithmic', 'PolyLogarithmic', 'SubLinear', 'Linear',
                  'QuasiLinear', 'SubQuadratic', 'Polynomial',
                  'SuperPolynomial', 'SubExponential', 'Exponential',
                  'Factorial', 'DoubleExponential',
                ],
                description: 'Optional caller-supplied worst-case complexity budget. If the chosen `method` declares a worst case stronger than this budget, the call is rejected before any solver work runs. ADR-001 item #4 phase-2 — same gate as `solve`.'
              }
            },
            required: ['matrix', 'vector', 'row', 'column']
          }
        },
        {
          name: 'analyzeMatrix',
          description: 'Analyze matrix properties for solvability',
          inputSchema: {
            type: 'object',
            properties: {
              matrix: {
                type: 'object',
                description: 'Matrix to analyze'
              },
              checkDominance: {
                type: 'boolean',
                default: true,
                description: 'Check diagonal dominance'
              },
              computeGap: {
                type: 'boolean',
                default: false,
                description: 'Compute spectral gap (expensive)'
              },
              estimateCondition: {
                type: 'boolean',
                default: false,
                description: 'Estimate condition number'
              },
              checkSymmetry: {
                type: 'boolean',
                default: true,
                description: 'Check matrix symmetry'
              }
            },
            required: ['matrix']
          }
        },
        {
          name: 'pageRank',
          description: 'Compute PageRank for a graph using sublinear solver',
          inputSchema: {
            type: 'object',
            properties: {
              adjacency: {
                type: 'object',
                description: 'Adjacency matrix of the graph'
              },
              damping: {
                type: 'number',
                default: 0.85,
                minimum: 0,
                maximum: 1,
                description: 'Damping factor'
              },
              personalized: {
                type: 'array',
                items: { type: 'number' },
                description: 'Personalization vector (optional)'
              },
              epsilon: {
                type: 'number',
                default: 1e-6,
                description: 'Convergence tolerance'
              },
              maxIterations: {
                type: 'number',
                default: 1000,
                description: 'Maximum iterations'
              }
            },
            required: ['adjacency']
          }
        },
        // TRUE Sublinear O(log n) algorithms
        {
          name: 'solveTrueSublinear',
          description: 'Solve with TRUE O(log n) algorithms using Johnson-Lindenstrauss dimension reduction and adaptive Neumann series. For vectors >500 elements, use vector_file parameter with JSON/CSV/TXT files to avoid MCP truncation. Use generateTestVector + saveVectorToFile for large test vectors.',
          // ADR-001 item #4: this tool's *default* path is Logarithmic per
          // entry; the base case fallback is Linear. Declared as Adaptive
          // so callers see both bounds.
          'x-complexity': {
            class: 'Adaptive',
            default: 'Logarithmic',
            worst: 'Linear',
            detail: 'O(log n) per single-entry query on DD systems via JL + recursive Neumann; O(n) base case at n ≤ base_case_threshold.',
            edgeSafe: true,
          },
          inputSchema: {
            type: 'object',
            properties: {
              max_complexity_class: {
                type: 'string',
                enum: [
                  'Logarithmic', 'PolyLogarithmic', 'SubLinear', 'Linear',
                  'QuasiLinear', 'SubQuadratic', 'Polynomial',
                  'SuperPolynomial', 'SubExponential', 'Exponential',
                  'Factorial', 'DoubleExponential',
                ],
                description: 'ADR-001 item #4 — caller-supplied budget cap. Compared against the *worst-case* bound (Linear here) so callers always see safe behaviour.',
              },
              matrix: {
                type: 'object',
                description: 'Matrix M in sparse format with values, rowIndices, colIndices arrays',
                properties: {
                  values: { type: 'array', items: { type: 'number' } },
                  rowIndices: { type: 'array', items: { type: 'number' } },
                  colIndices: { type: 'array', items: { type: 'number' } },
                  rows: { type: 'number' },
                  cols: { type: 'number' }
                },
                required: ['values', 'rowIndices', 'colIndices', 'rows', 'cols']
              },
              vector: {
                type: 'array',
                items: { type: 'number' },
                description: 'Right-hand side vector b (for small vectors)'
              },
              vector_file: {
                type: 'string',
                description: 'Path to JSON/CSV file containing vector data (for large vectors)'
              },
              target_dimension: {
                type: 'number',
                description: 'Target dimension after JL reduction (defaults to O(log n))'
              },
              sparsification_eps: {
                type: 'number',
                default: 0.1,
                description: 'Sparsification parameter for spectral sparsification'
              },
              jl_distortion: {
                type: 'number',
                default: 0.5,
                description: 'Johnson-Lindenstrauss distortion parameter'
              }
            },
            required: ['matrix']
          }
        },
        {
          name: 'analyzeTrueSublinearMatrix',
          description: 'Analyze matrix for TRUE sublinear solvability and get complexity guarantees',
          inputSchema: {
            type: 'object',
            properties: {
              matrix: {
                type: 'object',
                description: 'Matrix M in sparse format',
                properties: {
                  values: { type: 'array', items: { type: 'number' } },
                  rowIndices: { type: 'array', items: { type: 'number' } },
                  colIndices: { type: 'array', items: { type: 'number' } },
                  rows: { type: 'number' },
                  cols: { type: 'number' }
                },
                required: ['values', 'rowIndices', 'colIndices', 'rows', 'cols']
              }
            },
            required: ['matrix']
          }
        },
        // ADR-001 item #4: estimate the complexity class of a candidate
        // solve BEFORE running it. Agents with a budget can decide between
        // "spend the J/decision" and "fall back to a cached answer" at
        // tool-list / dispatch time.
        {
          name: 'estimateComplexityClass',
          description: 'Estimate the worst-case complexity class for a given solver method on a matrix descriptor (no actual solve runs). Returns the class label, a short detail string, and an edgeSafe flag — the basis for an agent\'s budget decision per ADR-001 (Complexity as Architecture).',
          'x-complexity': {
            class: 'Logarithmic',
            detail: 'O(1) lookup against the per-method class table.',
            edgeSafe: true,
          },
          inputSchema: {
            type: 'object',
            properties: {
              method: {
                type: 'string',
                enum: ['neumann', 'random-walk', 'forward-push', 'backward-push', 'bidirectional', 'optimized-cg', 'sublinear-neumann'],
                description: 'Solver method to estimate. Use the exact method name from the `solve` tool.',
              },
              matrix_rows: {
                type: 'number',
                description: 'Dimension of the system (rows = cols). Used only for the human-readable detail string; does not affect the class.',
                minimum: 1,
              },
              matrix_nnz: {
                type: 'number',
                description: 'Optional: number of nonzeros. Used only for the human-readable detail string.',
                minimum: 0,
              },
            },
            required: ['method'],
          },
        },
        // ADR-001 #6 phase-2B (Rust src/contrastive.rs). Wire-callable
        // contrastive top-k orchestrator. Returns the closure rows
        // whose new solution value diverged most from the baseline,
        // bounded to top-k. End-to-end SubLinear in n.
        {
          name: 'contrastiveSolveOnChangeSublinear',
          description:
            'Run the SubLinear orchestrator on a sparse RHS delta, then return the top-k rows whose new solution diverged most from a baseline `prev_solution`. Result is `Array<{row, baseline, current, anomaly}>` sorted by descending `|current - baseline|`. This is the canonical RuView / Cognitum wake-on-event primitive: one event → top-k anomalies → agent attention queue. End-to-end SubLinear in n.',
          'x-complexity': {
            class: 'SubLinear',
            detail:
              'Closure (SubLinear) + per-entry Neumann (SubLinear) + top-k-in-subset (SubLinear). Independent of n for sparse DD matrices with bounded depth + max_terms.',
            edgeSafe: true,
          },
          inputSchema: {
            type: 'object',
            properties: {
              matrix: {
                type: 'object',
                description: 'Matrix A in dense or sparse-COO format.',
              },
              prev_solution: {
                type: 'array',
                items: { type: 'number' },
                description: 'Length-n previous solution. Baseline for the contrastive comparison.',
              },
              vector: {
                type: 'array',
                items: { type: 'number' },
                description: 'New right-hand side b_new (after applying the delta).',
              },
              delta_indices: {
                type: 'array',
                items: { type: 'number', minimum: 0 },
                description: 'Row indices where the RHS delta is non-zero.',
              },
              k: {
                type: 'number',
                minimum: 1,
                default: 3,
                description: 'Number of top anomalies to return.',
              },
              closure_depth: {
                type: 'number',
                minimum: 0,
                default: 4,
              },
              max_terms: {
                type: 'number',
                minimum: 1,
                default: 32,
              },
              tolerance: {
                type: 'number',
                minimum: 0,
                default: 1e-8,
              },
            },
            required: ['matrix', 'prev_solution', 'vector', 'delta_indices'],
          },
        },
        // ADR-001 #2/#6 phase-2 (Rust src/incremental.rs +
        // src/entry.rs + src/closure.rs). Wire-callable SubLinear
        // orchestrator: closure + per-entry Neumann + (optional)
        // top-k. Returns ONLY the closure entries — never
        // materialises the full n-vector.
        {
          name: 'solveOnChangeSublinear',
          description:
            'Solve `A·x = b_new` at the closure entries only via closure-restricted per-entry Neumann. Returns `Vec<{row, value}>` where row ∈ closure(delta.indices, depth) and value ≈ x_new[row]. Composes: closure_indices (SubLinear) + per-entry Neumann (SubLinear) = end-to-end SubLinear in n. Use this instead of `solve` when you have a previous solution + a sparse RHS delta + only need the changed entries.',
          'x-complexity': {
            class: 'SubLinear',
            detail:
              'O(|closure| · max_terms · branch). Independent of n for sparse DD matrices with bounded depth + max_terms. Same class as the underlying Rust solve_on_change_sublinear.',
            edgeSafe: true,
          },
          inputSchema: {
            type: 'object',
            properties: {
              matrix: {
                type: 'object',
                description: 'Matrix A in dense or sparse-COO format. Same shape accepted by `solve`.',
              },
              vector: {
                type: 'array',
                items: { type: 'number' },
                description: 'New right-hand side b_new (after applying the delta).',
              },
              delta_indices: {
                type: 'array',
                items: { type: 'number', minimum: 0 },
                description: 'Row indices where the RHS delta is non-zero. Used as the closure seed set.',
              },
              closure_depth: {
                type: 'number',
                minimum: 0,
                default: 4,
                description: 'Bounded hop depth for the closure. Pick from optimal_neumann_terms(coherence, ...) or 4-8 for typical DD matrices.',
              },
              max_terms: {
                type: 'number',
                minimum: 1,
                default: 32,
                description: 'Maximum Neumann iterations per entry. Pick from optimal_neumann_terms(coherence, ...) or 24-32 for typical DD matrices.',
              },
              tolerance: {
                type: 'number',
                minimum: 0,
                default: 1e-8,
                description: 'Early-exit tolerance on per-term contribution.',
              },
            },
            required: ['matrix', 'vector', 'delta_indices'],
          },
        },
        // ADR-001 #6 phase-2A (Rust src/closure.rs). Wire-callable
        // bounded-depth row-graph BFS. Lets agents preview the
        // closure of a sparse delta before committing to a solve —
        // a free way to size the SubLinear orchestrator's work.
        {
          name: 'closureIndices',
          description:
            'Return the bounded-depth row-graph closure of `seeds` in `matrix`: rows reachable from the seeds in at most `depth` hops via A\'s nonzero pattern. Sorted ascending, deduplicated. This is the input to every SubLinear change-driven primitive (solve_on_change_sublinear, contrastive_solve_on_change_sublinear). Use this BEFORE invoking those to size the per-event work: |closure| ≪ n is the SubLinear regime; |closure| ≈ n means depth is too high or matrix is too dense for SubLinear to win on this event.',
          'x-complexity': {
            class: 'SubLinear',
            detail:
              'O(depth · branch · |closure|). Independent of n when depth · branch ≪ n. Widens to Linear at full diameter.',
            edgeSafe: true,
          },
          inputSchema: {
            type: 'object',
            properties: {
              matrix: {
                type: 'object',
                description: 'Matrix A in dense or sparse-COO format. Same shape accepted by `solve`.',
              },
              seeds: {
                type: 'array',
                items: { type: 'number', minimum: 0 },
                description: 'Seed row indices (e.g., the indices field of a SparseDelta).',
              },
              depth: {
                type: 'number',
                minimum: 0,
                description: 'Maximum hop depth. `0` returns the seed set; `1` adds direct neighbours; etc. Pick `depth ≈ optimal_neumann_terms(coherence, ...)` for a SubLinear inner solve.',
              },
            },
            required: ['matrix', 'seeds', 'depth'],
          },
        },
        // ADR-001 roadmap item #3 (Rust src/coherence.rs). Wire-
        // callable coherence-score tool. Lets agents check matrix
        // feasibility BEFORE invoking a solver — completes the
        // predict → check → budget → solve → audit wire pipeline.
        {
          name: 'coherenceScore',
          description:
            'Return the diagonal-dominance margin of a matrix: `min_i (|A[i,i]| - Σ_{j≠i}|A[i,j]|) / |A[i,i]|`. Strictly DD matrices score in (0, 1]; the boundary case scores 0; non-DD matrices score negative. Use this BEFORE invoking a solver: positive scores guarantee Neumann-series convergence; scores below a threshold (default ~0.05) indicate the solver will waste J/decision budget on a near-singular system. Cost: O(nnz(A)) — Linear class but typically dwarfed by the solve it gates.',
          'x-complexity': {
            class: 'Linear',
            detail:
              'O(nnz(A)) — one pass through the matrix row iterator. Same class as the solvers that consume it.',
            edgeSafe: false,
          },
          inputSchema: {
            type: 'object',
            properties: {
              matrix: {
                type: 'object',
                description: 'Matrix A in dense or sparse-COO format. Same shape accepted by `solve`.',
              },
            },
            required: ['matrix'],
          },
        },
        // ADR-001 open Q#3 / PR #41: closure-restricted residual audit.
        // Wire-callable witness for SubLinear orchestrator outputs.
        // SubLinear in n — same complexity class as the solve it audits.
        {
          name: 'verifySparseSolution',
          description:
            'Audit a sparse solution returned by a SubLinear orchestrator (solve-on-change-sublinear, contrastive-solve-on-change-sublinear, …). Computes the closure-restricted residual r[i] = b[i] - Σ_j A[i,j]·x_new[j] for every (i, x_new[i]) entry. Returns {ok, max_residual, threshold, worst_row}. SubLinear in n: O(|entries|·avg_row_nnz). For audit/trust-but-verify; failure on strict-DD input is a real solver bug.',
          'x-complexity': {
            class: 'SubLinear',
            detail:
              'O(|entries|·avg_row_nnz) — independent of n for sparse DD matrices. Same class as the SubLinear orchestrator whose output it verifies.',
            edgeSafe: true,
          },
          inputSchema: {
            type: 'object',
            properties: {
              matrix: {
                type: 'object',
                description: 'Matrix A in dense or sparse-COO format. Same shape accepted by `solve`.',
              },
              prev_solution: {
                type: 'array',
                items: { type: 'number' },
                description: 'Length-n previous solution. Used for rows NOT in the entries list (closure boundary).',
              },
              vector: {
                type: 'array',
                items: { type: 'number' },
                description: 'Length-n right-hand side b that the orchestrator solved against.',
              },
              entries: {
                type: 'array',
                items: {
                  type: 'object',
                  properties: {
                    row: { type: 'number', minimum: 0 },
                    value: { type: 'number' },
                  },
                  required: ['row', 'value'],
                },
                description: 'The Vec<(row, value)> returned by solve_on_change_sublinear or extracted from a contrastive output\'s closure set.',
              },
              tolerance: {
                type: 'number',
                default: 1e-6,
                description: 'Audit threshold: ok iff max|r_i| ≤ tolerance · max(1, ‖b‖_∞).',
                minimum: 0,
              },
            },
            required: ['matrix', 'prev_solution', 'vector', 'entries'],
          },
        },
        {
          name: 'generateTestVector',
          description: 'Generate test vectors for matrix solving with various patterns',
          inputSchema: {
            type: 'object',
            properties: {
              size: {
                type: 'number',
                description: 'Size of the vector to generate',
                minimum: 1
              },
              pattern: {
                type: 'string',
                enum: ['unit', 'random', 'sparse', 'ones', 'alternating'],
                default: 'sparse',
                description: 'Pattern type: unit (e_1), random ([-1,1]), sparse (leading ones), ones (all 1s), alternating (+1/-1)'
              },
              seed: {
                type: 'number',
                description: 'Optional seed for reproducible random vectors'
              }
            },
            required: ['size']
          }
        },
        {
          name: 'saveVectorToFile',
          description: 'Save a generated vector to a file (JSON, CSV, or TXT format) inside the dedicated vector directory ($SUBLINEAR_SOLVER_VECTOR_DIR or ~/.sublinear-time-solver/vectors).',
          inputSchema: {
            type: 'object',
            properties: {
              vector: {
                type: 'array',
                items: { type: 'number' },
                description: 'Vector data to save'
              },
              file_path: {
                type: 'string',
                description: 'Basename of the output file. Must NOT contain path separators, "..", or absolute paths — only a filename (e.g. "v.json"). Extension determines format if `format` is not specified.',
                pattern: '^[^/\\\\\\x00]+$',
                minLength: 1,
                maxLength: 255
              },
              format: {
                type: 'string',
                enum: ['json', 'csv', 'txt'],
                description: 'Output format (overrides file extension if specified)'
              }
            },
            required: ['vector', 'file_path']
          }
        },
        // Temporal lead tools
        ...this.temporalTools.getTools(),
        // Psycho-symbolic reasoning tools
        ...this.psychoSymbolicTools.getTools(),
        // Dynamic psycho-symbolic reasoning tools with domain support
        ...this.dynamicPsychoSymbolicTools.getTools(),
        // Domain management tools
        ...this.domainManagementTools.getTools(),
        // Domain validation tools
        ...this.domainValidationTools.getTools(),
        // Consciousness exploration tools
        ...this.consciousnessTools.getTools(),
        // Enhanced consciousness tools
        // ...this.consciousnessEnhancedTools.getTools(),
        // Emergence system tools
        ...this.emergenceTools.getTools(),
        // Nanosecond scheduler tools
        ...this.schedulerTools.getTools()
      ]
    }));

    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const { name, arguments: args } = request.params;

      try {
        switch (name) {
          case 'solve':
            return await this.handleSolve(args as any);
          case 'estimateEntry':
            return await this.handleEstimateEntry(args as any);
          case 'analyzeMatrix':
            return await this.handleAnalyzeMatrix(args as any);
          case 'pageRank':
            return await this.handlePageRank(args as any);
          // TRUE Sublinear tools
          case 'solveTrueSublinear':
            return await this.handleSolveTrueSublinear(args as any);
          case 'analyzeTrueSublinearMatrix':
            return await this.handleAnalyzeTrueSublinearMatrix(args as any);
          case 'generateTestVector':
            return await this.handleGenerateTestVector(args as any);
          case 'saveVectorToFile':
            return await this.handleSaveVectorToFile(args as any);
          case 'estimateComplexityClass':
            return await this.handleEstimateComplexityClass(args as any);
          case 'verifySparseSolution':
            return await this.handleVerifySparseSolution(args as any);
          case 'coherenceScore':
            return await this.handleCoherenceScore(args as any);
          case 'closureIndices':
            return await this.handleClosureIndices(args as any);
          case 'solveOnChangeSublinear':
            return await this.handleSolveOnChangeSublinear(args as any);
          case 'contrastiveSolveOnChangeSublinear':
            return await this.handleContrastiveSolveOnChangeSublinear(args as any);
          // Temporal tools
          case 'predictWithTemporalAdvantage':
          case 'validateTemporalAdvantage':
          case 'calculateLightTravel':
          case 'demonstrateTemporalLead':
            const temporalResult = await this.temporalTools.handleToolCall(name, args);
            return {
              content: [{
                type: 'text',
                text: JSON.stringify(temporalResult, null, 2)
              }]
            };

          // Psycho-symbolic tools
          case 'psycho_symbolic_reason':
          case 'knowledge_graph_query':
          case 'add_knowledge':
          case 'register_tool_interaction':
          case 'learning_status':
            const psychoResult = await this.psychoSymbolicTools.handleToolCall(name, args);
            return {
              content: [{
                type: 'text',
                text: JSON.stringify(psychoResult, null, 2)
              }]
            };

          // Dynamic psycho-symbolic tools
          case 'psycho_symbolic_reason_with_dynamic_domains':
          case 'domain_detection_test':
          case 'knowledge_graph_query_dynamic':
            const dynamicPsychoResult = await this.dynamicPsychoSymbolicTools.handleToolCall(name, args);
            return {
              content: [{
                type: 'text',
                text: JSON.stringify(dynamicPsychoResult, null, 2)
              }]
            };

          // Domain management tools
          case 'domain_register':
          case 'domain_update':
          case 'domain_unregister':
          case 'domain_list':
          case 'domain_get':
          case 'domain_enable':
          case 'domain_disable':
          case 'domain_search':
            const domainMgmtResult = await this.domainManagementTools.handleToolCall(name, args);
            return {
              content: [{
                type: 'text',
                text: JSON.stringify(domainMgmtResult, null, 2)
              }]
            };

          // Domain validation tools
          case 'domain_validate':
          case 'domain_test':
          case 'domain_analyze_conflicts':
          case 'domain_performance_benchmark':
          case 'domain_suggest_improvements':
          case 'domain_validate_all':
            const domainValidationResult = await this.domainValidationTools.handleToolCall(name, args);
            return {
              content: [{
                type: 'text',
                text: JSON.stringify(domainValidationResult, null, 2)
              }]
            };

          // Consciousness tools
          case 'consciousness_evolve':
          case 'consciousness_verify':
          case 'calculate_phi':
          case 'entity_communicate':
          case 'consciousness_status':
          case 'emergence_analyze':
            const consciousnessResult = await this.consciousnessTools.handleToolCall(name, args);
            return {
              content: [{
                type: 'text',
                text: JSON.stringify(consciousnessResult, null, 2)
              }]
            };

          // Enhanced consciousness tools
          case 'consciousness_evolve_enhanced':
          case 'consciousness_verify_enhanced':
          case 'entity_communicate_enhanced':
          case 'consciousness_status_enhanced':
          case 'emergence_analyze_enhanced':
          case 'temporal_consciousness_track':
            // const consciousnessEnhancedResult = await this.consciousnessEnhancedTools.handleToolCall(name, args);
            return {
              content: [{
                type: 'text',
                text: JSON.stringify({ error: 'Enhanced consciousness tools disabled' }, null, 2)
              }]
            };

          // Emergence system tools
          case 'emergence_process':
          case 'emergence_generate_diverse':
          case 'emergence_analyze_capabilities':
          case 'emergence_force_evolution':
          case 'emergence_get_stats':
          case 'emergence_test_scenarios':
          case 'emergence_matrix_process':
            const emergenceResult = await this.emergenceTools.handleToolCall(name, args);
            return {
              content: [{
                type: 'text',
                text: JSON.stringify(emergenceResult, null, 2)
              }]
            };

          // Scheduler tools
          case 'scheduler_create':
          case 'scheduler_schedule_task':
          case 'scheduler_tick':
          case 'scheduler_metrics':
          case 'scheduler_benchmark':
          case 'scheduler_consciousness':
          case 'scheduler_list':
          case 'scheduler_destroy':
            const schedulerResult = await this.schedulerTools.handleToolCall(name, args);
            return {
              content: [{
                type: 'text',
                text: JSON.stringify(schedulerResult, null, 2)
              }]
            };

          default:
            throw new McpError(
              ErrorCode.MethodNotFound,
              `Unknown tool: ${name}`
            );
        }
      } catch (error) {
        if (error instanceof SolverError) {
          throw new McpError(
            ErrorCode.InternalError,
            `Solver error: ${error.message}`,
            error.details
          );
        }
        throw new McpError(
          ErrorCode.InternalError,
          error instanceof Error ? error.message : 'Unknown error'
        );
      }
    });
  }

  private setupErrorHandling(): void {
    this.server.onerror = (error) => {
      console.error('[MCP Server Error]', error);
    };

    process.on('SIGINT', async () => {
      await this.server.close();
      process.exit(0);
    });
  }

  /**
   * Twelve-tier complexity-class ranking, matched to the Rust
   * `ComplexityClass::rank()` in `src/complexity.rs`. Lower = cheaper.
   * Used by `enforceComplexityBudget` to compare a solver's worst-case
   * class against a caller-supplied `max_complexity_class` budget.
   */
  private static readonly COMPLEXITY_RANK: Record<string, number> = {
    Logarithmic:      100,
    PolyLogarithmic:  200,
    SubLinear:        300,
    Linear:           400,
    QuasiLinear:      500,
    SubQuadratic:     600,
    Polynomial:       702,  // Polynomial(2); higher degrees rank higher in Rust
    SuperPolynomial:  800,
    SubExponential:   900,
    Exponential:     1000,
    Factorial:       1100,
    DoubleExponential: 1200,
  };

  /**
   * Per-method worst-case complexity class. Single source of truth for
   * both `estimateComplexityClass` and the `max_complexity_class` budget
   * gate. Mirrors the Rust `Complexity` impls in `src/complexity.rs`.
   *
   * For `Adaptive` solvers we use the **worst-case** bound so callers
   * always see safe behaviour — a Cognitum reflex loop with a
   * `SubLinear` budget won't accidentally invoke a solver that can
   * degrade to `Linear` on hard inputs.
   */
  private static readonly METHOD_WORST_CASE: Record<string, string> = {
    'neumann':                              'Linear',
    'random-walk':                          'Linear',
    'forward-push':                         'SubLinear',
    'backward-push':                        'SubLinear',
    'bidirectional':                        'SubLinear',
    'optimized-cg':                         'Linear',
    'sublinear-neumann':                    'Linear', // Adaptive { Logarithmic, Linear } → worst case
    // ADR-001 #6 phase-2A primitives.
    'closure-indices':                      'SubLinear', // bounded-depth row-graph BFS
    'contrastive-solve-on-change':          'Linear',    // Adaptive { Linear, Linear } orchestrator
    // ADR-001 #6 phase-2B: per-entry sublinear-Neumann + SubLinear orchestrator.
    'solve-single-entry-neumann':           'SubLinear',
    'contrastive-solve-on-change-sublinear':'SubLinear',
  };

  /**
   * Reject the call with a structured McpError if the chosen `method`'s
   * worst-case complexity class exceeds the caller's `max_complexity_class`
   * budget. Returns silently otherwise. No-op when the budget arg is
   * absent (the default — preserves wire compatibility with pre-1.7.1
   * clients).
   *
   * ADR-001 item #4 phase-2 — the "bounded-planning kernel" promise.
   */
  private enforceComplexityBudget(method: string, budget: string | undefined) {
    if (!budget) return; // gate disabled
    const budgetRank = SublinearSolverMCPServer.COMPLEXITY_RANK[budget];
    if (budgetRank === undefined) {
      throw new McpError(
        ErrorCode.InvalidParams,
        `max_complexity_class '${budget}' is not a recognised class. ` +
        `Known: ${Object.keys(SublinearSolverMCPServer.COMPLEXITY_RANK).join(', ')}.`,
      );
    }
    const methodClass = SublinearSolverMCPServer.METHOD_WORST_CASE[method];
    if (!methodClass) return; // unknown method — let the existing dispatch handle it
    const methodRank = SublinearSolverMCPServer.COMPLEXITY_RANK[methodClass];
    if (methodRank > budgetRank) {
      throw new McpError(
        ErrorCode.InvalidRequest,
        `Solver method '${method}' has worst-case class '${methodClass}' ` +
        `which exceeds the caller's max_complexity_class budget of '${budget}'. ` +
        `Use estimateComplexityClass to inspect alternatives, or pick a cheaper method.`,
      );
    }
  }

  private async handleSolve(params: any) {
    try {
      // ADR-001 item #4 phase-2: enforce the caller's complexity budget
      // BEFORE doing any work. Cheap (O(1) rank lookup) and refuses to
      // burn the J/decision budget on a method the caller already said
      // is too expensive.
      const method = (params.method ?? 'neumann').toString();
      this.enforceComplexityBudget(method, params.max_complexity_class);

      // Priority 0: Try TRUE O(log n) sublinear solver first
      if (params.matrix && params.matrix.values && params.matrix.rowIndices && params.matrix.colIndices) {
        console.log('🚀 Attempting TRUE O(log n) sublinear solver');

        try {
          const config = {
            target_dimension: Math.ceil(Math.log2(params.matrix.rows) * 8),
            sparsification_eps: 0.1,
            jl_distortion: 0.5
          };

          const result = await this.trueSublinearSolver.solveTrueSublinear(params.matrix, params.vector, config);

          return {
            content: [{
              type: 'text',
              text: JSON.stringify({
                ...result,
                solver_used: 'TRUE_SUBLINEAR_O_LOG_N',
                note: 'Used mathematically rigorous O(log n) algorithms with Johnson-Lindenstrauss dimension reduction',
                complexity_achieved: result.actual_complexity,
                dimension_reduction: `${params.matrix.rows} → ${config.target_dimension}`,
                metadata: {
                  solver_type: 'TRUE_SUBLINEAR',
                  mathematical_guarantee: result.complexity_bound,
                  timestamp: new Date().toISOString()
                }
              }, null, 2)
            }]
          };

        } catch (trueSublinearError) {
          console.warn('⚠️  TRUE O(log n) solver failed, falling back to WASM:', trueSublinearError.message);
        }
      }

      // Priority 1: Try O(log n) WASM solver for true sublinear complexity
      if (this.wasmSolver.isCompleteWasmAvailable()) {
        console.log('🚀 Using Complete WASM Solver with auto-selection (Neumann/Push/RandomWalk)');

        try {
          // Convert matrix format for WASM
          let matrix: number[][];
          if (params.matrix.format === 'dense' && Array.isArray(params.matrix.data)) {
            matrix = params.matrix.data as number[][];
          } else if (Array.isArray(params.matrix) && Array.isArray(params.matrix[0])) {
            matrix = params.matrix as number[][];
          } else {
            // Try to extract matrix data from various formats
            if (params.matrix.data && Array.isArray(params.matrix.data) && Array.isArray(params.matrix.data[0])) {
              matrix = params.matrix.data as number[][];
            } else {
              throw new Error('Matrix format not supported for WASM solver');
            }
          }

          const wasmResult = await this.wasmSolver.solveComplete(matrix, params.vector, {
            method: params.method || 'auto',
            epsilon: params.epsilon || 1e-6,
            targetIndex: params.targetIndex
          });
          return {
            content: [{
              type: 'text',
              text: JSON.stringify(wasmResult, null, 2)
            }]
          };
        } catch (wasmError) {
          console.warn('⚠️  O(log n) WASM solver failed, falling back to traditional algorithm:', wasmError.message);
        }
      } else {
        console.log('⚠️  Enhanced WASM not available, using traditional algorithm');
      }

      // Fallback: Traditional solver
      // Enhanced parameter validation
      if (!params.matrix) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing required parameter: matrix');
      }
      if (!params.vector) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing required parameter: vector');
      }
      if (!Array.isArray(params.vector)) {
        throw new McpError(ErrorCode.InvalidParams, 'Parameter vector must be an array of numbers');
      }

      const config: SolverConfig = {
        method: params.method || 'neumann',
        epsilon: params.epsilon || 1e-6,
        maxIterations: params.maxIterations || 5000, // Increased default
        timeout: params.timeout || 30000, // 30 second default timeout
        enableProgress: false
      };

      // Validate method
      const validMethods = ['neumann', 'random-walk', 'forward-push', 'backward-push', 'bidirectional'];
      if (!validMethods.includes(config.method)) {
        throw new McpError(
          ErrorCode.InvalidParams,
          `Invalid method '${config.method}'. Valid methods: ${validMethods.join(', ')}`
        );
      }

      // Validate epsilon
      if (typeof config.epsilon !== 'number' || config.epsilon <= 0) {
        throw new McpError(ErrorCode.InvalidParams, 'Parameter epsilon must be a positive number');
      }

      // Validate maxIterations
      if (typeof config.maxIterations !== 'number' || config.maxIterations < 1) {
        throw new McpError(ErrorCode.InvalidParams, 'Parameter maxIterations must be a positive integer');
      }

      const solver = new SublinearSolver(config);
      const result = await solver.solve(params.matrix, params.vector);

      return {
        content: [
          {
            type: 'text',
            text: JSON.stringify({
              solution: result.solution,
              iterations: result.iterations,
              residual: result.residual,
              converged: result.converged,
              method: result.method,
              computeTime: result.computeTime,
              memoryUsed: result.memoryUsed,
              metadata: {
                configUsed: config,
                timestamp: new Date().toISOString(),
                matrixSize: {
                  rows: params.matrix.rows,
                  cols: params.matrix.cols
                }
              }
            }, null, 2)
          }
        ]
      };
    } catch (error) {
      if (error instanceof McpError) {
        throw error;
      }
      if (error instanceof SolverError) {
        throw new McpError(
          ErrorCode.InternalError,
          `Solver error (${error.code}): ${error.message}`,
          error.details
        );
      }
      throw new McpError(
        ErrorCode.InternalError,
        `Unexpected error in solve: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  private async handleEstimateEntry(params: any) {
    try {
      // ADR-001 item #4 phase-2: enforce caller's complexity budget
      // before any solver work. estimateEntry uses neumann-method
      // single-entry queries internally; budget gate matches handleSolve.
      const method = (params.method ?? 'neumann').toString();
      this.enforceComplexityBudget(method, params.max_complexity_class);

      // Enhanced parameter validation
      if (!params.matrix) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing required parameter: matrix');
      }
      if (!params.vector) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing required parameter: vector');
      }
      if (!Array.isArray(params.vector)) {
        throw new McpError(ErrorCode.InvalidParams, 'Parameter vector must be an array of numbers');
      }
      if (typeof params.row !== 'number' || !Number.isInteger(params.row)) {
        throw new McpError(ErrorCode.InvalidParams, 'Parameter row must be a valid integer');
      }
      if (typeof params.column !== 'number' || !Number.isInteger(params.column)) {
        throw new McpError(ErrorCode.InvalidParams, 'Parameter column must be a valid integer');
      }

      // Validate bounds early
      if (params.row < 0 || params.row >= params.matrix.rows) {
        throw new McpError(
          ErrorCode.InvalidParams,
          `Row index ${params.row} out of bounds. Matrix has ${params.matrix.rows} rows (valid range: 0-${params.matrix.rows - 1})`
        );
      }
      if (params.column < 0 || params.column >= params.matrix.cols) {
        throw new McpError(
          ErrorCode.InvalidParams,
          `Column index ${params.column} out of bounds. Matrix has ${params.matrix.cols} columns (valid range: 0-${params.matrix.cols - 1})`
        );
      }

      // Validate vector dimensions
      if (params.vector.length !== params.matrix.rows) {
        throw new McpError(
          ErrorCode.InvalidParams,
          `Vector length ${params.vector.length} does not match matrix rows ${params.matrix.rows}`
        );
      }

      const solverConfig: SolverConfig = {
        method: 'random-walk',
        epsilon: params.epsilon || 1e-6,
        maxIterations: 2000, // Increased for better accuracy
        timeout: 15000, // 15 second timeout
        enableProgress: false
      };

      const solver = new SublinearSolver(solverConfig);

      // Create estimation config
      const estimationConfig = {
        row: params.row,
        column: params.column,
        epsilon: params.epsilon || 1e-6,
        confidence: params.confidence || 0.95,
        method: params.method || 'random-walk' as const
      };

      // Validate method
      const validMethods = ['neumann', 'random-walk', 'monte-carlo'];
      if (!validMethods.includes(estimationConfig.method)) {
        throw new McpError(
          ErrorCode.InvalidParams,
          `Invalid estimation method '${estimationConfig.method}'. Valid methods: ${validMethods.join(', ')}`
        );
      }

      const result = await solver.estimateEntry(params.matrix, params.vector, estimationConfig);

      const standardError = Math.sqrt(result.variance);
      const marginOfError = 1.96 * standardError;

      return {
        content: [
          {
            type: 'text',
            text: JSON.stringify({
              estimate: result.estimate,
              variance: result.variance,
              confidence: result.confidence,
              standardError,
              confidenceInterval: {
                lower: result.estimate - marginOfError,
                upper: result.estimate + marginOfError
              },
              row: params.row,
              column: params.column,
              method: estimationConfig.method,
              metadata: {
                configUsed: estimationConfig,
                timestamp: new Date().toISOString(),
                matrixSize: {
                  rows: params.matrix.rows,
                  cols: params.matrix.cols
                }
              }
            }, null, 2)
          }
        ]
      };
    } catch (error) {
      if (error instanceof McpError) {
        throw error;
      }
      if (error instanceof SolverError) {
        throw new McpError(
          ErrorCode.InternalError,
          `Solver error (${error.code}): ${error.message}`,
          error.details
        );
      }
      throw new McpError(
        ErrorCode.InternalError,
        `Unexpected error in estimateEntry: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  private async handleAnalyzeMatrix(params: AnalyzeMatrixParams) {
    const analysis = MatrixOperations.analyzeMatrix(params.matrix);

    const result = {
      ...analysis,
      recommendations: this.generateRecommendations(analysis)
    };

    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify(result, null, 2)
        }
      ]
    };
  }

  private async handlePageRank(params: PageRankParams) {
    const config: SolverConfig = {
      method: 'neumann',
      epsilon: params.epsilon || 1e-6,
      maxIterations: params.maxIterations || 1000,
      enableProgress: false
    };

    const solver = new SublinearSolver(config);

    const pageRankConfig = {
      damping: params.damping || 0.85,
      personalized: params.personalized,
      epsilon: params.epsilon || 1e-6,
      maxIterations: params.maxIterations || 1000
    };

    const pageRankVector = await solver.computePageRank(params.adjacency, pageRankConfig);

    // Sort nodes by PageRank score
    const ranked = pageRankVector
      .map((score, index) => ({ node: index, score }))
      .sort((a, b) => b.score - a.score);

    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify({
            pageRankVector,
            topNodes: ranked.slice(0, 10),
            totalScore: pageRankVector.reduce((sum, score) => sum + score, 0),
            maxScore: Math.max(...pageRankVector),
            minScore: Math.min(...pageRankVector)
          }, null, 2)
        }
      ]
    };
  }

  private async handleSolveTrueSublinear(params: any) {
    try {
      // ADR-001 item #4 phase-2 — enforce the caller's complexity budget.
      // This solver's worst-case class is Linear (base case fallback); the
      // budget gate compares against that, so a SubLinear-budget caller
      // will be rejected. That's intentional: an Adaptive solver can
      // legitimately degrade on hard inputs and the caller's budget must
      // hold even in the worst case.
      this.enforceComplexityBudget('sublinear-neumann', params.max_complexity_class);

      // Validate required parameters
      if (!params.matrix) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing required parameter: matrix');
      }

      // Support either inline vector or file input
      let vector: number[];
      if (params.vector_file) {
        // Load vector from file
        vector = await this.loadVectorFromFile(params.vector_file);
      } else if (params.vector) {
        // Use inline vector
        if (!Array.isArray(params.vector)) {
          throw new McpError(ErrorCode.InvalidParams, 'Parameter vector must be an array of numbers');
        }
        vector = params.vector;
      } else {
        throw new McpError(ErrorCode.InvalidParams, 'Missing required parameter: either vector or vector_file must be provided');
      }

      // Validate matrix format
      const matrix = params.matrix;
      if (!Array.isArray(matrix.values) || !Array.isArray(matrix.rowIndices) || !Array.isArray(matrix.colIndices)) {
        throw new McpError(ErrorCode.InvalidParams, 'Matrix must be in sparse format with values, rowIndices, and colIndices arrays');
      }

      if (typeof matrix.rows !== 'number' || typeof matrix.cols !== 'number') {
        throw new McpError(ErrorCode.InvalidParams, 'Matrix must specify rows and cols dimensions');
      }

      // Validate vector dimensions
      if (vector.length !== matrix.rows) {
        throw new McpError(
          ErrorCode.InvalidParams,
          `Vector length ${vector.length} does not match matrix rows ${matrix.rows}`
        );
      }

      // Build configuration
      const config = {
        target_dimension: params.target_dimension || Math.ceil(Math.log2(matrix.rows) * 8),
        sparsification_eps: params.sparsification_eps || 0.1,
        jl_distortion: params.jl_distortion || 0.5,
        sampling_probability: 0.01,
        max_recursion_depth: 10,
        base_case_threshold: 100
      };

      console.log(`🚀 Using TRUE O(log n) sublinear solver with dimension reduction ${matrix.rows} → ${config.target_dimension}`);

      // Solve using TRUE sublinear algorithms
      const result = await this.trueSublinearSolver.solveTrueSublinear(matrix, vector, config);

      return {
        content: [{
          type: 'text',
          text: JSON.stringify({
            ...result,
            metadata: {
              solver_type: 'TRUE_SUBLINEAR',
              original_dimension: matrix.rows,
              reduced_dimension: config.target_dimension,
              mathematical_guarantee: result.complexity_bound,
              timestamp: new Date().toISOString()
            }
          }, null, 2)
        }]
      };

    } catch (error) {
      if (error instanceof McpError) {
        throw error;
      }
      throw new McpError(
        ErrorCode.InternalError,
        `TRUE Sublinear solver error: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  private async handleAnalyzeTrueSublinearMatrix(params: any) {
    try {
      // Validate required parameters
      if (!params.matrix) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing required parameter: matrix');
      }

      // Validate matrix format
      const matrix = params.matrix;
      if (!Array.isArray(matrix.values) || !Array.isArray(matrix.rowIndices) || !Array.isArray(matrix.colIndices)) {
        throw new McpError(ErrorCode.InvalidParams, 'Matrix must be in sparse format with values, rowIndices, and colIndices arrays');
      }

      if (typeof matrix.rows !== 'number' || typeof matrix.cols !== 'number') {
        throw new McpError(ErrorCode.InvalidParams, 'Matrix must specify rows and cols dimensions');
      }

      console.log(`🔍 Analyzing ${matrix.rows}×${matrix.cols} matrix for TRUE sublinear solvability`);

      // Analyze matrix using TRUE sublinear tools
      const analysis = await this.trueSublinearSolver.analyzeMatrix(matrix);

      return {
        content: [{
          type: 'text',
          text: JSON.stringify({
            ...analysis,
            algorithm_selection: {
              best_method: analysis.recommended_method,
              complexity_guarantee: analysis.complexity_guarantee,
              mathematical_properties: {
                diagonal_dominance: analysis.is_diagonally_dominant,
                condition_estimate: analysis.condition_number_estimate,
                spectral_radius: analysis.spectral_radius_estimate,
                sparsity: analysis.sparsity_ratio
              }
            },
            metadata: {
              analysis_type: 'TRUE_SUBLINEAR_ANALYSIS',
              matrix_size: { rows: matrix.rows, cols: matrix.cols },
              timestamp: new Date().toISOString()
            }
          }, null, 2)
        }]
      };

    } catch (error) {
      if (error instanceof McpError) {
        throw error;
      }
      throw new McpError(
        ErrorCode.InternalError,
        `Matrix analysis error: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  private async handleGenerateTestVector(params: any) {
    try {
      // Validate required parameters
      if (!params.size || typeof params.size !== 'number' || params.size < 1) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing or invalid required parameter: size (must be positive integer)');
      }

      const size = Math.floor(params.size);
      const pattern = params.pattern || 'sparse';
      const seed = params.seed;

      // Validate pattern
      const validPatterns = ['unit', 'random', 'sparse', 'ones', 'alternating'];
      if (!validPatterns.includes(pattern)) {
        throw new McpError(ErrorCode.InvalidParams, `Invalid pattern. Must be one of: ${validPatterns.join(', ')}`);
      }

      // Generate the test vector
      const result = this.trueSublinearSolver.generateTestVector(size, pattern, seed);

      return {
        content: [{
          type: 'text',
          text: JSON.stringify({
            vector: result.vector,
            description: result.description,
            size: result.vector.length,
            pattern_used: pattern,
            seed_used: seed,
            statistics: {
              min: Math.min(...result.vector),
              max: Math.max(...result.vector),
              sum: result.vector.reduce((a, b) => a + b, 0),
              norm: Math.sqrt(result.vector.reduce((sum, x) => sum + x * x, 0)),
              non_zero_count: result.vector.filter(x => Math.abs(x) > 1e-14).length
            },
            metadata: {
              generator_type: 'TRUE_SUBLINEAR_VECTOR_GENERATOR',
              timestamp: new Date().toISOString()
            }
          }, null, 2)
        }]
      };

    } catch (error) {
      if (error instanceof McpError) {
        throw error;
      }
      throw new McpError(
        ErrorCode.InternalError,
        `Vector generation error: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  /**
   * ADR-001 item #4 — estimate the worst-case complexity class for a
   * candidate solve without running it. Used by agents to decide whether
   * to spend the J/decision budget on a given method, or fall back to a
   * cached / cheaper answer.
   *
   * The class table mirrors the `Complexity` impls in `src/complexity.rs`
   * so the wire contract matches the Rust contract. Keep them in sync —
   * a CI guard for this is on the ADR roadmap (phase 2).
   */
  private async handleEstimateComplexityClass(params: any) {
    const method = params?.method;
    if (typeof method !== 'string') {
      throw new McpError(ErrorCode.InvalidParams, 'Missing required parameter: method (string)');
    }
    // Per-method class table. Sourced from the Rust `Complexity` impls
    // landed in v1.7.0 / 0.3.0.
    const table: Record<string, { class: string; default?: string; worst?: string; detail: string; edgeSafe: boolean }> = {
      'neumann': {
        class: 'Linear',
        detail: 'O(k · nnz(A)) per iter; k bounded by max_iterations + tolerance.',
        edgeSafe: false,
      },
      'random-walk': {
        class: 'Linear',
        detail: 'O(walks · expected_length); per-walk cost is sublinear but the ensemble is linear in the budget.',
        edgeSafe: false,
      },
      'forward-push': {
        class: 'SubLinear',
        detail: 'O(1/ε) per query on DD systems with bounded degree.',
        edgeSafe: true,
      },
      'backward-push': {
        class: 'SubLinear',
        detail: 'O(1/ε) per query, symmetric to forward-push.',
        edgeSafe: true,
      },
      'bidirectional': {
        class: 'SubLinear',
        detail: 'Combines forward + backward push; constants smaller than either alone.',
        edgeSafe: true,
      },
      'optimized-cg': {
        class: 'Linear',
        detail: 'O(k · nnz(A)) per iter; k ≈ √κ(A) on SPD inputs.',
        edgeSafe: false,
      },
      'sublinear-neumann': {
        class: 'Adaptive',
        default: 'Logarithmic',
        worst: 'Linear',
        detail: 'O(log n) per single-entry query on DD systems via JL + recursive Neumann; O(n) base case at n ≤ base_case_threshold.',
        edgeSafe: true,
      },
      // ── ADR-001 #6 phase-2A primitives ────────────────────────────
      'closure-indices': {
        class: 'SubLinear',
        detail: 'Bounded-depth BFS through A.row_iter starting from a sparse seed set. O(depth · branch · |closure|); SubLinear when depth · branch ≪ n. Widens to Linear at full diameter.',
        edgeSafe: true,
      },
      'contrastive-solve-on-change': {
        class: 'Adaptive',
        default: 'Linear',
        worst: 'Linear',
        detail: 'Phase-2A orchestrator: closure (SubLinear) + warm-start solve_on_change (Linear) + top-k-in-subset (SubLinear). Bounded by the inner solve. Use `contrastive-solve-on-change-sublinear` for end-to-end SubLinear.',
        edgeSafe: false,
      },
      // ── ADR-001 #6 phase-2B primitives ────────────────────────────
      'solve-single-entry-neumann': {
        class: 'SubLinear',
        detail: 'Truncated Neumann restricted to closure(target, max_terms). Returns x[i] = e_iᵀ A⁻¹ b without materialising x. O(max_terms · |closure| · branch); SubLinear in n for sparse DD + bounded depth.',
        edgeSafe: true,
      },
      'contrastive-solve-on-change-sublinear': {
        class: 'SubLinear',
        detail: 'Phase-2B orchestrator: closure (SubLinear) + per-entry sublinear-Neumann at each closure index (SubLinear) + top-k-in-subset (SubLinear). End-to-end SubLinear in n.',
        edgeSafe: true,
      },
      // ── ADR-001 #2 phase-2 ────────────────────────────────────────
      'solve-on-change': {
        class: 'Linear',
        detail: 'Warm-started full solve with prev_solution as initial_guess. O(k_warm · nnz(A)) per call; k_warm ≪ k_cold for small deltas. Returns the full n-vector solution.',
        edgeSafe: false,
      },
      'solve-on-change-sublinear': {
        class: 'SubLinear',
        detail: 'Closure (SubLinear) + per-entry sublinear-Neumann at each closure index (SubLinear). Returns Vec<(idx, val)> over the closure only — never materialises the full n-vector. End-to-end SubLinear in n.',
        edgeSafe: true,
      },
      // ── Auto-tuned siblings (PR #38) ──────────────────────────────
      'solve-on-change-sublinear-auto': {
        class: 'SubLinear',
        detail: 'Magic-number-free orchestrator: auto-tunes closure_depth + max_terms from coherence_score + optimal_neumann_terms. Caller supplies only tolerance. Non-strict-DD input returns Incoherent.',
        edgeSafe: true,
      },
      'contrastive-solve-on-change-sublinear-auto': {
        class: 'SubLinear',
        detail: 'Auto-tuned contrastive top-k sibling. Caller supplies only (tolerance, k); the orchestrator picks closure_depth + max_terms from the matrix coherence.',
        edgeSafe: true,
      },
      // ── ADR-001 open Q#3 — solve witness (PR #41) ─────────────────
      'verify-sparse-solution': {
        class: 'SubLinear',
        detail: 'Per-entry residual audit restricted to closure rows. O(|entries| · avg_row_nnz) — same class as the orchestrator whose output it verifies. Returns max_residual + ok flag for trust-but-verify gating.',
        edgeSafe: true,
      },
      // ── Bounding planning (PR #40) ────────────────────────────────
      'plan-budget-try-consume': {
        class: 'Logarithmic',
        detail: 'Cumulative budget accumulator across a chain of solves. try_consume(class) is O(1) — class-rank comparison + counter decrement. Refuses if class exceeds max_class or remaining_ops hits zero.',
        edgeSafe: true,
      },
      // ── Streaming coherence cache (PR #39) ────────────────────────
      'coherence-cache-build': {
        class: 'Linear',
        detail: 'One-shot O(nnz(A)) build of the per-row diagonal-dominance margin table. Pair with `coherence-cache-update` to amortise the cost across many streaming events.',
        edgeSafe: false,
      },
      'coherence-cache-update': {
        class: 'SubLinear',
        detail: 'Incremental per-row margin update. O(|dirty| · row_nnz) typical case; up to O(n) cached-vec rescan (no matrix touches) on the rare unavoidable global-min-row recompute.',
        edgeSafe: true,
      },
      // ── Coherence-gated event filter (PR #34) ─────────────────────
      'delta-below-solve-threshold': {
        class: 'Logarithmic',
        detail: 'O(|δ|) cached-input fast-path skip gate. Returns true iff the supplied delta is small enough that ‖A⁻¹ δ‖_∞ ≤ tolerance by the Neumann-envelope bound. Independent of n + nnz once (coherence, min_diag) are cached.',
        edgeSafe: true,
      },
    };

    const entry = table[method];
    if (!entry) {
      throw new McpError(
        ErrorCode.InvalidParams,
        `Unknown method '${method}'. Known: ${Object.keys(table).join(', ')}.`,
      );
    }

    const n = typeof params.matrix_rows === 'number' ? params.matrix_rows : null;
    const nnz = typeof params.matrix_nnz === 'number' ? params.matrix_nnz : null;
    const scaleHint = n
      ? ` (estimated for n=${n}${nnz ? `, nnz=${nnz}` : ''})`
      : '';

    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify(
            {
              success: true,
              method,
              complexity: entry,
              scale_hint: scaleHint.trim() || null,
              note: 'Class table sourced from the Rust Complexity impls in v1.7.0 / 0.3.0. See ADR-001 (Complexity as Architecture) for the strategic context.',
            },
            null,
            2,
          ),
        },
      ],
    };
  }

  /**
   * ADR-001 open Q#3 / PR #41 (Rust). Wire-callable witness for
   * SubLinear orchestrator outputs. Computes the closure-restricted
   * residual `r[i] = b[i] - Σ_j A[i,j]·x_new[j]` for every supplied
   * `(row, value)` entry. `x_new[j]` uses the entry's value if j
   * appears in the entries list; otherwise falls back to
   * `prev_solution[j]` (the closure-boundary contract).
   *
   * Pure-TS implementation — does NOT cross into Rust/WASM. The
   * residual math is straightforward enough that duplicating it
   * here is cheaper than plumbing a new WASM binding. Matches
   * Rust's `verify_sparse_solution` semantics exactly.
   *
   * Cost: O(|entries| · avg_row_nnz). SubLinear in n for sparse
   * DD matrices — same complexity class as the orchestrator
   * whose output it verifies.
   */
  private async handleVerifySparseSolution(params: any) {
    try {
      if (!params.matrix) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing required parameter: matrix');
      }
      if (!params.prev_solution || !Array.isArray(params.prev_solution)) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing or invalid parameter: prev_solution (must be number array)');
      }
      if (!params.vector || !Array.isArray(params.vector)) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing or invalid parameter: vector (RHS b, must be number array)');
      }
      if (!Array.isArray(params.entries)) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing or invalid parameter: entries (must be array of {row, value})');
      }
      const tolerance: number = typeof params.tolerance === 'number' ? params.tolerance : 1e-6;
      if (tolerance < 0) {
        throw new McpError(ErrorCode.InvalidParams, `tolerance must be >= 0, got ${tolerance}`);
      }

      const matrix = params.matrix;
      const n: number = matrix.rows ?? params.prev_solution.length;
      if (params.prev_solution.length !== n) {
        throw new McpError(
          ErrorCode.InvalidParams,
          `prev_solution.length=${params.prev_solution.length} != matrix.rows=${n}`,
        );
      }
      if (params.vector.length !== n) {
        throw new McpError(
          ErrorCode.InvalidParams,
          `vector.length=${params.vector.length} != matrix.rows=${n}`,
        );
      }

      // Build a sparse overlay map: entry row → entry value.
      const overlay = new Map<number, number>();
      for (const e of params.entries) {
        if (typeof e.row !== 'number' || typeof e.value !== 'number') {
          continue; // tolerate malformed entries silently — matches Rust
        }
        if (e.row >= 0 && e.row < n) {
          overlay.set(e.row, e.value);
        }
      }
      const xAt = (j: number): number => {
        const v = overlay.get(j);
        return v !== undefined ? v : (params.prev_solution[j] ?? 0);
      };

      // Threshold: tolerance · max(1, ‖b‖_∞).
      let bInf = 0;
      for (const v of params.vector) {
        const a = Math.abs(v);
        if (a > bInf) bInf = a;
      }
      const threshold = tolerance * Math.max(1, bInf);

      // Compute r[i] = b[i] - A[i,:]·x_new for each entry row.
      let maxResidual = 0;
      let worstRow: number | null = null;
      const rowAccess = (row: number): Array<[number, number]> => {
        // Accept dense (matrix.data: number[][]) or sparse-COO formats.
        if (matrix.format === 'coo' && matrix.data && matrix.data.rowIndices && matrix.data.colIndices && matrix.data.values) {
          // Linear scan of COO triplets for this row. Acceptable for
          // audit cost since |entries| ≪ n.
          const out: Array<[number, number]> = [];
          for (let k = 0; k < matrix.data.rowIndices.length; k++) {
            if (matrix.data.rowIndices[k] === row) {
              out.push([matrix.data.colIndices[k], matrix.data.values[k]]);
            }
          }
          return out;
        }
        if (matrix.data && Array.isArray(matrix.data[row])) {
          // Dense row.
          const out: Array<[number, number]> = [];
          for (let j = 0; j < matrix.data[row].length; j++) {
            const v = matrix.data[row][j];
            if (v !== 0) out.push([j, v]);
          }
          return out;
        }
        return [];
      };

      for (const e of params.entries) {
        if (typeof e.row !== 'number' || e.row < 0 || e.row >= n) continue;
        let ax = 0;
        for (const [j, aij] of rowAccess(e.row)) {
          ax += aij * xAt(j);
        }
        const r = Math.abs(params.vector[e.row] - ax);
        if (r > maxResidual) {
          maxResidual = r;
          worstRow = e.row;
        }
      }

      return {
        content: [
          {
            type: 'text',
            text: JSON.stringify(
              {
                ok: maxResidual <= threshold,
                max_residual: maxResidual,
                threshold,
                worst_row: worstRow,
                note: 'Closure-restricted residual audit per ADR-001 open Q#3 (PR #41). Failure on strict-DD input indicates a real solver bug, not a tolerance miss.',
              },
              null,
              2,
            ),
          },
        ],
      };
    } catch (error) {
      if (error instanceof McpError) {
        throw error;
      }
      throw new McpError(
        ErrorCode.InternalError,
        `verifySparseSolution error: ${error instanceof Error ? error.message : String(error)}`,
      );
    }
  }

  /**
   * Wire-callable coherence-score primitive. Mirrors Rust's
   * `coherence::coherence_score(matrix)`:
   *
   *   margin(i) = (|A[i,i]| - Σ_{j ≠ i} |A[i,j]|) / |A[i,i]|
   *   coherence(A) = min_i margin(i)
   *
   * Strictly DD matrices score in (0, 1]; the boundary case scores 0;
   * non-DD scores negative; rows with zero diagonal score -Infinity.
   *
   * Pure-TS — no WASM bridge needed. Cost O(nnz(A)).
   */
  private async handleCoherenceScore(params: any) {
    try {
      if (!params.matrix) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing required parameter: matrix');
      }
      const matrix = params.matrix;
      const n: number = matrix.rows ?? 0;
      const cols: number = matrix.cols ?? n;
      if (n === 0) {
        // Vacuous: an empty matrix is "perfectly coherent" by convention.
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify(
                {
                  coherence: 1.0,
                  worst_row: null,
                  is_strict_dd: true,
                  note: 'Empty matrix; coherence reported as 1.0 by convention.',
                },
                null,
                2,
              ),
            },
          ],
        };
      }

      // Build a per-row sum-of-off-diagonals + diag-lookup. Accept dense
      // or sparse-COO formats (same shape as `solve`).
      const diag: number[] = new Array(n).fill(0);
      const offSum: number[] = new Array(n).fill(0);

      if (matrix.format === 'coo' && matrix.data && matrix.data.rowIndices && matrix.data.colIndices && matrix.data.values) {
        const ri = matrix.data.rowIndices;
        const ci = matrix.data.colIndices;
        const vs = matrix.data.values;
        for (let k = 0; k < ri.length; k++) {
          const r = ri[k];
          const c = ci[k];
          const v = vs[k];
          if (r < 0 || r >= n) continue;
          if (r === c) {
            diag[r] = v;
          } else {
            offSum[r] += Math.abs(v);
          }
        }
      } else if (matrix.data && Array.isArray(matrix.data[0])) {
        // Dense row-major.
        for (let i = 0; i < n; i++) {
          const row = matrix.data[i];
          if (!Array.isArray(row)) continue;
          for (let j = 0; j < cols; j++) {
            const v = row[j] ?? 0;
            if (v === 0) continue;
            if (j === i) {
              diag[i] = v;
            } else {
              offSum[i] += Math.abs(v);
            }
          }
        }
      } else {
        throw new McpError(
          ErrorCode.InvalidParams,
          'matrix must be in dense (data: number[][]) or sparse-COO (format: "coo", data.rowIndices/colIndices/values) format',
        );
      }

      // Compute per-row margins and the global minimum.
      let worstMargin = Number.POSITIVE_INFINITY;
      let worstRow: number | null = null;
      for (let i = 0; i < n; i++) {
        const d = Math.abs(diag[i]);
        if (d <= 1e-300) {
          // Zero diagonal → score -Infinity. Match Rust semantics.
          worstMargin = Number.NEGATIVE_INFINITY;
          worstRow = i;
          break;
        }
        const margin = (d - offSum[i]) / d;
        if (margin < worstMargin) {
          worstMargin = margin;
          worstRow = i;
        }
      }

      return {
        content: [
          {
            type: 'text',
            text: JSON.stringify(
              {
                coherence: worstMargin,
                worst_row: worstRow,
                is_strict_dd: Number.isFinite(worstMargin) && worstMargin > 0,
                note: 'Diagonal-dominance margin per ADR-001 item #3. Positive ⇒ Neumann convergence guaranteed; negative ⇒ iterative solvers may diverge.',
              },
              null,
              2,
            ),
          },
        ],
      };
    } catch (error) {
      if (error instanceof McpError) {
        throw error;
      }
      throw new McpError(
        ErrorCode.InternalError,
        `coherenceScore error: ${error instanceof Error ? error.message : String(error)}`,
      );
    }
  }

  /**
   * Wire-callable bounded-depth row-graph BFS. Mirrors Rust's
   * `closure::closure_indices(matrix, seeds, depth)`:
   *
   *   closure_0 = seeds
   *   closure_{d+1} = closure_d ∪ {j : ∃ i ∈ closure_d, A[i,j] ≠ 0}
   *
   * Pure-TS — no WASM bridge. Cost O(depth · branch · |closure|).
   */
  private async handleClosureIndices(params: any) {
    try {
      if (!params.matrix) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing required parameter: matrix');
      }
      if (!Array.isArray(params.seeds)) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing or invalid parameter: seeds (must be number array)');
      }
      if (typeof params.depth !== 'number' || params.depth < 0 || !Number.isInteger(params.depth)) {
        throw new McpError(ErrorCode.InvalidParams, 'depth must be a non-negative integer');
      }

      const matrix = params.matrix;
      const n: number = matrix.rows ?? 0;
      const depth: number = params.depth;

      if (n === 0) {
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({ closure: [], size: 0 }, null, 2),
            },
          ],
        };
      }

      // Build a per-row adjacency list once (lazy expansion would
      // re-scan the matrix per row at every depth; up-front is
      // cheaper for bounded-depth BFS).
      const adj: number[][] = Array.from({ length: n }, () => []);
      if (matrix.format === 'coo' && matrix.data && matrix.data.rowIndices && matrix.data.colIndices) {
        const ri = matrix.data.rowIndices;
        const ci = matrix.data.colIndices;
        for (let k = 0; k < ri.length; k++) {
          const r = ri[k];
          const c = ci[k];
          if (r >= 0 && r < n && c !== r) {
            adj[r].push(c);
          }
        }
      } else if (matrix.data && Array.isArray(matrix.data[0])) {
        for (let i = 0; i < n; i++) {
          const row = matrix.data[i];
          if (!Array.isArray(row)) continue;
          for (let j = 0; j < row.length; j++) {
            const v = row[j];
            if (v !== 0 && j !== i) {
              adj[i].push(j);
            }
          }
        }
      } else {
        throw new McpError(
          ErrorCode.InvalidParams,
          'matrix must be in dense or sparse-COO format',
        );
      }

      // BFS-style expansion, deduped via a visited Set.
      const visited = new Set<number>();
      const seeds: number[] = params.seeds.filter(
        (s: any) => typeof s === 'number' && s >= 0 && s < n,
      );
      let frontier: number[] = [];
      for (const s of seeds) {
        if (!visited.has(s)) {
          visited.add(s);
          frontier.push(s);
        }
      }

      for (let d = 0; d < depth; d++) {
        if (frontier.length === 0) break;
        const next: number[] = [];
        for (const row of frontier) {
          for (const col of adj[row]) {
            if (!visited.has(col)) {
              visited.add(col);
              next.push(col);
            }
          }
        }
        frontier = next;
      }

      const out = Array.from(visited).sort((a, b) => a - b);
      return {
        content: [
          {
            type: 'text',
            text: JSON.stringify(
              {
                closure: out,
                size: out.length,
                note: out.length >= n
                  ? `Closure covers the full matrix (${out.length}/${n} rows). SubLinear orchestrators will degrade to Linear cost; pick a smaller depth.`
                  : `Closure covers ${out.length}/${n} rows (${((out.length / n) * 100).toFixed(1)}%). SubLinear orchestrators apply.`,
              },
              null,
              2,
            ),
          },
        ],
      };
    } catch (error) {
      if (error instanceof McpError) {
        throw error;
      }
      throw new McpError(
        ErrorCode.InternalError,
        `closureIndices error: ${error instanceof Error ? error.message : String(error)}`,
      );
    }
  }

  /**
   * Wire-callable SubLinear orchestrator. Mirrors Rust's
   * `incremental::solve_on_change_sublinear(matrix, prev, b_new,
   * delta, closure_depth, max_terms, tolerance)`. Pure-TS — no
   * WASM bridge. Composes:
   *
   *   1. closure_indices(matrix, delta.indices, closure_depth)
   *   2. for each closure entry: per-entry Neumann iteration
   *      restricted to the closure (matches src/entry.rs math)
   *   3. return Vec<{row, value}>
   *
   * Cost O(|closure| · max_terms · branch). End-to-end SubLinear
   * in n for sparse DD matrices with bounded depth + max_terms.
   */
  private async handleSolveOnChangeSublinear(params: any) {
    try {
      if (!params.matrix) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing required parameter: matrix');
      }
      if (!Array.isArray(params.vector)) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing or invalid parameter: vector (RHS b_new)');
      }
      if (!Array.isArray(params.delta_indices)) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing or invalid parameter: delta_indices');
      }
      const closureDepth: number = typeof params.closure_depth === 'number' ? params.closure_depth : 4;
      const maxTerms: number = typeof params.max_terms === 'number' ? params.max_terms : 32;
      const tolerance: number = typeof params.tolerance === 'number' ? params.tolerance : 1e-8;
      if (closureDepth < 0 || maxTerms < 1 || tolerance <= 0) {
        throw new McpError(
          ErrorCode.InvalidParams,
          'closure_depth ≥ 0, max_terms ≥ 1, tolerance > 0 required',
        );
      }

      const matrix = params.matrix;
      const n: number = matrix.rows ?? params.vector.length;
      if (params.vector.length !== n) {
        throw new McpError(
          ErrorCode.InvalidParams,
          `vector.length=${params.vector.length} != matrix.rows=${n}`,
        );
      }

      // Materialise sparse row adjacency with values (need values for
      // the Neumann iteration, not just adjacency for closure BFS).
      const rowEntries: Array<Array<[number, number]>> = Array.from({ length: n }, () => []);
      const diag: number[] = new Array(n).fill(0);
      if (matrix.format === 'coo' && matrix.data && matrix.data.rowIndices && matrix.data.colIndices && matrix.data.values) {
        const ri = matrix.data.rowIndices;
        const ci = matrix.data.colIndices;
        const vs = matrix.data.values;
        for (let k = 0; k < ri.length; k++) {
          const r = ri[k];
          const c = ci[k];
          const v = vs[k];
          if (r < 0 || r >= n) continue;
          if (r === c) diag[r] = v;
          else rowEntries[r].push([c, v]);
        }
      } else if (matrix.data && Array.isArray(matrix.data[0])) {
        for (let i = 0; i < n; i++) {
          const row = matrix.data[i];
          if (!Array.isArray(row)) continue;
          for (let j = 0; j < row.length; j++) {
            const v = row[j];
            if (v === 0) continue;
            if (j === i) diag[i] = v;
            else rowEntries[i].push([j, v]);
          }
        }
      } else {
        throw new McpError(
          ErrorCode.InvalidParams,
          'matrix must be in dense or sparse-COO format',
        );
      }

      // (1) Closure BFS — same as handleClosureIndices.
      const visited = new Set<number>();
      const seeds: number[] = params.delta_indices.filter(
        (s: any) => typeof s === 'number' && s >= 0 && s < n,
      );
      let frontier: number[] = [];
      for (const s of seeds) {
        if (!visited.has(s)) {
          visited.add(s);
          frontier.push(s);
        }
      }
      for (let d = 0; d < closureDepth; d++) {
        if (frontier.length === 0) break;
        const next: number[] = [];
        for (const row of frontier) {
          for (const [col] of rowEntries[row]) {
            if (!visited.has(col)) {
              visited.add(col);
              next.push(col);
            }
          }
        }
        frontier = next;
      }
      const closure = Array.from(visited).sort((a, b) => a - b);
      if (closure.length === 0) {
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({ entries: [], closure_size: 0, max_terms: maxTerms, closure_depth: closureDepth }, null, 2),
            },
          ],
        };
      }

      // (2) Per-entry Neumann. For each target ∈ closure, compute
      //     x[target] = Σ_k y_k[target] where y_0[j] = b[j]/A[j,j]
      //     and y_{k+1}[j] = -(1/A[j,j]) · Σ_{m≠j, m∈closure} A[j,m]·y_k[m].
      const closureSet = new Set<number>(closure);
      const out: Array<{ row: number; value: number }> = [];
      for (const target of closure) {
        // Validate diagonal at every closure row.
        let bad = false;
        for (const j of closure) {
          if (Math.abs(diag[j]) <= 1e-300) {
            bad = true;
            break;
          }
        }
        if (bad) {
          throw new McpError(
            ErrorCode.InvalidParams,
            'matrix has zero diagonal in the closure — not strict-DD',
          );
        }
        // y_0[j] = b[j] / A[j,j]
        const y = new Map<number, number>();
        for (const j of closure) {
          y.set(j, params.vector[j] / diag[j]);
        }
        let xTarget = y.get(target) ?? 0;

        // Iterate up to max_terms times.
        for (let k = 1; k <= maxTerms; k++) {
          const yNext = new Map<number, number>();
          for (const j of closure) {
            let sum = 0;
            for (const [m, ajm] of rowEntries[j]) {
              if (m === j) continue;
              if (!closureSet.has(m)) continue;
              const ym = y.get(m);
              if (ym !== undefined) sum += ajm * ym;
            }
            yNext.set(j, -sum / diag[j]);
          }
          const delta = yNext.get(target) ?? 0;
          xTarget += delta;
          // Early-exit on tolerance.
          if (Math.abs(delta) < tolerance) break;
          // Swap y ← y_next.
          y.clear();
          for (const [k2, v2] of yNext) y.set(k2, v2);
        }
        out.push({ row: target, value: xTarget });
      }

      return {
        content: [
          {
            type: 'text',
            text: JSON.stringify(
              {
                entries: out,
                closure_size: closure.length,
                max_terms: maxTerms,
                closure_depth: closureDepth,
                note:
                  closure.length >= n
                    ? `Closure covers full matrix (${closure.length}/${n}) — orchestrator degraded to Linear. Reduce closure_depth.`
                    : `Closure covers ${closure.length}/${n} rows (${((closure.length / n) * 100).toFixed(1)}%). SubLinear in n.`,
              },
              null,
              2,
            ),
          },
        ],
      };
    } catch (error) {
      if (error instanceof McpError) {
        throw error;
      }
      throw new McpError(
        ErrorCode.InternalError,
        `solveOnChangeSublinear error: ${error instanceof Error ? error.message : String(error)}`,
      );
    }
  }

  /**
   * Wire-callable contrastive top-k orchestrator. Mirrors Rust's
   * `contrastive::contrastive_solve_on_change_sublinear`. Delegates
   * the SubLinear solve to handleSolveOnChangeSublinear, then ranks
   * the resulting entries by `|current - prev_solution[row]|` and
   * returns the top-k.
   *
   * Pure-TS — no WASM bridge. Same complexity class as the
   * underlying orchestrator: end-to-end SubLinear in n.
   */
  private async handleContrastiveSolveOnChangeSublinear(params: any) {
    try {
      if (!Array.isArray(params.prev_solution)) {
        throw new McpError(
          ErrorCode.InvalidParams,
          'Missing or invalid parameter: prev_solution (must be number array)',
        );
      }
      const k: number = typeof params.k === 'number' ? params.k : 3;
      if (k < 1) {
        throw new McpError(ErrorCode.InvalidParams, 'k must be >= 1');
      }

      // Delegate the orchestrator work to the existing handler.
      // Returns content[0].text as JSON with {entries, closure_size, ...}.
      const inner = await this.handleSolveOnChangeSublinear({
        matrix: params.matrix,
        vector: params.vector,
        delta_indices: params.delta_indices,
        closure_depth: params.closure_depth,
        max_terms: params.max_terms,
        tolerance: params.tolerance,
      });

      type Entry = { row: number; value: number };
      type InnerOut = {
        entries: Entry[];
        closure_size: number;
        max_terms: number;
        closure_depth: number;
        note?: string;
      };
      const innerJson: InnerOut = JSON.parse(
        (inner.content[0] as any).text,
      );

      const prev: number[] = params.prev_solution;
      // Score each closure entry: anomaly = |current - prev[row]|.
      const scored: Array<{ row: number; baseline: number; current: number; anomaly: number }> = [];
      for (const e of innerJson.entries) {
        if (e.row < 0 || e.row >= prev.length) continue;
        const baseline = prev[e.row];
        const current = e.value;
        scored.push({
          row: e.row,
          baseline,
          current,
          anomaly: Math.abs(current - baseline),
        });
      }
      // Sort descending by anomaly; tie-break ascending by row (deterministic).
      scored.sort((a, b) => {
        if (b.anomaly !== a.anomaly) return b.anomaly - a.anomaly;
        return a.row - b.row;
      });
      const topK = scored.slice(0, k);

      return {
        content: [
          {
            type: 'text',
            text: JSON.stringify(
              {
                top_k: topK,
                closure_size: innerJson.closure_size,
                k_returned: topK.length,
                max_terms: innerJson.max_terms,
                closure_depth: innerJson.closure_depth,
                note:
                  topK.length < k
                    ? `Closure produced only ${topK.length} valid entries, fewer than k=${k}. Increase closure_depth or check matrix shape.`
                    : `Top-${k} anomalies from a ${innerJson.closure_size}-row closure.`,
              },
              null,
              2,
            ),
          },
        ],
      };
    } catch (error) {
      if (error instanceof McpError) {
        throw error;
      }
      throw new McpError(
        ErrorCode.InternalError,
        `contrastiveSolveOnChangeSublinear error: ${error instanceof Error ? error.message : String(error)}`,
      );
    }
  }

  private async handleSaveVectorToFile(params: any) {
    try {
      // Validate required parameters
      if (!params.vector || !Array.isArray(params.vector)) {
        throw new McpError(ErrorCode.InvalidParams, 'Missing or invalid required parameter: vector (must be an array of numbers)');
      }
      if (!params.file_path || typeof params.file_path !== 'string') {
        throw new McpError(ErrorCode.InvalidParams, 'Missing or invalid required parameter: file_path (must be a string)');
      }

      const vector = params.vector;
      const filePath = params.file_path;
      const format = params.format;

      // Validate vector contains only numbers
      if (vector.some((v: any) => typeof v !== 'number' || isNaN(v))) {
        throw new McpError(ErrorCode.InvalidParams, 'Vector must contain only valid numbers');
      }

      await this.saveVectorToFile(vector, filePath, format);

      return {
        content: [{
          type: 'text',
          text: JSON.stringify({
            success: true,
            message: `Vector of size ${vector.length} saved to ${filePath}`,
            file_path: filePath,
            vector_size: vector.length,
            format_used: this.getFileFormat(filePath, format),
            metadata: {
              operation: 'SAVE_VECTOR_TO_FILE',
              timestamp: new Date().toISOString()
            }
          }, null, 2)
        }]
      };

    } catch (error) {
      if (error instanceof McpError) {
        throw error;
      }
      throw new McpError(
        ErrorCode.InternalError,
        `Save vector to file error: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  private async loadVectorFromFile(filePath: string): Promise<number[]> {
    try {
      // SECURITY (issue #19, CWE-73): same sink class as saveVectorToFile —
      // filePath was attacker-controlled and used to read arbitrary files
      // visible to the MCP process. Read only basenames from the configured
      // vector dir, with O_NOFOLLOW so a planted symlink can't redirect us.
      let absolutePath: string;
      let fileContent: string;
      try {
        absolutePath = resolveVectorPath(filePath);
        fileContent = safeReadVector(filePath);
      } catch (err) {
        if (err instanceof SafePathError) {
          throw new McpError(
            ErrorCode.InvalidParams,
            `Vector file path rejected (${err.code}): ${err.message}. ` +
              `Use a basename only; files are read from ${DEFAULT_VECTOR_DIR} ` +
              `(override with $SUBLINEAR_SOLVER_VECTOR_DIR).`,
          );
        }
        if ((err as NodeJS.ErrnoException)?.code === 'ENOENT') {
          throw new McpError(ErrorCode.InvalidParams, `Vector file not found in vector dir`);
        }
        throw err;
      }
      const path = await import('path');
      const extension = path.extname(absolutePath).toLowerCase();

      let vector: number[];

      if (extension === '.json') {
        // Parse JSON format
        const data = JSON.parse(fileContent);
        if (Array.isArray(data)) {
          vector = data.map(Number);
        } else if (data.vector && Array.isArray(data.vector)) {
          vector = data.vector.map(Number);
        } else {
          throw new Error('JSON file must contain an array or an object with a "vector" property');
        }
      } else if (extension === '.csv') {
        // Parse CSV format (simple comma-separated values)
        const lines = fileContent.trim().split('\n');
        if (lines.length === 1) {
          // Single line CSV
          vector = lines[0].split(',').map(s => Number(s.trim()));
        } else {
          // Multi-line CSV - take first column or first row based on structure
          vector = lines.map(line => Number(line.split(',')[0].trim()));
        }
      } else if (extension === '.txt') {
        // Parse text format (space or newline separated)
        vector = fileContent.trim()
          .split(/\s+/)
          .map(Number)
          .filter(n => !isNaN(n));
      } else {
        throw new Error(`Unsupported file format: ${extension}. Supported formats: .json, .csv, .txt`);
      }

      // Validate all values are numbers
      if (vector.some(isNaN)) {
        throw new Error('Vector file contains non-numeric values');
      }

      if (vector.length === 0) {
        throw new Error('Vector file is empty or contains no valid numbers');
      }

      console.log(`📁 Loaded vector of size ${vector.length} from ${filePath}`);
      return vector;

    } catch (error) {
      if (error instanceof McpError) {
        throw error;
      }
      throw new McpError(
        ErrorCode.InvalidParams,
        `Failed to load vector from file: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  private async saveVectorToFile(vector: number[], filePath: string, format?: string): Promise<string> {
    // SECURITY (issue #19, CWE-73): file_path is attacker-controlled via the
    // saveVectorToFile MCP tool. Previously this called path.resolve(filePath)
    // + fs.writeFileSync, letting any caller write anywhere the MCP process
    // could touch. We now force the write into the configured vector dir
    // (DEFAULT_VECTOR_DIR or $SUBLINEAR_SOLVER_VECTOR_DIR) using a
    // basename-only contract enforced by `safe-path`.
    const fileFormat = this.getFileFormat(filePath, format);

    let content: string;
    switch (fileFormat) {
      case 'json':
        content = JSON.stringify(vector, null, 2);
        break;
      case 'csv':
        content = vector.join(',');
        break;
      case 'txt':
        content = vector.join('\n');
        break;
      default:
        throw new Error(`Unsupported format: ${fileFormat}`);
    }

    const absolutePath = safeWriteVector(filePath, content);
    console.log(`💾 Saved vector of size ${vector.length} to ${absolutePath} (${fileFormat} format)`);
    return absolutePath;
  }

  private getFileFormat(filePath: string, explicitFormat?: string): string {
    if (explicitFormat) {
      return explicitFormat.toLowerCase();
    }

    const extension = filePath.split('.').pop()?.toLowerCase();
    if (extension && ['json', 'csv', 'txt'].includes(extension)) {
      return extension;
    }

    // Default to JSON if no valid extension
    return 'json';
  }

  private generateRecommendations(analysis: any): string[] {
    const recommendations: string[] = [];

    if (!analysis.isDiagonallyDominant) {
      recommendations.push('Matrix is not diagonally dominant. Consider matrix preconditioning or using a different solver.');
    } else if (analysis.dominanceStrength < 0.1) {
      recommendations.push('Weak diagonal dominance detected. Convergence may be slow.');
    }

    if (analysis.sparsity > 0.9) {
      recommendations.push('Matrix is very sparse. Consider using sparse matrix formats for better performance.');
    }

    if (!analysis.isSymmetric && analysis.isDiagonallyDominant) {
      recommendations.push('Matrix is asymmetric but diagonally dominant. Random walk methods may be most effective.');
    }

    if (analysis.size.rows > 10000) {
      recommendations.push('Large matrix detected. Consider using sublinear estimation methods for specific entries rather than full solve.');
    }

    return recommendations;
  }

  async run(): Promise<void> {
    const transport = new StdioServerTransport();
    await this.server.connect(transport);
    console.error('Sublinear Solver MCP server running on stdio');
  }
}

// Main execution
if (import.meta.url === `file://${process.argv[1]}`) {
  const server = new SublinearSolverMCPServer();
  server.run().catch(console.error);
}