//#![allow(non_local_definitions)]

use crate::components::navigation::NavElems;
use flams_ontology::{
    narration::paragraphs::ParagraphKind,
    uris::{DocumentElementURI, DocumentURI, Name, NarrativeURI},
};
use flams_utils::{time::Timestamp, CSS};
use flams_web_utils::do_css;
use leptos::{
    either::{Either, EitherOf4},
    prelude::*,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[cfg_attr(feature = "ts", derive(tsify_next::Tsify))]
/// A section that has been "covered" at the specified timestamp; will be marked accordingly
/// in the TOC.
pub struct Gotto {
    pub uri: DocumentElementURI,
    #[serde(default)]
    pub timestamp: Option<Timestamp>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[cfg_attr(feature = "ts", derive(tsify_next::Tsify))]
#[serde(tag = "type")]
/// An entry in a table of contents. Either:
/// 1. a section; the title is assumed to be an HTML string, or
/// 2. an inputref to some other document; the URI is the one for the
///    inputref itself; not the referenced Document. For the TOC,
///    which document is inputrefed is actually irrelevant.
pub enum TOCElem {
    /// A section; the title is assumed to be an HTML string
    Section {
        title: Option<String>,
        uri: DocumentElementURI,
        id: String,
        children: Vec<TOCElem>,
    },
    SkippedSection {
        children: Vec<TOCElem>,
    },
    /// An inputref to some other document; the URI is the one for the
    /// referenced Document.
    Inputref {
        uri: DocumentURI,
        title: Option<String>,
        id: String,
        children: Vec<TOCElem>,
    },
    Paragraph {
        //uri:DocumentElementURI,
        styles: Vec<Name>,
        kind: ParagraphKind,
    },
    Slide, //{uri:DocumentElementURI}
}

pub trait TOCIter<'a> {
    fn elem_iter(&'a self) -> std::slice::Iter<'a, TOCElem>;
    fn iter_elems(&'a self) -> impl Iterator<Item = &'a TOCElem> {
        struct TOCIterator<'b> {
            curr: std::slice::Iter<'b, TOCElem>,
            stack: Vec<std::slice::Iter<'b, TOCElem>>,
        }
        impl<'b> Iterator for TOCIterator<'b> {
            type Item = &'b TOCElem;
            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    if let Some(elem) = self.curr.next() {
                        let children: &'b [_] = match elem {
                            TOCElem::Section { children, .. }
                            | TOCElem::Inputref { children, .. }
                            | TOCElem::SkippedSection { children } => children,
                            _ => return Some(elem),
                        };
                        self.stack
                            .push(std::mem::replace(&mut self.curr, children.iter()));
                        return Some(elem);
                    } else if let Some(s) = self.stack.pop() {
                        self.curr = s;
                    } else {
                        return None;
                    }
                }
            }
        }
        TOCIterator {
            curr: self.elem_iter(),
            stack: Vec::new(),
        }
    }
    fn do_titles(&'a self) {
        NavElems::update_untracked(|nav| {
            for e in self.iter_elems() {
                if let TOCElem::Inputref {
                    title: Some(title),
                    uri,
                    ..
                } = e
                {
                    nav.set_title(uri.clone(), title.clone());
                }
            }
            nav.initialized.set(true);
        });
    }
}
impl<'a, A> TOCIter<'a> for &'a A
where
    A: std::ops::Deref<Target = [TOCElem]>,
{
    #[inline]
    fn elem_iter(&'a self) -> std::slice::Iter<'a, TOCElem> {
        self.deref().iter()
    }
}
impl<'a> TOCIter<'a> for &'a [TOCElem] {
    #[inline]
    fn elem_iter(&'a self) -> std::slice::Iter<'a, TOCElem> {
        self.iter()
    }
}

impl TOCElem {
    fn into_view(self, gottos: &mut Gottos) -> impl IntoView + use<> {
        use flams_web_utils::components::{AnchorLink, Header};
        use leptos_dyn_dom::DomStringCont;
        let style = if gottos.current.is_some() {
            "background-color:var(--colorPaletteYellowBorder1);"
        } else {
            ""
        };
        let after = gottos.current.as_ref().and_then(|e| e.timestamp).map(|ts| {
            view! {
                <sup><i>" Covered: "{ts.into_date().to_string()}</i></sup>
            }
        });
        match self {
            Self::Section {
                title: Some(title),
                id,
                children,
                uri,
                ..
            } => {
                gottos.next(&uri);
                let id = format!("#{id}");
                let ch = children
                    .into_iter()
                    .map(|e| e.into_view(gottos))
                    .collect_view();
                Some(Either::Left(view! {
                  <AnchorLink href=id>
                    <Header slot>
                      <div style=style><DomStringCont html=title cont=crate::iterate/>{after}</div>
                    </Header>
                    {ch}
                  </AnchorLink>
                }))
            }
            Self::Section {
                title: None,
                children,
                uri,
                ..
            } => {
                gottos.next(&uri);
                Some(Either::Right(
                    children
                        .into_iter()
                        .map(|e| e.into_view(gottos))
                        .collect_view()
                        .into_any(),
                ))
            }
            Self::Inputref { children, .. } | Self::SkippedSection { children } => {
                Some(Either::Right(
                    children
                        .into_iter()
                        .map(|e| e.into_view(gottos))
                        .collect_view()
                        .into_any(),
                ))
            }
            _ => None,
        }
    }
}

struct Gottos {
    current: Option<Gotto>,
    iter: std::vec::IntoIter<Gotto>,
}
impl Gottos {
    fn next(&mut self, uri: &DocumentElementURI) {
        if let Some(c) = self.current.as_ref() {
            if c.uri == *uri {
                self.current = self.iter.next();
            }
        }
    }
}

#[component]
pub fn Toc(
    #[prop(optional)] css: Vec<CSS>,
    toc: Vec<TOCElem>,
    mut gottos: Vec<Gotto>,
) -> impl IntoView {
    use flams_web_utils::components::Anchor;
    use thaw::Scrollbar;
    for css in css {
        do_css(css);
    }
    //let max = with_context::<SectionCounters>(|ctrs| ctrs.max).unwrap_or(SectionLevel::Section);

    gottos.retain(|e| {
        toc.as_slice().iter_elems().any(|s| {
            if let TOCElem::Section { uri, .. } = s {
                *uri == e.uri
            } else {
                false
            }
        })
    });
    //gottos.sort_by_key(|k| k.timestamp.unwrap_or_default());
    let mut gottos = gottos.into_iter();
    let current = gottos.next();
    let mut gottos = Gottos {
        current,
        iter: gottos,
    };
    view! {
      <div><Scrollbar style="max-height: 400px;"><Anchor>{
        toc.into_iter().map(|e| e.into_view(&mut gottos)).collect_view()
      }</Anchor></Scrollbar></div>
    }
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub enum TOCSource {
    #[default]
    None,
    Ready(Vec<TOCElem>),
    //Loading(Resource<Result<(Vec<CSS>,Vec<TOCElem>),ServerFnError<String>>>),
    Get,
}

#[allow(clippy::match_wildcard_for_single_variants)]
pub fn do_toc<V: IntoView + 'static>(
    toc: TOCSource,
    gottos: Vec<Gotto>,
    wrap: impl FnOnce(Option<AnyView>) -> V,
) -> impl IntoView {
    use TOCIter;

    /* ------------------------
    let gottos = vec![Gotto {
        uri: "https://mathhub.info?a=courses/FAU/AI/course&p=course/sec&d=ml&l=en&e=section"
            .parse()
            .unwrap(),
        timestamp: Some(Timestamp::now()),
    }];
    // ------------------------ */
    match toc {
        TOCSource::None => EitherOf4::A(wrap(None)),
        TOCSource::Ready(toc) => {
            let ctw = expect_context::<RwSignal<Option<Vec<TOCElem>>>>();
            ctw.set(Some(toc.clone()));
            EitherOf4::B(view! {
                {toc.as_slice().do_titles()}
                {wrap(Some(view!(<Toc toc gottos/>).into_any()))}
            })
        }
        TOCSource::Get => match expect_context() {
            NarrativeURI::Document(uri) => {
                let r = Resource::new(
                    || (),
                    move |()| crate::remote::server_config.get_toc(uri.clone()),
                );
                EitherOf4::C(view! {
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
                                Some(view!(<Toc toc gottos=gottos.clone()/>))
                            }
                            Err(e) => {
                                tracing::error!(e);
                                None
                            }
                        })
                    )).into_any()))}
                })
            }
            _ => EitherOf4::D(wrap(None)),
        },
    }
}
