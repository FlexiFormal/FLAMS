use std::borrow::Cow;
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use std::path::Path;
use html5ever::{Attribute, ExpandedName, parse_document, ParseOpts, QualName};
use immt_api::backend::manager::ArchiveManager;
use html5ever::tokenizer::*;
use html5ever::interface::{ElementFlags, NextParserState, NodeOrText, QuirksMode, TreeSink};
use html5ever::tendril::StrTendril;
use immt_api::core::content::{ArgType, ArrayVec, ContentElement, InformalChild, Module, Notation, NotationComponent, Term, VarNameOrURI};
use immt_api::core::narration::{CSS, Document, DocumentElement, FullDocument, Language, NarrativeRef, Title};
use immt_api::core::uris::documents::DocumentURI;
use kuchikiki::{ElementData, NodeRef};
use tendril::{SliceExt, TendrilSink};
use immt_api::backend::Backend;
use immt_api::core::ontology::rdf::terms::Quad;
use immt_api::core::ulo;
use immt_api::core::uris::{ModuleURI, Name, NarrativeURI, NarrDeclURI, SymbolURI};
use immt_api::core::utils::sourcerefs::{ByteOffset, SourceRange};
use immt_api::core::utils::VecMap;
use crate::docs::OpenTerm;
use crate::parsing::shtml::OpenElem;

pub(crate) type OpenElems = ArrayVec<OpenElem,8>;

macro_rules! sanity_check {
    ($s:expr,$a:expr) => {
        ()//if !$a { brk($s) }
    }
}
macro_rules! debug_check {
    ($($t:tt)*) => {
        ()//println!($($t)*)
    }
}


#[derive(Debug)]
pub(crate) struct NodeData {
    pub(crate) range:SourceRange<ByteOffset>,
    pub(crate) parent:Option<NodeWithSource>,
    pub(crate) children:Vec<NodeWithSource>,
    pub(crate) elem:OpenElems,
    pub(crate) closed:bool
}

