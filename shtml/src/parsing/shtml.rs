use html5ever::Attribute;
use crate::docs::{OpenTerm};
use crate::parsing::parser::{Content, HTMLParser, Narr, NodeWithSource, OpenElems};
use std::str::FromStr;
use immt_api::backend::archives::{Archive, Storage};
use immt_api::backend::Backend;
use immt_api::backend::manager::ArchiveManager;
use immt_api::core::content::{Arg, ArgSpec, ArgType, ArrayVec, AssocType, Constant, ContentElement, MathStructure, Module, Morphism, Notation, NotationRef, Term, TermOrList, VarNameOrURI, VarOrSym};
use immt_api::core::narration::{CognitiveDimension, DocumentElement, DocumentMathStructure, DocumentModule, DocumentMorphism, DocumentReference, Language, LogicalParagraph, NarrativeRef, Problem, Section, SectionLevel, StatementKind, Title};
use immt_api::core::ontology::rdf::ontologies::{dc, rdfs};
use immt_api::core::ontology::rdf::terms::{GraphName, NamedNode, Quad, Triple};
use immt_api::core::ulo;
use immt_api::core::uris::archives::{ArchiveId, ArchiveURI};
use immt_api::core::uris::base::BaseURI;
use immt_api::core::uris::documents::DocumentURI;
use immt_api::core::uris::modules::ModuleURI;
use immt_api::core::uris::symbols::SymbolURI;
use immt_api::core::utils::sourcerefs::{ByteOffset, SourceRange};
use immt_api::core::utils::VecMap;
use immt_api::core::uris::{ContentURI, Name, NarrativeURI, NarrDeclURI};

macro_rules! iterate {
    ($n:ident $(($($i:ident:$t:ty=$d:expr),+))? $(-> $rest:ident)?, $e:ident => $f:expr;$or:expr) => {
        $($( let $i : $t = $d; )*)?
        $(
            for $e in $rest.iter_mut() { $f }
        )?
        fn iter($n:&NodeWithSource$(,$($i:$t),*)?) -> bool {
            iterate!(@I $n,
                $e => $f;
                p => iter(p$(,$($i),*)?);
                {$or;false}
            )
        }
        iter($n$(,$($i),*)?)
    };
    (@I $n:expr,$e:ident => $f:expr;$p:ident => $cont:expr;$or:expr) => {
        if let Some($p) = &$n.data.borrow().parent {
            let mut data = $p.data.borrow_mut();
            for $e in data.elem.iter_mut().rev() { $f }
            drop(data); return $cont
        } else {
            $or
        }
    };
}

impl<'a> HTMLParser<'a> {
    pub(crate) fn do_shtml(&mut self, attrs: &mut Vec<Attribute>) -> OpenElems {
        let mut ret = ArrayVec::new();
        attrs.sort_by_key(|a|
            SHTMLTag::from_str(a.name.local.as_ref()).ok().map(|a| a.weight()).unwrap_or(
                if a.name.local.as_ref().starts_with("shtml:") {254} else {255}
            )
        );
        self.parse_shtml(attrs, &mut ret);
        ret
    }

    #[inline]
    pub(crate) fn uri(&self) -> NarrativeURI {
        self.narratives.last().unwrap().uri
    }
    #[inline]
    fn content_uri(&self) -> ModuleURI {
        self.contents.last().unwrap_or_else(|| todo!("wut")).uri
    }
    #[inline]
    pub(crate) fn iri(&self) -> NamedNode {
        self.narratives.last().unwrap().iri.clone()
    }
    #[inline]
    fn content_iri(&self) -> NamedNode {
        self.contents.last().unwrap_or_else(|| todo!("wut") ).iri.clone()
    }
    #[inline]
    fn add_doc(&mut self,e:DocumentElement) {
        self.narratives.last_mut().unwrap().children.push(e)
    }
    #[inline]
    fn add_content(&mut self,e:ContentElement) {
        if let Some(p) = self.contents.last_mut() {
            p.children.push(e)
        }
    }

    #[inline]
    fn add_variable(&mut self,uri:NarrDeclURI,is_seq:bool) {
        self.narratives.last_mut().unwrap().vars.push((uri,is_seq))
    }

    #[inline]
    pub(crate) fn resolve_variable(&self,name:Name) -> VarNameOrURI {
        match self.narratives.iter().rev().flat_map(|n| n.vars.iter().rev().map(|(uri,_)| uri))
            .find(|uri| uri.name() == name) {
                Some(v) => VarNameOrURI::URI(*v),
                _ => VarNameOrURI::Name(name)
        }
    }

    #[inline]
    pub(crate) fn add_triple(&mut self, t:Triple) {
        let q = Quad {
            subject: t.subject.into(),
            predicate: t.predicate.into(),
            object: t.object.into(),
            graph_name: GraphName::NamedNode(self.narratives.first().unwrap().iri.clone())
        };
        self.triples.push(q)
    }
}


