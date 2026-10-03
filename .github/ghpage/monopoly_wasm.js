/* @ts-self-types="./monopoly_wasm.d.ts" */

export class WasmBoard {
    static __wrap(ptr) {
        const obj = Object.create(WasmBoard.prototype);
        obj.__wbg_ptr = ptr;
        WasmBoardFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmBoardFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmboard_free(ptr, 0);
    }
    /**
     * Get arrival reason descriptions
     * @returns {string[]}
     */
    get_arrival_reason_descs() {
        const ret = wasm.wasmboard_get_arrival_reason_descs(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]);
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Get spaces
     * @returns {string[]}
     */
    get_spaces() {
        const ret = wasm.wasmboard_get_spaces(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]);
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Get number of turns performed
     * @returns {bigint}
     */
    get_turns() {
        const ret = wasm.wasmboard_get_turns(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * Run the game
     * @param {number} ticks
     * @returns {WasmStats}
     */
    run(ticks) {
        const ret = wasm.wasmboard_run(this.__wbg_ptr, ticks);
        return WasmStats.__wrap(ret);
    }
}
if (Symbol.dispose) WasmBoard.prototype[Symbol.dispose] = WasmBoard.prototype.free;

export class WasmFreq {
    static __wrap(ptr) {
        const obj = Object.create(WasmFreq.prototype);
        obj.__wbg_ptr = ptr;
        WasmFreqFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmFreqFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmfreq_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get movereason_prob_stride() {
        const ret = wasm.__wbg_get_wasmfreq_movereason_prob_stride(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Float64Array}
     */
    get movereason_prob() {
        const ret = wasm.__wbg_get_wasmfreq_movereason_prob(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    get space_prob() {
        const ret = wasm.__wbg_get_wasmfreq_space_prob(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @param {number} arg0
     */
    set movereason_prob_stride(arg0) {
        wasm.__wbg_set_wasmfreq_movereason_prob_stride(this.__wbg_ptr, arg0);
    }
    /**
     * @param {Float64Array} arg0
     */
    set movereason_prob(arg0) {
        const ptr0 = passArrayF64ToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_wasmfreq_movereason_prob(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {Float64Array} arg0
     */
    set space_prob(arg0) {
        const ptr0 = passArrayF64ToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_wasmfreq_space_prob(this.__wbg_ptr, ptr0, len0);
    }
}
if (Symbol.dispose) WasmFreq.prototype[Symbol.dispose] = WasmFreq.prototype.free;

export class WasmStats {
    static __wrap(ptr) {
        const obj = Object.create(WasmStats.prototype);
        obj.__wbg_ptr = ptr;
        WasmStatsFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmStatsFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmstats_free(ptr, 0);
    }
    /**
     * @returns {BigUint64Array}
     */
    get arrivals() {
        const ret = wasm.__wbg_get_wasmstats_arrivals(this.__wbg_ptr);
        var v1 = getArrayU64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {BigUint64Array}
     */
    get doubles() {
        const ret = wasm.__wbg_get_wasmstats_doubles(this.__wbg_ptr);
        var v1 = getArrayU64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {boolean}
     */
    get jailwait() {
        const ret = wasm.__wbg_get_wasmstats_jailwait(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {bigint}
     */
    get moves() {
        const ret = wasm.__wbg_get_wasmstats_moves(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @returns {number}
     */
    get reasons_stride() {
        const ret = wasm.__wbg_get_wasmstats_reasons_stride(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {BigUint64Array}
     */
    get reasons() {
        const ret = wasm.__wbg_get_wasmstats_reasons(this.__wbg_ptr);
        var v1 = getArrayU64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {BigUint64Array}
     */
    get rollfreq() {
        const ret = wasm.__wbg_get_wasmstats_rollfreq(this.__wbg_ptr);
        var v1 = getArrayU64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {bigint}
     */
    get turns() {
        const ret = wasm.__wbg_get_wasmstats_turns(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @param {BigUint64Array} arg0
     */
    set arrivals(arg0) {
        const ptr0 = passArray64ToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_wasmstats_arrivals(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {BigUint64Array} arg0
     */
    set doubles(arg0) {
        const ptr0 = passArray64ToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_wasmstats_doubles(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {boolean} arg0
     */
    set jailwait(arg0) {
        wasm.__wbg_set_wasmstats_jailwait(this.__wbg_ptr, arg0);
    }
    /**
     * @param {bigint} arg0
     */
    set moves(arg0) {
        wasm.__wbg_set_wasmstats_moves(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set reasons_stride(arg0) {
        wasm.__wbg_set_wasmstats_reasons_stride(this.__wbg_ptr, arg0);
    }
    /**
     * @param {BigUint64Array} arg0
     */
    set reasons(arg0) {
        const ptr0 = passArray64ToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_wasmstats_reasons(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {BigUint64Array} arg0
     */
    set rollfreq(arg0) {
        const ptr0 = passArray64ToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_wasmstats_rollfreq(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {bigint} arg0
     */
    set turns(arg0) {
        wasm.__wbg_set_wasmstats_turns(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) WasmStats.prototype[Symbol.dispose] = WasmStats.prototype.free;

/**
 * @param {boolean} jailwait
 * @returns {WasmBoard}
 */
export function create_board(jailwait) {
    const ret = wasm.create_board(jailwait);
    return WasmBoard.__wrap(ret);
}

/**
 * @param {boolean} jailwait
 * @returns {WasmFreq}
 */
export function get_expected_frequencies(jailwait) {
    const ret = wasm.get_expected_frequencies(jailwait);
    return WasmFreq.__wrap(ret);
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_41e9ee4f547fc59a: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_getRandomValues_436a51d0629d84e1: function() { return handleError(function (arg0, arg1) {
            globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
        }, arguments); },
        __wbindgen_generic_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./monopoly_wasm_bg.js": import0,
    };
}

const WasmBoardFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmboard_free(ptr, 1));
const WasmFreqFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmfreq_free(ptr, 1));
const WasmStatsFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmstats_free(ptr, 1));

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

function getArrayF64FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat64ArrayMemory0().subarray(ptr / 8, ptr / 8 + len);
}

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(wasm.__wbindgen_externrefs.get(mem.getUint32(i, true)));
    }
    wasm.__externref_drop_slice(ptr, len);
    return result;
}

function getArrayU64FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getBigUint64ArrayMemory0().subarray(ptr / 8, ptr / 8 + len);
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedBigUint64ArrayMemory0 = null;
function getBigUint64ArrayMemory0() {
    if (cachedBigUint64ArrayMemory0 === null || cachedBigUint64ArrayMemory0.byteLength === 0) {
        cachedBigUint64ArrayMemory0 = new BigUint64Array(wasm.memory.buffer);
    }
    return cachedBigUint64ArrayMemory0;
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let cachedFloat64ArrayMemory0 = null;
function getFloat64ArrayMemory0() {
    if (cachedFloat64ArrayMemory0 === null || cachedFloat64ArrayMemory0.byteLength === 0) {
        cachedFloat64ArrayMemory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachedFloat64ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function passArray64ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 8, 8) >>> 0;
    getBigUint64ArrayMemory0().set(arg, ptr / 8);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passArrayF64ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 8, 8) >>> 0;
    getFloat64ArrayMemory0().set(arg, ptr / 8);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
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

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedBigUint64ArrayMemory0 = null;
    cachedDataViewMemory0 = null;
    cachedFloat64ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (!module.ok) {
            throw new Error(`failed to fetch Wasm: ${module.status} ${module.statusText} fetching '${module.url}'`);
        }

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
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

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('monopoly_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
