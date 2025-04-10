use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use eyre::eyre;
use flams_ontology::narration::{
    documents::{Document, UncheckedDocument},
    problems::{Quiz, QuizElement, QuizQuestion},
    DocumentElement, NarrationTrait,
};
use flams_utils::{impossible, vecmap::VecSet};
use smallvec::SmallVec;

use super::Backend;

/*
#[derive(Debug)]
pub struct Offsets {
    pub refs_offset: u32,
    pub css_offset: u32,
    pub html_offset: u32,
    pub body_offset: u32,
    pub body_len: u32,
}
*/

pub struct PreDocFile;

impl PreDocFile {
    pub(crate) fn read_from_file(path: &Path) -> Option<UncheckedDocument> {
        macro_rules! err {
            ($e:expr) => {
                match $e {
                    Ok(e) => e,
                    Err(e) => {
                        tracing::error!("Error loading {}: {e}", path.display());
                        return None;
                    }
                }
            };
        }
        let file = err!(File::open(path));
        let file = BufReader::new(file);
        //UncheckedDocument::from_byte_stream(&mut file).ok()
        Some(err!(bincode::serde::decode_from_reader(
            file,
            bincode::config::standard()
        )))
        //let offsets = Self::read_initials(&mut file)?;
        //let doc = UncheckedDocument::from_byte_stream(&mut file).ok()?;
        //Some(doc)//Some(Self { path, doc, offsets })
    }
}

pub trait QuizExtension {
    /// #### Errors
    fn as_quiz(&self, backend: &impl Backend) -> eyre::Result<Quiz>;
}
impl QuizExtension for Document {
    #[allow(clippy::redundant_else)]
    #[allow(clippy::too_many_lines)]
    fn as_quiz(&self, backend: &impl Backend) -> eyre::Result<Quiz> {
        let mut css = VecSet::default();
        let mut elements = Vec::new();
        let mut solutions = HashMap::default();
        let mut answer_classes: HashMap<_, Vec<_>> = HashMap::default();
        let mut in_problem = false;

        let mut stack: SmallVec<_, 2> = SmallVec::new();
        let mut curr = self.children().iter();

        macro_rules! push {
            ($c:expr;$e:expr) => {
                stack.push((
                    std::mem::replace(&mut curr, $c),
                    std::mem::take(&mut elements),
                    $e,
                ))
            };
        }
        macro_rules! pop {
            () => {
                if let Some((c, mut e, s)) = stack.pop() {
                    curr = c;
                    std::mem::swap(&mut elements, &mut e);
                    match s {
                        Some(either::Either::Left(s)) => elements.push(QuizElement::Section {
                            title: s,
                            elements: e,
                        }),
                        Some(either::Either::Right(b)) => {
                            in_problem = b;
                            elements.extend(e.into_iter());
                        }
                        _ => elements.extend(e.into_iter()),
                    }
                    continue;
                } else {
                    break;
                }
            };
        }

        loop {
            let Some(e) = curr.next() else { pop!() };
            match e {
                DocumentElement::DocumentReference { target, .. } => {
                    let ret = if let Some(d) = target.get() {
                        d.as_quiz(backend)?
                    } else {
                        let uri = target.id();
                        let Some(d) = backend.get_document(&uri) else {
                            return Err(eyre!("Missing document {uri}"));
                        };
                        d.as_quiz(backend)?
                    };
                    for c in ret.css {
                        css.insert(c);
                    }
                    elements.extend(ret.elements);
                    for (u, s) in ret.solutions {
                        solutions.insert(u, s);
                    }
                }
                DocumentElement::Section(sect) => {
                    if let Some(title) = sect.title {
                        let Some((c, s)) = backend.get_html_fragment(self.uri(), title) else {
                            return Err(eyre!("Missing FTML fragment for {}", sect.uri));
                        };
                        for c in c {
                            css.insert(c);
                        }
                        push!(sect.children().iter();Some(either::Either::Left(s)));
                    } else {
                        push!(sect.children().iter();None);
                    }
                }
                DocumentElement::Paragraph(p) => {
                    let Some((c, html)) = backend.get_html_fragment(self.uri(), p.range) else {
                        return Err(eyre!("Missing FTML fragment for {}", p.uri));
                    };
                    for c in c {
                        css.insert(c);
                    }
                    elements.push(QuizElement::Paragraph { html });
                }
                DocumentElement::Problem(e) if in_problem => {
                    let Some(solution) = backend.get_reference(&e.solutions) else {
                        return Err(eyre!("Missing solutions for {}", e.uri));
                    };
                    let Some(solution) = solution.to_jstring() else {
                        return Err(eyre!("Invalid solutions for {}", e.uri));
                    };
                    solutions.insert(e.uri.clone(), solution);
                }
                DocumentElement::Problem(e) => {
                    let Some((c, html)) = backend.get_html_fragment(self.uri(), e.range) else {
                        return Err(eyre!("Missing FTML fragment for {}", e.uri));
                    };
                    for c in c {
                        css.insert(c);
                    }
                    let Some(solution) = backend.get_reference(&e.solutions) else {
                        return Err(eyre!("Missing solutions for {}", e.uri));
                    };
                    let title_html = if let Some(ttl) = e.title {
                        let Some(t) = backend.get_html_fragment(self.uri(), ttl) else {
                            return Err(eyre!("Missing FTML fragment for title of {}", e.uri));
                        };
                        Some(t.1)
                    } else {
                        None
                    };
                    let Some(solution) = solution.to_jstring() else {
                        return Err(eyre!("Invalid solutions for {}", e.uri));
                    };
                    for note in &e.gnotes {
                        let Some(gnote) = backend.get_reference(note) else {
                            return Err(eyre!("Missing gnote for {}", e.uri));
                        };
                        answer_classes
                            .entry(e.uri.clone())
                            .or_default()
                            .extend(gnote.answer_classes);
                    }
                    solutions.insert(e.uri.clone(), solution);
                    elements.push(QuizElement::Question(QuizQuestion {
                        html, //solution,
                        title_html,
                        uri: e.uri.clone(),
                        preconditions: e.preconditions.to_vec(),
                        objectives: e.objectives.to_vec(),
                        total_points: e.points,
                    }));
                    push!(e.children().iter();Some(either::Either::Right(in_problem)));
                    in_problem = true;
                }
                e => {
                    let c = e.children();
                    if !c.is_empty() {
                        push!(c.iter();None);
                    }
                }
            }
        }
        if elements.len() == 1 && matches!(elements.first(), Some(QuizElement::Section { .. })) {
            let Some(QuizElement::Section { elements: es, .. }) = elements.pop() else {
                impossible!()
            };
            elements = es;
        }
        Ok(Quiz {
            title: self.title().map(ToString::to_string),
            answer_classes,
            elements,
            css: css.0,
            solutions,
        })
    }
}
