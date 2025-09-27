let wasm;

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_export_2.set(idx, obj);
    return idx;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    }
}

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}
/**
 * Initialize panic hook for better error messages in browser console
 */
export function init_panic_hook() {
    wasm.init_panic_hook();
}

let cachedFloat64ArrayMemory0 = null;

function getFloat64ArrayMemory0() {
    if (cachedFloat64ArrayMemory0 === null || cachedFloat64ArrayMemory0.byteLength === 0) {
        cachedFloat64ArrayMemory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachedFloat64ArrayMemory0;
}

function passArrayF64ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 8, 8) >>> 0;
    getFloat64ArrayMemory0().set(arg, ptr / 8);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_export_2.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

function getArrayF64FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat64ArrayMemory0().subarray(ptr / 8, ptr / 8 + len);
}
/**
 * Utility function to generate test data for demos
 * @param {number} n_points
 * @param {number} dt
 * @returns {Float64Array}
 */
export function generate_lorenz_data(n_points, dt) {
    const ret = wasm.generate_lorenz_data(n_points, dt);
    var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v1;
}

/**
 * Generate Hénon map data
 * @param {number} n_points
 * @returns {Float64Array}
 */
export function generate_henon_data(n_points) {
    const ret = wasm.generate_henon_data(n_points);
    var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v1;
}

/**
 * Version information
 * @returns {string}
 */
export function version() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.version();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

const TemporalAttractorStudioFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_temporalattractorstudio_free(ptr >>> 0, 1));
/**
 * Main WASM interface for Temporal Attractor Studio
 */
