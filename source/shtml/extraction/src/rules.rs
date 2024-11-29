use smallvec::SmallVec;
use crate::extractor::{Attributes, SHTMLExtractor};
use crate::open::OpenSHTMLElement;
use crate::prelude::SHTMLNode;
use crate::tags::SHTMLTag;

#[allow(type_alias_bounds)]
pub type Call<E:SHTMLExtractor> = for <'a> fn(&mut E,&mut E::Attr<'a>,&mut SmallVec<SHTMLExtractionRule<E>,4>) -> Option<OpenSHTMLElement>;

#[derive(PartialEq, Eq,Hash)]
pub struct SHTMLExtractionRule<E:SHTMLExtractor>{
    pub(crate) tag:SHTMLTag, pub(crate) attr:&'static str,
    call:Call<E>
}
impl<E:SHTMLExtractor> Copy for SHTMLExtractionRule<E> {}
impl<E:SHTMLExtractor> Clone for SHTMLExtractionRule<E> {
    #[inline]
    fn clone(&self) -> Self { *self }
}
impl<E:SHTMLExtractor> SHTMLExtractionRule<E> {
    #[inline]
    pub(crate) const fn new(tag:SHTMLTag,attr:&'static str,call:Call<E>) -> Self {
        Self { tag,attr,call }
    }
    #[inline]
    fn applies(&self, s:&str) -> bool { 
        //tracing::trace!("{s} == {}? => {}",self.attr,s == self.attr);
        s == self.attr 
    }
}

#[derive(Debug,Clone)]
pub struct SHTMLElements {
    pub elems:SmallVec<OpenSHTMLElement,4>
}
impl SHTMLElements {
    #[inline]#[must_use]
    pub fn is_empty(&self) -> bool {
        self.elems.is_empty()
    }
    #[inline]#[must_use]
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
    pub fn close<E:SHTMLExtractor,N:SHTMLNode>(&mut self,extractor:&mut E,node:&N) {
        let mut ret = Self{elems:SmallVec::default()};
        while let Some(e) = self.elems.pop() {
            if let Some(r) = e.close(self,&mut ret,extractor,node) {
                ret.elems.push(r);
            }
        }
        *self = ret;
    }
    #[inline]#[must_use]
    pub fn take(self) -> SmallVec<OpenSHTMLElement,4> {
        self.elems
    }
}
impl<'a> IntoIterator for &'a SHTMLElements {
    type Item = &'a OpenSHTMLElement;
    type IntoIter = std::iter::Rev<std::slice::Iter<'a,OpenSHTMLElement>>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.elems.iter().rev()
    }
}


pub trait RuleSet<E:SHTMLExtractor> {
    type I<'i>:Iterator<Item=SHTMLExtractionRule<E>> where Self:'i,E:'i;

    fn iter_rules(&self) -> Self::I<'_>;

    #[allow(clippy::cognitive_complexity)]
    fn applicable_rules<'a>(&self,extractor:&mut E,attrs:&'a mut E::Attr<'a>) -> Option<SHTMLElements> {
        let mut stripped = attrs.keys().filter(|s| {
            if s.starts_with("data-shtml-") {
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
                tracing::debug!("found {:?}",rule.tag);
                rules.push(rule);
                stripped.remove(i);
            }
        }
        for s in stripped {
            tracing::warn!("Unknown shtml attribute: {s} = {}",attrs.value(s).expect("wut").as_ref());
        }
        //tracing::trace!("Found {:?} applicable rules",rules.len());
        if rules.is_empty() {
            //tracing::trace!("returning elements");
            return None
        }
        Self::do_rules(extractor, attrs, rules)
    }

    fn do_rules<'a>(extractor:&mut E,attrs:&'a mut E::Attr<'a>,mut rules:SmallVec<SHTMLExtractionRule<E>,4>) -> Option<SHTMLElements> {
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
            SHTMLElements { elems: ret }
        )}
    }

}
impl<const L:usize,E:SHTMLExtractor> RuleSet<E> for [SHTMLExtractionRule<E>;L] {
    type I<'i> = std::iter::Copied<std::slice::Iter<'i, SHTMLExtractionRule<E>>> where E:'i;
    fn iter_rules(&self) -> Self::I<'_> { self.iter().copied() }
}

