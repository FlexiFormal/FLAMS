use flams_ontology::uris::NarrativeURITrait;
use flams_utils::{vecmap::VecSet, CSS};
use ftml_viewer_components::components::TOCElem;

#[cfg(feature="ssr")]
pub async fn from_document(doc:&flams_ontology::narration::documents::Document) -> (Vec<CSS>,Vec<TOCElem>) {
  use flams_ontology::narration::{sections::Section, DocumentElement, NarrationTrait};
  use flams_system::backend::Backend;
  let mut curr = doc.children().iter();
  let mut prefix = String::new();
  let mut stack = Vec::new();
  let mut ret = Vec::new();
  let mut css = VecSet::new();
  loop {
    while let Some(elem) = curr.next() {
      match elem {
        DocumentElement::Section(Section{ uri, title,children,.. }) => {
          let old = std::mem::replace(&mut curr, children.iter());
          let title = if let Some(title) = title {
            if let Some((c,h)) = super::backend!(get_html_fragment(uri.document(), *title)) {
              for c in c {css.insert(c);}
              Some(h)
            } else {None}
          } else {None};
          stack.push((old,Some(TOCElem::Section{
            title, // TODO
            id: prefix.clone(),
            uri: uri.clone(),
            children: std::mem::take(&mut ret)
          })));
          prefix = if prefix.is_empty() {uri.name().last_name().to_string()} else { format!("{prefix}/{}",uri.name().last_name()) };
        }
        DocumentElement::DocumentReference { id, target,.. } if target.is_resolved() => {
          let Some(d) = target.get() else { unreachable!() };
          let title = d.title().map(ToString::to_string);
          let uri = d.uri().clone();
          let old = std::mem::replace(&mut curr, d.children().iter());
          stack.push((old,Some(TOCElem::Inputref{
            id: prefix.clone(),
            uri,title,
            children: std::mem::take(&mut ret)
          })));
          prefix = if prefix.is_empty() {id.name().last_name().to_string()} else { format!("{prefix}/{}",id.name().last_name()) };
        }
        DocumentElement::Module {  children,.. } |
        DocumentElement::Morphism {children,..} |
        DocumentElement::MathStructure {children,..} => {
          let old = std::mem::replace(&mut curr, children.iter());
          stack.push((old,None));
        }
        _ => ()
    }}
    match stack.pop() {
      None => break,
      Some((iter,Some(TOCElem::Inputref{mut id,uri,title,mut children}))) => {
        curr = iter;
        std::mem::swap(&mut prefix,&mut id);
        std::mem::swap(&mut ret,&mut children);
        if !children.is_empty() {
          ret.push(TOCElem::Inputref{id,uri,title,children});
        }
      }
      Some((iter,Some(TOCElem::Section{mut id,uri,title,mut children}))) => {
        curr = iter;
        std::mem::swap(&mut prefix,&mut id);
        std::mem::swap(&mut ret,&mut children);
        if title.is_some() || !children.is_empty() {
          ret.push(TOCElem::Section{id,uri,title,children});
        }
      }
      Some((iter,None)) => curr = iter
    }
  }
  (super::insert_base_url(css.0),ret)
}