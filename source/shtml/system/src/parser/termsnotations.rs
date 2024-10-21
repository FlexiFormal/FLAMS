use either::Either;
use html5ever::serialize::{HtmlSerializer, SerializeOpts, Serializer, TraversalScope};
use immt_ontology::{content::terms::{ArgMode, Informal, Term}, narration::notations::{NotationComponent, OpNotation}};
use shtml_extraction::{open::OpenSHTMLElement, prelude::{NotationSpec, SHTMLNode}};

use crate::parser::nodes::ElementData;

use super::nodes::NodeRef;

impl NodeRef {
  pub(super) fn do_notation(&self) -> NotationSpec {
    self.as_element().map_or_else(
      || {
        let mut ret = "<span>".to_string();
        ret.push_str(&self.string());
        ret.push_str("</span>");
        NotationSpec {attribute_index:5,components:vec![NotationComponent::S(ret.into())].into_boxed_slice(),is_text:true}
    }, |n| {
      let (is_text,attribute_index) = get_is_text_and_offset(n);
      let mut ret = Vec::new();
      let mut strng = Vec::new();
      let _ = rec(self,&mut ret,&mut strng);
      if !strng.is_empty() {
        ret.push(NotationComponent::S(String::from_utf8_lossy(&strng).into_owned().into_boxed_str()));
      }
      NotationSpec {attribute_index,components:ret.into_boxed_slice(),is_text}
    })
  }

  pub(super) fn do_op_notation(&self) -> OpNotation {
    self.as_element().map_or_else(
      || todo!("should be impossible"),
      |n| {
        let (is_text,attribute_index) = get_is_text_and_offset(n);
        let s = self.string();
        OpNotation {attribute_index,text:s.into(),is_text}
    })
  }

  #[allow(clippy::cast_possible_truncation)]
  pub(super) fn do_term(&self) -> Term {
    if let Some(elem) = self.as_element() {
      if let Some(mut shtml) = elem.shtml.take() {
        if let Some(i) = shtml.iter().position(|e| matches!(e,OpenSHTMLElement::ClosedTerm(_))) {
          let OpenSHTMLElement::ClosedTerm(t) = shtml.elems.remove(i) else {unreachable!()};
          return t
        }
        elem.shtml.set(Some(shtml));
      }
      if self.children().count() == 1 && self.first_child().is_some_and(|e| e.as_element().is_some()) {
        return self.first_child().unwrap_or_else(|| unreachable!()).do_term()
      }
      let tag = elem.name.local.to_string();
      let attrs = elem.attributes.borrow().0.iter().map(|(k,v)| 
        (k.local.to_string().into_boxed_str(),v.to_string().into_boxed_str()) 
      ).collect::<Vec<_>>().into_boxed_slice();
      let mut terms = Vec::new();
      let mut children = Vec::new();
      for c in self.children() {
        if let Some(t) = c.as_text() {
          let t = t.borrow();
          let t = t.trim();
          if t.is_empty() { continue }
          children.push(Informal::Text(t.to_string().into_boxed_str()));
        } else if c.as_element().is_some() {
          let l = terms.len() as u8;
          match c.as_term() {
            Term::Informal { tag, attributes, children:mut chs, terms:tms } => {
              terms.extend(tms.into_vec().into_iter());
              for c in &mut chs {
                if let Some(iter) = c.iter_mut_opt() {
                  for c in iter {
                    if let Informal::Term(ref mut u) = c {
                      *u += l;
                    }
                  }
                }
              }
              children.push(Informal::Node {
                tag,attributes,children:chs
              });
            }
            t => {
              terms.push(t);
              children.push(Informal::Term(l));
            }
          }
        }
      }
      Term::Informal {
        tag,attributes:attrs,children:children.into_boxed_slice(),terms:terms.into_boxed_slice()
      }
    } else {
      unreachable!("This should not happen")
    }
  }
}

