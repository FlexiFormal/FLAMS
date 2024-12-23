mod nodes;
pub mod termsnotations;

use std::cell::{Cell, RefCell};

use html5ever::{interface::{NodeOrText, TreeSink}, parse_document, serialize::SerializeOpts, tendril::{SliceExt, StrTendril, TendrilSink}, ParseOpts, QualName};
use immt_ontology::{languages::Language, narration::{documents::UncheckedDocument, LazyDocRef}, triple, uris::{ArchiveId, ArchiveURI, ArchiveURITrait, BaseURI, DocumentURI, ModuleURI, SymbolURI, URIOrRefTrait, URIRefTrait, URIWithLanguage}, DocumentRange};
use immt_system::{backend::{AnyBackend, Backend}, formats::{HTMLData, OMDocResult}};
use immt_utils::{prelude::HSet, CSS};
use nodes::{ElementData, NodeData, NodeRef};
use shtml_extraction::{errors::SHTMLError, open::{terms::{OpenTerm, VarOrSym}, OpenSHTMLElement}, prelude::{all_rules, Attributes, ExtractorState, RuleSet, SHTMLElements, SHTMLNode, SHTMLTag, StatefulExtractor}};

pub struct HTMLParser<'p> {
  document_node:NodeRef,
  rel_path:&'p str,
  extractor:RefCell<Extractor<'p>>,
  body:Cell<(DocumentRange,usize)>
}

struct Extractor<'a> {
  errors:String,
  css:Vec<CSS>,
  refs:Vec<u8>,
  triples:HSet<immt_ontology::rdf::Triple>,
  title:Option<Box<str>>,
  //document:UncheckedDocument,
  backend:&'a AnyBackend,
  state:ExtractorState
}

impl StatefulExtractor for Extractor<'_> {
  type Attr<'a> = nodes::Attributes;
  const RDF:bool=true;

  fn add_resource<T:immt_ontology::Resourcable>(&mut self,t:&T) -> LazyDocRef<T> {
      struct VecWriter<'a>(&'a mut Vec<u8>);
      impl bincode::enc::write::Writer for VecWriter<'_> {
          fn write(&mut self, bytes: &[u8]) -> Result<(), bincode::error::EncodeError> {
              self.0.extend_from_slice(bytes);
              Ok(())
          }
      }
      let off = self.refs.len();
      let _ = bincode::serde::encode_into_writer(t, VecWriter(&mut self.refs), bincode::config::standard());
      LazyDocRef::new(off,self.refs.len(),self.state.document_uri().clone())
  }

  #[inline]
  fn state(&self) -> &ExtractorState {
      &self.state
  }
  #[inline]
  fn state_mut(&mut self) -> &mut ExtractorState {
      &mut self.state
  }
  #[inline]
  fn set_document_title(&mut self,title:Box<str>) {
      self.title = Some(title);
  }

  #[inline]
  fn add_triples<const N:usize>(&mut self, triples:[immt_ontology::rdf::Triple;N]) {
      self.triples.extend(triples);
  }
  #[inline]
  fn add_error(&mut self,err:SHTMLError) {
    self.errors.push_str(&(err.to_string() + "\n"));
  }

/*
  fn resolve_variable_name(&self,_name:&Name) -> Var {todo!()}
  fn in_notation(&self) -> bool {todo!()}
  fn set_in_notation(&mut self,_value:bool) {todo!()}
  fn in_term(&self) -> bool {todo!()}
  fn set_in_term(&mut self,_value:bool) {todo!()}
*/

} 

impl<'p> HTMLParser<'p> {
  pub fn run(input:&str,uri:DocumentURI,rel_path:&'p str,backend:&'p AnyBackend) -> Result<(OMDocResult,String),String> {
    let iri = uri.to_iri();
    let mut triples = HSet::default();
    for t in [
        triple!(<(iri.clone())> dc:LANGUAGE = (uri.language().to_string()) ),
        triple!(<(iri.clone())> : ulo:DOCUMENT),
        triple!(<(uri.archive_uri().to_iri())> ulo:CONTAINS <(iri)>)
    ] {
      triples.insert(t);
    }
    /*
    let document = UncheckedDocument {
      uri,
      title:None,
      elements:Vec::new()
    };*/

    parse_document(Self {
      document_node:NodeRef::new_document(),
      body:Cell::new((DocumentRange{start:0,end:0},0)),
      rel_path,
      extractor:RefCell::new(Extractor {
        backend, triples, //document,
        errors:String::new(),
        title:None,
        css:Vec::new(),
        refs:Vec::new(),
        state: ExtractorState::new(uri)
      })
    }, ParseOpts::default()).from_utf8().one(input.as_bytes().to_tendril())
  }
}

