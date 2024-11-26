use immt_ontology::uris::{DocumentElementURI, NarrativeURI};
use leptos::{prelude::*,context::Provider};
use crate::{IdPrefix, SectionContinuation};
use super::navigation::{NavElems, SectionOrInputref};

pub(super) fn section<V:IntoView+'static>(uri:DocumentElementURI,children:impl FnOnce() -> V + Send + 'static) -> impl IntoView {

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

#[cfg(feature="ts")]
impl SectionContinuation {
    /// #### Errors
  pub fn do_call(&self,uri:&immt_ontology::uris::DocumentElementURI) -> Result<Option<leptos::web_sys::Element>,wasm_bindgen::JsValue> {
    use wasm_bindgen::JsCast;
    let uri = uri.to_string();
    let result = self.call(&uri);
    if result.is_null() || result.is_undefined() {
      return Ok(None);
    }
    let elem : leptos::web_sys::Element = result.dyn_into()?;
    Ok(Some(elem))
  }
}

#[cfg(not(feature="ts"))]
impl SectionContinuation {
    /// #### Errors
    pub const fn do_call(&self,uri:&immt_ontology::uris::DocumentElementURI) -> Result<Option<leptos::web_sys::Element>,wasm_bindgen::JsValue> {
        Ok(None)
    }
}

#[derive(Copy,Clone)]
pub struct OnSectionBegin(StoredValue<send_wrapper::SendWrapper<SectionContinuation>>);
impl OnSectionBegin {
    /// #### Errors
    pub fn call(&self,uri:&immt_ontology::uris::DocumentElementURI) -> Result<Option<leptos::web_sys::Element>,wasm_bindgen::JsValue> {
        self.0.with_value(|f| f.do_call(uri))
    }
    pub fn set(f:SectionContinuation) {
        let f = Self(StoredValue::new(send_wrapper::SendWrapper::new(f)));
        provide_context(f);
    }
}
#[derive(Copy,Clone)]
pub struct OnSectionEnd(StoredValue<send_wrapper::SendWrapper<SectionContinuation>>);
impl OnSectionEnd {
    /// #### Errors
    pub fn call(&self,uri:&immt_ontology::uris::DocumentElementURI) -> Result<Option<leptos::web_sys::Element>,wasm_bindgen::JsValue> {
        self.0.with_value(|f| f.do_call(uri))
    }
    pub fn set(f:SectionContinuation) {
        let f = Self(StoredValue::new(send_wrapper::SendWrapper::new(f)));
        provide_context(f);
    }
}