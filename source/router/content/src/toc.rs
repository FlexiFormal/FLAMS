use flams_math_archives::backend::LocalBackend;
use flams_utils::vecmap::VecSet;
use ftml_ontology::{
    narrative::{
        documents::{Document, TocElem},
        elements::{DocumentElement, Problem, Section},
    },
    utils::Css,
};

pub async fn from_document(doc: Document) -> (Box<[Css]>, Box<[TocElem]>) {
    let (css, e) = from_document_i(doc, String::new(), VecSet::default()).await;
    (css.0.into_boxed_slice(), e)
}

fn from_document_i(
    doc: Document,
    mut prefix: String,
    mut css: VecSet<Css>,
) -> impl Future<Output = (VecSet<Css>, Box<[TocElem]>)> + Send {
    use flams_system::backend::backend;
    async move {
        let mut curr = doc.elements.iter();
        let mut stack = Vec::new();
        let mut ret = Vec::new();
        loop {
            while let Some(elem) = curr.next() {
                match elem {
                    DocumentElement::Slide(ftml_ontology::narrative::elements::Slide {
                        /*uri,*/ children,
                        ..
                    }) => {
                        let old = std::mem::replace(&mut curr, children.iter());
                        stack.push((old, None));
                        ret.push(TocElem::Slide /*{uri:uri.clone()}*/);
                    }
                    DocumentElement::Section(Section {
                        uri,
                        title,
                        children,
                        ..
                    }) => {
                        let old = std::mem::replace(&mut curr, children.iter());
                        stack.push((
                            old,
                            Some(TocElem::Section {
                                title: title.clone(), // TODO
                                id: prefix.clone(),
                                uri: uri.clone(),
                                children: std::mem::take(&mut ret),
                            }),
                        ));
                        prefix = if prefix.is_empty() {
                            uri.name().last().to_string()
                        } else {
                            format!("{prefix}/{}", uri.name().last())
                        };
                    }
                    DocumentElement::DocumentReference { uri, target, .. } => {
                        let Ok(d) = backend()
                            .get_document_async::<flams_system::TokioEngine>(target)
                            .await
                        else {
                            continue;
                        };
                        let title = d.title.clone();
                        let mut id = prefix.clone();

                        prefix = if prefix.is_empty() {
                            uri.name().last().to_string()
                        } else {
                            format!("{prefix}/{}", uri.name().last())
                        };
                        let fut =
                            Box::pin(from_document_i(d, prefix.clone(), std::mem::take(&mut css)))
                                as std::pin::Pin<
                                    Box<dyn Future<Output = (_, Box<[TocElem]>)> + Send>,
                                >;
                        let (ncss, children) = fut.await;
                        css = ncss;

                        std::mem::swap(&mut prefix, &mut id);
                        if !children.is_empty() {
                            ret.push(TocElem::Inputref {
                                uri: target.clone(),
                                title,
                                id,
                                children: children.into_vec(),
                            });
                        }

                        /*
                        let old = std::mem::replace(&mut curr, d.elements.iter());
                        stack.push((
                            old,
                            Some(TocElem::Inputref {
                                id: prefix.clone(),
                                uri: target.clone(),
                                title,
                                children: std::mem::take(&mut ret),
                            }),
                        ));
                        prefix = if prefix.is_empty() {
                            uri.name().last().to_string()
                        } else {
                            format!("{prefix}/{}", uri.name().last())
                        };
                        */
                    }
                    DocumentElement::Paragraph(p) => {
                        ret.push(TocElem::Paragraph {
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
                            Some(TocElem::SkippedSection {
                                children: std::mem::take(&mut ret),
                            }),
                        ));
                    }
                    DocumentElement::SymbolDeclaration(_)
                    | DocumentElement::SymbolReference { .. }
                    | DocumentElement::Notation { .. }
                    | DocumentElement::VariableNotation { .. }
                    | DocumentElement::VariableDeclaration(_)
                    | DocumentElement::Definiendum { .. }
                    | DocumentElement::VariableReference { .. }
                    | DocumentElement::Term { .. }
                    | DocumentElement::UseModule { .. }
                    | DocumentElement::ImportModule { .. } => (), //_ => ()
                }
            }
            match stack.pop() {
            None => break,
            Some((
                _,
                Some(TocElem::Inputref {
                    /*mut id,
                    uri,
                    title,
                    mut children*/
                    ..
                }),
            )) => unreachable!(), /*{
            curr = iter;
            std::mem::swap(&mut prefix, &mut id);
            std::mem::swap(&mut ret, &mut children);
            if !children.is_empty() {
            ret.push(TocElem::Inputref {
            id,
            uri,
            title,
            children,
            });
            }
            }*/
            Some((
                iter,
                Some(TocElem::Section {
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
                    ret.push(TocElem::Section {
                        id,
                        uri,
                        title,
                        children,
                    });
                }
            }
            Some((iter, Some(TocElem::SkippedSection { mut children }))) => {
                curr = iter;
                std::mem::swap(&mut ret, &mut children);
                if !children.is_empty() {
                    ret.push(TocElem::SkippedSection { children });
                }
            }
            Some((iter, _)) => curr = iter,
        }
        }
        (css, ret.into_boxed_slice())
    }
}
