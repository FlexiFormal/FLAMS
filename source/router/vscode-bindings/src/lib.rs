#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::must_use_candidate)]

#[cfg(any(
    all(feature = "ssr", feature = "hydrate", not(feature = "docs-only")),
    not(any(feature = "ssr", feature = "hydrate"))
))]
compile_error!("exactly one of the features \"ssr\" or \"hydrate\" must be enabled");

use ftml_dom::utils::css::inject_css;
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
    #[must_use]
    pub fn get() -> Option<Self> {
        use_context()
    }
    /// # Errors
    pub fn post_message<T: leptos::server_fn::serde::Serialize + std::fmt::Debug>(
        &self,
        t: T,
    ) -> Result<(), String> {
        #[cfg(feature = "hydrate")]
        {
            let e = serde_wasm_bindgen::to_value(&t).map_err(|e| e.to_string())?;
            let parent = unwrap!(unwrap!(unwrap!(leptos::web_sys::window()).parent().ok()));
            unwrap!(parent.post_message(&e, &self.origin).ok());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct VSCWrap;
#[leptos_router::lazy_route]
impl leptos_router::LazyRoute for VSCWrap {
    fn data() -> Self {
        Self
    }
    fn view(VSCWrap: Self) -> AnyView {
        use flams_router_login::components::LoginProvider;
        use leptos::either::EitherOf3;
        ftml_dom::global_setup(|| {
            flams_router_content::Views::top_safe(|| {
                inject_css("flams-vscode", include_str!("vscode.css"));
                let lsp = Resource::new(|| (), |()| is_lsp());
                if let Some(origin) =
                    leptos_router::hooks::use_query_map().with_untracked(|q| q.get("origin"))
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
            })
        })
        .into_any()
    }
}

#[component(transparent)]
pub fn VSCodeWrap() -> impl IntoView {
    use flams_router_login::components::LoginProvider;
    use leptos::either::EitherOf3;
    ftml_dom::global_setup(|| {
        flams_router_content::Views::top_safe(|| {
            inject_css("flams-vscode", include_str!("vscode.css"));
            let lsp = Resource::new(|| (), |()| is_lsp());
            if let Some(origin) =
                leptos_router::hooks::use_query_map().with_untracked(|q| q.get("origin"))
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
        })
    })
}
