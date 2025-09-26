#![recursion_limit = "256"]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(any(
    all(feature = "ssr", feature = "hydrate", not(feature = "docs-only")),
    not(any(feature = "ssr", feature = "hydrate"))
))]
compile_error!("exactly one of the features \"ssr\" or \"hydrate\" must be enabled");

pub mod backend;
pub mod components;
//pub mod errors;
pub mod server_fns;
#[cfg(feature = "ssr")]
mod toc;

pub type Views = ftml_components::Views<backend::FtmlBackend>;

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
