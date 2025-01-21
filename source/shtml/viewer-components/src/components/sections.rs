use immt_ontology::uris::{DocumentElementURI, NarrativeURI};
use leptos::{prelude::*,context::Provider};
use web_sys::HtmlDivElement;
use crate::{config::IdPrefix, ts::{JsFun, JsOrRsF, NamedJsFunction, SectionContinuation, TsCont}};
use super::navigation::{NavElems, SectionOrInputref};

pub(super) fn section<V:IntoView+'static>(uri:DocumentElementURI,children:impl FnOnce() -> V + Send + 'static) -> impl IntoView {

  let id = expect_context::<IdPrefix>().new_id(uri.name().last_name().as_ref());
  NavElems::update_untracked(|ne| {
    ne.ids.insert(id.clone(),SectionOrInputref::Section);
  });

  let begin = use_context::<OnSectionBegin>().map(|s| s.view(&uri));
  let end = use_context::<OnSectionEnd>().map(|s| s.view(&uri));


  view!{
    <Provider value=IdPrefix(id.clone())>
      <Provider value=NarrativeURI::Element(uri)>
      <div id=id style="display:content">
        {begin}
        {children()}
        {end}
      </div>
      </Provider>
    </Provider>
  }
}

type SectCont = JsOrRsF<DocumentElementURI,Option<JsOrRsF<HtmlDivElement,()>>>;

#[derive(Clone)]
pub struct OnSectionBegin(SectCont);
impl OnSectionBegin {
    pub fn view(&self,uri:&DocumentElementURI) -> impl IntoView {
      TsCont::res_into_view(self.0.apply(uri))
    }
  
    pub fn set(f:SectCont) {
        //let f = Self(StoredValue::new(send_wrapper::SendWrapper::new(f)));
        provide_context(Self(f));
    }
}
#[derive(Clone)]
pub struct OnSectionEnd(SectCont);
impl OnSectionEnd {
    pub fn view(&self,uri:&DocumentElementURI) -> impl IntoView {
      TsCont::res_into_view(self.0.apply(uri))
    }

    pub fn set(f:SectCont) {
        //let f = Self(StoredValue::new(send_wrapper::SendWrapper::new(f)));
        provide_context(Self(f));
    }
}