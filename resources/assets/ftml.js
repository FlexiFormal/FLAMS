let ftmlViewer = (function(exports) {
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
            __wbg_Error_92b29b0548f8b746: function(arg0, arg1) {
                const ret = Error(getStringFromWasm0(arg0, arg1));
                return addHeapObject(ret);
            },
            __wbg_Number_9a4e0ecb0fa16705: function(arg0) {
                const ret = Number(getObject(arg0));
                return ret;
            },
            __wbg___wbindgen_bigint_get_as_i64_d968e41184ae354f: function(arg0, arg1) {
                const v = getObject(arg1);
                const ret = typeof(v) === 'bigint' ? v : undefined;
                getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
            },
            __wbg___wbindgen_boolean_get_fa956cfa2d1bd751: function(arg0) {
                const v = getObject(arg0);
                const ret = typeof(v) === 'boolean' ? v : undefined;
                return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
            },
            __wbg___wbindgen_debug_string_c25d447a39f5578f: function(arg0, arg1) {
                const ret = debugString(getObject(arg1));
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg___wbindgen_in_aca499c5de7ff5e5: function(arg0, arg1) {
                const ret = getObject(arg0) in getObject(arg1);
                return ret;
            },
            __wbg___wbindgen_is_bigint_2f76dc55065b4273: function(arg0) {
                const ret = typeof(getObject(arg0)) === 'bigint';
                return ret;
            },
            __wbg___wbindgen_is_falsy_a6dfe792ff282f10: function(arg0) {
                const ret = !getObject(arg0);
                return ret;
            },
            __wbg___wbindgen_is_function_1ff95bcc5517c252: function(arg0) {
                const ret = typeof(getObject(arg0)) === 'function';
                return ret;
            },
            __wbg___wbindgen_is_null_ea9085d691f535d3: function(arg0) {
                const ret = getObject(arg0) === null;
                return ret;
            },
            __wbg___wbindgen_is_object_a27215656b807791: function(arg0) {
                const val = getObject(arg0);
                const ret = typeof(val) === 'object' && val !== null;
                return ret;
            },
            __wbg___wbindgen_is_string_ea5e6cc2e4141dfe: function(arg0) {
                const ret = typeof(getObject(arg0)) === 'string';
                return ret;
            },
            __wbg___wbindgen_is_undefined_c05833b95a3cf397: function(arg0) {
                const ret = getObject(arg0) === undefined;
                return ret;
            },
            __wbg___wbindgen_jsval_eq_e659fcf7b0e32763: function(arg0, arg1) {
                const ret = getObject(arg0) === getObject(arg1);
                return ret;
            },
            __wbg___wbindgen_jsval_loose_eq_db4c3b15f63fc170: function(arg0, arg1) {
                const ret = getObject(arg0) == getObject(arg1);
                return ret;
            },
            __wbg___wbindgen_number_get_394265ed1e1b84ee: function(arg0, arg1) {
                const obj = getObject(arg1);
                const ret = typeof(obj) === 'number' ? obj : undefined;
                getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
            },
            __wbg___wbindgen_string_get_b0ca35b86a603356: function(arg0, arg1) {
                const obj = getObject(arg1);
                const ret = typeof(obj) === 'string' ? obj : undefined;
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg___wbindgen_throw_344f42d3211c4765: function(arg0, arg1) {
                throw new Error(getStringFromWasm0(arg0, arg1));
            },
            __wbg__wbg_cb_unref_fffb441def202758: function(arg0) {
                getObject(arg0)._wbg_cb_unref();
            },
            __wbg_addEventListener_109ae44e5cc4d506: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), arg4 !== 0);
            }, arguments); },
            __wbg_addEventListener_d85450ee1320c989: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
            }, arguments); },
            __wbg_add_04124418b84abf5a: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                getObject(arg0).add(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            }, arguments); },
            __wbg_add_38cee25662852903: function() { return handleError(function (arg0, arg1, arg2) {
                getObject(arg0).add(getStringFromWasm0(arg1, arg2));
            }, arguments); },
            __wbg_altKey_50f830d1793a2eea: function(arg0) {
                const ret = getObject(arg0).altKey;
                return ret;
            },
            __wbg_appendChild_f553e8704c4f14a6: function() { return handleError(function (arg0, arg1) {
                const ret = getObject(arg0).appendChild(getObject(arg1));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_append_6c3c5a4e89d0c763: function() { return handleError(function (arg0, arg1) {
                getObject(arg0).append(getObject(arg1));
            }, arguments); },
            __wbg_body_40ec34e0a2931fe8: function(arg0) {
                const ret = getObject(arg0).body;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_bottom_e6ed49b80d965dae: function(arg0) {
                const ret = getObject(arg0).bottom;
                return ret;
            },
            __wbg_buffer_54b87055582c8a81: function(arg0) {
                const ret = getObject(arg0).buffer;
                return addHeapObject(ret);
            },
            __wbg_byobRequest_06b654bb15590436: function(arg0) {
                const ret = getObject(arg0).byobRequest;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_byteLength_41862ca4020b9c43: function(arg0) {
                const ret = getObject(arg0).byteLength;
                return ret;
            },
            __wbg_byteOffset_d42e18c4441f628b: function(arg0) {
                const ret = getObject(arg0).byteOffset;
                return ret;
            },
            __wbg_call_8a2dd23819f8a60a: function() { return handleError(function (arg0, arg1) {
                const ret = getObject(arg0).call(getObject(arg1));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_call_a6e5c5dce5018821: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_cancelAnimationFrame_086d6084925c4e06: function() { return handleError(function (arg0, arg1) {
                getObject(arg0).cancelAnimationFrame(arg1);
            }, arguments); },
            __wbg_cancelBubble_5b5f51787bb379dc: function(arg0) {
                const ret = getObject(arg0).cancelBubble;
                return ret;
            },
            __wbg_charCodeAt_2a30bc7c17474cc6: function(arg0, arg1) {
                const ret = getObject(arg0).charCodeAt(arg1 >>> 0);
                return ret;
            },
            __wbg_checked_596d0d7b35f55a01: function(arg0) {
                const ret = getObject(arg0).checked;
                return ret;
            },
            __wbg_childNodes_c4fcb612cf4b6de1: function(arg0) {
                const ret = getObject(arg0).childNodes;
                return addHeapObject(ret);
            },
            __wbg_classList_8c12288eeff7eadb: function(arg0) {
                const ret = getObject(arg0).classList;
                return addHeapObject(ret);
            },
            __wbg_clearTimeout_8f80437be2324e09: function(arg0, arg1) {
                getObject(arg0).clearTimeout(arg1);
            },
            __wbg_clientWidth_6852617da948be39: function(arg0) {
                const ret = getObject(arg0).clientWidth;
                return ret;
            },
            __wbg_clientX_e8c6c674634344de: function(arg0) {
                const ret = getObject(arg0).clientX;
                return ret;
            },
            __wbg_clientY_ffea953797502d5d: function(arg0) {
                const ret = getObject(arg0).clientY;
                return ret;
            },
            __wbg_cloneNode_5f99da4333e10617: function() { return handleError(function (arg0, arg1) {
                const ret = getObject(arg0).cloneNode(arg1 !== 0);
                return addHeapObject(ret);
            }, arguments); },
            __wbg_cloneNode_cec725abcb361e80: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).cloneNode();
                return addHeapObject(ret);
            }, arguments); },
            __wbg_close_249a23304523681b: function() { return handleError(function (arg0) {
                getObject(arg0).close();
            }, arguments); },
            __wbg_close_72d318d9c16e83ef: function() { return handleError(function (arg0) {
                getObject(arg0).close();
            }, arguments); },
            __wbg_code_89c999e407c79eef: function(arg0, arg1) {
                const ret = getObject(arg1).code;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_commonAncestorContainer_a686597547314484: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).commonAncestorContainer;
                return addHeapObject(ret);
            }, arguments); },
            __wbg_composedPath_3d7ca98a55bce60f: function(arg0) {
                const ret = getObject(arg0).composedPath();
                return addHeapObject(ret);
            },
            __wbg_construct_4e1a16de27aea5b9: function() { return handleError(function (arg0, arg1) {
                const ret = Reflect.construct(getObject(arg0), getObject(arg1));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_contains_eb74fff24d3f5d63: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).contains(getStringFromWasm0(arg1, arg2));
                return ret;
            },
            __wbg_content_dd23488ae58df3e5: function(arg0) {
                const ret = getObject(arg0).content;
                return addHeapObject(ret);
            },
            __wbg_createComment_003419d0740789d4: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).createComment(getStringFromWasm0(arg1, arg2));
                return addHeapObject(ret);
            },
            __wbg_createElementNS_013b3fb26f4796ec: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                const ret = getObject(arg0).createElementNS(arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_createElement_fcbc0805de826d62: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_createTextNode_4dad5b18435dda7c: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).createTextNode(getStringFromWasm0(arg1, arg2));
                return addHeapObject(ret);
            },
            __wbg_createTreeWalker_e8e6ce0471342cef: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                const ret = getObject(arg0).createTreeWalker(getObject(arg1), arg2 >>> 0, getObject(arg3));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_ctrlKey_2e52816fa7160097: function(arg0) {
                const ret = getObject(arg0).ctrlKey;
                return ret;
            },
            __wbg_deleteProperty_36be13e7a282429c: function() { return handleError(function (arg0, arg1) {
                const ret = Reflect.deleteProperty(getObject(arg0), getObject(arg1));
                return ret;
            }, arguments); },
            __wbg_documentElement_b7ec99417969bfbc: function(arg0) {
                const ret = getObject(arg0).documentElement;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_document_179650d6cb13c263: function(arg0) {
                const ret = getObject(arg0).document;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_done_89b2b13e91a60321: function(arg0) {
                const ret = getObject(arg0).done;
                return ret;
            },
            __wbg_enqueue_6d83b4c6281bafd6: function() { return handleError(function (arg0, arg1) {
                getObject(arg0).enqueue(getObject(arg1));
            }, arguments); },
            __wbg_entries_015dc610cd81ede0: function(arg0) {
                const ret = Object.entries(getObject(arg0));
                return addHeapObject(ret);
            },
            __wbg_error_744744ff0c9861e6: function(arg0) {
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
            __wbg_exec_408b889762cde4c2: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).exec(getStringFromWasm0(arg1, arg2));
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_fetch_8d9b732df7467c44: function(arg0) {
                const ret = fetch(getObject(arg0));
                return addHeapObject(ret);
            },
            __wbg_firstChild_2bbf157943b5dddb: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).firstChild();
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_firstChild_984b883406cc95b8: function(arg0) {
                const ret = getObject(arg0).firstChild;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_firstElementChild_09d2c7dc8dd1cfd9: function(arg0) {
                const ret = getObject(arg0).firstElementChild;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_focus_2f77051f98540625: function() { return handleError(function (arg0) {
                getObject(arg0).focus();
            }, arguments); },
            __wbg_fromEntries_0eddbdac354a0a78: function() { return handleError(function (arg0) {
                const ret = Object.fromEntries(getObject(arg0));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_fullscreenElement_9f50a5e63bb433a8: function(arg0) {
                const ret = getObject(arg0).fullscreenElement;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_getAttributeNames_5f86619d226af9e5: function(arg0) {
                const ret = getObject(arg0).getAttributeNames();
                return addHeapObject(ret);
            },
            __wbg_getAttribute_5a601ba4718b922a: function(arg0, arg1, arg2, arg3) {
                const ret = getObject(arg1).getAttribute(getStringFromWasm0(arg2, arg3));
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_getBoundingClientRect_e828e6c31c66dea6: function(arg0) {
                const ret = getObject(arg0).getBoundingClientRect();
                return addHeapObject(ret);
            },
            __wbg_getComputedStyle_961681bdf7e518e8: function() { return handleError(function (arg0, arg1) {
                const ret = getObject(arg0).getComputedStyle(getObject(arg1));
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_getElementById_1cbd8f06dbe8eb8e: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).getElementById(getStringFromWasm0(arg1, arg2));
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_getItem_b96269ddc16cf24a: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                const ret = getObject(arg1).getItem(getStringFromWasm0(arg2, arg3));
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            }, arguments); },
            __wbg_getPropertyValue_dc6b061239dad6f1: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                const ret = getObject(arg1).getPropertyValue(getStringFromWasm0(arg2, arg3));
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            }, arguments); },
            __wbg_getRandomValues_bf16787eede473f5: function() { return handleError(function (arg0, arg1) {
                globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
            }, arguments); },
            __wbg_getRangeAt_384abd95ef1e9620: function() { return handleError(function (arg0, arg1) {
                const ret = getObject(arg0).getRangeAt(arg1 >>> 0);
                return addHeapObject(ret);
            }, arguments); },
            __wbg_getSelection_5d3a5a6e5b6a1ddd: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).getSelection();
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_getTime_d6f070c088c9b5ed: function(arg0) {
                const ret = getObject(arg0).getTime();
                return ret;
            },
            __wbg_getTimezoneOffset_dc9862c79e5a81a3: function(arg0) {
                const ret = getObject(arg0).getTimezoneOffset();
                return ret;
            },
            __wbg_get_507a50627bffa49b: function(arg0, arg1) {
                const ret = getObject(arg0)[arg1 >>> 0];
                return addHeapObject(ret);
            },
            __wbg_get_78f252d074a84d0b: function() { return handleError(function (arg0, arg1) {
                const ret = Reflect.get(getObject(arg0), getObject(arg1));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_get_7df959e12c8cb1e0: function() { return handleError(function (arg0, arg1) {
                const ret = Reflect.get(getObject(arg0), arg1 >>> 0);
                return addHeapObject(ret);
            }, arguments); },
            __wbg_get_b2053e9bfdf3ca8e: function(arg0, arg1) {
                const ret = getObject(arg0)[arg1 >>> 0];
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_get_c7eb1f358a7654df: function() { return handleError(function (arg0, arg1) {
                const ret = Reflect.get(getObject(arg0), getObject(arg1));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_get_unchecked_6e0ad6d2a41b06f6: function(arg0, arg1) {
                const ret = getObject(arg0)[arg1 >>> 0];
                return addHeapObject(ret);
            },
            __wbg_get_with_ref_key_6412cf3094599694: function(arg0, arg1) {
                const ret = getObject(arg0)[getObject(arg1)];
                return addHeapObject(ret);
            },
            __wbg_hash_508149c4291ec8c2: function() { return handleError(function (arg0, arg1) {
                const ret = getObject(arg1).hash;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            }, arguments); },
            __wbg_head_41a60f9034e0b41a: function(arg0) {
                const ret = getObject(arg0).head;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_height_96c07d9559d0200a: function(arg0) {
                const ret = getObject(arg0).height;
                return ret;
            },
            __wbg_host_18450e7fb2bf2108: function(arg0) {
                const ret = getObject(arg0).host;
                return addHeapObject(ret);
            },
            __wbg_id_2bb4f5057d3bfc99: function(arg0, arg1) {
                const ret = getObject(arg1).id;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_includes_78c9a3115b08eddc: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).includes(getObject(arg1), arg2);
                return ret;
            },
            __wbg_innerHeight_92315939e482496d: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).innerHeight;
                return addHeapObject(ret);
            }, arguments); },
            __wbg_innerWidth_dec7d2ac73df3e63: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).innerWidth;
                return addHeapObject(ret);
            }, arguments); },
            __wbg_insertBefore_9121f73148bc4f7c: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = getObject(arg0).insertBefore(getObject(arg1), getObject(arg2));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_instanceof_ArrayBuffer_4480b9e0068a8adb: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof ArrayBuffer;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Element_beebfaab75d12d9d: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof Element;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Error_1fdac9f13a8181ba: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof Error;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_HtmlElement_4493a09212d3586f: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof HTMLElement;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_HtmlInputElement_ad3be04339d0e4df: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof HTMLInputElement;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_KeyboardEvent_be49f2d8e15d587a: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof KeyboardEvent;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Map_e5b5e3db98422fcc: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof Map;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Node_d29e7ded486fd76a: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof Node;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_RegExp_eb4797a049ce5618: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof RegExp;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Response_c8b64b2256f01bec: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof Response;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_ShadowRoot_8ab3038bc5e14d84: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof ShadowRoot;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Uint8Array_309b927aaf7a3fc7: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof Uint8Array;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Window_05ba1ee4f6781663: function(arg0) {
                let result;
                try {
                    result = getObject(arg0) instanceof Window;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_isArray_0677c962b281d01a: function(arg0) {
                const ret = Array.isArray(getObject(arg0));
                return ret;
            },
            __wbg_isArray_82995d8620818ac5: function(arg0) {
                const ret = Array.isArray(getObject(arg0));
                return ret;
            },
            __wbg_isSafeInteger_04f36e4056f1b851: function(arg0) {
                const ret = Number.isSafeInteger(getObject(arg0));
                return ret;
            },
            __wbg_is_7b9d0b289033c7de: function(arg0, arg1) {
                const ret = Object.is(getObject(arg0), getObject(arg1));
                return ret;
            },
            __wbg_iterator_6f722e4a93058b71: function() {
                const ret = Symbol.iterator;
                return addHeapObject(ret);
            },
            __wbg_keyCode_f9ab89c2dd6c3770: function(arg0) {
                const ret = getObject(arg0).keyCode;
                return ret;
            },
            __wbg_key_803dca86cdcfa8dd: function(arg0, arg1) {
                const ret = getObject(arg1).key;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_lastChild_6deddc310b0dbb3d: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).lastChild();
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_left_7e76a74d0db1754f: function(arg0) {
                const ret = getObject(arg0).left;
                return ret;
            },
            __wbg_length_02c64e687322fa34: function(arg0) {
                const ret = getObject(arg0).length;
                return ret;
            },
            __wbg_length_1f0964f4a5e2c6d8: function(arg0) {
                const ret = getObject(arg0).length;
                return ret;
            },
            __wbg_length_370319915dc99107: function(arg0) {
                const ret = getObject(arg0).length;
                return ret;
            },
            __wbg_localStorage_5bf6ce3f8e51412a: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).localStorage;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_location_c9a2271428996698: function(arg0) {
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
            __wbg_log_d267660666346fb3: function(arg0) {
                console.log(getObject(arg0));
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
            __wbg_message_8326fb1d549bebc5: function(arg0) {
                const ret = getObject(arg0).message;
                return addHeapObject(ret);
            },
            __wbg_metaKey_d961c7572a9f84f5: function(arg0) {
                const ret = getObject(arg0).metaKey;
                return ret;
            },
            __wbg_name_b0b4809690944614: function(arg0) {
                const ret = getObject(arg0).name;
                return addHeapObject(ret);
            },
            __wbg_new_08cb2fa678b17a48: function() { return handleError(function (arg0, arg1) {
                const ret = new URL(getStringFromWasm0(arg0, arg1));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_new_0_3da9e97f24fc69be: function() {
                const ret = new Date();
                return addHeapObject(ret);
            },
            __wbg_new_0d809930cd1354c6: function() { return handleError(function () {
                const ret = new Headers();
                return addHeapObject(ret);
            }, arguments); },
            __wbg_new_1bd3e2f781a79b55: function(arg0, arg1, arg2, arg3) {
                const ret = new RegExp(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
                return addHeapObject(ret);
            },
            __wbg_new_227d7c05414eb861: function() {
                const ret = new Error();
                return addHeapObject(ret);
            },
            __wbg_new_b667d279fd5aa943: function(arg0, arg1) {
                const ret = new Error(getStringFromWasm0(arg0, arg1));
                return addHeapObject(ret);
            },
            __wbg_new_cc984128914cfc6f: function(arg0) {
                const ret = new Date(getObject(arg0));
                return addHeapObject(ret);
            },
            __wbg_new_cd45aabdf6073e84: function(arg0) {
                const ret = new Uint8Array(getObject(arg0));
                return addHeapObject(ret);
            },
            __wbg_new_da52cf8fe3429cb2: function() {
                const ret = new Object();
                return addHeapObject(ret);
            },
            __wbg_new_f0787df90791d9ba: function() { return handleError(function () {
                const ret = new URLSearchParams();
                return addHeapObject(ret);
            }, arguments); },
            __wbg_new_typed_1824d93f294193e5: function(arg0, arg1) {
                try {
                    var state0 = {a: arg0, b: arg1};
                    var cb0 = (arg0, arg1) => {
                        const a = state0.a;
                        state0.a = 0;
                        try {
                            return __wasm_bindgen_func_elem_33411(a, state0.b, arg0, arg1);
                        } finally {
                            state0.a = a;
                        }
                    };
                    const ret = new Promise(cb0);
                    return addHeapObject(ret);
                } finally {
                    state0.a = 0;
                }
            },
            __wbg_new_with_args_200d82645b6544eb: function(arg0, arg1, arg2, arg3) {
                const ret = new Function(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
                return addHeapObject(ret);
            },
            __wbg_new_with_byte_offset_and_length_54c7724ee3ec7d82: function(arg0, arg1, arg2) {
                const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
                return addHeapObject(ret);
            },
            __wbg_new_with_length_f8cbc3a5b9ff9368: function(arg0) {
                const ret = new Array(arg0 >>> 0);
                return addHeapObject(ret);
            },
            __wbg_new_with_str_54bc0f9c32770e1e: function() { return handleError(function (arg0, arg1) {
                const ret = new Request(getStringFromWasm0(arg0, arg1));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_new_with_str_and_init_d95cbe11ce28e65e: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = new Request(getStringFromWasm0(arg0, arg1), getObject(arg2));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_new_with_year_month_day_hr_min_sec_c04713baa3b5e1a0: function(arg0, arg1, arg2, arg3, arg4, arg5) {
                const ret = new Date(arg0 >>> 0, arg1, arg2, arg3, arg4, arg5);
                return addHeapObject(ret);
            },
            __wbg_nextNode_3ae679e5a9e39c47: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).nextNode();
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_nextSibling_0e94ccfa3c22fa3c: function(arg0) {
                const ret = getObject(arg0).nextSibling;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_next_6dbf2c0ac8cde20f: function(arg0) {
                const ret = getObject(arg0).next;
                return addHeapObject(ret);
            },
            __wbg_next_71f2aa1cb3d1e37e: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).next();
                return addHeapObject(ret);
            }, arguments); },
            __wbg_nodeType_9cfba00db3a42b85: function(arg0) {
                const ret = getObject(arg0).nodeType;
                return ret;
            },
            __wbg_offsetHeight_242135c11b7fdaec: function(arg0) {
                const ret = getObject(arg0).offsetHeight;
                return ret;
            },
            __wbg_offsetWidth_f7d4d93df1ead153: function(arg0) {
                const ret = getObject(arg0).offsetWidth;
                return ret;
            },
            __wbg_outerHTML_daba577a9aca74d9: function(arg0, arg1) {
                const ret = getObject(arg1).outerHTML;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_ownKeys_a2745e10effd5d46: function() { return handleError(function (arg0) {
                const ret = Reflect.ownKeys(getObject(arg0));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_parentElement_5030754e30795652: function(arg0) {
                const ret = getObject(arg0).parentElement;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_parentNode_fecbbdea2a930547: function(arg0) {
                const ret = getObject(arg0).parentNode;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_prepend_bf57fbcabcd38761: function() { return handleError(function (arg0, arg1) {
                getObject(arg0).prepend(getObject(arg1));
            }, arguments); },
            __wbg_preventDefault_b64888c857500682: function(arg0) {
                getObject(arg0).preventDefault();
            },
            __wbg_previousNode_6341f3269efd18c2: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).previousNode();
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_prototypesetcall_4770620bbe4688a0: function(arg0, arg1, arg2) {
                Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), getObject(arg2));
            },
            __wbg_querySelector_b966f59fa9848d69: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = getObject(arg0).querySelector(getStringFromWasm0(arg1, arg2));
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_querySelector_fd7d157ebe17cd16: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = getObject(arg0).querySelector(getStringFromWasm0(arg1, arg2));
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            }, arguments); },
            __wbg_queueMicrotask_0ab5b2d2393e99b9: function(arg0) {
                const ret = getObject(arg0).queueMicrotask;
                return addHeapObject(ret);
            },
            __wbg_queueMicrotask_6a09b7bc46549209: function(arg0) {
                queueMicrotask(getObject(arg0));
            },
            __wbg_readyState_a45f4559d42cf34f: function(arg0, arg1) {
                const ret = getObject(arg1).readyState;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_removeAttribute_1e7d2c409776d836: function() { return handleError(function (arg0, arg1, arg2) {
                getObject(arg0).removeAttribute(getStringFromWasm0(arg1, arg2));
            }, arguments); },
            __wbg_removeEventListener_a3f23c70077bdcc1: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
            }, arguments); },
            __wbg_removeEventListener_c0e097844dc1021c: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), arg4 !== 0);
            }, arguments); },
            __wbg_removeItem_78e03a38da96e0ae: function() { return handleError(function (arg0, arg1, arg2) {
                getObject(arg0).removeItem(getStringFromWasm0(arg1, arg2));
            }, arguments); },
            __wbg_removeProperty_70da952bc1b493fa: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                const ret = getObject(arg1).removeProperty(getStringFromWasm0(arg2, arg3));
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            }, arguments); },
            __wbg_remove_0cf146d24a80a6be: function(arg0) {
                getObject(arg0).remove();
            },
            __wbg_remove_281d6b5594fade8f: function() { return handleError(function (arg0, arg1, arg2) {
                getObject(arg0).remove(getStringFromWasm0(arg1, arg2));
            }, arguments); },
            __wbg_remove_642a9edea7386a3d: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                getObject(arg0).remove(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            }, arguments); },
            __wbg_remove_ce1b54059317fe8a: function(arg0) {
                getObject(arg0).remove();
            },
            __wbg_requestAnimationFrame_1a85deeab66448c2: function() { return handleError(function (arg0, arg1) {
                const ret = getObject(arg0).requestAnimationFrame(getObject(arg1));
                return ret;
            }, arguments); },
            __wbg_requestFullscreen_ba637845d23582f1: function() { return handleError(function (arg0) {
                getObject(arg0).requestFullscreen();
            }, arguments); },
            __wbg_resolve_2191a4dfe481c25b: function(arg0) {
                const ret = Promise.resolve(getObject(arg0));
                return addHeapObject(ret);
            },
            __wbg_respond_510e32df8aeb6817: function() { return handleError(function (arg0, arg1) {
                getObject(arg0).respond(arg1 >>> 0);
            }, arguments); },
            __wbg_right_36c53e00496f4f0a: function(arg0) {
                const ret = getObject(arg0).right;
                return ret;
            },
            __wbg_root_460c52a3da4d68fb: function(arg0) {
                const ret = getObject(arg0).root;
                return addHeapObject(ret);
            },
            __wbg_scrollIntoView_8aebc47f4e6dd724: function(arg0, arg1) {
                getObject(arg0).scrollIntoView(arg1 !== 0);
            },
            __wbg_scrollIntoView_d8b806f471b7418e: function(arg0, arg1) {
                getObject(arg0).scrollIntoView(getObject(arg1));
            },
            __wbg_scrollLeft_120fb764adaa1a05: function(arg0) {
                const ret = getObject(arg0).scrollLeft;
                return ret;
            },
            __wbg_scrollTop_66d239739313b868: function(arg0) {
                const ret = getObject(arg0).scrollTop;
                return ret;
            },
            __wbg_scrollX_9da7f7defce2297e: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).scrollX;
                return ret;
            }, arguments); },
            __wbg_scrollY_b4c56e98c6d976ad: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).scrollY;
                return ret;
            }, arguments); },
            __wbg_search_c905fb82fd20bc6b: function(arg0, arg1) {
                const ret = getObject(arg1).search;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_setAttribute_71039043be82d098: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            }, arguments); },
            __wbg_setItem_364a11cf21db9039: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                getObject(arg0).setItem(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            }, arguments); },
            __wbg_setProperty_e4e51b1b1d681d15: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                getObject(arg0).setProperty(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            }, arguments); },
            __wbg_setTimeout_cfa2cf195c3738db: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = getObject(arg0).setTimeout(getObject(arg1), arg2);
                return ret;
            }, arguments); },
            __wbg_set_4d7dd76f3dae2926: function(arg0, arg1, arg2) {
                getObject(arg0).set(getArrayU8FromWasm0(arg1, arg2));
            },
            __wbg_set_8535240470bf2500: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
                return ret;
            }, arguments); },
            __wbg_set_8a16b38e4805b298: function(arg0, arg1, arg2) {
                getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
            },
            __wbg_set_accept_node_fadec5589740752a: function(arg0, arg1) {
                getObject(arg0).acceptNode = getObject(arg1);
            },
            __wbg_set_behavior_af2ac621388b739f: function(arg0, arg1) {
                getObject(arg0).behavior = __wbindgen_enum_ScrollBehavior[arg1];
            },
            __wbg_set_block_8135b3acafa1ca88: function(arg0, arg1) {
                getObject(arg0).block = __wbindgen_enum_ScrollLogicalPosition[arg1];
            },
            __wbg_set_body_029f2d171e0a005f: function(arg0, arg1) {
                getObject(arg0).body = getObject(arg1);
            },
            __wbg_set_currentNode_bee33060fede85c3: function(arg0, arg1) {
                getObject(arg0).currentNode = getObject(arg1);
            },
            __wbg_set_headers_9c61d123c3ee1f10: function(arg0, arg1) {
                getObject(arg0).headers = getObject(arg1);
            },
            __wbg_set_innerHTML_f78a45a07f97e136: function(arg0, arg1, arg2) {
                getObject(arg0).innerHTML = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_method_5532d59b92d76467: function(arg0, arg1, arg2) {
                getObject(arg0).method = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_nodeValue_62c524895505a99d: function(arg0, arg1, arg2) {
                getObject(arg0).nodeValue = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_open_ac65a528ddc36ede: function(arg0, arg1) {
                getObject(arg0).open = arg1 !== 0;
            },
            __wbg_set_scrollLeft_170d8936bb4869f7: function(arg0, arg1) {
                getObject(arg0).scrollLeft = arg1;
            },
            __wbg_set_scrollTop_d6a7026b97f7b3e6: function(arg0, arg1) {
                getObject(arg0).scrollTop = arg1;
            },
            __wbg_set_search_f9700de567764208: function(arg0, arg1, arg2) {
                getObject(arg0).search = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_textContent_54dcad83ae15772d: function(arg0, arg1, arg2) {
                getObject(arg0).textContent = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_value_e0caa78ebf9917a8: function(arg0, arg1, arg2) {
                getObject(arg0).value = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_x_61040538c5b06e30: function(arg0, arg1) {
                getObject(arg0).x = arg1;
            },
            __wbg_set_y_3e0c514698974674: function(arg0, arg1) {
                getObject(arg0).y = arg1;
            },
            __wbg_slice_50189eefc9ab9fe9: function(arg0, arg1, arg2) {
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
            __wbg_static_accessor_GLOBAL_4ef717fb391d88b7: function() {
                const ret = typeof global === 'undefined' ? null : global;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_static_accessor_GLOBAL_THIS_8d1badc68b5a74f4: function() {
                const ret = typeof globalThis === 'undefined' ? null : globalThis;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_static_accessor_SELF_146583524fe1469b: function() {
                const ret = typeof self === 'undefined' ? null : self;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_static_accessor_WINDOW_f2829a2234d7819e: function() {
                const ret = typeof window === 'undefined' ? null : window;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_status_c45b3b9b3033184a: function(arg0) {
                const ret = getObject(arg0).status;
                return ret;
            },
            __wbg_stopPropagation_4c4ff88c29f9bc38: function(arg0) {
                getObject(arg0).stopPropagation();
            },
            __wbg_stringify_b54333f60f1e4dad: function() { return handleError(function (arg0) {
                const ret = JSON.stringify(getObject(arg0));
                return addHeapObject(ret);
            }, arguments); },
            __wbg_style_6657aed849e5d757: function(arg0) {
                const ret = getObject(arg0).style;
                return addHeapObject(ret);
            },
            __wbg_tagName_d99c8072027f3c98: function(arg0, arg1) {
                const ret = getObject(arg1).tagName;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_target_e759594a8d965ed7: function(arg0) {
                const ret = getObject(arg0).target;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_textContent_37277f66248f39e6: function(arg0, arg1) {
                const ret = getObject(arg1).textContent;
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_text_d3a29f7525a132c3: function() { return handleError(function (arg0) {
                const ret = getObject(arg0).text();
                return addHeapObject(ret);
            }, arguments); },
            __wbg_then_16d107c451e9905d: function(arg0, arg1, arg2) {
                const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
                return addHeapObject(ret);
            },
            __wbg_then_6ec10ae38b3e92f7: function(arg0, arg1) {
                const ret = getObject(arg0).then(getObject(arg1));
                return addHeapObject(ret);
            },
            __wbg_toString_b201c2690bbe445a: function(arg0) {
                const ret = getObject(arg0).toString();
                return addHeapObject(ret);
            },
            __wbg_toString_bac9199ff382784d: function(arg0) {
                const ret = getObject(arg0).toString();
                return addHeapObject(ret);
            },
            __wbg_top_fe120acfa924a430: function(arg0) {
                const ret = getObject(arg0).top;
                return ret;
            },
            __wbg_url_f6cd241d61f89b82: function(arg0, arg1) {
                const ret = getObject(arg1).url;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_value_1f687dfa7d6c3d08: function(arg0, arg1) {
                const ret = getObject(arg1).value;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_value_a5d5488a9589444a: function(arg0) {
                const ret = getObject(arg0).value;
                return addHeapObject(ret);
            },
            __wbg_value_d7621df0105931d8: function(arg0, arg1) {
                const ret = getObject(arg1).value;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_view_21f1d4a4f175dfa9: function(arg0) {
                const ret = getObject(arg0).view;
                return isLikeNone(ret) ? 0 : addHeapObject(ret);
            },
            __wbg_width_219185400361db86: function(arg0) {
                const ret = getObject(arg0).width;
                return ret;
            },
            __wbg_x_881f8331a1789f24: function(arg0) {
                const ret = getObject(arg0).x;
                return ret;
            },
            __wbg_y_cd458b2c5b870c7c: function(arg0) {
                const ret = getObject(arg0).y;
                return ret;
            },
            __wbindgen_cast_0000000000000001: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 6737, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_28615);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000002: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 6931, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_33409);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000003: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Event")], shim_idx: 1924, ret: Unit, inner_ret: Some(Unit) }, mutable: false }) -> Externref`.
                const ret = makeClosure(arg0, arg1, __wasm_bindgen_func_elem_9972);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000004: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Event")], shim_idx: 6717, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_28342);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000005: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Event")], shim_idx: 6737, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_28615_4);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000006: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("MouseEvent")], shim_idx: 5092, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_19320);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000007: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Node")], shim_idx: 5865, ret: U32, inner_ret: Some(U32) }, mutable: false }) -> Externref`.
                const ret = makeClosure(arg0, arg1, __wasm_bindgen_func_elem_24784);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000008: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 6716, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_28341);
                return addHeapObject(ret);
            },
            __wbindgen_cast_0000000000000009: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 6736, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_28614);
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

    function __wasm_bindgen_func_elem_28341(arg0, arg1) {
        wasm.__wasm_bindgen_func_elem_28341(arg0, arg1);
    }

    function __wasm_bindgen_func_elem_28614(arg0, arg1) {
        wasm.__wasm_bindgen_func_elem_28614(arg0, arg1);
    }

    function __wasm_bindgen_func_elem_28615(arg0, arg1, arg2) {
        wasm.__wasm_bindgen_func_elem_28615(arg0, arg1, addHeapObject(arg2));
    }

    function __wasm_bindgen_func_elem_9972(arg0, arg1, arg2) {
        wasm.__wasm_bindgen_func_elem_9972(arg0, arg1, addHeapObject(arg2));
    }

    function __wasm_bindgen_func_elem_28342(arg0, arg1, arg2) {
        wasm.__wasm_bindgen_func_elem_28342(arg0, arg1, addHeapObject(arg2));
    }

    function __wasm_bindgen_func_elem_28615_4(arg0, arg1, arg2) {
        wasm.__wasm_bindgen_func_elem_28615_4(arg0, arg1, addHeapObject(arg2));
    }

    function __wasm_bindgen_func_elem_19320(arg0, arg1, arg2) {
        wasm.__wasm_bindgen_func_elem_19320(arg0, arg1, addHeapObject(arg2));
    }

    function __wasm_bindgen_func_elem_24784(arg0, arg1, arg2) {
        const ret = wasm.__wasm_bindgen_func_elem_24784(arg0, arg1, addHeapObject(arg2));
        return ret >>> 0;
    }

    function __wasm_bindgen_func_elem_33409(arg0, arg1, arg2) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wasm_bindgen_func_elem_33409(retptr, arg0, arg1, addHeapObject(arg2));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }

    function __wasm_bindgen_func_elem_33411(arg0, arg1, arg2, arg3) {
        wasm.__wasm_bindgen_func_elem_33411(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
    }


    const __wbindgen_enum_ReadableStreamType = ["bytes"];


    const __wbindgen_enum_ScrollBehavior = ["auto", "instant", "smooth"];


    const __wbindgen_enum_ScrollLogicalPosition = ["start", "center", "end", "nearest"];
    const IntoUnderlyingByteSourceFinalization = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingbytesource_free(ptr, 1));
    const IntoUnderlyingSinkFinalization = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingsink_free(ptr, 1));
    const IntoUnderlyingSourceFinalization = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingsource_free(ptr, 1));

    function addHeapObject(obj) {
        if (heap_next === heap.length) heap.push(heap.length + 1);
        const idx = heap_next;
        heap_next = heap[idx];

        heap[idx] = obj;
        return idx;
    }

    const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(state => wasm.__wbindgen_export5(state.a, state.b));

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
        return decodeText(ptr >>> 0, len);
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

    function makeClosure(arg0, arg1, f) {
        const state = { a: arg0, b: arg1, cnt: 1 };
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
                wasm.__wbindgen_export5(state.a, state.b);
                state.a = 0;
                CLOSURE_DTORS.unregister(state);
            }
        };
        CLOSURE_DTORS.register(real, state, state);
        return real;
    }

    function makeMutClosure(arg0, arg1, f) {
        const state = { a: arg0, b: arg1, cnt: 1 };
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
                wasm.__wbindgen_export5(state.a, state.b);
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

    let wasmModule, wasmInstance, wasm;
    function __wbg_finalize_init(instance, module) {
        wasmInstance = instance;
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
