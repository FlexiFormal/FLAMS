//#![allow(non_local_definitions)]

use flams_ontology::{narration::paragraphs::ParagraphKind, uris::{DocumentElementURI, DocumentURI, Name, NarrativeURI}};
use flams_utils::CSS;
use flams_web_utils::do_css;
use leptos::{either::{Either, EitherOf4}, prelude::*};

use crate::components::navigation::NavElems;


#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
#[cfg_attr(feature="ts", derive(tsify_next::Tsify))]
#[serde(untagged)]
/// A Table of contents; Either:
/// 1. an already known TOC, consisting of a list of [`TOCElem`]s, or
/// 2. the URI of a Document. In that case, the relevant FLAMS server
///    will be requested to obtain the TOC for that document.
pub enum TOC {
    Full(Vec<TOCElem>),
    Get(
      DocumentURI
    )
}


#[derive(Debug,Clone,serde::Serialize,serde::Deserialize,PartialEq)]
#[cfg_attr(feature="ts", derive(tsify_next::Tsify))]
#[serde(tag = "type")]
/// An entry in a table of contents. Either:
/// 1. a section; the title is assumed to be an HTML string, or
/// 2. an inputref to some other document; the URI is the one for the
///    inputref itself; not the referenced Document. For the TOC,
///    which document is inputrefed is actually irrelevant.
pub enum TOCElem {
  /// A section; the title is assumed to be an HTML string
  Section{
    title:Option<String>,
    uri:DocumentElementURI,
    id:String,
    children:Vec<TOCElem>
  },
  /// An inputref to some other document; the URI is the one for the
  /// referenced Document.
  Inputref{
    uri:DocumentURI,
    title:Option<String>,
    id:String,
    children:Vec<TOCElem>
  },
  Paragraph{
    styles:Vec<Name>,
    kind:ParagraphKind,
  },
  Slide
}

pub trait TOCIter<'a> {
  fn elem_iter(&'a self) -> std::slice::Iter<'a,TOCElem>;
  fn iter_elems(&'a self) -> impl Iterator<Item=&'a TOCElem> {
    struct TOCIterator<'b> {
      curr:std::slice::Iter<'b,TOCElem>,
      stack:Vec<std::slice::Iter<'b,TOCElem>>
    }
    impl<'b> Iterator for TOCIterator<'b> {
      type Item = &'b TOCElem;
      fn next(&mut self) -> Option<Self::Item> {
        loop {
          if let Some(elem) = self.curr.next() {
            let children: &'b [_] = match elem {
              TOCElem::Section{children,..} |
              TOCElem::Inputref{children,..} => children,
              _ => return Some(elem)
            };
            self.stack.push(std::mem::replace(&mut self.curr,children.iter()));
            return Some(elem)
          } else if let Some(s) = self.stack.pop() {
            self.curr = s;
          } else { return None }
        }
      }
    }
    TOCIterator { curr: self.elem_iter(), stack: Vec::new() }
  }
  fn do_titles(&'a self) {
    NavElems::update_untracked(|nav| {
      for e in self.iter_elems() { 
        if let TOCElem::Inputref{title:Some(title),uri,..} = e {
          nav.set_title(uri.clone(), title.clone());
        }
      }
    });
  }
}
impl<'a,A> TOCIter<'a> for &'a A where A:std::ops::Deref<Target=[TOCElem]> {
  #[inline]
  fn elem_iter(&'a self) -> std::slice::Iter<'a,TOCElem> { self.deref().iter() }
}
impl<'a> TOCIter<'a> for &'a [TOCElem] {
  #[inline]
  fn elem_iter(&'a self) -> std::slice::Iter<'a,TOCElem> { self.iter() }
}

impl TOCElem {
  fn into_view(self) -> impl IntoView {
    use flams_web_utils::components::{AnchorLink,Header};
    use leptos_dyn_dom::DomStringCont;
    match self {
      Self::Section{title:Some(title),id,children,..} => {
        let id = format!("#{id}");
        Some(Either::Left(view!{
          <AnchorLink href=id>
            <Header slot>
              <DomStringCont html=title cont=crate::iterate/>
            </Header>
            {children.into_iter().map(Self::into_view).collect_view()}
          </AnchorLink>
        }))
      }
      Self::Section{title:None,children,..} |
        Self::Inputref{children,..} => {
        Some(Either::Right(children.into_iter().map(Self::into_view).collect_view().into_any()))
      }
      _ => None
    }
  }
}

#[component]
pub fn Toc(#[prop(optional)] css:Vec<CSS>,toc:Vec<TOCElem>) -> impl IntoView {
  use flams_web_utils::components::Anchor;
  use thaw::Scrollbar;
  for css in css { do_css(css); }
  view!{
    <div /*style="position:fixed;right:20px;z-index:5;background-color:inherit;"*/><Scrollbar style="max-height: 400px;"><Anchor>{
      toc.into_iter().map(TOCElem::into_view).collect_view()
    }</Anchor></Scrollbar></div>
  }
}

#[derive(Debug,Default,Clone,serde::Serialize,serde::Deserialize)]
pub enum TOCSource {
    #[default] None,
    Ready(Vec<TOCElem>),
    //Loading(Resource<Result<(Vec<CSS>,Vec<TOCElem>),ServerFnError<String>>>),
    Get
}

#[allow(clippy::match_wildcard_for_single_variants)]
pub fn do_toc<V:IntoView+'static>(toc:TOCSource,wrap:impl FnOnce(Option<AnyView>) -> V) -> impl IntoView {
    use TOCIter;
    match toc {
        TOCSource::None => EitherOf4::A(wrap(None)),
        TOCSource::Ready(toc) => {
          let ctw = expect_context::<RwSignal::<Option<Vec<TOCElem>>>>();
          ctw.set(Some(toc.clone()));
          EitherOf4::B(view!{
              {toc.as_slice().do_titles()}
              {wrap(Some(view!(<Toc toc/>).into_any()))}
          })
        }
        TOCSource::Get => match expect_context() {
            NarrativeURI::Document(uri) => {
                let r = Resource::new(|| (),move |()| crate::remote::server_config.get_toc(uri.clone()));
                EitherOf4::C(view!{
                    {move || r.with(|r| if let Some(Ok((_,toc))) = r {
                        toc.as_slice().do_titles();
                        let ctw = expect_context::<RwSignal::<Option<Vec<TOCElem>>>>();
                        ctw.set(Some(toc.clone()));
                    })}
                    {wrap(Some((move || r.get().map_or_else(
                        || Either::Left(view!(<flams_web_utils::components::Spinner/>)),
                        |r| Either::Right(match r {
                            Ok((css,toc)) => {
                                for c in css { do_css(c); }
                                Some(view!(<Toc toc/>))
                            }
                            Err(e) => {
                                tracing::error!(e);
                                None
                            }
                        })
                    )).into_any()))}
                })
            }
            _ => EitherOf4::D(wrap(None))
        }
    }
}
