use immt_api::narration::document::{Document, DocumentElement, Language, SectionLevel, CSS};
use immt_api::uris::DocumentURI;
use immt_api::utils::iter::VecMap;
use immt_api::utils::sourcerefs::{ByteOffset, SourceRange};
use immt_api::utils::HMap;
use immt_api::{FinalStr, HTMLStr};
use immt_system::utils::parse::{ParseSource, ParseStr, StringOrStr};
use std::fmt::{Display, Write};
use std::path::Path;
use std::str::FromStr;

pub struct HTMLParser<'a> {
    reader: ParseStr<'a, ByteOffset>,
    out: String,
    open_nodes: Vec<OpenNode<'a>>,
    in_body: bool,
    doc: DocumentURI,
    css: Vec<CSS>,
    elements: Vec<DocumentElement>,
    title: Option<String>,
    section_id: usize,
    inputref_id: usize,
    language: Language,
}
impl<'a> HTMLParser<'a> {
    pub fn new(input: &'a str, path: &Path, uri: DocumentURI) -> Self {
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
            title: None,
            section_id: 0,
            inputref_id: 0,
            language,
        }
    }
    fn reset_off(&mut self) {
        self.reader.trim_start();
        self.reader.offset().offset = self.out.len();
    }
    fn get(&self, range: SourceRange<ByteOffset>) -> &str {
        self.out.get(range.start.offset..range.end.offset).unwrap()
    }
    pub fn run(mut self) -> (String, Document) {
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
        (self.out, doc)
    }

    fn do_body(&mut self) {
        self.reader.read_until_str("<body");
        let mut body = self.read_open_node();
        body.tag = Tag::Div;
        self.out.push_str(&body.to_string());
        while !self.reader.starts_with_str("</body>") {
            match self.next_node() {
                Some(n) if !n.tag.allowed_in_body() => todo!("{:?}", n),
                Some(n)
                    if n.attributes
                        .inner
                        .iter()
                        .any(|(k, v)| k.starts_with("shtml:")) =>
                {
                    self.do_shtml(n)
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

    fn do_shtml(&mut self, mut node: OpenNode<'a>) {
        for (i, (k, v)) in node.attributes.inner.iter().enumerate() {
            match *k {
                "shtml:sectionlevel" => {
                    let tagstr = node.tag.as_closing_str();
                    self.reader.read_until_str(tagstr);
                    self.reader.skip(tagstr.len());
                    self.reset_off();
                    self.get_doc_container()
                        .push(DocumentElement::SetSectionLevel(
                            u8::from_str(v).unwrap().try_into().ok().unwrap(), // TODO no unwrap
                        ));
                    return;
                }
                "shtml:inputref" => {
                    let tagstr = node.tag.as_closing_str();
                    self.reader.read_until_str(tagstr);
                    self.reader.skip(tagstr.len());
                    self.reset_off();
                    let start = node.start;
                    let uri = replace_uri(v);
                    let str = format!("<span shtml:inputref=\"{}\"></span>", uri);
                    let end = ByteOffset {
                        offset: start.offset + str.len(),
                    };
                    self.out.push_str(&str);
                    let inputref = DocumentElement::InputRef {
                        id: {
                            let r = format!("ID_{}", self.inputref_id).into();
                            self.inputref_id += 1;
                            r
                        },
                        target: uri,
                        range: SourceRange { start, end },
                    };
                    self.get_doc_container().push(inputref);
                    self.reset_off();
                    return;
                }
                "shtml:doctitle" => {
                    let tagstr = node.tag.as_closing_str();
                    let title = self.reader.read_until_str(tagstr);
                    self.reader.skip(tagstr.len());
                    self.reset_off();
                    self.title = Some(title.into());
                    return;
                }
                "shtml:section" => {
                    let level = u8::from_str(v).unwrap().try_into().ok().unwrap(); // TODO no unwrap
                    node.attributes.inner.remove(i);
                    node.element = Some(OpenDocElem::Section {
                        level,
                        title: None,
                        children: Vec::new(),
                    });
                    self.out.push_str(&node.to_string());
                    self.open_nodes.push(node);
                    self.reset_off();
                    return;
                }
                "shtml:sectiontitle" => {
                    node.attributes.inner.remove(i);
                    node.element = Some(OpenDocElem::SectionTitle {
                        children: Vec::new(),
                    });
                    self.out.push_str(&node.to_string());
                    self.open_nodes.push(node);
                    self.reset_off();
                    return;
                }
                "shtml:visible" => {
                    node.attributes.inner.remove(i);
                    node.element = Some(OpenDocElem::Invisible);
                    return self.do_shtml(node);
                }
                "shtml:language" | "shtml:metatheory" | "shtml:signature"
                    if node
                        .attributes
                        .inner
                        .iter()
                        .any(|(k, _)| *k == "sthml:theory") =>
                {
                    ()
                }
                "shtml:styles" | "shtml:inline" | "shtml:fors" => (),
                _ if k.starts_with("shtml:") => {
                    todo!("{k} = {v}");
                }
                _ => (),
            }
        }
        self.out.push_str(&node.to_string());
        self.open_nodes.push(node);
    }

    fn get_doc_container(&mut self) -> &mut Vec<DocumentElement> {
        for e in self.open_nodes.iter_mut().rev() {
            if let Some(element) = &mut e.element {
                if let Some(children) = element.children() {
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
                        && node.attributes.get(&"rel") == Some(&"stylesheet")
                        && node.attributes.get(&"href").is_some() =>
                {
                    let href = *node.attributes.get(&"href").unwrap();
                    self.css.push(CSS::Link(href.into()));
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
        let mut attributes = VecMap::default();
        while rest.contains('=') {
            let (n, r) = rest.split_once('=').unwrap();
            rest = r;
            let delim = rest.chars().next().unwrap();
            rest = &rest[1..];
            let (v, r) = rest.split_once(delim).unwrap();
            rest = r;
            attributes.insert(n.trim(), v);
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

#[derive(Debug)]
struct OpenNode<'a> {
    tag: Tag,
    attributes: VecMap<&'a str, &'a str>,
    start: ByteOffset,
    str: &'a str,
    element: Option<OpenDocElem>,
}
impl Display for OpenNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('<')?;
        f.write_str(self.tag.as_str())?;
        for (k, v) in &self.attributes.inner {
            write!(f, " {}={}", k, v)?;
        }
        f.write_char('>')
    }
}

#[derive(Debug)]
enum OpenDocElem {
    Section {
        level: SectionLevel,
        title: Option<(String, SourceRange<ByteOffset>)>,
        children: Vec<DocumentElement>,
    },
    SectionTitle {
        children: Vec<DocumentElement>,
    },
    Invisible,
}
impl OpenDocElem {
    fn children(&mut self) -> Option<&mut Vec<DocumentElement>> {
        match self {
            OpenDocElem::Section { children, .. } | OpenDocElem::SectionTitle { children } => {
                Some(children)
            }
            OpenDocElem::Invisible => None,
        }
    }
    fn close(self, p: &mut HTMLParser<'_>, node: OpenNode) -> Option<DocumentElement> {
        match self {
            OpenDocElem::Section {
                level,
                title,
                children,
            } => Some(DocumentElement::Section {
                id: {
                    let ret = format!("ID_{}", p.section_id).into();
                    p.section_id += 1;
                    ret
                },
                range: SourceRange {
                    start: node.start,
                    end: *p.reader.curr_pos(),
                },
                level,
                title,
                children: children.into(),
            }),
            OpenDocElem::Invisible => {
                p.out.truncate(node.start.offset);
                p.reset_off();
                None
            }
            OpenDocElem::SectionTitle { children } => {
                for c in p.open_nodes.iter_mut().rev() {
                    if let Some(OpenDocElem::Section { title, .. }) = &mut c.element {
                        let s = &p.out[node.start.offset..];
                        *title = Some((
                            s.into(),
                            SourceRange {
                                start: node.start,
                                end: *p.reader.curr_pos(),
                            },
                        ));
                        return None;
                    }
                }
                todo!()
            }
        }
    }
}

use const_format::concatcp;

macro_rules! tags {
    ($(
        $(% $hid:ident=$hl:literal )?
        $(%+ $hida:ident=$hla:literal )?
        $(!+ $bida:ident=$bla:literal )?
        $(! $bid:ident=$bl:literal )?
        $(+ $ida:ident=$la:literal )?
        $(* $mid:ident=$ml:literal )?
        $($id:ident=$l:literal )?
    );*) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        enum Tag {
            $( $($hid)? $($hida)? $($bida)? $($bid)? $($ida)? $($id)? $($mid)? ),*
        }
        impl Tag {
            fn auto_closes(self) -> bool {
                use Tag::*;
                tags!(@mat self => $( $($hida;)? $($bida;)? $($ida;)? )* )
            }
            fn allowed_in_head(self) -> bool {
                use Tag::*;
                tags!(@mat self => $( $($hid;)? $($hida;)? )* )
            }
            fn allowed_in_body(self) -> bool {
                use Tag::*;
                tags!(@mat self => $( $($bid;)? $($bida;)? $($mid;)? )* )
            }
            fn is_math(self) -> bool {
                use Tag::*;
                tags!(@mat self => $( $($mid;)? )* )
            }
            fn from_str(str:&str) -> Self {
                use Tag::*;
                match str {
                    $( $($hl => $hid,)? $($hla => $hida,)? $($bla => $bida,)? $($bl => $bid,)? $($la => $ida,)? $($ml => $mid,)? $($l => $id,)? )*
                    _ => panic!("Unknown tag: {}", str),
                }
            }
            fn as_str(self) -> &'static str {
                use Tag::*;
                match self {
                    $( $($hid => $hl,)? $($hida => $hla,)? $($bida => $bla,)? $($bid => $bl,)? $($ida => $la,)? $($mid => $ml,)? $($id => $l,)? )*
                }
            }
            fn as_closing_str(self) -> &'static str {
                use Tag::*;
                match self {
                    $( $($hid => concatcp!("</",$hl,">"),)? $($hida => concatcp!("</",$hla,">"),)? $($bida => concatcp!("</",$bla,">"),)?
                    $($bid => concatcp!("</",$bl,">"),)? $($ida => concatcp!("</",$la,">"),)? $($mid => concatcp!("</",$ml,">"),)? $($id => concatcp!("</",$l,">"),)? )*
                }
            }
        }
    };
    (@mat $s:ident => $($i:ident);*; ) => { matches!($s, $($i)|*) }
}

tags!(
  Head = "head";
    %+Meta = "meta"; %Title = "title"; %+Link = "link";
    Body = "body";
    !Article = "article"; !Div = "div"; !Span = "span"; !+Br = "br"; !+Hr = "hr"; !A = "a";
    !Table = "table"; !Tbody = "tbody"; !Tr = "tr"; !Td = "td";
    *Math = "math"; *Mrow = "mrow"; *Mi = "mi"; *Mo = "mo";
);

// TODO deprecate
fn replace_uri(s: &str) -> DocumentURI {
    let mut s = s.strip_prefix("http://mathhub.info/").unwrap();
    let archive = ARCHIVES
        .iter()
        .find(|a| s.starts_with(**a))
        .map(|a| {
            s = &s.strip_prefix(a).unwrap()[1..];
            *a
        })
        .unwrap();
    let (path, file) = s.rsplit_once('/').unwrap();
    DocumentURI::new_unchecked(format!(
        "http://mathhub.info/:sTeX?a={archive}&D&f={path}&n={file}"
    ))
}

static ARCHIVES: &[&str; 2] = &["HelloWorld/hwexam", "HelloWorld/smglom"];
