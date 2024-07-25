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
use immt_api::core::narration::{Document, DocumentElement, HTMLDocSpec, Language};
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
    pub(crate) data:std::rc::Rc<std::cell::RefCell<NodeData>>,
    count:usize
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
    fn new(node:NodeRef,len:usize,elem:OpenElems,count:usize) -> Self {
        NodeWithSource {
            node,
            data:std::rc::Rc::new(std::cell::RefCell::new(NodeData {
                range: SourceRange { start: ByteOffset { offset: 0 }, end: ByteOffset { offset: len } },
                parent:None,
                children: Vec::new(),
                elem,closed:false
            })),count
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
        //println!("HERE: {}",self.node.to_string());
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
        let elem = self.node.as_element().unwrap();
        let tag = elem.name.local.to_string();
        let attrs : VecMap<String,String> = elem.attributes.borrow().map.iter().map(|(k,v)| {
            (k.local.to_string(),v.value.clone())
        }).collect();
        let mut terms = Vec::new();
        let mut children = Vec::new();
        let _dt = self.data.borrow();
        let mut nws = _dt.children.iter();

        for c in self.node.children() {
            if let Some(t) = c.as_text() {
                let t = t.borrow().to_string();
                let t = t.trim();
                if t.is_empty() { continue }
                children.push(InformalChild::Text(t.to_string()))
            } else if let Some(_) = c.as_element() {
                let c = nws.next().unwrap();
                let mut rest = { c.data.borrow_mut().elem.take() };
                match c.as_term(Some(&mut rest)) {
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
                unreachable!()
            }
        }

        assert!(nws.next().is_none());

        let tm = Term::Informal {
            tag,attributes:attrs,children,terms
        };
        //println!("Here: {tm:?}");
        tm



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
    counter:usize,
    head:Option<NodeWithSource>,
    body:Option<NodeWithSource>
    // This should really be an HMap, but NodeRef doesn't implement Hash
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
        if let Some(h) = self.head.take() {
            h.kill()
        }
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
        let doc = NodeWithSource::new(NodeRef::new_document(),0,ArrayVec::default(),0);
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
            head:None,
            counter:1,
            body:None
        }
    }
    pub fn run(mut self) -> (HTMLDocSpec, Vec<Module>) {
        let doc = self.input;
        let mut p = parse_document(self, ParseOpts::default())
            .from_utf8();
        p.one(doc.as_bytes().to_tendril())
    }
    fn count(&mut self) -> usize {
        self.counter += 1;self.counter -1
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
        let head = self.head.as_ref().unwrap_or_else(|| {
            todo!()
        }).data.borrow().range;
        let body = self.body.as_ref().unwrap_or_else(|| {
            todo!()
        }).data.borrow().range;
        let html = self.document.node.to_string();
        self.kill();
        let spec = HTMLDocSpec {
            doc:Document {
                language:self.language,
                uri:self.uri,
                title:self.title,
                elements:self.elems
            },
            head,body,html,
            notations:self.notations
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
        NodeWithSource::new(node,len,elem,self.count())
    }

    #[inline]
    fn create_comment(&mut self, text: StrTendril) -> Self::Handle {
        let elem = NodeRef::new_comment(text);
        let len = elem.to_string().len();
        NodeWithSource::new(elem,len,ArrayVec::default(),self.count())
    }

    #[inline]
    fn create_pi(&mut self, target: StrTendril, data: StrTendril) -> Self::Handle {
        let elem = NodeRef::new_processing_instruction(target,data);
        let len = elem.to_string().len();
        NodeWithSource::new(elem,len,ArrayVec::default(),self.count())
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
                    if e.name.local.as_ref() == "head" {
                        self.head = Some(node.clone())
                    } else if e.name.local.as_ref() == "body" {
                        self.body = Some(node.clone())
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
        if {node.data.borrow().closed} { return }
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
            match &h {
                OpenElem::TopLevelTerm(_) | OpenElem::Conclusion{..}| OpenElem::Type{..} | OpenElem::Definiens{..} | OpenElem::Rule {..}
                    => self.in_term = false,
                OpenElem::Notation {..} | OpenElem::VarNotation {..} => self.in_notation = false,
                _ => ()
            }

            if !self.close(node,h,&mut elem) && self.strip { delete = true }
        }
        if delete { self.delete(node) }
        node.data.borrow_mut().closed = true
    }
}


/*
use std::borrow::Cow;
use std::path::Path;
use immt_api::backend::manager::ArchiveManager;
use immt_api::core::content::Module;
use immt_api::core::narration::{CSS, Document, DocumentElement, Language};
use immt_api::core::uris::documents::DocumentURI;
use immt_api::core::utils::parse::{ParseStr,ParseSource};
use immt_api::core::utils::sourcerefs::{ByteOffset, SourceRange};
use immt_api::core::utils::VecMap;
use crate::parsing::{OpenNode, Tag};

pub struct HTMLParser<'a> {
    pub(crate) backend:&'a ArchiveManager,
    pub(crate) reader: ParseStr<'a, ByteOffset>,
    pub(crate) out: String,
    pub(crate) open_nodes: Vec<OpenNode<'a>>,
    pub(crate) in_body: bool,
    pub(crate) doc: DocumentURI,
    pub(crate) css: Vec<CSS>,
    pub(crate) elements: Vec<DocumentElement>,
    pub(crate) title: Option<String>,
    pub(crate) section_id: usize,
    pub(crate) inputref_id: usize,
    pub(crate) paragraph_id: usize,
    pub(crate) language: Language,
    pub(crate) modules:Vec<Module>,
    pub(crate) in_term:bool,
    pub(crate) strip:bool
}

impl<'a> HTMLParser<'a> {
    pub fn new(input: &'a str, path: &Path, uri: DocumentURI, backend:&'a ArchiveManager,strip:bool) -> Self {
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let language = if stem.ends_with(".en") {
            Language::English
        } else if stem.ends_with(".de") {
            Language::German
        } else if stem.ends_with(".fr") {
            Language::French
        } else {
            Language::English
        };
        HTMLParser {
            reader: ParseStr::new(input),
            out: String::with_capacity(input.len()),
            open_nodes: Vec::new(),
            in_body: false,
            doc: uri,
            css: Vec::new(),
            elements: Vec::new(),
            modules: Vec::new(),
            title: None,
            section_id: 0,
            inputref_id: 0,
            paragraph_id: 0,
            in_term:false,
            language,backend,
            strip
        }
    }
    pub(crate) fn reset_off(&mut self) {
        self.reader.trim_start();
        self.reader.offset().offset = self.out.len();
    }
    fn get(&self, range: SourceRange<ByteOffset>) -> &str {
        self.out.get(range.start.offset..range.end.offset).unwrap()
    }
    pub fn run(mut self) -> (String, Document, Vec<Module>) {
        self.reader.trim_start();
        self.reader.offset().offset = 0;
        if self.reader.starts_with_str("<!DOCTYPE") {
            let dt = self.reader.read_until(|c| c == '>');
            //self.out.push_str(dt);
            //self.out.push_str(">\n");
            self.reader.pop_head();
            self.reset_off();
        }
        if self.reader.rest().contains("<head") {
            self.do_head();
        }
        self.do_body();
        let doc = Document {
            uri: self.doc,
            title: self.title.unwrap_or_else(|| "Untitled".into()),
            css: self.css.into(),
            elements: self.elements.into(),
            language: self.language,
        };
        (self.out, doc, self.modules)
    }

    fn is_shtml(node: &OpenNode) -> bool {
        node.attributes.iter().any(|(k, _)| k.starts_with("shtml:"))
    }

    fn do_body(&mut self) {
        self.reader.read_until_str("<body");
        let mut body = self.read_open_node();
        body.tag = Tag::Div;
        self.out.push_str(&body.to_string());
        while !self.reader.starts_with_str("</body>") {
            match self.next_node() {
                Some(n) if !n.tag.allowed_in_body() => todo!("{:?}", n),
                Some(n) if Self::is_shtml(&n) =>
                    self.do_shtml(n),
                Some(n) if n.tag == Tag::Img => {
                    todo!("Image encoding");
                    /*
                    let src = n.attributes.get(&"src").unwrap(); // TODO no unwrap here
                    if let Some((a,d,s)) = self.backend.find_archive(Path::new(src.as_ref())) {
                        //self.out.push_str(&format!("<img src=\"/img?a={}&path={d}/{s}\"",urlencoding::encode(&a.to_string())));
                        for (k,v) in n.attributes.iter() {
                            if *k != "src" { self.out.push_str(&format!(" {k}=\"{v}\"")); }
                        }
                        self.out.push('>');
                        self.reset_off();
                    } else {
                        todo!()
                    }

                     */
                }
                Some(n) => {
                    self.out.push_str(n.str);
                    self.open_nodes.push(n);
                }
                None if self.reader.starts_with_str("</") => self.close_node(),
                None => todo!(),
            }
            self.skip_until_node();
        }
        if self.reader.starts_with_str("</body>") {
            self.reader.skip(7);
            self.out.push_str("</div>");
        } else {
            todo!()
        }
    }

    pub(crate) fn get_doc_container(&mut self) -> &mut Vec<DocumentElement> {
        for e in self.open_nodes.iter_mut().rev() {
            if let Some(element) = &mut e.element {
                if let Some(children) = element.doc_children() {
                    return children;
                }
            }
        }
        &mut self.elements
    }

    fn do_head(&mut self) {
        self.reader.read_until_str("<head");
        //self.out.push_str(self.reader.read_until_str("<head"));
        self.read_open_node();
        //self.out.push_str(head.str);
        // self.open_nodes.push(node);
        while !self.reader.starts_with_str("</head>") {
            self.reader.read_until(|c| c == '<');
            match self.next_node() {
                Some(node)
                if node.tag == Tag::Link
                    && node.attributes.get(&"rel").is_some_and(|s| s == "stylesheet")
                    && node.attributes.get(&"href").is_some() =>
                    {
                        let href = node.attributes.get(&"href").unwrap();
                        self.css.push(CSS::Link(href.as_ref().into()));
                        //self.out.push_str(node.str);
                    }
                Some(node) if node.tag.auto_closes() => {
                    //self.out.push_str(node.str);
                }
                Some(node) if node.tag == Tag::Title => self.skip_node(node),
                Some(node) => {
                    //self.out.push_str(node.str);
                    self.open_nodes.push(node);
                }
                None => break,
            }
        }
        if self.reader.starts_with_str("</head>") {
            self.reader.skip(7);
            //self.out.push_str("</head>");
            self.reset_off();
        } else {
            todo!()
        }
    }

    fn close_node(&mut self) {
        let r = self.reader.read_until_inclusive(|c| c == '>');
        let tag = Tag::from_str(&r[2..r.len() - 1]);
        if let Some(mut node) = self.open_nodes.pop() {
            if node.tag != tag {
                todo!()
            }
            self.out.push_str(r);
            if let Some(e) = std::mem::take(&mut node.element) {
                if let Some(ret) = e.close(self, node) {
                    self.get_doc_container().push(ret);
                }
            }
        } else {
            todo!()
        }
    }

    fn skip_node(&mut self, open_node: OpenNode) {
        self.reader.read_until_str(open_node.tag.as_closing_str());
        self.reader.skip(open_node.tag.as_closing_str().len());
        self.reset_off();
    }

    fn next_node(&mut self) -> Option<OpenNode<'a>> {
        self.skip_until_node();
        if self.reader.starts_with_str("</") {
            None
        } else {
            Some(self.read_open_node())
        }
    }
    fn skip_until_node(&mut self) {
        self.out.push_str(self.reader.read_until_str("<"));
    }

    fn read_open_node(&mut self) -> OpenNode<'a> {
        let start = *self.reader.curr_pos();
        let ret = self.reader.read_until_inclusive(|c| c == '>');
        let mut rest = &ret[1..ret.len() - 1];
        let (tag, r) = rest.split_once(' ').unwrap_or((rest, ""));
        rest = r;
        let tag = Tag::from_str(tag);
        let mut attributes: VecMap<&'a str,Cow<'a, str>> = VecMap::default();
        while rest.contains('=') {
            let (n, r) = rest.split_once('=').unwrap();
            rest = r;
            let delim = rest.chars().next().unwrap();
            rest = &rest[1..];
            let (v, r) = rest.split_once(delim).unwrap();
            rest = r;
            attributes.insert(n.trim(), v.into());
        }
        OpenNode {
            tag,
            attributes,
            start,
            str: ret,
            element: None,
        }
    }
}

 */