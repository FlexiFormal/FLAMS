use flams_ontology::{narration::sections::SectionLevel, uris::{DocumentElementURI, NarrativeURI}};
use flams_web_utils::inject_css;
use leptos::{prelude::*,context::Provider};
use web_sys::HtmlDivElement;
use crate::{config::{IdPrefix, LogicalLevel, SectionCounters}, ts::{JsFun, JsOrRsF, NamedJsFunction, SectionContinuation, TsCont}};
use super::navigation::{NavElems, SectionOrInputref};


pub(super) fn section<V:IntoView+'static>(uri:DocumentElementURI,children:impl FnOnce() -> V + Send + 'static) -> impl IntoView {

  let id = expect_context::<IdPrefix>().new_id(uri.name().last_name().as_ref());
  NavElems::update_untracked(|ne| {
    ne.ids.insert(id.clone(),SectionOrInputref::Section);
  });
  inject_css("ftml-sections", include_str!("sections.css"));

  let end = use_context::<OnSectionEnd>().map(|s| s.view(&uri));
  let mut counters : SectionCounters = expect_context();
  let (style,cls) = match &mut counters.current {
    LogicalLevel::Section(l) => {
      *l = l.inc();
      (None,Some(match *l {
        SectionLevel::Part => "ftml-part",
        SectionLevel::Chapter => "ftml-chapter",
        SectionLevel::Section => "ftml-section",
        SectionLevel::Subsection => "ftml-subsection",
        SectionLevel::Subsubsection => "ftml-subsubsection",
        SectionLevel::Paragraph => "ftml-paragraph",
        SectionLevel::Subparagraph => "ftml-subparagraph",
      }))
    }
    LogicalLevel::None => {
      counters.current = LogicalLevel::Section(counters.max);
      (None,Some(match counters.max {
        SectionLevel::Part => "ftml-part",
        SectionLevel::Chapter => "ftml-chapter",
        SectionLevel::Section => "ftml-section",
        SectionLevel::Subsection => "ftml-subsection",
        SectionLevel::Subsubsection => "ftml-subsubsection",
        SectionLevel::Paragraph => "ftml-paragraph",
        SectionLevel::Subparagraph => "ftml-subparagraph",
      }))
    }
    _ => (Some("display:content"),None)
  };

  view!{
    <Provider value=IdPrefix(id.clone())>
      <Provider value=NarrativeURI::Element(uri)>
      <Provider value=counters>
      <div id=id style=style class=cls>
        {children()}
        {end}
      </div>
      </Provider>
      </Provider>
    </Provider>
  }
}

type SectCont = JsOrRsF<DocumentElementURI,Option<TsCont>>;

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

pub(super) fn title<V:IntoView+'static>(children:impl FnOnce() -> V + Send + 'static) -> impl IntoView {
  let counters : SectionCounters = expect_context();
  let begin = match counters.current {
    LogicalLevel::Section(l) => {
      if let Some(NarrativeURI::Element(uri)) = use_context() {
        use_context::<OnSectionBegin>().map(|s| s.view(&uri))
      } else {
        tracing::error!("Sectioning error");
        None
      }
    }
    _ => None
  };
  view!{
    <div class="ftml-title">{children()}</div>
    {begin}
  }
}