#[allow(clippy::module_inception)]
#[allow(unused_macros)]
pub mod rules {
    use immt_ontology::content::declarations::symbols::{ArgSpec, AssocType};
    use immt_ontology::narration::paragraphs::ParagraphKind;
    use immt_ontology::uris::{DocumentElementURI, Name};
    use immt_utils::vecmap::VecSet;
    use smallvec::SmallVec;
    use crate::errors::SHTMLError;
    use crate::open::OpenSHTMLElement;
    use crate::prelude::{Attributes, SHTMLExtractor};
    use crate::rules::SHTMLExtractionRule;
    use crate::open::terms::{OpenArg, OpenTerm, OpenTermKind, PreVar, VarOrSym};
    use crate::tags::tagstrings;
    use std::borrow::Cow;
    use std::str::FromStr;

    //type Value<'a,E:SHTMLExtractor> = <E::Attr<'a> as Attributes>::Value<'a>;
    #[allow(type_alias_bounds)]
    type SV<E:SHTMLExtractor> = SmallVec<SHTMLExtractionRule<E>,4>;

    lazy_static::lazy_static! {
        static ref ERROR : Name = "ERROR".into();
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
                Err(SHTMLError::InvalidKeyFor(_, Some(s))) if s.is_empty() => None,
                Err(e@SHTMLError::InvalidKeyFor(_, Some(_))) => {
                    $extractor.add_error(e);
                    None
                }
                _ => None
            }
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    #[allow(clippy::unnecessary_wraps)]
    impl super::SHTMLTag {
        pub(crate) fn no_op<E:SHTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> { None }

        /*pub(crate) fn todo<E:SHTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>,tag:Self) -> Option<OpenSHTMLElement> {
            todo!("Tag {}",tag.as_str()) 
        }*/

        pub(crate) fn invisible<E:SHTMLExtractor>(_extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            if attrs.take_bool(Self::Invisible) {
                Some(OpenSHTMLElement::Invisible)
            } else { None }
        }

        pub(crate) fn setsectionlevel<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            let lvl = err!(extractor,attrs.get_section_level(Self::SetSectionLevel));
            Some(OpenSHTMLElement::SetSectionLevel(lvl))
        }

