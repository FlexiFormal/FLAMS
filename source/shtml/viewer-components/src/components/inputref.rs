use immt_ontology::uris::{DocumentURI, NarrativeURI};
use leptos::{context::Provider, prelude::*};
use immt_web_utils::{do_css, inject_css};
use leptos_dyn_dom::{DomChildrenCont, OriginalNode};

use crate::{components::navigation::{NavElems, SectionOrInputref}, extractor::DOMExtractor, IdPrefix};

#[derive(Copy,Clone)]
pub struct InInputRef(pub bool);

#[component]
pub fn InputRef<'a>(uri:DocumentURI,id: &'a str) -> impl IntoView {
  inputref(uri,id)
}

#[allow(clippy::similar_names)]
pub(super) fn inputref(uri:DocumentURI,id:&str) -> impl IntoView {
  inject_css("shtml-inputref", r#"
.shtml-inputref {
  margin: 5px 15px 5px 15px;
  display: block;
  border: 3px solid blue;
  cursor:cell;
  padding: 0 5px;
  width:auto;
  font-size:medium;
  font-family:serif;
  color:blue;
}
.shtml-inputref::before { content: "⊞ ";}"#);
  let name = uri.name().to_string();
  let replace = RwSignal::new(false);
  let replaced = RwSignal::new(false);
  let on_click = move |_| { replace.set(true); };
  let id = expect_context::<IdPrefix>().new_id(id);
  NavElems::update_untracked(|ne| {
    ne.ids.insert(id.clone(), SectionOrInputref::Inputref(replace,replaced));
  });

  view!{
    <Provider value=InInputRef(true)><Provider value=IdPrefix(id.clone())> {
      move || if replace.get() { do_inputref(uri.clone(),replaced).into_any() } else {
        view!(<div id=id.clone() on:click=on_click class="shtml-inputref">{
          name.clone()
        }</div>).into_any()
    }}</Provider></Provider>
  }
}


fn do_inputref(uri:DocumentURI,on_load:RwSignal<bool>) -> impl IntoView {
  use immt_web_utils::components::wait;
  use leptos_dyn_dom::DomStringCont;
  let uriclone = uri.clone();
  wait(
    move || {
      tracing::info!("Inputref fetching {uri}");
      let uri = uri.clone();
      async move {crate::config::server_config.inputref(uri).await.ok()}
    },
    move |(css,html)| {
      for c in css { do_css(c); }
      view!(<span style="display:contents">
      <Provider value=NarrativeURI::Document(uriclone.clone())>
      <Provider value = RwSignal::new(DOMExtractor::default())>
        <DomStringCont html cont=crate::iterate on_load/>
      </Provider></Provider>
      </span>)
    },
    "Error loading document reference".to_string(),
  )
}

#[component]
pub fn IfInputref(value:bool,children:Children) -> impl IntoView {
  let in_inputref = expect_context::<InInputRef>().0;
  if in_inputref == value { children() } else {
    view!{<span data-if-inputref="false"/>}.into_any()
  }
}

pub(super) fn if_inputref(val:bool,orig:OriginalNode) -> impl IntoView {
  let in_inputref = expect_context::<InInputRef>().0;
  if in_inputref == val { 
    view!{<span style="display:contents">
      <DomChildrenCont orig cont=crate::iterate/>
    </span>}.into_any() 
  } else { view!{<span data-if-inputref="false"/>}.into_any() } 
}