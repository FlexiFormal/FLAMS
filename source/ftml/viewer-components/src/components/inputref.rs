use flams_ontology::{narration::sections::SectionLevel, uris::{DocumentURI, NarrativeURI}};
use leptos::{context::Provider, either::Either, prelude::*};
use flams_web_utils::{do_css, inject_css};
use leptos_dyn_dom::{DomChildrenCont, OriginalNode};

use crate::{components::navigation::{NavElems, SectionOrInputref}, config::{IdPrefix, LogicalLevel, SectionCounters}, extractor::DOMExtractor};

#[derive(Copy,Clone)]
pub struct InInputRef(pub bool);

#[component]
pub fn InputRef<'a>(uri:DocumentURI,id: &'a str) -> impl IntoView {
  inputref(uri,id)
}

#[allow(clippy::similar_names)]
pub(super) fn inputref(uri:DocumentURI,id:&str) -> impl IntoView {
  //leptos::logging::log!("inputref");
  inject_css("ftml-inputref", include_str!("./inputref.css"));
  let replace = RwSignal::new(false);
  let replaced = RwSignal::new(false);
  let on_click = move |_| { replace.set(true); };
  let id = expect_context::<IdPrefix>().new_id(id);
  let title = NavElems::update_untracked(|ne| {
    ne.ids.insert(id.clone(), SectionOrInputref::Inputref(replace,replaced));
    ne.get_title(uri.clone())
  });
  let ctrs: SectionCounters = expect_context();
  match ctrs.current {
    LogicalLevel::Section(lvl) if lvl > SectionLevel::Subsection => (),
    _ => replace.set(true)
  }

  view!{
    <Provider value=InInputRef(true)><Provider value=IdPrefix(id.clone())> {
      move || if replace.get() { Either::Left(do_inputref(uri.clone(),replaced)) } else {
        Either::Right(view!(<div id=id.clone() on:click=on_click class="ftml-inputref">{
          move || view!(<span inner_html=title/>)
        }</div>))
    }}</Provider></Provider>
  }
}


fn do_inputref(uri:DocumentURI,on_load:RwSignal<bool>) -> impl IntoView {
  use flams_web_utils::components::wait;
  use leptos_dyn_dom::DomStringCont;
  let uricl = uri.clone();
  wait(
    move || {
      tracing::info!("Inputref fetching {uri}");
      let uri = uri.clone();
      async move {crate::remote::server_config.inputref(uri).await.ok()}
    },
    move |(_,css,html)| {
      for c in css { do_css(c); }
      view!(<span style="display:contents">
      <Provider value=NarrativeURI::Document(uricl.clone())>
      <Provider value = RwSignal::new(DOMExtractor::default())>
        <DomStringCont html cont=crate::iterate on_load/>
      </Provider></Provider>
      </span>)
    },
    "Error loading document reference".to_string(),
  )
}

#[component]
pub fn IfInputref<Ch:IntoView+'static>(value:bool,children:TypedChildren<Ch>) -> impl IntoView {
  let children = children.into_inner();
  let in_inputref = expect_context::<InInputRef>().0;
  if in_inputref == value { Either::Left(children()) } else {
    Either::Right(view!{<span data-if-inputref="false"/>})
  }
}

pub(super) fn if_inputref(val:bool,orig:OriginalNode) -> impl IntoView {
  let in_inputref = expect_context::<InInputRef>().0;
  if in_inputref == val { 
    Either::Left(view!{<span style="display:contents">
      <DomChildrenCont orig cont=crate::iterate/>
    </span>}) 
  } else { Either::Right(view!{<span data-if-inputref="false"/>}) } 
}