#[derive(Clone,Debug)]
pub struct NodeWithSource {
    pub(crate) node:NodeRef,
    pub(crate) data:std::rc::Rc<std::cell::RefCell<NodeData>>
}
impl PartialEq for NodeWithSource {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}
impl NodeWithSource {
    pub(crate) fn kill(&self) {
        let mut d = self.data.borrow_mut();
        let _ = d.parent.take();
        let c = std::mem::take(&mut d.children);
        drop(d);
        for c in c { c.kill() }
    }
    fn new(node:NodeRef,len:usize,elem:OpenElems) -> Self {
        NodeWithSource {
            node,
            data:std::rc::Rc::new(std::cell::RefCell::new(NodeData {
                range: SourceRange { start: ByteOffset { offset: 0 }, end: ByteOffset { offset: len } },
                parent:None,
                children: Vec::new(),
                elem,closed:false
            }))
        }
    }
    fn end(&self) -> usize {
        self.data.borrow().range.end.offset
    }
    fn prolong(&self,len:usize) {
        self.data.borrow_mut().range.end.offset += len;
        let data = self.data.borrow();
        if let Some(n) = &data.parent {
            n.prolong(len)
        }
    }
    fn len(&self) -> usize {
        let rng = &self.data.borrow().range;
        rng.end.offset - rng.start.offset
    }
    fn shift(&self,off:usize) {
        let mut rng = self.data.borrow_mut();
        rng.range.start.offset += off;
        rng.range.end.offset += off;
    }
    fn add(&self,node:NodeWithSource) {
        let inner = node.node.clone();
        self.node.append(inner);
        node.data.borrow_mut().parent = Some(self.clone());
        self.data.borrow_mut().children.push(node)
    }
    fn add_text(&self,text:StrTendril) -> usize {
        if let Some(last_child) = self.node.last_child() {
            if let Some(existing) = last_child.as_text() {
                let t = last_child.to_string().len();
                existing.borrow_mut().push_str(&text);
                last_child.to_string().len() - t
            } else {
                let n = NodeRef::new_text(text);
                let l = n.to_string().len();
                self.node.append(n);l
            }
        } else {
            let n = NodeRef::new_text(text);
            let l = n.to_string().len();
            self.node.append(n);l
        }
    }
    pub(crate) fn as_notation(&self,id:Name,op:Option<Self>,precedence:isize,argprecs:ArrayVec<isize,9>) -> Notation {
        use std::fmt::Write;
        fn get_is_text_and_offset(e:&ElementData) -> (bool,u8) {
            match e.name.local.as_ref() {
                s@ ("span"|"div") => (true,s.len() as u8 + 1),
                s => (false,s.len() as u8 + 1)
            }
        }
        let op = op.map(|e|
            if let Some(n) = e.node.as_element() {
                let (is_text,attribute_index) = get_is_text_and_offset(n);
                let s = e.node.to_string();
                (s,attribute_index,is_text)
            } else {
                todo!()
            }
        );
        //println!("HERE! {}",self.node.to_string());
        if let Some(n) = self.node.as_element() {
            let (is_text,attribute_index) = get_is_text_and_offset(n);
            // TODO de-recursify
            fn rec(node:&NodeWithSource,ret:&mut Vec<NotationComponent>,currstr:&mut String) -> (u8,ArgType) {
                let elems = { node.data.borrow_mut().elem.take() };
                let data = node.data.borrow();
                let mut index = 0;
                let mut tp = ArgType::Normal;
                for e in elems.into_iter() { match e {
                    OpenElem::NotationArg { arg,mode} => {
                        if !currstr.is_empty() {
                            ret.push(NotationComponent::S(std::mem::take(currstr).into()));
                        }
                        index = arg.index();
                        ret.push(NotationComponent::Arg(arg,mode));
                        return (arg.index(),mode)
                    }
                    OpenElem::Comp => {
                        if !currstr.is_empty() {
                            ret.push(NotationComponent::S(std::mem::take(currstr).into()));
                        }
                        ret.push(NotationComponent::Comp(node.node.to_string().into()));
                        return (index,tp)
                    }
                    OpenElem::Maincomp => {
                        if !currstr.is_empty() {
                            ret.push(NotationComponent::S(std::mem::take(currstr).into()));
                        }
                        ret.push(NotationComponent::MainComp(node.node.to_string().into()));
                        return (index,tp)
                    }
                    OpenElem::ArgSep => {
                        if !currstr.is_empty() {
                            ret.push(NotationComponent::S(std::mem::take(currstr).into()));
                        }
                        let mut sep = String::new();
                        let mut nret = Vec::new();
                        let mut idx = 0;
                        let mut tp = ArgType::Sequence;
                        for c in &data.children {
                            let (r,t) = rec(c,&mut nret,&mut sep);
                            if r != 0 {
                                idx = r; tp = t;
                            }
                        }
                        if !sep.is_empty() {
                            nret.push(NotationComponent::S(sep.into()));
                        }
                        ret.push(NotationComponent::ArgSep{index:idx,tp,sep:nret});
                        return (index,tp)
                    }
                    OpenElem::ArgMap => {
                        if !currstr.is_empty() {
                            ret.push(NotationComponent::S(std::mem::take(currstr).into()));
                        }

                        let mut sep = String::new();
                        let mut nret = Vec::new();
                        let mut idx = 0;
                        let mut tp = ArgType::Sequence;
                        for c in &data.children {
                            let (r,t) = rec(c,&mut nret,&mut sep);
                            if r != 0 {
                                idx = r; tp = t;
                            }
                        }
                        if !sep.is_empty() {
                            nret.push(NotationComponent::S(sep.into()));
                        }
                        ret.push(NotationComponent::ArgMap{index:idx,segments:nret});
                        return (index,tp)
                    }
                    _ => ()
                }}

                if let Some(elem) = node.node.as_element() {
                    write!(currstr,"<{}",elem.name.local.as_ref()).unwrap();
                    for (k,v) in elem.attributes.borrow().map.iter() {
                        write!(currstr," {}=\"{}\"",k.local.as_ref(),v.value).unwrap();
                    }
                    currstr.push('>');
                    let mut nws = data.children.iter();
                    for c in node.node.children() {
                        if let Some(_) = c.as_comment() {
                            nws.next();
                            // ignore
                        } else if let Some(t) = c.as_text() {
                            currstr.push_str(&**t.borrow());
                        } else if let Some(_) = c.as_element() {
                            let nc = nws.next().unwrap();
                            assert_eq!(nc.node, c);
                            /*(index,tp) =*/ rec(nc,ret,currstr);
                        } else {
                            unreachable!("??? {c:?}")
                        }
                    }
                    write!(currstr,"</{}>",elem.name.local.as_ref()).unwrap();
                    assert!(nws.next().is_none());
                } else if let Some(_) = node.node.as_comment() {
                    // ignore
                }  else if let Some(t) = node.node.as_text() {
                    currstr.push_str(&**t.borrow())
                }  else {
                    todo!("Unknown notation node {}",node.node.to_string())
                }
                (index,tp)
            }
            let mut ret = Vec::new();
            let mut str = String::new();
            rec(self,&mut ret,&mut str);
            if !str.is_empty() {
                ret.push(NotationComponent::S(str.into()))
            }
            //println!("HERE NOTATION: {ret:?}");
            Notation {id,precedence,attribute_index,argprecs,nt:ret,op,is_text}
        } else {
            let mut ret = "<span>".to_string();
            ret.push_str(&self.node.to_string());
            ret.push_str("</span>");
            Notation {id,precedence,attribute_index:5,argprecs,nt:vec![NotationComponent::S(ret.into())],op,is_text:true}
        }
    }
    pub(crate) fn as_term(&self,rest:Option<&mut OpenElems>,parser:&mut HTMLParser) -> Term {
        //println!("Term: {}",self.node.to_string());
        if let Some(rest) = rest {for (i,e) in rest.iter().enumerate() {
            //println!("  - {e:?}");
            match e {
                OpenElem::Term{tm:OpenTerm::Complex(_,None)} => (),
                OpenElem::Term{..} => {
                    if let OpenElem::Term{tm} = rest.remove(i) {
                        return tm.close(parser)
                    } else {unreachable!()}
                }
                _ => ()
        }}}
        let mut data = self.data.borrow_mut();
        for (i,e) in data.elem.iter().enumerate() {
            //println!("  - {e:?}");
            match e {
                OpenElem::Term{tm:OpenTerm::Complex(_,None)} => (),
                OpenElem::Term{..} => {
                    if let OpenElem::Term{tm} = data.elem.remove(i) {
                        return tm.close(parser)
                    } else {unreachable!()}
                }
                _ => ()
            }}
        drop(data);
        if let Some(elem) = self.node.as_element() {
            let data = self.data.borrow();
            if data.children.len() == 1 {
                let c = data.children.first().unwrap();
                return c.as_term(None,parser)
            }

            let tag = elem.name.local.to_string();
            let attrs: VecMap<String, String> = elem.attributes.borrow().map.iter().map(|(k, v)| {
                (k.local.to_string(), v.value.clone())
            }).collect();
            let mut terms = Vec::new();
            let mut children = Vec::new();
            let mut nws = data.children.iter();

            for c in self.node.children() {
                if let Some(_) = c.as_comment() {
                    nws.next();
                    // ignore
                } else if let Some(t) = c.as_text() {
                    let t = t.borrow().to_string();
                    let t = t.trim();
                    if t.is_empty() { continue }
                    children.push(InformalChild::Text(t.to_string()))
                } else if let Some(_) = c.as_element() {
                    let nc = nws.next().unwrap();
                    assert_eq!(nc.node, c);
                    let mut rest = { nc.data.borrow_mut().elem.take() };
                    match nc.as_term(Some(&mut rest),parser) {
                        Term::Informal { tag, attributes, children: mut chs, terms: tms } => {
                            let l = terms.len() as u8;
                            terms.extend(tms);
                            for c in chs.iter_mut() {
                                if let Some(mut iter) = c.iter_mut() {
                                    for c in iter {
                                        if let InformalChild::Term(ref mut u) = c {
                                            *u += l
                                        }
                                    }
                                }
                            }
                            children.push(InformalChild::Node {
                                tag,
                                attributes,
                                children: chs
                            })
                        },
                        t => {
                            let l = terms.len() as u8;
                            terms.push(t);
                            children.push(InformalChild::Term(l))
                        }
                    }
                } else {
                    unreachable!("??? {c:?}")
                }
            }

            assert!(nws.next().is_none());
            if tag == "mi" && children.len() == 1 {
                match children.first() {
                    Some(InformalChild::Text(s)) if s.chars().count() == 1 => {
                        let name = parser.resolve_variable(Name::new(s));
                        return Term::OMV(name)
                    }
                    _ => ()
                }
            }

            let tm = Term::Informal {
                tag,attributes:attrs,children,terms
            };
            //println!("Here: {tm:?}");
            tm
        } else if let Some(_) = self.node.as_comment() {
            Term::OMV(VarNameOrURI::Name(Name::new("ERROR")))
        } else {
            todo!("Unknown term node {}",self.node.to_string())
        }
    }
}

