let ftmlViewer;
(function() {
    const __exports = {};
    let script_src;
    if (typeof document !== 'undefined' && document.currentScript !== null) {
        script_src = new URL(document.currentScript.src, location.href).toString();
    }
    let wasm = undefined;

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

    let heap = new Array(128).fill(undefined);

    heap.push(undefined, null, true, false);

    let heap_next = heap.length;

    function addHeapObject(obj) {
        if (heap_next === heap.length) heap.push(heap.length + 1);
        const idx = heap_next;
        heap_next = heap[idx];

        heap[idx] = obj;
        return idx;
    }

    function getObject(idx) { return heap[idx]; }

    function handleError(f, args) {
        try {
            return f.apply(this, args);
        } catch (e) {
            wasm.__wbindgen_export_0(addHeapObject(e));
        }
    }

    function isLikeNone(x) {
        return x === undefined || x === null;
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

    function getArrayU8FromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
    }

    function dropObject(idx) {
        if (idx < 132) return;
        heap[idx] = heap_next;
        heap_next = idx;
    }

    function takeObject(idx) {
        const ret = getObject(idx);
        dropObject(idx);
        return ret;
    }

    function debugString(val) {
        // primitive types
        const type = typeof val;
        if (type == 'number' || type == 'boolean' || val == null) {
            return  `${val}`;
        }
        if (type == 'string') {
            return `"${val}"`;
        }
        if (type == 'symbol') {
            const description = val.description;
            if (description == null) {
                return 'Symbol';
            } else {
                return `Symbol(${description})`;
            }
        }
        if (type == 'function') {
            const name = val.name;
            if (typeof name == 'string' && name.length > 0) {
                return `Function(${name})`;
            } else {
                return 'Function';
            }
        }
        // objects
        if (Array.isArray(val)) {
            const length = val.length;
            let debug = '[';
            if (length > 0) {
                debug += debugString(val[0]);
            }
            for(let i = 1; i < length; i++) {
                debug += ', ' + debugString(val[i]);
            }
            debug += ']';
            return debug;
        }
        // Test for built-in
        const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
        let className;
        if (builtInMatches && builtInMatches.length > 1) {
            className = builtInMatches[1];
        } else {
            // Failed to match the standard '[object ClassName]'
            return toString.call(val);
        }
        if (className == 'Object') {
            // we're a user defined class or Object
            // JSON.stringify avoids problems with cycles, and is generally much
            // easier than looping through ownProperties of `val`.
            try {
                return 'Object(' + JSON.stringify(val) + ')';
            } catch (_) {
                return 'Object';
            }
        }
        // errors
        if (val instanceof Error) {
            return `${val.name}: ${val.message}\n${val.stack}`;
        }
        // TODO we could test for more things here, like `Set`s and `Map`s.
        return className;
    }

    const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(
    state => {
        wasm.__wbindgen_export_4.get(state.dtor)(state.a, state.b);
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
                    wasm.__wbindgen_export_4.get(state.dtor)(a, state.b);
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

    function makeClosure(arg0, arg1, dtor, f) {
        const state = { a: arg0, b: arg1, cnt: 1, dtor };
        const real = (...args) => {

            // First up with a closure we increment the internal reference
            // count. This ensures that the Rust closure environment won't
            // be deallocated while we're invoking it.
            state.cnt++;
            try {
                return f(state.a, state.b, ...args);
            } finally {
                if (--state.cnt === 0) {
                    wasm.__wbindgen_export_4.get(state.dtor)(state.a, state.b); state.a = 0;
                    CLOSURE_DTORS.unregister(state);
                }
            }
        };
        real.original = state;
        CLOSURE_DTORS.register(real, state, state);
        return real;
    }

    __exports.clear_cache = function() {
        wasm.clear_cache();
    };

    __exports.run = function() {
        wasm.run();
    };

    __exports.print_cache = function() {
        wasm.print_cache();
    };

    function __wbg_adapter_6(arg0, arg1) {
        wasm.__wbindgen_export_5(arg0, arg1);
    }

    function __wbg_adapter_11(arg0, arg1, arg2) {
        wasm.__wbindgen_export_6(arg0, arg1, addHeapObject(arg2));
    }

    function __wbg_adapter_14(arg0, arg1, arg2) {
        wasm.__wbindgen_export_7(arg0, arg1, addHeapObject(arg2));
    }

    function __wbg_adapter_17(arg0, arg1, arg2) {
        const ret = wasm.__wbindgen_export_8(arg0, arg1, addHeapObject(arg2));
        return ret >>> 0;
    }

    function __wbg_adapter_20(arg0, arg1, arg2) {
        wasm.__wbindgen_export_9(arg0, arg1, addHeapObject(arg2));
    }

    function __wbg_adapter_23(arg0, arg1, arg2) {
        wasm.__wbindgen_export_10(arg0, arg1, addHeapObject(arg2));
    }

    function __wbg_adapter_28(arg0, arg1) {
        wasm.__wbindgen_export_11(arg0, arg1);
    }

    function __wbg_adapter_33(arg0, arg1, arg2) {
        wasm.__wbindgen_export_12(arg0, arg1, addHeapObject(arg2));
    }

    function __wbg_adapter_417(arg0, arg1, arg2, arg3) {
        wasm.__wbindgen_export_13(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
    }

    /**
     * @enum {0 | 1 | 2 | 3}
     */
    __exports.HighlightStyle = Object.freeze({
        Colored: 0, "0": "Colored",
        Subtle: 1, "1": "Subtle",
        Off: 2, "2": "Off",
        None: 3, "3": "None",
    });

    const __wbindgen_enum_ReadableStreamType = ["bytes"];

    const __wbindgen_enum_ScrollBehavior = ["auto", "instant", "smooth"];

    const __wbindgen_enum_ScrollLogicalPosition = ["start", "center", "end", "nearest"];

    const IntoUnderlyingByteSourceFinalization = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingbytesource_free(ptr >>> 0, 1));

    class IntoUnderlyingByteSource {

        __destroy_into_raw() {
            const ptr = this.__wbg_ptr;
            this.__wbg_ptr = 0;
            IntoUnderlyingByteSourceFinalization.unregister(this);
            return ptr;
        }

        free() {
            const ptr = this.__destroy_into_raw();
            wasm.__wbg_intounderlyingbytesource_free(ptr, 0);
        }
        /**
         * @returns {number}
         */
        get autoAllocateChunkSize() {
            const ret = wasm.intounderlyingbytesource_autoAllocateChunkSize(this.__wbg_ptr);
            return ret >>> 0;
        }
        /**
         * @param {ReadableByteStreamController} controller
         * @returns {Promise<any>}
         */
        pull(controller) {
            const ret = wasm.intounderlyingbytesource_pull(this.__wbg_ptr, addHeapObject(controller));
            return takeObject(ret);
        }
        /**
         * @param {ReadableByteStreamController} controller
         */
        start(controller) {
            wasm.intounderlyingbytesource_start(this.__wbg_ptr, addHeapObject(controller));
        }
        /**
         * @returns {ReadableStreamType}
         */
        get type() {
            const ret = wasm.intounderlyingbytesource_type(this.__wbg_ptr);
            return __wbindgen_enum_ReadableStreamType[ret];
        }
        cancel() {
            const ptr = this.__destroy_into_raw();
            wasm.intounderlyingbytesource_cancel(ptr);
        }
    }
    if (Symbol.dispose) IntoUnderlyingByteSource.prototype[Symbol.dispose] = IntoUnderlyingByteSource.prototype.free;

    __exports.IntoUnderlyingByteSource = IntoUnderlyingByteSource;

    const IntoUnderlyingSinkFinalization = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingsink_free(ptr >>> 0, 1));

    class IntoUnderlyingSink {

        __destroy_into_raw() {
            const ptr = this.__wbg_ptr;
            this.__wbg_ptr = 0;
            IntoUnderlyingSinkFinalization.unregister(this);
            return ptr;
        }

        free() {
            const ptr = this.__destroy_into_raw();
            wasm.__wbg_intounderlyingsink_free(ptr, 0);
        }
        /**
         * @param {any} reason
         * @returns {Promise<any>}
         */
        abort(reason) {
            const ptr = this.__destroy_into_raw();
            const ret = wasm.intounderlyingsink_abort(ptr, addHeapObject(reason));
            return takeObject(ret);
        }
        /**
         * @returns {Promise<any>}
         */
        close() {
            const ptr = this.__destroy_into_raw();
            const ret = wasm.intounderlyingsink_close(ptr);
            return takeObject(ret);
        }
        /**
         * @param {any} chunk
         * @returns {Promise<any>}
         */
        write(chunk) {
            const ret = wasm.intounderlyingsink_write(this.__wbg_ptr, addHeapObject(chunk));
            return takeObject(ret);
        }
    }
    if (Symbol.dispose) IntoUnderlyingSink.prototype[Symbol.dispose] = IntoUnderlyingSink.prototype.free;

    __exports.IntoUnderlyingSink = IntoUnderlyingSink;

    const IntoUnderlyingSourceFinalization = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingsource_free(ptr >>> 0, 1));

    class IntoUnderlyingSource {

        __destroy_into_raw() {
            const ptr = this.__wbg_ptr;
            this.__wbg_ptr = 0;
            IntoUnderlyingSourceFinalization.unregister(this);
            return ptr;
        }

        free() {
            const ptr = this.__destroy_into_raw();
            wasm.__wbg_intounderlyingsource_free(ptr, 0);
        }
        /**
         * @param {ReadableStreamDefaultController} controller
         * @returns {Promise<any>}
         */
        pull(controller) {
            const ret = wasm.intounderlyingsource_pull(this.__wbg_ptr, addHeapObject(controller));
            return takeObject(ret);
        }
        cancel() {
            const ptr = this.__destroy_into_raw();
            wasm.intounderlyingsource_cancel(ptr);
        }
    }
    if (Symbol.dispose) IntoUnderlyingSource.prototype[Symbol.dispose] = IntoUnderlyingSource.prototype.free;

    __exports.IntoUnderlyingSource = IntoUnderlyingSource;

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
        imports.wbg.__wbg_Error_e17e777aac105295 = function(arg0, arg1) {
            const ret = Error(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_Number_998bea33bd87c3e0 = function(arg0) {
            const ret = Number(getObject(arg0));
            return ret;
        };
        imports.wbg.__wbg_addEventListener_03fb1ed16f4bb2f6 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), arg4 !== 0);
        }, arguments) };
        imports.wbg.__wbg_addEventListener_775911544ac9d643 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
        }, arguments) };
        imports.wbg.__wbg_add_0b4fde499a424c50 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).add(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments) };
        imports.wbg.__wbg_add_4e0283c00f7ecabe = function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).add(getStringFromWasm0(arg1, arg2));
        }, arguments) };
        imports.wbg.__wbg_altKey_5ac2d88882a93598 = function(arg0) {
            const ret = getObject(arg0).altKey;
            return ret;
        };
        imports.wbg.__wbg_appendChild_87a6cc0aeb132c06 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).appendChild(getObject(arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_body_8822ca55cb3730d2 = function(arg0) {
            const ret = getObject(arg0).body;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_bottom_64e47c2bef78a1ec = function(arg0) {
            const ret = getObject(arg0).bottom;
            return ret;
        };
        imports.wbg.__wbg_buffer_8d40b1d762fb3c66 = function(arg0) {
            const ret = getObject(arg0).buffer;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_byobRequest_2c036bceca1e6037 = function(arg0) {
            const ret = getObject(arg0).byobRequest;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_byteLength_331a6b5545834024 = function(arg0) {
            const ret = getObject(arg0).byteLength;
            return ret;
        };
        imports.wbg.__wbg_byteOffset_49a5b5608000358b = function(arg0) {
            const ret = getObject(arg0).byteOffset;
            return ret;
        };
        imports.wbg.__wbg_call_13410aac570ffff7 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).call(getObject(arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_call_a5400b25a865cfd8 = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_cancelAnimationFrame_a3b3c0d5b5c0056d = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).cancelAnimationFrame(arg1);
        }, arguments) };
        imports.wbg.__wbg_cancelBubble_a4c48803e199b5e8 = function(arg0) {
            const ret = getObject(arg0).cancelBubble;
            return ret;
        };
        imports.wbg.__wbg_charCodeAt_2dee013561ccecd0 = function(arg0, arg1) {
            const ret = getObject(arg0).charCodeAt(arg1 >>> 0);
            return ret;
        };
        imports.wbg.__wbg_checked_e7bd7d943b3d69dd = function(arg0) {
            const ret = getObject(arg0).checked;
            return ret;
        };
        imports.wbg.__wbg_childNodes_5c44c2ec67a90732 = function(arg0) {
            const ret = getObject(arg0).childNodes;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_classList_61149e0de7c668c5 = function(arg0) {
            const ret = getObject(arg0).classList;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_clearTimeout_50a601223dd18306 = function(arg0, arg1) {
            getObject(arg0).clearTimeout(arg1);
        };
        imports.wbg.__wbg_clientWidth_8a498b7a82cae772 = function(arg0) {
            const ret = getObject(arg0).clientWidth;
            return ret;
        };
        imports.wbg.__wbg_clientX_ea858fbae3debd3c = function(arg0) {
            const ret = getObject(arg0).clientX;
            return ret;
        };
        imports.wbg.__wbg_clientY_cbb39a771d53208f = function(arg0) {
            const ret = getObject(arg0).clientY;
            return ret;
        };
        imports.wbg.__wbg_cloneNode_79d46b18d5619863 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).cloneNode();
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_cloneNode_82bea7899ad17d25 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).cloneNode(arg1 !== 0);
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_close_cccada6053ee3a65 = function() { return handleError(function (arg0) {
            getObject(arg0).close();
        }, arguments) };
        imports.wbg.__wbg_close_d71a78219dc23e91 = function() { return handleError(function (arg0) {
            getObject(arg0).close();
        }, arguments) };
        imports.wbg.__wbg_code_9c657b2df9e85331 = function(arg0, arg1) {
            const ret = getObject(arg1).code;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_composedPath_e5b3f0b3e8415bb5 = function(arg0) {
            const ret = getObject(arg0).composedPath();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_construct_870f8043a65bfeee = function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.construct(getObject(arg0), getObject(arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_contains_ee9cf412c1b422da = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).contains(getStringFromWasm0(arg1, arg2));
            return ret;
        };
        imports.wbg.__wbg_content_a26016a510c10d06 = function(arg0) {
            const ret = getObject(arg0).content;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_createComment_08abf524559fd4d7 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).createComment(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_createElementNS_ffbb8bb20b2a7e4c = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            const ret = getObject(arg0).createElementNS(arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_createElement_4909dfa2011f2abe = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_createTextNode_c71a51271fadf515 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).createTextNode(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_createTreeWalker_f11ae4ae3c808dad = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg0).createTreeWalker(getObject(arg1), arg2 >>> 0, getObject(arg3));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_ctrlKey_d85b3ef2e41e6483 = function(arg0) {
            const ret = getObject(arg0).ctrlKey;
            return ret;
        };
        imports.wbg.__wbg_deleteProperty_5fe99f4fd0f66ebe = function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.deleteProperty(getObject(arg0), getObject(arg1));
            return ret;
        }, arguments) };
        imports.wbg.__wbg_documentElement_c05bfb3e7a95df41 = function(arg0) {
            const ret = getObject(arg0).documentElement;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_document_7d29d139bd619045 = function(arg0) {
            const ret = getObject(arg0).document;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_done_75ed0ee6dd243d9d = function(arg0) {
            const ret = getObject(arg0).done;
            return ret;
        };
        imports.wbg.__wbg_enqueue_452bc2343d1c2ff9 = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).enqueue(getObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_entries_2be2f15bd5554996 = function(arg0) {
            const ret = Object.entries(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_error_7534b8e9a36f1ab4 = function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_export_3(deferred0_0, deferred0_1, 1);
            }
        };
        imports.wbg.__wbg_error_99981e16d476aa5c = function(arg0) {
            console.error(getObject(arg0));
        };
        imports.wbg.__wbg_exec_6d2aea98c93aa12f = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).exec(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_fetch_44b6058021aef5e3 = function(arg0, arg1) {
            const ret = getObject(arg0).fetch(getObject(arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_firstChild_28e9f6aeb33680c1 = function(arg0) {
            const ret = getObject(arg0).firstChild;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_firstChild_59d8fdfd4e32bf63 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).firstChild();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_firstElementChild_27076cbfeed86254 = function(arg0) {
            const ret = getObject(arg0).firstElementChild;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_focus_8541343802c6721b = function() { return handleError(function (arg0) {
            getObject(arg0).focus();
        }, arguments) };
        imports.wbg.__wbg_fullscreenElement_46b4e1ed8248950d = function(arg0) {
            const ret = getObject(arg0).fullscreenElement;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_getAttributeNames_4978b6608b12126c = function(arg0) {
            const ret = getObject(arg0).getAttributeNames();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_getAttribute_8bfaf67e99ed2ee3 = function(arg0, arg1, arg2, arg3) {
            const ret = getObject(arg1).getAttribute(getStringFromWasm0(arg2, arg3));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_getBoundingClientRect_a2461829d8aa0b30 = function(arg0) {
            const ret = getObject(arg0).getBoundingClientRect();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_getComputedStyle_06167bcde483501e = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).getComputedStyle(getObject(arg1));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_getElementById_3c3d00d9a16a01dd = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).getElementById(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_getItem_9fc74b31b896f95a = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg1).getItem(getStringFromWasm0(arg2, arg3));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_getPropertyValue_da119dca19ff1bd7 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg1).getPropertyValue(getStringFromWasm0(arg2, arg3));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_getRandomValues_38a1ff1ea09f6cc7 = function() { return handleError(function (arg0, arg1) {
            globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
        }, arguments) };
        imports.wbg.__wbg_getTime_6bb3f64e0f18f817 = function(arg0) {
            const ret = getObject(arg0).getTime();
            return ret;
        };
        imports.wbg.__wbg_getTimezoneOffset_1e3ddc1382e7c8b0 = function(arg0) {
            const ret = getObject(arg0).getTimezoneOffset();
            return ret;
        };
        imports.wbg.__wbg_get_0da715ceaecea5c8 = function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_get_2a8fe8079145daf7 = function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_get_458e874b43b18b25 = function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(getObject(arg0), getObject(arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_get_faea092da74bce6c = function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(getObject(arg0), arg1 >>> 0);
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_getwithrefkey_1dc361bd10053bfe = function(arg0, arg1) {
            const ret = getObject(arg0)[getObject(arg1)];
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_hash_61a93e84f71459f5 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).hash;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_head_e5bcca7f38d7ca47 = function(arg0) {
            const ret = getObject(arg0).head;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_height_36fae1857b6cca70 = function(arg0) {
            const ret = getObject(arg0).height;
            return ret;
        };
        imports.wbg.__wbg_host_484d55073e076054 = function(arg0) {
            const ret = getObject(arg0).host;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_id_cf626ed5d83cb98d = function(arg0, arg1) {
            const ret = getObject(arg1).id;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_includes_3c3caf0bc2f4911a = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).includes(getObject(arg1), arg2);
            return ret;
        };
        imports.wbg.__wbg_innerHeight_eacbddff807274db = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).innerHeight;
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_innerWidth_dd42bfe6b5e91e59 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).innerWidth;
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_insertBefore_30228206e8f1d3fb = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).insertBefore(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_instanceof_ArrayBuffer_67f3012529f6a2dd = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof ArrayBuffer;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Element_162e4334c7d6f450 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Element;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Error_76149ae9b431750e = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Error;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_HtmlElement_d60c51c41eb8699a = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof HTMLElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_HtmlInputElement_486d1ca974d99353 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof HTMLInputElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_KeyboardEvent_6a3bc9e945ca1c24 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof KeyboardEvent;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Map_ebb01a5b6b5ffd0b = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Map;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Node_17373399b6647f60 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Node;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_RegExp_8943b2af21c4b953 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof RegExp;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Response_50fde2cd696850bf = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Response;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_ShadowRoot_f3723967133597a3 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof ShadowRoot;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Uint8Array_9a8378d955933db7 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Uint8Array;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Window_12d20d558ef92592 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_isArray_030cce220591fb41 = function(arg0) {
            const ret = Array.isArray(getObject(arg0));
            return ret;
        };
        imports.wbg.__wbg_isArray_52653600d4b65388 = function(arg0) {
            const ret = Array.isArray(getObject(arg0));
            return ret;
        };
        imports.wbg.__wbg_isSafeInteger_1c0d1af5542e102a = function(arg0) {
            const ret = Number.isSafeInteger(getObject(arg0));
            return ret;
        };
        imports.wbg.__wbg_is_8346b6c36feaf71a = function(arg0, arg1) {
            const ret = Object.is(getObject(arg0), getObject(arg1));
            return ret;
        };
        imports.wbg.__wbg_iterator_f370b34483c71a1c = function() {
            const ret = Symbol.iterator;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_json_267c5cee469b58ab = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).json();
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_keyCode_669253200ff252c2 = function(arg0) {
            const ret = getObject(arg0).keyCode;
            return ret;
        };
        imports.wbg.__wbg_key_caac8fafdd6d5317 = function(arg0, arg1) {
            const ret = getObject(arg1).key;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_lastChild_dc1221812165e68e = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).lastChild();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_left_31939b629ff732e9 = function(arg0) {
            const ret = getObject(arg0).left;
            return ret;
        };
        imports.wbg.__wbg_length_186546c51cd61acd = function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        };
        imports.wbg.__wbg_length_6bb7e81f9d7713e4 = function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        };
        imports.wbg.__wbg_length_9d771c54845e987f = function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        };
        imports.wbg.__wbg_localStorage_9330af8bf39365ba = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).localStorage;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_location_92d89c32ae076cab = function(arg0) {
            const ret = getObject(arg0).location;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_log_0cc1b7768397bcfe = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.log(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3), getStringFromWasm0(arg4, arg5), getStringFromWasm0(arg6, arg7));
            } finally {
                wasm.__wbindgen_export_3(deferred0_0, deferred0_1, 1);
            }
        };
        imports.wbg.__wbg_log_6c7b5f4f00b8ce3f = function(arg0) {
            console.log(getObject(arg0));
        };
        imports.wbg.__wbg_log_cb9e190acc5753fb = function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.log(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_export_3(deferred0_0, deferred0_1, 1);
            }
        };
        imports.wbg.__wbg_mark_7438147ce31e9d4b = function(arg0, arg1) {
            performance.mark(getStringFromWasm0(arg0, arg1));
        };
        imports.wbg.__wbg_measure_fb7825c11612c823 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            let deferred0_0;
            let deferred0_1;
            let deferred1_0;
            let deferred1_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                deferred1_0 = arg2;
                deferred1_1 = arg3;
                performance.measure(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
            } finally {
                wasm.__wbindgen_export_3(deferred0_0, deferred0_1, 1);
                wasm.__wbindgen_export_3(deferred1_0, deferred1_1, 1);
            }
        }, arguments) };
        imports.wbg.__wbg_message_125a1b2998b3552a = function(arg0) {
            const ret = getObject(arg0).message;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_metaKey_3faf4a14e870c3d6 = function(arg0) {
            const ret = getObject(arg0).metaKey;
            return ret;
        };
        imports.wbg.__wbg_name_f733db82b3c2804d = function(arg0) {
            const ret = getObject(arg0).name;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new0_b0a0a38c201e6df5 = function() {
            const ret = new Date();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_19c25a3f2fa63a02 = function() {
            const ret = new Object();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_2e3c58a15f39f5f9 = function(arg0, arg1) {
            try {
                var state0 = {a: arg0, b: arg1};
                var cb0 = (arg0, arg1) => {
                    const a = state0.a;
                    state0.a = 0;
                    try {
                        return __wbg_adapter_417(a, state0.b, arg0, arg1);
                    } finally {
                        state0.a = a;
                    }
                };
                const ret = new Promise(cb0);
                return addHeapObject(ret);
            } finally {
                state0.a = state0.b = 0;
            }
        };
        imports.wbg.__wbg_new_5a2ae4557f92b50e = function(arg0) {
            const ret = new Date(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_638ebfaedbf32a5e = function(arg0) {
            const ret = new Uint8Array(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_8a6f238a6ece86ea = function() {
            const ret = new Error();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_da9dc54c5db29dfa = function(arg0, arg1) {
            const ret = new Error(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_f6e53210afea8e45 = function() { return handleError(function () {
            const ret = new Headers();
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_newnoargs_254190557c45b4ec = function(arg0, arg1) {
            const ret = new Function(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithargs_b8065bb443501079 = function(arg0, arg1, arg2, arg3) {
            const ret = new Function(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithbyteoffsetandlength_e8f53910b4d42b45 = function(arg0, arg1, arg2) {
            const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithlength_c2868113efe93526 = function(arg0) {
            const ret = new Array(arg0 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithstrandinit_b5d168a29a3fd85f = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = new Request(getStringFromWasm0(arg0, arg1), getObject(arg2));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_newwithyearmonthdayhrminsec_3194e83daea43c6d = function(arg0, arg1, arg2, arg3, arg4, arg5) {
            const ret = new Date(arg0 >>> 0, arg1, arg2, arg3, arg4, arg5);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_nextNode_4427450a4a034417 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).nextNode();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_nextSibling_1fb03516719cac0f = function(arg0) {
            const ret = getObject(arg0).nextSibling;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_next_5b3530e612fde77d = function(arg0) {
            const ret = getObject(arg0).next;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_next_692e82279131b03c = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).next();
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_nodeType_21b604b4479f9971 = function(arg0) {
            const ret = getObject(arg0).nodeType;
            return ret;
        };
        imports.wbg.__wbg_offsetHeight_e1b96512ac8f6bef = function(arg0) {
            const ret = getObject(arg0).offsetHeight;
            return ret;
        };
        imports.wbg.__wbg_offsetWidth_b88cdf0c2f174fdb = function(arg0) {
            const ret = getObject(arg0).offsetWidth;
            return ret;
        };
        imports.wbg.__wbg_outerHTML_5fe297cb1fc146f2 = function(arg0, arg1) {
            const ret = getObject(arg1).outerHTML;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_ownKeys_36e096e00ffe2676 = function() { return handleError(function (arg0) {
            const ret = Reflect.ownKeys(getObject(arg0));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_parentElement_48440a9a6ba034a8 = function(arg0) {
            const ret = getObject(arg0).parentElement;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_parentNode_cc820baee7401ca3 = function(arg0) {
            const ret = getObject(arg0).parentNode;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_parseFloat_a2a3ff5e990a3609 = function(arg0, arg1) {
            const ret = Number.parseFloat(getStringFromWasm0(arg0, arg1));
            return ret;
        };
        imports.wbg.__wbg_prepend_7dff607c90dfb92b = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).prepend(getObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_preventDefault_fab9a085b3006058 = function(arg0) {
            getObject(arg0).preventDefault();
        };
        imports.wbg.__wbg_previousNode_cb3722094ac6f695 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).previousNode();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_prototypesetcall_3d4a26c1ed734349 = function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), getObject(arg2));
        };
        imports.wbg.__wbg_querySelector_2bda8764de90ea1d = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).querySelector(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_querySelector_438f2df379fbb8d2 = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).querySelector(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_queueMicrotask_25d0739ac89e8c88 = function(arg0) {
            queueMicrotask(getObject(arg0));
        };
        imports.wbg.__wbg_queueMicrotask_4488407636f5bf24 = function(arg0) {
            const ret = getObject(arg0).queueMicrotask;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_readyState_ab8f5acc8f40d379 = function(arg0, arg1) {
            const ret = getObject(arg1).readyState;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_removeAttribute_cf35412842be6ae4 = function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).removeAttribute(getStringFromWasm0(arg1, arg2));
        }, arguments) };
        imports.wbg.__wbg_removeEventListener_6d5be9c2821a511e = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
        }, arguments) };
        imports.wbg.__wbg_removeEventListener_b370c9d66874eec6 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), arg4 !== 0);
        }, arguments) };
        imports.wbg.__wbg_removeItem_487c5a070c7adaf7 = function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).removeItem(getStringFromWasm0(arg1, arg2));
        }, arguments) };
        imports.wbg.__wbg_removeProperty_8912427f4d0f6361 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg1).removeProperty(getStringFromWasm0(arg2, arg3));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_remove_0cd5aff2ca3eb753 = function(arg0) {
            getObject(arg0).remove();
        };
        imports.wbg.__wbg_remove_1c0c3973415d34f5 = function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).remove(getStringFromWasm0(arg1, arg2));
        }, arguments) };
        imports.wbg.__wbg_remove_ec5a2220bdb66157 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).remove(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments) };
        imports.wbg.__wbg_remove_fec7bce376b31b32 = function(arg0) {
            getObject(arg0).remove();
        };
        imports.wbg.__wbg_requestAnimationFrame_ddc84a7def436784 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).requestAnimationFrame(getObject(arg1));
            return ret;
        }, arguments) };
        imports.wbg.__wbg_requestFullscreen_a6892593b011e8d9 = function() { return handleError(function (arg0) {
            getObject(arg0).requestFullscreen();
        }, arguments) };
        imports.wbg.__wbg_resolve_4055c623acdd6a1b = function(arg0) {
            const ret = Promise.resolve(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_respond_6c2c4e20ef85138e = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).respond(arg1 >>> 0);
        }, arguments) };
        imports.wbg.__wbg_right_0e878bf84db3b0b9 = function(arg0) {
            const ret = getObject(arg0).right;
            return ret;
        };
        imports.wbg.__wbg_root_29355e4a2f0366f2 = function(arg0) {
            const ret = getObject(arg0).root;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_scrollIntoView_5974f35cea21457b = function(arg0, arg1) {
            getObject(arg0).scrollIntoView(arg1 !== 0);
        };
        imports.wbg.__wbg_scrollIntoView_e0b71bd3fc8a958e = function(arg0, arg1) {
            getObject(arg0).scrollIntoView(getObject(arg1));
        };
        imports.wbg.__wbg_scrollLeft_a4407513e33e9e52 = function(arg0) {
            const ret = getObject(arg0).scrollLeft;
            return ret;
        };
        imports.wbg.__wbg_scrollTop_f8726681d51de08b = function(arg0) {
            const ret = getObject(arg0).scrollTop;
            return ret;
        };
        imports.wbg.__wbg_scrollX_aef4ea179e1efd4d = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).scrollX;
            return ret;
        }, arguments) };
        imports.wbg.__wbg_scrollY_9e7c08e7b97acdbd = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).scrollY;
            return ret;
        }, arguments) };
        imports.wbg.__wbg_setAttribute_d1baf9023ad5696f = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments) };
        imports.wbg.__wbg_setItem_7add5eb06a28b38f = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setItem(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments) };
        imports.wbg.__wbg_setProperty_a4431938dd3e6945 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setProperty(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments) };
        imports.wbg.__wbg_setTimeout_2966518f28aef92e = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).setTimeout(getObject(arg1), arg2);
            return ret;
        }, arguments) };
        imports.wbg.__wbg_set_1353b2a5e96bc48c = function(arg0, arg1, arg2) {
            getObject(arg0).set(getArrayU8FromWasm0(arg1, arg2));
        };
        imports.wbg.__wbg_set_453345bcda80b89a = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
            return ret;
        }, arguments) };
        imports.wbg.__wbg_set_90f6c0f7bd8c0415 = function(arg0, arg1, arg2) {
            getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
        };
        imports.wbg.__wbg_setacceptnode_4b1f6d6dc2fbc44c = function(arg0, arg1) {
            getObject(arg0).acceptNode = getObject(arg1);
        };
        imports.wbg.__wbg_setbehavior_c7b6d42c62082175 = function(arg0, arg1) {
            getObject(arg0).behavior = __wbindgen_enum_ScrollBehavior[arg1];
        };
        imports.wbg.__wbg_setblock_4f090bcac575a38c = function(arg0, arg1) {
            getObject(arg0).block = __wbindgen_enum_ScrollLogicalPosition[arg1];
        };
        imports.wbg.__wbg_setcurrentNode_4a111f199328f4e2 = function(arg0, arg1) {
            getObject(arg0).currentNode = getObject(arg1);
        };
        imports.wbg.__wbg_setheaders_0052283e2f3503d1 = function(arg0, arg1) {
            getObject(arg0).headers = getObject(arg1);
        };
        imports.wbg.__wbg_setinnerHTML_34e240d6b8e8260c = function(arg0, arg1, arg2) {
            getObject(arg0).innerHTML = getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_setmethod_9b504d5b855b329c = function(arg0, arg1, arg2) {
            getObject(arg0).method = getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_setnodeValue_629799145cb84fd8 = function(arg0, arg1, arg2) {
            getObject(arg0).nodeValue = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_setopen_3a524fae88576ee7 = function(arg0, arg1) {
            getObject(arg0).open = arg1 !== 0;
        };
        imports.wbg.__wbg_setscrollLeft_8bf07096a8426b3e = function(arg0, arg1) {
            getObject(arg0).scrollLeft = arg1;
        };
        imports.wbg.__wbg_setscrollTop_dab9c942402b0fcc = function(arg0, arg1) {
            getObject(arg0).scrollTop = arg1;
        };
        imports.wbg.__wbg_settextContent_b55fe2f5f1399466 = function(arg0, arg1, arg2) {
            getObject(arg0).textContent = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_setvalue_62c6a57a4254987d = function(arg0, arg1, arg2) {
            getObject(arg0).value = getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_setx_feec91714721ae4c = function(arg0, arg1) {
            getObject(arg0).x = arg1;
        };
        imports.wbg.__wbg_sety_e7ec55e609df1262 = function(arg0, arg1) {
            getObject(arg0).y = arg1;
        };
        imports.wbg.__wbg_slice_974daea329f5c01d = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).slice(arg1 >>> 0, arg2 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_stack_0ed75d68575b0f3c = function(arg0, arg1) {
            const ret = getObject(arg1).stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_static_accessor_GLOBAL_8921f820c2ce3f12 = function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_static_accessor_GLOBAL_THIS_f0a4409105898184 = function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_static_accessor_SELF_995b214ae681ff99 = function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_static_accessor_WINDOW_cde3890479c675ea = function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_status_3fea3036088621d6 = function(arg0) {
            const ret = getObject(arg0).status;
            return ret;
        };
        imports.wbg.__wbg_stopPropagation_648b944424ad6fc4 = function(arg0) {
            getObject(arg0).stopPropagation();
        };
        imports.wbg.__wbg_stringify_4a34a65f0d4e236f = function(arg0, arg1) {
            const ret = JSON.stringify(getObject(arg1));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_stringify_b98c93d0a190446a = function() { return handleError(function (arg0) {
            const ret = JSON.stringify(getObject(arg0));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_style_32a3c8393b46a115 = function(arg0) {
            const ret = getObject(arg0).style;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_tagName_0052175ec2444f12 = function(arg0, arg1) {
            const ret = getObject(arg1).tagName;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_target_f2c963b447be6283 = function(arg0) {
            const ret = getObject(arg0).target;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_textContent_4e2b2a6c46694642 = function(arg0, arg1) {
            const ret = getObject(arg1).textContent;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_text_0f69a215637b9b34 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).text();
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_then_b33a773d723afa3e = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_then_e22500defe16819f = function(arg0, arg1) {
            const ret = getObject(arg0).then(getObject(arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_toString_d8f537919ef401d6 = function(arg0) {
            const ret = getObject(arg0).toString();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_top_447ffcfee64c5d99 = function(arg0) {
            const ret = getObject(arg0).top;
            return ret;
        };
        imports.wbg.__wbg_value_89377c01d73b6102 = function(arg0, arg1) {
            const ret = getObject(arg1).value;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_value_dd9372230531eade = function(arg0) {
            const ret = getObject(arg0).value;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_value_fdf54c7557edc2e8 = function(arg0, arg1) {
            const ret = getObject(arg1).value;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_view_91cc97d57ab30530 = function(arg0) {
            const ret = getObject(arg0).view;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_wbindgenbigintgetasi64_ac743ece6ab9bba1 = function(arg0, arg1) {
            const v = getObject(arg1);
            const ret = typeof(v) === 'bigint' ? v : undefined;
            getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        };
        imports.wbg.__wbg_wbindgenbooleanget_3fe6f642c7d97746 = function(arg0) {
            const v = getObject(arg0);
            const ret = typeof(v) === 'boolean' ? v : undefined;
            return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
        };
        imports.wbg.__wbg_wbindgencbdrop_eb10308566512b88 = function(arg0) {
            const obj = getObject(arg0).original;
            if (obj.cnt-- == 1) {
                obj.a = 0;
                return true;
            }
            const ret = false;
            return ret;
        };
        imports.wbg.__wbg_wbindgendebugstring_99ef257a3ddda34d = function(arg0, arg1) {
            const ret = debugString(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_wbindgenin_d7a1ee10933d2d55 = function(arg0, arg1) {
            const ret = getObject(arg0) in getObject(arg1);
            return ret;
        };
        imports.wbg.__wbg_wbindgenisbigint_ecb90cc08a5a9154 = function(arg0) {
            const ret = typeof(getObject(arg0)) === 'bigint';
            return ret;
        };
        imports.wbg.__wbg_wbindgenisfalsy_03f4059e2ea4ee87 = function(arg0) {
            const ret = !getObject(arg0);
            return ret;
        };
        imports.wbg.__wbg_wbindgenisfunction_8cee7dce3725ae74 = function(arg0) {
            const ret = typeof(getObject(arg0)) === 'function';
            return ret;
        };
        imports.wbg.__wbg_wbindgenisnull_f3037694abe4d97a = function(arg0) {
            const ret = getObject(arg0) === null;
            return ret;
        };
        imports.wbg.__wbg_wbindgenisobject_307a53c6bd97fbf8 = function(arg0) {
            const val = getObject(arg0);
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        };
        imports.wbg.__wbg_wbindgenisstring_d4fa939789f003b0 = function(arg0) {
            const ret = typeof(getObject(arg0)) === 'string';
            return ret;
        };
        imports.wbg.__wbg_wbindgenisundefined_c4b71d073b92f3c5 = function(arg0) {
            const ret = getObject(arg0) === undefined;
            return ret;
        };
        imports.wbg.__wbg_wbindgenjsvaleq_e6f2ad59ccae1b58 = function(arg0, arg1) {
            const ret = getObject(arg0) === getObject(arg1);
            return ret;
        };
        imports.wbg.__wbg_wbindgenjsvallooseeq_9bec8c9be826bed1 = function(arg0, arg1) {
            const ret = getObject(arg0) == getObject(arg1);
            return ret;
        };
        imports.wbg.__wbg_wbindgennumberget_f74b4c7525ac05cb = function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        };
        imports.wbg.__wbg_wbindgenstringget_0f16a6ddddef376f = function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_wbindgenthrow_451ec1a8469d7eb6 = function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        };
        imports.wbg.__wbg_wbindgentryintonumber_aef53fe1d23c5fd4 = function(arg0) {
            let result;
            try { result = +getObject(arg0) } catch (e) { result = e }
            const ret = result;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_width_d69b4eebaa4e6b70 = function(arg0) {
            const ret = getObject(arg0).width;
            return ret;
        };
        imports.wbg.__wbg_x_8013374bcfc92717 = function(arg0) {
            const ret = getObject(arg0).x;
            return ret;
        };
        imports.wbg.__wbg_x_e9119c26710f8c7c = function(arg0) {
            const ret = getObject(arg0).x;
            return ret;
        };
        imports.wbg.__wbg_y_6aeac925681c4176 = function(arg0) {
            const ret = getObject(arg0).y;
            return ret;
        };
        imports.wbg.__wbindgen_cast_0e7748fe55686e2b = function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 6662, function: Function { arguments: [Externref], shim_idx: 6663, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, 6662, __wbg_adapter_33);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_2241b6af4c4b2941 = function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_4625c577ab2ec9ee = function(arg0) {
            // Cast intrinsic for `U64 -> Externref`.
            const ret = BigInt.asUintN(64, arg0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_47011614187f3ac4 = function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 6579, function: Function { arguments: [], shim_idx: 6580, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, 6579, __wbg_adapter_28);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_61fafb7af07d57b3 = function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 5708, function: Function { arguments: [NamedExternref("MouseEvent")], shim_idx: 5709, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, 5708, __wbg_adapter_20);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_655baf922fc53ce6 = function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 6567, function: Function { arguments: [NamedExternref("Event")], shim_idx: 6568, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, 6567, __wbg_adapter_23);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_68c506aafd3322e5 = function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 4511, function: Function { arguments: [NamedExternref("Event")], shim_idx: 4512, ret: Unit, inner_ret: Some(Unit) }, mutable: false }) -> Externref`.
            const ret = makeClosure(arg0, arg1, 4511, __wbg_adapter_14);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_77b8fa5e49f6738d = function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 6577, function: Function { arguments: [NamedExternref("Event")], shim_idx: 6578, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, 6577, __wbg_adapter_11);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_9ae0607507abb057 = function(arg0) {
            // Cast intrinsic for `I64 -> Externref`.
            const ret = arg0;
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_b610dc19e18f939f = function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 4825, function: Function { arguments: [NamedExternref("Node")], shim_idx: 4826, ret: U32, inner_ret: Some(U32) }, mutable: false }) -> Externref`.
            const ret = makeClosure(arg0, arg1, 4825, __wbg_adapter_17);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_d6cd19b81560fd6e = function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_cast_da1eb9466abb7835 = function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 6567, function: Function { arguments: [], shim_idx: 6569, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, 6567, __wbg_adapter_6);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_object_clone_ref = function(arg0) {
            const ret = getObject(arg0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
            takeObject(arg0);
        };

        return imports;
    }

    function __wbg_init_memory(imports, memory) {

    }

    function __wbg_finalize_init(instance, module) {
        wasm = instance.exports;
        __wbg_init.__wbindgen_wasm_module = module;
        cachedDataViewMemory0 = null;
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

        if (typeof module_or_path === 'undefined' && typeof script_src !== 'undefined') {
            module_or_path = script_src.replace(/\.js$/, '_bg.wasm');
        }
        const imports = __wbg_get_imports();

        if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
            module_or_path = fetch(module_or_path);
        }

        __wbg_init_memory(imports);

        const { instance, module } = await __wbg_load(await module_or_path, imports);

        return __wbg_finalize_init(instance, module);
    }

    ftmlViewer = Object.assign(__wbg_init, { initSync }, __exports);

})();
ftmlViewer();
