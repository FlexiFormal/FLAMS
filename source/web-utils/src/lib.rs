#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![recursion_limit = "256"]

pub mod components;
pub mod mathml;


#[cfg(feature = "ssr")]
/// #### Errors
pub async fn blocking_server_fn<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, leptos::prelude::ServerFnError<String>> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| e.to_string())?
        .map_err(Into::into)
}

/*
pub fn do_css(css: Css) {
    match css {
        CSS::Inline(s) => {
            let id = hashstr("id_", &s);
            #[cfg(not(target_family = "wasm"))]
            let s = String::from(s);
            do_inject_css(id.into(), s.into());
        }
        CSS::Class { name, css } => {
            #[cfg(not(target_family = "wasm"))]
            let name = String::from(name);
            #[cfg(not(target_family = "wasm"))]
            let css = String::from(css);
            do_inject_css(name.into(), css.into());
        }
        CSS::Link(s) => {
            let id = hashstr("id_", &s);
            #[cfg(feature = "ssr")]
            {
                use leptos::prelude::expect_context;
                use leptos_meta::Stylesheet;
                let ids = expect_context::<CssIds>();
                let mut ids = ids.0.lock();
                if !ids.0.contains(&std::borrow::Cow::Borrowed(&id)) {
                    ids.insert(id.clone().into());
                    let _ = leptos::view! {
                        <Stylesheet id=id href=s.to_string()/>
                    };
                }
                drop(ids);
            }
            #[cfg(all(any(feature = "hydrate", feature = "csr"), not(feature = "ssr")))]
            {
                use leptos::prelude::document;
                let Some(head) = document().head() else {
                    leptos::logging::log!("ERROR: head does not exist");
                    return;
                };
                match head.query_selector(&format!("link#{id}")) {
                    Ok(Some(_)) => return,
                    Err(e) => {
                        leptos::logging::log!("ERROR: query link element error: {e:?}");
                        return;
                    }
                    Ok(None) => (),
                };
                let Ok(style) = document().create_element("link") else {
                    leptos::logging::log!("ERROR: error creating style element");
                    return;
                };
                _ = style.set_attribute("id", &id);
                _ = style.set_attribute("rel", "stylesheet");
                _ = style.set_attribute("href", &s);
                _ = head.prepend_with_node_1(&style);
            }
        }
    }
}

#[inline]
pub fn inject_css(id: &'static str, content: &'static str) {
    do_inject_css(Cow::Borrowed(id), Cow::Borrowed(content));
}
 */

#[macro_export]
macro_rules! console_log {
    () => {};
    ($arg:expr) => {
        ::web_sys::console::log_1(&::web_sys::js_sys::JsValue::from($l))
    };
    ($arg1:expr,$arg2:expr) => {
        ::web_sys::console::log_2(
            &::web_sys::js_sys::JsValue::from($l),
            &::web_sys::js_sys::JsValue::from($l),
        )
    };
}

//#[cfg(any(feature = "csr", feature = "ssr"))]
/// # Errors
pub fn try_catch<R>(run: impl FnOnce() -> R) -> Result<R, leptos::wasm_bindgen::JsError> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(run)).map_err(|e| {
        if let Some(s) = e.downcast_ref::<&str>() {
            return leptos::wasm_bindgen::JsError::new(*s);
        }
        if let Some(s) = e.downcast_ref::<String>() {
            return leptos::wasm_bindgen::JsError::new(s);
        }
        leptos::wasm_bindgen::JsError::new("Box<dyn Error>")
    })
}

#[cfg(feature = "ssr")]
pub use http;
#[cfg(feature = "ssr")]
pub use leptos_axum;

#[cfg(feature = "ssr")]
#[macro_export]
macro_rules! not_found{
    () => {
        let response = expect_context::<$crate::leptos_axum::ResponseOptions>();
        response.set_status($crate::http::StatusCode::NOT_FOUND);
    };
    (! $($e:tt)*) => { {
        let response = expect_context::<$crate::leptos_axum::ResponseOptions>();
        response.set_status($crate::http::StatusCode::NOT_FOUND);
        format!($($e)*).into()
    }};
    ($($e:tt)*) => { return Err(not_found!(! $($e)*))};
}