pub(crate) struct Narr {
    pub(crate) uri: NarrativeURI,
    pub(crate) children:Vec<DocumentElement>,
    pub(crate) iri:immt_api::core::ontology::rdf::terms::NamedNode,
    pub(crate) vars:Vec<(NarrDeclURI,bool)>
}
impl Narr {
    pub fn new(uri:NarrativeURI) -> Self {
        let iri = uri.to_iri();
        Self {
            uri,children:Vec::new(),iri,vars:Vec::new()
        }
    }
}

pub(crate) struct Content {
    pub(crate) uri: ModuleURI,
    pub(crate) children:Vec<ContentElement>,
    pub(crate) iri:immt_api::core::ontology::rdf::terms::NamedNode
}
impl Content {
    pub fn new(uri:ModuleURI) -> Self {
        let iri = uri.to_iri();
        Self {
            uri,
            children: Vec::new(),
            iri
        }
    }

}

pub struct HTMLParser<'a> {
    pub(crate) narratives:Vec<Narr>,
    pub(crate) contents:Vec<Content>,
    pub(crate) backend:&'a Backend,
    input:&'a str,
    pub(crate) document:NodeWithSource,
    pub(crate) elems:Vec<DocumentElement>,
    notations:Vec<String>,
    pub(crate) strip:bool,
    path:&'a Path,
    pub(crate) in_term:bool,
    pub(crate) in_notation:bool,
    pub(crate) modules:Vec<Module>,
    pub(crate) title:Option<Box<str>>,
    pub(crate) id_counter: usize,
    refs: Vec<u8>,
    body:Option<NodeWithSource>,
    css:Vec<CSS>,
    pub(crate) triples:Vec<Quad>,
}
impl HTMLParser<'_> {
    pub(crate) fn store_resource<T:serde::Serialize>(&mut self,t:&T) -> NarrativeRef<T> {
        let off = self.refs.len();
        struct VecWriter<'a>(&'a mut Vec<u8>);
        impl bincode::enc::write::Writer for VecWriter<'_> {
            fn write(&mut self, bytes: &[u8]) -> Result<(), bincode::error::EncodeError> {
                self.0.extend_from_slice(bytes);
                Ok(())
            }
        }
        bincode::serde::encode_into_writer(t,VecWriter(&mut self.refs),bincode::config::standard()).unwrap();
        let end = self.refs.len();
        NarrativeRef::new(off,end,self.narratives.first().unwrap().uri.doc())
    }
    pub(crate) fn store_node(&mut self,n:&NodeWithSource) -> NarrativeRef<String> {
        self.store_resource(&n.node.to_string())
    }
    fn kill(&mut self) {
        self.document.kill();
        if let Some(b) = self.body.take() {
            b.kill()
        }
    }
}

