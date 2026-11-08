/**
 * Enforce the numeric constraints a tool's inputSchema already declares
 * (`type: number|integer`, `minimum`, `maximum`) before the handler runs.
 *
 * The schemas advertised bounds such as `iterations <= 10000`, but nothing
 * checked them, so one request like `{"iterations": 1e12}` pinned the event
 * loop or ran the server out of memory (1.8.0 review). Only properties the
 * schema describes are checked; unknown properties are left to the handler.
 */
const MAX_DEPTH = 4;
function isNumericType(t) {
    const types = Array.isArray(t) ? t : t ? [t] : [];
    if (types.includes('integer'))
        return 'integer';
    if (types.includes('number'))
        return 'number';
    return undefined;
}
/** Returns an error message, or undefined when `args` satisfies the bounds. */
export function checkNumericBounds(schema, args, path = '', depth = 0) {
    if (!schema?.properties || !args || typeof args !== 'object' || Array.isArray(args) || depth > MAX_DEPTH)
        return undefined;
    for (const key of Object.keys(schema.properties)) {
        if (!Object.prototype.hasOwnProperty.call(args, key))
            continue;
        const prop = schema.properties[key];
        const value = args[key];
        const where = path ? `${path}.${key}` : key;
        if (value === undefined || value === null)
            continue;
        const numeric = isNumericType(prop.type);
        if (numeric) {
            if (typeof value !== 'number' || !Number.isFinite(value))
                return `${where} must be a finite number`;
            if (numeric === 'integer' && !Number.isInteger(value))
                return `${where} must be an integer`;
            if (typeof prop.minimum === 'number' && value < prop.minimum)
                return `${where} must be >= ${prop.minimum}, got ${value}`;
            if (typeof prop.maximum === 'number' && value > prop.maximum)
                return `${where} must be <= ${prop.maximum}, got ${value}`;
        }
        else if (prop.properties) {
            const nested = checkNumericBounds(prop, value, where, depth + 1);
            if (nested)
                return nested;
        }
    }
    return undefined;
}
