/**
 * Enforce the numeric constraints a tool's inputSchema already declares
 * (`type: number|integer`, `minimum`, `maximum`) before the handler runs.
 *
 * The schemas advertised bounds such as `iterations <= 10000`, but nothing
 * checked them, so one request like `{"iterations": 1e12}` pinned the event
 * loop or ran the server out of memory (1.8.0 review). Only properties the
 * schema describes are checked; unknown properties are left to the handler.
 */
interface NumericSchema {
    type?: string | string[];
    minimum?: number;
    maximum?: number;
    properties?: Record<string, NumericSchema>;
}
/** Returns an error message, or undefined when `args` satisfies the bounds. */
export declare function checkNumericBounds(schema: NumericSchema | undefined, args: unknown, path?: string, depth?: number): string | undefined;
export {};
