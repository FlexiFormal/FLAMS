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

pub type Views = ftml_leptos::Views<backend::FtmlBackend>;

#[cfg(feature = "ssr")]
mod ssr {
    use ftml_ontology::utils::Css;

    pub(crate) fn insert_base_url(mut v: Box<[Css]>) -> Box<[Css]> {
        //v.sort();
        for c in v.iter_mut() {
            if let Css::Link(lnk) = c {
                if let Some(r) = lnk.strip_prefix("srv:") {
                    *lnk = format!(
                        "{}{r}",
                        flams_system::settings::Settings::get().external_url()
                    )
                    .into_boxed_str()
                }
            }
        }
        v
    }
    /*
    macro_rules! backend {
      ($fn:ident!($($args:tt)*)) => {
        if flams_system::settings::Settings::get().lsp {
          let Some(state) = ::flams_lsp::STDIOLSPServer::global_state() else {
            panic!("no lsp server");
          };
          state.backend().$fn($($args)*)
        } else {
          ::paste::paste!{
            flams_system::backend::GlobalBackend::get().[<$fn _async>]($($args)*).await
          }
        }
      };
      ($fn:ident SYNC!($($args:tt)*)) => {
        if flams_system::settings::Settings::get().lsp {
          let Some(state) = ::flams_lsp::STDIOLSPServer::global_state() else {
              panic!("no lsp server");
          };
          state.backend().$fn($($args)*)
        } else {
            flams_system::backend::GlobalBackend::get().$fn($($args)*)
        }
      };
      ($fn:ident($($args:tt)*)) => {
        if flams_system::settings::Settings::get().lsp {
            let Some(state) = ::flams_lsp::STDIOLSPServer::global_state() else {
                panic!("no lsp server");
              };
            state.backend().$fn($($args)*)
        } else {
          flams_system::backend::GlobalBackend::get().$fn($($args)*)
        }
      };
      (! $fn:ident($($args:tt)*)) => {
        if flams_system::settings::Settings::get().lsp {
            let state = ::flams_utils::unwrap!(::flams_lsp::STDIOLSPServer::global_state());
            state.backend().$fn($($args)*)
        } else {
          flams_system::backend::GlobalBackend::get().$fn($($args)*)
        }
      };
      ($b:ident => {$($lsp:tt)*}{$($global:tt)*}) => {
        if flams_system::settings::Settings::get().lsp {
          let Some(state) = ::flams_lsp::STDIOLSPServer::global_state() else {
              panic!("no lsp server");
          };
          let $b = state.backend();
          $($lsp)*
        } else {
          let $b = flams_system::backend::GlobalBackend::get();
          $($global)*
        }
      };
    }

    pub(crate) use backend;
     */
}