impl TreeSink for HTMLParser<'_> {
  type Handle = NodeRef;
  type Output = Result<(OMDocResult,String),String>;
  type ElemName<'a> = &'a QualName where Self:'a;

  fn finish(self) -> Self::Output {
    for c in self.document_node.children() {
      self.pop(&c);
    }
    let mut html = Vec::new();
    let Extractor {
      errors,css,refs,title,triples,state,backend,..
    } = self.extractor.into_inner();
    if !errors.is_empty() {
      return Err(errors);
      /*
      return BuildResult {
        log:Either::Left(errors),
        result:Err(Vec::new())
      } */
    }
    let Ok((uri,elems,modules)) = state.take() else {
      return Err("Unbalanced sHTML document".to_string())
      /*return BuildResult {
        log:Either::Left("Unbalanced sHTML document".to_string()),
        result:Err(Vec::new())
      }*/
    };
    
    let _ = html5ever::serialize(&mut html, &self.document_node, SerializeOpts::default());
    let html = String::from_utf8_lossy_owned(html);
    backend.submit_triples(&uri,self.rel_path,triples.into_iter());
    let (body,inner_offset) = self.body.get();
    Ok((OMDocResult {
      document: UncheckedDocument {
        uri,title,elements:elems
      },
      html:HTMLData {
        html,css,refs,
        body,inner_offset
      },modules
    },errors))
    /*
    BuildResult {
      log:Either::Left(errors),
      result:Ok(BuildResultArtifact::Data(Box::new(OMDocResult {
        document: UncheckedDocument {
          uri,title,elements:elems
        },
        html,css,refs,modules,
        body,inner_offset
      })))
    } */
  }

  #[inline]
  fn parse_error(&self, msg: std::borrow::Cow<'static, str>) {
      self.extractor.borrow_mut().errors.push_str(&msg);
  }
  #[inline]
  fn get_document(&self) -> Self::Handle {
      self.document_node.clone()
  }
  #[inline]
  fn set_quirks_mode(&self, mode: html5ever::interface::QuirksMode) {
    let NodeData::Document(r) = self.document_node.data() else {unreachable!()};
    r.set(mode);
  }

  #[inline]
  fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
      x == y
  }

  #[inline]
  fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'a> {
      &target.as_element().unwrap_or_else(|| unreachable!()).name
  }

  #[inline]
  fn create_element(
    &self,
    name: QualName,
    attrs: Vec<html5ever::Attribute>,
    _: html5ever::interface::ElementFlags,
  ) -> Self::Handle {
    NodeRef::new_element(name, attrs.into())
  }
  #[inline]
  fn create_comment(&self, text: StrTendril) -> NodeRef {
      NodeRef::new_comment(text)
  }
  #[inline]
  fn create_pi(&self, target: StrTendril, data: StrTendril) -> Self::Handle {
    NodeRef::new_processing_instruction(target, data)
  }

  #[allow(clippy::cast_possible_wrap)]
  fn append(&self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
    if let Some(e) = parent.last_child() {
      self.pop(&e);
    }
    //println!("Current parent: {}: >>>>{}<<<<",parent.len(),parent.string().replace('\n'," ").replace('\t'," "));
    //assert_eq!(parent.len(),parent.string().len());
    match child {
        NodeOrText::AppendNode(child) => {
          if child.as_element().is_some_and(|n| n.name.local.as_ref().eq_ignore_ascii_case("img")) {
            let Some(child_elem) = child.as_element() else {unreachable!()};
            let mut attributes = child_elem.attributes.borrow_mut();
            if let Some(src) = attributes.value("src") {
              let path = std::path::Path::new(src);
              if let Some(newsrc) = self.extractor.borrow().backend.archive_of(path, |a,rp| {
                format!("/img?a={}&rp={}",a.id(),&rp[1..])
              }) {
                attributes.set("src",&newsrc);
              } else {
                let kpsewhich = &*tex_engine::engine::filesystem::kpathsea::KPATHSEA;
                let last = src.rsplit_once('/').map_or(src,|(_,p)| p);
                if let Some(file) = kpsewhich.which(last) {
                  if file == path {
                    let file = format!("/img?kpse={last}");
                    attributes.set("src",&file);
                  }
                } else {
                  let file = format!("/img?file={src}");
                  attributes.set("src",&file);
                }
                // TODO
              };
            }
            drop(attributes);
            NodeRef::update_len(child_elem);
          }
          //println!("Current Child: {}: >>>>{}<<<<",child.len(),child.string().replace('\n'," ").replace('\t'," "));
          //assert_eq!(child.len(),child.string().len());
          if parent.as_document().is_some() {
            if let Some(child_elem) = child.as_element() {
              let new_start = parent.len();
              let len = child.len();
              child_elem.start_offset.set(new_start);
              child_elem.end_offset.set(new_start + len);
            }
          } else if let Some(parent_elem) = parent.as_element() {
            let new_start = parent_elem.end_offset.get() - nodes::tag_len(&parent_elem.name) - 1;
            if let Some(child_elem) = child.as_element() {
              {
                let mut attributes = child_elem.attributes.borrow_mut();
                let mut extractor = self.extractor.borrow_mut();
                if let Some(elements) = all_rules().applicable_rules(&mut *extractor, &mut *attributes) {
                  drop(attributes);
                  update_attributes(&elements,child_elem);
                  child_elem.shtml.set(Some(elements));
                } else {
                  drop(attributes);
                  NodeRef::update_len(child_elem);
                }
              }
              let len = child.len();
              child_elem.start_offset.set(new_start);
              child_elem.end_offset.set(new_start + len);
            }
            //println!("Updated Child: {}: >>>>{}<<<<",child.len(),child.string().replace('\n'," ").replace('\t'," "));
            //assert_eq!(child.len(),child.string().len());
            prolong(parent,child.len() as isize);
          }
          parent.append(child);
          //println!("New parent: {}: >>>>{}<<<<",parent.len(),parent.string().replace('\n'," ").replace('\t'," "));
          //assert_eq!(parent.len(),parent.string().len());
        },
        NodeOrText::AppendText(text) => {
          if let Some(elem) = parent.as_element() {
            let len = if matches!(&*elem.name.local,
              "style" | "script" | "xmp" | "iframe" | "noembed" | "noframes" | "plaintext" | "noscript"
            ) { text.as_bytes().len() } else { nodes::escaped_len(&text, false) };
            prolong(parent,len as isize);
          }
          if let Some(last_child) = parent.last_child() {
              if let Some(existing) = last_child.as_text() {
                  existing.borrow_mut().extend(text.chars());
                  return;
              }
          }
          parent.append(NodeRef::new_text(text));
          //assert_eq!(parent.len(),parent.string().len());
        }
    }
  }

  #[inline]
  fn append_doctype_to_document(
      &self,
      name: StrTendril,
      public_id: StrTendril,
      system_id: StrTendril,
  ) {
      self.document_node.append(NodeRef::new_doctype(name, public_id, system_id));
  }

  #[inline]
  fn append_based_on_parent_node(
      &self,
      element: &Self::Handle,
      prev_element: &Self::Handle,
      child: NodeOrText<Self::Handle>,
  ) {  
    if element.parent().is_some() {
        self.append_before_sibling(element, child);
    } else {
        self.append(prev_element, child);
    }
  }

  fn pop(&self, node: &Self::Handle) {
      let Some(elem) = node.as_element() else {return};
      if elem.closed.get() {return}
      elem.closed.set(true);
      for c in node.children() { self.pop(&c) }
      if &elem.name.local == "body" {
        let range = DocumentRange{start:elem.start_offset.get(),end:elem.end_offset.get()};
        let off = elem.attributes.borrow().len();
        self.body.set((range,"<body>".len() + off));
      } else if matches!(&*elem.name.local,"link"|"style") {
        if let Some(p) = node.parent() {
          if let Some(pe) = p.as_element() {
            if &pe.name.local == "head" {
              match &*elem.name.local {
                "link" => {
                  let attrs = elem.attributes.borrow();
                  if attrs.value("rel") == Some("stylesheet") {
                    if let Some(lnk) = attrs.value("href") {
                      self.extractor.borrow_mut().css.push(CSS::Link(lnk.into()));
                      node.delete();
                      return
                    }
                  }
                }
                "style" => {
                  let str = node.children().map(|c| c.string()).collect::<String>();
                  self.extractor.borrow_mut().css.push(CSS::Inline(str.into()));
                  node.delete();
                  return
                }
                _ => unreachable!()
              }
            }
          }
        }
      }
      if let Some(mut elems) = elem.shtml.take() {
        let mut extractor = self.extractor.borrow_mut();
        elems.close(&mut *extractor,node);
        if !elems.is_empty() {
          elem.shtml.set(Some(elems));
        }
      }
  }

  #[inline]
  fn append_before_sibling(&self, _sibling: &Self::Handle, _child: NodeOrText<Self::Handle>) {  
    unreachable!()
    /*
    match child {
      NodeOrText::AppendNode(node) => sibling.insert_before(node),
      NodeOrText::AppendText(text) => {
          if let Some(previous_sibling) = sibling.previous_sibling() {
              if let Some(existing) = previous_sibling.as_text() {
                  existing.borrow_mut().extend(text.chars());
                  return;
              }
          }
          sibling.insert_before(NodeRef::new_text(text));
      }
    }
     */
  }

  #[inline]
  fn remove_from_parent(&self, _target: &Self::Handle) {
    unreachable!()
  }
  #[inline]
  fn reparent_children(&self, _node: &Self::Handle, _new_parent: &Self::Handle) {
    unreachable!()
  }
  #[inline]
  fn mark_script_already_started(&self, _node: &Self::Handle) {
    unreachable!()
  }
  fn get_template_contents(&self, _target: &Self::Handle) -> Self::Handle {
    unreachable!()
  }
  #[inline]
  fn add_attrs_if_missing(&self, _target: &Self::Handle, _attrs: Vec<html5ever::Attribute>) {
    unreachable!()
  }

}

