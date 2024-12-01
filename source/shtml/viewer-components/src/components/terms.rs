use immt_ontology::{content::terms::ArgMode, uris::{ArchiveURITrait, ContentURI, DocumentElementURI, URIWithLanguage, URI}};
use immt_web_utils::{components::{DivOrMrow, Popover, OnClickModal,PopoverSize, PopoverTriggerType}, do_css, inject_css};
use leptos::{context::Provider, either::{Either, EitherOf3}, prelude::*};
use leptos_dyn_dom::{DomCont, OriginalNode};
use shtml_extraction::open::terms::{OpenArg, OpenTerm, PreVar, VarOrSym};

use crate::{components::{IntoLOs, LOs}, SHTMLString};

#[cfg(feature="omdoc")]
enum DomTermArgs {
  Open(Vec<Option<(ArgMode,either::Either<String,Vec<Option<String>>>)>>),
  Closed(Vec<(ArgMode,either::Either<String,Vec<String>>)>)
}

#[derive(Clone)]
pub(super) struct InTermState {
  owner:VarOrSym,
  is_hovered:RwSignal<bool>,
  #[cfg(feature="omdoc")]
  args:RwSignal<DomTermArgs>
}
impl InTermState {
  fn new(owner:VarOrSym) -> Self {
    Self {
      owner,
      is_hovered:RwSignal::new(false),
      #[cfg(feature="omdoc")]
      args:RwSignal::new(DomTermArgs::Open(Vec::new()))
    }
  }
}

#[derive(Clone)]
pub(super) struct DisablePopover;

#[cfg(feature="omdoc")]
pub(super) fn math_term(skip:usize,elements:shtml_extraction::prelude::SHTMLElements,orig:OriginalNode,on_load:RwSignal<bool>,t:OpenTerm) -> impl IntoView {
  /* 
  let head = InTermState::new(t.take_head());

  let args = head.args;
  Effect::new(move || if on_load.get() {
    if args.with_untracked(|e| matches!(e,DomTermArgs::Open(_))) {
      args.update(|args| {
        let DomTermArgs::Open(v) = args else {unreachable!()};
        tracing::info!("Closing term with {} arguments",v.len());
        let mut v = std::mem::take(v).into_iter();
        let mut ret = Vec::new();
        while let Some(Some((mode,s))) = v.next() {
          match (mode,s) {
            (ArgMode::Normal|ArgMode::Binding, either::Left(s)) => ret.push((mode,either::Left(s))),
            (ArgMode::Sequence|ArgMode::BindingSequence, either::Right(v)) => {
              let mut r = Vec::new();
              let mut iter = v.into_iter();
              while let Some(Some(s)) = iter.next() {
                r.push(s);
              }
              for a in iter {
                if a.is_some() {
                  tracing::error!("Missing argument in associative argument list");
                }
              }
              ret.push((mode,either::Right(r)));
            }
            (ArgMode::Sequence|ArgMode::BindingSequence,either::Left(s)) => ret.push((mode,either::Right(vec![s]))),
            (ArgMode::Normal|ArgMode::Binding,_) => tracing::error!("Argument of mode {mode:?} has multiple entries"),
          }
        }
        for e in v {
          if e.is_some() {
            tracing::error!("Missing argument in term");
          }
        }
        *args = DomTermArgs::Closed(ret);
      });
    }
  });
  let uri = match &head.owner {
    VarOrSym::S(s@ContentURI::Symbol(_)) => Some((false,URI::Content(s.clone()))),
    VarOrSym::V(PreVar::Resolved(v)) => Some((true,URI::Narrative(v.clone().into()))),
    _ => None
  };
  let notation_signal = 
    uri.as_ref().map(|(_,uri)| expect_context::<crate::NotationForces>().get(&uri));
  view!{<Provider value=Some(head)>{
    if let Some(notation_signal) = notation_signal {
      Either::Left(move || {
        if let Some(u) = notation_signal.get() {
          Either::Left(view!{
            <mtext style="color:red">TODO</mtext>
          })
        } else {
          Either::Right(
            super::do_components::<true>(skip+1,elements.clone(),orig.clone(),RwSignal::new(false)).into_any()
          )
        }
      })
    } else {
      Either::Right(
        super::do_components::<true>(skip+1,elements,orig,on_load).into_any()
      )
    }
  }</Provider>}
  */
  do_term::<_,true>(t,move || 
    super::do_components::<true>(skip+1,elements,orig,on_load).into_any()
  )
}

pub(super) fn do_term<V:IntoView+'static,const MATH:bool>(t:OpenTerm,children:impl FnOnce() -> V + Send + 'static) -> impl IntoView + 'static {
  let head = InTermState::new(t.take_head());
  view!{
    <Provider value=Some(head)>{
      children()
    }</Provider>
  }
}