#[allow(clippy::too_many_lines)]
fn rec(node:&NodeRef,ret:&mut Vec<NotationComponent>,currstr:&mut Vec<u8>) -> (u8,ArgMode) {
  let mut index = 0;
  let tp = ArgMode::Normal;
  let mut ser = HtmlSerializer::new(currstr, SerializeOpts {
    traversal_scope:TraversalScope::IncludeNode,
    ..Default::default()
  });
  if let Some(elem) = node.as_element() {
    if let Some(s) = elem.shtml.take() {
      for e in &s.elems {match e {
        OpenSHTMLElement::Comp => {
          if !ser.writer.is_empty() {
            ret.push(NotationComponent::S(
              String::from_utf8_lossy(&std::mem::take(ser.writer)).into_owned().into_boxed_str()
            ));
          }
          ret.push(NotationComponent::Comp(node.string().into_boxed_str()));
          return (index,tp)
        }
        OpenSHTMLElement::MainComp => {
          if !ser.writer.is_empty() {
            ret.push(NotationComponent::S(
              String::from_utf8_lossy(&std::mem::take(ser.writer)).into_owned().into_boxed_str()
            ));
          }
          ret.push(NotationComponent::MainComp(node.string().into_boxed_str()));
          return (index,tp)
        }
        OpenSHTMLElement::Arg(arg) => {
          if !ser.writer.is_empty() {
            ret.push(NotationComponent::S(
              String::from_utf8_lossy(&std::mem::take(ser.writer)).into_owned().into_boxed_str()
            ));
          }
          index = match arg.index {
            Either::Left(u) | Either::Right((u,_)) => u
          };
          ret.push(NotationComponent::Arg(index, arg.mode));
          return (index,arg.mode)
        }
        OpenSHTMLElement::ArgSep => {
          if !ser.writer.is_empty() {
            ret.push(NotationComponent::S(
              String::from_utf8_lossy(&std::mem::take(ser.writer)).into_owned().into_boxed_str()
            ));
          }
          let mut separator = Vec::new();
          let mut nret = Vec::new();
          let mut idx = 0;
          let mut new_mode = ArgMode::Sequence;
          for c in node.children() {
            let (r,m) = rec(&c,&mut nret,&mut separator);
            if r != 0 {
              idx = r; new_mode = m;
            }
          }
          if !separator.is_empty() {
            nret.push(NotationComponent::S(
              String::from_utf8_lossy(&separator).into_owned().into_boxed_str()
            ));
          }
          ret.push(NotationComponent::ArgSep{index:idx,tp:new_mode,sep:nret.into_boxed_slice()});
          return (index,tp)
        }
        OpenSHTMLElement::ArgMap => {
          if !ser.writer.is_empty() {
            ret.push(NotationComponent::S(
              String::from_utf8_lossy(&std::mem::take(ser.writer)).into_owned().into_boxed_str()
            ));
          }
          let mut separator = Vec::new();
          let mut nret = Vec::new();
          let mut idx = 0;
          //let mut new_mode = ArgMode::Sequence;
          for c in node.children() {
            let (r,_) = rec(&c,&mut nret,&mut separator);
            if r != 0 {
              idx = r; //new_mode = m;
            }
          }
          if !separator.is_empty() {
            nret.push(NotationComponent::S(
              String::from_utf8_lossy(&separator).into_owned().into_boxed_str()
            ));
          }
          ret.push(NotationComponent::ArgMap{index:idx,segments:nret.into_boxed_slice()});
          return (index,tp)
        }
        // TODO ArgMapSep
        _ => {}
      }}
    }
    let attrs = elem.attributes.borrow();
    let _ = ser.start_elem(elem.name.clone(),attrs.0.iter().map(|(name, value)| (name, &**value)));
    drop(attrs);
    for c in node.children() {
      if let Some(t) = c.as_text() {
        let t = t.borrow();
        let _ = ser.write_text(&t);
      } else if c.as_element().is_some() {
        let _ = rec(&c,ret,ser.writer);
      }
    }
    let _ = ser.end_elem(elem.name.clone());
  } else if let Some(t) = node.as_text() {
    let t = t.borrow();
    let _ = ser.write_text(&t);
  } 
  (index,tp)
}

#[allow(clippy::cast_possible_truncation)]
fn get_is_text_and_offset(e:&ElementData) -> (bool,u8) {
  match e.name.local.as_ref() {
      s@ ("span"|"div") => (true,s.len() as u8 + 1),
      s => (false,s.len() as u8 + 1)
  }
}