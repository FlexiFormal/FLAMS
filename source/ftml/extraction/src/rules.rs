use flams_ontology::ftml::FTMLKey;
use smallvec::SmallVec;
use crate::extractor::{Attributes, FTMLExtractor};
use crate::open::OpenFTMLElement;
use crate::prelude::FTMLNode;

pub use super::tags::{rule,all_rules};

#[allow(type_alias_bounds)]
pub type Call<E:FTMLExtractor> = for <'a> fn(&mut E,&mut E::Attr<'a>,&mut SmallVec<FTMLExtractionRule<E>,4>) -> Option<OpenFTMLElement>;

#[derive(PartialEq, Eq,Hash)]
pub struct FTMLExtractionRule<E:FTMLExtractor>{
    pub(crate) tag:FTMLKey, pub(crate) attr:&'static str,
    call:Call<E>
}
impl<E:FTMLExtractor> Copy for FTMLExtractionRule<E> {}
impl<E:FTMLExtractor> Clone for FTMLExtractionRule<E> {
    #[inline]
    fn clone(&self) -> Self { *self }
}
impl<E:FTMLExtractor> FTMLExtractionRule<E> {
    #[inline]
    pub(crate) const fn new(tag:FTMLKey,attr:&'static str,call:Call<E>) -> Self {
        Self { tag,attr,call }
    }
    #[inline]
    fn applies(&self, s:&str) -> bool { 
        //tracing::trace!("{s} == {}? => {}",self.attr,s == self.attr);
        s == self.attr 
    }
}

#[derive(Debug,Clone)]
pub struct FTMLElements {
    pub elems:SmallVec<OpenFTMLElement,4>
}
impl FTMLElements {
    #[inline]#[must_use]
    pub fn is_empty(&self) -> bool {
        self.elems.is_empty()
    }
    #[inline]#[must_use]
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
    pub fn close<E:FTMLExtractor,N:FTMLNode>(&mut self,extractor:&mut E,node:&N) {
        let mut ret = Self{elems:SmallVec::default()};
        while let Some(e) = self.elems.pop() {
            if let Some(r) = e.close(self,&mut ret,extractor,node) {
                ret.elems.push(r);
            }
        }
        *self = ret;
    }
    #[inline]#[must_use]
    pub fn take(self) -> SmallVec<OpenFTMLElement,4> {
        self.elems
    }
}
impl<'a> IntoIterator for &'a FTMLElements {
    type Item = &'a OpenFTMLElement;
    type IntoIter = std::iter::Rev<std::slice::Iter<'a,OpenFTMLElement>>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.elems.iter().rev()
    }
}


pub trait RuleSet<E:FTMLExtractor> {
    type I<'i>:Iterator<Item=FTMLExtractionRule<E>> where Self:'i,E:'i;

    fn iter_rules(&self) -> Self::I<'_>;

    #[allow(clippy::cognitive_complexity)]
    fn applicable_rules<'a>(&self,extractor:&mut E,attrs:&'a mut E::Attr<'a>) -> Option<FTMLElements> {
        let mut stripped = attrs.keys().filter(|s| {
            if s.starts_with(flams_ontology::ftml::PREFIX) {
                //tracing::trace!("attribute {s} ({:?})",std::thread::current().id());
                true
            } else { false }
        }).collect::<SmallVec<_,4>>();
        if stripped.is_empty() {
            //tracing::trace!("no applicable attributes");
            return None
        }
        //tracing::trace!("Found {:?} applicable attributes",stripped.len());
        let mut rules = SmallVec::<_,4>::new();
        for rule in self.iter_rules() {
            if let Some((i,_)) = stripped.iter().enumerate().find(|(_,s)| rule.applies(s)) {
                //tracing::debug!("found {:?}",rule.tag);
                rules.push(rule);
                stripped.remove(i);
            }
        }
        for s in stripped {
            tracing::warn!("Unknown ftml attribute: {s} = {}",attrs.value(s).expect("wut").as_ref());
        }
        //tracing::trace!("Found {:?} applicable rules",rules.len());
        if rules.is_empty() {
            //tracing::trace!("returning elements");
            return None
        }
        Self::do_rules(extractor, attrs, rules)
    }

    fn do_rules<'a>(extractor:&mut E,attrs:&'a mut E::Attr<'a>,mut rules:SmallVec<FTMLExtractionRule<E>,4>) -> Option<FTMLElements> {
        rules.reverse();
        let mut ret = SmallVec::new();
        while let Some(rule) = rules.pop() {
            //tracing::trace!("calling rule {:?}",rule.tag);
            if let Some(r) = (rule.call)(extractor,attrs,&mut rules) {
                //println!("{{{r:?}");
                ret.push(r);
            }
        }
        //tracing::trace!("returning elements");
        if ret.is_empty() {None} else {Some(
            FTMLElements { elems: ret }
        )}
    }

}
impl<const L:usize,E:FTMLExtractor> RuleSet<E> for [FTMLExtractionRule<E>;L] {
    type I<'i> = std::iter::Copied<std::slice::Iter<'i, FTMLExtractionRule<E>>> where E:'i;
    fn iter_rules(&self) -> Self::I<'_> { self.iter().copied() }
}

