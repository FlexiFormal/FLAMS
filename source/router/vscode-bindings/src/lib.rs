#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(any(
    all(feature = "ssr", feature = "hydrate", not(feature = "docs-only")),
    not(any(feature = "ssr", feature = "hydrate"))
))]
compile_error!("exactly one of the features \"ssr\" or \"hydrate\" must be enabled");

use flams_web_utils::inject_css;
pub use leptos::prelude::*;
pub mod components;
use flams_utils::unwrap;

#[server]
#[allow(clippy::unused_async)]
async fn is_lsp() -> Result<bool, ServerFnError> {
    Ok(flams_system::settings::Settings::get().lsp)
}

#[derive(Clone)]
pub struct VSCode {
    origin: String,
}
impl VSCode {
    pub fn get() -> Option<Self> {
        use_context()
    }
    pub fn post_message<T: leptos::server_fn::serde::Serialize + std::fmt::Debug>(
        &self,
        t: T,
    ) -> Result<(), String> {
        #[cfg(feature = "hydrate")]
        {
            leptos::logging::log!("Here: {t:?}");
            let e = serde_wasm_bindgen::to_value(&t).map_err(|e| e.to_string())?;
            leptos::logging::log!("Got value");
            let parent = unwrap!(unwrap!(unwrap!(leptos::web_sys::window()).parent().ok()));
            leptos::logging::log!("Got parent");
            unwrap!(parent.post_message(&e, &self.origin).ok());
        }
        Ok(())
    }
}

#[component(transparent)]
pub fn VSCodeWrap() -> impl IntoView {
    use flams_router_login::components::LoginProvider;
    use leptos::either::EitherOf3;
    inject_css("flams-vscode", include_str!("vscode.css"));
    let lsp = Resource::new(|| (), |()| is_lsp());
    if let Some(origin) = leptos_router::hooks::use_query_map().with_untracked(|q| q.get("origin"))
    {
        provide_context(VSCode { origin });
    }
    view!(
        <LoginProvider><Suspense>{move ||
            match lsp.get() {
                Some(Ok(true)) => EitherOf3::A(view!(
                    <div class="flams-vscode">
                        <leptos_router::components::Outlet/>
                    </div>
                )),
                Some(_) => EitherOf3::B("ERROR"),
                None => EitherOf3::C(view!(<flams_web_utils::components::Spinner/>)),
            }
        }
        </Suspense></LoginProvider>
    )
}
