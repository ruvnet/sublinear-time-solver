
let imports = {};
imports['__wbindgen_placeholder__'] = module.exports;
let wasm;
const { TextDecoder } = require(`util`);

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

function decodeText(ptr, len) {
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

function isLikeNone(x) {
    return x === undefined || x === null;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(
state => {
    wasm.__wbindgen_export_3.get(state.dtor)(state.a, state.b);
}
);

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_3.get(state.dtor)(a, state.b);
                CLOSURE_DTORS.unregister(state);
            } else {
                state.a = a;
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
}
/**
 * @param {number} max_iterations
 * @param {boolean} enable_quantum
 * @returns {Promise<string>}
 */
module.exports.evolve_consciousness_neural = function(max_iterations, enable_quantum) {
    const ret = wasm.evolve_consciousness_neural(max_iterations, enable_quantum);
    return ret;
};

/**
 * @param {number} qubits
 * @returns {Promise<string>}
 */
module.exports.quantum_create_enhanced = function(qubits) {
    const ret = wasm.quantum_create_enhanced(qubits);
    return ret;
};

/**
 * @param {number} qubits
 * @returns {Promise<string>}
 */
module.exports.quantum_measure_enhanced = function(qubits) {
    const ret = wasm.quantum_measure_enhanced(qubits);
    return ret;
};

/**
 * @param {number} agent_count
 * @param {bigint} duration_ms
 * @returns {Promise<string>}
 */
module.exports.run_enhanced_nano_swarm = function(agent_count, duration_ms) {
    const ret = wasm.run_enhanced_nano_swarm(agent_count, duration_ms);
    return ret;
};

module.exports.init_wasm = function() {
    wasm.init_wasm();
};

/**
 * @returns {string}
 */
module.exports.get_version = function() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.get_version();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * @param {number} agent_count
 * @returns {string}
 */
module.exports.create_nano_swarm = function(agent_count) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.create_nano_swarm(agent_count);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * @param {number} ticks
 * @returns {number}
 */
module.exports.run_swarm_ticks = function(ticks) {
    const ret = wasm.run_swarm_ticks(ticks);
    return ret >>> 0;
};

/**
 * @param {number} qubits
 * @returns {string}
 */
module.exports.quantum_superposition = function(qubits) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.quantum_superposition(qubits);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * @param {number} qubits
 * @returns {string}
 */
module.exports.quantum_superposition_old = function(qubits) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.quantum_superposition_old(qubits);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * @param {number} qubits
 * @returns {number}
 */
module.exports.measure_quantum_state = function(qubits) {
    const ret = wasm.measure_quantum_state(qubits);
    return ret >>> 0;
};

/**
 * @param {number} qubits
 * @returns {number}
 */
module.exports.measure_quantum_state_old = function(qubits) {
    const ret = wasm.measure_quantum_state_old(qubits);
    return ret >>> 0;
};

/**
 * @param {number} max_iterations
 * @param {boolean} enable_quantum
 * @returns {Promise<string>}
 */
module.exports.consciousness_evolve = function(max_iterations, enable_quantum) {
    const ret = wasm.consciousness_evolve(max_iterations, enable_quantum);
    return ret;
};

/**
 * @param {number} agent_count
 * @returns {Promise<string>}
 */
module.exports.nano_swarm_create = function(agent_count) {
    const ret = wasm.nano_swarm_create(agent_count);
    return ret;
};

/**
 * @param {number} duration_ms
 * @returns {Promise<string>}
 */
module.exports.nano_swarm_run = function(duration_ms) {
    const ret = wasm.nano_swarm_run(duration_ms);
    return ret;
};

/**
 * @param {number} qubits
 * @returns {Promise<string>}
 */
module.exports.quantum_container_create = function(qubits) {
    const ret = wasm.quantum_container_create(qubits);
    return ret;
};

/**
 * @param {number} qubits
 * @returns {Promise<string>}
 */
module.exports.quantum_measure = function(qubits) {
    const ret = wasm.quantum_measure(qubits);
    return ret;
};

/**
 * @param {number} history_size
 * @param {bigint} horizon_ns
 * @returns {Promise<string>}
 */
module.exports.temporal_predictor_create = function(history_size, horizon_ns) {
    const ret = wasm.temporal_predictor_create(history_size, horizon_ns);
    return ret;
};

let cachedFloat64ArrayMemory0 = null;

function getFloat64ArrayMemory0() {
    if (cachedFloat64ArrayMemory0 === null || cachedFloat64ArrayMemory0.byteLength === 0) {
        cachedFloat64ArrayMemory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachedFloat64ArrayMemory0;
}

let WASM_VECTOR_LEN = 0;

function passArrayF64ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 8, 8) >>> 0;
    getFloat64ArrayMemory0().set(arg, ptr / 8);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}