        pub(crate) fn importmodule<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            let uri = err!(extractor,attrs.take_module_uri(Self::ImportModule, extractor));
            Some(OpenSHTMLElement::ImportModule(uri))
        }

        pub(crate) fn usemodule<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            let uri = err!(extractor,attrs.take_module_uri(Self::UseModule, extractor));
            Some(OpenSHTMLElement::UseModule(uri))
        }

        pub(crate) fn module<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            let uri = err!(extractor,attrs.take_module_uri(Self::Module, extractor));
            let _ = attrs.take_language(Self::Language);
            let meta = opt!(extractor,attrs.take_module_uri(Self::Metatheory, extractor));
            let signature = opt!(extractor,attrs.take_language(Self::Signature));
            extractor.open_content(uri.clone());
            extractor.open_narrative(None);
            Some(OpenSHTMLElement::Module { 
                uri, meta, signature, 
                //narrative: Vec::new(), content: Vec::new() 
            })
        }

        pub(crate) fn mathstructure<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            let Some(uri) = err!(extractor,attrs.take_module_uri(Self::MathStructure, extractor)).into_symbol() else {
                extractor.add_error(SHTMLError::InvalidKeyFor(Self::MathStructure.as_str(), None));
                return None
            };
            let macroname = attrs.remove(Self::Macroname).map(|s| Into::<String>::into(s).into_boxed_str());
            extractor.open_content(uri.clone().into_module());
            extractor.open_narrative(None);
            Some(OpenSHTMLElement::MathStructure { 
                uri,macroname, //content: Vec::new(), narrative:Vec::new()
            })
        }

        pub(crate) fn morphism<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            let Some(v) = attrs.remove(Self::Morphism) else {
                extractor.add_error(SHTMLError::InvalidKeyFor(Self::Morphism.as_str(), None));
                return None
            };
            let Some(uri) = extractor.get_sym_uri_as_mod(v.as_ref()) else {
                extractor.add_error(SHTMLError::InvalidKeyFor(Self::Morphism.as_str(), Some(v)));
                return None
            };
            let Some(uri) = uri.into_symbol() else {
                extractor.add_error(SHTMLError::InvalidKeyFor(Self::Morphism.as_str(), Some(v)));
                return None
            };
            let domain = err!(extractor,attrs.take_module_uri(Self::MorphismDomain, extractor));
            let total = attrs.take_bool(Self::MorphismTotal);
            extractor.open_content(uri.clone().into_module());
            extractor.open_narrative(None);
            Some(OpenSHTMLElement::Morphism {
                uri:Some(uri), // TODO
                domain,total,//content:Vec::new(),narrative:Vec::new()
            })
        }

        pub(crate) fn assign<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            let symbol = err!(extractor,attrs.get_symbol_uri(Self::Assign,extractor));
            extractor.open_complex_term();
            Some(OpenSHTMLElement::Assign(symbol))
        }

        pub(crate) fn section<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            let lvl = err!(extractor,attrs.get_section_level(Self::Section));
            let id = attrs.get_id(extractor,Cow::Borrowed("section"));
            let uri = extractor.get_narrative_uri() & &*id;
            extractor.open_section(uri.clone());
            Some(OpenSHTMLElement::Section { lvl, uri })
        }

        pub(crate) fn definition<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            Self::do_paragraph(extractor, attrs, nexts, ParagraphKind::Definition)
        }
        pub(crate) fn paragraph<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            Self::do_paragraph(extractor, attrs, nexts, ParagraphKind::Paragraph)
        }
        pub(crate) fn assertion<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            Self::do_paragraph(extractor, attrs, nexts, ParagraphKind::Assertion)
        }
        pub(crate) fn example<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            Self::do_paragraph(extractor, attrs, nexts, ParagraphKind::Example)
        }
        pub(crate) fn proof<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            Self::do_paragraph(extractor, attrs, nexts, ParagraphKind::Proof)
        }
        pub(crate) fn subproof<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            Self::do_paragraph(extractor, attrs, nexts, ParagraphKind::SubProof)
        }

        fn do_paragraph<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>,kind:ParagraphKind) -> Option<OpenSHTMLElement> {
            let id = attrs.get_id(extractor,Cow::Borrowed(kind.as_str()));
            let uri = extractor.get_narrative_uri() & &*id;
            let inline = attrs.get_bool(Self::Inline);
            let mut fors = VecSet::new();
            if let Some(f) = attrs.get(Self::Fors) {
                for f in f.as_ref().split(',') {
                    if let Some(f) = extractor.get_sym_uri(f.trim()) {
                        fors.insert(f);
                    } else {
                        extractor.add_error(SHTMLError::InvalidKeyFor(Self::Fors.as_str(), Some(f.trim().into())));
                    };
                }
            }
            let styles = opt!(extractor,attrs.get_typed(Self::Styles, 
                |s| Result::<_,()>::Ok(s.split(',').map(|s| s.trim().to_string().into_boxed_str()).collect::<Vec<_>>().into_boxed_slice())
            )).unwrap_or_default();
            extractor.open_paragraph(uri.clone(), fors);
            Some(OpenSHTMLElement::Paragraph { kind, inline, styles,uri })
        }

        pub(crate) fn exercise<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            Self::do_exercise(extractor,attrs,nexts,false)
        }

        pub(crate) fn subexercise<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            Self::do_exercise(extractor,attrs,nexts,true)
        }

        fn do_exercise<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>,sub_exercise:bool) -> Option<OpenSHTMLElement> {
            let styles = opt!(extractor,attrs.get_typed(Self::Styles, 
                |s| Result::<_,()>::Ok(s.split(',').map(|s| s.trim().to_string().into_boxed_str()).collect::<Vec<_>>().into_boxed_slice())
            )).unwrap_or_default();
            let id = attrs.get_id(extractor,Cow::Borrowed("exercise"));
            let uri = extractor.get_narrative_uri() & &*id;
            let _ = attrs.take_language(Self::Language);
            let autogradable = attrs.get_bool(Self::Autogradable);
            let points = attrs.get(Self::ProblemPoints)
                .and_then(|s| s.as_ref().parse().ok());
            extractor.open_exercise(uri.clone());
            Some(OpenSHTMLElement::Exercise { sub_exercise, styles, uri, autogradable, points })
        }

        pub(crate) fn doctitle<E:SHTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            Some(OpenSHTMLElement::Doctitle)
        }

        pub(crate) fn title<E:SHTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            Some(OpenSHTMLElement::Title)
        }

        pub(crate) fn precondition<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            let uri = err!(extractor,attrs.get_symbol_uri(Self::PreconditionSymbol,extractor));
            let dim = err!(extractor,attrs.get_typed(Self::PreconditionDimension,|s| s.parse()));
            extractor.add_precondition(uri, dim);
            None
        }

        pub(crate) fn objective<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            let uri = err!(extractor,attrs.get_symbol_uri(Self::ObjectiveSymbol,extractor));
            let dim = err!(extractor,attrs.get_typed(Self::ObjectiveDimension,|s| s.parse()));
            extractor.add_objective(uri, dim);
            None
        }

        pub(crate) fn symdecl<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            let uri = err!(extractor,attrs.get_symbol_uri(Self::Symdecl,extractor));
            let role = opt!(extractor,attrs.get_typed(Self::Role, 
                |s| Result::<_,()>::Ok(s.split(',').map(|s| s.trim().to_string().into_boxed_str()).collect::<Vec<_>>().into_boxed_slice())
            )).unwrap_or_default();
            let assoctype = opt!(extractor,attrs.get_typed(Self::AssocType,AssocType::from_str));
            let arity = opt!(extractor,attrs.get_typed(Self::Args,ArgSpec::from_str)).unwrap_or_default();
            let reordering = attrs.get(Self::ArgumentReordering).map(|s| Into::<String>::into(s).into_boxed_str());
            let macroname = attrs.get(Self::Macroname).map(|s| Into::<String>::into(s).into_boxed_str());
            extractor.open_decl();
            Some(OpenSHTMLElement::Symdecl { uri, arity, macroname, role, assoctype, reordering })
        }

        pub(crate) fn vardecl<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            Self::do_vardecl(extractor, attrs, nexts,Self::Vardef, false)
        }
        pub(crate) fn varseq<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            Self::do_vardecl(extractor, attrs, nexts, Self::Varseq, true)
        }

        pub(crate) fn do_vardecl<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>,tag:Self,is_seq:bool) -> Option<OpenSHTMLElement> {
            let Some(name) = attrs.get(tag).and_then(|v| Name::from_str(v.as_ref()).ok()) else {
                extractor.add_error(SHTMLError::InvalidKeyFor(tag.as_str(), None));
                return None
            };
            let role = opt!(extractor,attrs.get_typed(Self::Role, 
                |s| Result::<_,()>::Ok(s.split(',').map(|s| s.trim().to_string().into_boxed_str()).collect::<Vec<_>>().into_boxed_slice())
            )).unwrap_or_default();
            let assoctype = opt!(extractor,attrs.get_typed(Self::AssocType,AssocType::from_str));
            let arity = opt!(extractor,attrs.get_typed(Self::Args,ArgSpec::from_str)).unwrap_or_default();
            let reordering = attrs.get(Self::ArgumentReordering).map(|s| Into::<String>::into(s).into_boxed_str());
            let macroname = attrs.get(Self::Macroname).map(|s| Into::<String>::into(s).into_boxed_str());
            let bind = attrs.get_bool(Self::Bind);
            extractor.open_decl();
            let uri = extractor.get_narrative_uri() & name;
            Some(OpenSHTMLElement::Vardecl { uri, arity, macroname, role, assoctype, reordering, bind, is_seq })
        }

        pub(crate) fn notation<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            let symbol = if let Ok(s) = attrs.get_symbol_uri(Self::Notation, extractor) {
                VarOrSym::S(s.into())
            } else if let Some(v) = attrs.get(Self::Notation) {
                VarOrSym::V(PreVar::Unresolved(v.as_ref().parse().unwrap_or_else(|_| unreachable!())))
            } else {
                extractor.add_error(SHTMLError::InvalidKeyFor(Self::Notation.as_str(), None));
                return None
            };
            let mut fragment = attrs.get(Self::NotationFragment).map(|s| Into::<String>::into(s).into_boxed_str());
            if fragment.as_ref().is_some_and(|s| s.is_empty()) { fragment = None };
            let id = fragment.as_ref().map_or("notation", |s| &**s).to_string();
            let id = extractor.new_id(Cow::Owned(id));
            let prec = if let Some(v) = attrs.get(Self::Precedence) {
                if let Ok(v) = isize::from_str(v.as_ref()) { v } else {
                    extractor.add_error(SHTMLError::InvalidKeyFor(Self::Precedence.as_str(), None));
                    return None
                }
            } else {0};
            let mut argprecs = SmallVec::default();
            if let Some(s) = attrs.get(Self::Argprecs) {
                for s in s.as_ref().split(',') {
                    if s.is_empty() { continue }
                    if let Ok(v) =  isize::from_str(s.trim()) { argprecs.push(v) } else {
                        extractor.add_error(SHTMLError::InvalidKeyFor(Self::Argprecs.as_str(), None));
                        return None
                    } 
                }
            }
            extractor.open_notation();
            Some(OpenSHTMLElement::Notation { id, symbol, precedence: prec, argprecs })
        }

        pub(crate) fn notationcomp<E:SHTMLExtractor>(_extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            attrs.remove(Self::NotationComp);
            attrs.remove(Self::Term);
            attrs.remove(Self::Head);
            attrs.remove(Self::NotationId);
            attrs.remove(Self::Invisible);
            nexts.retain(|r| !matches!(r.tag,Self::Term|Self::Head|Self::NotationId|Self::Invisible));
            Some(OpenSHTMLElement::NotationComp)
        }
        pub(crate) fn notationopcomp<E:SHTMLExtractor>(_extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            attrs.remove(Self::NotationComp);
            attrs.remove(Self::Term);
            attrs.remove(Self::Head);
            attrs.remove(Self::NotationId);
            attrs.remove(Self::Invisible);
            nexts.retain(|r| !matches!(r.tag,Self::Term|Self::Head|Self::NotationId|Self::Invisible));
            Some(OpenSHTMLElement::NotationOpComp)
        }
        
        pub(crate) fn definiendum<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            let uri = err!(extractor,attrs.get_symbol_uri(Self::Definiendum,extractor));
            extractor.add_definiendum(uri.clone());
            Some(OpenSHTMLElement::Definiendum(uri))
        }

        pub(crate) fn r#type<E:SHTMLExtractor>(extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            if extractor.in_term() {
                extractor.add_error(SHTMLError::InvalidKey);
                return None
            }
            extractor.set_in_term(true);
            Some(OpenSHTMLElement::Type)
        }

        pub(crate) fn conclusion<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            let uri = err!(extractor,attrs.get_symbol_uri(Self::Conclusion,extractor));
            let in_term = extractor.in_term();
            extractor.set_in_term(true);
            Some(OpenSHTMLElement::Conclusion { uri, in_term })
        }

        pub(crate) fn definiens<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            let uri = opt!(extractor,attrs.get_symbol_uri(Self::Definiens,extractor));
            let in_term = extractor.in_term();
            extractor.set_in_term(true);
            Some(OpenSHTMLElement::Definiens { uri, in_term })
        }

        pub(crate) fn mmtrule<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            let id = attrs.get(Self::Rule).unwrap_or_else(|| unreachable!()).as_ref().to_string().into_boxed_str();
            extractor.open_args();
            Some(OpenSHTMLElement::MMTRule(id))
        }

        pub(crate) fn argsep<E:SHTMLExtractor>(_extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            attrs.remove(Self::Term);
            attrs.remove(Self::ArgSep);
            attrs.remove(Self::Head);
            attrs.remove(Self::NotationId);
            attrs.remove(Self::Invisible);
            nexts.retain(|r| !matches!(r.tag,Self::Term|Self::Head|Self::NotationId|Self::Invisible|Self::ArgSep));
            Some(OpenSHTMLElement::ArgSep)
        }

        pub(crate) fn argmap<E:SHTMLExtractor>(_extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            attrs.remove(Self::Term);
            attrs.remove(Self::Head);
            attrs.remove(Self::ArgMap);
            attrs.remove(Self::NotationId);
            attrs.remove(Self::Invisible);
            nexts.retain(|r| !matches!(r.tag,Self::Term|Self::Head|Self::NotationId|Self::Invisible|Self::ArgMap));
            Some(OpenSHTMLElement::ArgMap)
        }

        pub(crate) fn argmapsep<E:SHTMLExtractor>(_extractor:&mut E,attrs:&mut E::Attr<'_>,nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            attrs.remove(Self::Term);
            attrs.remove(Self::Head);
            attrs.remove(Self::ArgMapSep);
            attrs.remove(Self::NotationId);
            attrs.remove(Self::Invisible);
            nexts.retain(|r| !matches!(r.tag,Self::Term|Self::Head|Self::NotationId|Self::Invisible|Self::ArgMapSep));
            Some(OpenSHTMLElement::ArgMapSep)
        }

        pub(crate) fn term<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            if extractor.in_notation() { return None }
            let notation = attrs.value(tagstrings::NOTATION_ID).map(|n|
                n.as_ref().into()
            );
            let head = match attrs.value(tagstrings::HEAD) {
                None => {
                    extractor.add_error(SHTMLError::MissingHeadForTerm);
                    VarOrSym::V(PreVar::Unresolved(ERROR.clone()))
                },
                Some(v) => {
                    let v = v.as_ref();
                    extractor.get_sym_uri(v).map_or_else(
                        || extractor.get_mod_uri(v).map_or_else(
                              || DocumentElementURI::from_str(v).map_or_else(
                                |_| {
                                            if v.contains('?') {
                                                tracing::warn!("Suspicious variable name containing '?': {v}");
                                            }
                                            VarOrSym::V(PreVar::Unresolved(v.into()))
                                        },
                                        |d| VarOrSym::V(PreVar::Resolved(d))
                              ),
                            |m| VarOrSym::S(m.into())),
                        |s| VarOrSym::S(s.into())
                    )
                }
            };
            //attrs.set(tagstrings::HEAD,&head.to_string());
            let kind = attrs.value(tagstrings::TERM).unwrap_or_else(|| unreachable!());
            let kind: OpenTermKind = kind.as_ref().parse().unwrap_or_else(|()| {
                extractor.add_error(SHTMLError::InvalidTermKind(kind.into()));
                OpenTermKind::OMA
            });
            let term = match (kind,head) {
                (OpenTermKind::OMID,VarOrSym::S(uri))
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
                    extractor.add_error(SHTMLError::InvalidHeadForTermKind(k,head.clone()));
                    extractor.open_args();
                    OpenTerm::OMA { head, notation}//, args: SmallVec::new() }
                }
            };
            let is_top = if extractor.in_term() { false } else {
                extractor.set_in_term(true);
                true
            };
            Some(OpenSHTMLElement::OpenTerm{term, is_top})
        }

        pub(crate) fn arg<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            let Some(value) = attrs.value(tagstrings::ARG) else {
                extractor.add_error(SHTMLError::InvalidArgSpec);
                return None
            };
            let arg = OpenArg::from_strs(value,attrs.value(tagstrings::ARG_MODE));
            let Some(arg) = arg else {
                extractor.add_error(SHTMLError::InvalidArgSpec);
                return None
            };
            Some(OpenSHTMLElement::Arg(arg))
        }

        pub(crate) fn headterm<E:SHTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            Some(OpenSHTMLElement::HeadTerm)
        }

        pub(crate) fn inputref<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            let uri = err!(extractor,attrs.get_document_uri(Self::InputRef,extractor));
            let id = attrs.get_id(extractor,Cow::Owned(uri.name().last_name().to_string()));
            Some(OpenSHTMLElement::Inputref { uri, id })
        }

        pub(crate) fn ifinputref<E:SHTMLExtractor>(_extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            let value = attrs.get_bool(Self::IfInputref); 
            Some(OpenSHTMLElement::IfInputref(value))
        }

        pub(crate) fn comp<E:SHTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            Some(OpenSHTMLElement::Comp)
        }

        pub(crate) fn maincomp<E:SHTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            Some(OpenSHTMLElement::MainComp)
        }

    }
}