#[allow(clippy::module_inception)]
#[allow(unused_macros)]
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub mod rules {
    use flams_ontology::content::declarations::symbols::{ArgSpec, AssocType};
    use flams_ontology::narration::exercises::{AnswerClass, AnswerKind, Choice, FillInSolOption, SolutionData};
    use flams_ontology::narration::paragraphs::ParagraphKind;
    use flams_ontology::ftml::FTMLKey;
    use flams_ontology::uris::{DocumentElementURI, ModuleURI, Name, SymbolURI};
    use flams_utils::vecmap::VecSet;
    use smallvec::SmallVec;
    use crate::errors::FTMLError;
    use crate::open::OpenFTMLElement;
    use crate::prelude::{Attributes, FTMLExtractor};
    use crate::rules::FTMLExtractionRule;
    use crate::open::terms::{OpenArg, OpenTerm, OpenTermKind, PreVar, VarOrSym};
    use std::borrow::Cow;
    use std::str::FromStr;

    //type Value<'a,E:FTMLExtractor> = <E::Attr<'a> as Attributes>::Value<'a>;
    #[allow(type_alias_bounds)]
    pub type SV<E:FTMLExtractor> = SmallVec<FTMLExtractionRule<E>,4>;

    lazy_static::lazy_static! {
        static ref ERROR : Name = "ERROR".parse().unwrap_or_else(|_| unreachable!());
    }

    macro_rules! err {
        ($extractor:ident,$f:expr) => {
            match $f {
                Ok(r) => r,
                Err(e) => {
                    $extractor.add_error(e);
                    return None
                }
            }
        }
    }

    macro_rules! opt {
        ($extractor:ident,$f:expr) => {
            match $f {
                Ok(r) => Some(r),
                Err(FTMLError::InvalidKeyFor(_, Some(s))) if s.is_empty() => None,
                Err(e@FTMLError::InvalidKeyFor(_, Some(_))) => {
                    $extractor.add_error(e);
                    None
                }
                _ => None
            }
        }
    }

    //pub(crate) use rules_impl::*;

    //mod rules_impl {
    //    use flams_ontology::ftml::FTMLKey;
    //    use std::str::FromStr;
    //    use crate::{open::OpenFTMLElement, prelude::{Attributes, FTMLExtractor}};

        pub fn no_op<E:FTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> { None }

        /*pub(crate) fn todo<E:FTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>,tag:FTMLKey) -> Option<OpenFTMLElement> {
            todo!("Tag {}",tag.as_str()) 
        }*/

        pub fn invisible<E:FTMLExtractor>(_extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            if attrs.take_bool(FTMLKey::Invisible) {
                Some(OpenFTMLElement::Invisible)
            } else { None }
        }

        pub fn setsectionlevel<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let lvl = err!(extractor,attrs.get_section_level(FTMLKey::SetSectionLevel));
            Some(OpenFTMLElement::SetSectionLevel(lvl))
        }

        pub fn importmodule<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let uri = err!(extractor,attrs.take_module_uri(FTMLKey::ImportModule, extractor));
            Some(OpenFTMLElement::ImportModule(uri))
        }

        pub fn usemodule<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let uri = err!(extractor,attrs.take_module_uri(FTMLKey::UseModule, extractor));
            Some(OpenFTMLElement::UseModule(uri))
        }

        pub fn module<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let uri = err!(extractor,attrs.take_new_module_uri(FTMLKey::Module, extractor));
            let _ = attrs.take_language(FTMLKey::Language);
            let meta = opt!(extractor,attrs.take_module_uri(FTMLKey::Metatheory, extractor));
            let signature = opt!(extractor,attrs.take_language(FTMLKey::Signature));
            extractor.open_content(uri.clone());
            extractor.open_narrative(None);
            Some(OpenFTMLElement::Module { 
                uri, meta, signature, 
                //narrative: Vec::new(), content: Vec::new() 
            })
        }

        pub fn mathstructure<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let uri = err!(extractor,attrs.take_new_symbol_uri(FTMLKey::MathStructure, extractor));
            let macroname = attrs.remove(FTMLKey::Macroname).map(|s| Into::<String>::into(s).into_boxed_str());
            extractor.open_content(uri.clone().into_module());
            extractor.open_narrative(None);
            Some(OpenFTMLElement::MathStructure { 
                uri,macroname, //content: Vec::new(), narrative:Vec::new()
            })
        }

        pub fn morphism<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let uri = err!(extractor,attrs.take_new_symbol_uri(FTMLKey::Morphism,extractor));
            let domain = err!(extractor,attrs.take_module_uri(FTMLKey::MorphismDomain, extractor));
            let total = attrs.take_bool(FTMLKey::MorphismTotal);
            extractor.open_content(uri.clone().into_module());
            extractor.open_narrative(None);
            Some(OpenFTMLElement::Morphism {
                uri,domain,total,//content:Vec::new(),narrative:Vec::new()
            })
        }

        pub fn assign<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let symbol = err!(extractor,attrs.get_symbol_uri(FTMLKey::Assign,extractor));
            extractor.open_complex_term();
            Some(OpenFTMLElement::Assign(symbol))
        }

        pub fn section<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let lvl = err!(extractor,attrs.get_section_level(FTMLKey::Section));
            let id = attrs.get_id(extractor,Cow::Borrowed("section"));
            let uri = match extractor.get_narrative_uri() & &*id {
                Ok(uri) => uri,
                Err(e) => {
                    extractor.add_error(FTMLError::InvalidURI(format!("7: {id}")));
                    return None
                }
            };
            extractor.open_section(uri.clone());
            Some(OpenFTMLElement::Section { lvl, uri })
        }

        pub fn definition<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            do_paragraph(extractor, attrs, nexts, ParagraphKind::Definition)
        }
        pub fn paragraph<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            do_paragraph(extractor, attrs, nexts, ParagraphKind::Paragraph)
        }
        pub fn assertion<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            do_paragraph(extractor, attrs, nexts, ParagraphKind::Assertion)
        }
        pub fn example<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            do_paragraph(extractor, attrs, nexts, ParagraphKind::Example)
        }
        pub fn proof<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            do_paragraph(extractor, attrs, nexts, ParagraphKind::Proof)
        }
        pub fn subproof<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            do_paragraph(extractor, attrs, nexts, ParagraphKind::SubProof)
        }

        fn do_paragraph<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>,kind:ParagraphKind) -> Option<OpenFTMLElement> {
            let id = attrs.get_id(extractor,Cow::Borrowed(kind.as_str()));
            let uri = match extractor.get_narrative_uri() & &*id {
                Ok(uri) => uri,
                Err(e) => {
                    extractor.add_error(FTMLError::InvalidURI(format!("8: {id}")));
                    return None
                }
            };
            let inline = attrs.get_bool(FTMLKey::Inline);
            let mut fors = VecSet::new();
            if let Some(f) = attrs.get(FTMLKey::Fors) {
                for f in f.as_ref().split(',') {
                    if let Ok(f) = f.trim().parse() {
                        fors.insert(f);
                    } else {
                        extractor.add_error(FTMLError::InvalidKeyFor(FTMLKey::Fors.as_str(), Some(f.trim().into())));
                    };
                }
            }
            let styles = opt!(extractor,attrs.get_typed(FTMLKey::Styles, 
                |s| Result::<_,()>::Ok(s.split(',').map(|s| s.trim().to_string().into_boxed_str()).collect::<Vec<_>>().into_boxed_slice())
            )).unwrap_or_default();
            extractor.open_paragraph(uri.clone(), fors);
            Some(OpenFTMLElement::Paragraph { kind, inline, styles,uri })
        }

        pub fn exercise<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            do_exercise(extractor,attrs,nexts,false)
        }

        pub fn subexercise<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            do_exercise(extractor,attrs,nexts,true)
        }

        fn do_exercise<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>,sub_exercise:bool) -> Option<OpenFTMLElement> {
            let styles = opt!(extractor,attrs.get_typed(FTMLKey::Styles, 
                |s| Result::<_,()>::Ok(s.split(',').map(|s| s.trim().to_string().into_boxed_str()).collect::<Vec<_>>().into_boxed_slice())
            )).unwrap_or_default();
            let id = attrs.get_id(extractor,Cow::Borrowed("exercise"));
            let uri = match extractor.get_narrative_uri() & &*id {
                Ok(uri) => uri,
                Err(e) => {
                    extractor.add_error(FTMLError::InvalidURI(format!("9: {id}")));
                    return None
                }
            };
            let _ = attrs.take_language(FTMLKey::Language);
            let autogradable = attrs.get_bool(FTMLKey::Autogradable);
            let points = attrs.get(FTMLKey::ProblemPoints)
                .and_then(|s| s.as_ref().parse().ok());
            extractor.open_exercise(uri.clone());
            Some(OpenFTMLElement::Exercise { sub_exercise, styles, uri, autogradable, points })
        }

        pub fn problem_hint<E:FTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            // TODO Check if in problem!
            Some(OpenFTMLElement::ProblemHint)
        }

        pub fn solution<E:FTMLExtractor>(_extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            // TODO Check if in problem!
            let mut id = attrs.remove(FTMLKey::AnswerClass).map(Into::into);
            nexts.retain(|r| !matches!(r.tag,FTMLKey::AnswerClass));
            if id.as_ref().is_some_and(|s:&Box<str>| s.is_empty()) { id = None }
            Some(OpenFTMLElement::ExerciseSolution(id))
        }

        pub fn gnote<E:FTMLExtractor>(extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            extractor.open_gnote();
            Some(OpenFTMLElement::ExerciseGradingNote)
        }

        pub fn answer_class<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let id = attrs.get_id(extractor,Cow::Borrowed("AC"));
            let kind = opt!(extractor,attrs.get_typed(FTMLKey::AnswerClassPts,str::parse)).unwrap_or(AnswerKind::Trait(0.0));
            extractor.push_answer_class(id,kind);
            Some(OpenFTMLElement::AnswerClass)
        }

        pub fn ac_feedback<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            Some(OpenFTMLElement::AnswerClassFeedback)
        }

        pub fn multiple_choice_block<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let styles = opt!(extractor,attrs.get_typed(FTMLKey::Styles, 
                |s| Result::<_,()>::Ok(s.split(',').map(|s| s.trim().to_string().into_boxed_str()).collect::<Vec<_>>().into_boxed_slice())
            )).unwrap_or_default();
            let inline = styles.iter().any(|s| &**s == "inline");
            extractor.open_choice_block(true,styles);
            Some(OpenFTMLElement::ChoiceBlock{multiple:true,inline})
        }

        pub fn single_choice_block<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let styles = opt!(extractor,attrs.get_typed(FTMLKey::Styles, 
                |s| Result::<_,()>::Ok(s.split(',').map(|s| s.trim().to_string().into_boxed_str()).collect::<Vec<_>>().into_boxed_slice())
            )).unwrap_or_default();
            let inline = styles.iter().any(|s| &**s == "inline");
            extractor.open_choice_block(false,styles);
            Some(OpenFTMLElement::ChoiceBlock{multiple:false,inline})
        }

        pub fn problem_choice<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let correct = attrs.get_bool(FTMLKey::ProblemChoice);//attrs.take_bool(FTMLKey::ProblemChoice);
            attrs.set(FTMLKey::ProblemChoice.attr_name(), "");
            extractor.push_problem_choice(correct);
            Some(OpenFTMLElement::ProblemChoice)
        }

        pub fn problem_choice_verdict<E:FTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            Some(OpenFTMLElement::ProblemChoiceVerdict)
        }

        pub fn problem_choice_feedback<E:FTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            Some(OpenFTMLElement::ProblemChoiceFeedback)
        }

        #[allow(clippy::cast_precision_loss)]
        pub fn fillinsol<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let val = attrs.get_typed(FTMLKey::ProblemFillinsolWidth, 
                |s| {
                    if s.contains('.') {
                        s.parse::<f32>().map_err(|_| ())
                    } else {
                        s.parse::<i32>().map(|i| i as f32).map_err(|_| ())
                    }
                }
            ).ok();
            extractor.open_fillinsol(val);
            Some(OpenFTMLElement::Fillinsol(val))
        }

        pub fn fillinsol_case<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let Some(val) = attrs.remove(FTMLKey::ProblemFillinsolCase) else {unreachable!()};
            let verdict = attrs.take_bool(FTMLKey::ProblemFillinsolCaseVerdict);
            let Some(value) = attrs.remove(FTMLKey::ProblemFillinsolCaseValue) else {
                extractor.add_error(FTMLError::IncompleteArgs(5));
                return None
            };
            let Some(opt) = FillInSolOption::from_values(&val,&value,verdict) else {
                extractor.add_error(FTMLError::IncompleteArgs(6));
                return None
            };
            extractor.push_fillinsol_case(opt);
            Some(OpenFTMLElement::FillinsolCase)
        }

        pub fn doctitle<E:FTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            Some(OpenFTMLElement::Doctitle)
        }

        pub fn title<E:FTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            Some(OpenFTMLElement::Title)
        }

        pub fn precondition<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let uri = err!(extractor,attrs.get_symbol_uri(FTMLKey::PreconditionSymbol,extractor));
            let dim = err!(extractor,attrs.get_typed(FTMLKey::PreconditionDimension,str::parse));
            extractor.add_precondition(uri, dim);
            None
        }

        pub fn objective<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let uri = err!(extractor,attrs.get_symbol_uri(FTMLKey::ObjectiveSymbol,extractor));
            let dim = err!(extractor,attrs.get_typed(FTMLKey::ObjectiveDimension,str::parse));
            extractor.add_objective(uri, dim);
            None
        }

        pub fn symdecl<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let uri = err!(extractor,attrs.get_new_symbol_uri(FTMLKey::Symdecl,extractor));
            let role = opt!(extractor,attrs.get_typed(FTMLKey::Role, 
                |s| Result::<_,()>::Ok(s.split(',').map(|s| s.trim().to_string().into_boxed_str()).collect::<Vec<_>>().into_boxed_slice())
            )).unwrap_or_default();
            let assoctype = opt!(extractor,attrs.get_typed(FTMLKey::AssocType,AssocType::from_str));
            let arity = opt!(extractor,attrs.get_typed(FTMLKey::Args,ArgSpec::from_str)).unwrap_or_default();
            let reordering = attrs.get(FTMLKey::ArgumentReordering).map(|s| Into::<String>::into(s).into_boxed_str());
            let macroname = attrs.get(FTMLKey::Macroname).map(|s| Into::<String>::into(s).into_boxed_str());
            extractor.open_decl();
            Some(OpenFTMLElement::Symdecl { uri, arity, macroname, role, assoctype, reordering })
        }

        pub fn vardecl<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            do_vardecl(extractor, attrs, nexts,FTMLKey::Vardef, false)
        }
        pub fn varseq<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            do_vardecl(extractor, attrs, nexts, FTMLKey::Varseq, true)
        }

        pub fn do_vardecl<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>,tag:FTMLKey,is_seq:bool) -> Option<OpenFTMLElement> {
            let Some(name) = attrs.get(tag).and_then(|v| Name::from_str(v.as_ref()).ok()) else {
                extractor.add_error(FTMLError::InvalidKeyFor(tag.as_str(), None));
                return None
            };
            let role = opt!(extractor,attrs.get_typed(FTMLKey::Role, 
                |s| Result::<_,()>::Ok(s.split(',').map(|s| s.trim().to_string().into_boxed_str()).collect::<Vec<_>>().into_boxed_slice())
            )).unwrap_or_default();
            let assoctype = opt!(extractor,attrs.get_typed(FTMLKey::AssocType,AssocType::from_str));
            let arity = opt!(extractor,attrs.get_typed(FTMLKey::Args,ArgSpec::from_str)).unwrap_or_default();
            let reordering = attrs.get(FTMLKey::ArgumentReordering).map(|s| Into::<String>::into(s).into_boxed_str());
            let macroname = attrs.get(FTMLKey::Macroname).map(|s| Into::<String>::into(s).into_boxed_str());
            let bind = attrs.get_bool(FTMLKey::Bind);
            extractor.open_decl();
            let uri = extractor.get_narrative_uri() & name;
            Some(OpenFTMLElement::Vardecl { uri, arity, macroname, role, assoctype, reordering, bind, is_seq })
        }

        pub fn notation<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let symbol = if let Ok(s) = attrs.get_symbol_uri(FTMLKey::Notation, extractor) {
                VarOrSym::S(s.into())
            } else if let Some(v) = attrs.get(FTMLKey::Notation) {
                let Ok(n) = v.as_ref().parse() else {
                    extractor.add_error(FTMLError::InvalidURI(format!("10: {}",v.as_ref())));
                    return None
                };
                VarOrSym::V(PreVar::Unresolved(n))
            } else {
                extractor.add_error(FTMLError::InvalidKeyFor(FTMLKey::Notation.as_str(), None));
                return None
            };
            let mut fragment = attrs.get(FTMLKey::NotationFragment).map(|s| Into::<String>::into(s).into_boxed_str());
            if fragment.as_ref().is_some_and(|s| s.is_empty()) { fragment = None };
            let id = fragment.as_ref().map_or("notation", |s| &**s).to_string();
            let id = extractor.new_id(Cow::Owned(id));
            let prec = if let Some(v) = attrs.get(FTMLKey::Precedence) {
                if let Ok(v) = isize::from_str(v.as_ref()) { v } else {
                    extractor.add_error(FTMLError::InvalidKeyFor(FTMLKey::Precedence.as_str(), None));
                    return None
                }
            } else {0};
            let mut argprecs = SmallVec::default();
            if let Some(s) = attrs.get(FTMLKey::Argprecs) {
                for s in s.as_ref().split(',') {
                    if s.is_empty() { continue }
                    if let Ok(v) =  isize::from_str(s.trim()) { argprecs.push(v) } else {
                        extractor.add_error(FTMLError::InvalidKeyFor(FTMLKey::Argprecs.as_str(), None));
                        return None
                    } 
                }
            }
            extractor.open_notation();
            Some(OpenFTMLElement::Notation { id, symbol, precedence: prec, argprecs })
        }

        pub fn notationcomp<E:FTMLExtractor>(_extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            attrs.remove(FTMLKey::NotationComp);
            attrs.remove(FTMLKey::Term);
            attrs.remove(FTMLKey::Head);
            attrs.remove(FTMLKey::NotationId);
            attrs.remove(FTMLKey::Invisible);
            nexts.retain(|r| !matches!(r.tag,FTMLKey::Term|FTMLKey::Head|FTMLKey::NotationId|FTMLKey::Invisible));
            Some(OpenFTMLElement::NotationComp)
        }
        pub fn notationopcomp<E:FTMLExtractor>(_extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            attrs.remove(FTMLKey::NotationComp);
            attrs.remove(FTMLKey::Term);
            attrs.remove(FTMLKey::Head);
            attrs.remove(FTMLKey::NotationId);
            attrs.remove(FTMLKey::Invisible);
            nexts.retain(|r| !matches!(r.tag,FTMLKey::Term|FTMLKey::Head|FTMLKey::NotationId|FTMLKey::Invisible));
            Some(OpenFTMLElement::NotationOpComp)
        }
        
        pub fn definiendum<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let uri = err!(extractor,attrs.get_symbol_uri(FTMLKey::Definiendum,extractor));
            extractor.add_definiendum(uri.clone());
            Some(OpenFTMLElement::Definiendum(uri))
        }

        pub fn r#type<E:FTMLExtractor>(extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            if extractor.in_term() {
                extractor.add_error(FTMLError::InvalidKey);
                return None
            }
            extractor.set_in_term(true);
            Some(OpenFTMLElement::Type)
        }

        pub fn conclusion<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let uri = err!(extractor,attrs.get_symbol_uri(FTMLKey::Conclusion,extractor));
            let in_term = extractor.in_term();
            extractor.set_in_term(true);
            Some(OpenFTMLElement::Conclusion { uri, in_term })
        }

        pub fn definiens<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let uri = opt!(extractor,attrs.get_symbol_uri(FTMLKey::Definiens,extractor));
            let in_term = extractor.in_term();
            extractor.set_in_term(true);
            Some(OpenFTMLElement::Definiens { uri, in_term })
        }

        pub fn mmtrule<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let id = attrs.get(FTMLKey::Rule).unwrap_or_else(|| unreachable!()).as_ref().to_string().into_boxed_str();
            extractor.open_args();
            Some(OpenFTMLElement::MMTRule(id))
        }

        pub fn argsep<E:FTMLExtractor>(_extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            attrs.remove(FTMLKey::Term);
            attrs.remove(FTMLKey::ArgSep);
            attrs.remove(FTMLKey::Head);
            attrs.remove(FTMLKey::NotationId);
            attrs.remove(FTMLKey::Invisible);
            nexts.retain(|r| !matches!(r.tag,FTMLKey::Term|FTMLKey::Head|FTMLKey::NotationId|FTMLKey::Invisible|FTMLKey::ArgSep));
            Some(OpenFTMLElement::ArgSep)
        }

        pub fn argmap<E:FTMLExtractor>(_extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            attrs.remove(FTMLKey::Term);
            attrs.remove(FTMLKey::Head);
            attrs.remove(FTMLKey::ArgMap);
            attrs.remove(FTMLKey::NotationId);
            attrs.remove(FTMLKey::Invisible);
            nexts.retain(|r| !matches!(r.tag,FTMLKey::Term|FTMLKey::Head|FTMLKey::NotationId|FTMLKey::Invisible|FTMLKey::ArgMap));
            Some(OpenFTMLElement::ArgMap)
        }

        pub fn argmapsep<E:FTMLExtractor>(_extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            attrs.remove(FTMLKey::Term);
            attrs.remove(FTMLKey::Head);
            attrs.remove(FTMLKey::ArgMapSep);
            attrs.remove(FTMLKey::NotationId);
            attrs.remove(FTMLKey::Invisible);
            nexts.retain(|r| !matches!(r.tag,FTMLKey::Term|FTMLKey::Head|FTMLKey::NotationId|FTMLKey::Invisible|FTMLKey::ArgMapSep));
            Some(OpenFTMLElement::ArgMapSep)
        }

        pub fn term<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            if extractor.in_notation() { return None }
            let notation = attrs.value(FTMLKey::NotationId.attr_name()).and_then(|n| {
                let asr = n.as_ref().trim();
                if asr.is_empty() {return None }
                match asr.parse::<Name>() {
                    Ok(n) => Some(n),
                    Err(e) => {
                        extractor.add_error(FTMLError::InvalidURI(format!("12: {}",n.as_ref())));
                        Some(ERROR.clone())
                    }
                }
            });
            let head = match attrs.value(FTMLKey::Head.attr_name()) {
                None => {
                    extractor.add_error(FTMLError::MissingHeadForTerm);
                    VarOrSym::V(PreVar::Unresolved(ERROR.clone()))
                },
                Some(v) => {
                    let v = v.as_ref();
                    v.parse::<SymbolURI>().ok().map_or_else(
                        || v.parse::<ModuleURI>().map_or_else(
                              |_| DocumentElementURI::from_str(v).map_or_else(
                                |_| {
                                            if v.contains('?') {
                                                tracing::warn!("Suspicious variable name containing '?': {v}");
                                            }
                                            match v.parse() {
                                                Ok(v) => Some(VarOrSym::V(PreVar::Unresolved(v))),
                                                Err(e) => {
                                                    extractor.add_error(FTMLError::InvalidURI(format!("13: {v}")));
                                                    None
                                                }
                                            }
                                        },
                                        |d| Some(VarOrSym::V(PreVar::Resolved(d)))
                              ),
                            |m| Some(VarOrSym::S(m.into()))),
                        |s| Some(VarOrSym::S(s.into()))
                    )?
                }
            };
            //attrs.set(tagstrings::HEAD,&head.to_string());
            let kind = attrs.value(FTMLKey::Term.attr_name()).unwrap_or_else(|| unreachable!());
            let kind: OpenTermKind = kind.as_ref().parse().unwrap_or_else(|()| {
                extractor.add_error(FTMLError::InvalidTermKind(kind.into()));
                OpenTermKind::OMA
            });
            let term = match (kind,head) {
                (OpenTermKind::OMID|OpenTermKind::OMV,VarOrSym::S(uri))
                    => OpenTerm::Symref { uri, notation },
                (OpenTermKind::OMID|OpenTermKind::OMV,VarOrSym::V(name))
                    => OpenTerm::Varref { name, notation },
                (OpenTermKind::OML,VarOrSym::V(PreVar::Unresolved(name)))
                    => {
                        extractor.open_decl();
                        OpenTerm::OML { name}//, tp: None, df: None }
                    }
                (OpenTermKind::OMA,head) 
                    => {
                        extractor.open_args();
                        OpenTerm::OMA { head, notation}//, args: SmallVec::new() }
                    }
                (OpenTermKind::Complex,head)
                    => {
                    extractor.open_complex_term();
                    OpenTerm::Complex(head)
                },
                (k,head) => {
                    extractor.add_error(FTMLError::InvalidHeadForTermKind(k,head.clone()));
                    extractor.open_args();
                    OpenTerm::OMA { head, notation}//, args: SmallVec::new() }
                }
            };
            let is_top = if extractor.in_term() { false } else {
                extractor.set_in_term(true);
                true
            };
            Some(OpenFTMLElement::OpenTerm{term, is_top})
        }

        pub fn arg<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let Some(value) = attrs.value(FTMLKey::Arg.attr_name()) else {
                extractor.add_error(FTMLError::InvalidArgSpec);
                return None
            };
            let arg = OpenArg::from_strs(value,attrs.value(FTMLKey::ArgMode.attr_name()));
            let Some(arg) = arg else {
                extractor.add_error(FTMLError::InvalidArgSpec);
                return None
            };
            Some(OpenFTMLElement::Arg(arg))
        }

        pub fn headterm<E:FTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            Some(OpenFTMLElement::HeadTerm)
        }

        pub fn inputref<E:FTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let uri = err!(extractor,attrs.get_document_uri(FTMLKey::InputRef,extractor));
            let id = attrs.get_id(extractor,Cow::Owned(uri.name().last_name().to_string()));
            Some(OpenFTMLElement::Inputref { uri, id })
        }

        pub fn ifinputref<E:FTMLExtractor>(_extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            let value = attrs.get_bool(FTMLKey::IfInputref); 
            Some(OpenFTMLElement::IfInputref(value))
        }

        pub fn comp<E:FTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            Some(OpenFTMLElement::Comp)
        }

        pub fn maincomp<E:FTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            Some(OpenFTMLElement::MainComp)
        }

        pub fn defcomp<E:FTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenFTMLElement> {
            Some(OpenFTMLElement::DefComp)
        }

    //}
}