/**
 * @param {Float64Array} current_values
 * @param {bigint} horizon_ns
 * @returns {Promise<string>}
 */
module.exports.temporal_predict = function(current_values, horizon_ns) {
    const ptr0 = passArrayF64ToWasm0(current_values, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.temporal_predict(ptr0, len0, horizon_ns);
    return ret;
};

/**
 * @returns {Promise<string>}
 */
module.exports.system_info = function() {
    const ret = wasm.system_info();
    return ret;
};

/**
 * @param {number} agent_count
 * @param {number} duration_ms
 * @returns {Promise<string>}
 */
module.exports.benchmark_run = function(agent_count, duration_ms) {
    const ret = wasm.benchmark_run(agent_count, duration_ms);
    return ret;
};

/**
 * @param {number} iterations
 * @returns {number}
 */
module.exports.evolve_consciousness = function(iterations) {
    const ret = wasm.evolve_consciousness(iterations);
    return ret;
};

/**
 * @param {number} sigma
 * @param {number} rho
 * @param {number} beta
 * @returns {string}
 */
module.exports.create_lorenz_attractor = function(sigma, rho, beta) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.create_lorenz_attractor(sigma, rho, beta);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * @param {number} x
 * @param {number} y
 * @param {number} z
 * @param {number} dt
 * @returns {string}
 */
module.exports.step_attractor = function(x, y, z, dt) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.step_attractor(x, y, z, dt);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * @param {number} size
 * @param {number} tolerance
 * @returns {string}
 */
module.exports.solve_linear_system_sublinear = function(size, tolerance) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.solve_linear_system_sublinear(size, tolerance);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * @param {number} size
 * @param {number} tolerance
 * @returns {string}
 */
module.exports.solve_linear_system_sublinear_old = function(size, tolerance) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.solve_linear_system_sublinear_old(size, tolerance);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * @param {number} nodes
 * @param {number} damping
 * @returns {string}
 */
module.exports.compute_pagerank = function(nodes, damping) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.compute_pagerank(nodes, damping);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * @param {number} horizon
 * @returns {string}
 */
module.exports.create_retrocausal_loop = function(horizon) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.create_retrocausal_loop(horizon);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * @param {number} current_value
 * @param {number} horizon_ms
 * @returns {number}
 */
module.exports.predict_future_state = function(current_value, horizon_ms) {
    const ret = wasm.predict_future_state(current_value, horizon_ms);
    return ret;
};

/**
 * @param {number} constant
 * @returns {string}
 */
module.exports.create_lipschitz_loop = function(constant) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.create_lipschitz_loop(constant);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * @param {number} lipschitz_constant
 * @param {number} iterations
 * @returns {boolean}
 */
module.exports.verify_convergence = function(lipschitz_constant, iterations) {
    const ret = wasm.verify_convergence(lipschitz_constant, iterations);
    return ret !== 0;
};

/**
 * @param {number} elements
 * @param {number} connections
 * @returns {number}
 */
module.exports.calculate_phi = function(elements, connections) {
    const ret = wasm.calculate_phi(elements, connections);
    return ret;
};

/**
 * @param {number} phi
 * @param {number} emergence
 * @param {number} coherence
 * @returns {string}
 */
module.exports.verify_consciousness = function(phi, emergence, coherence) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.verify_consciousness(phi, emergence, coherence);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * @param {number} window_size
 * @returns {string}
 */
module.exports.detect_temporal_patterns = function(window_size) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.detect_temporal_patterns(window_size);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * @param {number} qubits
 * @param {number} classical_bits
 * @returns {string}
 */
module.exports.quantum_classical_hybrid = function(qubits, classical_bits) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.quantum_classical_hybrid(qubits, classical_bits);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * @param {number} learning_rate
 * @returns {string}
 */
module.exports.create_self_modifying_loop = function(learning_rate) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.create_self_modifying_loop(learning_rate);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * @param {number} agent_count
 * @returns {string}
 */
module.exports.benchmark_nano_agents = function(agent_count) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.benchmark_nano_agents(agent_count);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * @returns {string}
 */
module.exports.get_system_info = function() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.get_system_info();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * @param {number} pair_type
 * @returns {string}
 */
module.exports.create_bell_state = function(pair_type) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.create_bell_state(pair_type);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * @param {number} qubits
 * @returns {number}
 */
module.exports.quantum_entanglement_entropy = function(qubits) {
    const ret = wasm.quantum_entanglement_entropy(qubits);
    return ret;
};

/**
 * @param {number} value
 * @returns {string}
 */
module.exports.quantum_gate_teleportation = function(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.quantum_gate_teleportation(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * @param {number} qubits
 * @param {number} temperature_mk
 * @returns {number}
 */
module.exports.quantum_decoherence_time = function(qubits, temperature_mk) {
    const ret = wasm.quantum_decoherence_time(qubits, temperature_mk);
    return ret;
};

/**
 * @param {number} database_size
 * @returns {number}
 */
module.exports.quantum_grover_iterations = function(database_size) {
    const ret = wasm.quantum_grover_iterations(database_size);
    return ret >>> 0;
};

/**
 * @param {number} theta
 * @returns {string}
 */
module.exports.quantum_phase_estimation = function(theta) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.quantum_phase_estimation(theta);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * HONEST quantum simulation - simplified but real
 * @param {number} qubits
 * @returns {string}
 */
module.exports.quantum_simulate_honest = function(qubits) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.quantum_simulate_honest(qubits);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * HONEST quantum measurement with real randomness
 * @param {number} qubits
 * @returns {number}
 */
module.exports.quantum_measure_honest = function(qubits) {
    const ret = wasm.quantum_measure_honest(qubits);
    return ret >>> 0;
};

/**
 * HONEST consciousness metric - acknowledges it's just math
 * @param {number} iterations
 * @returns {string}
 */
module.exports.consciousness_simulate_honest = function(iterations) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.consciousness_simulate_honest(iterations);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * HONEST swarm simulation - single-threaded for WASM
 * @param {number} agents
 * @returns {string}
 */
module.exports.swarm_simulate_honest = function(agents) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.swarm_simulate_honest(agents);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * HONEST solver - actually does simple computation
 * @param {number} size
 * @returns {string}
 */
module.exports.solve_simple_honest = function(size) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.solve_simple_honest(size);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * Get real random number between 0 and 1
 * @returns {number}
 */
module.exports.random_real = function() {
    const ret = wasm.random_real();
    return ret;
};

/**
 * Benchmark honesty check
 * @returns {string}
 */
module.exports.benchmark_honest = function() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.benchmark_honest();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

function __wbg_adapter_8(arg0, arg1, arg2) {
    wasm.closure80_externref_shim(arg0, arg1, arg2);
}

function __wbg_adapter_94(arg0, arg1, arg2, arg3) {
    wasm.closure108_externref_shim(arg0, arg1, arg2, arg3);
}

module.exports.__wbg_call_2f8d426a20a307fe = function() { return handleError(function (arg0, arg1) {
    const ret = arg0.call(arg1);
    return ret;
}, arguments) };

module.exports.__wbg_call_f53f0647ceb9c567 = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = arg0.call(arg1, arg2);
    return ret;
}, arguments) };