macro_rules! tags {
    (@open $i:ident $parser:ident +) => { $i += 1 };
    (@open $i:ident $parser:ident PAR:$k:ident) => { {
        let fors = get!(!"shtml:fors",s =>
            s.split(",").map(|s| {
                if let Some(uri) = get_sym_uri(s.trim(),$parser.backend,$parser.uri().language()) {uri} else {
                    todo!()
                }
            }).collect()
        ).unwrap_or_default();
        let inline = get!(!"shtml:inline",c => c.eq_ignore_ascii_case("true")).unwrap_or(false);
        let styles:Vec<String> = get!(!"shtml:styles",s => s.split(',').map(|s| s.trim().to_string()).collect()).unwrap_or_default();
        let id = get!(ID);
        let uri = $parser.uri() / id;
        $parser.narratives.push(Narr::new(uri.into()));
        add!(OpenElem::LogicalParagraph {
            styles,inline,fors,title:None,kind:StatementKind::$k,
            terms:VecMap::new()
        })
    } };
    (@open $i:ident $parser:ident $($open:tt)*) => { {$($open)*} };
    (@close !) => { true };
    (@close $($close:tt)*) => { {$($close)*} };
    ($v:ident,$node:ident,$slf:ident,$attrs:ident,$i:ident,$rest:ident,
        $(
            $tag:ident$(($($n:ident:$t:ty),+))?
            = $shtml:literal
            : $weight:literal
            //$(,cont=$cont:ident)?
            //$(,narr=$narr:ident)?
            {$($open:tt)*}
            $(, then {$($on_open:tt)*})?
            => {$($close:tt)*}
        ;)*
    ) => {
        #[derive(Debug,Copy,Clone)]
        enum SHTMLTag {
            $($tag),*
        }
        impl SHTMLTag {
            fn weight(&self) -> u8 {
                match self {
                    $(SHTMLTag::$tag => $weight),*
                }
            }
        }
        impl FromStr for SHTMLTag {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($shtml => Ok(SHTMLTag::$tag)),*,
                    _ => Err(())
                }
            }
        }
        impl<'a> HTMLParser<'a> {
            #[allow(unused_variables)]
            fn parse_shtml(&mut self,$attrs:&mut Vec<Attribute>,ret:&mut OpenElems) {
                let mut $i = 0;
                let $slf = self;
                macro_rules! add {
                    (@common $e:expr) => {
                        {
                            let r = $e;
                            //println!(" - Adding {:?}",r);
                            r
                        }
                    };
                    (-$e:expr) => {{
                        let r = add!(@common $e);
                        if $slf.strip {$attrs.remove($i);} else {$i += 1};
                        ret.push(r);
                    }};
                    ($e:expr) => {{
                        let r = add!(@common $e);
                        $i += 1;
                        ret.push(r)
                    }}
                }
                macro_rules! get {
                    (ID) => {
                        if let Some(id) = get!("shtml:id",s => {
                            let s = s.rsplit_once('?').map(|(_,s)| s).unwrap_or(s);
                            Name::new(s)
                        }) {id} else {
                            let id = Name::new(&format!("ID_{}", $slf.id_counter));
                            $slf.id_counter += 1;
                            id
                        }
                    };
                    (!$key:literal,$name:ident => $f:expr) => {
                        $attrs.iter().find(|a| a.name.local.as_ref() == $key).map(|s| {
                            let $name = s.value.as_ref();
                            $f
                        })
                    };
                    ($key:literal,$name:ident => $f:expr) => {
                        if $slf.strip {
                            if let Some((i,_)) = $attrs.iter().enumerate().find(|(i,a)| a.name.local.as_ref() == $key) {
                                let _n = $attrs.remove(i);
                                let $name = _n.value.as_ref();
                                Some($f)
                            } else { None }
                        } else {
                            get!(!$key,$name => $f)
                        }
                    }
                }
                /*for a in $attrs.iter() {
                    println!("HERE: {a:?}");
                }
                print!("");*/
                while let Some(a) =  $attrs.get($i) {
                    let k = if let Ok(k) = SHTMLTag::from_str(a.name.local.as_ref()) {k} else {
                        if a.name.local.starts_with("shtml:") {
                            //println!("Here: {} = {}",a.name.local,a.value);
                            todo!("Here: {} = {}",a.name.local,a.value);
                        }
                        break;
                    };
                    let $v = a.value.as_ref();
                    //println!("Here: {k:?} = {}",$v);
                    match k {
                        $( SHTMLTag::$tag => tags!(@open $i $slf $($open)* ) ),*
                    }
                }
            }
            /// returns whether to keep the node (or delete it)
            #[allow(unused_variables)]
            pub(crate) fn close(&mut self,$node:&NodeWithSource,elem:OpenElem, $rest:&mut OpenElems) -> bool {
                let $slf = self;
                //println!(" - Closing {elem:?}");
                match elem {
                    OpenElem::LogicalParagraph {inline,kind,fors,styles,title,terms} => {
                        let Some(Narr {children,uri:NarrativeURI::Decl(uri),..}) = $slf.narratives.pop() else {unreachable!()};

                        let iri = uri.to_iri();
                        if kind.is_definition_like(&styles) {
                            for s in fors.iter() {
                                $slf.add_triple(ulo!((iri.clone()) DEFINES (s.to_iri())));
                            }
                        } else if kind == StatementKind::Example {
                            for s in fors.iter() {
                                $slf.add_triple(ulo!((iri.clone()) EXAMPLE_FOR (s.to_iri())));
                            }
                        }
                        $slf.add_triple(ulo!(($slf.iri()) CONTAINS (iri.clone())));
                        let tp = kind.rdf_type();
                        $slf.add_triple(ulo!((iri) : (tp.into_owned())));
                        $slf.add_doc(DocumentElement::Paragraph(LogicalParagraph {
                            uri,styles,inline,fors:fors.into_iter().map(|u| u.into()).collect(),children,title,kind,
                            range: $node.data.borrow().range,terms
                        }));
                        true
                    }
                    OpenElem::VarNotation {name,id,precedence,argprecs,comp,op} => {
                        $slf.in_notation = false;
                        if let Some(n) = comp {
                            let nt = n.as_notation(id,op,precedence,argprecs);
                            let nt = $slf.store_resource(&nt);
                            $slf.add_doc(DocumentElement::VarNotation {
                                name,id,notation:nt
                            });
                        }
                        false
                    }
                    OpenElem::Symref { uri,notation} => {
                        $slf.add_triple(ulo!(($slf.iri()) CROSSREFS (uri.to_iri())));
                        $slf.add_doc(DocumentElement::Symref {uri,notation,range:$node.data.borrow().range});
                        true
                    }
                    OpenElem::Varref { name,notation} => {
                        let name = $slf.resolve_variable(name);
                        $slf.add_doc(DocumentElement::Varref {name,notation,range:$node.data.borrow().range});
                        true
                    }
                    OpenElem::TopLevelTerm(t) => {
                        $slf.in_term = false;
                        let t = t.close($slf);
                        $slf.add_doc(DocumentElement::TopTerm(t));
                        true
                    }
                    OpenElem::NotationArg { arg, mode } => {
                        $node.data.borrow_mut().elem.push(OpenElem::NotationArg { arg, mode });true
                    }
                    $(OpenElem::$tag$({$($n),+})? => tags!(@close $($close)* ) ),*
                }
            }
        }
        #[derive(Debug)]
        pub(crate) enum OpenElem {
            TopLevelTerm(OpenTerm),
            Symref {
                uri:ContentURI,
                notation:Option<Name>,
            },
            Varref {
                name:Name,
                notation:Option<Name>,
            },
            VarNotation {
                name:VarNameOrURI,
                id:Name,
                precedence:isize,
                argprecs:ArrayVec<isize,9>,
                comp:Option<NodeWithSource>,
                op:Option<NodeWithSource>
            },
            NotationArg { arg:Arg, mode:ArgType },
            LogicalParagraph {
                inline:bool,
                kind: StatementKind,
                fors:Vec<SymbolURI>,
                styles:Vec<String>,
                title: Option<Title>,
                terms:VecMap<SymbolURI,Term>
            },
            $(
                $tag$({$($n:$t),+})?
            ),*
        }
        impl OpenElem {
            /*
            #[allow(unused_variables)]
            fn content(&mut self) -> Option<&mut Vec<ContentElement>> {
                match self {$(
                    OpenElem::$tag$({$($n),+})? => (
                        $( return Some($cont) )?
                    ),
                    )*
                    _ => ()
                }
                None
            }
            #[allow(unused_variables)]
            fn narration(&mut self) -> Option<&mut Vec<DocumentElement>> {
                match self {
                    Self::LogicalParagraph{children,..} => return Some(children),
                    $(
                    OpenElem::$tag$({$($n),+})? => (
                        $( return Some($narr) )?
                    ),
                    )*
                    _ => ()
                }
                None
            }

             */
            #[allow(unused_variables)]
            pub(crate) fn on_add(&mut self,$slf:&mut HTMLParser) -> bool {
                let $node = self;
                match $node {
                    OpenElem::TopLevelTerm(..) => {
                        $slf.in_term = true;
                    }
                    OpenElem::VarNotation{..} => {
                        $slf.in_notation = true;
                    }
                    $( OpenElem::$tag$({$($n),+})? => {$( return {$($on_open)*} )?;} )*
                    _ => ()
                }
                true
            }
        }
    }
}

