use crate::extractor::{Attributes, FTMLExtractor};
use crate::open::OpenFTMLElement;
use crate::prelude::FTMLNode;
use ftml_core::FtmlKey;
use smallvec::SmallVec;

#[allow(type_alias_bounds)]
pub type Call<E: FTMLExtractor> = for<'a> fn(
    &mut E,
    &mut E::Attr<'a>,
    &mut SmallVec<FTMLExtractionRule<E>, 4>,
) -> Option<OpenFTMLElement>;

#[derive(PartialEq, Eq, Hash)]
pub struct FTMLExtractionRule<E: FTMLExtractor> {
    pub(crate) tag: FtmlKey,
    pub(crate) attr: &'static str,
    call: Call<E>,
}
impl<E: FTMLExtractor> Copy for FTMLExtractionRule<E> {}
impl<E: FTMLExtractor> Clone for FTMLExtractionRule<E> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
impl<E: FTMLExtractor> FTMLExtractionRule<E> {
    #[inline]
    pub(crate) const fn new(tag: FtmlKey, attr: &'static str, call: Call<E>) -> Self {
        Self { tag, attr, call }
    }
    #[inline]
    fn applies(&self, s: &str) -> bool {
        //tracing::trace!("{s} == {}? => {}",self.attr,s == self.attr);
        s == self.attr
    }
}

#[derive(Debug, Clone)]
pub struct FTMLElements {
    pub elems: SmallVec<OpenFTMLElement, 4>,
}
impl FTMLElements {
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.elems.is_empty()
    }
    #[inline]
    #[must_use]
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
    pub fn close<E: FTMLExtractor, N: FTMLNode>(&mut self, extractor: &mut E, node: &N) {
        let mut ret = Self {
            elems: SmallVec::default(),
        };
        while let Some(e) = self.elems.pop() {
            if let Some(r) = e.close(self, &mut ret, extractor, node) {
                ret.elems.push(r);
            }
        }
        *self = ret;
    }
    #[inline]
    #[must_use]
    pub fn take(self) -> SmallVec<OpenFTMLElement, 4> {
        self.elems
    }
}
impl<'a> IntoIterator for &'a FTMLElements {
    type Item = &'a OpenFTMLElement;
    type IntoIter = std::iter::Rev<std::slice::Iter<'a, OpenFTMLElement>>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.elems.iter().rev()
    }
}

pub trait RuleSet<E: FTMLExtractor> {
    type I<'i>: Iterator<Item = FTMLExtractionRule<E>>
    where
        Self: 'i,
        E: 'i;

    fn iter_rules(&self) -> Self::I<'_>;

    #[allow(clippy::cognitive_complexity)]
    fn applicable_rules<'a>(
        &self,
        extractor: &mut E,
        attrs: &'a mut E::Attr<'a>,
    ) -> Option<FTMLElements> {
        let mut stripped = attrs
            .keys()
            .filter(|s| {
                if s.starts_with(ftml_core::PREFIX) {
                    //tracing::trace!("attribute {s} ({:?})",std::thread::current().id());
                    true
                } else {
                    false
                }
            })
            .collect::<SmallVec<_, 4>>();
        if stripped.is_empty() {
            //tracing::trace!("no applicable attributes");
            return None;
        }
        //tracing::trace!("Found {:?} applicable attributes",stripped.len());
        let mut rules = SmallVec::<_, 4>::new();
        for rule in self.iter_rules() {
            if let Some((i, _)) = stripped.iter().enumerate().find(|(_, s)| rule.applies(s)) {
                //tracing::debug!("found {:?}",rule.tag);
                rules.push(rule);
                stripped.remove(i);
            }
        }
        for s in stripped {
            tracing::warn!(
                "Unknown ftml attribute: {s} = {}",
                attrs.value(s).expect("wut").as_ref()
            );
        }
        //tracing::trace!("Found {:?} applicable rules",rules.len());
        if rules.is_empty() {
            //tracing::trace!("returning elements");
            return None;
        }
        Self::do_rules(extractor, attrs, rules)
    }

    fn do_rules<'a>(
        extractor: &mut E,
        attrs: &'a mut E::Attr<'a>,
        mut rules: SmallVec<FTMLExtractionRule<E>, 4>,
    ) -> Option<FTMLElements> {
        rules.reverse();
        let mut ret = SmallVec::new();
        while let Some(rule) = rules.pop() {
            //tracing::trace!("calling rule {:?}",rule.tag);
            if let Some(r) = (rule.call)(extractor, attrs, &mut rules) {
                //println!("{{{r:?}");
                ret.push(r);
            }
        }
        //tracing::trace!("returning elements");
        if ret.is_empty() {
            None
        } else {
            Some(FTMLElements { elems: ret })
        }
    }
}
impl<const L: usize, E: FTMLExtractor> RuleSet<E> for [FTMLExtractionRule<E>; L] {
    type I<'i>
        = std::iter::Copied<std::slice::Iter<'i, FTMLExtractionRule<E>>>
    where
        E: 'i;
    fn iter_rules(&self) -> Self::I<'_> {
        self.iter().copied()
    }
}