impl std::fmt::Debug for HTMLParser<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("HTMLParser");
        s.field("in_term",&self.in_term);
        s.field("in_notation",&self.in_term);
        s.field("doc",&self.document.node.to_string());
        s.finish()
    }
}

impl<'a> HTMLParser<'a> {
    pub fn new(input: &'a str, path: &'a Path, uri: DocumentURI, backend:&'a Backend,strip:bool) -> Self {
        use immt_api::core::ontology::rdf::ontologies::*;
        let doc = NodeWithSource::new(NodeRef::new_document(),0,ArrayVec::default());
        let iri = uri.to_iri();
        let triples = vec![
            ulo!((iri.clone()) (dc::LANGUAGE) = (uri.language().to_string()) IN iri.clone()),
            ulo!( (iri.clone()) : DOCUMENT IN iri.clone()),
            ulo!((uri.archive().to_iri()) CONTAINS (iri.clone()) IN iri.clone())
        ];
        HTMLParser {
            backend,
            refs:Vec::new(),
            input,strip,path,narratives:vec![Narr {
                iri, uri:uri.into(), children: Vec::new(),vars:Vec::new()
            }],
            contents:Vec::new(),
            document:doc.into(),
            modules:Vec::new(),
            in_term:false,in_notation:false,
            id_counter:0,
            notations:Vec::new(),elems:Vec::new(),title:None,
            body:None,css:Vec::new(),triples
        }
    }
    pub fn run(mut self) -> FullDocument {
        let doc = self.input;
        let mut p = parse_document(self, ParseOpts::default())
            .from_utf8();
        p.one(doc.as_bytes().to_tendril())
    }