tags!{v,node,parser,attrs,i,rest,
    Module(
        meta:Option<ModuleURI>,
        language:Language,
        signature:Option<Language>
    ) = "shtml:theory" : 0 {
        let uri = if let Some(uri) = get_mod_uri(v,parser.backend,parser.uri().language()) {uri} else {
            todo!("HERE: {v}");
        };
        let language = get!("shtml:language",s =>
            if s.is_empty() {parser.narratives.last().unwrap().uri.language()} else {if let Ok(l) = s.try_into() {l} else {
                todo!("HERE: {s}")
            }}
        ).unwrap_or(parser.narratives.last().unwrap().uri.language());
        let meta = get!("shtml:metatheory",s =>
            if let Some(m) = get_mod_uri(s,parser.backend,language) {m} else {
                todo!("HERE: {s}");
            });
        let signature: Option<Language> = get!("shtml:signature",s =>
            if s.is_empty() {None} else {if let Ok(l) = s.try_into() {Some(l)} else {
                todo!("HERE: {s}")
            }}
        ).flatten();
        parser.narratives.push(Narr::new((parser.uri() / uri.name()).into()));
        parser.contents.push(Content::new(uri));
        add!(-OpenElem::Module {
            meta,language,signature
        })
    } => {
        let Some(Narr {children:narrative_children,uri:NarrativeURI::Decl(narr_uri),..}) = parser.narratives.pop() else {unreachable!()};
        let Some(Content {children:content_children,uri:module_uri,iri}) = parser.contents.pop() else {unreachable!()};
        parser.add_triple(
            ulo!((parser.iri()) CONTAINS (iri.clone()))
        );
        parser.add_triple(
            ulo!((iri) : THEORY)
        );
        let dm = DocumentElement::Module(DocumentModule {
            uri:narr_uri,module_uri,
            range: node.data.borrow().range,
            children: narrative_children,
        });
        parser.add_doc(dm);
        let m = Module {
            uri:module_uri, meta, signature, elements: content_children,
        };
        if let Some(cm) = parser.contents.last_mut() {
            cm.children.push(ContentElement::NestedModule(m))
        } else {
            parser.modules.push(m)
        }
        true
    };

    MathStructure(
        macroname:Option<String>
    ) = "shtml:feature-structure" : 0 {
        let uri = if let Some(uri) = get_mod_uri(v,parser.backend,parser.uri().language()) {uri} else {
            todo!("HERE: {v}");
        };
        //println!("Here structure: {uri}");
        let macroname = get!("shtml:macroname",s => if s.is_empty() {None} else {Some(s.to_string())}).flatten();
        let name = uri.name();
        let name = Name::new(name.as_ref().rsplit_once('/').map(|(_,b)| b).unwrap_or(name.as_ref()));
        parser.narratives.push(Narr::new((parser.uri() / name).into()));
        parser.contents.push(Content::new(uri));
        add!(-OpenElem::MathStructure {
            macroname
        })
    } => {
        let Some(Narr {children:narrative_children,uri:NarrativeURI::Decl(narr_uri),..}) = parser.narratives.pop() else {unreachable!()};
        let Some(Content {children:content_children,uri:module_uri,iri}) = parser.contents.pop() else {unreachable!()};
        parser.add_triple(
            ulo!((parser.iri()) CONTAINS (iri.clone()))
        );
        parser.add_triple(
            ulo!((iri) : STRUCTURE)
        );
        let dm = DocumentElement::MathStructure(DocumentMathStructure {
            uri:narr_uri,module_uri,
            range: node.data.borrow().range,
            children: narrative_children,
        });
        parser.add_doc(dm);
        let m = MathStructure {
            uri:module_uri, elements: content_children,macroname
        };
        if let Some(cm) = parser.contents.last_mut() {
            cm.children.push(ContentElement::MathStructure(m))
        } else {
            todo!()
        }
        true
    };

    Morphism(domain:ModuleURI,total:bool) = "shtml:feature-morphism": 250 {
        let uri = if let Some(uri) = get_mod_uri(v,parser.backend,parser.uri().language()) {uri} else {
            todo!("HERE: {v}");
        };
        let domain = get!("shtml:domain",s => if let Some(d) = get_mod_uri(s,parser.backend,parser.uri().language()) {d} else {
            todo!("HERE: {s}");
        }).unwrap_or_else(|| todo!("HERE: Domain missing"));
        let total = get!("shtml:total",s => s.eq_ignore_ascii_case("true")).unwrap_or(false);
        let name = uri.name();
        let name = Name::new(name.as_ref().rsplit_once('/').map(|(_,b)| b).unwrap_or(name.as_ref()));
        parser.narratives.push(Narr::new((parser.uri() / name).into()));
        parser.contents.push(Content::new(uri));
        add!(-OpenElem::Morphism{domain,total})
    } => {
        let Some(Narr {children:narrative_children,uri:NarrativeURI::Decl(narr_uri),..}) = parser.narratives.pop() else {unreachable!()};
        let Some(Content {children:content_children,uri:module_uri,iri}) = parser.contents.pop() else {unreachable!()};
        parser.add_triple(
            ulo!((parser.iri()) CONTAINS (iri.clone()))
        );
        parser.add_triple(ulo!((iri.clone()) : MORPHISM));
        parser.add_triple(ulo!((iri) !(rdfs::DOMAIN) (domain.to_iri()) ));
        let dm = DocumentElement::Morphism(DocumentMorphism {
            uri:narr_uri,content_uri:module_uri,
            range: node.data.borrow().range,
            children: narrative_children,total,domain
        });
        parser.add_doc(dm);
        let c = Morphism {
            uri:module_uri, domain, total, elements: content_children
        };
        parser.add_content(ContentElement::Morphism(c));
        true
    };
    MorphismDomain = "shtml:domain": 250 {+} => {!};
    MorphismTotal = "shtml:total": 250 {+} => {!};
    Rename = "shtml:rename": 250 {+} => {!};
    RenameTo = "shtml:to": 250 {+} => {!};
    AssignMorphismFrom = "shtml:assignmorphismfrom": 250 {+} => {!};
    AssignMorphismTo = "shtml:assignmorphismto": 250 {+} => {!};
    Assign(tm:Option<Term>) = "shtml:assign": 250 {
        add!(- OpenElem::Assign {tm:None})
    } => {!};

    Section(
        level:SectionLevel,
        title:Option<Title>
    ) = "shtml:section" : 10 {
        if let Some(level) = u8::from_str(v).ok().map(|u| u.try_into().ok()).flatten() {
            let id = get!(ID);
            parser.narratives.push(Narr::new((parser.uri() / id).into()));
            add!(-OpenElem::Section {
                level,title:None
            })
        } else {
            todo!()
        }
    } => {
        let Some(Narr {children,uri:NarrativeURI::Decl(uri),iri,..}) = parser.narratives.pop() else {unreachable!()};
        parser.add_triple(
            ulo!((parser.iri()) CONTAINS (iri.clone()))
        );
        parser.add_triple(ulo!((iri) : SECTION));
        parser.add_doc(DocumentElement::Section(Section {
            level,title,children,range:node.data.borrow().range,uri
        }));
        true
    };
    Definition = "shtml:definition": 10 {PAR: Definition} => {!};
    Paragraph = "shtml:paragraph": 10 {PAR: Paragraph} => {!};
    Assertion = "shtml:assertion": 10 {PAR: Assertion} => {!};
    Example = "shtml:example": 10 {PAR: Example} => {!};
    Problem(
        autogradable:bool,
        points:Option<f32>,
        title:Option<Title>,
        solutions:Vec<NarrativeRef<String>>,
        hints:Vec<NarrativeRef<String>>,
        notes:Vec<NarrativeRef<String>>,
        gnotes:Vec<NarrativeRef<String>>,
        preconditions:Vec<(CognitiveDimension,SymbolURI)>,
        objectives:Vec<(CognitiveDimension,SymbolURI)>
    ) = "shtml:problem": 10 {
        let autogradable = get!(!"shtml:autogradable",c => c.eq_ignore_ascii_case("true")).unwrap_or(false);
        let _ = get!("shtml:language",s => ());
        let points = get!("shtml:problempoints",s => s.parse().ok()).flatten();
        let id = get!(ID);
        parser.narratives.push(Narr::new((parser.uri() / id).into()));
        add!(OpenElem::Problem {
            autogradable,points,solutions:Vec::new(),hints:Vec::new(),
            notes:Vec::new(),gnotes:Vec::new(),title:None,
            preconditions:Vec::new(),objectives:Vec::new()
        })
    } => {
        let Some(Narr {children,uri:NarrativeURI::Decl(uri),iri,..}) = parser.narratives.pop() else {unreachable!()};
        parser.add_triple(ulo!((parser.iri()) CONTAINS (iri.clone())));
        parser.add_triple(ulo!((iri.clone()) : PROBLEM));
        for (d,s) in &preconditions {
            let n = immt_api::core::ontology::rdf::terms::BlankNode::default();
            parser.add_triple(
                ulo!((iri.clone()) PRECONDITION >>(immt_api::core::ontology::rdf::terms::Term::BlankNode(n.clone())))
            );
            parser.add_triple(
                ulo!(>>(immt_api::core::ontology::rdf::terms::Subject::BlankNode(n.clone())) COGDIM (d.to_iri().into_owned()))
            );
            parser.add_triple(
                ulo!(>>(immt_api::core::ontology::rdf::terms::Subject::BlankNode(n.clone())) POSYMBOL (s.to_iri()))
            );
        }
        for (d,s) in &objectives {
            let n = immt_api::core::ontology::rdf::terms::BlankNode::default();
            parser.add_triple(
                ulo!((iri.clone()) OBJECTIVE >>(immt_api::core::ontology::rdf::terms::Term::BlankNode(n.clone())))
            );
            parser.add_triple(
                ulo!(>>(immt_api::core::ontology::rdf::terms::Subject::BlankNode(n.clone())) COGDIM (d.to_iri().into_owned()))
            );
            parser.add_triple(
                ulo!(>>(immt_api::core::ontology::rdf::terms::Subject::BlankNode(n.clone())) POSYMBOL (s.to_iri()))
            );
        }
        parser.add_doc(DocumentElement::Problem(Problem {
            uri,autogradable,points,solutions,hints,notes,gnotes,title,children,sub:false,
            preconditions,objectives
        }));
        true
    };
    SubProblem(
        autogradable:bool,
        points:Option<f32>,
        title:Option<Title>,
        solutions:Vec<NarrativeRef<String>>,
        hints:Vec<NarrativeRef<String>>,
        notes:Vec<NarrativeRef<String>>,
        gnotes:Vec<NarrativeRef<String>>,
        preconditions:Vec<(CognitiveDimension,SymbolURI)>,
        objectives:Vec<(CognitiveDimension,SymbolURI)>
    ) = "shtml:subproblem": 10 {
        let autogradable = get!(!"shtml:autogradable",c => c.eq_ignore_ascii_case("true")).unwrap_or(false);
        let _ = get!("shtml:language",s => ());
        let points = get!("shtml:problempoints",s => s.parse().ok()).flatten();
        let id = get!(ID);
        parser.narratives.push(Narr::new((parser.uri() / id).into()));
        add!(OpenElem::Problem {
            autogradable,points,solutions:Vec::new(),hints:Vec::new(),
            notes:Vec::new(),gnotes:Vec::new(),title:None,
            preconditions:Vec::new(),objectives:Vec::new()
        })
    } => {
        let Some(Narr {children,uri:NarrativeURI::Decl(uri),iri,..}) = parser.narratives.pop() else {unreachable!()};
        parser.add_triple(ulo!((parser.iri()) CONTAINS (iri.clone())));
        parser.add_triple(ulo!((iri.clone()) : SUBPROBLEM));
        for (d,s) in &preconditions {
            let n = immt_api::core::ontology::rdf::terms::BlankNode::default();
            parser.add_triple(
                ulo!((iri.clone()) PRECONDITION >>(immt_api::core::ontology::rdf::terms::Term::BlankNode(n.clone())))
            );
            parser.add_triple(
                ulo!(>>(immt_api::core::ontology::rdf::terms::Subject::BlankNode(n.clone())) COGDIM (d.to_iri().into_owned()))
            );
            parser.add_triple(
                ulo!(>>(immt_api::core::ontology::rdf::terms::Subject::BlankNode(n.clone())) POSYMBOL (s.to_iri()))
            );
        }
        for (d,s) in &objectives {
            let n = immt_api::core::ontology::rdf::terms::BlankNode::default();
            parser.add_triple(
                ulo!((iri.clone()) OBJECTIVE >>(immt_api::core::ontology::rdf::terms::Term::BlankNode(n.clone())))
            );
            parser.add_triple(
                ulo!(>>(immt_api::core::ontology::rdf::terms::Subject::BlankNode(n.clone())) COGDIM (d.to_iri().into_owned()))
            );
            parser.add_triple(
                ulo!(>>(immt_api::core::ontology::rdf::terms::Subject::BlankNode(n.clone())) POSYMBOL (s.to_iri()))
            );
        }
        parser.add_doc(DocumentElement::Problem(Problem {
            uri,autogradable,points,solutions,hints,notes,gnotes,title,children,sub:true,
            preconditions,objectives
        }));
        true
    };

    Doctitle = "shtml:doctitle" : 20 {add!(-OpenElem::Doctitle)} => {
        parser.title = Some(Title {
            range:node.data.borrow().range
        });
        false
    };
    SectionTitle = "shtml:sectiontitle": 20 {
        add!(-OpenElem::SectionTitle)
    } => {
        let title = Title {
            range:node.data.borrow().range
        };
        iterate!(node(ttl:Title=title,range:SourceRange<ByteOffset>=node.data.borrow().range),
            e => if let OpenElem::Section { title, .. } = e {
                *title = Some(ttl);return true
            };
            todo!()
        );
        true
    };
    StatementTitle = "shtml:statementtitle": 20 {
        add!(-OpenElem::StatementTitle)
    } => {
        let title = Title {
            range:node.data.borrow().range
        };
        iterate!(node(ttl:Title=title,range:SourceRange<ByteOffset>=node.data.borrow().range),
            e => if let OpenElem::LogicalParagraph { title, .. } = e {
                *title = Some(ttl); return true
            };
            todo!()
        );
        true
    };
    ProofTitle = "shtml:prooftitle": 20 {
        add!(-OpenElem::ProofTitle)
    } => {
        let title = Title {
            range:node.data.borrow().range
        };
        iterate!(node(ttl:Title=title,range:SourceRange<ByteOffset>=node.data.borrow().range),
            e => if let OpenElem::LogicalParagraph { title,kind:StatementKind::Proof, .. } = e {
                *title = Some(ttl); return true
            };
            todo!()
        );
        true
    };
    ProblemTitle = "shtml:problemtitle": 20 {
        add!(-OpenElem::ProblemTitle)
    } => {
        let title = Title {
            range:node.data.borrow().range
        };
        iterate!(node(ttl:Title=title,range:SourceRange<ByteOffset>=node.data.borrow().range),
            e => if let OpenElem::Problem { title, .. } | OpenElem::SubProblem {title,..} = e {
                *title = Some(ttl); return true
            };
            todo!()
        );
        true
    };

    Symdecl(uri:SymbolURI,
        arity:ArgSpec,
        macroname:Option<String>,
        role:Option<Vec<String>>,
        tp:Option<Term>,
        df:Option<Term>,
        assoctype : Option<AssocType>,
        reordering: Option<String>
    ) = "shtml:symdecl":30 {
        let uri = if let Some(uri) = get_sym_uri(v,parser.backend,parser.uri().language()) {uri} else {
            todo!();
        };
        //println!("Here symbol: {uri}");
        let role = get!("shtml:role",s => s.split(',').map(|s| s.trim().to_string()).collect());
        let assoctype = get!("shtml:assoctype",s => s.trim().parse().ok()).flatten();
        let arity = get!("shtml:args",s =>
            if let Ok(a) = s.parse() { a } else { todo!("args {s}")}).unwrap_or_default();
        let reordering = get!("shtml:reoderargs",s => s.to_string());
        let macroname = get!("shtml:macroname",s => if s.is_empty() {None} else {Some(s.to_string())}).flatten();

        add!(- OpenElem::Symdecl { uri, arity, macroname, role,tp:None,df:None, assoctype, reordering });
    } => {
        let iri = uri.to_iri();
        parser.add_triple(
            ulo!((parser.content_iri()) DECLARES (iri.clone()))
        );
        parser.add_triple(
            ulo!((iri) : DECLARATION)
        );
        parser.add_content(ContentElement::Constant(Constant {
            uri:uri.clone(),arity,macroname,role,tp,df,assoctype,reordering
        }));
        parser.add_doc(DocumentElement::ConstantDecl(uri));
        false
    };

    VarDef(uri:NarrDeclURI,
        arity:ArgSpec,
        macroname:Option<String>,
        role:Option<Vec<String>>,
        tp:Option<Term>,
        df:Option<Term>,
        is_sequence:bool,
        assoctype : Option<AssocType>,
        reordering: Option<String>,
        bind:bool
    ) = "shtml:vardef":30 {
        let name = Name::new(v);
        let uri = parser.uri() / name;
        let role = get!("shtml:role",s => s.split(',').map(|s| s.trim().to_string()).collect());
        let arity = get!("shtml:args",s =>
            if let Ok(a) = s.parse() { a } else { todo!()}).unwrap_or_default();
        let assoctype = get!("shtml:assoctype",s => s.trim().parse().ok()).flatten();
        let reordering = get!("shtml:reoderargs",s => s.to_string());
        let bind = get!("shtml:bind",s => s.eq_ignore_ascii_case("true")).unwrap_or(false);
        let macroname = get!("shtml:macroname",s => if s.is_empty() {None} else {Some(s.to_string())}).flatten();
        add!(-OpenElem::VarDef { uri, arity, macroname,role,tp:None,df:None,is_sequence:false,assoctype,reordering,bind });
    }, then {
        parser.add_variable(*uri,*is_sequence);true
    } => {
        parser.add_doc(DocumentElement::VarDef {
            uri,arity,macroname,range:node.data.borrow().range,role,tp,df,is_sequence,
            assoctype,reordering,bind
        });
        false
    };
    VarSeq = "shtml:varseq":30 {
        let name = Name::new(v);
        let uri = parser.uri() / name;
        let role = get!("shtml:role",s => s.split(',').map(|s| s.trim().to_string()).collect());
        let arity = get!("shtml:args",s =>
            if let Ok(a) = s.parse() { a } else { todo!()}).unwrap_or_default();
        let assoctype = get!("shtml:assoctype",s => s.trim().parse().ok()).flatten();
        let reordering = get!("shtml:reoderargs",s => s.to_string());
        let bind = get!("shtml:bind",s => s.eq_ignore_ascii_case("true")).unwrap_or(false);
        let macroname = get!("shtml:macroname",s => if s.is_empty() {None} else {Some(s.to_string())}).flatten();
        add!(-OpenElem::VarDef { uri, arity, macroname,role,tp:None,df:None,is_sequence:true,assoctype,reordering,bind });
    } => { unreachable!()};

    Notation(symbol:SymbolURI,
        uri:SymbolURI,
        precedence:isize,
        argprecs:ArrayVec<isize,9>,
        comp:Option<NodeWithSource>,
        op:Option<NodeWithSource>
    ) = "shtml:notation":30 {
        let symbol = if let Some(uri) = get_sym_uri(v,parser.backend,parser.uri().language()) {Ok(uri.into())}
        else if !v.contains('?') {
            Err(Name::new(v))
        } else {
            //println!("Wut: {v}");
            todo!("Wut: {v}");
        };
        let fragment = get!("shtml:notationfragment",s =>
            if s.is_empty() {None} else {Some(Name::new(s))}
        ).flatten();
        let prec = get!("shtml:precedence",s => s.parse().ok()).flatten();
        let argprecs: ArrayVec<_,9> = get!("shtml:argprecs",s =>
            if s.is_empty() {Default::default()} else {s.split(',').map(|s| s.trim().parse().unwrap_or(0)).collect()}
        ).unwrap_or_default();
        let id = fragment.unwrap_or_else(|| {
            let r = Name::new(&format!("ID_{}",parser.id_counter));
            parser.id_counter += 1;
            r
        });
        add!(- match symbol {
            Ok(symbol) => {
                //println!("Here notation: {symbol}");
                OpenElem::Notation {
                    symbol, uri:parser.content_uri() & id, precedence:prec.unwrap_or(0), argprecs,comp:None,op:None
                }
            }
            Err(name) => {
                let name = parser.resolve_variable(name);
                OpenElem::VarNotation {
                    name, id, precedence:prec.unwrap_or(0), argprecs,comp:None,op:None
                }
            }
        })
    }, then { parser.in_notation = true;true } => {
        parser.in_notation = false;
        let iri = uri.to_iri();
        parser.add_triple(
            ulo!((parser.content_iri()) DECLARES (iri.clone()))
        );
        parser.add_triple(
            ulo!((iri.clone()) NOTATION_FOR (symbol.to_iri()))
        );
        parser.add_triple(
            ulo!((iri) : NOTATION)
        );
        if let Some(n) = comp {
            let nt = n.as_notation(uri.name(),op,precedence,argprecs);
            let nt = parser.store_resource(&nt);
            parser.add_content(ContentElement::Notation(NotationRef {
                symbol,uri,range:nt
            }));
        }
        false
    };

    NotationComp = "shtml:notationcomp": 60 {
        let _ = get!("shtml:term",_e => ());
        let _ = get!("shtml:head",_e => ());
        let _ = get!("shtml:notationid",_e => ());
        let _ = get!("shtml:visible",_e => ());
        add!(-OpenElem::NotationComp)
    } => {
        fn get_node(node:&NodeWithSource) -> NodeWithSource {
            match node.node.as_element().map(|e| e.name.local.as_ref()) {
                Some(p) if p == "math" && node.data.borrow().children.len() ==1 =>
                    get_node(&node.data.borrow().children[0]),
                _ => node.clone()
            }
        }
        let not = get_node(node);
        iterate!(node(n:NodeWithSource=not,parser:&HTMLParser=parser) -> rest,
            e => if let OpenElem::Notation{comp,..}|OpenElem::VarNotation{comp,..} = e {
            *comp = Some(n);return true
        };
            println!("TODO: Not in notation...? ({})",parser.uri())
        );
        true
    };
    NotationOpComp = "shtml:notationopcomp": 60 {
        let _ = get!("shtml:term",_e => ());
        let _ = get!("shtml:head",_e => ());
        let _ = get!("shtml:notationid",_e => ());
        let _ = get!("shtml:visible",_e => ());
        add!(-OpenElem::NotationOpComp)
    } => {
        fn get_node(node:&NodeWithSource) -> NodeWithSource {
            match node.node.as_element().map(|e| e.name.local.as_ref()) {
                Some(p) if (p == "math"|| p == "mrow") && node.data.borrow().children.len() ==1 =>
                    get_node(&node.data.borrow().children[0]),
                _ => node.clone()
            }
        }
        let not = get_node(node);
        iterate!(node(n:NodeWithSource=not) -> rest,
            e => if let OpenElem::Notation{op,..}|OpenElem::VarNotation{op,..} = e {
            *op = Some(n);return true
        };
            todo!("Not in notation...?")
        );
        true
    };

    Definiendum(uri:SymbolURI) = "shtml:definiendum": 40 {
        if let Some(uri) = get_sym_uri(v,parser.backend,parser.uri().language()) {
            attrs.get_mut(i).unwrap().value = uri.to_string().into();
            add!(OpenElem::Definiendum { uri })
        } else {
            i += 1
        }
    } => {
        iterate!(node(uri:&SymbolURI=&uri),
            e => if let OpenElem::LogicalParagraph { fors,kind,styles, .. } = e {
                if kind.is_definition_like(styles) {
                    if !fors.contains(uri) { fors.push(uri.clone()) }
                    return true
                }
            };
            todo!()
        );
        parser.add_doc(DocumentElement::Definiendum {uri:*uri, range:node.data.borrow().range });
        true
    };

    Type = "shtml:type": 50 {
        add!(- OpenElem::Type)
    }, then { parser.in_term = true;true } => {
        let t = node.as_term(Some(rest),parser);
        iterate!(node(t:Term=t,parser:&mut HTMLParser=parser) -> rest,
            e => if let OpenElem::Symdecl {tp,..} | OpenElem::VarDef {tp,..} = e {
                *tp = Some(t);parser.in_term = false;return true
            };
            todo!()
        );
        parser.in_term = false;
        true
    };

    Conclusion(uri:SymbolURI,in_term:bool) = "shtml:conclusion": 50 {
        let uri = if let Some(uri) = get_sym_uri(v,parser.backend,parser.uri().language()) {uri} else {
            todo!();
        };
        let it = parser.in_term;
        add!(- OpenElem::Conclusion{uri,in_term:it})
    }, then { parser.in_term = true;true } => {
        let t = node.as_term(Some(rest),parser);
        iterate!(node(uri:SymbolURI=uri,t:Term=t,parser:&mut HTMLParser=parser,in_term:bool=in_term) -> rest,
            e => if let OpenElem::LogicalParagraph {kind:StatementKind::Assertion,terms,..} = e {
                terms.insert(uri,t);parser.in_term = in_term;return true
            };
            todo!()
        );
        parser.in_term = in_term;
        true
    };
    Definiens(uri:Option<SymbolURI>,in_term:bool) = "shtml:definiens": 50 {
        let uri = if !v.is_empty() {if let Some(uri) = get_sym_uri(v,parser.backend,parser.uri().language()) {Some(uri)} else {
            todo!();
        }} else {None};
        let it = parser.in_term;
        add!(OpenElem::Definiens{uri,in_term:it})
    }, then { parser.in_term = true; true } => {
        let t = node.as_term(Some(rest),parser);
        iterate!(node(uri:Option<SymbolURI>=uri,t:Term=t,parser:&mut HTMLParser=parser,in_term:bool=in_term) -> rest,
            e => {
                if let OpenElem::LogicalParagraph {terms,..} = e {
                    if let Some(uri) = uri {terms.insert(uri,t);parser.in_term = in_term;return true}
                }
                if let OpenElem::Symdecl {df,..} | OpenElem::VarDef {df,..} = e {
                    *df = Some(t);parser.in_term = in_term;return true
                }
                if let OpenElem::Assign {tm} = e {
                    *tm = Some(t);parser.in_term = in_term;return true
                }
                if let OpenElem::Term{tm:OpenTerm::OML{df,..}}|OpenElem::TopLevelTerm(OpenTerm::OML{df,..}) = e{
                    *df = Some(t);parser.in_term = in_term;return true
                }
            };
            println!("TODO: Definiens is fishy ({})",parser.uri())
        );
        parser.in_term = in_term;
        true
    };
    Rule(id:String,args:ArrayVec<Option<(Term,ArgType)>,9>) = "shtml:rule": 50 { // TODO
        let id = v.to_string();
        add!(- OpenElem::Rule {
            id,args:ArrayVec::new()
        })
    }, then { parser.in_term = true;true } => {parser.in_term = false;false};

    ArgSep = "shtml:argsep": 60 {
        let _ = get!("shtml:term",_e => ());
        let _ = get!("shtml:head",_e => ());
        let _ = get!("shtml:notationid",_e => ());
        let _ = get!("shtml:visible",_e => ());
        add!(-OpenElem::ArgSep)
    } => {
        node.data.borrow_mut().elem.push(OpenElem::ArgSep);
        true
    };
    ArgMap = "shtml:argmap": 60 {
        let _ = get!("shtml:term",_e => ());
        let _ = get!("shtml:head",_e => ());
        let _ = get!("shtml:notationid",_e => ());
        let _ = get!("shtml:visible",_e => ());
        add!(-OpenElem::ArgMap)
    } => {
        node.data.borrow_mut().elem.push(OpenElem::ArgMap);
        true
    };
    ArgMapSep = "shtml:argmap-sep": 60 {
        add!(-OpenElem::ArgMapSep)
    } => {
        node.data.borrow_mut().elem.push(OpenElem::ArgMapSep);
        true
    };

    Term(tm:OpenTerm) = "shtml:term": 100 {
            let notation = get!(!"shtml:notationid",s => if s.is_empty() {None} else {Some(s.to_string())}).flatten().map(Name::new);
            let head = get!(!"shtml:head",s =>
                if let Some(uri) = get_sym_uri(s,parser.backend,parser.uri().language()) {VarOrSym::S(uri.into())}
                else if let Some(uri) = get_mod_uri(s,parser.backend,parser.uri().language()) {VarOrSym::S(uri.into())}
                else if !s.contains('?') {VarOrSym::V(VarNameOrURI::Name(Name::new(s)))} else {
                    println!("Fishy: {s} ({})",parser.uri());
                    VarOrSym::V(VarNameOrURI::Name(Name::new("ERROR")))
                }
            ).unwrap_or_else(|| {
                todo!()
            });
            #[derive(PartialEq,Eq,Debug)]
            enum TMK {
                OMV,OMID,OMA,OMB,OML,Complex
            }
            let tk = match v {
                "OMID" => TMK::OMID,
                "OMV" => TMK::OMV,
                "OMA" => TMK::OMA,
                "OMBIND" => TMK::OMB,
                "OML" => TMK::OML,
                "complex" => TMK::Complex,
                "OMMOD" => TMK::OMID,
                _ => todo!("HERE: {v}")
            };

            attrs.iter_mut().find(|a| a.name.local.as_ref() == "shtml:head").unwrap()
                .value = head.to_string().into();
            let term = match (tk,head) {
                (TMK::OMID,VarOrSym::S(uri)) => OpenElem::Term{tm:OpenTerm::Symref {uri,notation}},
                (TMK::OMV|TMK::OMID,VarOrSym::V(name)) => OpenElem::Term{tm:OpenTerm::OMV {name,notation}},
                (TMK::OML,VarOrSym::V(VarNameOrURI::Name(name))) => OpenElem::Term{tm:OpenTerm::OML {name,df:None}},
                (TMK::OMA,head) => OpenElem::Term{tm:OpenTerm::OMA {head,notation,head_term:None,args:ArrayVec::new()}},
                (TMK::OMB,head) => OpenElem::Term{tm:OpenTerm::OMBIND {head,notation,head_term:None,args:ArrayVec::new()}},
                (TMK::Complex,head) => OpenElem::Term{tm:OpenTerm::Complex(head,None)},
                (t,h) => {
                    println!("TODO: Term is fishy: {t:?} {h} ({})",parser.uri());
                    OpenElem::Term{tm:OpenTerm::OMV{name:VarNameOrURI::Name(Name::new("TODO")),notation:None}}
                }
            };
            add!(term)
    },then{ if parser.in_notation {false} else {
        if !parser.in_term {
            match tm {
                OpenTerm::Symref {..} | OpenTerm::OMV {..} => (),
                _ => {
                    parser.in_term = true;
                    *node = OpenElem::TopLevelTerm(std::mem::replace(tm,OpenTerm::OMV {name:VarNameOrURI::Name(Name::new("")),notation:None}));
                }
            }
        }
        true
    } } => {
        node.data.borrow_mut().elem.push(OpenElem::Term{tm});true
    };

    Arg(arg:Arg, mode:ArgType) = "shtml:arg": 110 {
        let arg = get!(!"shtml:arg",s => s.parse().ok()).flatten().unwrap_or_else(|| {
            println!("{attrs:?}\n{parser:?} ({})",parser.uri());
            todo!("{attrs:?}")
        });
        let mode = get!(!"shtml:argmode",s => s.parse().ok()).flatten().unwrap_or_else(|| {
            println!("{attrs:?}\n{parser:?} ({})",parser.uri());
            todo!("{attrs:?}")
        });
        add!(OpenElem::Arg{arg,mode})
    }, then {
        if parser.in_notation {
            *node = OpenElem::NotationArg{arg:std::mem::replace(arg,Arg::Ib(0)),mode:*mode};
        }
        true
    } => {
        let t = node.as_term(Some(rest),parser);
        for e in rest.iter_mut() {
            if let OpenElem::Term{tm:OpenTerm::OMA{args,..}|OpenTerm::OMBIND{args,..}}
            | OpenElem::TopLevelTerm(OpenTerm::OMA{args,..}|OpenTerm::OMBIND{args,..})= e {
                let ext = (args.len()..arg.index() as usize).map(|_| None);
                args.extend(ext);
                match *args.get_mut((arg.index() - 1) as usize).unwrap() {
                    Some((TermOrList::List(ref mut ls),_)) => ls.push(t),
                    ref mut o => *o = Some((TermOrList::Term(t),mode))
                }
                return true
            } else if let OpenElem::Rule{args,..} = e {
                let ext = (args.len()..arg.index() as usize).map(|_| None);
                args.extend(ext);
                *args.get_mut((arg.index() - 1) as usize).unwrap() = Some((t,mode));
                return true
            };
        }
        iterate!(node(s:&HTMLParser=parser,t:Term=t,arg:Arg=arg,mode:ArgType=mode),
            e => if let OpenElem::Term{tm:OpenTerm::OMA{args,..}|OpenTerm::OMBIND{args,..}}
            | OpenElem::TopLevelTerm(OpenTerm::OMA{args,..}|OpenTerm::OMBIND{args,..})= e {
                let len = args.len();
                if arg.index() as usize > len {
                    //println!("HERE: {arg:?}");
                    args.extend((len..arg.index() as usize).map(|_| None));
                }
                match *args.get_mut((arg.index() - 1) as usize).unwrap() {
                    Some((TermOrList::List(ref mut ls),_)) => ls.push(t),
                    ref mut o@None if matches!(arg,Arg::AB(..)) =>
                        *o = Some((TermOrList::List(vec![t]),mode)),
                    ref mut o => *o = Some((TermOrList::Term(t),mode))
                }
                return true
            } else if let OpenElem::Rule{args,..} = e {
                let ext = (args.len()..arg.index() as usize).map(|_| None);
                args.extend(ext);
                *args.get_mut((arg.index() - 1) as usize).unwrap() = Some((t,mode));
                // sequences
                return true
            };
            {
                println!("OOOOOF\n\n{}\n({})",s.document.node.to_string(),s.uri());
            }
        );
        true
    };

    HeadTerm = "shtml:headterm": 115 {
        add!(OpenElem::HeadTerm)
    } => {
        let t = node.as_term(Some(rest),parser);
        iterate!(node(t:Term=t,parser:&HTMLParser=parser) -> rest,
            e => {
                if let OpenElem::Term {tm:OpenTerm::Complex(_,n)|OpenTerm::OMA{head_term:n,..}|OpenTerm::OMBIND {head_term:n,..}} |
                    OpenElem::TopLevelTerm(OpenTerm::Complex(_,n)|OpenTerm::OMA{head_term:n,..}|OpenTerm::OMBIND {head_term:n,..}) = e {
                    *n=Some(t);return true
                }
            };
            println!("TODO: Something is fishy here ({})",parser.uri())
        );
        true
    };

    Importmodule(uri:ModuleURI) = "shtml:import": 150 {
        let uri = if let Some(uri) = get_mod_uri(v,parser.backend,parser.uri().language()) {uri} else {
            todo!("HERE: {v}");
        };
        add!(-OpenElem::Importmodule{uri})
    } => {
        parser.add_triple(ulo!((parser.content_iri()) IMPORTS (uri.to_iri())));
        parser.add_content(ContentElement::Import(uri));
        false
    };
    Usemodule(uri:ModuleURI) = "shtml:usemodule": 150 {
        let uri = if let Some(uri) = get_mod_uri(v,parser.backend,parser.uri().language()) {uri} else {
            todo!("HERE: {v}");
        };
        add!(-OpenElem::Usemodule{uri})
    } => {
        parser.add_triple(ulo!((parser.iri()) !(dc::REQUIRES) (uri.to_iri())));
        parser.add_doc(DocumentElement::UseModule(uri));
        false
    };

    InputRef(id:Name,target:DocumentURI) = "shtml:inputref": 160 {
        let uri = if let Some(uri) = get_doc_uri(v,parser.backend) {uri} else {
            todo!("HERE: {v}");
        };
        add!(-OpenElem::InputRef {
            id: get!(ID),
            target: uri,
        })
    } => {
        let range = {node.data.borrow().range};
        parser.add_triple(ulo!((parser.iri()) !(dc::HAS_PART) (target.to_iri())));
        parser.add_doc(DocumentElement::InputRef(DocumentReference {
            id,target,range
        }));true
    };
    SetSectionLevel(lvl:SectionLevel) = "shtml:sectionlevel": 160 {
        if let Some(lvl) = u8::from_str(v).ok().map(|u| u.try_into().ok()).flatten() {
            add!(-OpenElem::SetSectionLevel{lvl})
        } else {
            todo!()
        }
    } => {
        parser.add_doc(DocumentElement::SetSectionLevel(lvl));false
    };

    Argmode = "shtml:argmode": 170 {+} => {!};
    Argnum = "shtml:argnum": 170 {+} => {!};



    Proof = "shtml:proof": 10 {PAR: Proof} => {!}; // TODO
    SubProof = "shtml:subproof": 10 {PAR: SubProof} => {!}; // TODO

    Precondition(dim:CognitiveDimension,sym:SymbolURI) = "shtml:preconditiondimension": 240 {
        let dim: CognitiveDimension = v.parse().ok().unwrap_or_else(|| {
            todo!()
        });
        let sym = get!("shtml:preconditionsymbol",s =>
            if let Some(uri) = get_sym_uri(s,parser.backend,parser.uri().language()) {uri} else {
                todo!()
            }
        ).unwrap_or_else(|| {
            todo!()
        });
        add!(- OpenElem::Precondition {dim,sym})
    } => {
        iterate!(node(dim:CognitiveDimension=dim,sym:SymbolURI=sym,parser:&HTMLParser=parser),
            e => if let OpenElem::Problem {preconditions,..} | OpenElem::SubProblem {preconditions,..} = e {
                preconditions.push((dim,sym));return true
            };
            println!("TODO: precondition without a problem ({})",parser.uri())
        );
        true
    };
    PreconditionSymbol = "shtml:preconditionsymbol": 250 {+} => {!};

    Objective(dim:CognitiveDimension,sym:SymbolURI) = "shtml:objectivedimension": 240 {
        let dim: CognitiveDimension = v.parse().ok().unwrap_or_else(|| {
            todo!()
        });
        let sym = get!("shtml:objectivesymbol",s =>
            if let Some(uri) = get_sym_uri(s,parser.backend,parser.uri().language()) {uri} else {
                todo!()
            }
        ).unwrap_or_else(|| {
            todo!()
        });
        add!(- OpenElem::Objective {dim,sym})
    } => {
        iterate!(node(dim:CognitiveDimension=dim,sym:SymbolURI=sym,parser:&HTMLParser=parser),
            e => if let OpenElem::Problem {objectives,..} | OpenElem::SubProblem {objectives,..} = e {
                objectives.push((dim,sym));return true
            };
            println!("TODO: precondition without a problem ({})",parser.uri())
        );
        true
    };
    ObjectiveSymbol = "shtml:objectivesymbol": 250 {+} => {!};

    // --- TODO -------------
    SRef = "shtml:sref" : 249 {+} => {!};
    SRefIn = "shtml:srefin" : 249 {+} => {!};
    Framenumber = "shtml:framenumber" : 249 {+} => {!};
    Slideshow = "shtml:slideshow" : 249 {+} => {!};
    SlideshowSlide = "shtml:slideshow-slide" : 249 {+} => {!};
    SkipSection = "shtml:skipsection": 40 {+} => {!};
    Answerclass = "shtml:answerclass": 40 {+} => {!};
    AnswerclassPts = "shtml:answerclass-pts": 40 {+} => {!};
    AnswerclassFeedback = "shtml:answerclass-feedback": 40 {+} => {!};
    ProblemMinutes = "shtml:problemminutes": 40 {+} => {!};
    ProblemMCB = "shtml:multiple-choice-block": 40 {+} => {!};
    ProblemSCB = "shtml:single-choice-block": 40 {+} => {!};
    ProblemMCC = "shtml:mcc": 40 {+} => {!};
    ProblemMCCSolution = "shtml:mcc-solution": 40 {+} => {!};
    ProblemSCC = "shtml:scc": 40 {+} => {!};
    ProblemSCCSolution = "shtml:scc-solution": 40 {+} => {!};
    ReturnType = "shtml:returntype": 50 {+} => {!};
    ArgTypes = "shtml:argtypes": 50 {+} => {!};
    Fillinsol = "shtml:fillinsol": 250 {+} => {!};
    FillinsolCase = "shtml:fillin-case": 250 {+} => {!};
    FillinsolCaseValue = "shtml:fillin-case-value": 250 {+} => {!};
    FillinsolCaseVerdict = "shtml:fillin-case-verdict": 250 {+} => {!};
    FillinsolValue = "shtml:fillin-value": 250 {+} => {!};
    FillinsolVerdict = "shtml:fillin-verdict": 250 {+} => {!};
    IfInputref = "shtml:ifinputref": 250 {+} => {!};
    // --- TODO -------------


    Invisible = "shtml:visible": 254 {add!(- OpenElem::Invisible)} => {parser.in_term};
    Problempoints(pts:f32) = "shtml:problempoints": 254 {
        let pts = v.parse().ok().unwrap_or_else(|| {
            todo!()
        });
        add!(- OpenElem::Problempoints{pts})
    } => {
        iterate!(node(pts:f32=pts,parser:&HTMLParser=parser),
            e => if let OpenElem::Problem {points,..} | OpenElem::SubProblem {points,..} = e {
                *points = Some(pts);return true
            };
            println!("TODO: problempoints without a problem ({})",parser.uri())
        );
        true
    };
    Solution = "shtml:solution": 254 {add!(- OpenElem::Solution)} => {
        let rf = parser.store_node(node);
        iterate!(node(rf:NarrativeRef<String>=rf,parser:&HTMLParser=parser),e =>
            if let OpenElem::Problem {ref mut solutions,..} | OpenElem::SubProblem {ref mut solutions,..} = e {
                solutions.push(rf);return true
            };
            println!("TODO: solution without a problem ({})",parser.uri())
        );
        node.kill();
        true
    };
    ProblemHint = "shtml:problemhint": 254 {add!(- OpenElem::ProblemHint)} => {
        let rf = parser.store_node(node);
        iterate!(node(rf:NarrativeRef<String>=rf,parser:&HTMLParser=parser),e =>
            if let OpenElem::Problem {ref mut hints,..} | OpenElem::SubProblem {ref mut hints,..} = e {
                hints.push(rf);return true
            };
            println!("TODO: hint without a problem ({})",parser.uri())
        );
        node.kill();
        true
    };
    ProblemNote = "shtml:problemnote": 254 {add!(- OpenElem::ProblemNote)} => {
        let rf = parser.store_node(node);
        iterate!(node(rf:NarrativeRef<String>=rf,parser:&HTMLParser=parser),e =>
            if let OpenElem::Problem {ref mut notes,..} | OpenElem::SubProblem{ref mut notes,..} = e {
                notes.push(rf);return true
            };
            println!("TODO: note without a problem ({})",parser.uri())
        );
        node.kill();
        true
    };
    ProblemGradingNote = "shtml:problemgnote": 254 {add!(- OpenElem::ProblemGradingNote)} => {
        let rf = parser.store_node(node);
        iterate!(node(rf:NarrativeRef<String>=rf,parser:&HTMLParser=parser),e =>
            if let OpenElem::Problem {ref mut gnotes,..}  | OpenElem::SubProblem{ref mut gnotes,..} = e {
                gnotes.push(rf);return true
            };
            println!("TODO: gnote without a problem ({})",parser.uri())
        );
        node.kill();
        true
    };
    Comp = "shtml:comp": 254 {
        if !v.is_empty() && parser.strip {
            attrs.get_mut(i).unwrap().value = "".into()
        }
        i += 1;
    } => {!};
    Varcomp = "shtml:varcomp": 254 {
        if !v.is_empty() && parser.strip {
            attrs.get_mut(i).unwrap().value = "".into()
        }
        i += 1;
    } => {!};
    Maincomp = "shtml:maincomp": 254 {
        if !v.is_empty() && parser.strip {
            attrs.get_mut(i).unwrap().value = "".into()
        }
        i += 1;
    } => {
        node.data.borrow_mut().elem.push(OpenElem::Maincomp);true
    };
    ProofMethod = "shtml:proofmethod": 254 {+} => {!};
    ProofSketch = "shtml:proofsketch": 254 {+} => {!};
    ProofTerm = "shtml:proofterm": 254 {+} => {!};
    ProofBody = "shtml:proofbody": 254 {+} => {!};
    ProofAssumption = "shtml:spfassumption": 254 {+} => {!};
    ProofHide = "shtml:proofhide": 254 {+} => {!};
    ProofStep = "shtml:spfstep": 254 {+} => {!};
    ProofStepName = "shtml:stepname": 254 {+} => {!};
    ProofEqStep = "shtml:spfeqstep": 254 {+} => {!};
    ProofPremise = "shtml:premise": 254 {+} => {!};
    ProofConclusion = "shtml:spfconclusion": 254 {+} => {!};
    AssocType = "shtml:assoctype": 254 {+} => {!};
    ArgumentReordering = "shtml:reorderargs": 254 {+} => {!};
    Bind = "shtml:bind":254 {+} => {!};
    Frame = "shtml:frame": 254 {+} => {!};
    Head = "shtml:head": 254 {+} => {!};
    NotationId = "shtml:notationid": 254 {+} => {!};
    Language = "shtml:language": 254 {+} => {!};
    Metatheory = "shtml:metatheory": 254 {+} => {!};
    Signature = "shtml:signature": 254 {+} => {!};
    Args = "shtml:args": 254 {+} => {!};
    Macroname = "shtml:macroname": 254 {+} => {!};
    CurrentSectionLevel = "shtml:currentsectionlevel": 254 {+} => {!};
    Styles = "shtml:styles": 254 {+} => {!};
    Inline = "shtml:inline": 254 {+} => {!};
    Fors = "shtml:fors": 254 {+} => {!};
    Id = "shtml:id": 254 {+} => {!};
    NotationFragment = "shtml:notationfragment": 254 {+} => {!};
    Precedence = "shtml:precedence": 254 {+} => {!};
    Role = "shtml:role": 254 {+} => {!};
    Argprecs = "shtml:argprecs": 254 {+} => {!};
    Autogradable = "shtml:autogradable": 254 {+} => {!};
    Capitalize = "shtml:capitalize": 254 {+} => {!};
}



