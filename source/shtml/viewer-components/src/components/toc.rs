use immt_ontology::uris::DocumentElementURI;
use immt_utils::CSS;
use immt_web_utils::{do_css, inject_css};
use leptos::prelude::*;

#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub enum TOCElem {
  Section{
    title:Option<String>,
    uri:DocumentElementURI,
    id:String,
    children:Vec<TOCElem>
  },
  Inputref{
    uri:DocumentElementURI,
    id:String,
    children:Vec<TOCElem>
  }
}
impl TOCElem {
  fn into_view(self,sub:bool) -> impl IntoView {
    use thaw::{NavItem,NavSubItem,NavCategory,NavCategoryItem};
    match self {
      Self::Section{title:Some(title),id,children,..} if children.is_empty() => {
        if sub {
          view!(<NavSubItem value=id><span inner_html=title/></NavSubItem>).into_any()
        } else {
          view!(<NavItem value=id><span inner_html=title/></NavItem>).into_any()
        }
      }
      Self::Section{title:None,children,..} |
        Self::Inputref{children,..} => {
        children.into_iter().map(|e| e.into_view(sub)).collect_view().into_any()
      }
      Self::Section{title:Some(title),id,children,..} => {
        let cls = if sub {"shtml-toc-nested"} else {""};
        view!{
          <span style="--immt-toc-nesting-h:var(--immt-toc-nesting);">
            <NavCategory value=id>
              <NavCategoryItem slot class=cls><span inner_html=title/></NavCategoryItem>
              <div style="--immt-toc-nesting:calc(var(--immt-toc-nesting-h) + 45px);">{children.into_iter().map(|e| e.into_view(true)).collect_view().into_any()}</div>
            </NavCategory>
          </span>
        }.into_any()
      }
    }
  }
}

#[component]
pub fn Toc(toc:RwSignal<Option<(Vec<CSS>,Vec<TOCElem>)>>) -> impl IntoView {
  use thaw::NavDrawer;
  inject_css("shtml-toc", ".shtml-toc-nested {padding-left:var(--immt-toc-nesting) !important;}");
  move || toc.get().map(|(css,toc)| {
    for css in css { do_css(css); }
    leptos::logging::log!("toc: {toc:?}");
    Some(view!{
      <div style="position:fixed;right:20px;z-index:5;--immt-toc-nesting:0px;--immt-toc-nesting-h:0px;"><NavDrawer>{
        toc.into_iter().map(|e| e.into_view(false)).collect_view()
      }</NavDrawer></div>
    })
  })
}