#![allow(non_local_definitions)]

use immt_web_utils::{components::wait, do_css};
use leptos_dyn_dom::DomStringCont;
use shtml_viewer_components::{components::TOCElem, SectionContinuation};
use immt_ontology::uris::{DocumentElementURI, DocumentURI};
use wasm_bindgen::prelude::wasm_bindgen;
use leptos::prelude::*;

#[derive(Debug,Clone,serde::Serialize,serde::Deserialize,tsify_next::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
/// Options for rendering an SHTML document
/// - FromBackend: calls the backend for the document
///     uri: the URI of the document (as string)
///     toc: if defined, will render a table of contents for the document
// - Prerendered: Take the existent DOM HTMLElement as is
/// - HtmlString: render the provided HTML String
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



#[wasm_bindgen]
/// render an SHTML document to the provided element
pub fn render_document(
  to:leptos::web_sys::HtmlElement,
  document:DocumentOptions,
  on_section_start: Option<SectionContinuation>,
  on_section_end: Option<SectionContinuation>
) -> Result<(),String> {
  use shtml_viewer_components::SHTMLDocument;
  use shtml_viewer_components::components::Toc;

  leptos::prelude::mount_to(to,|| {
    let on_load = RwSignal::new(false);
    match document {
      DocumentOptions::FromBackend{uri,toc} => {
        wait(
          move || {
            tracing::info!("fetching {uri}");
            let fut = shtml_viewer_components::config::server_config.full_doc(uri.clone());
            async move {fut.await.ok()}
          },
          move |(uri,css,html)| {
            for c in css { do_css(c); }
            let toc = toc.clone().map(|toc| match toc {
              TOCOptions::FromBackend => {
                let uriclone = uri.clone();
                let r = Resource::new(|| (),move |()| shtml_viewer_components::config::server_config.get_toc(uriclone.clone()));
                let toc_signal = RwSignal::new(None);
                let _f = Effect::new(move || {
                  if let Some(Ok((css,toc))) = r.get() {
                    toc_signal.set(Some((css,toc)));
                  }
                });
                move || if on_load.get() {
                  toc_signal.get().map(|(css,toc)| view!(<Toc css toc/>))
                } else {None}
              }.into_any(),
              TOCOptions::Predefined(toc) => (move || if on_load.get() {Some(view!(<Toc css=Vec::new() toc=toc.clone()/>))} else {None}).into_any(),
            });
            view!(
              <SHTMLDocument uri on_load>
                {toc}
                <DomStringCont html on_load cont=shtml_viewer_components::iterate/>
              </SHTMLDocument>
            )
          },
          "Error loading document reference".to_string(),
        ).into_any()
      }
      DocumentOptions::HtmlString{html,toc} => view!{
        <SHTMLDocument on_load>
          {toc.map(|toc| 
            move || if on_load.get() {Some(view!(<Toc css=Vec::new() toc=toc.clone()/>))} else {None}
          )}
          <DomStringCont html on_load cont=shtml_viewer_components::iterate/>
        </SHTMLDocument>
      }.into_any()
    }
  }).forget();
  Ok(())
}