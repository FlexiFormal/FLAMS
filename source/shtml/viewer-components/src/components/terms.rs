use immt_ontology::uris::ContentURI;
use immt_web_utils::{components::{DivOrMrow, Popover, PopoverSize, PopoverTriggerType}, do_css, inject_css};
use leptos::{context::Provider, prelude::*};
use shtml_extraction::open::terms::{OpenTerm, VarOrSym};

use crate::SHTMLString;

#[derive(Clone)]
pub(super) struct InTermState {
  owner:VarOrSym,
  is_hovered:RwSignal<bool>
}

#[component]
pub fn SHTMLTerm(#[prop(optional)] in_math:bool, t:OpenTerm,children:Children) -> impl IntoView {
  if in_math { 
    do_term::<_,true>(t, children).into_any()
  } else { 
    do_term::<_,false>(t,children).into_any()
  };
}

pub(super) fn do_term<V:IntoView+'static,const MATH:bool>(t:OpenTerm,children:impl FnOnce() -> V + Send + 'static) -> impl IntoView + 'static {
  let head = InTermState {owner:t.take_head(), is_hovered:RwSignal::new(false) };
  view!{
    <Provider value=Some(head)>{
      children()
    }</Provider>
  }
}

pub(super) fn do_comp<V:IntoView+'static,const MATH:bool>(children:impl FnOnce() -> V + Send + 'static) -> impl IntoView {
  use immt_web_utils::components::PopoverTrigger;
  let in_term = use_context::<Option<InTermState>>();
  if let Some(Some(in_term)) = in_term {
    let is_hovered = in_term.is_hovered;
    tracing::debug!("comp of term {:?}",in_term.owner);
    let is_var = matches!(in_term.owner,VarOrSym::V(_));
    let class = Memo::new(move |_| 
      match (is_hovered.get(), is_var) {
        (true, true) => "shtml-var-comp shtml-comp-hover".to_string(),
        (true, false) => "shtml-comp shtml-comp-hover".to_string(),
        (false, true) => "shtml-var-comp".to_string(),
        (false, false) => "shtml-comp".to_string(),
      }
    );
    let s = in_term.owner;
    let node_type = if MATH { DivOrMrow::Mrow } else { DivOrMrow::Div };
    view!(
      <Popover node_type class size=PopoverSize::Small on_open=move || is_hovered.set(true) on_close=move || is_hovered.set(false)>
        <PopoverTrigger class slot>{children()}</PopoverTrigger>
        //<div style="max-width:600px;">
          {match s {
            VarOrSym::V(v) => view!{<div>"Variable" {v.name().last_name().to_string()}</div>}.into_any(),
            VarOrSym::S(ContentURI::Symbol(s)) => {
              let r = Resource::new(|| (),move |()| crate::config::server_config.definition(s.clone()));
              view!{
                <Suspense fallback=|| view!(<immt_web_utils::components::Spinner/>)>{move || {
                  if let Some(Ok((css,s))) = r.get() {
                    for c in css { do_css(c); }
                    Some(view!(<div style="color:black;background-color:white;padding:3px;"><SHTMLString html=s/></div>))
                  } else {None}
                }}</Suspense>
              }.into_any()
            }
            VarOrSym::S(ContentURI::Module(m)) =>
              view!{<div>"Module" {m.name().last_name().to_string()}</div>}.into_any(),
        }}//</div>
      </Popover>
    ).into_any()
  } else { children().into_any() }
}


pub(super) fn do_arg<V:IntoView+'static>(children:impl FnOnce() -> V + Send + 'static) -> impl IntoView {
  let value : Option<InTermState> = None;
  view!{<Provider value>{children()}</Provider>}
}