fn update_attributes(elements:&SHTMLElements,child:&ElementData) {
  let mut attrs = child.attributes.borrow_mut();
  for e in &elements.elems { match e {
    OpenSHTMLElement::ImportModule(uri) =>
      attrs.update(SHTMLTag::ImportModule, uri),
    OpenSHTMLElement::UseModule(uri) =>
      attrs.update(SHTMLTag::UseModule, uri),
    OpenSHTMLElement::MathStructure { uri, .. } => {
      attrs.update(SHTMLTag::MathStructure, &uri.clone().into_module());
    }
    OpenSHTMLElement::Morphism { uri,domain,..} => {
      attrs.update(SHTMLTag::MorphismDomain, domain);
      if let Some(uri) = uri {
        attrs.update(SHTMLTag::Morphism, &uri.clone().into_module());
      }
    }
    OpenSHTMLElement::Assign(uri) => {
      attrs.update(SHTMLTag::Assign, uri);
    }
    // Paragraphs: fors-list
    OpenSHTMLElement::Symdecl{uri,..} => {
      attrs.update(SHTMLTag::Symdecl, uri);
    }
    OpenSHTMLElement::Notation{symbol:VarOrSym::S(uri),..} => {
      attrs.update(SHTMLTag::Notation, uri);
    }
    OpenSHTMLElement::Definiendum(uri) => {
      attrs.update(SHTMLTag::Definiendum, uri);
    }
    OpenSHTMLElement::Conclusion{uri,..} => {
      attrs.update(SHTMLTag::Conclusion, uri);
    }
    OpenSHTMLElement::Definiens{uri:Some(uri),..} => {
      attrs.update(SHTMLTag::Definiens, uri);
    }
    OpenSHTMLElement::Inputref{uri,..} => {
      attrs.update(SHTMLTag::InputRef, uri);
    }
    OpenSHTMLElement::OpenTerm{term:
      OpenTerm::Symref{uri,..} |
      OpenTerm::OMA{head:VarOrSym::S(uri),..} | 
      OpenTerm::Complex(VarOrSym::S(uri),..),..
    } => 
      attrs.update(SHTMLTag::Head, uri),
    _ => ()
  }}
  drop(attrs);
  NodeRef::update_len(child);
}

#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_wrap)]
fn prolong(parent:&NodeRef,len:isize) {
  if let Some(elem) = parent.as_element() {
    let end = elem.end_offset.get();
    elem.end_offset.set(((end as isize) + len) as usize);
    if let Some(p) = parent.parent() {
      prolong(&p,len);
    }
  }
}