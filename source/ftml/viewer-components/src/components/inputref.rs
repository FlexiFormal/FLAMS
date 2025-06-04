use flams_ontology::{
    narration::sections::SectionLevel,
    uris::{DocumentURI, NarrativeURI},
};
use flams_web_utils::{do_css, inject_css};
use leptos::{context::Provider, either::Either, prelude::*};
use leptos_posthoc::{DomChildrenCont, OriginalNode};

use crate::{
    components::{
        counters::{LogicalLevel, SectionCounters},
        navigation::{NavElems, SectionOrInputref},
    },
    config::IdPrefix,
    extractor::DOMExtractor,
};

#[derive(Copy, Clone)]
pub struct InInputRef(pub bool);

#[component]
pub fn InputRef<'a>(uri: DocumentURI, id: &'a str) -> impl IntoView {
    inputref(uri, id)
}

#[allow(clippy::similar_names)]
pub(super) fn inputref(uri: DocumentURI, id: &str) -> impl IntoView {
    use leptos_posthoc::DomStringCont;
    //leptos::logging::log!("inputref");
    inject_css("ftml-inputref", include_str!("./inputref.css"));
    let replace = RwSignal::new(false);
    let replaced = RwSignal::new(false);
    let on_click = move |_| {
        replace.set(true);
    };
    let id = expect_context::<IdPrefix>().new_id(id);
    let title = NavElems::update_untracked(|ne| {
        ne.ids
            .insert(id.clone(), SectionOrInputref::Inputref(replace, replaced));
        ne.get_title(uri.clone())
    });
    let ctrs = SectionCounters::inputref(uri.clone(), id.to_string());
    match ctrs.current_level() {
        LogicalLevel::Section(lvl) if lvl < SectionLevel::Section => (),
        _ => replace.set(true),
    }
    if !replace.get_untracked() {
        let uri = uri.clone();
        let f = move || {
            NavElems::update_untracked(|ne| {
                if ne.initialized.get() {
                    if ne
                        .get_title(uri.clone())
                        .with_untracked(|ttl| *ttl == uri.name().to_string())
                    {
                        replace.set(true);
                    }
                }
            });
        };
        if NavElems::with_untracked(|e| !e.initialized.get_untracked()) {
            let _ = Effect::new(f);
        } else {
            f();
        }
    }

    view! {
      <Provider value=InInputRef(true)><Provider value=IdPrefix(id.clone())><Provider value=ctrs> {
        move || if replace.get() { Either::Left(do_inputref(uri.clone(),replaced)) } else {
          Either::Right(view!(<div id=id.clone() on:click=on_click class="ftml-inputref">{
            move || {
              let title = title.get();
              if title.is_empty() { None } else {
                Some(view!(<DomStringCont html=title cont=crate::iterate/>))
              }
            }
          }</div>))
      }}</Provider></Provider></Provider>
    }
}

fn do_inputref(uri: DocumentURI, on_load: RwSignal<bool>) -> impl IntoView {
    use flams_web_utils::components::wait_local;
    use leptos_posthoc::DomStringCont;
    let uricl = uri.clone();
    wait_local(
        move || {
            let uri = uri.clone();
            async move { crate::remote::server_config.inputref(uri).await.ok() }
        },
        move |(_, css, html)| {
            for c in css {
                do_css(c);
            }
            view!(<span style="display:contents">
      <Provider value=NarrativeURI::Document(uricl.clone())>
      <Provider value = RwSignal::new(DOMExtractor::default())>
        <DomStringCont html cont=crate::iterate on_load/>
      </Provider></Provider>
      </span>)
        },
        "Error loading document reference".to_string(),
    )
}

#[component]
pub fn IfInputref<Ch: IntoView + 'static>(
    value: bool,
    children: TypedChildren<Ch>,
) -> impl IntoView {
    let children = children.into_inner();
    let in_inputref = use_context::<InInputRef>().map(|i| i.0).unwrap_or(false);
    if in_inputref == value {
        Either::Left(children())
    } else {
        Either::Right(view! {<span data-if-inputref="false"/>})
    }
}

pub(super) fn if_inputref(val: bool, orig: OriginalNode) -> impl IntoView {
    let in_inputref = use_context::<InInputRef>().map(|i| i.0).unwrap_or(false);
    if in_inputref == val {
        Either::Left(view! {<span style="display:contents">
          <DomChildrenCont orig cont=crate::iterate/>
        </span>})
    } else {
        Either::Right(view! {<span data-if-inputref="false"/>})
    }
}
