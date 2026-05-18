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
export declare const DEFAULT_VECTOR_DIR: string;
export declare class SafePathError extends Error {
    readonly code: string;
    constructor(message: string, code?: string);
}
export declare function assertSafeBasename(name: unknown): asserts name is string;
export declare function resolveVectorPath(filename: unknown, options?: {
    stateDir?: string;
}): string;
export declare function openSafeWriteFd(absPath: string): number;
export declare function openSafeReadFd(absPath: string): number;
export declare function safeWriteVector(filename: unknown, data: string | Buffer, options?: {
    stateDir?: string;
}): string;
export declare function safeReadVector(filename: unknown, options?: {
    stateDir?: string;
}): string;
