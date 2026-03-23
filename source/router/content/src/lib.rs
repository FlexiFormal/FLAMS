#![recursion_limit = "256"]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(any(
    all(feature = "ssr", feature = "hydrate", not(feature = "docs-only")),
    not(any(feature = "ssr", feature = "hydrate"))
))]
compile_error!("exactly one of the features \"ssr\" or \"hydrate\" must be enabled");

pub mod backend;
pub mod checks;
pub mod components;
//pub mod errors;
pub mod server_fns;
#[cfg(feature = "ssr")]
mod toc;

use leptos::prelude::*;

#[server(prefix = "/api", endpoint = "checklog")]
pub async fn get_check_log(
    uri: ftml_uris::DocumentUri,
) -> Result<String /*ftml_solver_trace::results::DocumentCheckResult*/, ServerFnError<String>> {
    use flams_math_archives::BuildableArchive;
    use flams_math_archives::backend::LocalBackend;
    use ftml_uris::IsNarrativeUri;
    use ftml_uris::UriWithArchive;
    use ftml_uris::UriWithPath;
    tokio::task::spawn_blocking(move || {
        let s = flams_system::backend::backend().with_local_archive(uri.archive_id(), |a| {
            a.and_then(|a| {
                let rp = a.rel_path_of(uri.path(), uri.document_name(), uri.language)?;
                let f = a.get_log(rp.as_os_str().to_str()?, ftml_solver::CHECK.id());
                std::fs::read_to_string(f).ok()
            })
        });
        s.map_or_else(
            || Err("No checking log found".to_string()),
            Ok, //|s| ftml_solver_trace::results::DocumentCheckResult::from_json(&s),
        )
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(Into::into)
}

pub struct Continuations;
impl ftml_components::ViewContinuations for Continuations {
    fn document_drawer(
        &self,
        doc: &ftml_ontology::narrative::documents::Document,
    ) -> leptos::prelude::AnyView {
        use crate::checks::ResultExt;
        use flams_web_utils::components::wait_and_then_fn;
        use ftml_components::utils::Header;
        use ftml_components::utils::collapsible::LazyCollapsible;
        use thaw::Caption1Strong;
        let uri = doc.uri.clone();
        view! {
            <LazyCollapsible>
                <Header slot><Caption1Strong>"Checking results"</Caption1Strong></Header>
                {
                    let uri = uri.clone();
                    wait_and_then_fn(
                        move || get_check_log(uri.clone()),
                        |s| ftml_solver_trace::results::DocumentCheckResult::from_json(&s)
                            .map_or_else(|e| e.into_any(),|e| e.render())//ResultExt::render
                    )
                }
            </LazyCollapsible>
        }
        .into_any()
    }
}

pub type Views = ftml_components::Views;

#[cfg(feature = "ssr")]
mod ssr {
    use ftml_ontology::utils::Css;

    pub fn insert_base_url(mut v: Box<[Css]>) -> Box<[Css]> {
        //v.sort();
        for c in &mut v {
            if let Css::Link(lnk) = c
                && let Some(r) = lnk.strip_prefix("srv:")
            {
                *lnk = format!(
                    "{}{r}",
                    flams_system::settings::Settings::get().external_url()
                )
                .into_boxed_str();
            }
        }
        v
    }
}
