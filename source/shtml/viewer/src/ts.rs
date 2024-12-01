#![allow(non_local_definitions)]

use shtml_viewer_components::{components::{TOCElem,TOCSource}, DocumentFromURI, SectionContinuation};
use immt_ontology::uris::DocumentURI;
use wasm_bindgen::prelude::wasm_bindgen;
use leptos::{either::Either, prelude::*};

#[derive(Debug,Clone,serde::Serialize,serde::Deserialize,tsify_next::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
/// Options for rendering an SHTML document
/// - `FromBackend`: calls the backend for the document
///     uri: the URI of the document (as string)
///     toc: if defined, will render a table of contents for the document
// - Prerendered: Take the existent DOM HTMLElement as is
/// - `HtmlString`: render the provided HTML String
///     html: the HTML String
///     toc: if defined, will render a table of contents for the document
pub enum DocumentOptions {
    FromBackend {
        #[tsify(type = "string")]
        uri:DocumentURI,
        toc:Option<TOCOptions>
    },
    //Prerendered,
    HtmlString {
        html:String,
        toc:Option<Vec<TOCElem>>
    }
}

#[derive(Debug,Clone,serde::Serialize,serde::Deserialize,tsify_next::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
/// Options for rendering a table of contents
/// `FromBackend` will retrieve it from the remote backend
/// `Predefined(toc)` will render the provided TOC
pub enum TOCOptions {
    FromBackend,
    Predefined(Vec<TOCElem>)
}

impl From<TOCOptions> for TOCSource {
  fn from(value: TOCOptions) -> Self {
      match value {
          TOCOptions::FromBackend =>Self::Get,
          TOCOptions::Predefined(toc) => Self::Ready(toc)
      }
  }
}

#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
/// render an SHTML document to the provided element
/// #### Errors
pub fn render_document(
  to:leptos::web_sys::HtmlElement,
  document:DocumentOptions,
  on_section_start: Option<SectionContinuation>,
  on_section_end: Option<SectionContinuation>
) -> Result<(),String> {
  use shtml_viewer_components::DocumentString;
  use immt_web_utils::components::Themer;

  let comp = move || match document {
    DocumentOptions::HtmlString{html,toc} => {
      let toc = toc.map_or(TOCSource::None,TOCSource::Ready);
      Either::Left(view!{<Themer><DocumentString html toc/></Themer>})
    }
    DocumentOptions::FromBackend{uri,toc} => {
      let toc = toc.map_or(TOCSource::None,Into::into);
      Either::Right(view!{<Themer><DocumentFromURI uri toc/></Themer>})
    }
  };

  leptos::prelude::mount_to(to,move || {
    if let Some(start) = on_section_start {
      shtml_viewer_components::components::OnSectionBegin::set(start);
    };
    if let Some(end) = on_section_end {
      shtml_viewer_components::components::OnSectionEnd::set(end);
    };
    comp()
  }).forget();
  Ok(())
}