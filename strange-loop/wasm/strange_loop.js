
let imports = {};
imports['__wbindgen_placeholder__'] = module.exports;
let wasm;
const { TextDecoder } = require(`util`);

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
 * @param {number} iterations
 * @returns {number}
 */
module.exports.evolve_consciousness = function(iterations) {
    const ret = wasm.evolve_consciousness(iterations);
    return ret;
};

module.exports.__wbg_wbindgenthrow_4c11a24fca429ccf = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

module.exports.__wbindgen_init_externref_table = function() {
    const table = wasm.__wbindgen_export_0;
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