#[allow(clippy::module_inception)]
#[allow(unused_macros)]
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub mod rules {
    use crate::errors::FTMLError;
    use crate::open::terms::{OpenArg, OpenTerm, OpenTermKind, PreVar, VarOrSym};
    use crate::open::OpenFTMLElement;
    use crate::prelude::{Attributes, FTMLExtractor};
    use crate::rules::FTMLExtractionRule;
    use flams_ontology::content::declarations::symbols::{ArgSpec, AssocType};
    use flams_ontology::ftml::FtmlKey;
    use flams_ontology::narration::documents::{DocumentStyle, SectionCounter};
    use flams_ontology::narration::paragraphs::{ParagraphFormatting, ParagraphKind};
    use flams_ontology::narration::problems::{AnswerKind, FillInSolOption};
    use flams_ontology::uris::{DocumentElementUri, DocumentUri, ModuleUri, SymbolUri, UriName};
    use flams_utils::vecmap::VecSet;
    use ftml_uris::IsNarrativeUri;
    use smallvec::SmallVec;
    use std::borrow::Cow;
    use std::str::FromStr;

    //type Value<'a,E:FTMLExtractor> = <E::Attr<'a> as Attributes>::Value<'a>;
    #[allow(type_alias_bounds)]
    pub type SV<E: FTMLExtractor> = SmallVec<FTMLExtractionRule<E>, 4>;

    static ERROR: std::sync::LazyLock<UriName> =
        std::sync::LazyLock::new(|| "ERROR".parse().expect("is a valid name"));
    static SKIP: std::sync::LazyLock<UriName> =
        std::sync::LazyLock::new(|| "skip".parse().expect("is a valid name"));

    macro_rules! err {
        ($extractor:ident,$f:expr) => {
            match $f {
                Ok(r) => r,
                Err(e) => {
                    $extractor.add_error(e);
                    return None;
                }
            }
        };
    }

    macro_rules! opt {
        ($extractor:ident,$f:expr) => {
            match $f {
                Ok(r) => Some(r),
                Err(FTMLError::InvalidKeyFor(_, Some(s))) if s.is_empty() => None,
                Err(e @ FTMLError::InvalidKeyFor(_, Some(_))) => {
                    $extractor.add_error(e);
                    None
                }
                _ => None,
            }
        };
    }

    //pub(crate) use rules_impl::*;

    //mod rules_impl {
    //    use flams_ontology::ftml::FtmlKey;
    //    use std::str::FromStr;
    //    use crate::{open::OpenFTMLElement, prelude::{Attributes, FTMLExtractor}};

    pub fn no_op<E: FTMLExtractor>(
        _extractor: &mut E,
        _attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        None
    }

    /*pub(crate) fn todo<E:FTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>,tag:FtmlKey) -> Option<OpenFTMLElement> {
        todo!("Tag {}",tag.as_str())
    }*/

    pub fn invisible<E: FTMLExtractor>(
        _extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        if attrs.take_bool(FtmlKey::Invisible) {
            Some(OpenFTMLElement::Invisible)
        } else {
            None
        }
    }

    pub fn setsectionlevel<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let lvl = err!(extractor, attrs.get_section_level(FtmlKey::SetSectionLevel));
        Some(OpenFTMLElement::SetSectionLevel(lvl))
    }

    pub fn style_rule<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let Some(style) = attrs.get(FtmlKey::Style) else {
            unreachable!()
        };
        let Ok(mut style) = DocumentStyle::from_str(style.as_ref()) else {
            extractor.add_error(FTMLError::InvalidURI(style.into()));
            return None;
        };
        if let Some(count) = attrs.get(FtmlKey::Counter) {
            nexts.retain(|e| e.tag != FtmlKey::Counter);
            if !count.as_ref().is_empty() {
                if let Ok(name) = count.as_ref().parse() {
                    style.counter = Some(name);
                } else {
                    extractor.add_error(FTMLError::InvalidURI(count.into()));
                    return None;
                }
            }
        }
        extractor.styles().styles.push(style);
        None
    }

    pub fn counter_parent<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let name = if let Some(count) = attrs.get(FtmlKey::Counter) {
            nexts.retain(|e| e.tag != FtmlKey::Counter);
            if let Ok(name) = count.as_ref().parse() {
                name
            } else {
                extractor.add_error(FTMLError::InvalidURI(count.into()));
                return None;
            }
        } else {
            extractor.add_error(FTMLError::MissingArguments);
            return None;
        };
        let parent = opt!(extractor, attrs.get_section_level(FtmlKey::CounterParent));
        extractor
            .styles()
            .counters
            .push(SectionCounter { name, parent });
        None
    }

    pub fn importmodule<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let uri = err!(
            extractor,
            attrs.take_module_uri(FtmlKey::ImportModule, extractor)
        );
        Some(OpenFTMLElement::ImportModule(uri))
    }

    pub fn usemodule<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let uri = err!(
            extractor,
            attrs.take_module_uri(FtmlKey::UseModule, extractor)
        );
        Some(OpenFTMLElement::UseModule(uri))
    }

    pub fn module<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let uri = err!(
            extractor,
            attrs.take_new_module_uri(FtmlKey::Module, extractor)
        );
        let _ = attrs.take_language(FtmlKey::Language);
        let meta = opt!(
            extractor,
            attrs.take_module_uri(FtmlKey::Metatheory, extractor)
        );
        let signature = opt!(extractor, attrs.take_language(FtmlKey::Signature));
        extractor.open_content(uri.clone());
        extractor.open_narrative(None);
        Some(OpenFTMLElement::Module {
            uri,
            meta,
            signature,
            //narrative: Vec::new(), content: Vec::new()
        })
    }

    pub fn mathstructure<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let uri = err!(
            extractor,
            attrs.take_new_symbol_uri(FtmlKey::MathStructure, extractor)
        );
        let macroname = attrs
            .remove(FtmlKey::Macroname)
            .map(|s| Into::<String>::into(s).into_boxed_str());
        extractor.open_content(uri.clone().into_module());
        extractor.open_narrative(None);
        Some(OpenFTMLElement::MathStructure {
            uri,
            macroname, //content: Vec::new(), narrative:Vec::new()
        })
    }

    pub fn morphism<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let uri = err!(
            extractor,
            attrs.take_new_symbol_uri(FtmlKey::Morphism, extractor)
        );
        let domain = err!(
            extractor,
            attrs.take_module_uri(FtmlKey::MorphismDomain, extractor)
        );
        let total = attrs.take_bool(FtmlKey::MorphismTotal);
        extractor.open_content(uri.clone().into_module());
        extractor.open_narrative(None);
        Some(OpenFTMLElement::Morphism {
            uri,
            domain,
            total, //content:Vec::new(),narrative:Vec::new()
        })
    }

    pub fn assign<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let symbol = err!(extractor, attrs.get_symbol_uri(FtmlKey::Assign, extractor));
        extractor.open_complex_term();
        Some(OpenFTMLElement::Assign(symbol))
    }

    pub fn section<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let lvl = err!(extractor, attrs.get_section_level(FtmlKey::Section));
        let id = attrs.get_id(extractor, Cow::Borrowed("section"));
        let Ok(uri) = id
            .parse()
            .map(|id: UriName| extractor.get_narrative_uri() & id)
        else {
            extractor.add_error(FTMLError::InvalidURI(format!("7: {id}")));
            return None;
        };
        extractor.open_section(uri.clone());
        Some(OpenFTMLElement::Section { lvl, uri })
    }

    pub fn slide<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let id = attrs.get_id(extractor, Cow::Borrowed("slide"));
        let Ok(uri) = id
            .parse()
            .map(|id: UriName| extractor.get_narrative_uri() & id)
        else {
            extractor.add_error(FTMLError::InvalidURI(format!("7: {id}")));
            return None;
        };
        extractor.open_slide();
        Some(OpenFTMLElement::Slide(uri))
    }

    pub fn slide_number<E: FTMLExtractor>(
        _extractor: &mut E,
        _attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        Some(OpenFTMLElement::SlideNumber)
    }

    pub fn skipsection<E: FTMLExtractor>(
        extractor: &mut E,
        _attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        extractor.open_section(DocumentUri::no_doc().clone() & SKIP.clone());
        Some(OpenFTMLElement::SkipSection)
    }

    pub fn definition<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        do_paragraph(extractor, attrs, nexts, ParagraphKind::Definition)
    }
    pub fn paragraph<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        do_paragraph(extractor, attrs, nexts, ParagraphKind::Paragraph)
    }
    pub fn assertion<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        do_paragraph(extractor, attrs, nexts, ParagraphKind::Assertion)
    }
    pub fn example<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        do_paragraph(extractor, attrs, nexts, ParagraphKind::Example)
    }
    pub fn proof<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        do_paragraph(extractor, attrs, nexts, ParagraphKind::Proof)
    }
    pub fn subproof<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        do_paragraph(extractor, attrs, nexts, ParagraphKind::SubProof)
    }

    fn do_paragraph<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
        kind: ParagraphKind,
    ) -> Option<OpenFTMLElement> {
        let id = attrs.get_id(extractor, Cow::Borrowed(kind.as_str()));
        let Ok(uri) = id
            .parse()
            .map(|id: UriName| extractor.get_narrative_uri().clone() & id)
        else {
            extractor.add_error(FTMLError::InvalidURI(format!("8: {id}")));
            return None;
        };
        let inline = attrs.get_bool(FtmlKey::Inline);
        let mut fors = VecSet::new();
        if let Some(f) = attrs.get(FtmlKey::Fors) {
            for f in f.as_ref().split(',') {
                if let Ok(f) = f.trim().parse() {
                    fors.insert(f);
                } else {
                    extractor.add_error(FTMLError::InvalidKeyFor(
                        FtmlKey::Fors.as_str(),
                        Some(f.trim().into()),
                    ));
                };
            }
        }
        let styles = opt!(
            extractor,
            attrs.get_typed_vec(FtmlKey::Styles, |s| s.trim().parse())
        )
        .unwrap_or_default();
        extractor.open_paragraph(uri.clone(), fors);
        let formatting = if inline {
            ParagraphFormatting::Inline
        } else if matches!(kind, ParagraphKind::Proof | ParagraphKind::SubProof) {
            let hide = attrs.get_bool(FtmlKey::ProofHide);
            if hide {
                ParagraphFormatting::Collapsed
            } else {
                ParagraphFormatting::Block
            }
        } else {
            ParagraphFormatting::Block
        };
        Some(OpenFTMLElement::Paragraph {
            kind,
            formatting,
            styles: styles.into_boxed_slice(),
            uri,
        })
    }

    pub fn proofbody<E: FTMLExtractor>(
        _extractor: &mut E,
        _attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        Some(OpenFTMLElement::ProofBody)
    }

    pub fn problem<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        do_problem(extractor, attrs, nexts, false)
    }

    pub fn subproblem<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        do_problem(extractor, attrs, nexts, true)
    }

    fn do_problem<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
        sub_problem: bool,
    ) -> Option<OpenFTMLElement> {
        let styles = opt!(
            extractor,
            attrs.get_typed_vec(FtmlKey::Styles, |s| s.trim().parse())
        )
        .unwrap_or_default();
        let id = attrs.get_id(extractor, Cow::Borrowed("problem"));
        let Ok(uri) = id
            .parse()
            .map(|id: UriName| extractor.get_narrative_uri().clone() & id)
        else {
            extractor.add_error(FTMLError::InvalidURI(format!("9: {id}")));
            return None;
        };
        let _ = attrs.take_language(FtmlKey::Language);
        let autogradable = attrs.get_bool(FtmlKey::Autogradable);
        let points = attrs.get(FtmlKey::ProblemPoints).and_then(|s| {
            s.as_ref()
                .parse()
                .ok()
                .or_else(|| Some(s.as_ref().parse::<i32>().ok()? as f32))
        });
        extractor.open_problem(uri.clone());
        Some(OpenFTMLElement::Problem {
            sub_problem,
            styles: styles.into_boxed_slice(),
            uri,
            autogradable,
            points,
        })
    }

    pub fn problem_hint<E: FTMLExtractor>(
        _extractor: &mut E,
        _attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        // TODO Check if in problem!
        Some(OpenFTMLElement::ProblemHint)
    }

    #[allow(clippy::borrowed_box)]
    pub fn solution<E: FTMLExtractor>(
        _extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        // TODO Check if in problem!
        let mut id = attrs.remove(FtmlKey::AnswerClass).map(Into::into);
        nexts.retain(|r| !matches!(r.tag, FtmlKey::AnswerClass));
        if id.as_ref().is_some_and(|s: &Box<str>| s.is_empty()) {
            id = None
        }
        Some(OpenFTMLElement::ProblemSolution(id))
    }

    pub fn gnote<E: FTMLExtractor>(
        extractor: &mut E,
        _attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        extractor.open_gnote();
        Some(OpenFTMLElement::ProblemGradingNote)
    }

    pub fn answer_class<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let id = attrs.get_id(extractor, Cow::Borrowed("AC"));
        let kind = opt!(
            extractor,
            attrs.get_typed(FtmlKey::AnswerClassPts, str::parse)
        )
        .unwrap_or(AnswerKind::Trait(0.0));
        extractor.push_answer_class(id, kind);
        Some(OpenFTMLElement::AnswerClass)
    }

    pub fn ac_feedback<E: FTMLExtractor>(
        _extractor: &mut E,
        _attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        Some(OpenFTMLElement::AnswerClassFeedback)
    }

    pub fn multiple_choice_block<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let styles = opt!(
            extractor,
            attrs.get_typed(FtmlKey::Styles, |s| Result::<_, ()>::Ok(
                s.split(',')
                    .map(|s| s.trim().to_string().into_boxed_str())
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            ))
        )
        .unwrap_or_default();
        let inline = styles.iter().any(|s| &**s == "inline");
        extractor.open_choice_block(true, styles);
        Some(OpenFTMLElement::ChoiceBlock {
            multiple: true,
            inline,
        })
    }

    pub fn single_choice_block<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let styles = opt!(
            extractor,
            attrs.get_typed(FtmlKey::Styles, |s| Result::<_, ()>::Ok(
                s.split(',')
                    .map(|s| s.trim().to_string().into_boxed_str())
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            ))
        )
        .unwrap_or_default();
        let inline = styles.iter().any(|s| &**s == "inline");
        extractor.open_choice_block(false, styles);
        Some(OpenFTMLElement::ChoiceBlock {
            multiple: false,
            inline,
        })
    }

    pub fn problem_choice<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let correct = attrs.get_bool(FtmlKey::ProblemChoice); //attrs.take_bool(FtmlKey::ProblemChoice);
        attrs.set(FtmlKey::ProblemChoice.attr_name(), "");
        extractor.push_problem_choice(correct);
        Some(OpenFTMLElement::ProblemChoice)
    }

    pub fn problem_choice_verdict<E: FTMLExtractor>(
        _extractor: &mut E,
        _attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        Some(OpenFTMLElement::ProblemChoiceVerdict)
    }

    pub fn problem_choice_feedback<E: FTMLExtractor>(
        _extractor: &mut E,
        _attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        Some(OpenFTMLElement::ProblemChoiceFeedback)
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn fillinsol<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let val = attrs
            .get_typed(FtmlKey::ProblemFillinsolWidth, |s| {
                if s.contains('.') {
                    s.parse::<f32>().map_err(|_| ())
                } else {
                    s.parse::<i32>().map(|i| i as f32).map_err(|_| ())
                }
            })
            .ok();
        extractor.open_fillinsol(val);
        Some(OpenFTMLElement::Fillinsol(val))
    }

    pub fn fillinsol_case<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let Some(val) = attrs.remove(FtmlKey::ProblemFillinsolCase) else {
            unreachable!()
        };
        let verdict = attrs.take_bool(FtmlKey::ProblemFillinsolCaseVerdict);
        let Some(value) = attrs.remove(FtmlKey::ProblemFillinsolCaseValue) else {
            extractor.add_error(FTMLError::IncompleteArgs(5));
            return None;
        };
        let Some(opt) = FillInSolOption::from_values(&val, &value, verdict) else {
            extractor.add_error(FTMLError::IncompleteArgs(6));
            return None;
        };
        extractor.push_fillinsol_case(opt);
        Some(OpenFTMLElement::FillinsolCase)
    }

    pub fn doctitle<E: FTMLExtractor>(
        _extractor: &mut E,
        _attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        Some(OpenFTMLElement::Doctitle)
    }

    pub fn title<E: FTMLExtractor>(
        _extractor: &mut E,
        _attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        Some(OpenFTMLElement::Title)
    }

    pub fn prooftitle<E: FTMLExtractor>(
        _extractor: &mut E,
        _attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        Some(OpenFTMLElement::ProofTitle)
    }

    pub fn subprooftitle<E: FTMLExtractor>(
        _extractor: &mut E,
        _attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        Some(OpenFTMLElement::SubproofTitle)
    }

    pub fn precondition<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let uri = err!(
            extractor,
            attrs.get_symbol_uri(FtmlKey::PreconditionSymbol, extractor)
        );
        let dim = err!(
            extractor,
            attrs.get_typed(FtmlKey::PreconditionDimension, str::parse)
        );
        extractor.add_precondition(uri, dim);
        None
    }

    pub fn objective<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let uri = err!(
            extractor,
            attrs.get_symbol_uri(FtmlKey::ObjectiveSymbol, extractor)
        );
        let dim = err!(
            extractor,
            attrs.get_typed(FtmlKey::ObjectiveDimension, str::parse)
        );
        extractor.add_objective(uri, dim);
        None
    }

    pub fn symdecl<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let uri = err!(
            extractor,
            attrs.get_new_symbol_uri(FtmlKey::Symdecl, extractor)
        );
        let role = opt!(
            extractor,
            attrs.get_typed(FtmlKey::Role, |s| Result::<_, ()>::Ok(
                s.split(',')
                    .map(|s| s.trim().to_string().into_boxed_str())
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            ))
        )
        .unwrap_or_default();
        let assoctype = opt!(
            extractor,
            attrs.get_typed(FtmlKey::AssocType, AssocType::from_str)
        );
        let arity =
            opt!(extractor, attrs.get_typed(FtmlKey::Args, ArgSpec::from_str)).unwrap_or_default();
        let reordering = attrs
            .get(FtmlKey::ArgumentReordering)
            .map(|s| Into::<String>::into(s).into_boxed_str());
        let macroname = attrs
            .get(FtmlKey::Macroname)
            .map(|s| Into::<String>::into(s).into_boxed_str());
        extractor.open_decl();
        Some(OpenFTMLElement::Symdecl {
            uri,
            arity,
            macroname,
            role,
            assoctype,
            reordering,
        })
    }

    pub fn vardecl<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        do_vardecl(extractor, attrs, nexts, FtmlKey::Vardef, false)
    }
    pub fn varseq<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        do_vardecl(extractor, attrs, nexts, FtmlKey::Varseq, true)
    }

    pub fn do_vardecl<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
        tag: FtmlKey,
        is_seq: bool,
    ) -> Option<OpenFTMLElement> {
        let Some(name) = attrs
            .get(tag)
            .and_then(|v| UriName::from_str(v.as_ref()).ok())
        else {
            extractor.add_error(FTMLError::InvalidKeyFor(tag.as_str(), None));
            return None;
        };
        let role = opt!(
            extractor,
            attrs.get_typed(FtmlKey::Role, |s| Result::<_, ()>::Ok(
                s.split(',')
                    .map(|s| s.trim().to_string().into_boxed_str())
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            ))
        )
        .unwrap_or_default();
        let assoctype = opt!(
            extractor,
            attrs.get_typed(FtmlKey::AssocType, AssocType::from_str)
        );
        let arity =
            opt!(extractor, attrs.get_typed(FtmlKey::Args, ArgSpec::from_str)).unwrap_or_default();
        let reordering = attrs
            .get(FtmlKey::ArgumentReordering)
            .map(|s| Into::<String>::into(s).into_boxed_str());
        let macroname = attrs
            .get(FtmlKey::Macroname)
            .map(|s| Into::<String>::into(s).into_boxed_str());
        let bind = attrs.get_bool(FtmlKey::Bind);
        extractor.open_decl();
        let uri = extractor.get_narrative_uri() & name;
        Some(OpenFTMLElement::Vardecl {
            uri,
            arity,
            macroname,
            role,
            assoctype,
            reordering,
            bind,
            is_seq,
        })
    }

    pub fn notation<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let symbol = if let Ok(s) = attrs.get_symbol_uri(FtmlKey::Notation, extractor) {
            VarOrSym::S(s.into())
        } else if let Some(v) = attrs.get(FtmlKey::Notation) {
            let Ok(n) = v.as_ref().parse() else {
                extractor.add_error(FTMLError::InvalidURI(format!("10: {}", v.as_ref())));
                return None;
            };
            VarOrSym::V(PreVar::Unresolved(n))
        } else {
            extractor.add_error(FTMLError::InvalidKeyFor(FtmlKey::Notation.as_str(), None));
            return None;
        };
        let mut fragment = attrs
            .get(FtmlKey::NotationFragment)
            .map(|s| Into::<String>::into(s).into_boxed_str());
        if fragment.as_ref().is_some_and(|s| s.is_empty()) {
            fragment = None
        };
        let id = fragment.as_ref().map_or("notation", |s| &**s).to_string();
        let id = extractor.new_id(Cow::Owned(id));
        let prec = if let Some(v) = attrs.get(FtmlKey::Precedence) {
            if let Ok(v) = isize::from_str(v.as_ref()) {
                v
            } else {
                extractor.add_error(FTMLError::InvalidKeyFor(FtmlKey::Precedence.as_str(), None));
                return None;
            }
        } else {
            0
        };
        let mut argprecs = SmallVec::default();
        if let Some(s) = attrs.get(FtmlKey::Argprecs) {
            for s in s.as_ref().split(',') {
                if s.is_empty() {
                    continue;
                }
                if let Ok(v) = isize::from_str(s.trim()) {
                    argprecs.push(v)
                } else {
                    extractor.add_error(FTMLError::InvalidKeyFor(FtmlKey::Argprecs.as_str(), None));
                    return None;
                }
            }
        }
        extractor.open_notation();
        Some(OpenFTMLElement::Notation {
            id,
            symbol,
            precedence: prec,
            argprecs,
        })
    }

    pub fn notationcomp<E: FTMLExtractor>(
        _extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        attrs.remove(FtmlKey::NotationComp);
        attrs.remove(FtmlKey::Term);
        attrs.remove(FtmlKey::Head);
        attrs.remove(FtmlKey::NotationId);
        attrs.remove(FtmlKey::Invisible);
        nexts.retain(|r| {
            !matches!(
                r.tag,
                FtmlKey::Term | FtmlKey::Head | FtmlKey::NotationId | FtmlKey::Invisible
            )
        });
        Some(OpenFTMLElement::NotationComp)
    }
    pub fn notationopcomp<E: FTMLExtractor>(
        _extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        attrs.remove(FtmlKey::NotationComp);
        attrs.remove(FtmlKey::Term);
        attrs.remove(FtmlKey::Head);
        attrs.remove(FtmlKey::NotationId);
        attrs.remove(FtmlKey::Invisible);
        nexts.retain(|r| {
            !matches!(
                r.tag,
                FtmlKey::Term | FtmlKey::Head | FtmlKey::NotationId | FtmlKey::Invisible
            )
        });
        Some(OpenFTMLElement::NotationOpComp)
    }

    pub fn definiendum<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let uri = err!(
            extractor,
            attrs.get_symbol_uri(FtmlKey::Definiendum, extractor)
        );
        extractor.add_definiendum(uri.clone());
        Some(OpenFTMLElement::Definiendum(uri))
    }

    pub fn r#type<E: FTMLExtractor>(
        extractor: &mut E,
        _attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        if extractor.in_term() {
            extractor.add_error(FTMLError::InvalidKey);
            return None;
        }
        extractor.set_in_term(true);
        Some(OpenFTMLElement::Type)
    }

    pub fn conclusion<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let uri = err!(
            extractor,
            attrs.get_symbol_uri(FtmlKey::Conclusion, extractor)
        );
        let in_term = extractor.in_term();
        extractor.set_in_term(true);
        Some(OpenFTMLElement::Conclusion { uri, in_term })
    }

    pub fn definiens<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let uri = opt!(
            extractor,
            attrs.get_symbol_uri(FtmlKey::Definiens, extractor)
        );
        let in_term = extractor.in_term();
        extractor.set_in_term(true);
        Some(OpenFTMLElement::Definiens { uri, in_term })
    }

    pub fn mmtrule<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let id = attrs
            .get(FtmlKey::Rule)
            .unwrap_or_else(|| unreachable!())
            .as_ref()
            .to_string()
            .into_boxed_str();
        extractor.open_args();
        Some(OpenFTMLElement::MMTRule(id))
    }

    pub fn argsep<E: FTMLExtractor>(
        _extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        attrs.remove(FtmlKey::Term);
        attrs.remove(FtmlKey::ArgSep);
        attrs.remove(FtmlKey::Head);
        attrs.remove(FtmlKey::NotationId);
        attrs.remove(FtmlKey::Invisible);
        nexts.retain(|r| {
            !matches!(
                r.tag,
                FtmlKey::Term
                    | FtmlKey::Head
                    | FtmlKey::NotationId
                    | FtmlKey::Invisible
                    | FtmlKey::ArgSep
            )
        });
        Some(OpenFTMLElement::ArgSep)
    }

    pub fn argmap<E: FTMLExtractor>(
        _extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        attrs.remove(FtmlKey::Term);
        attrs.remove(FtmlKey::Head);
        attrs.remove(FtmlKey::ArgMap);
        attrs.remove(FtmlKey::NotationId);
        attrs.remove(FtmlKey::Invisible);
        nexts.retain(|r| {
            !matches!(
                r.tag,
                FtmlKey::Term
                    | FtmlKey::Head
                    | FtmlKey::NotationId
                    | FtmlKey::Invisible
                    | FtmlKey::ArgMap
            )
        });
        Some(OpenFTMLElement::ArgMap)
    }

    pub fn argmapsep<E: FTMLExtractor>(
        _extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        attrs.remove(FtmlKey::Term);
        attrs.remove(FtmlKey::Head);
        attrs.remove(FtmlKey::ArgMapSep);
        attrs.remove(FtmlKey::NotationId);
        attrs.remove(FtmlKey::Invisible);
        nexts.retain(|r| {
            !matches!(
                r.tag,
                FtmlKey::Term
                    | FtmlKey::Head
                    | FtmlKey::NotationId
                    | FtmlKey::Invisible
                    | FtmlKey::ArgMapSep
            )
        });
        Some(OpenFTMLElement::ArgMapSep)
    }

    pub fn term<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        if extractor.in_notation() {
            return None;
        }
        let notation = attrs.value(FtmlKey::NotationId.attr_name()).and_then(|n| {
            let asr = n.as_ref().trim();
            if asr.is_empty() {
                return None;
            }
            Some(asr.parse::<UriName>().unwrap_or_else(|_| {
                extractor.add_error(FTMLError::InvalidURI(format!("12: {}", n.as_ref())));
                ERROR.clone()
            }))
        });
        let head = match attrs.value(FtmlKey::Head.attr_name()) {
            None => {
                extractor.add_error(FTMLError::MissingHeadForTerm);
                VarOrSym::V(PreVar::Unresolved(ERROR.clone()))
            }
            Some(v) => {
                let v = v.as_ref();
                v.parse::<SymbolUri>().ok().map_or_else(
                    || {
                        v.parse::<ModuleUri>().map_or_else(
                            |_| {
                                DocumentElementUri::from_str(v).map_or_else(
                                    |_| {
                                        if v.contains('?') {
                                            tracing::warn!(
                                                "Suspicious variable name containing '?': {v}"
                                            );
                                        }
                                        v.parse().ok().map_or_else(
                                            || {
                                                extractor.add_error(FTMLError::InvalidURI(
                                                    format!("13: {v}"),
                                                ));
                                                None
                                            },
                                            |v| Some(VarOrSym::V(PreVar::Unresolved(v))),
                                        )
                                    },
                                    |d| Some(VarOrSym::V(PreVar::Resolved(d))),
                                )
                            },
                            |m| Some(VarOrSym::S(m.into())),
                        )
                    },
                    |s| Some(VarOrSym::S(s.into())),
                )?
            }
        };
        //attrs.set(tagstrings::HEAD,&head.to_string());
        let kind = attrs
            .value(FtmlKey::Term.attr_name())
            .unwrap_or_else(|| unreachable!());
        let kind: OpenTermKind = kind.as_ref().parse().unwrap_or_else(|()| {
            extractor.add_error(FTMLError::InvalidTermKind(kind.into()));
            OpenTermKind::OMA
        });
        let term = match (kind, head) {
            (OpenTermKind::OMID | OpenTermKind::OMV, VarOrSym::S(uri)) => {
                OpenTerm::Symref { uri, notation }
            }
            (OpenTermKind::OMID | OpenTermKind::OMV, VarOrSym::V(name)) => {
                OpenTerm::Varref { name, notation }
            }
            (OpenTermKind::OML, VarOrSym::V(PreVar::Unresolved(name))) => {
                extractor.open_decl();
                OpenTerm::OML { name } //, tp: None, df: None }
            }
            (OpenTermKind::OMA, head) => {
                extractor.open_args();
                OpenTerm::OMA { head, notation } //, args: SmallVec::new() }
            }
            (OpenTermKind::Complex, head) => {
                extractor.open_complex_term();
                OpenTerm::Complex(head)
            }
            (k, head) => {
                extractor.add_error(FTMLError::InvalidHeadForTermKind(k, head.clone()));
                extractor.open_args();
                OpenTerm::OMA { head, notation } //, args: SmallVec::new() }
            }
        };
        let is_top = if extractor.in_term() {
            false
        } else {
            extractor.set_in_term(true);
            true
        };
        Some(OpenFTMLElement::OpenTerm { term, is_top })
    }

    pub fn arg<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let Some(value) = attrs.value(FtmlKey::Arg.attr_name()) else {
            extractor.add_error(FTMLError::InvalidArgSpec);
            return None;
        };
        let arg = OpenArg::from_strs(value, attrs.value(FtmlKey::ArgMode.attr_name()));
        let Some(arg) = arg else {
            extractor.add_error(FTMLError::InvalidArgSpec);
            return None;
        };
        Some(OpenFTMLElement::Arg(arg))
    }

    pub fn headterm<E: FTMLExtractor>(
        _extractor: &mut E,
        _attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        Some(OpenFTMLElement::HeadTerm)
    }

    pub fn inputref<E: FTMLExtractor>(
        extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let uri = err!(
            extractor,
            attrs.get_document_uri(FtmlKey::InputRef, extractor)
        );
        let id = attrs.get_id(extractor, Cow::Owned(uri.document_name().to_string()));
        Some(OpenFTMLElement::Inputref { uri, id })
    }

    pub fn ifinputref<E: FTMLExtractor>(
        _extractor: &mut E,
        attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        let value = attrs.get_bool(FtmlKey::IfInputref);
        Some(OpenFTMLElement::IfInputref(value))
    }

    pub fn comp<E: FTMLExtractor>(
        _extractor: &mut E,
        _attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        Some(OpenFTMLElement::Comp)
    }

    pub fn maincomp<E: FTMLExtractor>(
        _extractor: &mut E,
        _attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        Some(OpenFTMLElement::MainComp)
    }

    pub fn defcomp<E: FTMLExtractor>(
        _extractor: &mut E,
        _attrs: &mut E::Attr<'_>,
        _nexts: &mut SV<E>,
    ) -> Option<OpenFTMLElement> {
        Some(OpenFTMLElement::DefComp)
    }

    //}
}