module.exports.__wbg_crypto_574e78ad8b13b65f = function(arg0) {
    const ret = arg0.crypto;
    return ret;
};

module.exports.__wbg_getRandomValues_b8f5dbd5f3995a9e = function() { return handleError(function (arg0, arg1) {
    arg0.getRandomValues(arg1);
}, arguments) };

module.exports.__wbg_length_904c0910ed998bf3 = function(arg0) {
    const ret = arg0.length;
    return ret;
};

module.exports.__wbg_msCrypto_a61aeb35a24c1329 = function(arg0) {
    const ret = arg0.msCrypto;
    return ret;
};

module.exports.__wbg_new_d5e3800b120e37e1 = function(arg0, arg1) {
    try {
        var state0 = {a: arg0, b: arg1};
        var cb0 = (arg0, arg1) => {
            const a = state0.a;
            state0.a = 0;
            try {
                return __wbg_adapter_94(a, state0.b, arg0, arg1);
            } finally {
                state0.a = a;
            }
        };
        const ret = new Promise(cb0);
        return ret;
    } finally {
        state0.a = state0.b = 0;
    }
};

module.exports.__wbg_newnoargs_a81330f6e05d8aca = function(arg0, arg1) {
    const ret = new Function(getStringFromWasm0(arg0, arg1));
    return ret;
};

module.exports.__wbg_newwithlength_ed0ee6c1edca86fc = function(arg0) {
    const ret = new Uint8Array(arg0 >>> 0);
    return ret;
};

module.exports.__wbg_node_905d3e251edff8a2 = function(arg0) {
    const ret = arg0.node;
    return ret;
};

module.exports.__wbg_now_e3057dd824ca0191 = function() {
    const ret = Date.now();
    return ret;
};

module.exports.__wbg_process_dc0fbacc7c1c06f7 = function(arg0) {
    const ret = arg0.process;
    return ret;
};

module.exports.__wbg_prototypesetcall_c5f74efd31aea86b = function(arg0, arg1, arg2) {
    Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
};

