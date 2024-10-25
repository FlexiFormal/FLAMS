use immt_ontology::uris::{DocumentElementURI, NarrativeURI};
use leptos::{prelude::*,context::Provider};
use crate::IdPrefix;
use super::navigation::{NavElems, SectionOrInputref};

pub(super) fn section<V:IntoView+'static>(uri:DocumentElementURI,children:impl FnOnce() -> V + Send + 'static) -> impl IntoView {
  #[cfg(feature="ts")]
  use crate::{OnSectionBegin, OnSectionEnd};

  let id = expect_context::<IdPrefix>().new_id(uri.name().last_name().as_ref());
  NavElems::update_untracked(|ne| {
    ne.ids.insert(id.clone(),SectionOrInputref::Section);
  });

  let rf = NodeRef::new();


  #[cfg(feature="ts")]
  if let Some(bg) = use_context::<OnSectionBegin>() {
    let uri = uri.clone();
    let _f = Effect::new(move || if let Some(node) = rf.get() {
      let node : web_sys::HtmlDivElement = node;
      if let Ok(Some(e)) = bg.call(&uri) {
        let _ = node.prepend_with_node_1(&e);
      }
    });
  }

  #[cfg(feature="ts")]
  if let Some(end) = use_context::<OnSectionEnd>() {
    let uri = uri.clone();
    let _f = Effect::new(move || if let Some(node) = rf.get() {
      let node : web_sys::HtmlDivElement = node;
      if let Ok(Some(e)) = end.call(&uri) {
        let _ = node.append_child(&e);
      }
    });
  }


  //use_context::<OnSectionBegin>().map(|s|)

  view!{
    <Provider value=IdPrefix(id.clone())>
      <Provider value=NarrativeURI::Element(uri)>
        <div node_ref=rf id=id style="display:content">
          {children()}
        </div>
      </Provider>
    </Provider>
  }
}