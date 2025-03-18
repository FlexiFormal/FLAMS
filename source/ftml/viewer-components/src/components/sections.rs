use flams_ontology::{narration::sections::SectionLevel, uris::{DocumentElementURI, NarrativeURI}};
use flams_web_utils::inject_css;
use leptos::{prelude::*,context::Provider};
use web_sys::HtmlDivElement;
use crate::{components::counters::{LogicalLevel, SectionCounters}, config::IdPrefix, ts::{JsFun, JsOrRsF, NamedJsFunction, OnSectionTitle, SectionContinuation, TsCont}};
use super::navigation::{NavElems, SectionOrInputref};


pub(super) fn section<V:IntoView+'static>(uri:DocumentElementURI,children:impl FnOnce() -> V + Send + 'static) -> impl IntoView {

  let id = expect_context::<IdPrefix>().new_id(uri.name().last_name().as_ref());
  NavElems::update_untracked(|ne| {
    ne.ids.insert(id.clone(),SectionOrInputref::Section);
  });
  inject_css("ftml-sections", include_str!("sections.css"));

  //let end = use_context::<OnSectionEnd>().map(|s| s.view(&uri));
  let mut counters : SectionCounters = expect_context();
  let (style,cls) = counters.next_section();
  let lvl = counters.current_level();

  view!{
    <Provider value=IdPrefix(id.clone())>
      <Provider value=NarrativeURI::Element(uri.clone())>
      <Provider value=counters>
      <div id=id style=style class=cls>
        {
          if let LogicalLevel::Section(lvl) = lvl {
            leptos::either::Either::Left(SectionContinuation::wrap(&(uri,lvl) ,children()))
          } else {
            leptos::either::Either::Right(children())
          }
        }
        //{end}
      </div>
      </Provider>
      </Provider>
    </Provider>
  }
}

pub(super) fn title<V:IntoView+'static>(children:impl FnOnce() -> V + Send + 'static) -> impl IntoView {
  let counters : SectionCounters = expect_context();
  let (begin,cls) = match counters.current_level() {
    LogicalLevel::Section(l) => {
      (if let Some(NarrativeURI::Element(uri)) = use_context() {
        expect_context::<Option<OnSectionTitle>>().map(|s| 
          TsCont::res_into_view(s.0.apply(&(uri,l)))
        )
      } else {
        tracing::error!("Sectioning error");
        None
      },match l {
        SectionLevel::Part => "ftml-title-part",
        SectionLevel::Chapter => "ftml-title-chapter",
        SectionLevel::Section => "ftml-title-section",
        SectionLevel::Subsection => "ftml-title-subsection",
        SectionLevel::Subsubsection => "ftml-title-subsubsection",
        SectionLevel::Paragraph => "ftml-title-paragraph",
        SectionLevel::Subparagraph => "ftml-title-subparagraph",
      })
    }
    LogicalLevel::BeamerSlide => (None,"ftml-title-slide"),
    LogicalLevel::Paragraph => (None,"ftml-title-paragraph"),
    _ => (None,"ftml-title")
  };
  view!{
    <div class=cls>{children()}</div>
    {begin}
  }
}



pub(super) fn skip<V:IntoView+'static>(children:impl FnOnce() -> V + Send + 'static) -> impl IntoView {
  let mut counters : SectionCounters = expect_context();
  match counters.current_level() {
    LogicalLevel::Section(l) => {
      counters.current = LogicalLevel::Section(l.inc());//.set_section(l.inc());
    }
    LogicalLevel::None => {
      counters.current = LogicalLevel::Section(counters.max);//.set_section(counters.max);
    }
    _ => ()
  };

  view!{
      <Provider value=counters>
        {children()}
    </Provider>
  }
}