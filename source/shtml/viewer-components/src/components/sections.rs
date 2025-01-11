use immt_ontology::uris::{DocumentElementURI, NarrativeURI};
use leptos::{prelude::*,context::Provider};
use crate::{IdPrefix, SectionContinuation};
use super::navigation::{NavElems, SectionOrInputref};

#[cfg(feature="ts")]
pub fn do_begin(uri:&DocumentElementURI) -> Option<NodeRef<leptos::html::Div>> {
  let bg: OnSectionBegin = use_context()?;
  do_ref(bg.get(uri))
}

#[cfg(feature="ts")]
pub fn do_end(uri:&DocumentElementURI) -> Option<NodeRef<leptos::html::Div>> {
  let bg: OnSectionEnd = use_context()?;
  do_ref(bg.get(uri))
}

#[cfg(feature="ts")]
fn do_ref(f:Result<Option<SectionFn>,wasm_bindgen::JsValue>) -> Option<NodeRef<leptos::html::Div>> {
  let f = match f {
    Ok(f) => f?,
    Err(e) => {
      tracing::error!("Error getting section callback: {e:?}");
      return None
    }
  };
  let ret = NodeRef::new();
  let _f = Effect::new(move || if let Some(e) = ret.get() {
    if let Err(e) = f.call(&e) {
      tracing::error!("Error calling section callback: {e:?}");
    }
  });
  Some(ret)
}

pub(super) fn section<V:IntoView+'static>(uri:DocumentElementURI,children:impl FnOnce() -> V + Send + 'static) -> impl IntoView {

  let id = expect_context::<IdPrefix>().new_id(uri.name().last_name().as_ref());
  NavElems::update_untracked(|ne| {
    ne.ids.insert(id.clone(),SectionOrInputref::Section);
  });

  #[cfg(feature="ts")]
  let begin_rf = do_begin(&uri);
  #[cfg(feature="ts")]
  let end_rf = do_end(&uri);

  //use_context::<OnSectionBegin>().map(|s|)

  view!{
    <Provider value=IdPrefix(id.clone())>
      <Provider value=NarrativeURI::Element(uri)>
      <div id=id style="display:content">
        {#[cfg(feature="ts")]
        {begin_rf.map(|rf| view!(<div node_ref=rf/>))}
        }
        {children()}
        {#[cfg(feature="ts")]
        {end_rf.map(|rf| view!(<div node_ref=rf/>))}
        }
      </div>
      </Provider>
    </Provider>
  }
}


#[cfg(feature="ts")]
pub(crate) struct SectionFn(web_sys::js_sys::Function);
#[cfg(feature="ts")]
impl SectionFn {
  pub fn call(&self,e:&web_sys::HtmlDivElement) -> Result<(),wasm_bindgen::JsValue> {
    self.0.call1(&wasm_bindgen::JsValue::UNDEFINED,&wasm_bindgen::JsValue::from(e))?;
    Ok(())
  }
}

#[cfg(not(feature="ts"))]
pub(crate) struct SectionFn();

#[cfg(feature="ts")]
impl SectionContinuation {
    /// #### Errors
  pub fn get_fn(&self,uri:&immt_ontology::uris::DocumentElementURI) -> Result<Option<SectionFn>,wasm_bindgen::JsValue> {
    use wasm_bindgen::JsCast;
    let uri = uri.to_string();
    let f = self.call1(&wasm_bindgen::JsValue::UNDEFINED,&wasm_bindgen::JsValue::from_str(&uri))?;
    if f.is_null() || f.is_undefined() { return Ok(None); }
    let f = web_sys::js_sys::Function::from(f);
    if !f.is_function() { return Err(f.into()) }
    Ok(Some(SectionFn(f)))
  }
}

#[cfg(not(feature="ts"))]
impl SectionContinuation {
    /// #### Errors
    pub const fn get_fn(&self,uri:&immt_ontology::uris::DocumentElementURI) -> Result<Option<SectionFn>,wasm_bindgen::JsValue> {
        Ok(None)
    }
}

#[derive(Copy,Clone)]
pub struct OnSectionBegin(StoredValue<send_wrapper::SendWrapper<SectionContinuation>>);
impl OnSectionBegin {
    /// #### Errors
    pub fn get(&self,uri:&immt_ontology::uris::DocumentElementURI) -> Result<Option<SectionFn>,wasm_bindgen::JsValue> {
        self.0.with_value(|f| f.get_fn(uri))
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
    pub fn get(&self,uri:&immt_ontology::uris::DocumentElementURI) -> Result<Option<SectionFn>,wasm_bindgen::JsValue> {
        self.0.with_value(|f| f.get_fn(uri))
    }
    pub fn set(f:SectionContinuation) {
        let f = Self(StoredValue::new(send_wrapper::SendWrapper::new(f)));
        provide_context(f);
    }
}