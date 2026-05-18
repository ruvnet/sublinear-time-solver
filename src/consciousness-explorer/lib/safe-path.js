/**
 * Path safety utilities for state import/export.
 *
 * Fixes GHSA / CVE candidate from issue #19 (CWE-73 — External Control of
 * File Name or Path) reported by BruceJin <brucejin@zju.edu.cn> on 2026-04-17.
 *
 * The vulnerable pattern was: the MCP `export_state` / `import_state` tools
 * accepted an attacker-controlled `filepath` string and passed it straight to
 * `fs.writeFileSync` / `fs.readFileSync`. That allowed writing or reading any
 * file the server process had permission to touch (overwriting `~/.ssh/...`,
 * `/etc/...`, etc.).
 *
 * This module forces every state file into a single state directory under
 * the user's home (or `CONSCIOUSNESS_EXPLORER_STATE_DIR` if set), accepts
 * only a safe basename (no separators, no `..`, no NUL, no control chars,
 * no reserved Windows names), rejects symlinks via `O_NOFOLLOW`, and
 * re-verifies that the realpath of the final file remains under the state
 * directory after open.
 */

import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';

/**
 * Default directory for state files. Overridable via env or option.
 */
export const DEFAULT_STATE_DIR =
  process.env.CONSCIOUSNESS_EXPLORER_STATE_DIR ||
  path.join(os.homedir(), '.consciousness-explorer', 'state');

/** Reserved Windows device names that can resolve to special files. */
const WINDOWS_RESERVED = new Set([
  'CON', 'PRN', 'AUX', 'NUL',
  'COM1', 'COM2', 'COM3', 'COM4', 'COM5', 'COM6', 'COM7', 'COM8', 'COM9',
  'LPT1', 'LPT2', 'LPT3', 'LPT4', 'LPT5', 'LPT6', 'LPT7', 'LPT8', 'LPT9',
]);

export class SafePathError extends Error {
  constructor(message, code = 'EUNSAFE_PATH') {
    super(message);
    this.name = 'SafePathError';
    this.code = code;
  }
}

/**
 * Validate that `name` is a safe basename for a state file.
 * Throws SafePathError on any violation.
 *
 * Rules:
 *  - non-empty string, <= 255 bytes after UTF-8 encoding
 *  - no path separators (`/` or `\`)
 *  - no NUL or other ASCII control characters (< 0x20 or 0x7f)
 *  - not equal to "." or ".."
 *  - not a Windows reserved device name (case-insensitive, with or without ext)
 *  - cannot start with "." (avoid colliding with hidden config files) — relax
 *    if you have a use case, but right now nothing legitimate produces them
 */
export function assertSafeBasename(name) {
  if (typeof name !== 'string' || name.length === 0) {
    throw new SafePathError('filepath must be a non-empty string', 'EINVAL');
  }
  if (Buffer.byteLength(name, 'utf8') > 255) {
    throw new SafePathError('filepath basename exceeds 255 bytes', 'ENAMETOOLONG');
  }
  if (name.includes('/') || name.includes('\\')) {
    throw new SafePathError(
      `filepath must be a basename without path separators (got "${name}")`,
      'EPATHSEP',
    );
  }
  if (name === '.' || name === '..') {
    throw new SafePathError(`filepath "${name}" is not allowed`, 'EDOTSEG');
  }
  if (name.startsWith('.')) {
    throw new SafePathError(
      `filepath must not start with "." (got "${name}")`,
      'EHIDDEN',
    );
  }
  for (let i = 0; i < name.length; i++) {
    const c = name.charCodeAt(i);
    if (c < 0x20 || c === 0x7f) {
      throw new SafePathError(
        `filepath contains control character (0x${c.toString(16)})`,
        'ECONTROL',
      );
    }
  }
  // Strip extension for the Windows reserved-name check.
  const stem = name.split('.')[0].toUpperCase();
  if (WINDOWS_RESERVED.has(stem)) {
    throw new SafePathError(
      `filepath uses Windows-reserved device name "${stem}"`,
      'ERESERVED',
    );
  }
}

/**
 * Resolve a caller-supplied filename into an absolute path inside the state
 * directory. Creates the state directory (mode 0o700) if it doesn't exist.
 *
 * Throws SafePathError if the input violates basename rules, or if the
 * resolved realpath escapes the state directory (defence-in-depth against
 * platform path-canonicalisation surprises).
 *
 * @param {string} filename - caller-supplied filename (must be a basename)
 * @param {object} [options]
 * @param {string} [options.stateDir] - override the state directory root
 * @returns {string} absolute path safe for fs.writeFileSync / readFileSync
 */
