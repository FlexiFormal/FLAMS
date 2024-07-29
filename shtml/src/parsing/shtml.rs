use html5ever::Attribute;
use crate::docs::{Arg, OpenTerm};
use crate::parsing::parser::{HTMLParser, NodeWithSource, OpenElems};
use std::str::FromStr;
use kuchikiki::NodeRef;
use immt_api::backend::archives::{Archive, Storage};
use immt_api::backend::manager::ArchiveManager;
use immt_api::core::content::{ArgSpec, ArgType, ArrayVec, AssocType, Constant, ContentElement, MathStructure, Module, Notation, Term, TermOrList, VarOrSym};
use immt_api::core::narration::{DocumentElement, DocumentMathStructure, DocumentModule, DocumentReference, Language, LogicalParagraph, Problem, Section, SectionLevel, StatementKind};
use immt_api::core::uris::archives::{ArchiveId, ArchiveURI};
use immt_api::core::uris::base::BaseURI;
use immt_api::core::uris::documents::DocumentURI;
use immt_api::core::uris::modules::ModuleURI;
use immt_api::core::uris::symbols::SymbolURI;
use immt_api::core::utils::sourcerefs::{ByteOffset, SourceRange};
use immt_api::core::utils::VecMap;
use immt_api::core::uris::ContentURI;

macro_rules! iterate {
    ($node:expr,$e:ident => $f:expr;$p:ident => $cont:expr;$or:expr) => {
        if let Some($p) = &$node.data.borrow().parent {
            let mut data = $p.data.borrow_mut();
            for $e in data.elem.iter_mut().rev() { $f }
            drop(data); return $cont
        } else {
            $or
        }
    };
    (@F $node:ident $(($($i:ident:$t:ty=$d:expr),+))?,$e:ident => $f:expr;$or:expr) => {
        fn iter(node:&NodeWithSource$(,$($i:$t),*)?) {
            iterate!(node,
                $e => $f;
                p => iter(p$(,$($i),*)?);
                $or
            )
        }
        iter($node$(,$($d),*)?)
    };
}

impl<'a> HTMLParser<'a> {

    fn add_doc(&mut self,node:&NodeWithSource,elem:DocumentElement) {
        use OpenElem::*;
        iterate!(node,
            e => match e {
                Section {ref mut children,..} => return children.push(elem),
                SectionTitle {ref mut children,..} => return children.push(elem),
                LogicalParagraph {ref mut children,..} => return children.push(elem),
                Module {ref mut narrative_children,..} => return narrative_children.push(elem),
                MathStructure {ref mut narrative_children,..} => return narrative_children.push(elem),
                _ => ()
            };
            p => self.add_doc(p,elem);
            self.elems.push(elem)
        )
    }

    fn add_content(&mut self,node:&NodeWithSource,elem:ContentElement) {
        use OpenElem::*;
        if let Some(p) = &node.data.borrow().parent {
            let mut pd = p.data.borrow_mut();
            for e in pd.elem.iter_mut().rev() {match e {
                Module {content_children,..} |
                MathStructure {content_children,..} => return content_children.push(elem),
                _ => ()
            }}
            drop(pd);self.add_content(p, elem)
        } else {
            todo!()
        }
    }

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
}


