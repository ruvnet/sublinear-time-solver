/**
 * Proof suite for the 247-zeptosecond photon transit measurement
 * (Grundmann et al., Science 370, 339, 2020).
 *
 * Run: node --test validation/zeptosecond/
 */

import { test, describe } from 'node:test';
import assert from 'node:assert/strict';

import {
  CONSTANTS,
  H2,
  EXPERIMENT,
  ZS_PER_SECOND,
  ZeptoClock,
  photonTransitTime,
  distanceFromTransitTime,
  birthTimeDelay,
  electronDeBroglieWavelength,
  margolusLevitinMinTime,
  minEnergyBandwidth,
  expectedMeanAbsDelay,
  monteCarloMeanDelayNaive,
  monteCarloMeanDelayOptimized,
  simulateEventVectors,
  secondsToZs,
  makeLcg,
} from './zeptosecond-physics.mjs';

describe('physical constants', () => {
  test('speed of light and Planck constant are the exact SI values', () => {
    assert.equal(CONSTANTS.C, 299792458);
    assert.equal(CONSTANTS.PLANCK_H, 6.62607015e-34);
    assert.equal(CONSTANTS.EV, 1.602176634e-19);
  });

  test('H2 bond length is the accepted 74.14 pm equilibrium distance', () => {
    assert.ok(Math.abs(H2.BOND_LENGTH_M - 74.14e-12) < 1e-15);
  });
});

describe('the 247 zs proof: t = R / c', () => {
  test('predicted transit time matches the measured 247 zs within 0.5%', () => {
    const predictedZs = secondsToZs(photonTransitTime());
    const measuredZs = secondsToZs(EXPERIMENT.MEASURED_TRANSIT_S);
    const relError = Math.abs(predictedZs - measuredZs) / measuredZs;
    // R/c = 74.14pm / c = 247.30 zs vs measured 247 zs -> ~0.12% agreement
    assert.ok(
      relError < 0.005,
      `predicted ${predictedZs.toFixed(2)} zs vs measured ${measuredZs} zs (rel err ${(relError * 100).toFixed(3)}%)`,
    );
  });

  test('inverse check: 247 zs of light travel spans the H2 bond within 0.5%', () => {
    const impliedBondPm = distanceFromTransitTime(EXPERIMENT.MEASURED_TRANSIT_S) * 1e12;
    const actualBondPm = H2.BOND_LENGTH_M * 1e12;
    const relError = Math.abs(impliedBondPm - actualBondPm) / actualBondPm;
    assert.ok(
      relError < 0.005,
      `c * 247zs = ${impliedBondPm.toFixed(2)} pm vs bond ${actualBondPm} pm`,
    );
  });

  test('a zeptosecond is 10^-21 s: one nanosecond spans 10^12 zeptoseconds', () => {
    // Exact in BigInt — the float route (1e-9 / 1e-21) already rounds to
    // 1000000000000.0001, which is precisely why ZeptoClock exists.
    const zsPerNanosecond = ZS_PER_SECOND / 10n ** 9n;
    assert.equal(zsPerNanosecond, 10n ** 12n);
  });
});

describe('orientation-dependent birth-time delay', () => {
  test('delay is maximal (full transit time) for an aligned molecule', () => {
    assert.ok(Math.abs(birthTimeDelay(0) - photonTransitTime()) < 1e-30);
  });

  test('delay vanishes for a perpendicular molecule', () => {
    assert.ok(Math.abs(birthTimeDelay(Math.PI / 2)) < 1e-30);
  });

  test('delay is antisymmetric under axis reversal', () => {
    const t = 0.7;
    assert.ok(Math.abs(birthTimeDelay(t) + birthTimeDelay(Math.PI - t)) < 1e-30);
  });
});

