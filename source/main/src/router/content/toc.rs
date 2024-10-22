use immt_ontology::uris::NarrativeURITrait;
use immt_utils::{vecmap::VecSet, CSS};
use shtml_viewer_components::components::TOCElem;

#[cfg(feature="ssr")]
pub async fn from_document(doc:&immt_ontology::narration::documents::Document) -> (Vec<CSS>,Vec<TOCElem>) {
  use immt_ontology::narration::{DocumentElement,sections::Section};
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
            if let Some((c,h)) = immt_system::backend::GlobalBackend::get().get_html_fragment_async(uri.document(), *title).await {
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
        DocumentElement::DocumentReference { id, target:Ok(d),.. } => {
          let old = std::mem::replace(&mut curr, d.children().iter());
          stack.push((old,Some(TOCElem::Inputref{
            id: prefix.clone(),
            uri: id.clone(),
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
      Some((iter,Some(TOCElem::Inputref{mut id,uri,mut children}))) => {
        curr = iter;
        std::mem::swap(&mut prefix,&mut id);
        std::mem::swap(&mut ret,&mut children);
        if !children.is_empty() {
          ret.push(TOCElem::Inputref{id,uri,children});
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
  (css.0,ret)
}