macro_rules! tags {
    (@open $i:ident $parser:ident +) => { $i += 1 };
    (@open $i:ident $parser:ident PAR:$k:ident) => { {
        let fors = get!(!"shtml:fors",s =>
            s.split(",").map(|s| {
                if let Some(uri) = get_sym_uri(s.trim(),$parser.backend) {uri} else {
                    todo!()
                }
            }).collect()
        ).unwrap_or_default();
        let inline = get!(!"shtml:inline",c => c.eq_ignore_ascii_case("true")).unwrap_or(false);
        let styles:Vec<String> = get!(!"shtml:styles",s => s.split(',').map(|s| s.trim().to_string()).collect()).unwrap_or_default();
        let id = get!(ID);
        add!(OpenElem::LogicalParagraph {
            id,styles,inline,fors,children:Vec::new(),title:None,kind:StatementKind::$k,
            terms:VecMap::new()
        })
    } };
    (@open $i:ident $parser:ident $($open:tt)*) => { {$($open)*} };
    (@close !) => { true };
    (@close $($close:tt)*) => { {$($close)*} };
    ($v:ident,$node:ident,$slf:ident,$attrs:ident,$i:ident,$rest:ident,
        $($tag:ident$(($($n:ident:$t:ty),+))? = $shtml:literal : $weight:literal {$($open:tt)*} => {$($close:tt)*} $(=> {$on_open:expr})? ;)*
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
            fn parse_shtml(&mut self,$attrs:&mut Vec<Attribute>,ret:&mut OpenElems) {
                let mut $i = 0;
                let mut $slf = self;
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
                        if let Some(id) = get!("shtml:id",s => s.to_string()) {id} else {
                            let id = format!("ID_{}", $slf.id_counter);
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
                        break
                    };
                    let $v = a.value.as_ref();
                    //println!("Here: {k:?} = {}",$v);
                    match k {
                        $( SHTMLTag::$tag => tags!(@open $i $slf $($open)* ) ),*
                    }
                }
            }
            /// returns whether to keep the node (or delete it)
            pub(crate) fn close(&mut self,$node:&NodeWithSource,elem:OpenElem, $rest:&mut OpenElems) -> bool {
                let mut $slf = self;
                //println!(" - Closing {elem:?}");
                match elem {
                    OpenElem::LogicalParagraph {inline,kind,fors,styles,children,title,id,terms} => {
                        $slf.add_doc($node,DocumentElement::Paragraph(LogicalParagraph {
                            id,styles,inline,fors:fors.into_iter().map(|u| u.into()).collect(),children,title,kind,
                            range: $node.data.borrow().range,terms
                        }));
                        true
                    }
                    OpenElem::VarNotation {name,id,precedence,argprecs,inner} => {
                        $slf.in_notation = false;
                        $slf.add_doc($node,DocumentElement::VarNotation {
                            name,id,precedence,argprecs,inner:None
                        });
                        false
                    }
                    OpenElem::Symref { uri,notation} => {
                        $slf.add_doc($node,DocumentElement::Symref {uri,notation,range:$node.data.borrow().range});
                        true
                    }
                    OpenElem::Varref { name,notation} => {
                        $slf.add_doc($node,DocumentElement::Varref {name,notation,range:$node.data.borrow().range});
                        true
                    }
                    OpenElem::TopLevelTerm(t) => {
                        $slf.in_term = false;
                        $slf.add_doc($node,DocumentElement::TopTerm(t.close()));
                        true
                    }
                    OpenElem::NotationArg { arg, mode } => {
                        $node.data.borrow_mut().elem.push(OpenElem::NotationArg { arg, mode });true
                    }
                    $(OpenElem::$tag$({$($n),+})? => tags!(@close $($close)* ) ),*,
                    _ => {
                        //println!("{elem:?}");
                        todo!("{elem:?}")
                    }
                }
            }
        }
        #[derive(Debug)]
        pub(crate) enum OpenElem {
            TopLevelTerm(OpenTerm),
            Symref {
                uri:ContentURI,
                notation:Option<String>,
            },
            Varref {
                name:String,
                notation:Option<String>,
            },
            VarNotation {
                name:String,
                id:String,
                precedence:isize,
                argprecs:ArrayVec<isize,9>,
                inner:Option<String>
            },
            NotationArg { arg:Arg, mode:ArgType },
            LogicalParagraph {
                inline:bool,
                kind: StatementKind,
                fors:Vec<SymbolURI>,
                styles:Vec<String>,
                children:Vec<DocumentElement>,
                title: Option<(String, SourceRange<ByteOffset>)>,
                id:String,
                terms:VecMap<SymbolURI,Term>
            },
            $(
                $tag$({$($n:$t),+})?
            ),*
        }
        impl OpenElem {
            pub(crate) fn on_add(&mut self,$slf:&mut HTMLParser) {
                let $node = self;
                match $node {
                    OpenElem::TopLevelTerm(..) => {
                        $slf.in_term = true;
                    }
                    OpenElem::VarNotation{..} => {
                        $slf.in_notation = true;
                    }
                    $( OpenElem::$tag$({$($n),+})? => {$( $on_open = true )?} )*
                    _ => ()
                }
            }
        }
    }
}

