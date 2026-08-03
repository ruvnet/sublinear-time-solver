/**
 * Lattice tier ACTIVATION test double — NOT a real embedding model.
 *
 * `@ruvector/lattice-wasm` does not exist on npm yet (it 404s). To prove the
 * fail-closed seam's ACTIVATION path actually works when a package IS present,
 * point RUFLO_LATTICE_WASM_PKG at this file:
 *
 *   RUFLO_LATTICE_WASM_PKG=./src/lattice-stub.mjs node run-flywheel.mjs
 *
 * It exposes the `embed(text, model)` surface the real module would, returning
 * a DETERMINISTIC char-hash vector. This has no semantic meaning — its ONLY
 * purpose is to demonstrate that the adapter probe, init, and provider wrapping
 * fire correctly. It must never be presented as a real embedding upgrade.
 */
const DIM = 64;

export function embed(text, _model) {
  const v = new Float32Array(DIM);
  const s = String(text);
  for (let i = 0; i < s.length; i++) {
    const c = s.charCodeAt(i);
    v[(c + i) % DIM] += Math.sin(c * 0.017 + i * 0.013);
  }
  // L2 normalize so cosine similarity is well-behaved.
  let n = 0; for (let i = 0; i < DIM; i++) n += v[i] * v[i];
  n = Math.sqrt(n) || 1;
  for (let i = 0; i < DIM; i++) v[i] /= n;
  return v;
}

export default { embed };