const MATHHUB: &str = "http://mathhub.info";
const META: &str = "http://mathhub.info/sTeX/meta";
const URTHEORIES: &str = "http://cds.omdoc.org/urtheories";

lazy_static::lazy_static! {
    static ref META_URI: ArchiveURI = ArchiveURI::new(BaseURI::new_unchecked("http://mathhub.info/:sTeX"),ArchiveId::new("sTeX/meta-inf"));
    static ref UR_URI: ArchiveURI = ArchiveURI::new(BaseURI::new_unchecked("http://cds.omdoc.org"),ArchiveId::new("MMT/urtheories"));

}

fn split(archives:&[Archive],p:&str) -> Option<(ArchiveURI,usize)> {
    if p.starts_with(META) {
        return Some((META_URI.clone(),29))
    } else if p == URTHEORIES {
        return Some((UR_URI.clone(),31))
    } else if p == "http://mathhub.info/my/archive" {
        return Some((ArchiveURI::new(BaseURI::new("http://mathhub.info").unwrap(),ArchiveId::new("my/archive")),30))
    } else if p == "http://kwarc.info/Papers/stex-mmt/paper" {
        return Some((ArchiveURI::new(BaseURI::new("https://stexmmt.mathhub.info/:sTeX").unwrap(),ArchiveId::new("Papers/22-CICM-Injecting-Formal-Mathematics")),34))
    } else if p == "http://kwarc.info/Papers/tug/paper" {
        return Some((ArchiveURI::new(BaseURI::new("https://stexmmt.mathhub.info/:sTeX").unwrap(),ArchiveId::new("Papers/22-TUG-sTeX")),34))
    }
    if p.starts_with(MATHHUB) {
        let mut p = &p[MATHHUB.len()..];
        let mut i = MATHHUB.len();
        if let Some(s) = p.strip_prefix('/') {
            p = s;
            i += 1;
        }
        return split_old(archives,p,i)
    }
    archives.iter().find_map(|a| {
        let base = a.uri().base().as_str();
        if p.starts_with(base) {
            let l = base.len();
            let np = &p[l..];
            let id = a.id().as_str();
            if np.starts_with(id) {
                Some((a.uri().to_owned(),l+id.len()))
            } else {None}
        } else { None }
    })
}

