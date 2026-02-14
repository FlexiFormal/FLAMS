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
            __wbg_Error_8c4e43fe74559d73: function(arg0, arg1) {
                const ret = Error(getStringFromWasm0(arg0, arg1));
                return addHeapObject(ret);
            },
            __wbg_Number_04624de7d0e8332d: function(arg0) {
                const ret = Number(getObject(arg0));
                return ret;
            },
            __wbg___wbindgen_bigint_get_as_i64_8fcf4ce7f1ca72a2: function(arg0, arg1) {
                const v = getObject(arg1);
                const ret = typeof(v) === 'bigint' ? v : undefined;
                getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
            },
            __wbg___wbindgen_boolean_get_bbbb1c18aa2f5e25: function(arg0) {
                const v = getObject(arg0);
                const ret = typeof(v) === 'boolean' ? v : undefined;
                return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
            },
            __wbg___wbindgen_debug_string_0bc8482c6e3508ae: function(arg0, arg1) {
                const ret = debugString(getObject(arg1));
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg___wbindgen_in_47fa6863be6f2f25: function(arg0, arg1) {
                const ret = getObject(arg0) in getObject(arg1);
                return ret;
            },
            __wbg___wbindgen_is_bigint_31b12575b56f32fc: function(arg0) {
                const ret = typeof(getObject(arg0)) === 'bigint';
                return ret;
            },
            __wbg___wbindgen_is_falsy_e623e5b815413d00: function(arg0) {
                const ret = !getObject(arg0);
                return ret;
            },
            __wbg___wbindgen_is_function_0095a73b8b156f76: function(arg0) {
                const ret = typeof(getObject(arg0)) === 'function';
                return ret;
            },
            __wbg___wbindgen_is_null_ac34f5003991759a: function(arg0) {
                const ret = getObject(arg0) === null;
                return ret;
            },
            __wbg___wbindgen_is_object_5ae8e5880f2c1fbd: function(arg0) {
                const val = getObject(arg0);
                const ret = typeof(val) === 'object' && val !== null;
                return ret;
            },
            __wbg___wbindgen_is_string_cd444516edc5b180: function(arg0) {
                const ret = typeof(getObject(arg0)) === 'string';
                return ret;
            },
            __wbg___wbindgen_is_undefined_9e4d92534c42d778: function(arg0) {
                const ret = getObject(arg0) === undefined;
                return ret;
            },
            __wbg___wbindgen_jsval_eq_11888390b0186270: function(arg0, arg1) {
                const ret = getObject(arg0) === getObject(arg1);
                return ret;
            },
            __wbg___wbindgen_jsval_loose_eq_9dd77d8cd6671811: function(arg0, arg1) {
                const ret = getObject(arg0) == getObject(arg1);
                return ret;
            },
            __wbg___wbindgen_number_get_8ff4255516ccad3e: function(arg0, arg1) {
                const obj = getObject(arg1);
                const ret = typeof(obj) === 'number' ? obj : undefined;
                getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
            },
            __wbg___wbindgen_string_get_72fb696202c56729: function(arg0, arg1) {
                const obj = getObject(arg1);
                const ret = typeof(obj) === 'string' ? obj : undefined;
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg___wbindgen_throw_be289d5034ed271b: function(arg0, arg1) {
                throw new Error(getStringFromWasm0(arg0, arg1));
            },
            __wbg___wbindgen_try_into_number_07cd61098e837866: function(arg0) {
                let result;
                try { result = +getObject(arg0) } catch (e) { result = e }
                const ret = result;
                return addHeapObject(ret);
            },
            __wbg__wbg_cb_unref_d9b87ff7982e3b21: function(arg0) {
                getObject(arg0)._wbg_cb_unref();
            },
            __wbg_addEventListener_3acb0aad4483804c: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
            }, arguments); },
            __wbg_addEventListener_5ef04ffb1d3af066: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), arg4 !== 0);
            }, arguments); },
            __wbg_add_5be83378df680c25: function() { return handleError(function (arg0, arg1, arg2) {
                getObject(arg0).add(getStringFromWasm0(arg1, arg2));
            }, arguments); },
            __wbg_add_b1faef021d32ba26: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                getObject(arg0).add(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            }, arguments); },
            __wbg_altKey_73c1173ba53073d5: function(arg0) {
                const ret = getObject(arg0).altKey;
                return ret;
            },
            __wbg_appendChild_dea38765a26d346d: function() { return handleError(function (arg0, arg1) {
                const ret = getObject(arg0).appendChild(getObject(arg1));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_body_f67922363a220026: function(arg0) {
                const ret = getObject(arg0).body;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_bottom_c7ec510a18034add: function(arg0) {
                const ret = getObject(arg0).bottom;
                return ret;
            },
            __wbg_buffer_26d0910f3a5bc899: function(arg0) {
                const ret = getObject(arg0).buffer;
                return addHeapObject(ret);
            },
            __wbg_byobRequest_80e594e6da4e1af7: function(arg0) {
                const ret = getObject(arg0).byobRequest;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_byteLength_3417f266f4bf562a: function(arg0) {
                const ret = getObject(arg0).byteLength;
                return ret;
            },
            __wbg_byteOffset_f88547ca47c86358: function(arg0) {
                const ret = getObject(arg0).byteOffset;
                return ret;
            },
            __wbg_call_389efe28435a9388: function() { return handleError(function (arg0, arg1) {
                const ret = getObject(arg0).call(getObject(arg1));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_call_4708e0c13bdc8e95: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_cancelAnimationFrame_cd35895d78cf4510: function() { return handleError(function (arg0, arg1) {
                getObject(arg0).cancelAnimationFrame(arg1);
            }, arguments); },
            __wbg_cancelBubble_d93ec09e9c46cd6f: function(arg0) {
                const ret = getObject(arg0).cancelBubble;
                return ret;
            },
            __wbg_charCodeAt_8fdb057472688076: function(arg0, arg1) {
                const ret = getObject(arg0).charCodeAt(arg1 >>> 0);
                return ret;
            },
            __wbg_checked_04db83ac6810bc82: function(arg0) {
                const ret = getObject(arg0).checked;
                return ret;
            },
            __wbg_childNodes_75d35de33c9f6fbb: function(arg0) {
                const ret = getObject(arg0).childNodes;
                return addHeapObject(ret);
            },
            __wbg_classList_1a87c34c6d81421e: function(arg0) {
                const ret = getObject(arg0).classList;
                return addHeapObject(ret);
            },
            __wbg_clearTimeout_df03cf00269bc442: function(arg0, arg1) {
                getObject(arg0).clearTimeout(arg1);
            },
            __wbg_clientWidth_dcf89c40d88df4a3: function(arg0) {
                const ret = getObject(arg0).clientWidth;
                return ret;
            },
            __wbg_clientX_a3c5f4ff30e91264: function(arg0) {
                const ret = getObject(arg0).clientX;
                return ret;
            },
            __wbg_clientY_e28509acb9b4a42a: function(arg0) {
                const ret = getObject(arg0).clientY;
                return ret;
            },
            __wbg_cloneNode_b85e9102a9a31b29: function() { return handleError(function (arg0, arg1) {
                const ret = getObject(arg0).cloneNode(arg1 !== 0);
                return addHeapObject(ret);
            }, arguments); },
            __wbg_cloneNode_eaf4ea08ebf633a5: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).cloneNode();
                return addHeapObject(ret);
            }, arguments); },
            __wbg_close_06dfa0a815b9d71f: function() { return handleError(function (arg0) {
                getObject(arg0).close();
            }, arguments); },
            __wbg_close_a79afee31de55b36: function() { return handleError(function (arg0) {
                getObject(arg0).close();
            }, arguments); },
            __wbg_code_dee0dae4730408e1: function(arg0, arg1) {
                const ret = getObject(arg1).code;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_commonAncestorContainer_5ecc20ee886193ef: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).commonAncestorContainer;
                return addHeapObject(ret);
            }, arguments); },
            __wbg_composedPath_9154ab2547c668d5: function(arg0) {
                const ret = getObject(arg0).composedPath();
                return addHeapObject(ret);
            },
            __wbg_construct_86626e847de3b629: function() { return handleError(function (arg0, arg1) {
                const ret = Reflect.construct(getObject(arg0), getObject(arg1));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_contains_6f4d5df35ef8a13a: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).contains(getStringFromWasm0(arg1, arg2));
                return ret;
            },
            __wbg_content_681ebf067b179491: function(arg0) {
                const ret = getObject(arg0).content;
                return addHeapObject(ret);
            },
            __wbg_createComment_b783f49934771bb3: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).createComment(getStringFromWasm0(arg1, arg2));
                return addHeapObject(ret);
            },
            __wbg_createElementNS_ee00621496b30ec2: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                const ret = getObject(arg0).createElementNS(arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_createElement_49f60fdcaae809c8: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_createTextNode_55029686c9591bf3: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).createTextNode(getStringFromWasm0(arg1, arg2));
                return addHeapObject(ret);
            },
            __wbg_createTreeWalker_63b4e1ab3eb463c8: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                const ret = getObject(arg0).createTreeWalker(getObject(arg1), arg2 >>> 0, getObject(arg3));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_ctrlKey_09a1b54d77dea92b: function(arg0) {
                const ret = getObject(arg0).ctrlKey;
                return ret;
            },
            __wbg_deleteProperty_8c8a05da881fea59: function() { return handleError(function (arg0, arg1) {
                const ret = Reflect.deleteProperty(getObject(arg0), getObject(arg1));
                return ret;
            }, arguments); },
            __wbg_documentElement_723733f86794182a: function(arg0) {
                const ret = getObject(arg0).documentElement;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_document_ee35a3d3ae34ef6c: function(arg0) {
                const ret = getObject(arg0).document;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_done_57b39ecd9addfe81: function(arg0) {
                const ret = getObject(arg0).done;
                return ret;
            },
            __wbg_enqueue_2c63f2044f257c3e: function() { return handleError(function (arg0, arg1) {
                getObject(arg0).enqueue(getObject(arg1));
            }, arguments); },
            __wbg_entries_58c7934c745daac7: function(arg0) {
                const ret = Object.entries(getObject(arg0));
                return addHeapObject(ret);
            },
            __wbg_error_7534b8e9a36f1ab4: function(arg0, arg1) {
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
            __wbg_error_9a7fe3f932034cde: function(arg0) {
                console.error(getObject(arg0));
            },
            __wbg_exec_48e0e0ad953102ac: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).exec(getStringFromWasm0(arg1, arg2));
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_fetch_e6e8e0a221783759: function(arg0, arg1) {
                const ret = getObject(arg0).fetch(getObject(arg1));
                return addHeapObject(ret);
            },
            __wbg_firstChild_2950111f6da7246c: function(arg0) {
                const ret = getObject(arg0).firstChild;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_firstChild_7abc8583e4ab2f1f: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).firstChild();
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_firstElementChild_f1c371cb2a4d5101: function(arg0) {
                const ret = getObject(arg0).firstElementChild;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_focus_128ff465f65677cc: function() { return handleError(function (arg0) {
                getObject(arg0).focus();
            }, arguments); },
            __wbg_fromEntries_7fb5bc874dbe50d5: function() { return handleError(function (arg0) {
                const ret = Object.fromEntries(getObject(arg0));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_fullscreenElement_25b445e2961e68ba: function(arg0) {
                const ret = getObject(arg0).fullscreenElement;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_getAttributeNames_4f000c5e47b26293: function(arg0) {
                const ret = getObject(arg0).getAttributeNames();
                return addHeapObject(ret);
            },
            __wbg_getAttribute_b9f6fc4b689c71b0: function(arg0, arg1, arg2, arg3) {
                const ret = getObject(arg1).getAttribute(getStringFromWasm0(arg2, arg3));
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_getBoundingClientRect_b5c8c34d07878818: function(arg0) {
                const ret = getObject(arg0).getBoundingClientRect();
                return addHeapObject(ret);
            },
            __wbg_getComputedStyle_2d1f9dfe4ee7e0b9: function() { return handleError(function (arg0, arg1) {
                const ret = getObject(arg0).getComputedStyle(getObject(arg1));
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_getElementById_e34377b79d7285f6: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).getElementById(getStringFromWasm0(arg1, arg2));
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_getItem_0c792d344808dcf5: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                const ret = getObject(arg1).getItem(getStringFromWasm0(arg2, arg3));
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            }, arguments); },
            __wbg_getPropertyValue_d6911b2a1f9acba9: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                const ret = getObject(arg1).getPropertyValue(getStringFromWasm0(arg2, arg3));
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            }, arguments); },
            __wbg_getRandomValues_9c5c1b115e142bb8: function() { return handleError(function (arg0, arg1) {
                globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
            }, arguments); },
            __wbg_getRangeAt_d6b398d5eb6c633f: function() { return handleError(function (arg0, arg1) {
                const ret = getObject(arg0).getRangeAt(arg1 >>> 0);
                return addHeapObject(ret);
            }, arguments); },
            __wbg_getSelection_d345357cda220f17: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).getSelection();
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_getTime_1e3cd1391c5c3995: function(arg0) {
                const ret = getObject(arg0).getTime();
                return ret;
            },
            __wbg_getTimezoneOffset_81776d10a4ec18a8: function(arg0) {
                const ret = getObject(arg0).getTimezoneOffset();
                return ret;
            },
            __wbg_get_0bdeda968867e10e: function() { return handleError(function (arg0, arg1) {
                const ret = Reflect.get(getObject(arg0), arg1 >>> 0);
                return addHeapObject(ret);
            }, arguments); },
            __wbg_get_9b94d73e6221f75c: function(arg0, arg1) {
                const ret = getObject(arg0)[arg1 >>> 0];
                return addHeapObject(ret);
            },
            __wbg_get_b3ed3ad4be2bc8ac: function() { return handleError(function (arg0, arg1) {
                const ret = Reflect.get(getObject(arg0), getObject(arg1));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_get_d8db2ad31d529ff8: function(arg0, arg1) {
                const ret = getObject(arg0)[arg1 >>> 0];
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_get_with_ref_key_1dc361bd10053bfe: function(arg0, arg1) {
                const ret = getObject(arg0)[getObject(arg1)];
                return addHeapObject(ret);
            },
            __wbg_hash_90eadad0e1447454: function() { return handleError(function (arg0, arg1) {
                const ret = getObject(arg1).hash;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            }, arguments); },
            __wbg_head_a64c2648b30c3faf: function(arg0) {
                const ret = getObject(arg0).head;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_height_45209601b4c4ede6: function(arg0) {
                const ret = getObject(arg0).height;
                return ret;
            },
            __wbg_host_fb29f8be35c2517d: function(arg0) {
                const ret = getObject(arg0).host;
                return addHeapObject(ret);
            },
            __wbg_id_ff64a5892a30d4e9: function(arg0, arg1) {
                const ret = getObject(arg1).id;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_includes_32215c836f1cd3fb: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).includes(getObject(arg1), arg2);
                return ret;
            },
            __wbg_innerHeight_54aa104da08becd2: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).innerHeight;
                return addHeapObject(ret);
            }, arguments); },
            __wbg_innerWidth_fa95c57321f4f033: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).innerWidth;
                return addHeapObject(ret);
            }, arguments); },
            __wbg_insertBefore_1468142836e61a51: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = getObject(arg0).insertBefore(getObject(arg1), getObject(arg2));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_instanceof_ArrayBuffer_c367199e2fa2aa04: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof ArrayBuffer;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Element_9e662f49ab6c6beb: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof Element;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Error_8573fe0b0b480f46: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof Error;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_HtmlElement_5abfac207260fd6f: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof HTMLElement;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_HtmlInputElement_c10b7260b4e0710a: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof HTMLInputElement;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_KeyboardEvent_ac14ca88fa76d153: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof KeyboardEvent;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Map_53af74335dec57f4: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof Map;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Node_da04bd8df43deba3: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof Node;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_RegExp_4f608a74aace1a6a: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof RegExp;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Response_ee1d54d79ae41977: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof Response;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_ShadowRoot_5285adde3587c73e: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof ShadowRoot;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Uint8Array_9b9075935c74707c: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof Uint8Array;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Window_ed49b2db8df90359: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof Window;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_isArray_a2cef7634fcb7c0d: function(arg0) {
                const ret = Array.isArray(getObject(arg0));
                return ret;
            },
            __wbg_isArray_d314bb98fcf08331: function(arg0) {
                const ret = Array.isArray(getObject(arg0));
                return ret;
            },
            __wbg_isSafeInteger_bfbc7332a9768d2a: function(arg0) {
                const ret = Number.isSafeInteger(getObject(arg0));
                return ret;
            },
            __wbg_is_f29129f676e5410c: function(arg0, arg1) {
                const ret = Object.is(getObject(arg0), getObject(arg1));
                return ret;
            },
            __wbg_iterator_6ff6560ca1568e55: function() {
                const ret = Symbol.iterator;
                return addHeapObject(ret);
            },
            __wbg_json_d214c3d336140979: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).json();
                return addHeapObject(ret);
            }, arguments); },
            __wbg_keyCode_155291a11654466e: function(arg0) {
                const ret = getObject(arg0).keyCode;
                return ret;
            },
            __wbg_key_d41e8e825e6bb0e9: function(arg0, arg1) {
                const ret = getObject(arg1).key;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_lastChild_c0f0b509f591d7bc: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).lastChild();
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_left_3b7c3c1030d5ca7a: function(arg0) {
                const ret = getObject(arg0).left;
                return ret;
            },
            __wbg_length_32ed9a279acd054c: function(arg0) {
                const ret = getObject(arg0).length;
                return ret;
            },
            __wbg_length_35a7bace40f36eac: function(arg0) {
                const ret = getObject(arg0).length;
                return ret;
            },
            __wbg_length_68dc7c5cf1b6d349: function(arg0) {
                const ret = getObject(arg0).length;
                return ret;
            },
            __wbg_localStorage_a22d31b9eacc4594: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).localStorage;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_location_df7ca06c93e51763: function(arg0) {
                const ret = getObject(arg0).location;
                return addHeapObject(ret);
            },
            __wbg_log_0cc1b7768397bcfe: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
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
            __wbg_log_6b5ca2e6124b2808: function(arg0) {
                console.log(getObject(arg0));
            },
            __wbg_log_cb9e190acc5753fb: function(arg0, arg1) {
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
            __wbg_mark_7438147ce31e9d4b: function(arg0, arg1) {
                performance.mark(getStringFromWasm0(arg0, arg1));
            },
            __wbg_measure_fb7825c11612c823: function() { return handleError(function (arg0, arg1, arg2, arg3) {
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
            __wbg_message_9ddc4b9a62a7c379: function(arg0) {
                const ret = getObject(arg0).message;
                return addHeapObject(ret);
            },
            __wbg_metaKey_67113fb40365d736: function(arg0) {
                const ret = getObject(arg0).metaKey;
                return ret;
            },
            __wbg_name_446e25ef2cfdab5a: function(arg0) {
                const ret = getObject(arg0).name;
                return addHeapObject(ret);
            },
            __wbg_new_0_73afc35eb544e539: function() {
                const ret = new Date();
                return addHeapObject(ret);
            },
            __wbg_new_245cd5c49157e602: function(arg0) {
                const ret = new Date(getObject(arg0));
                return addHeapObject(ret);
            },
            __wbg_new_361308b2356cecd0: function() {
                const ret = new Object();
                return addHeapObject(ret);
            },
            __wbg_new_64284bd487f9d239: function() { return handleError(function () {
                const ret = new Headers();
                return addHeapObject(ret);
            }, arguments); },
            __wbg_new_72b49615380db768: function(arg0, arg1) {
                const ret = new Error(getStringFromWasm0(arg0, arg1));
                return addHeapObject(ret);
            },
            __wbg_new_8a6f238a6ece86ea: function() {
                const ret = new Error();
                return addHeapObject(ret);
            },
            __wbg_new_b5d9e2fb389fef91: function(arg0, arg1) {
                try {
                    var state0 = {a: arg0, b: arg1};
                    var cb0 = (arg0, arg1) => {
                        const a = state0.a;
                        state0.a = 0;
                        try {
                            return __wasm_bindgen_func_elem_31291(a, state0.b, arg0, arg1);
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
            __wbg_new_dd2b680c8bf6ae29: function(arg0) {
                const ret = new Uint8Array(getObject(arg0));
                return addHeapObject(ret);
            },
            __wbg_new_de07934a2f24c2ec: function(arg0, arg1, arg2, arg3) {
                const ret = new RegExp(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
                return addHeapObject(ret);
            },
            __wbg_new_no_args_1c7c842f08d00ebb: function(arg0, arg1) {
                const ret = new Function(getStringFromWasm0(arg0, arg1));
                return addHeapObject(ret);
            },
            __wbg_new_with_args_7bba34e94b1cfa40: function(arg0, arg1, arg2, arg3) {
                const ret = new Function(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
                return addHeapObject(ret);
            },
            __wbg_new_with_byte_offset_and_length_aa261d9c9da49eb1: function(arg0, arg1, arg2) {
                const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
                return addHeapObject(ret);
            },
            __wbg_new_with_length_1763c527b2923202: function(arg0) {
                const ret = new Array(arg0 >>> 0);
                return addHeapObject(ret);
            },
            __wbg_new_with_str_and_init_a61cbc6bdef21614: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = new Request(getStringFromWasm0(arg0, arg1), getObject(arg2));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_new_with_year_month_day_hr_min_sec_f82362c71c4dfc23: function(arg0, arg1, arg2, arg3, arg4, arg5) {
                const ret = new Date(arg0 >>> 0, arg1, arg2, arg3, arg4, arg5);
                return addHeapObject(ret);
            },
            __wbg_nextNode_a36904c0012eddf8: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).nextNode();
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_nextSibling_2e988d9bbe3e06f0: function(arg0) {
                const ret = getObject(arg0).nextSibling;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_next_3482f54c49e8af19: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).next();
                return addHeapObject(ret);
            }, arguments); },
            __wbg_next_418f80d8f5303233: function(arg0) {
                const ret = getObject(arg0).next;
                return addHeapObject(ret);
            },
            __wbg_nodeType_1a77807cb3800514: function(arg0) {
                const ret = getObject(arg0).nodeType;
                return ret;
            },
            __wbg_offsetHeight_34f7abc1686733cc: function(arg0) {
                const ret = getObject(arg0).offsetHeight;
                return ret;
            },
            __wbg_offsetWidth_f37b33a53e513101: function(arg0) {
                const ret = getObject(arg0).offsetWidth;
                return ret;
            },
            __wbg_outerHTML_baa741c8917e0c70: function(arg0, arg1) {
                const ret = getObject(arg1).outerHTML;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_ownKeys_c7100fb5fa376c6f: function() { return handleError(function (arg0) {
                const ret = Reflect.ownKeys(getObject(arg0));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_parentElement_75863410a8617953: function(arg0) {
                const ret = getObject(arg0).parentElement;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_parentNode_d44bd5ec58601e45: function(arg0) {
                const ret = getObject(arg0).parentNode;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_parseFloat_e6e8a128ed3db65d: function(arg0, arg1) {
                const ret = Number.parseFloat(getStringFromWasm0(arg0, arg1));
                return ret;
            },
            __wbg_prepend_12ed960547d9dfe1: function() { return handleError(function (arg0, arg1) {
                getObject(arg0).prepend(getObject(arg1));
            }, arguments); },
            __wbg_preventDefault_cdcfcd7e301b9702: function(arg0) {
                getObject(arg0).preventDefault();
            },
            __wbg_previousNode_2d518665943f9aeb: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).previousNode();
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_prototypesetcall_bdcdcc5842e4d77d: function(arg0, arg1, arg2) {
                Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), getObject(arg2));
            },
            __wbg_querySelector_1be4292f202b1597: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = getObject(arg0).querySelector(getStringFromWasm0(arg1, arg2));
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_querySelector_c3b0df2d58eec220: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = getObject(arg0).querySelector(getStringFromWasm0(arg1, arg2));
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_queueMicrotask_0aa0a927f78f5d98: function(arg0) {
                const ret = getObject(arg0).queueMicrotask;
                return addHeapObject(ret);
            },
            __wbg_queueMicrotask_5bb536982f78a56f: function(arg0) {
                queueMicrotask(getObject(arg0));
            },
            __wbg_readyState_cf11f0728fc7b46c: function(arg0, arg1) {
                const ret = getObject(arg1).readyState;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_removeAttribute_87259aab06d9f286: function() { return handleError(function (arg0, arg1, arg2) {
                getObject(arg0).removeAttribute(getStringFromWasm0(arg1, arg2));
            }, arguments); },
            __wbg_removeEventListener_0c0902ed5009dd9f: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), arg4 !== 0);
            }, arguments); },
            __wbg_removeEventListener_e63328781a5b9af9: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
            }, arguments); },
            __wbg_removeItem_f6369b1a6fa39850: function() { return handleError(function (arg0, arg1, arg2) {
                getObject(arg0).removeItem(getStringFromWasm0(arg1, arg2));
            }, arguments); },
            __wbg_removeProperty_a0d2ff8a76ffd2b1: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                const ret = getObject(arg1).removeProperty(getStringFromWasm0(arg2, arg3));
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            }, arguments); },
            __wbg_remove_31c39325eee968fc: function(arg0) {
                getObject(arg0).remove();
            },
            __wbg_remove_85daa9b3dddf4e6d: function(arg0) {
                getObject(arg0).remove();
            },
            __wbg_remove_e5f8ef4fa8fb0b71: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                getObject(arg0).remove(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            }, arguments); },
            __wbg_remove_f9451697e0bc6ca0: function() { return handleError(function (arg0, arg1, arg2) {
                getObject(arg0).remove(getStringFromWasm0(arg1, arg2));
            }, arguments); },
            __wbg_requestAnimationFrame_43682f8e1c5e5348: function() { return handleError(function (arg0, arg1) {
                const ret = getObject(arg0).requestAnimationFrame(getObject(arg1));
                return ret;
            }, arguments); },
            __wbg_requestFullscreen_5ecd19871639369d: function() { return handleError(function (arg0) {
                getObject(arg0).requestFullscreen();
            }, arguments); },
            __wbg_resolve_002c4b7d9d8f6b64: function(arg0) {
                const ret = Promise.resolve(getObject(arg0));
                return addHeapObject(ret);
            },
            __wbg_respond_bf6ab10399ca8722: function() { return handleError(function (arg0, arg1) {
                getObject(arg0).respond(arg1 >>> 0);
            }, arguments); },
            __wbg_right_154af6c2b1bf0c89: function(arg0) {
                const ret = getObject(arg0).right;
                return ret;
            },
            __wbg_root_ffdfe1ec2e4216ac: function(arg0) {
                const ret = getObject(arg0).root;
                return addHeapObject(ret);
            },
            __wbg_scrollIntoView_10646525aff3911a: function(arg0, arg1) {
                getObject(arg0).scrollIntoView(getObject(arg1));
            },
            __wbg_scrollIntoView_b1a18b195d281d73: function(arg0, arg1) {
                getObject(arg0).scrollIntoView(arg1 !== 0);
            },
            __wbg_scrollLeft_2b817c7719d17438: function(arg0) {
                const ret = getObject(arg0).scrollLeft;
                return ret;
            },
            __wbg_scrollTop_0a3a77f9fcbe038e: function(arg0) {
                const ret = getObject(arg0).scrollTop;
                return ret;
            },
            __wbg_scrollX_b3151cb810a681ae: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).scrollX;
                return ret;
            }, arguments); },
            __wbg_scrollY_8087997adf618f94: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).scrollY;
                return ret;
            }, arguments); },
            __wbg_setAttribute_cc8e4c8a2a008508: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            }, arguments); },
            __wbg_setItem_cf340bb2edbd3089: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                getObject(arg0).setItem(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            }, arguments); },
            __wbg_setProperty_cbb25c4e74285b39: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                getObject(arg0).setProperty(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            }, arguments); },
            __wbg_setTimeout_eff32631ea138533: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = getObject(arg0).setTimeout(getObject(arg1), arg2);
                return ret;
            }, arguments); },
            __wbg_set_6cb8631f80447a67: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
                return ret;
            }, arguments); },
            __wbg_set_accept_node_94e862534b4f77e5: function(arg0, arg1) {
                getObject(arg0).acceptNode = getObject(arg1);
            },
            __wbg_set_behavior_95b5c7eaefc26d7f: function(arg0, arg1) {
                getObject(arg0).behavior = __wbindgen_enum_ScrollBehavior[arg1];
            },
            __wbg_set_block_607a9575144934d2: function(arg0, arg1) {
                getObject(arg0).block = __wbindgen_enum_ScrollLogicalPosition[arg1];
            },
            __wbg_set_body_9a7e00afe3cfe244: function(arg0, arg1) {
                getObject(arg0).body = getObject(arg1);
            },
            __wbg_set_cc56eefd2dd91957: function(arg0, arg1, arg2) {
                getObject(arg0).set(getArrayU8FromWasm0(arg1, arg2));
            },
            __wbg_set_currentNode_f739cbfff8aa6251: function(arg0, arg1) {
                getObject(arg0).currentNode = getObject(arg1);
            },
            __wbg_set_f43e577aea94465b: function(arg0, arg1, arg2) {
                getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
            },
            __wbg_set_headers_cfc5f4b2c1f20549: function(arg0, arg1) {
                getObject(arg0).headers = getObject(arg1);
            },
            __wbg_set_innerHTML_edd39677e3460291: function(arg0, arg1, arg2) {
                getObject(arg0).innerHTML = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_method_c3e20375f5ae7fac: function(arg0, arg1, arg2) {
                getObject(arg0).method = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_nodeValue_d947eb0a476b80d7: function(arg0, arg1, arg2) {
                getObject(arg0).nodeValue = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_open_8d533f83e089afb7: function(arg0, arg1) {
                getObject(arg0).open = arg1 !== 0;
            },
            __wbg_set_scrollLeft_8de8fc187e3a6808: function(arg0, arg1) {
                getObject(arg0).scrollLeft = arg1;
            },
            __wbg_set_scrollTop_bebe746cd217a3d1: function(arg0, arg1) {
                getObject(arg0).scrollTop = arg1;
            },
            __wbg_set_textContent_3e87dba095d9cdbc: function(arg0, arg1, arg2) {
                getObject(arg0).textContent = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_value_c3556fce049236f0: function(arg0, arg1, arg2) {
                getObject(arg0).value = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_x_4e3dacaea3450c2e: function(arg0, arg1) {
                getObject(arg0).x = arg1;
            },
            __wbg_set_y_33b8b9adcd4b7d2a: function(arg0, arg1) {
                getObject(arg0).y = arg1;
            },
            __wbg_slice_b0fa09b1e0041d42: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).slice(arg1 >>> 0, arg2 >>> 0);
                return addHeapObject(ret);
            },
            __wbg_stack_0ed75d68575b0f3c: function(arg0, arg1) {
                const ret = getObject(arg1).stack;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_static_accessor_GLOBAL_12837167ad935116: function() {
                const ret = typeof global === 'undefined' ? null : global;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_static_accessor_GLOBAL_THIS_e628e89ab3b1c95f: function() {
                const ret = typeof globalThis === 'undefined' ? null : globalThis;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_static_accessor_SELF_a621d3dfbb60d0ce: function() {
                const ret = typeof self === 'undefined' ? null : self;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_static_accessor_WINDOW_f8727f0cf888e0bd: function() {
                const ret = typeof window === 'undefined' ? null : window;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_status_89d7e803db911ee7: function(arg0) {
                const ret = getObject(arg0).status;
                return ret;
            },
            __wbg_stopPropagation_6e5e2a085214ac63: function(arg0) {
                getObject(arg0).stopPropagation();
            },
            __wbg_stringify_8d1cc6ff383e8bae: function() { return handleError(function (arg0) {
                const ret = JSON.stringify(getObject(arg0));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_stringify_e4a940b133e6b7d8: function(arg0, arg1) {
                const ret = JSON.stringify(getObject(arg1));
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_style_0b7c9bd318f8b807: function(arg0) {
                const ret = getObject(arg0).style;
                return addHeapObject(ret);
            },
            __wbg_tagName_0cf6d7b647352f04: function(arg0, arg1) {
                const ret = getObject(arg1).tagName;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_target_521be630ab05b11e: function(arg0) {
                const ret = getObject(arg0).target;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_textContent_fc823fb432e90037: function(arg0, arg1) {
                const ret = getObject(arg1).textContent;
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_text_083b8727c990c8c0: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).text();
                return addHeapObject(ret);
            }, arguments); },
            __wbg_then_0d9fe2c7b1857d32: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
                return addHeapObject(ret);
            },
            __wbg_then_b9e7b3b5f1a9e1b5: function(arg0, arg1) {
                const ret = getObject(arg0).then(getObject(arg1));
                return addHeapObject(ret);
            },
            __wbg_toString_029ac24421fd7a24: function(arg0) {
                const ret = getObject(arg0).toString();
                return addHeapObject(ret);
            },
            __wbg_top_3d27ff6f468cf3fc: function(arg0) {
                const ret = getObject(arg0).top;
                return ret;
            },
            __wbg_value_0546255b415e96c1: function(arg0) {
                const ret = getObject(arg0).value;
                return addHeapObject(ret);
            },
            __wbg_value_d402dce7dcb16251: function(arg0, arg1) {
                const ret = getObject(arg1).value;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_value_e506a07878790ca0: function(arg0, arg1) {
                const ret = getObject(arg1).value;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_view_6c32e7184b8606ad: function(arg0) {
                const ret = getObject(arg0).view;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_width_ae46cb8e98ee102f: function(arg0) {
                const ret = getObject(arg0).width;
                return ret;
            },
            __wbg_x_1c3a1279b05ec817: function(arg0) {
                const ret = getObject(arg0).x;
                return ret;
            },
            __wbg_x_95222ef76724a332: function(arg0) {
                const ret = getObject(arg0).x;
                return ret;
            },
            __wbg_y_0b4e7ff7d5c0a5d7: function(arg0) {
                const ret = getObject(arg0).y;
                return ret;
            },
            __wbindgen_cast_0000000000000001: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { dtor_idx: 3871, function: Function { arguments: [NamedExternref("Event")], shim_idx: 3965, ret: Unit, inner_ret: Some(Unit) }, mutable: false }) -> Externref`.
                const ret = makeClosure(arg0, arg1, wasm.__wasm_bindgen_func_elem_14426, __wasm_bindgen_func_elem_14979);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000002: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { dtor_idx: 4334, function: Function { arguments: [NamedExternref("Node")], shim_idx: 4462, ret: U32, inner_ret: Some(U32) }, mutable: false }) -> Externref`.
                const ret = makeClosure(arg0, arg1, wasm.__wasm_bindgen_func_elem_17291, __wasm_bindgen_func_elem_18235);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000003: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { dtor_idx: 5480, function: Function { arguments: [NamedExternref("MouseEvent")], shim_idx: 5544, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm.__wasm_bindgen_func_elem_22320, __wasm_bindgen_func_elem_23164);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000004: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { dtor_idx: 6157, function: Function { arguments: [NamedExternref("Event")], shim_idx: 6158, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm.__wasm_bindgen_func_elem_26557, __wasm_bindgen_func_elem_26562);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000005: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { dtor_idx: 6157, function: Function { arguments: [], shim_idx: 6159, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm.__wasm_bindgen_func_elem_26557, __wasm_bindgen_func_elem_26561);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000006: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { dtor_idx: 6168, function: Function { arguments: [NamedExternref("Event")], shim_idx: 6169, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm.__wasm_bindgen_func_elem_26709, __wasm_bindgen_func_elem_26800);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000007: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { dtor_idx: 6168, function: Function { arguments: [], shim_idx: 6170, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm.__wasm_bindgen_func_elem_26709, __wasm_bindgen_func_elem_26799);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000008: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { dtor_idx: 6248, function: Function { arguments: [Externref], shim_idx: 6249, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm.__wasm_bindgen_func_elem_27558, __wasm_bindgen_func_elem_27575);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000009: function(arg0) {
                // Cast intrinsic for `F64 -> Externref`.
                const ret = arg0;
                return addHeapObject(ret);
            },
            __wbindgen_cast_000000000000000a: function(arg0) {
                // Cast intrinsic for `I64 -> Externref`.
                const ret = arg0;
                return addHeapObject(ret);
            },
            __wbindgen_cast_000000000000000b: function(arg0, arg1) {
                // Cast intrinsic for `Ref(String) -> Externref`.
                const ret = getStringFromWasm0(arg0, arg1);
                return addHeapObject(ret);
            },
            __wbindgen_cast_000000000000000c: function(arg0) {
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

    function __wasm_bindgen_func_elem_26561(arg0, arg1) {
        wasm.__wasm_bindgen_func_elem_26561(arg0, arg1);
    }

    function __wasm_bindgen_func_elem_26799(arg0, arg1) {
        wasm.__wasm_bindgen_func_elem_26799(arg0, arg1);
    }

    function __wasm_bindgen_func_elem_14979(arg0, arg1, arg2) {
        wasm.__wasm_bindgen_func_elem_14979(arg0, arg1, addHeapObject(arg2));
    }

    function __wasm_bindgen_func_elem_23164(arg0, arg1, arg2) {
        wasm.__wasm_bindgen_func_elem_23164(arg0, arg1, addHeapObject(arg2));
    }

    function __wasm_bindgen_func_elem_26562(arg0, arg1, arg2) {
        wasm.__wasm_bindgen_func_elem_26562(arg0, arg1, addHeapObject(arg2));
    }

    function __wasm_bindgen_func_elem_26800(arg0, arg1, arg2) {
        wasm.__wasm_bindgen_func_elem_26800(arg0, arg1, addHeapObject(arg2));
    }

    function __wasm_bindgen_func_elem_27575(arg0, arg1, arg2) {
        wasm.__wasm_bindgen_func_elem_27575(arg0, arg1, addHeapObject(arg2));
    }

    function __wasm_bindgen_func_elem_18235(arg0, arg1, arg2) {
        const ret = wasm.__wasm_bindgen_func_elem_18235(arg0, arg1, addHeapObject(arg2));
        return ret >>> 0;
    }

    function __wasm_bindgen_func_elem_31291(arg0, arg1, arg2, arg3) {
        wasm.__wasm_bindgen_func_elem_31291(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
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
        if (idx < 132) return;
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

    let heap = new Array(128).fill(undefined);
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
