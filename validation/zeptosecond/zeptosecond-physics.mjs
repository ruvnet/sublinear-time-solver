/**
 * Zeptosecond Photon-Transit Physics — validation library
 *
 * Models the 2020 Goethe University Frankfurt measurement (Grundmann et al.,
 * "Zeptosecond birth time delay in molecular photoionization", Science 370,
 * 339–341, 2020): the ~247 zeptosecond travel time of an X-ray photon across
 * the two atoms of a hydrogen molecule, measured at the PETRA III source
 * (DESY, Hamburg) via two-center photoelectron interference.
 *
 * The library provides:
 *   - CODATA physical constants and the H2 geometry
 *   - the transit-time prediction t = R / c and its inverse
 *   - the orientation-dependent birth-time delay model dt(theta) = R cos(theta) / c
 *   - exact zeptosecond-resolution timekeeping (BigInt), because IEEE-754
 *     doubles silently drop a 247 zs interval added to a 1 s epoch
 *   - Monte Carlo estimators (naive and optimized) over molecular orientations
 *   - quantum-limit cross checks: Margolus–Levitin minimum orthogonalization
 *     time and the energy–time uncertainty bandwidth needed to resolve 247 zs
 */

// ---------------------------------------------------------------------------
// Constants (CODATA 2018 exact/recommended values)
// ---------------------------------------------------------------------------

export const CONSTANTS = Object.freeze({
  /** Speed of light in vacuum, m/s (exact) */
  C: 299792458,
  /** Planck constant, J*s (exact) */
  PLANCK_H: 6.62607015e-34,
  /** Reduced Planck constant, J*s */
  HBAR: 6.62607015e-34 / (2 * Math.PI),
  /** Elementary charge, C (exact) — also J per eV */
  EV: 1.602176634e-19,
  /** Electron rest mass, kg */
  ELECTRON_MASS: 9.1093837015e-31,
});

export const H2 = Object.freeze({
  /** Equilibrium internuclear distance of H2, meters (74.14 pm) */
  BOND_LENGTH_M: 74.14e-12,
  /** Ionization energy of H2, eV */
  IONIZATION_EV: 15.43,
});

export const EXPERIMENT = Object.freeze({
  /** Reported photon transit time across H2, seconds (247 zs) */
  MEASURED_TRANSIT_S: 247e-21,
  /** Photon energy used at PETRA III beamline P04, eV */
  PHOTON_ENERGY_EV: 800,
});

/** Zeptoseconds per second, exact, as BigInt (1e21). */
export const ZS_PER_SECOND = 10n ** 21n;

// ---------------------------------------------------------------------------
// Unit helpers
// ---------------------------------------------------------------------------

export const secondsToZs = (s) => s * 1e21;
export const zsToSeconds = (zs) => zs * 1e-21;

// ---------------------------------------------------------------------------
// Core physics
// ---------------------------------------------------------------------------

/**
 * Time for light to traverse a distance in vacuum, seconds.
 * For the H2 bond this is the headline number: t = R / c ~= 247 zs.
 */
export function photonTransitTime(distanceM = H2.BOND_LENGTH_M) {
  if (!(distanceM > 0)) throw new RangeError(`distance must be > 0, got ${distanceM}`);
  return distanceM / CONSTANTS.C;
}

/** Inverse problem: the distance light covers in a given time, meters. */
export function distanceFromTransitTime(seconds) {
  if (!(seconds > 0)) throw new RangeError(`time must be > 0, got ${seconds}`);
  return seconds * CONSTANTS.C;
}

/**
 * Orientation-dependent birth-time delay between the two emission centers.
 *
 * The photon front reaches the "far" hydrogen atom later than the "near" one
 * by the projection of the bond onto the propagation direction:
 *   dt(theta) = R * cos(theta) / c
 * where theta is the angle between the molecular axis and the photon's
 * propagation direction. At theta = 0 (molecule aligned with the beam) the
 * delay is maximal and equals the full transit time, ~247 zs.
 */
export function birthTimeDelay(thetaRad, bondLengthM = H2.BOND_LENGTH_M) {
  return (bondLengthM * Math.cos(thetaRad)) / CONSTANTS.C;
}

/**
 * De Broglie wavelength of the emitted photoelectron, meters.
 * Two-center interference is only resolvable when this is comparable to or
 * smaller than the internuclear distance — the geometric reason keV-scale
 * X-rays were required.
 */
export function electronDeBroglieWavelength(kineticEnergyEv) {
  if (!(kineticEnergyEv > 0)) throw new RangeError('kinetic energy must be > 0');
  const eJ = kineticEnergyEv * CONSTANTS.EV;
  const p = Math.sqrt(2 * CONSTANTS.ELECTRON_MASS * eJ);
  return CONSTANTS.PLANCK_H / p;
}

// ---------------------------------------------------------------------------
// Quantum limits (mirrors src/temporal_nexus/quantum/speed_limits.rs)
// ---------------------------------------------------------------------------

/**
 * Margolus–Levitin minimum time for a quantum system of mean energy E to
 * evolve to an orthogonal state: tau_min = h / (4 E). Seconds.
 * Same formula as MargolousLevitinValidator::calculate_minimum_time in the
 * Rust crate (with safety_margin = 1.0).
 */
