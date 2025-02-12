use flams_ontology::{narration::paragraphs::ParagraphKind, uris::DocumentElementURI};
use flams_web_utils::inject_css;
use leptos::{prelude::*,context::Provider};

use crate::config::{LogicalLevel, SectionCounters};

pub(super) fn paragraph<V:IntoView+'static>(kind:ParagraphKind,uri:DocumentElementURI,styles:Box<[Box<str>]>,children:impl FnOnce() -> V + Send + 'static) -> impl IntoView {
  let mut counters : SectionCounters = expect_context();
  inject_css("ftml-sections", include_str!("sections.css"));
  counters.current = LogicalLevel::Paragraph;
  let prefix = match kind {
    ParagraphKind::Assertion => Some("ftml-assertion"),
    ParagraphKind::Definition => Some("ftml-definition"),
    ParagraphKind::Example => Some("ftml-example"),
    ParagraphKind::Paragraph => Some("ftml-paragraph"),
    _ => None
  };
  let cls = prefix.map(|p| {
    let mut s = String::new();
    s.push_str(p);
    for style in styles {
      s.push(' ');
      s.push_str(p);
      s.push('-');
      s.push_str(&style);
    }
    s
  });

  view!{
    <Provider value=counters>
    <div class=cls>{children()}</div>
    </Provider>
  }
}
