use crate::ssr::insert_base_url;

use super::ssr::backend;
use flams_ontology::narration::{
    DocumentElement, NarrationTrait, documents::Document, problems::Problem, sections::Section,
};
use flams_system::backend::Backend;
use flams_utils::{CSS, unwrap, vecmap::VecSet};
use ftml_viewer_components::components::TOCElem;

pub async fn from_document(doc: &Document) -> (Vec<CSS>, Vec<TOCElem>) {
    let mut curr = doc.children().iter();
    let mut prefix = String::new();
    let mut stack = Vec::new();
    let mut ret = Vec::new();
    let mut css = VecSet::new();
    loop {
        while let Some(elem) = curr.next() {
            match elem {
                DocumentElement::Slide {
                    /*uri,*/ children, ..
                } => {
                    let old = std::mem::replace(&mut curr, children.iter());
                    stack.push((old, None));
                    ret.push(TOCElem::Slide /*{uri:uri.clone()}*/);
                }
                DocumentElement::Section(Section {
                    uri,
                    title,
                    children,
                    ..
                }) => {
                    let old = std::mem::replace(&mut curr, children.iter());
                    let title = if let Some(title) = title {
                        if let Some((c, h)) = backend!(get_html_fragment(uri.document(), *title)) {
                            for c in c {
                                css.insert(c);
                            }
                            Some(h)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    stack.push((
                        old,
                        Some(TOCElem::Section {
                            title, // TODO
                            id: prefix.clone(),
                            uri: uri.clone(),
                            children: std::mem::take(&mut ret),
                        }),
                    ));
                    prefix = if prefix.is_empty() {
                        uri.name().last_name().to_string()
                    } else {
                        format!("{prefix}/{}", uri.name().last_name())
                    };
                }
                DocumentElement::DocumentReference { id, target, .. } if target.is_resolved() => {
                    let d = unwrap!(target.get());
                    let title = d.title().map(ToString::to_string);
                    let uri = d.uri().clone();
                    let old = std::mem::replace(&mut curr, d.children().iter());
                    stack.push((
                        old,
                        Some(TOCElem::Inputref {
                            id: prefix.clone(),
                            uri,
                            title,
                            children: std::mem::take(&mut ret),
                        }),
                    ));
                    prefix = if prefix.is_empty() {
                        id.name().last_name().to_string()
                    } else {
                        format!("{prefix}/{}", id.name().last_name())
                    };
                }
                DocumentElement::Paragraph(p) => {
                    ret.push(TOCElem::Paragraph {
                        styles: p.styles.clone().into_vec(),
                        kind: p.kind, /*,uri:p.uri.clone()*/
                    });
                }
                DocumentElement::Module { children, .. }
                | DocumentElement::Morphism { children, .. }
                | DocumentElement::MathStructure { children, .. }
                | DocumentElement::Extension { children, .. }
                | DocumentElement::Problem(Problem { children, .. }) => {
                    let old = std::mem::replace(&mut curr, children.iter());
                    stack.push((old, None));
                }
                DocumentElement::SkipSection(children) => {
                    let old = std::mem::replace(&mut curr, children.iter());
                    stack.push((
                        old,
                        Some(TOCElem::SkippedSection {
                            children: std::mem::take(&mut ret),
                        }),
                    ));
                }
                DocumentElement::SetSectionLevel(_)
                | DocumentElement::SymbolDeclaration(_)
                | DocumentElement::SymbolReference { .. }
                | DocumentElement::Notation { .. }
                | DocumentElement::VariableNotation { .. }
                | DocumentElement::Variable(_)
                | DocumentElement::Definiendum { .. }
                | DocumentElement::VariableReference { .. }
                | DocumentElement::TopTerm { .. }
                | DocumentElement::UseModule { .. }
                | DocumentElement::ImportModule { .. }
                | DocumentElement::DocumentReference { .. } => (), //_ => ()
            }
        }
        match stack.pop() {
            None => break,
            Some((
                iter,
                Some(TOCElem::Inputref {
                    mut id,
                    uri,
                    title,
                    mut children,
                }),
            )) => {
                curr = iter;
                std::mem::swap(&mut prefix, &mut id);
                std::mem::swap(&mut ret, &mut children);
                if !children.is_empty() {
                    ret.push(TOCElem::Inputref {
                        id,
                        uri,
                        title,
                        children,
                    });
                }
            }
            Some((
                iter,
                Some(TOCElem::Section {
                    mut id,
                    uri,
                    title,
                    mut children,
                }),
            )) => {
                curr = iter;
                std::mem::swap(&mut prefix, &mut id);
                std::mem::swap(&mut ret, &mut children);
                if title.is_some() || !children.is_empty() {
                    ret.push(TOCElem::Section {
                        id,
                        uri,
                        title,
                        children,
                    });
                }
            }
            Some((iter, Some(TOCElem::SkippedSection { mut children }))) => {
                curr = iter;
                std::mem::swap(&mut ret, &mut children);
                if !children.is_empty() {
                    ret.push(TOCElem::SkippedSection { children });
                }
            }
            Some((iter, _)) => curr = iter,
        }
    }
    (insert_base_url(css.0), ret)
}
