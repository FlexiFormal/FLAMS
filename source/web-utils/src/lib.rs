#![cfg_attr(docsrs, feature(doc_cfg))]
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
