#!/usr/bin/env node
/**
 * Prove the ruflo 3.25.0 Lattice WASM embedder tier is a correct, fail-closed,
 * zero-regression seam — WITHOUT fabricating a package that does not exist.
 *
 *   1. default (no package)      -> tier = fallback:hash, behaves as before
 *   2. bad package specifier     -> fails closed to hash (never throws)
 *   3. local stub activates      -> tier = lattice-wasm, retrieval still works
 *      (the stub is a labeled test double, NOT a real model)
 *   4. zero regression           -> the flywheel's held-out result under the
 *      fallback path is identical to the sealed receipt
 *
 * Run: node verify-embedder.mjs
 */
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

import { makeEmbeddingProvider } from './src/embedder.mjs';
import { compileAndMeasure } from './src/flywheel-core.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
let failures = 0;
const check = (name, ok, detail = '') => { console.log(`  ${ok ? 'PASS' : 'FAIL'}  ${name}${detail ? ' — ' + detail : ''}`); if (!ok) failures++; };
const approx = (a, b) => Math.abs(a - b) < 1e-9;

console.log('Verifying the Lattice WASM embedder tier (fail-closed, zero-regression):\n');

// 1. Default: no package installed -> hash fallback.
const def = await makeEmbeddingProvider({});
check('default resolves to hash fallback (package absent)', def.tier === 'fallback:hash', def.reason);
const dv = await def.provider.embed('hello');
check('fallback provider returns a usable vector', dv instanceof Float32Array && dv.length > 0, `dim=${dv.length}`);

// 2. Bad specifier: must fail closed, never throw.
let threw = false, bad;
try { bad = await makeEmbeddingProvider({ RUFLO_LATTICE_WASM_PKG: '@ruvector/definitely-not-real-xyz' }); } catch { threw = true; }
check('bad package specifier fails closed (no throw)', !threw && bad?.tier === 'fallback:hash', bad?.reason);

// 3. Local stub: the ACTIVATION path fires when a package IS present.
const stub = await makeEmbeddingProvider({ RUFLO_LATTICE_WASM_PKG: join(HERE, 'src/lattice-stub.mjs'), RUFLO_EMBED_MODEL: 'minilm' });
check('local stub activates the lattice tier', stub.tier === 'lattice-wasm', stub.reason);
check('activated tier reports its model', stub.model === 'minilm');
const sv = await stub.provider.embed('hello');
check('activated provider returns a real numeric vector', sv instanceof Float32Array && sv.length >= 8, `dim=${sv.length}`);

// 3b. The whole flywheel runs end-to-end through the activated stub tier.
const src = readFileSync(join(HERE, 'GUIDANCE.md'), 'utf8');
const stubRun = await compileAndMeasure(src, { RUFLO_LATTICE_WASM_PKG: join(HERE, 'src/lattice-stub.mjs') });
check('retrieval works end-to-end on the activated tier', stubRun.embedder.tier === 'lattice-wasm' && stubRun.cand.intentPrecision >= stubRun.base.intentPrecision,
  `precision ${stubRun.base.intentPrecision.toFixed(3)} -> ${stubRun.cand.intentPrecision.toFixed(3)}`);

// 4. Zero regression: the sealed receipt (produced on the fallback path) must
//    match a fresh fallback-path run exactly.
const receipt = JSON.parse(readFileSync(join(HERE, 'flywheel/RECEIPT.json'), 'utf8'));
const fresh = await compileAndMeasure(src, {}); // force fallback
check('sealed run used the fallback tier', receipt.embedder?.tier === 'fallback:hash', receipt.embedder?.tier);
check('zero regression: bundle digest identical under fallback', fresh.bundleDigest === receipt.bundleDigest);
check('zero regression: baseline precision identical under fallback', approx(fresh.base.intentPrecision, receipt.heldOut.baselineIntentPrecision), fresh.base.intentPrecision.toFixed(3));
check('zero regression: candidate precision identical under fallback', approx(fresh.cand.intentPrecision, receipt.heldOut.candidateIntentPrecision), fresh.cand.intentPrecision.toFixed(3));

console.log(`\n  the tier is optional, fail-closed, and zero-regression: absent package => embeddings resolve exactly as before.`);
console.log(`  the local stub proves the activation path fires when a package is present (it is a test double, not a real model).`);
console.log(`\n${failures === 0 ? 'VERIFIED' : 'FAILED'}: Lattice embedder tier, ${failures} check(s) failed.`);
process.exit(failures === 0 ? 0 : 1);