export class TemporalAttractorStudio {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TemporalAttractorStudioFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_temporalattractorstudio_free(ptr, 0);
    }
    /**
     * Create a new instance
     */
    constructor() {
        const ret = wasm.temporalattractorstudio_new();
        this.__wbg_ptr = ret >>> 0;
        TemporalAttractorStudioFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Calculate Lyapunov exponent from time series data
     *
     * # Arguments
     * * `data` - Flattened array of time series data (row-major order)
     * * `n_dims` - Number of dimensions per time point
     * * `dt` - Time step between measurements
     * * `k_fit` - Number of points for linear fitting (default 12)
     * * `theiler` - Theiler window to exclude temporal neighbors (default 20)
     * * `max_pairs` - Maximum trajectory pairs to analyze (default 1000)
     * * `min_sep` - Minimum initial separation (default 1e-10)
     * @param {Float64Array} data
     * @param {number} n_dims
     * @param {number} dt
     * @param {number | null} [k_fit]
     * @param {number | null} [theiler]
     * @param {number | null} [max_pairs]
     * @param {number | null} [min_sep]
     * @returns {WasmLyapunovResult}
     */
    calculate_lyapunov(data, n_dims, dt, k_fit, theiler, max_pairs, min_sep) {
        const ptr0 = passArrayF64ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.temporalattractorstudio_calculate_lyapunov(this.__wbg_ptr, ptr0, len0, n_dims, dt, isLikeNone(k_fit) ? 0x100000001 : (k_fit) >>> 0, isLikeNone(theiler) ? 0x100000001 : (theiler) >>> 0, isLikeNone(max_pairs) ? 0x100000001 : (max_pairs) >>> 0, !isLikeNone(min_sep), isLikeNone(min_sep) ? 0 : min_sep);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmLyapunovResult.__wrap(ret[0]);
    }
    /**
     * Perform delay embedding for univariate time series
     *
     * # Arguments
     * * `series` - Univariate time series data
     * * `embedding_dim` - Embedding dimension (typically 3-5)
     * * `tau` - Time delay (typically 1-10)
     * @param {Float64Array} series
     * @param {number} embedding_dim
     * @param {number} tau
     * @returns {Float64Array}
     */
    delay_embedding(series, embedding_dim, tau) {
        const ptr0 = passArrayF64ToWasm0(series, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.temporalattractorstudio_delay_embedding(this.__wbg_ptr, ptr0, len0, embedding_dim, tau);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v2 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v2;
    }
    /**
     * Initialize Echo-State Network for prediction
     *
     * # Arguments
     * * `reservoir_size` - Number of reservoir nodes (100-1000 typical)
     * * `input_dim` - Input dimension
     * * `output_dim` - Output dimension
     * * `spectral_radius` - Spectral radius (< 1 for stability, typically 0.9-0.99)
     * * `connectivity` - Reservoir connectivity (0.1-0.3 typical)
     * * `input_scaling` - Input scaling factor (0.1-1.0 typical)
     * * `leak_rate` - Leak rate for neurons (0.1-1.0 typical)
     * * `ridge_param` - Ridge regression parameter (1e-8 to 1e-4 typical)
     * @param {number} reservoir_size
     * @param {number} input_dim
     * @param {number} output_dim
     * @param {number} spectral_radius
     * @param {number} connectivity
     * @param {number} input_scaling
     * @param {number} leak_rate
     * @param {number} ridge_param
     */
    init_echo_network(reservoir_size, input_dim, output_dim, spectral_radius, connectivity, input_scaling, leak_rate, ridge_param) {
        const ret = wasm.temporalattractorstudio_init_echo_network(this.__wbg_ptr, reservoir_size, input_dim, output_dim, spectral_radius, connectivity, input_scaling, leak_rate, ridge_param);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Train the Echo-State Network
     *
     * # Arguments
     * * `inputs` - Training input data (flattened, row-major)
     * * `targets` - Training target data (flattened, row-major)
     * * `n_samples` - Number of training samples
     * * `input_dim` - Input dimension
     * * `output_dim` - Output dimension
     * @param {Float64Array} inputs
     * @param {Float64Array} targets
     * @param {number} n_samples
     * @param {number} input_dim
     * @param {number} output_dim
     * @returns {number}
     */
    train_echo_network(inputs, targets, n_samples, input_dim, output_dim) {
        const ptr0 = passArrayF64ToWasm0(inputs, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayF64ToWasm0(targets, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.temporalattractorstudio_train_echo_network(this.__wbg_ptr, ptr0, len0, ptr1, len1, n_samples, input_dim, output_dim);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Predict next values using Echo-State Network
     *
     * # Arguments
     * * `input` - Current state vector
     * @param {Float64Array} input
     * @returns {Float64Array}
     */
    predict_next(input) {
        const ptr0 = passArrayF64ToWasm0(input, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.temporalattractorstudio_predict_next(this.__wbg_ptr, ptr0, len0);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v2 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v2;
    }
    /**
     * Multi-step prediction
     *
     * # Arguments
     * * `initial_state` - Starting state vector
     * * `n_steps` - Number of steps to predict
     * @param {Float64Array} initial_state
     * @param {number} n_steps
     * @returns {Float64Array}
     */
    predict_trajectory(initial_state, n_steps) {
        const ptr0 = passArrayF64ToWasm0(initial_state, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.temporalattractorstudio_predict_trajectory(this.__wbg_ptr, ptr0, len0, n_steps);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v2 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v2;
    }
    /**
     * Analyze chaos in real-time streaming data
     *
     * # Arguments
     * * `new_point` - New data point to add
     * * `window_size` - Size of sliding window for analysis
     * * `n_dims` - Number of dimensions
     * @param {Float64Array} new_point
     * @param {number} window_size
     * @param {number} n_dims
     * @returns {WasmLyapunovResult}
     */
    analyze_streaming(new_point, window_size, n_dims) {
        const ptr0 = passArrayF64ToWasm0(new_point, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.temporalattractorstudio_analyze_streaming(this.__wbg_ptr, ptr0, len0, window_size, n_dims);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmLyapunovResult.__wrap(ret[0]);
    }
    /**
     * Get chaos interpretation for a Lyapunov exponent value
     * @param {number} lambda
     * @returns {string}
     */
    interpret_chaos(lambda) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.temporalattractorstudio_interpret_chaos(this.__wbg_ptr, lambda);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Detect regime changes in time series
     *
     * # Arguments
     * * `data` - Time series data (flattened)
     * * `n_dims` - Dimensions per point
     * * `window_size` - Size of analysis window
     * * `stride` - Stride between windows
     * @param {Float64Array} data
     * @param {number} n_dims
     * @param {number} window_size
     * @param {number} stride
     * @returns {Float64Array}
     */
    detect_regime_changes(data, n_dims, window_size, stride) {
        const ptr0 = passArrayF64ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.temporalattractorstudio_detect_regime_changes(this.__wbg_ptr, ptr0, len0, n_dims, window_size, stride);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v2 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v2;
    }
    /**
     * Calculate fractal dimension using box-counting
     * @param {Float64Array} data
     * @param {number} n_dims
     * @returns {number}
     */
    estimate_fractal_dimension(data, n_dims) {
        const ptr0 = passArrayF64ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.temporalattractorstudio_estimate_fractal_dimension(this.__wbg_ptr, ptr0, len0, n_dims);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Get recommended parameters for analysis based on data characteristics
     * @param {number} n_points
     * @param {number} n_dims
     * @param {number} sampling_rate
     * @returns {string}
     */
    recommend_parameters(n_points, n_dims, sampling_rate) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.temporalattractorstudio_recommend_parameters(this.__wbg_ptr, n_points, n_dims, sampling_rate);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) TemporalAttractorStudio.prototype[Symbol.dispose] = TemporalAttractorStudio.prototype.free;

const WasmLyapunovResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmlyapunovresult_free(ptr >>> 0, 1));
/**
 * JavaScript-friendly result for Lyapunov calculation
 */
export class WasmLyapunovResult {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmLyapunovResult.prototype);
        obj.__wbg_ptr = ptr;
        WasmLyapunovResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmLyapunovResultFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmlyapunovresult_free(ptr, 0);
    }
    /**
     * The maximum Lyapunov exponent
     * @returns {number}
     */
    get lambda() {
        const ret = wasm.__wbg_get_wasmlyapunovresult_lambda(this.__wbg_ptr);
        return ret;
    }
    /**
     * The maximum Lyapunov exponent
     * @param {number} arg0
     */
    set lambda(arg0) {
        wasm.__wbg_set_wasmlyapunovresult_lambda(this.__wbg_ptr, arg0);
    }
    /**
     * Number of pairs found within constraints
     * @returns {number}
     */
    get pairs_found() {
        const ret = wasm.__wbg_get_wasmlyapunovresult_pairs_found(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Number of pairs found within constraints
     * @param {number} arg0
     */
    set pairs_found(arg0) {
        wasm.__wbg_set_wasmlyapunovresult_pairs_found(this.__wbg_ptr, arg0);
    }
    /**
     * Total pairs considered
     * @returns {number}
     */
    get pairs_total() {
        const ret = wasm.__wbg_get_wasmlyapunovresult_pairs_total(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Total pairs considered
     * @param {number} arg0
     */
    set pairs_total(arg0) {
        wasm.__wbg_set_wasmlyapunovresult_pairs_total(this.__wbg_ptr, arg0);
    }
    /**
     * Lyapunov time (1/lambda) - predictability horizon
     * @returns {number}
     */
    get lyapunov_time() {
        const ret = wasm.__wbg_get_wasmlyapunovresult_lyapunov_time(this.__wbg_ptr);
        return ret;
    }
    /**
     * Lyapunov time (1/lambda) - predictability horizon
     * @param {number} arg0
     */
    set lyapunov_time(arg0) {
        wasm.__wbg_set_wasmlyapunovresult_lyapunov_time(this.__wbg_ptr, arg0);
    }
    /**
     * Doubling time (ln(2)/lambda) - error doubling period
     * @returns {number}
     */
    get doubling_time() {
        const ret = wasm.__wbg_get_wasmlyapunovresult_doubling_time(this.__wbg_ptr);
        return ret;
    }
    /**
     * Doubling time (ln(2)/lambda) - error doubling period
     * @param {number} arg0
     */
    set doubling_time(arg0) {
        wasm.__wbg_set_wasmlyapunovresult_doubling_time(this.__wbg_ptr, arg0);
    }
    /**
     * Is the system chaotic (lambda > 0)
     * @returns {boolean}
     */
    get is_chaotic() {
        const ret = wasm.__wbg_get_wasmlyapunovresult_is_chaotic(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Is the system chaotic (lambda > 0)
     * @param {boolean} arg0
     */
    set is_chaotic(arg0) {
        wasm.__wbg_set_wasmlyapunovresult_is_chaotic(this.__wbg_ptr, arg0);
    }
    /**
     * Chaos strength interpretation
     * @returns {string}
     */
    get chaos_level() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.__wbg_get_wasmlyapunovresult_chaos_level(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Chaos strength interpretation
     * @param {string} arg0
     */
    set chaos_level(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_wasmlyapunovresult_chaos_level(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Recommended prediction horizon
     * @returns {number}
     */
    get safe_prediction_steps() {
        const ret = wasm.__wbg_get_wasmlyapunovresult_safe_prediction_steps(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Recommended prediction horizon
     * @param {number} arg0
     */
    set safe_prediction_steps(arg0) {
        wasm.__wbg_set_wasmlyapunovresult_safe_prediction_steps(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) WasmLyapunovResult.prototype[Symbol.dispose] = WasmLyapunovResult.prototype.free;

const EXPECTED_RESPONSE_TYPES = new Set(['basic', 'cors', 'default']);

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                const validResponse = module.ok && EXPECTED_RESPONSE_TYPES.has(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg_call_13410aac570ffff7 = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.call(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_call_a5400b25a865cfd8 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.call(arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_crypto_574e78ad8b13b65f = function(arg0) {
        const ret = arg0.crypto;
        return ret;
    };
    imports.wbg.__wbg_error_7534b8e9a36f1ab4 = function(arg0, arg1) {
        let deferred0_0;
        let deferred0_1;
        try {
            deferred0_0 = arg0;
            deferred0_1 = arg1;
            console.error(getStringFromWasm0(arg0, arg1));
        } finally {
            wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
        }
    };
    imports.wbg.__wbg_getRandomValues_b8f5dbd5f3995a9e = function() { return handleError(function (arg0, arg1) {
        arg0.getRandomValues(arg1);
    }, arguments) };
    imports.wbg.__wbg_length_6bb7e81f9d7713e4 = function(arg0) {
        const ret = arg0.length;
        return ret;
    };
    imports.wbg.__wbg_msCrypto_a61aeb35a24c1329 = function(arg0) {
        const ret = arg0.msCrypto;
        return ret;
    };
    imports.wbg.__wbg_new_8a6f238a6ece86ea = function() {
        const ret = new Error();
        return ret;
    };
    imports.wbg.__wbg_newnoargs_254190557c45b4ec = function(arg0, arg1) {
        const ret = new Function(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_newwithlength_a167dcc7aaa3ba77 = function(arg0) {
        const ret = new Uint8Array(arg0 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_node_905d3e251edff8a2 = function(arg0) {
        const ret = arg0.node;
        return ret;
    };
    imports.wbg.__wbg_process_dc0fbacc7c1c06f7 = function(arg0) {
        const ret = arg0.process;
        return ret;
    };
    imports.wbg.__wbg_prototypesetcall_3d4a26c1ed734349 = function(arg0, arg1, arg2) {
        Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
    };
    imports.wbg.__wbg_randomFillSync_ac0988aba3254290 = function() { return handleError(function (arg0, arg1) {
        arg0.randomFillSync(arg1);
    }, arguments) };
    imports.wbg.__wbg_require_60cc747a6bc5215a = function() { return handleError(function () {
        const ret = module.require;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_stack_0ed75d68575b0f3c = function(arg0, arg1) {
        const ret = arg1.stack;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_static_accessor_GLOBAL_8921f820c2ce3f12 = function() {
        const ret = typeof global === 'undefined' ? null : global;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_GLOBAL_THIS_f0a4409105898184 = function() {
        const ret = typeof globalThis === 'undefined' ? null : globalThis;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_SELF_995b214ae681ff99 = function() {
        const ret = typeof self === 'undefined' ? null : self;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_WINDOW_cde3890479c675ea = function() {
        const ret = typeof window === 'undefined' ? null : window;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_subarray_70fd07feefe14294 = function(arg0, arg1, arg2) {
        const ret = arg0.subarray(arg1 >>> 0, arg2 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_versions_c01dfd4722a88165 = function(arg0) {
        const ret = arg0.versions;
        return ret;
    };
    imports.wbg.__wbg_wbindgenisfunction_8cee7dce3725ae74 = function(arg0) {
        const ret = typeof(arg0) === 'function';
        return ret;
    };
    imports.wbg.__wbg_wbindgenisobject_307a53c6bd97fbf8 = function(arg0) {
        const val = arg0;
        const ret = typeof(val) === 'object' && val !== null;
        return ret;
    };
    imports.wbg.__wbg_wbindgenisstring_d4fa939789f003b0 = function(arg0) {
        const ret = typeof(arg0) === 'string';
        return ret;
    };
    imports.wbg.__wbg_wbindgenisundefined_c4b71d073b92f3c5 = function(arg0) {
        const ret = arg0 === undefined;
        return ret;
    };
    imports.wbg.__wbg_wbindgenthrow_451ec1a8469d7eb6 = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_cast_2241b6af4c4b2941 = function(arg0, arg1) {
        // Cast intrinsic for `Ref(String) -> Externref`.
        const ret = getStringFromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_cast_cb9088102bce6b30 = function(arg0, arg1) {
        // Cast intrinsic for `Ref(Slice(U8)) -> NamedExternref("Uint8Array")`.
        const ret = getArrayU8FromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_export_2;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
        ;
    };

    return imports;
}

function __wbg_init_memory(imports, memory) {

}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedFloat64ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();

    __wbg_init_memory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('temporal_attractor_studio_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    __wbg_init_memory(imports);

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;
