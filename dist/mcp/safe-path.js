/**
 * Path safety utilities for sublinear-time-solver MCP file I/O.
 *
 * Companion to `src/consciousness-explorer/lib/safe-path.js`. Same contract,
 * separate state directory so generated test vectors and consciousness state
 * snapshots don't share a namespace.
 *
 * Closes the same CWE-73 class of bug reported in issue #19 for the
 * `saveVectorToFile` / `solveTrueSublinear { vector_file }` flow in the main
 * MCP server: a caller-controlled file_path landed in
 * `fs.writeFileSync(path.resolve(filePath), …)` / `fs.readFileSync` with no
 * containment, so the MCP process could be coerced into overwriting or
 * reading any file it had permission to touch.
 */
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as os from 'node:os';
export const DEFAULT_VECTOR_DIR = process.env.SUBLINEAR_SOLVER_VECTOR_DIR ||
    path.join(os.homedir(), '.sublinear-time-solver', 'vectors');
const WINDOWS_RESERVED = new Set([
    'CON', 'PRN', 'AUX', 'NUL',
    'COM1', 'COM2', 'COM3', 'COM4', 'COM5', 'COM6', 'COM7', 'COM8', 'COM9',
    'LPT1', 'LPT2', 'LPT3', 'LPT4', 'LPT5', 'LPT6', 'LPT7', 'LPT8', 'LPT9',
]);
export class SafePathError extends Error {
    code;
    constructor(message, code = 'EUNSAFE_PATH') {
        super(message);
        this.name = 'SafePathError';
        this.code = code;
    }
}
export function assertSafeBasename(name) {
    if (typeof name !== 'string' || name.length === 0) {
        throw new SafePathError('file_path must be a non-empty string', 'EINVAL');
    }
    if (Buffer.byteLength(name, 'utf8') > 255) {
        throw new SafePathError('file_path basename exceeds 255 bytes', 'ENAMETOOLONG');
    }
    if (name.includes('/') || name.includes('\\')) {
        throw new SafePathError(`file_path must be a basename without path separators (got "${name}")`, 'EPATHSEP');
    }
    if (name === '.' || name === '..') {
        throw new SafePathError(`file_path "${name}" is not allowed`, 'EDOTSEG');
    }
    if (name.startsWith('.')) {
        throw new SafePathError(`file_path must not start with "." (got "${name}")`, 'EHIDDEN');
    }
    for (let i = 0; i < name.length; i++) {
        const c = name.charCodeAt(i);
        if (c < 0x20 || c === 0x7f) {
            throw new SafePathError(`file_path contains control character (0x${c.toString(16)})`, 'ECONTROL');
        }
    }
    const stem = name.split('.')[0].toUpperCase();
    if (WINDOWS_RESERVED.has(stem)) {
        throw new SafePathError(`file_path uses Windows-reserved device name "${stem}"`, 'ERESERVED');
    }
}
export function resolveVectorPath(filename, options = {}) {
    assertSafeBasename(filename);
    const stateDir = options.stateDir ?? DEFAULT_VECTOR_DIR;
    const baseAbs = path.resolve(stateDir);
    fs.mkdirSync(baseAbs, { recursive: true, mode: 0o700 });
    const candidate = path.resolve(baseAbs, filename);
    const rel = path.relative(baseAbs, candidate);
    if (rel.startsWith('..') || path.isAbsolute(rel) || rel.includes(`..${path.sep}`)) {
        throw new SafePathError(`resolved path "${candidate}" escapes vector dir "${baseAbs}"`, 'EOUTSIDE');
    }
    return candidate;
}
export function openSafeWriteFd(absPath) {
    const flags = fs.constants.O_WRONLY |
        fs.constants.O_CREAT |
        fs.constants.O_TRUNC |
        (fs.constants.O_NOFOLLOW || 0) |
        (fs.constants.O_CLOEXEC || 0);
    return fs.openSync(absPath, flags, 0o600);
}
export function openSafeReadFd(absPath) {
    const flags = fs.constants.O_RDONLY |
        (fs.constants.O_NOFOLLOW || 0) |
        (fs.constants.O_CLOEXEC || 0);
    return fs.openSync(absPath, flags);
}
export function safeWriteVector(filename, data, options) {
    const abs = resolveVectorPath(filename, options);
    const fd = openSafeWriteFd(abs);
    try {
        if (typeof data === 'string') {
            fs.writeSync(fd, data);
        }
        else {
            fs.writeSync(fd, data);
        }
    }
    finally {
        fs.closeSync(fd);
    }
    return abs;
}
export function safeReadVector(filename, options) {
    const abs = resolveVectorPath(filename, options);
    const fd = openSafeReadFd(abs);
    try {
        const stat = fs.fstatSync(fd);
        if (!stat.isFile()) {
            throw new SafePathError(`vector path "${abs}" is not a regular file`, 'ENOTFILE');
        }
        const buf = Buffer.allocUnsafe(stat.size);
        let read = 0;
        while (read < stat.size) {
            read += fs.readSync(fd, buf, read, stat.size - read, null);
        }
        return buf.toString('utf8');
    }
    finally {
        fs.closeSync(fd);
    }
}
