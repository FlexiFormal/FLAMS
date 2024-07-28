use std::borrow::Cow;
use std::cmp::PartialEq;
use std::fmt::Formatter;
use std::path::Path;
use html5ever::{Attribute, ExpandedName, parse_document, ParseOpts, QualName};
use immt_api::backend::manager::ArchiveManager;
use html5ever::tokenizer::*;
use html5ever::interface::{ElementFlags, NextParserState, NodeOrText, QuirksMode, TreeSink};
use html5ever::tendril::StrTendril;
use immt_api::core::content::{ArrayVec, InformalChild, Module, Term};
use immt_api::core::narration::{CSS, Document, DocumentElement, HTMLDocSpec, Language};
use immt_api::core::uris::documents::DocumentURI;
use kuchikiki::NodeRef;
use tendril::{SliceExt, TendrilSink};
use immt_api::core::utils::sourcerefs::{ByteOffset, SourceRange};
use immt_api::core::utils::VecMap;
use crate::docs::OpenTerm;
use crate::parsing::shtml::OpenElem;

pub(crate) type OpenElems = ArrayVec<OpenElem,8>;

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
    pub(crate) fn as_term(&self,rest:Option<&mut OpenElems>) -> Term {
        //println!("Term: {}",self.node.to_string());
        if let Some(rest) = rest {for (i,e) in rest.iter().enumerate() {
            //println!("  - {e:?}");
            match e {
                OpenElem::Term{tm:OpenTerm::Complex(None)} => (),
                OpenElem::Term{..} => {
                    if let OpenElem::Term{tm} = rest.remove(i) {
                        return tm.close()
                    } else {unreachable!()}
                }
                _ => ()
        }}}
        let mut data = self.data.borrow_mut();
        for (i,e) in data.elem.iter().enumerate() {
            //println!("  - {e:?}");
            match e {
                OpenElem::Term{tm:OpenTerm::Complex(None)} => (),
                OpenElem::Term{..} => {
                    if let OpenElem::Term{tm} = data.elem.remove(i) {
                        return tm.close()
                    } else {unreachable!()}
                }
                _ => ()
            }}
        drop(data);
        if let Some(elem) = self.node.as_element() {
            let data = self.data.borrow();
            if data.children.len() == 1 {
                let c = data.children.first().unwrap();
                return c.as_term(None)
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
                    match nc.as_term(Some(&mut rest)) {
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

            let tm = Term::Informal {
                tag,attributes:attrs,children,terms
            };
            //println!("Here: {tm:?}");
            tm
        } else if let Some(_) = self.node.as_comment() {
            Term::OMV("ERROR".to_string())
        } else {
            todo!("Unknown term node {}",self.node.to_string())
        }



        /*
        if self.data.borrow().children.is_empty() {
            if self.node.children().count() == 1 {
                if let Some(s) = self.node.children().last().unwrap().as_text() {
                    let s = s.borrow().to_string();
                    if s.chars().count() == 1 { // TODO: number literals
                        return Term::OMV(s.chars().next().unwrap().to_string())
                    }
                }
            }
        }
        todo!()

         */
    }
}

pub struct HTMLParser<'a> {
    pub(crate) backend:&'a ArchiveManager,
    input:&'a str,
    pub(crate) document:NodeWithSource,
    pub(crate) elems:Vec<DocumentElement>,
    notations:Vec<String>,
    pub(crate) strip:bool,
    path:&'a Path,
    uri:DocumentURI,
    pub(crate) in_term:bool,
    pub(crate) in_notation:bool,
    pub(crate) modules:Vec<Module>,
    pub(crate) title:String,
    pub(crate) id_counter: usize,
    pub(crate) language: Language,
    refs: String,
    body:Option<NodeWithSource>,
    css:Vec<CSS>
}
impl HTMLParser<'_> {
    pub(crate) fn store_string(&mut self,s:&str) -> SourceRange<ByteOffset> {
        let off = self.refs.len();
        let end = off + s.len();
        self.refs.push_str(s);
        SourceRange { start: ByteOffset { offset: off }, end: ByteOffset { offset: end } }
    }
    pub(crate) fn store_node(&mut self,n:&NodeWithSource) -> SourceRange<ByteOffset> {
        let off = self.refs.len();
        let end = off + n.len();
        self.refs.push_str(&n.node.to_string());
        SourceRange { start: ByteOffset { offset: off }, end: ByteOffset { offset: end } }
    }
    fn kill(&mut self) {
        self.document.kill();
        if let Some(b) = self.body.take() {
            b.kill()
        }
    }
}
/*
impl Drop for HTMLParser<'_> {
    fn drop(&mut self) {
        self.kill()
    }
}*/
impl std::fmt::Debug for HTMLParser<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("HTMLParser");
        s.field("in_term",&self.in_term);
        s.field("in_notation",&self.in_term);
        s.field("doc",&self.document.node.to_string());
        s.finish()
    }
}

macro_rules! sanity_check {
    ($s:expr,$a:expr) => {
        ()//if !$a { brk($s) }
    }
}


impl<'a> HTMLParser<'a> {
    pub fn new(input: &'a str, path: &'a Path, uri: DocumentURI, backend:&'a ArchiveManager,strip:bool) -> Self {
        let doc = NodeWithSource::new(NodeRef::new_document(),0,ArrayVec::default());
        HTMLParser {
            backend,
            refs:String::new(),
            input,strip,path,uri,
            document:doc.into(),
            modules:Vec::new(),
            in_term:false,in_notation:false,
            id_counter:0,
            notations:Vec::new(),elems:Vec::new(),title:String::new(),
            language:Language::from_file(path),
            body:None,css:Vec::new()
        }
    }
    pub fn run(mut self) -> (HTMLDocSpec, Vec<Module>) {
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
/*
struct Tokenizer<'a> {
    inner:html5ever::tokenizer::Tokenizer<HTMLParser<'a>>
}

impl TokenSink for Tokenizer {
    type Handle = ();
    fn process_token(&mut self, token: Token, line_number: u64) -> TokenSinkResult<Self::Handle> {
        todo!()
    }
}

 */

fn is_html(attrs:&Vec<Attribute>) -> bool {
    attrs.iter().any(|a| a.name.local.starts_with("shtml:"))
}


fn brk(s:&HTMLParser) {
    println!("Document offset mismatch!\n\n{}",s.document.node.to_string());
    todo!("Document offset mismatch!")
}

impl<'a> TreeSink for HTMLParser<'a> {
    type Handle = NodeWithSource;
    type Output = (HTMLDocSpec,Vec<Module>);

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
        self.kill();
        let spec = HTMLDocSpec {
            doc:Document {
                language:self.language,
                uri:self.uri,
                title:self.title,
                elements:self.elems
            },
            body,html,refs:self.refs,css:self.css
        };
        (spec,self.modules)
    }

    #[inline]
    fn parse_error(&mut self, msg: Cow<'static, str>) {
        todo!()
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
        let elem = if is_html(&attrs) {
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
                if let Some(e) = node.node.as_element() {
                    if e.name.local.as_ref() == "body" {
                        self.body = Some(node.clone())
                    }
                    for e in &mut node.data.borrow_mut().elem {
                        e.on_add(self);
                    }
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

fn replace_css(s:String) -> String {
    match s.as_str() {
        "file:///home/jazzpirate/work/Software/sTeX/RusTeXNew/rustex/src/resources/rustex.css" => "/rustex.css".to_string(),
        _ => s
    }
}