export function margolusLevitinMinTime(energyJ) {
  if (!(energyJ > 0)) return Infinity;
  return CONSTANTS.PLANCK_H / (4 * energyJ);
}

/**
 * Energy–time uncertainty: the minimum energy bandwidth needed to localize an
 * event to deltaT directly, dE >= hbar / (2 dt). Joules.
 * For 247 zs this lands in the keV X-ray regime — which is why the experiment
 * (a) needed an X-ray source and (b) extracted the delay interferometrically
 * from the electron phase rather than by direct chronometry.
 */
export function minEnergyBandwidth(deltaTSeconds) {
  if (!(deltaTSeconds > 0)) return Infinity;
  return CONSTANTS.HBAR / (2 * deltaTSeconds);
}

// ---------------------------------------------------------------------------
// Exact zeptosecond timekeeping
// ---------------------------------------------------------------------------

/**
 * A zeptosecond-resolution clock backed by BigInt.
 *
 * Rationale: IEEE-754 doubles have ~15.95 significant decimal digits, so a
 * 1-second epoch (1e21 zs) cannot represent a 247 zs increment:
 *   1.0 + 247e-21 === 1.0   // true — the interval vanishes
 * BigInt zeptosecond ticks keep every interval exact, extending the repo's
 * nanosecond-scheduler philosophy 12 orders of magnitude further down.
 */
export class ZeptoClock {
  #zs = 0n;

  /** Advance by an integer number of zeptoseconds. */
  advanceZs(zs) {
    const t = typeof zs === 'bigint' ? zs : BigInt(zs);
    if (t < 0n) throw new RangeError('clock cannot go backwards');
    this.#zs += t;
    return this;
  }

  /** Advance by an integer number of seconds, exactly. */
  advanceSeconds(s) {
    if (!Number.isInteger(s) || s < 0) throw new RangeError('seconds must be a non-negative integer');
    this.#zs += BigInt(s) * ZS_PER_SECOND;
    return this;
  }

  /** Total elapsed zeptoseconds, exact. */
  get zeptoseconds() {
    return this.#zs;
  }

  /** Elapsed time in seconds as a (lossy) Number, for display only. */
  get seconds() {
    return Number(this.#zs) / 1e21;
  }
}

// ---------------------------------------------------------------------------
// Monte Carlo estimators over isotropic molecular orientations
// ---------------------------------------------------------------------------
// For molecules oriented isotropically, cos(theta) is uniform on [-1, 1], so
// the expected magnitude of the birth-time delay is:
//   E[|dt|] = (R / c) * E[|cos theta|] = (R / c) / 2  ~= 123.65 zs
// Both estimators below must converge to this; the optimized one exists to
// prove the optimization changes cost, not physics.

export const expectedMeanAbsDelay = (bondLengthM = H2.BOND_LENGTH_M) =>
  photonTransitTime(bondLengthM) / 2;

/**
 * Naive estimator: per-event object allocation, trig sampling, functional
 * reduction. Deliberately idiomatic-but-wasteful baseline.
 */
export function monteCarloMeanDelayNaive(nEvents, rng = Math.random) {
  const events = [];
  for (let i = 0; i < nEvents; i++) {
    const theta = Math.acos(2 * rng() - 1); // isotropic polar angle
    events.push({ theta, delay: Math.abs(birthTimeDelay(theta)) });
  }
  return events.reduce((acc, e) => acc + e.delay, 0) / nEvents;
}

/**
 * Optimized estimator: no per-event allocation, no acos/cos round-trip
 * (cos(theta) is sampled directly as u ~ U[-1,1]), single fused pass,
 * hoisted R/c factor. Returns the same statistic as the naive version.
 */
export function monteCarloMeanDelayOptimized(nEvents, rng = Math.random) {
  const scale = H2.BOND_LENGTH_M / CONSTANTS.C;
  let sum = 0;
  for (let i = 0; i < nEvents; i++) {
    const cosTheta = 2 * rng() - 1;
    sum += cosTheta < 0 ? -cosTheta : cosTheta;
  }
  return (sum / nEvents) * scale;
}

/**
 * Generate a batch of simulated measurement events as flat feature vectors,
 * suitable for indexing in a vector database (ruvector integration).
 * Each event: [cosTheta, sinTheta, normalizedDelay] with the delay normalized
 * by the full transit time so all features live in [-1, 1].
 */
export function simulateEventVectors(nEvents, rng = Math.random) {
  const transit = photonTransitTime();
  const vectors = new Array(nEvents);
  for (let i = 0; i < nEvents; i++) {
    const cosTheta = 2 * rng() - 1;
    const sinTheta = Math.sqrt(1 - cosTheta * cosTheta);
    const delayZs = secondsToZs(Math.abs(birthTimeDelay(Math.acos(cosTheta))));
    vectors[i] = {
      id: i,
      vector: Float32Array.from([cosTheta, sinTheta, Math.abs(cosTheta)]),
      delayZs,
      normalizedDelay: zsToSeconds(delayZs) / transit,
    };
  }
  return vectors;
}

/** Deterministic linear-congruential RNG for reproducible tests/benchmarks. */
export function makeLcg(seed = 42) {
  let state = seed >>> 0;
  return () => {
    state = (state * 1664525 + 1013904223) >>> 0;
    return state / 4294967296;
  };
}