tags!{v,node,parser,attrs,i,rest,
    Module(
        uri:ModuleURI,
        meta:Option<ModuleURI>,
        language:Option<Language>,
        signature:Option<Language>,
        content_children: Vec<ContentElement>,
        narrative_children: Vec<DocumentElement>
    ) = "shtml:theory" : 0 {
        let uri = if let Some(uri) = get_mod_uri(v,parser.backend) {uri} else {
            todo!("HERE: {v}");
        };
        let meta = get!("shtml:metatheory",s =>
            if let Some(m) = get_mod_uri(s,parser.backend) {m} else {
                todo!("HERE: {s}");
            });
        let language = get!("shtml:language",s =>
            if s.is_empty() {parser.language} else {if let Ok(l) = s.try_into() {l} else {
                todo!("HERE: {s}")
            }}
        );
        let signature: Option<Language> = get!("shtml:signature",s =>
            if s.is_empty() {None} else {if let Ok(l) = s.try_into() {Some(l)} else {
                todo!("HERE: {s}")
            }}
        ).flatten();
        add!(-OpenElem::Module {
            meta,language,signature,uri,content_children:Vec::new(),narrative_children:Vec::new()
        })
    } => {
        let dm = DocumentElement::Module(DocumentModule {
            name: uri.name().into(),
            range: node.data.borrow().range,
            children: narrative_children,
        });
        parser.add_doc(node,dm);
        let m = Module {
            uri, meta, language, signature, elements: content_children,
        };
        iterate!(@F node(s:&mut HTMLParser=parser,m:Module=m),
            e => if let OpenElem::Module {content_children,..} |
            OpenElem::MathStructure {content_children,..} = e {
                content_children.push(ContentElement::NestedModule(m));return
            };
            s.modules.push(m)
        );
        true
    };

    MathStructure(
        uri:ModuleURI,
        content_children: Vec<ContentElement>,
        narrative_children: Vec<DocumentElement>,
        macroname:Option<String>
    ) = "shtml:feature-structure" : 0 {
        let uri = if let Some(uri) = get_mod_uri(v,parser.backend) {uri} else {
            todo!("HERE: {v}");
        };
        let macroname = get!("shtml:macroname",s => if s.is_empty() {None} else {Some(s.to_string())}).flatten();
        add!(-OpenElem::MathStructure {
            uri,content_children:Vec::new(),narrative_children:Vec::new(),macroname
        })
    } => {
        let dm = DocumentElement::MathStructure(DocumentMathStructure {
            name: uri.name().split('/').last().unwrap().into(),
            range: node.data.borrow().range,
            children: narrative_children,
        });
        parser.add_doc(node,dm);
        let m = MathStructure {
            uri, elements: content_children,macroname
        };
        parser.add_content(node,ContentElement::MathStructure(m));
        true
    };

    Section(level:SectionLevel,title:Option<(String,SourceRange<ByteOffset>)>,children:Vec<DocumentElement>,id:String) = "shtml:section" : 10 {
        if let Some(level) = u8::from_str(v).ok().map(|u| u.try_into().ok()).flatten() {
            let id = get!(ID);
            add!(-OpenElem::Section {
                level,title:None,children:Vec::new(),id
            })
        } else {
            todo!()
        }
    } => {
        parser.add_doc(node,DocumentElement::Section(Section {
            level,title,children,range:node.data.borrow().range,id
        }));
        true
    };
    Definition = "shtml:definition": 10 {PAR: Definition} => {!};
    Paragraph = "shtml:paragraph": 10 {PAR: Paragraph} => {!};
    Assertion = "shtml:assertion": 10 {PAR: Assertion} => {!};
    Example = "shtml:example": 10 {PAR: Example} => {!};
    Proof = "shtml:proof": 10 {PAR: Proof} => {!};
    SubProof = "shtml:subproof": 10 {PAR: SubProof} => {!};
    Problem(
        id:String,
        autogradable:bool,
        language:Language,
        points:Option<f32>,
        title:Option<(String,SourceRange<ByteOffset>)>,
        solution:Option<SourceRange<ByteOffset>>,
        hint:Option<SourceRange<ByteOffset>>,
        note:Option<SourceRange<ByteOffset>>,
        gnote:Option<SourceRange<ByteOffset>>
    ) = "shtml:problem": 10 {
        let autogradable = get!(!"shtml:autogradable",c => c.eq_ignore_ascii_case("true")).unwrap_or(false);
        let language = get!("shtml:language",s =>
            if s.is_empty() {parser.language} else {if let Ok(l) = s.try_into() {l} else {
                todo!("HERE: {s}")
            }}
        ).unwrap_or(parser.language);
        let points = get!("shtml:problempoints",s => s.parse().ok()).flatten();
        let id = get!(ID);
        add!(OpenElem::Problem {
            id,autogradable,language,points,solution:None,hint:None,note:None,gnote:None,title:None
        })
    } => {
        parser.add_doc(node,DocumentElement::Problem(Problem {
            id,autogradable,language,points,solution,hint,note,gnote,title
        }));
        true
    };

    Doctitle = "shtml:doctitle" : 20 {add!(-OpenElem::Doctitle)} => {
        let title = node.node.children().filter_map(|e|
            if let Some(e) = e.as_text() {
                Some(e.borrow().clone())
            } else if e.as_element().is_some() {
                Some(e.to_string())
            } else { None }
        ).collect();
        parser.title = title;
        false
    };
    SectionTitle(children:Vec<DocumentElement>) = "shtml:sectiontitle": 20 {
        add!(-OpenElem::SectionTitle {children:Vec::new()})
    } => {
        let title = node.node.children().filter_map(|e|
            if let Some(e) = e.as_text() {
                Some(e.borrow().clone())
            } else if e.as_element().is_some() {
                Some(e.to_string())
            } else { None }
        ).collect();
        iterate!(@F node(ttl:String=title,range:SourceRange<ByteOffset>=node.data.borrow().range),
            e => if let OpenElem::Section { title, .. } = e {
                *title = Some((ttl, range));return
            };
            todo!()
        );
        true
    };
    StatementTitle(children:Vec<DocumentElement>) = "shtml:statementtitle": 20 {
        add!(-OpenElem::StatementTitle {children:Vec::new()})
    } => {
        // TODO add document children?
        let title = node.node.children().filter_map(|e|
            if let Some(e) = e.as_text() {
                Some(e.borrow().clone())
            } else if e.as_element().is_some() {
                Some(e.to_string())
            } else { None }
        ).collect();
        iterate!(@F node(ttl:String=title,range:SourceRange<ByteOffset>=node.data.borrow().range),
            e => if let OpenElem::LogicalParagraph { title, .. } = e {
                *title = Some((ttl, range)); return
            };
            todo!()
        );
        true
    };
    ProofTitle(children:Vec<DocumentElement>) = "shtml:prooftitle": 20 {
        add!(-OpenElem::ProofTitle {children:Vec::new()})
    } => {
        // TODO add document children?
        let title = node.node.children().filter_map(|e|
            if let Some(e) = e.as_text() {
                Some(e.borrow().clone())
            } else if e.as_element().is_some() {
                Some(e.to_string())
            } else { None }
        ).collect();
        iterate!(@F node(ttl:String=title,range:SourceRange<ByteOffset>=node.data.borrow().range),
            e => if let OpenElem::LogicalParagraph { title,kind:StatementKind::Proof, .. } = e {
                *title = Some((ttl, range)); return
            };
            todo!()
        );
        true
    };
    ProblemTitle(children:Vec<DocumentElement>) = "shtml:problemtitle": 20 {
        add!(-OpenElem::ProblemTitle {children:Vec::new()})
    } => {
        // TODO add document children?
        let title = node.node.children().filter_map(|e|
            if let Some(e) = e.as_text() {
                Some(e.borrow().clone())
            } else if e.as_element().is_some() {
                Some(e.to_string())
            } else { None }
        ).collect();
        iterate!(@F node(ttl:String=title,range:SourceRange<ByteOffset>=node.data.borrow().range),
            e => if let OpenElem::Problem { title, .. } = e {
                *title = Some((ttl, range)); return
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
        let uri = if let Some(uri) = get_sym_uri(v,parser.backend) {uri} else {
            todo!();
        };
        let role = get!("shtml:role",s => s.split(',').map(|s| s.trim().to_string()).collect());
        let assoctype = get!("shtml:assoctype",s => s.trim().parse().ok()).flatten();
        let arity = get!("shtml:args",s =>
            if let Ok(a) = s.parse() { a } else { todo!("args {s}")}).unwrap_or_default();
        let reordering = get!("shtml:reoderargs",s => s.to_string());
        let macroname = get!("shtml:macroname",s => if s.is_empty() {None} else {Some(s.to_string())}).flatten();
        add!(- OpenElem::Symdecl { uri, arity, macroname, role,tp:None,df:None, assoctype, reordering });
    } => {
        parser.add_content(node,ContentElement::Constant(Constant {
            uri,arity,macroname,role,tp,df,assoctype,reordering
        }));
        false
    };
    VarDef(name:String,
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
        let name = v.to_string();
        let role = get!("shtml:role",s => s.split(',').map(|s| s.trim().to_string()).collect());
        let arity = get!("shtml:args",s =>
            if let Ok(a) = s.parse() { a } else { todo!()}).unwrap_or_default();
        let assoctype = get!("shtml:assoctype",s => s.trim().parse().ok()).flatten();
        let reordering = get!("shtml:reoderargs",s => s.to_string());
        let bind = get!("shtml:bind",s => s.eq_ignore_ascii_case("true")).unwrap_or(false);
        let macroname = get!("shtml:macroname",s => if s.is_empty() {None} else {Some(s.to_string())}).flatten();
        add!(-OpenElem::VarDef { name, arity, macroname,role,tp:None,df:None,is_sequence:false,assoctype,reordering,bind });
    } => {
        parser.add_doc(node,DocumentElement::VarDef {
            name,arity,macroname,range:node.data.borrow().range,role,tp,df,is_sequence,
            assoctype,reordering,bind
        });
        false
    };
    VarSeq = "shtml:varseq":30 {
        let name = v.to_string();
        let role = get!("shtml:role",s => s.split(',').map(|s| s.trim().to_string()).collect());
        let arity = get!("shtml:args",s =>
            if let Ok(a) = s.parse() { a } else { todo!()}).unwrap_or_default();
        let assoctype = get!("shtml:assoctype",s => s.trim().parse().ok()).flatten();
        let reordering = get!("shtml:reoderargs",s => s.to_string());
        let bind = get!("shtml:bind",s => s.eq_ignore_ascii_case("true")).unwrap_or(false);
        let macroname = get!("shtml:macroname",s => if s.is_empty() {None} else {Some(s.to_string())}).flatten();
        add!(-OpenElem::VarDef { name, arity, macroname,role,tp:None,df:None,is_sequence:true,assoctype,reordering,bind });
    } => {!};
    Notation(uri:SymbolURI,
        id:String,
        precedence:isize,
        argprecs:ArrayVec<isize,9>,
        inner:Option<String>
    ) = "shtml:notation":30 {
        let uri = if let Some(uri) = get_sym_uri(v,parser.backend) {Ok(uri.into())}
        else if !v.contains('?') {
            Err(v.to_string())
        } else {
            //println!("Wut: {v}");
            todo!("Wut: {v}");
        };
        let fragment = get!("shtml:notationfragment",s =>
            if s.is_empty() {None} else {Some(s.to_string())}
        ).flatten();
        let prec = get!("shtml:precedence",s => s.parse().ok()).flatten();
        let argprecs: ArrayVec<_,9> = get!("shtml:argprecs",s =>
            s.split(',').map(|s| s.trim().parse().unwrap_or(0)).collect()
        ).unwrap_or_default();
        let id = fragment.unwrap_or_else(|| {
            let r = format!("ID_{}",parser.id_counter);
            parser.id_counter += 1;
            r
        });
        add!(- match uri {
            Ok(uri) => OpenElem::Notation {
                uri:uri, id, precedence:prec.unwrap_or(0), argprecs,inner:None
            },
            Err(name) => OpenElem::VarNotation {
                name, id, precedence:prec.unwrap_or(0), argprecs,inner:None
            }
        })
    } => {
        parser.in_notation = false;
        parser.add_content(node,ContentElement::Notation(Notation {
            uri,id,precedence,argprecs,range:node.data.borrow().range
        }));
        false
    } => { parser.in_notation };

    Definiendum(uri:SymbolURI) = "shtml:definiendum": 40 {
        if let Some(uri) = get_sym_uri(v,parser.backend) {
            attrs.get_mut(i).unwrap().value = uri.to_string().into();
            add!(OpenElem::Definiendum { uri })
        } else {
            i += 1
        }
    } => {
        iterate!(@F node(uri:&SymbolURI=&uri),
            e => if let OpenElem::LogicalParagraph { fors,kind,styles, .. } = e {
                if kind.is_definition_like(styles) {
                    if !fors.contains(uri) { fors.push(uri.clone()) }
                    return
                }
            };
            todo!()
        );
        parser.add_doc(node, DocumentElement::Definiendum {uri, range:node.data.borrow().range });
        true
    };

    Type = "shtml:type": 50 {
        add!(- OpenElem::Type)
    } => {
        let t = node.as_term(Some(rest));
        iterate!(@F node(t:Term=t),
            e => if let OpenElem::Symdecl {tp,..} | OpenElem::VarDef {tp,..} = e {
                *tp = Some(t);return
            };
            todo!()
        );
        parser.in_term = false;
        true
    } => { parser.in_term };

    Conclusion(uri:SymbolURI,in_term:bool) = "shtml:conclusion": 50 {
        let uri = if let Some(uri) = get_sym_uri(v,parser.backend) {uri} else {
            todo!();
        };
        let it = parser.in_term;
        add!(- OpenElem::Conclusion{uri,in_term:it})
    } => {
        let t = node.as_term(Some(rest));
        iterate!(@F node(uri:SymbolURI=uri,t:Term=t),
            e => if let OpenElem::LogicalParagraph {kind:StatementKind::Assertion,terms,..} = e {
                terms.insert(uri,t);return
            };
            todo!()
        );
        parser.in_term = in_term;
        true
    } => { parser.in_term };
    Definiens(uri:Option<SymbolURI>,in_term:bool) = "shtml:definiens": 50 {
        let uri = if !v.is_empty() {if let Some(uri) = get_sym_uri(v,parser.backend) {Some(uri)} else {
            todo!();
        }} else {None};
        let it = parser.in_term;
        add!(- OpenElem::Definiens{uri,in_term:it})
    } => {
        let t = node.as_term(Some(rest));
        iterate!(@F node(uri:Option<SymbolURI>=uri,t:Term=t),
            e => {
                if let OpenElem::LogicalParagraph {terms,..} = e {
                    if let Some(uri) = uri {terms.insert(uri,t);return}
                }
                if let OpenElem::Symdecl {df,..} | OpenElem::VarDef {df,..} = e {
                    *df = Some(t);return
                }
                if let OpenElem::Assign {tm} = e {
                    *tm = Some(t);return
                    return
                }
            };
            println!("TODO: Definiens is fishy")
        );
        parser.in_term = in_term;
        true
    } => { parser.in_term };
    Rule(id:String,args:ArrayVec<Option<(Term,ArgType)>,9>) = "shtml:rule": 50 { // TODO
        let id = v.to_string();
        add!(- OpenElem::Rule {
            id,args:ArrayVec::new()
        })
    } => {parser.in_term = false;false} => { parser.in_term };

    Term(tm:OpenTerm) = "shtml:term": 100 {
        if parser.in_notation {
            let _ = get!("shtml:term",_e => ());
            let _ = get!("shtml:head",_e => ());
            let _ = get!("shtml:notationid",_e => ());
            if !parser.strip { i+= 1}
        } else {
            let notation = get!(!"shtml:notationid",s => if s.is_empty() {None} else {Some(s.to_string())}).flatten();
            let head = get!(!"shtml:head",s =>
                if let Some(uri) = get_sym_uri(s,parser.backend) {VarOrSym::S(uri.into())}
                else if let Some(uri) = get_mod_uri(s,parser.backend) {VarOrSym::S(uri.into())}
                else if !s.contains('?') {VarOrSym::V(s.to_string())} else {
                    println!("HERE: {s}");
                    VarOrSym::V("ERROR".to_string())
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
            let term = match (tk,head,parser.in_term) {
                (TMK::OMID,VarOrSym::S(uri),true) => OpenElem::Term{tm:OpenTerm::Symref {uri,notation}},
                (TMK::OMID,VarOrSym::S(uri),_) => OpenElem::Symref { uri, notation },
                (TMK::OMV|TMK::OMID,VarOrSym::V(name),true) => OpenElem::Term{tm:OpenTerm::OMV {name,notation}},
                (TMK::OMV|TMK::OMID,VarOrSym::V(name),_) => OpenElem::Varref { name, notation },
                (TMK::OMV|TMK::OML,VarOrSym::V(name),true) => OpenElem::Term{tm:OpenTerm::OML {name}},
                (TMK::OMA,head,true) => OpenElem::Term{tm:OpenTerm::OMA {head,notation,args:ArrayVec::new()}},
                (TMK::OMA,head,_) => OpenElem::TopLevelTerm(OpenTerm::OMA {head,notation,args:ArrayVec::new()}),
                (TMK::OMB,head,true) => OpenElem::Term{tm:OpenTerm::OMBIND {head,notation,args:ArrayVec::new()}},
                (TMK::OMB,head,_) => OpenElem::TopLevelTerm(OpenTerm::OMBIND {head,notation,args:ArrayVec::new()}),
                (TMK::Complex,_,true) => OpenElem::Term{tm:OpenTerm::Complex(None)},
                (TMK::Complex,_,_) => OpenElem::TopLevelTerm(OpenTerm::Complex(None)),
                (t,h,b) => {
                    println!("TODO: Term is fishy: {t:?} {h} ({b})");
                    if b { OpenElem::Term{tm:OpenTerm::OMV{name:"TODO".to_string(),notation:None}} }
                    else { OpenElem::TopLevelTerm(OpenTerm::OMV{name:"TODO".to_string(),notation:None}) }
                }
            };
            add!(term)
        }
    } => {
        node.data.borrow_mut().elem.push(OpenElem::Term{tm});true
    };

    Arg(arg:crate::docs::Arg, mode:ArgType) = "shtml:arg": 110 {
        if parser.in_notation {
            let arg = get!(!"shtml:arg",s => s.parse().ok()).flatten().unwrap_or_else(|| {
                println!("{attrs:?}\n{parser:?}");
                todo!("{attrs:?}")
            });
            let mode = get!(!"shtml:argmode",s => s.parse().ok()).flatten().unwrap_or_else(|| {
                println!("{attrs:?}\n{parser:?}");
                todo!("{attrs:?}")
            });
            /*
            let num = get!(!"shtml:argnum",s => u8::from_str(s).ok()).flatten().unwrap_or_else(|| {
                println!("{attrs:?}\n{parser:?}");
                todo!("{attrs:?}")
            });*/
            add!(OpenElem::NotationArg{arg,mode})
        } else {
            let arg = get!(!"shtml:arg",s => s.parse().ok()).flatten().unwrap_or_else(|| {
                println!("{attrs:?}\n{parser:?}");
                todo!("{attrs:?}")
            });
            let mode = get!(!"shtml:argmode",s => s.parse().ok()).flatten().unwrap_or_else(|| {
                println!("{attrs:?}\n{parser:?}");
                todo!("{attrs:?}")
            });
            add!(OpenElem::Arg{arg,mode})
        }
    } => {
        let t = node.as_term(Some(rest));
        //println!("  = {t:?}");
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
        iterate!(@F node(s:&HTMLParser=parser,t:Term=t,arg:Arg=arg,mode:ArgType=mode),
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
                return
            } else if let OpenElem::Rule{args,..} = e {
                let ext = (args.len()..arg.index() as usize).map(|_| None);
                args.extend(ext);
                *args.get_mut((arg.index() - 1) as usize).unwrap() = Some((t,mode));
                // sequences
                return
            };
            {
                println!("OOOOOF\n\n{}",s.document.node.to_string());
            }
        );
        true
    };

    HeadTerm = "shtml:headterm": 115 {
        add!(OpenElem::HeadTerm)
    } => {
        let t = node.as_term(Some(rest));
        for e in rest.iter_mut() {
            if let OpenElem::Term {tm:OpenTerm::Complex(n)} | OpenElem::TopLevelTerm(OpenTerm::Complex(n)) = e {
                *n=Some(t);return true
            }
        }
        iterate!(@F node(t:Term=t),
            e => {
                if let OpenElem::Term {tm:OpenTerm::Complex(n)} | OpenElem::TopLevelTerm(OpenTerm::Complex(n)) = e {
                    *n=Some(t);return
                }
            };
            println!("TODO: Something is fishy here")
        );
        true
    };

    ArgSep = "shtml:argsep": 120 {
        let _ = get!("shtml:term",_e => ());
        let _ = get!("shtml:head",_e => ());
        let _ = get!("shtml:notationid",_e => ());
        let _ = get!("shtml:visible",_e => ());
        add!(-OpenElem::ArgSep)
    } => {!};

    NotationComp = "shtml:notationcomp": 130 {
        let _ = get!("shtml:term",_e => ());
        let _ = get!("shtml:head",_e => ());
        let _ = get!("shtml:notationid",_e => ());
        let _ = get!("shtml:visible",_e => ());
        add!(-OpenElem::NotationComp)
    } => {!};
    NotationOpComp = "shtml:notationopcomp": 130 {
        let _ = get!("shtml:term",_e => ());
        let _ = get!("shtml:head",_e => ());
        let _ = get!("shtml:notationid",_e => ());
        let _ = get!("shtml:visible",_e => ());
        add!(-OpenElem::NotationOpComp)
    } => {!};

    Importmodule(uri:ModuleURI) = "shtml:import": 150 {
        let uri = if let Some(uri) = get_mod_uri(v,parser.backend) {uri} else {
            todo!("HERE: {v}");
        };
        add!(-OpenElem::Importmodule{uri})
    } => {
        parser.add_content(node,ContentElement::Import(uri));
        false
    };
    Usemodule(uri:ModuleURI) = "shtml:usemodule": 150 {
        let uri = if let Some(uri) = get_mod_uri(v,parser.backend) {uri} else {
            todo!("HERE: {v}");
        };
        add!(-OpenElem::Usemodule{uri})
    } => {
        parser.add_doc(node,DocumentElement::UseModule(uri));
        false
    };

    InputRef(id:String,target:DocumentURI) = "shtml:inputref": 160 {
        let uri = if let Some(uri) = get_doc_uri(v,parser.backend) {uri} else {
            todo!("HERE: {v}");
        };
        add!(-OpenElem::InputRef {
            id: get!(ID),
            target: uri,
        })
    } => {
        let range = {node.data.borrow().range};
        parser.add_doc(node,DocumentElement::InputRef(DocumentReference {
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
        parser.add_doc(node,DocumentElement::SetSectionLevel(lvl));false
    };

    Argmode = "shtml:argmode": 170 {+} => {!};
    Argnum = "shtml:argnum": 170 {+} => {!};




    // --- TODO -------------
    SRef = "shtml:sref" : 249 {+} => {!};
    SRefIn = "shtml:srefin" : 249 {+} => {!};
    Framenumber = "shtml:framenumber" : 249 {+} => {!};
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
    PrecoditionDimension = "shtml:preconditiondimension": 250 {+} => {!};
    PrecoditionSymbol = "shtml:preconditionsymbol": 250 {+} => {!};
    ObjectiveDimension = "shtml:objectivedimension": 250 {+} => {!};
    ObjectiveSymbol = "shtml:objectivesymbol": 250 {+} => {!};
    Fillinsol = "shtml:fillinsol": 250 {+} => {!};
    FillinsolCase = "shtml:fillin-case": 250 {+} => {!};
    FillinsolCaseValue = "shtml:fillin-case-value": 250 {+} => {!};
    FillinsolValue = "shtml:fillin-value": 250 {+} => {!};
    FillinsolVerdict = "shtml:fillin-verdict": 250 {+} => {!};
    Subproblem = "shtml:subproblem": 250 {+} => {!};
    Morphism = "shtml:feature-morphism": 250 {+} => {!};
    MorphismDomain = "shtml:domain": 250 {+} => {!};
    MorphismTotal = "shtml:total": 250 {+} => {!};
    Rename = "shtml:rename": 250 {+} => {!};
    RenameTo = "shtml:to": 250 {+} => {!};
    AssignMorphismFrom = "shtml:assignmorphismfrom": 250 {+} => {!};
    AssignMorphismTo = "shtml:assignmorphismto": 250 {+} => {!};
    Assign(tm:Option<Term>) = "shtml:assign": 250 {
        add!(- OpenElem::Assign {tm:None})
    } => {!};
    IfInputref = "shtml:ifinputref": 250 {+} => {!};
    // --- TODO -------------




    Invisible = "shtml:visible": 254 {add!(- OpenElem::Invisible)} => {parser.in_term || parser.in_notation};
    Problempoints(pts:f32) = "shtml:problempoints": 254 {
        let pts = v.parse().ok().unwrap_or_else(|| {
            todo!()
        });
        add!(- OpenElem::Problempoints{pts})
    } => {
        iterate!(@F node(pts:f32=pts),
            e => if let OpenElem::Problem {points,..} = e {
                *points = Some(pts);return
            };
            todo!()
        );
        true
    };
    Solution = "shtml:solution": 254 {add!(- OpenElem::Solution)} => {
        let rf = parser.store_node(node);
        iterate!(@F node(rf:SourceRange<ByteOffset>=rf),e =>
            if let OpenElem::Problem {ref mut solution,..} = e {
                *solution = Some(rf);return
            };
            todo!()
        );
        node.kill();
        true
    };
    ProblemHint = "shtml:problemhint": 254 {add!(- OpenElem::ProblemHint)} => {
        let rf = parser.store_node(node);
        iterate!(@F node(rf:SourceRange<ByteOffset>=rf),e =>
            if let OpenElem::Problem {ref mut hint,..} = e {
                *hint = Some(rf);return
            };
            todo!()
        );
        node.kill();
        true
    };
    ProblemNote = "shtml:problemnote": 254 {add!(- OpenElem::ProblemNote)} => {
        let rf = parser.store_node(node);
        iterate!(@F node(rf:SourceRange<ByteOffset>=rf),e =>
            if let OpenElem::Problem {ref mut note,..} = e {
                *note = Some(rf);return
            };
            todo!()
        );
        node.kill();
        true
    };
    ProblemGradingNote = "shtml:problemgnote": 254 {add!(- OpenElem::ProblemGradingNote)} => {
        let rf = parser.store_node(node);
        iterate!(@F node(rf:SourceRange<ByteOffset>=rf),e =>
            if let OpenElem::Problem {ref mut gnote,..} = e {
                *gnote = Some(rf);return
            };
            todo!()
        );
        node.kill();
        true
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
    } => {!};
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
    static ref META_URI: ArchiveURI = ArchiveURI::new(BaseURI::new_unchecked(MATHHUB).unwrap(),ArchiveId::new("sTeX/meta-inf"));
    static ref UR_URI: ArchiveURI = ArchiveURI::new(BaseURI::new_unchecked("http://cds.omdoc.org").unwrap(),ArchiveId::new("MMT/urtheories"));

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
            let mut np = &p[l..];
            if let Some(s) = np.strip_prefix('/') {
                l += 1;
                np = s;
            }
            Some((a.uri().to_owned(),len + l))
        } else { None }
    })
}


fn get_doc_uri(s: &str,archives:&ArchiveManager) -> Option<DocumentURI> {
    let (mut p,m) = s.rsplit_once('/')?;
    let (a,l) = split(&archives.get_archives(),p)?;
    let mut path = if l < p.len() {&p[l..]} else {""};
    if path.starts_with('/') {
        path = &path[1..];
    }
    Some(DocumentURI::new(a,path,m))
}

fn get_mod_uri(s: &str,archives:&ArchiveManager) -> Option<ModuleURI> {
    let (mut p,m) = s.rsplit_once('?')?;
    if p.bytes().last() == Some(b'/') {
        p = &p[..p.len()-1];
    }
    let (a,l) = split(&archives.get_archives(),p)?;
    let mut path = if l < p.len() {&p[l..]} else {""};
    if path.starts_with('/') {
        path = &path[1..];
    }
    Some(ModuleURI::new(a,path,m))
}

fn get_sym_uri(s: &str,archives:&ArchiveManager) -> Option<SymbolURI> {
    let (m,s) = match s.split_once('[') {
        Some((m,_)) => {
            let (m,_) = m.rsplit_once('?')?;
            (m,&s[m.len()..])
        }
        None => s.rsplit_once('?')?
    };
    let m = get_mod_uri(m,archives)?;
    Some(SymbolURI::new(m,s))
}

fn replace_id(s:&str) -> String {
    if let Some((_,id)) = s.rsplit_once('?') {
        id.into()
    } else { s.into() }
}