fn split_old(archives:&[Archive],p:&str,len:usize) -> Option<(ArchiveURI,usize)> {
    archives.iter().find_map(|a| {
        if p.starts_with(a.id().as_str()) {
            let mut l = a.id().as_str().len();
            let np = &p[l..];
            if np.starts_with('/') {
                l += 1;
            }
            Some((a.uri().to_owned(),len + l))
        } else { None }
    })
}


fn get_doc_uri(s: &str,archives:&Backend) -> Option<DocumentURI> {
    let (p,mut m) = s.rsplit_once('/')?;
    let (a,l) = split(&archives.all_archives(),p)?;
    let mut path = if l < p.len() {&p[l..]} else {""};
    if path.starts_with('/') {
        path = &path[1..];
    }
    let lang = Language::from_rel_path(m);
    m = m.strip_suffix(&format!(".{}",lang.to_string())).unwrap_or(m);
    Some(DocumentURI::new(a,if path.is_empty() {None} else {Some(path)},m,lang))
}

fn get_mod_uri(s: &str,archives:&Backend,lang:Language) -> Option<ModuleURI> {
    let (mut p,mut m) = s.rsplit_once('?')?;
    m = m.strip_suffix("-module").unwrap_or(m);
    if p.bytes().last() == Some(b'/') {
        p = &p[..p.len()-1];
    }
    let (a,l) = split(&archives.all_archives(),p)?;
    let mut path = if l < p.len() {&p[l..]} else {""};
    if path.starts_with('/') {
        path = &path[1..];
    }
    let path = if path.is_empty() {None} else {Some(path)};
    Some(ModuleURI::new(a,path,m,lang))
}

fn get_sym_uri(s: &str,archives:&Backend,lang:Language) -> Option<SymbolURI> {
    let (m,s) = match s.split_once('[') {
        Some((m,s)) => {
            let (m,_) = m.rsplit_once('?')?;
            let (a,b) = s.rsplit_once(']')?;
            let am = get_mod_uri(a,archives,lang)?;
            let n = am.name() / b;
            let m = get_mod_uri(m,archives,lang)?;
            return Some(SymbolURI::new(m,n))
        }
        None => s.rsplit_once('?')?
    };
    let m = get_mod_uri(m,archives,lang)?;
    Some(SymbolURI::new(m,s))
}

fn replace_id(s:&str) -> String {
    if let Some((_,id)) = s.rsplit_once('?') {
        id.into()
    } else { s.into() }
}