describe('why X-rays were required', () => {
  test('photoelectron de Broglie wavelength resolves the two centers', () => {
    const keV = EXPERIMENT.PHOTON_ENERGY_EV - H2.IONIZATION_EV; // ~784.6 eV electron
    const lambda = electronDeBroglieWavelength(keV);
    // Two-center interference needs lambda comparable to the slit spacing
    assert.ok(
      lambda < H2.BOND_LENGTH_M,
      `lambda_dB = ${(lambda * 1e12).toFixed(1)} pm must be < bond ${H2.BOND_LENGTH_M * 1e12} pm`,
    );
  });

  test('directly resolving 247 zs demands keV-scale energy bandwidth', () => {
    const dE = minEnergyBandwidth(EXPERIMENT.MEASURED_TRANSIT_S);
    const dEeV = dE / CONSTANTS.EV;
    // hbar / (2 * 247zs) ~= 1.33 keV: firmly in the X-ray regime.
    assert.ok(dEeV > 1000 && dEeV < 2000, `required bandwidth ${dEeV.toFixed(0)} eV`);
  });

  test('Margolus-Levitin: an 800 eV system cannot orthogonalize in 247 zs', () => {
    // Cross-check against the repo's quantum speed-limit framework
    // (src/temporal_nexus/quantum/speed_limits.rs): tau_min = h / (4E).
    const tauMin = margolusLevitinMinTime(EXPERIMENT.PHOTON_ENERGY_EV * CONSTANTS.EV);
    const tauMinZs = secondsToZs(tauMin);
    assert.ok(tauMinZs > 1290 && tauMinZs < 1295, `tau_min = ${tauMinZs.toFixed(1)} zs`);
    // The 247 zs value is a light-propagation delay read out interferometrically,
    // NOT a state orthogonalization — so it must sit below tau_min without
    // violating anything. Both facts hold:
    assert.ok(secondsToZs(EXPERIMENT.MEASURED_TRANSIT_S) < tauMinZs);
  });
});

describe('exact zeptosecond arithmetic', () => {
  test('IEEE-754 doubles silently drop 247 zs against a 1 s epoch', () => {
    const epoch = 1.0;
    const later = epoch + 247e-21;
    assert.equal(later, epoch, 'float addition must lose the interval (the bug being proven)');
  });

  test('ZeptoClock keeps the interval exactly', () => {
    const clock = new ZeptoClock();
    clock.advanceSeconds(1).advanceZs(247n);
    assert.equal(clock.zeptoseconds, ZS_PER_SECOND + 247n);
    assert.equal(clock.zeptoseconds - ZS_PER_SECOND, 247n);
  });

  test('a million stacked transit events accumulate without loss', () => {
    const clock = new ZeptoClock();
    for (let i = 0; i < 1_000_000; i++) clock.advanceZs(247n);
    assert.equal(clock.zeptoseconds, 247_000_000n);
  });

  test('clock rejects negative time', () => {
    assert.throws(() => new ZeptoClock().advanceZs(-1n), RangeError);
    assert.throws(() => new ZeptoClock().advanceSeconds(-1), RangeError);
  });
});

describe('Monte Carlo estimators', () => {
  const N = 200_000;

  test('naive estimator converges to R/(2c) ~= 123.65 zs', () => {
    const mean = monteCarloMeanDelayNaive(N, makeLcg(1));
    const expected = expectedMeanAbsDelay();
    const relError = Math.abs(mean - expected) / expected;
    assert.ok(relError < 0.01, `mean ${secondsToZs(mean).toFixed(2)} zs, rel err ${(relError * 100).toFixed(2)}%`);
  });

  test('optimized estimator converges to the same value', () => {
    const mean = monteCarloMeanDelayOptimized(N, makeLcg(2));
    const expected = expectedMeanAbsDelay();
    const relError = Math.abs(mean - expected) / expected;
    assert.ok(relError < 0.01, `mean ${secondsToZs(mean).toFixed(2)} zs, rel err ${(relError * 100).toFixed(2)}%`);
  });

  test('optimization preserves physics: identical estimate under identical randomness', () => {
    // Same seed, same statistic — the fast path may not change the answer.
    const naive = monteCarloMeanDelayNaive(50_000, makeLcg(7));
    const optimized = monteCarloMeanDelayOptimized(50_000, makeLcg(7));
    const relDiff = Math.abs(naive - optimized) / naive;
    assert.ok(relDiff < 1e-9, `naive ${naive} vs optimized ${optimized}`);
  });

  test('event vector generator produces normalized, well-formed features', () => {
    const events = simulateEventVectors(1000, makeLcg(3));
    assert.equal(events.length, 1000);
    for (const e of events) {
      assert.equal(e.vector.length, 3);
      assert.ok(e.vector[0] >= -1 && e.vector[0] <= 1);
      assert.ok(e.vector[1] >= 0 && e.vector[1] <= 1);
      assert.ok(e.normalizedDelay >= 0 && e.normalizedDelay <= 1 + 1e-12);
      assert.ok(e.delayZs >= 0 && e.delayZs <= 247.5);
    }
  });
});
