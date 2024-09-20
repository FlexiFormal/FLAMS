use smallvec::SmallVec;
use crate::extractor::{Attributes, SHTMLExtractor};
use crate::open::OpenSHTMLElement;
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
        tracing::trace!("{s} == {}? => {}",self.attr,s == self.attr);
        s == self.attr 
    }
}

#[derive(Debug,Clone)]
pub struct SHTMLElements {
    elems:SmallVec<OpenSHTMLElement,4>
}
impl SHTMLElements {
    #[inline]#[must_use]
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
}
impl<'a> IntoIterator for &'a SHTMLElements {
    type Item = &'a OpenSHTMLElement;
    type IntoIter = std::iter::Rev<std::slice::Iter<'a,OpenSHTMLElement>>;
    fn into_iter(self) -> Self::IntoIter {
        self.elems.iter().rev()
    }
}


pub trait RuleSet<E:SHTMLExtractor> {
    type I<'i>:Iterator<Item=SHTMLExtractionRule<E>> where Self:'i,E:'i;

    fn iter_rules(&self) -> Self::I<'_>;

    fn applicable_rules<'a>(&self,extractor:&mut E,attrs:&'a mut E::Attr<'a>) -> Option<SHTMLElements> {
        let mut stripped = attrs.keys().filter(|s| {
            if s.starts_with("shtml:") {
                tracing::trace!("attribute {s} ({:?})",std::thread::current().id());
                true
            } else { false }
        }).collect::<SmallVec<_,4>>();
        if stripped.is_empty() {
            tracing::trace!("no applicable attributes");
            return None
        }
        tracing::trace!("Found {:?} applicable attributes",stripped.len());
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
        tracing::trace!("Found {:?} applicable rules",rules.len());
        if rules.is_empty() {
            tracing::trace!("returning elements");
            return None
        }
        Self::do_rules(extractor, attrs, rules)
    }

    fn do_rules<'a>(extractor:&mut E,attrs:&'a mut E::Attr<'a>,mut rules:SmallVec<SHTMLExtractionRule<E>,4>) -> Option<SHTMLElements> {
        rules.reverse();
        let mut ret = SmallVec::new();
        while let Some(rule) = rules.pop() {
            tracing::trace!("calling rule {:?}",rule.tag);
            if let Some(r) = (rule.call)(extractor,attrs,&mut rules) {
                ret.push(r);
            }
        }
        tracing::trace!("returning elements");
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
    use immt_ontology::uris::Name;
    use smallvec::SmallVec;
    use crate::errors::SHTMLError;
    use crate::open::OpenSHTMLElement;
    use crate::prelude::{Attributes, SHTMLExtractor};
    use crate::rules::SHTMLExtractionRule;
    use crate::open::terms::{OpenArg, OpenTerm, OpenTermKind, PreVar, VarOrSym};
    use crate::tags::tagstrings;

    //type Value<'a,E:SHTMLExtractor> = <E::Attr<'a> as Attributes>::Value<'a>;
    #[allow(type_alias_bounds)]
    type SV<E:SHTMLExtractor> = SmallVec<SHTMLExtractionRule<E>,4>;

    lazy_static::lazy_static! {
        static ref ERROR : Name = "ERROR".into();
    }

    macro_rules! do_macros {
        ($extractor:ident,$attrs:ident) => {
            macro_rules! err {
                ($e:expr) => {{
                    use SHTMLError::*;
                    $extractor.add_error($e);
                    return None
                }}
            }
            macro_rules! moduri {
                ($tag:ident) => {
                    if let Some(s) = $attrs.value(tagstrings::$tag){
                        if let Some(uri) = $extractor.get_mod_uri(s.as_ref()) {
                            uri
                        } else {err!(InvalidModuleURI(s.into()))}
                    } else {err!(InvalidModuleURI(String::new()))}
                }
            }
            macro_rules! docuri {
                ($tag:ident) => {
                    if let Some(s) = $attrs.value(tagstrings::$tag){
                        if let Some(uri) = $extractor.get_doc_uri(s.as_ref()) {
                            uri
                        } else {err!(MissingInputrefURI)}
                    } else {err!(MissingInputrefURI)}
                }
            }
            macro_rules! id {
                () => {
                    $attrs.value(tagstrings::ID).map(|s| Into::<String>::into(s).into_boxed_str())
                };
                ($lit:literal) => {
                    $attrs.value(tagstrings::ID).map(|s| Into::<String>::into(s).into_boxed_str()).unwrap_or_else(
                        ||  $extractor.new_id($lit)
                    )
                };
            }
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    #[allow(clippy::unnecessary_wraps)]
    impl super::SHTMLTag {
        pub(crate) fn no_op<E:SHTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> { None }

        pub(crate) fn todo<E:SHTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            todo!() 
        }

        pub(crate) fn module<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            do_macros!(extractor,attrs);
            let uri = moduri!(MODULE);
            todo!()
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
                            || {
                                if v.contains('?') {
                                    tracing::warn!("Suspicious variable name containing '?': {v}");
                                }
                                VarOrSym::V(PreVar::Unresolved(v.into()))
                            },
                            |m| VarOrSym::S(m.into())),
                        |s| VarOrSym::S(s.into())
                    )
                }
            };
            attrs.set(tagstrings::HEAD,&head.to_string());
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
                    => OpenTerm::OML { name, tp: None, df: None },
                (OpenTermKind::OMA,head) 
                    => OpenTerm::OMA { head, args: SmallVec::new(), notation, head_term:None },
                (OpenTermKind::Complex,head)
                    => OpenTerm::Complex(head, None),
                (k,head) => {
                    extractor.add_error(SHTMLError::InvalidHeadForTermKind(k,head.clone()));
                    OpenTerm::OMA { head, args: SmallVec::new(), notation, head_term:None }
                }
            };
            let is_top = if extractor.in_term() { false } else {
                extractor.set_in_term(true);
                true
            };
            Some(OpenSHTMLElement::Term{term, is_top})
        }

        pub(crate) fn inputref<E:SHTMLExtractor>(extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            do_macros!(extractor,attrs);
            let uri = docuri!(INPUT_REF);
            let id = id!();
            tracing::trace!("inputref: {}",uri);
            Some(OpenSHTMLElement::Inputref { uri, id })
        }

        pub(crate) fn ifinputref<E:SHTMLExtractor>(_extractor:&mut E,attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            let value : bool = attrs.value(tagstrings::IF_INPUTREF).and_then(|s| s.as_ref().parse().ok()).unwrap_or_default(); 
            Some(OpenSHTMLElement::IfInputref(value))
        }

        pub(crate) fn comp<E:SHTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            Some(OpenSHTMLElement::Comp)
        }

        pub(crate) fn maincomp<E:SHTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut SV<E>) -> Option<OpenSHTMLElement> {
            Some(OpenSHTMLElement::MainComp)
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

    }
}
