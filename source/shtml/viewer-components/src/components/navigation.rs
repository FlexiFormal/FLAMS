use immt_ontology::uris::DocumentURI;
use immt_utils::prelude::HMap;
use leptos::prelude::*;
use web_sys::Element;


#[derive(Debug,Clone)]
pub enum SectionOrInputref {
    Section,
    Inputref(RwSignal<bool>,RwSignal<bool>)
}

pub struct NavElems {
    pub ids: HMap<String,SectionOrInputref>,
    pub titles: HMap<DocumentURI,RwSignal<String>>
}
impl NavElems {
    pub fn get_title(&mut self,uri:DocumentURI) -> RwSignal<String> {
        match self.titles.entry(uri) {
            std::collections::hash_map::Entry::Occupied(e) => *e.get(),
            std::collections::hash_map::Entry::Vacant(e) => { 
                let name = e.key().name().to_string();
                *e.insert(RwSignal::new(name))
            }
        }
    }
    pub fn set_title(&mut self,uri:DocumentURI,title:String) {
        match self.titles.entry(uri) {
            std::collections::hash_map::Entry::Occupied(e) => e.get().set(title),
            std::collections::hash_map::Entry::Vacant(e) => { e.insert(RwSignal::new(title));}
        }
    }
    pub fn update_untracked<R>(f:impl FnOnce(&mut Self) -> R) -> R {
        expect_context::<RwSignal<Self>>().update_untracked(f)
    }
    pub fn with_untracked<R>(f:impl FnOnce(&Self) -> R) -> R {
        expect_context::<RwSignal<Self>>().try_with_untracked(f).expect("this should not happen")
    }
    /*pub fn update<R>(f:impl FnOnce(&mut Self) -> R) -> R {
        expect_context::<RwSignal<Self>>().try_update(f).expect("this should not happen")
    }
    pub fn with<R>(f:impl FnOnce(&Self) -> R) -> R {
        expect_context::<RwSignal<Self>>().with(f)
    }*/
    pub fn navigate_to(&self,id:&str) {
        #[cfg(any(feature="csr",feature="hydrate"))]
        {
            tracing::trace!("Looking for #{id}");
            let mut curr = id;
            loop {
                match self.ids.get(curr) {
                    None => (),
                    Some(SectionOrInputref::Section) => {
                        tracing::trace!("Navigating to #{curr}");
                        if let Some(e) = document().get_element_by_id(curr) {
                            let options = leptos::web_sys::ScrollIntoViewOptions::new();
                            options.set_behavior(leptos::web_sys::ScrollBehavior::Smooth);
                            options.set_block(leptos::web_sys::ScrollLogicalPosition::Start);
                            e.scroll_into_view_with_scroll_into_view_options(&options);
                        }
                        return
                    },
                    Some(SectionOrInputref::Inputref(s1,s2)) => {
                        if !s2.get_untracked() {
                            s1.set(true);
                            if s2.get() {
                                return self.navigate_to(id);
                            }
                        }
                    }
                }
                if let Some((a,_)) = curr.rsplit_once('/') {
                    curr = a;
                } else {return}
            }
        }
    }
}


fn get_anchor(e:Element) -> Option<Element> {
  let mut curr = e;
  loop {
      if curr.tag_name().to_uppercase() == "A" {
          return Some(curr)
      }
      if curr.tag_name().to_uppercase() == "BODY" {
          return None
      }
      if let Some(parent) = curr.parent_element() {
          curr = parent;
      } else {
          return None
      }
  }
}

#[derive(Copy,Clone,Debug)]
pub struct URLFragment(RwSignal<String>);
impl URLFragment {
  pub fn new() -> Self {

      use leptos::wasm_bindgen::JsCast;
        #[cfg(any(feature="csr",feature="hydrate"))]
      let signal = RwSignal::new(
          window().location().hash().ok().map(|s| 
              s.strip_prefix('#').map(ToString::to_string).unwrap_or(s)
          ).unwrap_or_default()
      );
      #[cfg(not(any(feature="csr",feature="hydrate")))]
      let signal = RwSignal::new(String::new());

      #[cfg(any(feature="csr",feature="hydrate"))]
      {
        let on_hash_change = wasm_bindgen::prelude::Closure::wrap(Box::new(move |_e: leptos::web_sys::Event| {
            let s = window().location().hash().ok().map(|s| 
                s.strip_prefix('#').map(ToString::to_string).unwrap_or(s)
            ).unwrap_or_default();
            tracing::trace!("Updating URL fragment to {s}");
            signal.set(s);
        }) as Box<dyn FnMut(_)>);

        let on_anchor_click = wasm_bindgen::prelude::Closure::wrap(Box::new(move |e: leptos::web_sys::MouseEvent| {
            if let Some(e) = e.target().and_then(|t| t.dyn_into::<Element>().ok()) {
                if let Some(e) = get_anchor(e) {
                    if let Some(href) = e.get_attribute("href") {
                        if let Some(href) =  href.strip_prefix('#') {
                            tracing::trace!("Updating URL fragment as {href}");
                            signal.set(href.to_string());
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        let _ = window().add_event_listener_with_callback("hashchange", on_hash_change.as_ref().unchecked_ref());
        let _ = window().add_event_listener_with_callback("popstate", on_hash_change.as_ref().unchecked_ref());
        let _ = window().add_event_listener_with_callback("click", on_anchor_click.as_ref().unchecked_ref());
        on_hash_change.forget();
        on_anchor_click.forget();
        }
      Self(signal)
  }
}

#[component]
pub fn Nav(on_load:Option<RwSignal<bool>>) -> impl IntoView {
  let fragment = expect_context::<URLFragment>();
  move || {
    tracing::trace!("Checking URL fragment");
    match on_load {
        None => {
            let s = fragment.0.get();
            if !s.is_empty() {
                NavElems::with_untracked(|e| e.navigate_to(&s));
            }
        }
        Some(signal) => if signal.get() {
            let s = fragment.0.get();
            if !s.is_empty() {
                NavElems::with_untracked(|e| e.navigate_to(&s));
            }
        }
    }
    ""
  }
}