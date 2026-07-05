#!/usr/bin/env node
/**
 * Independent verifier for the ruflo guidance flywheel receipt.
 *
 * Trusts NONE of the sealed service output. Re-executes from GUIDANCE.md:
 *   1. recompile the bundle          -> reproduce the bundle digest
 *   2. re-run both bounded policies  -> reproduce held-out intent-precision
 *   3. re-drive the real pipeline    -> reproduce divergence, gate verdict,
 *                                       promotion, and the decision hash
 *   4. recompute the proposal HMAC   -> prove the sealed proposal is authentic
 *                                       and untampered under the demo key
 *
 * proposalId + createdAt are non-deterministic (UUID / clock) and are the ONLY
 * things not reproduced; they do not enter the decision. Everything the gate
 * acted on is recomputed here.
 *
 * Run: node verify-flywheel.mjs
 */
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

import { compileAndMeasure, driveFlywheel, proposalSignature, K } from './src/flywheel-core.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
const OUT = join(HERE, 'flywheel');
let failures = 0;
const check = (name, ok, detail = '') => { console.log(`  ${ok ? 'PASS' : 'FAIL'}  ${name}${detail ? ' — ' + detail : ''}`); if (!ok) failures++; };
const approx = (a, b) => Math.abs(a - b) < 1e-9;

console.log('Verifying the ruflo guidance flywheel by independent re-execution:\n');

const receipt = JSON.parse(readFileSync(join(OUT, 'RECEIPT.json'), 'utf8'));
const src = readFileSync(join(HERE, 'GUIDANCE.md'), 'utf8');

// 1-2. Recompile + re-measure with the real compiler and retriever.
const { bundleDigest, base, cand } = await compileAndMeasure(src);
check('bundle digest reproduces', bundleDigest === receipt.bundleDigest, bundleDigest.slice(0, 12) + '…');
check('baseline intent-precision reproduces', approx(base.intentPrecision, receipt.heldOut.baselineIntentPrecision), base.intentPrecision.toFixed(3));
check('candidate intent-precision reproduces', approx(cand.intentPrecision, receipt.heldOut.candidateIntentPrecision), cand.intentPrecision.toFixed(3));
check('candidate genuinely beats baseline (no regression)', cand.intentPrecision > base.intentPrecision, `${base.intentPrecision.toFixed(3)} -> ${cand.intentPrecision.toFixed(3)}`);

// 3. Re-drive the real evolution pipeline; the decision must reproduce.
const { sim, promoted, decision, decisionHash } = driveFlywheel(base, cand);
check('divergence score reproduces', approx(sim.divergenceScore, receipt.simulation.divergenceScore), sim.divergenceScore.toFixed(4));
check('divergence within the stated drift limit', sim.divergenceScore <= receipt.decision.maxDivergence, `${sim.divergenceScore.toFixed(4)} <= ${receipt.decision.maxDivergence}`);
check('gate verdict reproduces (approved)', decision.approved === receipt.decision.approved && decision.approved === true);
check('promotion reproduces', promoted === receipt.decision.promoted && promoted === true);
check('rollout completed through all stages', decision.rolloutStatus === 'completed');
check('decision hash reproduces (bit-for-bit)', decisionHash === receipt.decisionHash);

// 4. Prove the sealed proposal is an authentic, untampered signed object.
const recomputedSig = proposalSignature(receipt.proposal);
check('sealed proposal signature is authentic (HMAC recomputes)', recomputedSig === receipt.proposal.signature);
check('sealed proposal was signed & promoted', receipt.proposal.status === 'promoted');

console.log(`\n  re-executed: precision ${base.intentPrecision.toFixed(3)} -> ${cand.intentPrecision.toFixed(3)}, drift ${sim.divergenceScore.toFixed(4)}, promoted=${promoted}.`);
console.log(`  the ONLY non-reproduced fields are proposalId + createdAt (UUID/clock); the gate decision is fully reproduced.`);
console.log(`\n${failures === 0 ? 'VERIFIED' : 'FAILED'}: ruflo guidance flywheel, ${failures} check(s) failed.`);
process.exit(failures === 0 ? 0 : 1);