pub(super) fn do_comp<V:IntoView+'static,const MATH:bool>(children:impl FnOnce() -> V + Send + 'static) -> impl IntoView {
  use immt_web_utils::components::PopoverTrigger;
  let in_term = use_context::<Option<InTermState>>().flatten();
  if let Some(in_term) = in_term {
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
    let do_popover = || use_context::<DisablePopover>().is_none();
    let s = in_term.owner;
    let node_type = if MATH { DivOrMrow::Mrow } else { DivOrMrow::Div };
    
    if do_popover() {
      let s_click = s.clone();
      Either::Left(view!(
        <Popover node_type class size=PopoverSize::Small on_open=move || is_hovered.set(true) on_close=move || is_hovered.set(false)>
          <PopoverTrigger class slot>{children()}</PopoverTrigger>
          <OnClickModal slot>{do_onclick(s_click)}</OnClickModal>
          //<div style="max-width:600px;">
            {match s {
              VarOrSym::V(v) => EitherOf3::A(view!{<span>"Variable "{v.name().last_name().to_string()}</span>}),
              VarOrSym::S(ContentURI::Symbol(s)) => EitherOf3::B(crate::config::get!(definition(s.clone()) = (css,s) => {
                for c in css { do_css(c); }
                Some(view!(<div style="color:black;background-color:white;padding:3px;"><SHTMLString html=s/></div>))
              })),
              VarOrSym::S(ContentURI::Module(m)) =>
                EitherOf3::C(view!{<div>"Module" {m.name().last_name().to_string()}</div>}),
          }}//</div>
        </Popover>
      ))
    } else { Either::Right(children()) }
  } else { Either::Right(children()) }
}

fn do_onclick(uri:VarOrSym) -> impl IntoView {
  use thaw::{Combobox,ComboboxOption,ComboboxOptionGroup,Divider};
  #[cfg(feature="omdoc")]
  let uriclone = uri.clone();
  let s = match uri {
    VarOrSym::V(v) => return EitherOf3::A(view!{<span>"Variable "{v.name().last_name().to_string()}</span>}),
    VarOrSym::S(ContentURI::Module(m)) =>
      return EitherOf3::B(view!{<div>"Module" {m.name().last_name().to_string()}</div>}),
    VarOrSym::S(ContentURI::Symbol(s)) => s
  };
  let name = s.name().last_name().to_string();

  EitherOf3::C(crate::config::get!(get_los(s.clone(),false) = v => {
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
      {#[cfg(feature="omdoc")]{

        let uri = match &uriclone {
          VarOrSym::S(s@ContentURI::Symbol(_)) => Some((false,URI::Content(s.clone()))),
          VarOrSym::V(PreVar::Resolved(v)) => Some((true,URI::Narrative(v.clone().into()))),
          _ => None
        };
        uri.map(|(is_variable,uri)| {let uricl = uri.clone();crate::config::get!(notations(uri.clone()) = v => {
          if v.is_empty() { None } else {Some({
            let uri = uricl.clone();
            let notation_signal = expect_context::<crate::NotationForces>().get(&uri);
            let selected = RwSignal::new(notation_signal.with_untracked(|v| 
              if let Some(v) = v { v.to_string()} else {"None".to_string()}
            ));
            Effect::new(move || {
              let Some(v) = selected.try_get() else {return};
              if v == "None" { notation_signal.maybe_update(|f|
                if f.is_some() {
                  *f = None; true
                } else {false}
              ); }
              else {
                let uri = v.parse().expect("This should be impossible");
                notation_signal.maybe_update(|v| match v {
                  Some(e) if *e == uri => false,
                  _ => {
                    *v = Some(uri); true
                  }
                })
              }
            });
            view!{<div style="margin:5px;"><Divider/></div>
            <div style="width:100%;"><div style="width:min-content;margin-left:auto;">
              <Combobox selected_options=selected placeholder="Force Notation">
                <ComboboxOption text="None" value="None">"None"</ComboboxOption>
                {let uri = uri;
                  v.into_iter().map(|(u,n)| {let html = n.display_shtml(false,is_variable,&uri).to_string();view!{
                    <ComboboxOption text="" value=u.to_string()>{
                      view!(
                        <Provider value=DisablePopover>
                            <crate::SHTMLStringMath html/>
                        </Provider>
                      )
                    }</ComboboxOption>
                  }}).collect_view()
                }
              </Combobox>
            </div></div>}
          })}
        })})
      }}
    }//</div>}
  }))
}

fn lo_line(uri:&DocumentElementURI) -> impl IntoView + 'static {
  let archive = uri.archive_id().to_string();
  let name = uri.name().to_string();
  let lang = uri.language().flag_svg();
  view!(<div><span>"["{archive}"] "{name}" "</span><div style="display:contents;" inner_html=lang/></div>)
}

pub(super) fn do_arg<V:IntoView + 'static>(orig:OriginalNode,arg:OpenArg,cont:impl FnOnce(OriginalNode) -> V + Send + 'static) -> impl IntoView {
  #[cfg(feature="omdoc")]
  {
    use immt_ontology::shtml::SHTMLKey;
    let tm = use_context::<Option<InTermState>>().flatten();
    if let Some(tm) = tm {
      tm.args.update_untracked(|args|
        if let DomTermArgs::Open(v) = args {
          let (index,sub) = match arg.index {
            either::Left(i) => ((i-1) as usize,None),
            either::Right((i,m)) => ((i-1) as usize,Some((m - 1) as usize))
          };
          if v.len() <= index { v.resize(index + 1, None); }
          let entry = &mut v[index];
          if let Some(sub) = sub {
            if let (_,either::Right(subs)) = entry.get_or_insert_with(|| (arg.mode,either::Right(Vec::new()))) {
              if subs.len() <= sub { subs.resize(sub + 1, None); }
              let entry = &mut subs[sub];
              *entry = Some(orig.html_string());
            } else {
              tracing::error!("{} is not a list",SHTMLKey::Arg.attr_name());
            }
          } else {
            *entry = Some((arg.mode,either::Left(orig.html_string())));
          }
        }
      );
    } else {
      tracing::error!("{} outside of a term",SHTMLKey::Arg.attr_name());
    }

  }

  let value : Option<InTermState> = None;
  view!{<Provider value>{cont(orig)}</Provider>}
}