use immt_ontology::uris::{ArchiveURITrait, ContentURI, DocumentElementURI, URIWithLanguage};
use immt_web_utils::{components::{DivOrMrow, Popover, OnClickModal,PopoverSize, PopoverTriggerType}, do_css, inject_css};
use leptos::{context::Provider, prelude::*};
use shtml_extraction::open::terms::{OpenTerm, VarOrSym};

use crate::{components::{IntoLOs, LOs}, SHTMLString};

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
    let s_click = s.clone();
    let node_type = if MATH { DivOrMrow::Mrow } else { DivOrMrow::Div };
    view!(
      <Popover node_type class size=PopoverSize::Small on_open=move || is_hovered.set(true) on_close=move || is_hovered.set(false)>
        <PopoverTrigger class slot>{children()}</PopoverTrigger>
        <OnClickModal slot>{do_onclick(s_click)}</OnClickModal>
        //<div style="max-width:600px;">
          {match s {
            VarOrSym::V(v) => view!{<span>"Variable "{v.name().last_name().to_string()}</span>}.into_any(),
            VarOrSym::S(ContentURI::Symbol(s)) => crate::config::get!(definition(s.clone()) = (css,s) => {
              for c in css { do_css(c); }
              Some(view!(<div style="color:black;background-color:white;padding:3px;"><SHTMLString html=s/></div>))
            }).into_any(),
            VarOrSym::S(ContentURI::Module(m)) =>
              view!{<div>"Module" {m.name().last_name().to_string()}</div>}.into_any(),
        }}//</div>
      </Popover>
    ).into_any()
  } else { children().into_any() }
}

fn do_onclick(uri:VarOrSym) -> impl IntoView {
  use thaw::{Combobox,ComboboxOption,ComboboxOptionGroup,Divider};
  let s = match uri {
    VarOrSym::V(v) => return view!{<span>"Variable "{v.name().last_name().to_string()}</span>}.into_any(),
    VarOrSym::S(ContentURI::Module(m)) =>
      return view!{<div>"Module" {m.name().last_name().to_string()}</div>}.into_any(),
    VarOrSym::S(ContentURI::Symbol(s)) => s
  };
  let name = s.name().last_name().to_string();

  crate::config::get!(get_los(s.clone(),false) = v => {
    let LOs {definitions,examples,..} = v.lo_sort();
    let ex_off = definitions.len();
    let selected = RwSignal::new(definitions.first().map(|_| "0".to_string()));
    let definitions = StoredValue::new(definitions);
    let examples = StoredValue::new(examples);
    view!{//<div>
      <div style="display:flex;flex-direction:row;">
        <div style="font-weight:bold;">{name.clone()}</div>
        <div style="margin-left:auto;"><Combobox selected_options=selected placeholder="Select Definition or Example">
          <ComboboxOptionGroup label="Definitions">{
              definitions.with_value(|v| v.iter().enumerate().map(|(i,d)| {
                let line = lo_line(d);
                let value = i.to_string();
                view!{
                  <ComboboxOption text="" value>{line}</ComboboxOption>
                }
            }).collect_view())
          }</ComboboxOptionGroup>
          <ComboboxOptionGroup label="Examples">{
            examples.with_value(|v| v.iter().enumerate().map(|(i,d)| {
              let line = lo_line(d);
              let value = (ex_off + i).to_string();
              view!{
                <ComboboxOption text="" value>{line}</ComboboxOption>
              }
            }).collect_view())
          }</ComboboxOptionGroup>
        </Combobox></div>
      </div>
      <div style="margin:5px;"><Divider/></div>
      {move || {
        let uri = selected.with(|s| s.as_ref().map(|s| {
          let i: usize = s.parse().unwrap_or_else(|_| unreachable!());
          if i < ex_off {
            definitions.with_value(|v:&Vec<DocumentElementURI>| v.as_slice()[i].clone())
          } else {
            examples.with_value(|v:&Vec<DocumentElementURI>| v.as_slice()[i - ex_off].clone())
          }
        }));
        uri.map(|uri| {
          crate::config::get!(paragraph(uri.clone()) = (css,html) => {
            for c in css { do_css(c); }
            view!(<div><SHTMLString html=html/></div>)
          })
        })
      }}
    }//</div>}
  }).into_any()
}

fn lo_line(uri:&DocumentElementURI) -> impl IntoView + 'static {
  let archive = uri.archive_id().to_string();
  let name = uri.name().to_string();
  let lang = uri.language().flag_svg();
  view!(<div><span>"["{archive}"] "{name}" "</span><div style="display:contents;" inner_html=lang/></div>)
}

pub(super) fn do_arg<V:IntoView+'static>(children:impl FnOnce() -> V + Send + 'static) -> impl IntoView {
  let value : Option<InTermState> = None;
  view!{<Provider value>{children()}</Provider>}
}