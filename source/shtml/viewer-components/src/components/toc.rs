#![allow(non_local_definitions)]

use immt_ontology::uris::{DocumentElementURI, DocumentURI};
use immt_utils::CSS;
use immt_web_utils::do_css;
use leptos::prelude::*;


#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
#[cfg_attr(feature="ts", derive(tsify_next::Tsify))]
/// A Table of contents; Either:
/// 1. an already known TOC, consisting of a list of [`TOCElem`]s, or
/// 2. the URI of a Document. In that case, the relevant iMMT server
///    will be requested to obtain the TOC for that document.
pub enum TOC {
    Full(Vec<TOCElem>),
    Get(
      #[cfg_attr(feature="ts", tsify(type = "string"))]
      DocumentURI
    )
}


#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
#[cfg_attr(feature="ts", derive(tsify_next::Tsify))]
/// An entry in a table of contents. Either:
/// 1. a section; the title is assumed to be an HTML string, or
/// 2. an inputref to some other document; the URI is the one for the
///    inputref itself; not the referenced Document. For the TOC,
///    which document is inputrefed is actually irrelevant.
pub enum TOCElem {
  /// A section; the title is assumed to be an HTML string
  Section{
    title:Option<String>,
    #[cfg_attr(feature="ts", tsify(type = "string"))]
    uri:DocumentElementURI,
    id:String,
    children:Vec<TOCElem>
  },
  /// An inputref to some other document; the URI is the one for the
  /// inputref itself; not the referenced Document. For the TOC,
  /// which document is inputrefed is actually irrelevant.
  Inputref{
    #[cfg_attr(feature="ts", tsify(type = "string"))]
    uri:DocumentElementURI,
    id:String,
    children:Vec<TOCElem>
  }
}

impl TOCElem {
  fn into_view(self) -> impl IntoView {
    use immt_web_utils::components::{AnchorLink,Header};
    use leptos_dyn_dom::DomStringCont;
    match self {
      Self::Section{title:Some(title),id,children,..} => {
        let id = format!("#{id}");
        view!{
          <AnchorLink href=id>
            <Header slot>
              <DomStringCont html=title cont=crate::iterate/>
            </Header>
            {children.into_iter().map(Self::into_view).collect_view()}
          </AnchorLink>
        }.into_any()
      }
      Self::Section{title:None,children,..} |
        Self::Inputref{children,..} => {
        children.into_iter().map(Self::into_view).collect_view().into_any()
      }
    }
  }
}

#[component]
pub fn Toc(css:Vec<CSS>,toc:Vec<TOCElem>) -> impl IntoView {
  use immt_web_utils::components::Anchor;
  use thaw::Scrollbar;
  for css in css { do_css(css); }
  leptos::logging::log!("toc: {toc:?}");
  view!{
    <div style="position:fixed;right:20px;z-index:5;background-color:var(--colorNeutralBackground1);"><Scrollbar style="max-height: 400px;"><Anchor>{
      toc.into_iter().map(TOCElem::into_view).collect_view()
    }</Anchor></Scrollbar></div>
  }
}