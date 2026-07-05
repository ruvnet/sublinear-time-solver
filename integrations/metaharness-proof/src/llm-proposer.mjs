/**
 * Open-ended proposer backed by the real Claude CLI (`claude -p`).
 *
 * Asks Claude to improve a solver genome and returns the proposed source. This
 * is the novelty engine's generator: it can propose ARBITRARY code (a new
 * algorithm), not a point in a fixed grid. Output is non-deterministic — the
 * DISCOVERY is validated by re-executing the proposed code, not by reproducing
 * the LLM call.
 */
import { execFileSync } from 'node:child_process';

// Static safety gate: genomes must be pure numeric ESM. Reject anything that
// could touch the host. Runaway loops are handled by the subprocess timeout.
const FORBIDDEN = /\b(import|require|process|child_process|fs|net|http|fetch|eval|Function|globalThis|__dirname|Deno|Bun|WebSocket|XMLHttpRequest)\b|`|=>\s*\{[^}]*while\s*\(\s*true/;

export function extractCode(text) {
  const fence = text.match(/```(?:javascript|js|mjs)?\s*([\s\S]*?)```/i);
  let code = (fence ? fence[1] : text).trim();
  if (!/export\s+function\s+solve/.test(code) && /function\s+solve/.test(code)) code = code.replace(/function\s+solve/, 'export function solve');
  return code;
}

export function isSafe(code) {
  if (!/export\s+function\s+solve\s*\(/.test(code)) return { ok: false, why: 'missing export function solve(...)' };
  if (FORBIDDEN.test(code)) return { ok: false, why: 'contains a forbidden token (import/io/eval/template-literal/etc.)' };
  if (code.length > 8000) return { ok: false, why: 'too long' };
  return { ok: true };
}

/**
 * @param {string} championSource
 * @param {number} solvedNow how many of the suite the champion solves
 * @param {number} total suite size
 * @param {string[]} failingKinds hints about which system classes fail
 * @param {string[]} priorNotes short notes from earlier rejected attempts
 */
export function propose(championSource, solvedNow, total, failingKinds, priorNotes = []) {
  const prompt = [
    'You are improving a numerical linear-system solver written in JavaScript.',
    '',
    'Current champion (a pure ES module). It solves ' + solvedNow + '/' + total + ' test systems:',
    '```js', championSource.trim(), '```',
    '',
    'The test suite mixes two classes of real sparse systems A x = b:',
    ' - diagonally-dominant (the champion solves these), and',
    ' - symmetric-positive-definite but NOT diagonally dominant (' + failingKinds.join(', ') + ').',
    'The champion uses plain Jacobi iteration, which DIVERGES on the second class no',
    'matter how many iterations you run — so simply increasing the iteration count',
    'will NOT help. You must change the numerical METHOD.',
    priorNotes.length ? '\nEarlier attempts that did NOT improve: ' + priorNotes.join('; ') : '',
    '',
    'Propose an improved solve(matrix, b) that solves MORE systems to residual < 1e-6.',
    'matrix is COO: { rows, cols, values, rowIndices, colIndices }. Return the solution as a number[].',
    'Constraints: a SINGLE self-contained ES module that `export function solve(matrix, b)`.',
    'Pure numeric code only — NO imports, NO I/O, NO eval, NO template literals. Keep it under ~120 lines.',
    'Return ONLY the code in one ```js code block.',
  ].join('\n');

  let raw;
  try {
    raw = execFileSync('claude', ['-p', prompt], { encoding: 'utf8', timeout: 180000, maxBuffer: 4 * 1024 * 1024 });
  } catch (e) {
    return { ok: false, why: 'claude -p failed: ' + (e.message || e).slice(0, 120) };
  }
  const code = extractCode(raw);
  const safe = isSafe(code);
  if (!safe.ok) return { ok: false, why: safe.why, raw: code.slice(0, 400) };
  return { ok: true, code, prompt };
}
