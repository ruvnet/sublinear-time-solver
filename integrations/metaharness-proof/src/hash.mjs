/**
 * Deterministic content hashing for the proof-of-mechanism receipt bundle.
 *
 * Every hash here is recomputable from sealed bytes alone, so a verifier never
 * has to trust the producer's claimed values.
 */
import { createHash } from 'node:crypto';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative, sep } from 'node:path';

export const sha256 = (buf) => createHash('sha256').update(buf).digest('hex');

/** Canonical JSON: object keys sorted recursively, no incidental whitespace. */
export function canonicalize(value) {
  if (Array.isArray(value)) return `[${value.map(canonicalize).join(',')}]`;
  if (value && typeof value === 'object') {
    const keys = Object.keys(value).sort();
    return `{${keys.map((k) => `${JSON.stringify(k)}:${canonicalize(value[k])}`).join(',')}}`;
  }
  return JSON.stringify(value);
}

export const hashJson = (value) => sha256(Buffer.from(canonicalize(value), 'utf8'));

/** Recursively list files under `dir`, relative paths, sorted, POSIX separators. */
function listFiles(dir) {
  const out = [];
  const walk = (d) => {
    for (const name of readdirSync(d).sort()) {
      const p = join(d, name);
      if (statSync(p).isDirectory()) walk(p);
      else out.push(relative(dir, p).split(sep).join('/'));
    }
  };
  walk(dir);
  return out.sort();
}

/**
 * Content manifest of a directory: the sorted list of {path, sha256} for every
 * file, plus a single manifest hash over that list. Two directories with
 * identical file contents produce identical manifest hashes regardless of
 * filesystem timestamps or traversal order.
 */
export function manifestOfDir(dir) {
  const files = listFiles(dir).map((rel) => ({
    path: rel,
    sha256: sha256(readFileSync(join(dir, rel))),
  }));
  return { files, manifestHash: hashJson(files) };
}