    fn delete(&self,node:&NodeWithSource) {
        let len = node.len();
        {
            if let Some(p) = &node.data.borrow().parent {
                sanity_check!(self,p.len() == p.node.to_string().len());
                node.node.detach();
                p.data.borrow_mut().children.retain(|s| s != node)
            }
        }
        fn r(s:&HTMLParser,n:&NodeWithSource,len:usize) {
            if let Some(p) = &n.data.borrow().parent {
                {p.data.borrow_mut().range.end.offset -= len}
                sanity_check!(s,p.len() == p.node.to_string().len());
                r(s,p,len)
            }
        }
        r(self,node,len);
        node.kill();
    }

}

fn is_shtml(attrs:&Vec<Attribute>) -> bool {
    attrs.iter().any(|a| a.name.local.starts_with("shtml:"))
}


fn brk(s:&HTMLParser) {
    println!("Document offset mismatch!\n\n{}",s.document.node.to_string());
    todo!("Document offset mismatch!")
}

struct Print<'a,A>(&'a A);
impl Display for Print<'_,QualName> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.0.local)
    }
}
impl Display for Print<'_,Vec<Attribute>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for a in self.0.iter() {
            write!(f," {}=\"{}\"",a.name.local,a.value)?
        }
        Ok(())
    }
}
impl Display for Print<'_,NodeWithSource> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.0.node.to_string().replace('\n',""))
    }
}

impl<'a> TreeSink for HTMLParser<'a> {
    type Handle = NodeWithSource;
    type Output = FullDocument;

    #[inline]
    fn finish(mut self) -> Self::Output {
        let bdnode = self.body.as_ref().unwrap_or_else(|| {
            todo!()
        }).data.borrow();
        let start = bdnode.children.first().map(|c| c.data.borrow().range.start.offset).unwrap_or_else(||
            bdnode.range.start.offset
        );
        let end = bdnode.range.end.offset - "</body>".len();
        drop(bdnode);
        let body = SourceRange { start: ByteOffset { offset: start }, end: ByteOffset { offset: end } };
        let html = self.document.node.to_string();
        let Some(Narr {
            uri:NarrativeURI::Doc(uri),
            children,..
        }) = self.narratives.pop() else { unreachable!() };
        self.kill();
        let spec = FullDocument {
            doc:Document {
                uri,
                title:self.title,
                elements:children
            },
            body,html,refs:self.refs,css:self.css,triples:self.triples,
            modules:self.modules
        };
        spec
    }

    #[inline]
    fn parse_error(&mut self, msg: Cow<'static, str>) {
        tracing::error!("{msg}")
    }

    #[inline]
    fn get_document(&mut self) -> Self::Handle {
        self.document.clone()
    }

    #[inline]
    fn set_quirks_mode(&mut self, mode: QuirksMode) {
        let le = self.document.node.to_string().len();
        self.document.node
            .as_document()
            .unwrap()
            ._quirks_mode
            .set(mode);
        sanity_check!(self,self.document.node.to_string().len() == le);
    }

