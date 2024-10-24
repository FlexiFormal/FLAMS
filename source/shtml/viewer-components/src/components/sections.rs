use immt_ontology::uris::{DocumentElementURI, NarrativeURI};
use leptos::{prelude::*,context::Provider};
use crate::IdPrefix;
use super::navigation::{NavElems, SectionOrInputref};

pub(super) fn section<V:IntoView+'static>(uri:DocumentElementURI,children:impl FnOnce() -> V + Send + 'static) -> impl IntoView {
  let id = expect_context::<IdPrefix>().new_id(uri.name().last_name().as_ref());
  NavElems::update_untracked(|ne| {
    ne.ids.insert(id.clone(),SectionOrInputref::Section);
  });

  view!{
    <div id=id.clone() style="display:content">
    <Provider value=IdPrefix(id.clone())>
      <Provider value=NarrativeURI::Element(uri)>
        {children()}
      </Provider>
    </Provider>
    </div>
  }
}