let wasm_bindgen = (function(exports) {
    let script_src;
    if (typeof document !== 'undefined' && document.currentScript !== null) {
        script_src = new URL(document.currentScript.src, location.href).toString();
    }

    /**
     * @enum {0 | 1 | 2 | 3}
     */
    const HighlightStyle = Object.freeze({
        Colored: 0, "0": "Colored",
        Subtle: 1, "1": "Subtle",
        Off: 2, "2": "Off",
        None: 3, "3": "None",
    });
    exports.HighlightStyle = HighlightStyle;

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
        cancel() {
            const ptr = this.__destroy_into_raw();
            wasm.intounderlyingbytesource_cancel(ptr);
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
    }
    if (Symbol.dispose) IntoUnderlyingByteSource.prototype[Symbol.dispose] = IntoUnderlyingByteSource.prototype.free;
    exports.IntoUnderlyingByteSource = IntoUnderlyingByteSource;

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
    exports.IntoUnderlyingSink = IntoUnderlyingSink;

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
        cancel() {
            const ptr = this.__destroy_into_raw();
            wasm.intounderlyingsource_cancel(ptr);
        }
        /**
         * @param {ReadableStreamDefaultController} controller
         * @returns {Promise<any>}
         */
        pull(controller) {
            const ret = wasm.intounderlyingsource_pull(this.__wbg_ptr, addHeapObject(controller));
            return takeObject(ret);
        }
    }
    if (Symbol.dispose) IntoUnderlyingSource.prototype[Symbol.dispose] = IntoUnderlyingSource.prototype.free;
    exports.IntoUnderlyingSource = IntoUnderlyingSource;

    function clear_cache() {
        wasm.clear_cache();
    }
    exports.clear_cache = clear_cache;

    function print_cache() {
        wasm.print_cache();
    }
    exports.print_cache = print_cache;

    function run() {
        wasm.run();
    }
    exports.run = run;

    function __wbg_get_imports() {
        const import0 = {
            __proto__: null,
            __wbg_Error_83742b46f01ce22d: function(arg0, arg1) {
                const ret = Error(getStringFromWasm0(arg0, arg1));
                return addHeapObject(ret);
            },
            __wbg_Number_a5a435bd7bbec835: function(arg0) {
                const ret = Number(getObject(arg0));
                return ret;
            },
            __wbg___wbindgen_bigint_get_as_i64_447a76b5c6ef7bda: function(arg0, arg1) {
                const v = getObject(arg1);
                const ret = typeof(v) === 'bigint' ? v : undefined;
                getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
            },
            __wbg___wbindgen_boolean_get_c0f3f60bac5a78d1: function(arg0) {
                const v = getObject(arg0);
                const ret = typeof(v) === 'boolean' ? v : undefined;
                return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
            },
            __wbg___wbindgen_debug_string_5398f5bb970e0daa: function(arg0, arg1) {
                const ret = debugString(getObject(arg1));
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg___wbindgen_in_41dbb8413020e076: function(arg0, arg1) {
                const ret = getObject(arg0) in getObject(arg1);
                return ret;
            },
            __wbg___wbindgen_is_bigint_e2141d4f045b7eda: function(arg0) {
                const ret = typeof(getObject(arg0)) === 'bigint';
                return ret;
            },
            __wbg___wbindgen_is_falsy_30906e697739fcc2: function(arg0) {
                const ret = !getObject(arg0);
                return ret;
            },
            __wbg___wbindgen_is_function_3c846841762788c1: function(arg0) {
                const ret = typeof(getObject(arg0)) === 'function';
                return ret;
            },
            __wbg___wbindgen_is_null_0b605fc6b167c56f: function(arg0) {
                const ret = getObject(arg0) === null;
                return ret;
            },
            __wbg___wbindgen_is_object_781bc9f159099513: function(arg0) {
                const val = getObject(arg0);
                const ret = typeof(val) === 'object' && val !== null;
                return ret;
            },
            __wbg___wbindgen_is_string_7ef6b97b02428fae: function(arg0) {
                const ret = typeof(getObject(arg0)) === 'string';
                return ret;
            },
            __wbg___wbindgen_is_undefined_52709e72fb9f179c: function(arg0) {
                const ret = getObject(arg0) === undefined;
                return ret;
            },
            __wbg___wbindgen_jsval_eq_ee31bfad3e536463: function(arg0, arg1) {
                const ret = getObject(arg0) === getObject(arg1);
                return ret;
            },
            __wbg___wbindgen_jsval_loose_eq_5bcc3bed3c69e72b: function(arg0, arg1) {
                const ret = getObject(arg0) == getObject(arg1);
                return ret;
            },
            __wbg___wbindgen_number_get_34bb9d9dcfa21373: function(arg0, arg1) {
                const obj = getObject(arg1);
                const ret = typeof(obj) === 'number' ? obj : undefined;
                getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
            },
            __wbg___wbindgen_string_get_395e606bd0ee4427: function(arg0, arg1) {
                const obj = getObject(arg1);
                const ret = typeof(obj) === 'string' ? obj : undefined;
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg___wbindgen_throw_6ddd609b62940d55: function(arg0, arg1) {
                throw new Error(getStringFromWasm0(arg0, arg1));
            },
            __wbg___wbindgen_try_into_number_aab92f8d9dc246ab: function(arg0) {
                let result;
                try { result = +getObject(arg0) } catch (e) { result = e }
                const ret = result;
                return addHeapObject(ret);
            },
            __wbg__wbg_cb_unref_6b5b6b8576d35cb1: function(arg0) {
                getObject(arg0)._wbg_cb_unref();
            },
            __wbg_addEventListener_2d985aa8a656f6dc: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
            }, arguments); },
            __wbg_addEventListener_e1265e2114ffe62c: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), arg4 !== 0);
            }, arguments); },
            __wbg_add_6c0a5a17e83d73e8: function() { return handleError(function (arg0, arg1, arg2) {
                getObject(arg0).add(getStringFromWasm0(arg1, arg2));
            }, arguments); },
            __wbg_add_add91282812c9d5f: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                getObject(arg0).add(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            }, arguments); },
            __wbg_altKey_a8e58d65866de029: function(arg0) {
                const ret = getObject(arg0).altKey;
                return ret;
            },
            __wbg_appendChild_8cb157b6ec5612a6: function() { return handleError(function (arg0, arg1) {
                const ret = getObject(arg0).appendChild(getObject(arg1));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_append_7005a163aaf45076: function() { return handleError(function (arg0, arg1) {
                getObject(arg0).append(getObject(arg1));
            }, arguments); },
            __wbg_body_5eb99e7257e5ae34: function(arg0) {
                const ret = getObject(arg0).body;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_bottom_bfd7fcb3cf1632f0: function(arg0) {
                const ret = getObject(arg0).bottom;
                return ret;
            },
            __wbg_buffer_60b8043cd926067d: function(arg0) {
                const ret = getObject(arg0).buffer;
                return addHeapObject(ret);
            },
            __wbg_byobRequest_6342e5f2b232c0f9: function(arg0) {
                const ret = getObject(arg0).byobRequest;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_byteLength_607b856aa6c5a508: function(arg0) {
                const ret = getObject(arg0).byteLength;
                return ret;
            },
            __wbg_byteOffset_b26b63681c83856c: function(arg0) {
                const ret = getObject(arg0).byteOffset;
                return ret;
            },
            __wbg_call_2d781c1f4d5c0ef8: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_call_e133b57c9155d22c: function() { return handleError(function (arg0, arg1) {
                const ret = getObject(arg0).call(getObject(arg1));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_cancelAnimationFrame_43fad84647f46036: function() { return handleError(function (arg0, arg1) {
                getObject(arg0).cancelAnimationFrame(arg1);
            }, arguments); },
            __wbg_cancelBubble_a023be4f28708976: function(arg0) {
                const ret = getObject(arg0).cancelBubble;
                return ret;
            },
            __wbg_charCodeAt_9a5ff0c67a023a8c: function(arg0, arg1) {
                const ret = getObject(arg0).charCodeAt(arg1 >>> 0);
                return ret;
            },
            __wbg_checked_7b8b07c4341e3e6c: function(arg0) {
                const ret = getObject(arg0).checked;
                return ret;
            },
            __wbg_childNodes_2deab43783cba95d: function(arg0) {
                const ret = getObject(arg0).childNodes;
                return addHeapObject(ret);
            },
            __wbg_classList_07fcb7252e322f6f: function(arg0) {
                const ret = getObject(arg0).classList;
                return addHeapObject(ret);
            },
            __wbg_clearTimeout_fdfb5a1468af1a97: function(arg0, arg1) {
                getObject(arg0).clearTimeout(arg1);
            },
            __wbg_clientWidth_33c7e9c1bcdf4a7e: function(arg0) {
                const ret = getObject(arg0).clientWidth;
                return ret;
            },
            __wbg_clientX_51eb1976d50e9aa7: function(arg0) {
                const ret = getObject(arg0).clientX;
                return ret;
            },
            __wbg_clientY_bf90b5a826653c23: function(arg0) {
                const ret = getObject(arg0).clientY;
                return ret;
            },
            __wbg_cloneNode_c2549c13a4ecec77: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).cloneNode();
                return addHeapObject(ret);
            }, arguments); },
            __wbg_cloneNode_d4cc926db030a95a: function() { return handleError(function (arg0, arg1) {
                const ret = getObject(arg0).cloneNode(arg1 !== 0);
                return addHeapObject(ret);
            }, arguments); },
            __wbg_close_690d36108c557337: function() { return handleError(function (arg0) {
                getObject(arg0).close();
            }, arguments); },
            __wbg_close_737b4b1fbc658540: function() { return handleError(function (arg0) {
                getObject(arg0).close();
            }, arguments); },
            __wbg_code_3c69123dcbcf263d: function(arg0, arg1) {
                const ret = getObject(arg1).code;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_commonAncestorContainer_761834a4a1b930a8: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).commonAncestorContainer;
                return addHeapObject(ret);
            }, arguments); },
            __wbg_composedPath_bb47f74544516629: function(arg0) {
                const ret = getObject(arg0).composedPath();
                return addHeapObject(ret);
            },
            __wbg_construct_526a6dedb187eba9: function() { return handleError(function (arg0, arg1) {
                const ret = Reflect.construct(getObject(arg0), getObject(arg1));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_contains_d1b71c766bb23709: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).contains(getStringFromWasm0(arg1, arg2));
                return ret;
            },
            __wbg_content_619b8326d45461ad: function(arg0) {
                const ret = getObject(arg0).content;
                return addHeapObject(ret);
            },
            __wbg_createComment_ffac108d87a2b6f1: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).createComment(getStringFromWasm0(arg1, arg2));
                return addHeapObject(ret);
            },
            __wbg_createElementNS_aac38e987dc3e061: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                const ret = getObject(arg0).createElementNS(arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_createElement_9b0aab265c549ded: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_createTextNode_1997fd99f09c6afd: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).createTextNode(getStringFromWasm0(arg1, arg2));
                return addHeapObject(ret);
            },
            __wbg_createTreeWalker_bcec34d383a40ea2: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                const ret = getObject(arg0).createTreeWalker(getObject(arg1), arg2 >>> 0, getObject(arg3));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_ctrlKey_a41da599a72ee93d: function(arg0) {
                const ret = getObject(arg0).ctrlKey;
                return ret;
            },
            __wbg_deleteProperty_c5d2b597a2113d49: function() { return handleError(function (arg0, arg1) {
                const ret = Reflect.deleteProperty(getObject(arg0), getObject(arg1));
                return ret;
            }, arguments); },
            __wbg_documentElement_bef4a2d2289074bb: function(arg0) {
                const ret = getObject(arg0).documentElement;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_document_c0320cd4183c6d9b: function(arg0) {
                const ret = getObject(arg0).document;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_done_08ce71ee07e3bd17: function(arg0) {
                const ret = getObject(arg0).done;
                return ret;
            },
            __wbg_enqueue_ec3552838b4b7fbf: function() { return handleError(function (arg0, arg1) {
                getObject(arg0).enqueue(getObject(arg1));
            }, arguments); },
            __wbg_entries_e8a20ff8c9757101: function(arg0) {
                const ret = Object.entries(getObject(arg0));
                return addHeapObject(ret);
            },
            __wbg_error_8d9a8e04cd1d3588: function(arg0) {
                console.error(getObject(arg0));
            },
            __wbg_error_a6fa202b58aa1cd3: function(arg0, arg1) {
                let deferred0_0;
                let deferred0_1;
                try {
                    deferred0_0 = arg0;
                    deferred0_1 = arg1;
                    console.error(getStringFromWasm0(arg0, arg1));
                } finally {
                    wasm.__wbindgen_export4(deferred0_0, deferred0_1, 1);
                }
            },
            __wbg_exec_203e2096c69172ee: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).exec(getStringFromWasm0(arg1, arg2));
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_fetch_f8a611684c3b5fe5: function(arg0, arg1) {
                const ret = getObject(arg0).fetch(getObject(arg1));
                return addHeapObject(ret);
            },
            __wbg_firstChild_6b7286f4a222fe7f: function(arg0) {
                const ret = getObject(arg0).firstChild;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_firstChild_7e144d0294dd2f26: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).firstChild();
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_firstElementChild_d9980c0530e50cdc: function(arg0) {
                const ret = getObject(arg0).firstElementChild;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_focus_885197ce680db9e0: function() { return handleError(function (arg0) {
                getObject(arg0).focus();
            }, arguments); },
            __wbg_fromEntries_8f078e02a548e8eb: function() { return handleError(function (arg0) {
                const ret = Object.fromEntries(getObject(arg0));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_fullscreenElement_8068aa5be9c86543: function(arg0) {
                const ret = getObject(arg0).fullscreenElement;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_getAttributeNames_261cf791b7205510: function(arg0) {
                const ret = getObject(arg0).getAttributeNames();
                return addHeapObject(ret);
            },
            __wbg_getAttribute_cf830fef39b6ba0e: function(arg0, arg1, arg2, arg3) {
                const ret = getObject(arg1).getAttribute(getStringFromWasm0(arg2, arg3));
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_getBoundingClientRect_b236f2e393fd0e7a: function(arg0) {
                const ret = getObject(arg0).getBoundingClientRect();
                return addHeapObject(ret);
            },
            __wbg_getComputedStyle_b12e52450a4be72c: function() { return handleError(function (arg0, arg1) {
                const ret = getObject(arg0).getComputedStyle(getObject(arg1));
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_getElementById_d1f25d287b19a833: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).getElementById(getStringFromWasm0(arg1, arg2));
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_getItem_a7cc1d4f154f2e6f: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                const ret = getObject(arg1).getItem(getStringFromWasm0(arg2, arg3));
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            }, arguments); },
            __wbg_getPropertyValue_d2181532557839cf: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                const ret = getObject(arg1).getPropertyValue(getStringFromWasm0(arg2, arg3));
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            }, arguments); },
            __wbg_getRandomValues_a1cf2e70b003a59d: function() { return handleError(function (arg0, arg1) {
                globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
            }, arguments); },
            __wbg_getRangeAt_5d9ee06243f91295: function() { return handleError(function (arg0, arg1) {
                const ret = getObject(arg0).getRangeAt(arg1 >>> 0);
                return addHeapObject(ret);
            }, arguments); },
            __wbg_getSelection_e25df586d31850a4: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).getSelection();
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_getTime_1dad7b5386ddd2d9: function(arg0) {
                const ret = getObject(arg0).getTime();
                return ret;
            },
            __wbg_getTimezoneOffset_639bcf2dde21158b: function(arg0) {
                const ret = getObject(arg0).getTimezoneOffset();
                return ret;
            },
            __wbg_get_326e41e095fb2575: function() { return handleError(function (arg0, arg1) {
                const ret = Reflect.get(getObject(arg0), getObject(arg1));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_get_3ef1eba1850ade27: function() { return handleError(function (arg0, arg1) {
                const ret = Reflect.get(getObject(arg0), getObject(arg1));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_get_a8ee5c45dabc1b3b: function(arg0, arg1) {
                const ret = getObject(arg0)[arg1 >>> 0];
                return addHeapObject(ret);
            },
            __wbg_get_c7546417fb0bec10: function(arg0, arg1) {
                const ret = getObject(arg0)[arg1 >>> 0];
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_get_deb8ff677e1f3e44: function() { return handleError(function (arg0, arg1) {
                const ret = Reflect.get(getObject(arg0), arg1 >>> 0);
                return addHeapObject(ret);
            }, arguments); },
            __wbg_get_unchecked_329cfe50afab7352: function(arg0, arg1) {
                const ret = getObject(arg0)[arg1 >>> 0];
                return addHeapObject(ret);
            },
            __wbg_get_with_ref_key_6412cf3094599694: function(arg0, arg1) {
                const ret = getObject(arg0)[getObject(arg1)];
                return addHeapObject(ret);
            },
            __wbg_hash_d4a10bb21e6ca48e: function() { return handleError(function (arg0, arg1) {
                const ret = getObject(arg1).hash;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            }, arguments); },
            __wbg_head_3cb79209fe5c12b3: function(arg0) {
                const ret = getObject(arg0).head;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_height_75b0d10baf97e535: function(arg0) {
                const ret = getObject(arg0).height;
                return ret;
            },
            __wbg_host_0b682963054cd706: function(arg0) {
                const ret = getObject(arg0).host;
                return addHeapObject(ret);
            },
            __wbg_id_491c6437a748ea9f: function(arg0, arg1) {
                const ret = getObject(arg1).id;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_includes_9f81335525be01f9: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).includes(getObject(arg1), arg2);
                return ret;
            },
            __wbg_innerHeight_ba245c3eff70b2a1: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).innerHeight;
                return addHeapObject(ret);
            }, arguments); },
            __wbg_innerWidth_e6af2d66d3b06991: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).innerWidth;
                return addHeapObject(ret);
            }, arguments); },
            __wbg_insertBefore_64157928ea5f5def: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = getObject(arg0).insertBefore(getObject(arg1), getObject(arg2));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_instanceof_ArrayBuffer_101e2bf31071a9f6: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof ArrayBuffer;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Element_7f177ac0337279af: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof Element;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Error_4691a5b466e32a80: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof Error;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_HtmlElement_ef05df8222c2b81b: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof HTMLElement;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_HtmlInputElement_f6b9c8ea98b1980f: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof HTMLInputElement;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_KeyboardEvent_b75e17d5aabde4f1: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof KeyboardEvent;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Map_f194b366846aca0c: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof Map;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Node_bdeed8fbed053a40: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof Node;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_RegExp_547d1eeb75506fe0: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof RegExp;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Response_9b4d9fd451e051b1: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof Response;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_ShadowRoot_a69345f84dc5ff1b: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof ShadowRoot;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Uint8Array_740438561a5b956d: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof Uint8Array;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Window_23e677d2c6843922: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof Window;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_isArray_33b91feb269ff46e: function(arg0) {
                const ret = Array.isArray(getObject(arg0));
                return ret;
            },
            __wbg_isArray_42f3245bcac28e65: function(arg0) {
                const ret = Array.isArray(getObject(arg0));
                return ret;
            },
            __wbg_isSafeInteger_ecd6a7f9c3e053cd: function(arg0) {
                const ret = Number.isSafeInteger(getObject(arg0));
                return ret;
            },
            __wbg_is_a166b9958c2438ad: function(arg0, arg1) {
                const ret = Object.is(getObject(arg0), getObject(arg1));
                return ret;
            },
            __wbg_iterator_d8f549ec8fb061b1: function() {
                const ret = Symbol.iterator;
                return addHeapObject(ret);
            },
            __wbg_json_602d0b5448ab6391: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).json();
                return addHeapObject(ret);
            }, arguments); },
            __wbg_keyCode_bec0b9a76cae4555: function(arg0) {
                const ret = getObject(arg0).keyCode;
                return ret;
            },
            __wbg_key_99eb0f0a1000963d: function(arg0, arg1) {
                const ret = getObject(arg1).key;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_lastChild_76936cd705aaef04: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).lastChild();
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_left_0050d4abe2736ee9: function(arg0) {
                const ret = getObject(arg0).left;
                return ret;
            },
            __wbg_length_15d3fc853a68bbbc: function(arg0) {
                const ret = getObject(arg0).length;
                return ret;
            },
            __wbg_length_b3416cf66a5452c8: function(arg0) {
                const ret = getObject(arg0).length;
                return ret;
            },
            __wbg_length_ea16607d7b61445b: function(arg0) {
                const ret = getObject(arg0).length;
                return ret;
            },
            __wbg_localStorage_51c38b3b222e1ed2: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).localStorage;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_location_fc8d47802682dd93: function(arg0) {
                const ret = getObject(arg0).location;
                return addHeapObject(ret);
            },
            __wbg_log_0c201ade58bb55e1: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
                let deferred0_0;
                let deferred0_1;
                try {
                    deferred0_0 = arg0;
                    deferred0_1 = arg1;
                    console.log(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3), getStringFromWasm0(arg4, arg5), getStringFromWasm0(arg6, arg7));
                } finally {
                    wasm.__wbindgen_export4(deferred0_0, deferred0_1, 1);
                }
            },
            __wbg_log_524eedafa26daa59: function(arg0) {
                console.log(getObject(arg0));
            },
            __wbg_log_ce2c4456b290c5e7: function(arg0, arg1) {
                let deferred0_0;
                let deferred0_1;
                try {
                    deferred0_0 = arg0;
                    deferred0_1 = arg1;
                    console.log(getStringFromWasm0(arg0, arg1));
                } finally {
                    wasm.__wbindgen_export4(deferred0_0, deferred0_1, 1);
                }
            },
            __wbg_mark_b4d943f3bc2d2404: function(arg0, arg1) {
                performance.mark(getStringFromWasm0(arg0, arg1));
            },
            __wbg_measure_84362959e621a2c1: function() { return handleError(function (arg0, arg1, arg2, arg3) {
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
                    wasm.__wbindgen_export4(deferred0_0, deferred0_1, 1);
                    wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
                }
            }, arguments); },
            __wbg_message_00d63f20c41713dd: function(arg0) {
                const ret = getObject(arg0).message;
                return addHeapObject(ret);
            },
            __wbg_metaKey_09c90f191df1276b: function(arg0) {
                const ret = getObject(arg0).metaKey;
                return ret;
            },
            __wbg_name_ecf53d5e050a495d: function(arg0) {
                const ret = getObject(arg0).name;
                return addHeapObject(ret);
            },
            __wbg_new_0837727332ac86ba: function() { return handleError(function () {
                const ret = new Headers();
                return addHeapObject(ret);
            }, arguments); },
            __wbg_new_0_1dcafdf5e786e876: function() {
                const ret = new Date();
                return addHeapObject(ret);
            },
            __wbg_new_227d7c05414eb861: function() {
                const ret = new Error();
                return addHeapObject(ret);
            },
            __wbg_new_5f486cdf45a04d78: function(arg0) {
                const ret = new Uint8Array(getObject(arg0));
                return addHeapObject(ret);
            },
            __wbg_new_aad8cb4adc774d03: function(arg0, arg1, arg2, arg3) {
                const ret = new RegExp(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
                return addHeapObject(ret);
            },
            __wbg_new_ab79df5bd7c26067: function() {
                const ret = new Object();
                return addHeapObject(ret);
            },
            __wbg_new_d15cb560a6a0e5f0: function(arg0, arg1) {
                const ret = new Error(getStringFromWasm0(arg0, arg1));
                return addHeapObject(ret);
            },
            __wbg_new_fd94ca5c9639abd2: function(arg0) {
                const ret = new Date(getObject(arg0));
                return addHeapObject(ret);
            },
            __wbg_new_typed_aaaeaf29cf802876: function(arg0, arg1) {
                try {
                    var state0 = {a: arg0, b: arg1};
                    var cb0 = (arg0, arg1) => {
                        const a = state0.a;
                        state0.a = 0;
                        try {
                            return __wasm_bindgen_func_elem_33265(a, state0.b, arg0, arg1);
                        } finally {
                            state0.a = a;
                        }
                    };
                    const ret = new Promise(cb0);
                    return addHeapObject(ret);
                } finally {
                    state0.a = state0.b = 0;
                }
            },
            __wbg_new_with_args_4c76adfe7b6e456b: function(arg0, arg1, arg2, arg3) {
                const ret = new Function(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
                return addHeapObject(ret);
            },
            __wbg_new_with_byte_offset_and_length_b2ec5bf7b2f35743: function(arg0, arg1, arg2) {
                const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
                return addHeapObject(ret);
            },
            __wbg_new_with_length_3259a525196bd8cc: function(arg0) {
                const ret = new Array(arg0 >>> 0);
                return addHeapObject(ret);
            },
            __wbg_new_with_str_and_init_b4b54d1a819bc724: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = new Request(getStringFromWasm0(arg0, arg1), getObject(arg2));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_new_with_year_month_day_hr_min_sec_6cf3950ed2c8fa20: function(arg0, arg1, arg2, arg3, arg4, arg5) {
                const ret = new Date(arg0 >>> 0, arg1, arg2, arg3, arg4, arg5);
                return addHeapObject(ret);
            },
            __wbg_nextNode_4ed83085563c43d1: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).nextNode();
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_nextSibling_afcfdc4e646795ea: function(arg0) {
                const ret = getObject(arg0).nextSibling;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_next_11b99ee6237339e3: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).next();
                return addHeapObject(ret);
            }, arguments); },
            __wbg_next_e01a967809d1aa68: function(arg0) {
                const ret = getObject(arg0).next;
                return addHeapObject(ret);
            },
            __wbg_nodeType_17b68b25b25c1b52: function(arg0) {
                const ret = getObject(arg0).nodeType;
                return ret;
            },
            __wbg_offsetHeight_d67fa0a1c7a582f1: function(arg0) {
                const ret = getObject(arg0).offsetHeight;
                return ret;
            },
            __wbg_offsetWidth_2e99270f392b1f3e: function(arg0) {
                const ret = getObject(arg0).offsetWidth;
                return ret;
            },
            __wbg_outerHTML_752fc2596e15dbab: function(arg0, arg1) {
                const ret = getObject(arg1).outerHTML;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_ownKeys_69606bb1d6ee5825: function() { return handleError(function (arg0) {
                const ret = Reflect.ownKeys(getObject(arg0));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_parentElement_6ea1a9ddfe78f22d: function(arg0) {
                const ret = getObject(arg0).parentElement;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_parentNode_f02c28ae3eec09bc: function(arg0) {
                const ret = getObject(arg0).parentNode;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_parseFloat_939abee6e9d53759: function(arg0, arg1) {
                const ret = Number.parseFloat(getStringFromWasm0(arg0, arg1));
                return ret;
            },
            __wbg_prepend_ebebf3d32ea17a06: function() { return handleError(function (arg0, arg1) {
                getObject(arg0).prepend(getObject(arg1));
            }, arguments); },
            __wbg_preventDefault_25a229bfe5c510f8: function(arg0) {
                getObject(arg0).preventDefault();
            },
            __wbg_previousNode_178f11bf6765bb83: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).previousNode();
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_prototypesetcall_d62e5099504357e6: function(arg0, arg1, arg2) {
                Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), getObject(arg2));
            },
            __wbg_querySelector_332d8dfa3e191085: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = getObject(arg0).querySelector(getStringFromWasm0(arg1, arg2));
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_querySelector_46ff1b81410aebea: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = getObject(arg0).querySelector(getStringFromWasm0(arg1, arg2));
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_queueMicrotask_0c399741342fb10f: function(arg0) {
                const ret = getObject(arg0).queueMicrotask;
                return addHeapObject(ret);
            },
            __wbg_queueMicrotask_a082d78ce798393e: function(arg0) {
                queueMicrotask(getObject(arg0));
            },
            __wbg_readyState_fbc69db5c1782a2f: function(arg0, arg1) {
                const ret = getObject(arg1).readyState;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_removeAttribute_c0738b49de4ead0b: function() { return handleError(function (arg0, arg1, arg2) {
                getObject(arg0).removeAttribute(getStringFromWasm0(arg1, arg2));
            }, arguments); },
            __wbg_removeEventListener_8ba966e6a7606bce: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), arg4 !== 0);
            }, arguments); },
            __wbg_removeEventListener_d27694700fc0df8b: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
            }, arguments); },
            __wbg_removeItem_95c258b9afdd7580: function() { return handleError(function (arg0, arg1, arg2) {
                getObject(arg0).removeItem(getStringFromWasm0(arg1, arg2));
            }, arguments); },
            __wbg_removeProperty_5b3523637b608633: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                const ret = getObject(arg1).removeProperty(getStringFromWasm0(arg2, arg3));
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            }, arguments); },
            __wbg_remove_42a5170abc5b5fd3: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                getObject(arg0).remove(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            }, arguments); },
            __wbg_remove_83b6b382cbfb297b: function() { return handleError(function (arg0, arg1, arg2) {
                getObject(arg0).remove(getStringFromWasm0(arg1, arg2));
            }, arguments); },
            __wbg_remove_892d3e438dfde09b: function(arg0) {
                getObject(arg0).remove();
            },
            __wbg_remove_9952dd558b29246a: function(arg0) {
                getObject(arg0).remove();
            },
            __wbg_requestAnimationFrame_206c97f410e7a383: function() { return handleError(function (arg0, arg1) {
                const ret = getObject(arg0).requestAnimationFrame(getObject(arg1));
                return ret;
            }, arguments); },
            __wbg_requestFullscreen_045a0765d640e198: function() { return handleError(function (arg0) {
                getObject(arg0).requestFullscreen();
            }, arguments); },
            __wbg_resolve_ae8d83246e5bcc12: function(arg0) {
                const ret = Promise.resolve(getObject(arg0));
                return addHeapObject(ret);
            },
            __wbg_respond_e286ee502e7cf7e4: function() { return handleError(function (arg0, arg1) {
                getObject(arg0).respond(arg1 >>> 0);
            }, arguments); },
            __wbg_right_ad93e95b5e30b7ff: function(arg0) {
                const ret = getObject(arg0).right;
                return ret;
            },
            __wbg_root_2c1b30a2c9160d63: function(arg0) {
                const ret = getObject(arg0).root;
                return addHeapObject(ret);
            },
            __wbg_scrollIntoView_4cc5cba1b073696e: function(arg0, arg1) {
                getObject(arg0).scrollIntoView(getObject(arg1));
            },
            __wbg_scrollIntoView_83bfe88a65d2d4e9: function(arg0, arg1) {
                getObject(arg0).scrollIntoView(arg1 !== 0);
            },
            __wbg_scrollLeft_35d02c51b91c2bca: function(arg0) {
                const ret = getObject(arg0).scrollLeft;
                return ret;
            },
            __wbg_scrollTop_5a132a1d650e67b8: function(arg0) {
                const ret = getObject(arg0).scrollTop;
                return ret;
            },
            __wbg_scrollX_6fb90d80db4f6ecc: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).scrollX;
                return ret;
            }, arguments); },
            __wbg_scrollY_606b06b31582377e: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).scrollY;
                return ret;
            }, arguments); },
            __wbg_setAttribute_f20d3b966749ab64: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            }, arguments); },
            __wbg_setItem_5f84aeef0d7f3c17: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                getObject(arg0).setItem(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            }, arguments); },
            __wbg_setProperty_ef29d2aa64a04d2b: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                getObject(arg0).setProperty(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            }, arguments); },
            __wbg_setTimeout_7f7035ad0b026458: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = getObject(arg0).setTimeout(getObject(arg1), arg2);
                return ret;
            }, arguments); },
            __wbg_set_282384002438957f: function(arg0, arg1, arg2) {
                getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
            },
            __wbg_set_7eaa4f96924fd6b3: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
                return ret;
            }, arguments); },
            __wbg_set_8c0b3ffcf05d61c2: function(arg0, arg1, arg2) {
                getObject(arg0).set(getArrayU8FromWasm0(arg1, arg2));
            },
            __wbg_set_accept_node_f98e1ef8675b325a: function(arg0, arg1) {
                getObject(arg0).acceptNode = getObject(arg1);
            },
            __wbg_set_behavior_ca6e842dd30ac1fc: function(arg0, arg1) {
                getObject(arg0).behavior = __wbindgen_enum_ScrollBehavior[arg1];
            },
            __wbg_set_block_5237fb07fb0019fd: function(arg0, arg1) {
                getObject(arg0).block = __wbindgen_enum_ScrollLogicalPosition[arg1];
            },
            __wbg_set_body_a3d856b097dfda04: function(arg0, arg1) {
                getObject(arg0).body = getObject(arg1);
            },
            __wbg_set_currentNode_00ea42c386c9e405: function(arg0, arg1) {
                getObject(arg0).currentNode = getObject(arg1);
            },
            __wbg_set_headers_3c8fecc693b75327: function(arg0, arg1) {
                getObject(arg0).headers = getObject(arg1);
            },
            __wbg_set_innerHTML_97039584c4ab4c83: function(arg0, arg1, arg2) {
                getObject(arg0).innerHTML = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_method_8c015e8bcafd7be1: function(arg0, arg1, arg2) {
                getObject(arg0).method = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_nodeValue_a32aaa3615b2da17: function(arg0, arg1, arg2) {
                getObject(arg0).nodeValue = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_open_f8609cb71f336b7f: function(arg0, arg1) {
                getObject(arg0).open = arg1 !== 0;
            },
            __wbg_set_scrollLeft_bcd6a9f838497d4c: function(arg0, arg1) {
                getObject(arg0).scrollLeft = arg1;
            },
            __wbg_set_scrollTop_8036d5634472f573: function(arg0, arg1) {
                getObject(arg0).scrollTop = arg1;
            },
            __wbg_set_textContent_1e964492a2410e92: function(arg0, arg1, arg2) {
                getObject(arg0).textContent = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_value_b41e35a8023c51e6: function(arg0, arg1, arg2) {
                getObject(arg0).value = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_x_2dfd549c3a3ef926: function(arg0, arg1) {
                getObject(arg0).x = arg1;
            },
            __wbg_set_y_eac6a1dc40874d73: function(arg0, arg1) {
                getObject(arg0).y = arg1;
            },
            __wbg_slice_66f9d6a0aa4717dd: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).slice(arg1 >>> 0, arg2 >>> 0);
                return addHeapObject(ret);
            },
            __wbg_stack_3b0d974bbf31e44f: function(arg0, arg1) {
                const ret = getObject(arg1).stack;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_static_accessor_GLOBAL_8adb955bd33fac2f: function() {
                const ret = typeof global === 'undefined' ? null : global;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_static_accessor_GLOBAL_THIS_ad356e0db91c7913: function() {
                const ret = typeof globalThis === 'undefined' ? null : globalThis;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_static_accessor_SELF_f207c857566db248: function() {
                const ret = typeof self === 'undefined' ? null : self;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_static_accessor_WINDOW_bb9f1ba69d61b386: function() {
                const ret = typeof window === 'undefined' ? null : window;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_status_318629ab93a22955: function(arg0) {
                const ret = getObject(arg0).status;
                return ret;
            },
            __wbg_stopPropagation_bab00a24a73a0c56: function(arg0) {
                getObject(arg0).stopPropagation();
            },
            __wbg_stringify_26436a2becff957e: function(arg0, arg1) {
                const ret = JSON.stringify(getObject(arg1));
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_stringify_5ae93966a84901ac: function() { return handleError(function (arg0) {
                const ret = JSON.stringify(getObject(arg0));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_style_b01fc765f98b99ff: function(arg0) {
                const ret = getObject(arg0).style;
                return addHeapObject(ret);
            },
            __wbg_tagName_3bd06789ca771c23: function(arg0, arg1) {
                const ret = getObject(arg1).tagName;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_target_7bc90f314634b37b: function(arg0) {
                const ret = getObject(arg0).target;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_textContent_75b4506705c8c793: function(arg0, arg1) {
                const ret = getObject(arg1).textContent;
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_text_372f5b91442c50f9: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).text();
                return addHeapObject(ret);
            }, arguments); },
            __wbg_then_098abe61755d12f6: function(arg0, arg1) {
                const ret = getObject(arg0).then(getObject(arg1));
                return addHeapObject(ret);
            },
            __wbg_then_9e335f6dd892bc11: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
                return addHeapObject(ret);
            },
            __wbg_toString_fca8b5e46235cfb4: function(arg0) {
                const ret = getObject(arg0).toString();
                return addHeapObject(ret);
            },
            __wbg_top_378559f0b38a1038: function(arg0) {
                const ret = getObject(arg0).top;
                return ret;
            },
            __wbg_value_21fc78aab0322612: function(arg0) {
                const ret = getObject(arg0).value;
                return addHeapObject(ret);
            },
            __wbg_value_567d71719bef8eda: function(arg0, arg1) {
                const ret = getObject(arg1).value;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_value_6aa0a31ba8758e68: function(arg0, arg1) {
                const ret = getObject(arg1).value;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_view_f68a712e7315f8b2: function(arg0) {
                const ret = getObject(arg0).view;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_width_f933723cb0daf368: function(arg0) {
                const ret = getObject(arg0).width;
                return ret;
            },
            __wbg_x_5115ed72622c1ee9: function(arg0) {
                const ret = getObject(arg0).x;
                return ret;
            },
            __wbg_x_663bdb24f78fdb4f: function(arg0) {
                const ret = getObject(arg0).x;
                return ret;
            },
            __wbg_y_30a7c06266f44f65: function(arg0) {
                const ret = getObject(arg0).y;
                return ret;
            },
            __wbindgen_cast_0000000000000001: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { dtor_idx: 1281, function: Function { arguments: [NamedExternref("Event")], shim_idx: 1909, ret: Unit, inner_ret: Some(Unit) }, mutable: false }) -> Externref`.
                const ret = makeClosure(arg0, arg1, wasm.__wasm_bindgen_func_elem_6042, __wasm_bindgen_func_elem_9904);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000002: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { dtor_idx: 4774, function: Function { arguments: [NamedExternref("Node")], shim_idx: 4843, ret: U32, inner_ret: Some(U32) }, mutable: false }) -> Externref`.
                const ret = makeClosure(arg0, arg1, wasm.__wasm_bindgen_func_elem_18648, __wasm_bindgen_func_elem_19563);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000003: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { dtor_idx: 5931, function: Function { arguments: [NamedExternref("MouseEvent")], shim_idx: 5964, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm.__wasm_bindgen_func_elem_23761, __wasm_bindgen_func_elem_24673);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000004: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { dtor_idx: 6707, function: Function { arguments: [NamedExternref("Event")], shim_idx: 6709, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm.__wasm_bindgen_func_elem_28334, __wasm_bindgen_func_elem_28339);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000005: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { dtor_idx: 6707, function: Function { arguments: [], shim_idx: 6708, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm.__wasm_bindgen_func_elem_28334, __wasm_bindgen_func_elem_28338);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000006: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { dtor_idx: 6720, function: Function { arguments: [Externref], shim_idx: 6722, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm.__wasm_bindgen_func_elem_28492, __wasm_bindgen_func_elem_28585);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000007: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { dtor_idx: 6720, function: Function { arguments: [NamedExternref("Event")], shim_idx: 6722, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm.__wasm_bindgen_func_elem_28492, __wasm_bindgen_func_elem_28585_6);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000008: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { dtor_idx: 6720, function: Function { arguments: [], shim_idx: 6721, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm.__wasm_bindgen_func_elem_28492, __wasm_bindgen_func_elem_28584);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000009: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { dtor_idx: 6800, function: Function { arguments: [Externref], shim_idx: 6917, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm.__wasm_bindgen_func_elem_29366, __wasm_bindgen_func_elem_33263);
                return addHeapObject(ret);
            },
            __wbindgen_cast_000000000000000a: function(arg0) {
                // Cast intrinsic for `F64 -> Externref`.
                const ret = arg0;
                return addHeapObject(ret);
            },
            __wbindgen_cast_000000000000000b: function(arg0) {
                // Cast intrinsic for `I64 -> Externref`.
                const ret = arg0;
                return addHeapObject(ret);
            },
            __wbindgen_cast_000000000000000c: function(arg0, arg1) {
                // Cast intrinsic for `Ref(String) -> Externref`.
                const ret = getStringFromWasm0(arg0, arg1);
                return addHeapObject(ret);
            },
            __wbindgen_cast_000000000000000d: function(arg0) {
                // Cast intrinsic for `U64 -> Externref`.
                const ret = BigInt.asUintN(64, arg0);
                return addHeapObject(ret);
            },
            __wbindgen_object_clone_ref: function(arg0) {
                const ret = getObject(arg0);
                return addHeapObject(ret);
            },
            __wbindgen_object_drop_ref: function(arg0) {
                takeObject(arg0);
            },
        };
        return {
            __proto__: null,
            "./ftml_bg.js": import0,
        };
    }

    function __wasm_bindgen_func_elem_28338(arg0, arg1) {
        wasm.__wasm_bindgen_func_elem_28338(arg0, arg1);
    }

    function __wasm_bindgen_func_elem_28584(arg0, arg1) {
        wasm.__wasm_bindgen_func_elem_28584(arg0, arg1);
    }

    function __wasm_bindgen_func_elem_9904(arg0, arg1, arg2) {
        wasm.__wasm_bindgen_func_elem_9904(arg0, arg1, addHeapObject(arg2));
    }

    function __wasm_bindgen_func_elem_24673(arg0, arg1, arg2) {
        wasm.__wasm_bindgen_func_elem_24673(arg0, arg1, addHeapObject(arg2));
    }

    function __wasm_bindgen_func_elem_28339(arg0, arg1, arg2) {
        wasm.__wasm_bindgen_func_elem_28339(arg0, arg1, addHeapObject(arg2));
    }

    function __wasm_bindgen_func_elem_28585(arg0, arg1, arg2) {
        wasm.__wasm_bindgen_func_elem_28585(arg0, arg1, addHeapObject(arg2));
    }

    function __wasm_bindgen_func_elem_28585_6(arg0, arg1, arg2) {
        wasm.__wasm_bindgen_func_elem_28585_6(arg0, arg1, addHeapObject(arg2));
    }

    function __wasm_bindgen_func_elem_19563(arg0, arg1, arg2) {
        const ret = wasm.__wasm_bindgen_func_elem_19563(arg0, arg1, addHeapObject(arg2));
        return ret >>> 0;
    }

    function __wasm_bindgen_func_elem_33263(arg0, arg1, arg2) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wasm_bindgen_func_elem_33263(retptr, arg0, arg1, addHeapObject(arg2));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }

    function __wasm_bindgen_func_elem_33265(arg0, arg1, arg2, arg3) {
        wasm.__wasm_bindgen_func_elem_33265(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
    }


    const __wbindgen_enum_ReadableStreamType = ["bytes"];


    const __wbindgen_enum_ScrollBehavior = ["auto", "instant", "smooth"];


    const __wbindgen_enum_ScrollLogicalPosition = ["start", "center", "end", "nearest"];
    const IntoUnderlyingByteSourceFinalization = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingbytesource_free(ptr >>> 0, 1));
    const IntoUnderlyingSinkFinalization = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingsink_free(ptr >>> 0, 1));
    const IntoUnderlyingSourceFinalization = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingsource_free(ptr >>> 0, 1));

    function addHeapObject(obj) {
        if (heap_next === heap.length) heap.push(heap.length + 1);
        const idx = heap_next;
        heap_next = heap[idx];

        heap[idx] = obj;
        return idx;
    }

    const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(state => state.dtor(state.a, state.b));

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

    function dropObject(idx) {
        if (idx < 1028) return;
        heap[idx] = heap_next;
        heap_next = idx;
    }

    function getArrayU8FromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
    }

    let cachedDataViewMemory0 = null;
    function getDataViewMemory0() {
        if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
            cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
        }
        return cachedDataViewMemory0;
    }

    function getStringFromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return decodeText(ptr, len);
    }

    let cachedUint8ArrayMemory0 = null;
    function getUint8ArrayMemory0() {
        if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
            cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
        }
        return cachedUint8ArrayMemory0;
    }

    function getObject(idx) { return heap[idx]; }

    function handleError(f, args) {
        try {
            return f.apply(this, args);
        } catch (e) {
            wasm.__wbindgen_export3(addHeapObject(e));
        }
    }

    let heap = new Array(1024).fill(undefined);
    heap.push(undefined, null, true, false);

    let heap_next = heap.length;

    function isLikeNone(x) {
        return x === undefined || x === null;
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
                real._wbg_cb_unref();
            }
        };
        real._wbg_cb_unref = () => {
            if (--state.cnt === 0) {
                state.dtor(state.a, state.b);
                state.a = 0;
                CLOSURE_DTORS.unregister(state);
            }
        };
        CLOSURE_DTORS.register(real, state, state);
        return real;
    }

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
                state.a = a;
                real._wbg_cb_unref();
            }
        };
        real._wbg_cb_unref = () => {
            if (--state.cnt === 0) {
                state.dtor(state.a, state.b);
                state.a = 0;
                CLOSURE_DTORS.unregister(state);
            }
        };
        CLOSURE_DTORS.register(real, state, state);
        return real;
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

    function takeObject(idx) {
        const ret = getObject(idx);
        dropObject(idx);
        return ret;
    }

    let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
    cachedTextDecoder.decode();
    function decodeText(ptr, len) {
        return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
    }

    const cachedTextEncoder = new TextEncoder();

    if (!('encodeInto' in cachedTextEncoder)) {
        cachedTextEncoder.encodeInto = function (arg, view) {
            const buf = cachedTextEncoder.encode(arg);
            view.set(buf);
            return {
                read: arg.length,
                written: buf.length
            };
        };
    }

    let WASM_VECTOR_LEN = 0;

    let wasmModule, wasm;
    function __wbg_finalize_init(instance, module) {
        wasm = instance.exports;
        wasmModule = module;
        cachedDataViewMemory0 = null;
        cachedUint8ArrayMemory0 = null;
        wasm.__wbindgen_start();
        return wasm;
    }

    async function __wbg_load(module, imports) {
        if (typeof Response === 'function' && module instanceof Response) {
            if (typeof WebAssembly.instantiateStreaming === 'function') {
                try {
                    return await WebAssembly.instantiateStreaming(module, imports);
                } catch (e) {
                    const validResponse = module.ok && expectedResponseType(module.type);

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

        if (module_or_path === undefined && script_src !== undefined) {
            module_or_path = script_src.replace(/\.js$/, "_bg.wasm");
        }
        const imports = __wbg_get_imports();

        if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
            module_or_path = fetch(module_or_path);
        }

        const { instance, module } = await __wbg_load(await module_or_path, imports);

        return __wbg_finalize_init(instance, module);
    }

    return Object.assign(__wbg_init, { initSync }, exports);
})({ __proto__: null });
ftmlViewer();
