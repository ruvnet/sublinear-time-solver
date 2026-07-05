/**
 * Lattice WASM embedder tier — fail-closed adapter seam (ruflo 3.25.0).
 *
 * ruflo 3.25.0 adds a real multi-model WASM embedder as the PRIMARY embedding
 * tier, ahead of the existing ONNX/hash path. Per ruvnet's own gist it is
 * currently a "fail-closed, optional adapter seam that is dormant" —
 * `@ruvector/lattice-wasm` 404s on npm today (verified: `npm view` → E404).
 *
 * This module implements that seam against the real `@claude-flow/guidance`
 * `IEmbeddingProvider` interface, matching the gist's contract exactly:
 *   optional · fail-closed · zero-regression. If the package is absent (the
 *   current reality), or WASM init fails, or the module exposes an unexpected
 *   surface, or the verification probe fails — the tier is skipped and
 *   embeddings resolve EXACTLY as before (HashEmbeddingProvider). When the
 *   package is published, retrieval upgrades to real embeddings with no code
 *   change here.
 *
 *   RUFLO_LATTICE_WASM_PKG  package specifier   (default @ruvector/lattice-wasm)
 *   RUFLO_EMBED_MODEL       model name          (default minilm)
 *
 * The package specifier may also be a local path (e.g. ./src/lattice-stub.mjs)
 * so the activation path itself can be exercised with a labeled test double.
 */
import { HashEmbeddingProvider } from '@claude-flow/guidance';

const DEFAULT_PKG = '@ruvector/lattice-wasm';
const MODELS = ['minilm', 'bge', 'paraphrase-miniLM', 'qwen3-0.6b'];

/** Coerce a WASM/JS embedding return into a Float32Array, or null if not numeric. */
function toFloat32(v) {
  if (v instanceof Float32Array) return v.length ? v : null;
  if (Array.isArray(v) && v.every((x) => typeof x === 'number')) return v.length ? Float32Array.from(v) : null;
  if (v && typeof v === 'object' && typeof v.length === 'number' && typeof v[0] === 'number') return Float32Array.from(v);
  return null;
}

/** Probe the plausible wasm-bindgen surfaces the gist lists. Returns an embed fn or null. */
function resolveEmbedFn(mod, model) {
  const m = mod?.default ?? mod;
  const surfaces = [
    () => typeof m.embed === 'function' && ((t) => m.embed(t, model)),
    () => typeof m.embedText === 'function' && ((t) => m.embedText(t, model)),
    () => typeof m.embed === 'function' && ((t) => m.embed(t)),
    () => typeof m.Embedder === 'function' && ((t) => new m.Embedder(model).embed(t)),
    () => typeof mod.embed === 'function' && ((t) => mod.embed(t, model)),
  ];
  for (const s of surfaces) { const fn = s(); if (fn) return fn; }
  return null;
}

/**
 * Resolve an embedding provider. Never throws — on any failure it returns the
 * hash fallback with a reason. Returns { provider, tier, model, reason }.
 */
export async function makeEmbeddingProvider(env = process.env) {
  const pkg = env.RUFLO_LATTICE_WASM_PKG || DEFAULT_PKG;
  const model = env.RUFLO_EMBED_MODEL || 'minilm';
  const fallback = (reason) => ({ provider: new HashEmbeddingProvider(), tier: 'fallback:hash', model: null, reason });

  let mod;
  try {
    mod = await import(pkg);
  } catch (e) {
    return fallback(`package "${pkg}" not loadable (${e?.code || String(e?.message || e).slice(0, 60)}) — resolving as before`);
  }

  const embedFn = resolveEmbedFn(mod, model);
  if (!embedFn) return fallback(`"${pkg}" loaded but exposes no known embed surface — skipped`);

  // Optional init hook (wasm-bindgen modules often need it).
  try { if (typeof (mod.default ?? mod).init === 'function') await (mod.default ?? mod).init(); } catch { /* non-fatal */ }

  // Verification probe: the surface MUST return a real numeric vector.
  let dim;
  try {
    const probe = toFloat32(await embedFn('lattice tier verification probe'));
    if (!probe || probe.length < 8) return fallback(`"${pkg}" probe returned a non-vector — skipped`);
    dim = probe.length;
  } catch (e) {
    return fallback(`"${pkg}" probe threw (${String(e?.message || e).slice(0, 50)}) — skipped`);
  }

  const provider = {
    async embed(text) {
      const v = toFloat32(await embedFn(text));
      if (!v) throw new Error('lattice embed returned a non-vector at runtime');
      return v;
    },
    async batchEmbed(texts) { return Promise.all(texts.map((t) => this.embed(t))); },
  };
  return { provider, tier: 'lattice-wasm', model, reason: `active: ${pkg} model=${model} dim=${dim}`, dim };
}

export { MODELS, DEFAULT_PKG };
