use either::Either;
use html5ever::{interface::ElemName, serialize::{HtmlSerializer, SerializeOpts, Serializer, TraversalScope}};
use flams_ontology::{content::terms::{ArgMode, Informal, Term}, narration::notations::{NotationComponent, OpNotation}};
use ftml_extraction::{open::OpenFTMLElement, prelude::{NotationSpec, FTMLNode}};

use crate::parser::nodes::ElementData;

use super::nodes::NodeRef;

impl NodeRef {
  pub(super) fn do_notation(&self) -> NotationSpec {
    self.as_element().map_or_else(
      || {
        let mut ret = "<span>".to_string();
        ret.push_str(&self.string());
        ret.push_str("</span>");
        NotationSpec {attribute_index:5,inner_index:6,components:vec![NotationComponent::S(ret.into())].into_boxed_slice(),is_text:true}
    }, |n| {
      let (is_text,attribute_index,inner_index) = get_is_text_and_offsets(n);
      let mut ret = Vec::new();
      let mut strng = Vec::new();
      let _ = rec(self,&mut ret,&mut strng);
      if !strng.is_empty() {
        ret.push(NotationComponent::S(String::from_utf8_lossy(&strng).into_owned().into_boxed_str()));
      }
      NotationSpec {attribute_index,inner_index,components:ret.into_boxed_slice(),is_text}
    })
  }

  pub(super) fn do_op_notation(&self) -> OpNotation {
    self.as_element().map_or_else(
      || todo!("should be impossible"),
      |n| {
        let (is_text,attribute_index,inner_index) = get_is_text_and_offsets(n);
        let s = self.string();
        OpNotation {attribute_index,inner_index,text:s.into(),is_text}
    })
  }

  #[allow(clippy::cast_possible_truncation)]
  pub(super) fn do_term(&self) -> Term {
    if let Some(elem) = self.as_element() {
      if let Some(mut ftml) = elem.ftml.take() {
        if let Some(i) = ftml.iter().position(|e| matches!(e,OpenFTMLElement::ClosedTerm(_))) {
          let OpenFTMLElement::ClosedTerm(t) = ftml.elems.remove(i) else {unreachable!()};
          return t
        }
        elem.ftml.set(Some(ftml));
      }
      /*if self.children().count() == 1 && self.first_child().is_some_and(|e| e.as_element().is_some()) {
        return self.first_child().unwrap_or_else(|| unreachable!()).do_term()
      }*/
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
    if let Some(s) = elem.ftml.take() {
      for e in &s.elems {match e {
        OpenFTMLElement::Comp => {
          if !ser.writer.is_empty() {
            ret.push(NotationComponent::S(
              String::from_utf8_lossy(&std::mem::take(ser.writer)).into_owned().into_boxed_str()
            ));
          }
          ret.push(NotationComponent::Comp(node.string().into_boxed_str()));
          return (index,tp)
        }
        OpenFTMLElement::MainComp => {
          if !ser.writer.is_empty() {
            ret.push(NotationComponent::S(
              String::from_utf8_lossy(&std::mem::take(ser.writer)).into_owned().into_boxed_str()
            ));
          }
          ret.push(NotationComponent::MainComp(node.string().into_boxed_str()));
          return (index,tp)
        }
        OpenFTMLElement::Arg(arg) => {
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
        OpenFTMLElement::ArgSep => {
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
          ret.push(NotationComponent::ArgSep{index:idx,mode:new_mode,sep:nret.into_boxed_slice()});
          return (index,tp)
        }
        OpenFTMLElement::ArgMap => {
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
fn get_is_text_and_offsets(e:&ElementData) -> (bool,u8,u16) {
  let (t,o) = match e.name.local.as_ref() {
      s@ ("span"|"div") => (true,s.len() as u8 + 1),
      s => (false,s.len() as u8 + 1)
  };
  let i = e.attributes.borrow().len() as u16 + (o as u16 + 1);
  (t,o,i)
}

pub(super) fn filter_node_term(mut node:NodeRef) -> NodeRef {
  //println!("Here: {}",node.string());
  'outer: while let Some(e) = node.as_element() {
      /*println!("Checking: {e:?}\nChildren:");
      for c in node.children() {
        println!("  - {}",c.string());
      }*/
      if let Some(a) = e.ftml.take() {
        if a.iter().any(|e| matches!(e,OpenFTMLElement::ClosedTerm(_))) {
          e.ftml.set(Some(a));
          return node;
        } else {
          e.ftml.set(Some(a));
        }
      }
      let num_children = node.children().filter(
          |n| n.as_element().is_some() || n.as_text().is_some_and(|t| !t.borrow().trim().is_empty())
      ).count();
      if matches!(e.name.local.as_ref(),"math") && num_children == 1 {
        if let Some(n) = node.children().find(|n| n.as_element().is_some()) {
          node = n;
          continue
        }
      }
      if matches!(e.name.local.as_ref(),"mrow") && num_children == 1 {
        if let Some(n) = node.children().find(|n| n.as_element().is_some()) {
          node = n;
          continue
        }
      }
      if matches!(e.name.local.as_ref(),"span"|"div") && num_children == 1 {
          if let Some(n) = node.children().find(|n| n.as_element().is_some()) {
            for (k,v) in &e.attributes.borrow().0.0 {
              let k = k.local_name().as_ref();
              let v = &**v;
              if (k == "class" && v != "rustex_contents") || k == "style" {
                break 'outer;
              }
            }
            node = n;
            continue
          }
      }
      break
  }
  node
}