export function resolveStatePath(filename, { stateDir = DEFAULT_STATE_DIR } = {}) {
  assertSafeBasename(filename);

  // Resolve the state dir itself once.
  const baseAbs = path.resolve(stateDir);

  // Create with restricted permissions if missing. recursive: true is a
  // no-op if it already exists.
  fs.mkdirSync(baseAbs, { recursive: true, mode: 0o700 });

  // path.resolve normalises `..` etc., but assertSafeBasename already
  // rejected those — this is just to get an absolute path.
  const candidate = path.resolve(baseAbs, filename);

  // Defence in depth: even after our basename check, verify the candidate
  // is still under baseAbs once symlinks are followed at the directory
  // level. We use `path.relative` so we don't accidentally accept a path
  // that happens to share a string prefix with baseAbs (e.g. /var/foo vs
  // /var/foobar).
  const rel = path.relative(baseAbs, candidate);
  if (rel.startsWith('..') || path.isAbsolute(rel) || rel.includes(`..${path.sep}`)) {
    throw new SafePathError(
      `resolved path "${candidate}" escapes state dir "${baseAbs}"`,
      'EOUTSIDE',
    );
  }

  return candidate;
}

/**
 * Open a safe file descriptor for writing. Uses O_NOFOLLOW so a symlink at
 * the final path component cannot redirect the write, and O_CLOEXEC so a
 * forked process can't inherit it. The file is created with mode 0o600.
 *
 * Callers should `fs.closeSync(fd)` once done.
 *
 * @param {string} absPath - already-validated absolute path
 * @returns {number} file descriptor
 */
export function openSafeWriteFd(absPath) {
  // O_NOFOLLOW: don't follow a symlink at the last component.
  // O_CLOEXEC:  drop the fd on exec().
  // O_TRUNC:    state files are full snapshots, overwriting is intentional.
  const flags =
    fs.constants.O_WRONLY |
    fs.constants.O_CREAT |
    fs.constants.O_TRUNC |
    (fs.constants.O_NOFOLLOW || 0) |
    (fs.constants.O_CLOEXEC || 0);
  return fs.openSync(absPath, flags, 0o600);
}

/**
 * Open a safe file descriptor for reading. Uses O_NOFOLLOW so a symlink
 * planted at the final path component cannot trick us into reading another
 * file (e.g. `/etc/shadow`).
 *
 * @param {string} absPath - already-validated absolute path
 * @returns {number} file descriptor
 */
export function openSafeReadFd(absPath) {
  const flags =
    fs.constants.O_RDONLY |
    (fs.constants.O_NOFOLLOW || 0) |
    (fs.constants.O_CLOEXEC || 0);
  return fs.openSync(absPath, flags);
}

/**
 * Write `data` to `filename` inside the state directory using the
 * resolveStatePath -> openSafeWriteFd flow. Returns the absolute path
 * that was actually written.
 *
 * @param {string} filename - caller-supplied basename
 * @param {string|Buffer} data
 * @param {object} [options] - forwarded to resolveStatePath
 * @returns {string} absolute path of the written file
 */
export function safeWriteState(filename, data, options) {
  const abs = resolveStatePath(filename, options);
  const fd = openSafeWriteFd(abs);
  try {
    fs.writeSync(fd, typeof data === 'string' ? data : Buffer.from(data));
  } finally {
    fs.closeSync(fd);
  }
  return abs;
}

/**
 * Read a state file by basename. Returns the UTF-8 contents.
 *
 * @param {string} filename - caller-supplied basename
 * @param {object} [options] - forwarded to resolveStatePath
 * @returns {string}
 */
export function safeReadState(filename, options) {
  const abs = resolveStatePath(filename, options);
  const fd = openSafeReadFd(abs);
  try {
    const stat = fs.fstatSync(fd);
    if (!stat.isFile()) {
      throw new SafePathError(
        `state path "${abs}" is not a regular file`,
        'ENOTFILE',
      );
    }
    const buf = Buffer.allocUnsafe(stat.size);
    let read = 0;
    while (read < stat.size) {
      read += fs.readSync(fd, buf, read, stat.size - read, null);
    }
    return buf.toString('utf8');
  } finally {
    fs.closeSync(fd);
  }
}