    #[inline]
    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x.node == y.node
    }

    #[inline]
    fn elem_name<'b>(&'b self, target: &'b Self::Handle) -> ExpandedName<'b> {
        target.node.as_element().unwrap().name.expanded()
    }

    #[inline]
    fn create_element(&mut self, name: QualName, mut attrs: Vec<Attribute>, _flags: ElementFlags) -> Self::Handle {
        debug_check!("Creating element <{} {}/>",Print(&name),Print(&attrs));
        let elem = if is_shtml(&attrs) {
            self.do_shtml(&mut attrs)
        } else { ArrayVec::default() };
        let node = NodeRef::new_element(
                name,
                attrs.into_iter().map(|attr| {
                    let Attribute {
                        name: QualName { prefix, ns, local },
                        value,
                    } = attr;
                    let value = String::from(value);
                    (
                        kuchikiki::ExpandedName { ns, local },
                        kuchikiki::Attribute { prefix, value },
                    )
                }),
            );
        let len = node.to_string().len();
        NodeWithSource::new(node,len,elem)
    }

    #[inline]
    fn create_comment(&mut self, text: StrTendril) -> Self::Handle {
        let elem = NodeRef::new_comment(text);
        let len = elem.to_string().len();
        NodeWithSource::new(elem,len,ArrayVec::default())
    }

    #[inline]
    fn create_pi(&mut self, target: StrTendril, data: StrTendril) -> Self::Handle {
        let elem = NodeRef::new_processing_instruction(target,data);
        let len = elem.to_string().len();
        NodeWithSource::new(elem,len,ArrayVec::default())
    }

    #[inline]
    fn append(&mut self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        let new_off = if let Some(d) = parent.node.as_document() {
            parent.end()
        } else if let Some(e) = parent.node.as_element() {
            parent.end() - e.name.local.len() - "</>".len()
        } else {
            todo!()
        };
        let pd = parent.data.borrow();
        let c = pd.children.last().cloned();drop(pd);
        if let Some(c) = c {
            self.pop(&c);
        }
        let len = match child {
            NodeOrText::AppendNode(node) => {
                debug_check!("\nAppending {}\n   to {}\n",Print(&node),Print(parent));
                if let Some(e) = node.node.as_element() {
                    if e.name.local.as_ref() == "body" {
                        self.body = Some(node.clone())
                    }
                    node.data.borrow_mut().elem.retain(|e| e.on_add(self))
                }
                sanity_check!(self,node.data.borrow().range.start.offset == 0);
                let len = node.end();
                node.shift(new_off);
                parent.add(node);
                len
            },
            NodeOrText::AppendText(text) => {
                parent.add_text(text)
            }
        };
        parent.prolong(len);
        sanity_check!(self,parent.len() == parent.node.to_string().len())
    }

    #[inline]
    fn append_before_sibling(&mut self, sibling: &Self::Handle, new_node: NodeOrText<Self::Handle>) {
        todo!()
    }

    #[inline]
    fn append_doctype_to_document(&mut self, name: StrTendril, public_id: StrTendril, system_id: StrTendril) {
        let len = name.len() + "<!DOCTYPE >".len();
        let doctype = NodeRef::new_doctype(name, public_id, system_id);
        self.document.node.append(doctype);
        let oldlen = self.document.len();
        self.document.prolong(len);
        sanity_check!(self,self.document.node.to_string().len() == oldlen + len);
    }

    #[inline]
    fn add_attrs_if_missing(&mut self, target: &Self::Handle, attrs: Vec<Attribute>) {
        todo!()
    }

    #[inline]
    fn remove_from_parent(&mut self, target: &Self::Handle) {
        todo!()
    }

    #[inline]
    fn reparent_children(&mut self, node: &Self::Handle, new_parent: &Self::Handle) {
        todo!()
    }

    #[inline]
    fn mark_script_already_started(&mut self, _node: &Self::Handle) {
        todo!()
    }

    #[inline]
    fn get_template_contents(&mut self, target: &Self::Handle) -> Self::Handle {
        todo!()
    }

    #[inline]
    fn append_based_on_parent_node(&mut self, element: &Self::Handle, prev_element: &Self::Handle, child: NodeOrText<Self::Handle>) {
        todo!()
    }


    fn pop(&mut self, node: &Self::Handle) {
        if node.data.borrow().closed { return }
        let pops = {node.data.borrow().children.iter().filter(|c|
            !c.data.borrow().closed
        ).cloned().collect::<ArrayVec<_,2>>()};
        for e in pops {
            self.pop(&e)
        }
        debug_check!("Closing {}",Print(node));
        let mut d = node.data.borrow_mut();
        let mut elem = d.elem.take();drop(d);
        elem.reverse();
        /*for e in elem.iter() {
            println!("Going to close {e:?}");
        }*/
        let mut delete = false;
        while !elem.is_empty() {
            let h = elem.drain(..1).next().unwrap();
            //println!("Closing {h:?}");
            if !self.close(node,h,&mut elem) && self.strip { delete = true }
        }
        if delete {
            //println!("Deleting {}",node.node.to_string());
            self.delete(node)
        }
        node.data.borrow_mut().closed = true;

        let data = node.data.borrow();
        match data.parent.as_ref().and_then(|n| n.node.as_element()) {
            Some(p) if p.name.local.as_ref() == "head" => {
                if let Some(n) = node.node.as_element() {
                    if n.name.local.as_ref() == "link" &&
                        n.attributes.borrow().map.iter().any(|(k,v)| k.local.as_ref() == "rel" && v.value == "stylesheet") {
                        let href = n.attributes.borrow().map.iter().find(|(k,_)| k.local.as_ref() == "href").unwrap().1.value.clone();
                        self.css.push(CSS::Link(replace_css(href)));
                    }
                }
            }
            _ => ()
        }
    }
}

fn replace_css(s:String) -> Box<str> {
    match s.as_str() {
        "file:///home/jazzpirate/work/Software/sTeX/RusTeXNew/rustex/src/resources/rustex.css" => "/rustex.css".into(),
        _ => s.into()
    }
}