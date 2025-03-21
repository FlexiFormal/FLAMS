#![recursion_limit = "256"]
#[cfg(any(
    all(feature = "ssr", feature = "hydrate", not(doc)),
    not(any(feature = "ssr", feature = "hydrate"))
))]
compile_error!("exactly one of the features \"ssr\" or \"hydrate\" must be enabled");

pub mod components;
pub mod server_fns;
#[cfg(feature = "ssr")]
mod toc;

#[cfg(feature = "ssr")]
mod ssr {
    use flams_utils::CSS;

    pub(crate) fn insert_base_url(mut v: Vec<CSS>) -> Vec<CSS> {
        //v.sort();
        for c in v.iter_mut() {
            if let CSS::Link(lnk) = c {
                if let Some(r) = lnk.strip_prefix("srv:") {
                    *lnk = format!(
                        "{}{r}",
                        flams_system::settings::Settings::get()
                            .external_url()
                            .unwrap_or("")
                    )
                    .into_boxed_str()
                }
            }
        }
        v
    }

    macro_rules! backend {
      ($fn:ident!($($args:tt)*)) => {
        if flams_system::settings::Settings::get().lsp {
          let Some(state) = ::flams_lsp::STDIOLSPServer::global_state() else {
            return Err("no lsp server".to_string().into())
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
            return Err("no lsp server".to_string())
          };
          state.backend().$fn($($args)*)
        } else {
            flams_system::backend::GlobalBackend::get().$fn($($args)*)
        }
      };
      ($fn:ident($($args:tt)*)) => {
        if flams_system::settings::Settings::get().lsp {
          ::flams_lsp::STDIOLSPServer::global_state().and_then(
            |state| state.backend().$fn($($args)*)
          )
        } else {
          flams_system::backend::GlobalBackend::get().$fn($($args)*)
        }
      };
      ($b:ident => {$($lsp:tt)*}{$($global:tt)*}) => {
        if flams_system::settings::Settings::get().lsp {
          let Some(state) = ::flams_lsp::STDIOLSPServer::global_state() else {
            return Err("no lsp server".to_string().into())
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
}