module.exports.__wbg_queueMicrotask_bcc6e26d899696db = function(arg0) {
    const ret = arg0.queueMicrotask;
    return ret;
};

module.exports.__wbg_queueMicrotask_f24a794d09c42640 = function(arg0) {
    queueMicrotask(arg0);
};

module.exports.__wbg_randomFillSync_ac0988aba3254290 = function() { return handleError(function (arg0, arg1) {
    arg0.randomFillSync(arg1);
}, arguments) };

module.exports.__wbg_random_57255a777f5a0573 = function() {
    const ret = Math.random();
    return ret;
};

module.exports.__wbg_require_60cc747a6bc5215a = function() { return handleError(function () {
    const ret = module.require;
    return ret;
}, arguments) };

module.exports.__wbg_resolve_5775c0ef9222f556 = function(arg0) {
    const ret = Promise.resolve(arg0);
    return ret;
};

module.exports.__wbg_static_accessor_GLOBAL_1f13249cc3acc96d = function() {
    const ret = typeof global === 'undefined' ? null : global;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

module.exports.__wbg_static_accessor_GLOBAL_THIS_df7ae94b1e0ed6a3 = function() {
    const ret = typeof globalThis === 'undefined' ? null : globalThis;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

module.exports.__wbg_static_accessor_SELF_6265471db3b3c228 = function() {
    const ret = typeof self === 'undefined' ? null : self;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

module.exports.__wbg_static_accessor_WINDOW_16fb482f8ec52863 = function() {
    const ret = typeof window === 'undefined' ? null : window;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

module.exports.__wbg_subarray_a219824899e59712 = function(arg0, arg1, arg2) {
    const ret = arg0.subarray(arg1 >>> 0, arg2 >>> 0);
    return ret;
};

module.exports.__wbg_then_9cc266be2bf537b6 = function(arg0, arg1) {
    const ret = arg0.then(arg1);
    return ret;
};

module.exports.__wbg_versions_c01dfd4722a88165 = function(arg0) {
    const ret = arg0.versions;
    return ret;
};

module.exports.__wbg_wbindgencbdrop_a85ed476c6a370b9 = function(arg0) {
    const obj = arg0.original;
    if (obj.cnt-- == 1) {
        obj.a = 0;
        return true;
    }
    const ret = false;
    return ret;
};

module.exports.__wbg_wbindgenisfunction_ea72b9d66a0e1705 = function(arg0) {
    const ret = typeof(arg0) === 'function';
    return ret;
};

module.exports.__wbg_wbindgenisobject_dfe064a121d87553 = function(arg0) {
    const val = arg0;
    const ret = typeof(val) === 'object' && val !== null;
    return ret;
};

module.exports.__wbg_wbindgenisstring_4b74e4111ba029e6 = function(arg0) {
    const ret = typeof(arg0) === 'string';
    return ret;
};

module.exports.__wbg_wbindgenisundefined_71f08a6ade4354e7 = function(arg0) {
    const ret = arg0 === undefined;
    return ret;
};

module.exports.__wbg_wbindgenthrow_4c11a24fca429ccf = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

module.exports.__wbindgen_cast_2241b6af4c4b2941 = function(arg0, arg1) {
    // Cast intrinsic for `Ref(String) -> Externref`.
    const ret = getStringFromWasm0(arg0, arg1);
    return ret;
};

module.exports.__wbindgen_cast_9d7b003571fd2c19 = function(arg0, arg1) {
    // Cast intrinsic for `Closure(Closure { dtor_idx: 79, function: Function { arguments: [Externref], shim_idx: 80, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
    const ret = makeMutClosure(arg0, arg1, 79, __wbg_adapter_8);
    return ret;
};

module.exports.__wbindgen_cast_cb9088102bce6b30 = function(arg0, arg1) {
    // Cast intrinsic for `Ref(Slice(U8)) -> NamedExternref("Uint8Array")`.
    const ret = getArrayU8FromWasm0(arg0, arg1);
    return ret;
};

module.exports.__wbindgen_init_externref_table = function() {
    const table = wasm.__wbindgen_export_2;
    const offset = table.grow(4);
    table.set(0, undefined);
    table.set(offset + 0, undefined);
    table.set(offset + 1, null);
    table.set(offset + 2, true);
    table.set(offset + 3, false);
    ;
};

const path = require('path').join(__dirname, 'strange_loop_bg.wasm');
const bytes = require('fs').readFileSync(path);

const wasmModule = new WebAssembly.Module(bytes);
const wasmInstance = new WebAssembly.Instance(wasmModule, imports);
wasm = wasmInstance.exports;
module.exports.__wasm = wasm;

wasm.__wbindgen_start();

