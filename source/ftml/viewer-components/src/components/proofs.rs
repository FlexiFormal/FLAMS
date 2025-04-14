use flams_ontology::{narration::paragraphs::ParagraphKind, uris::DocumentElementURI};
use leptos::{context::Provider, either::Either, prelude::*};
use leptos_dyn_dom::{DomCont, OriginalNode};

use crate::ts::FragmentContinuation;

#[derive(Copy, Clone, Default)]
struct Elem(RwSignal<Option<std::sync::Mutex<Option<OriginalNode>>>>);
impl Elem {
    fn set(&self, node: OriginalNode) {
        self.0.set(Some(std::sync::Mutex::new(Some(node))))
    }
    fn get(&self) -> Option<OriginalNode> {
        self.0.with(|v| {
            v.as_ref()
                .and_then(|l| l.lock().ok().and_then(|mut v| (&mut *v).take()))
        })
    }
}

#[derive(Copy, Clone)]
struct ProofOrSubproof {
    sub: bool,
    expanded: RwSignal<bool>,
    title: Elem,
    body: Elem,
}

pub fn proof<V: IntoView + 'static>(
    uri: DocumentElementURI,
    initial: bool,
    children: impl FnOnce() -> V + Send + 'static,
) -> impl IntoView {
    let value = ProofOrSubproof {
        sub: false,
        expanded: RwSignal::new(!initial),
        title: Elem::default(),
        body: Elem::default(),
    };
    let display = Memo::new(move |_| {
        if value.expanded.get() {
            "display:contents;"
        } else {
            "display:none;"
        }
    });
    let cls = Memo::new(move |_| {
        if value.expanded.get() {
            "ftml-proof-title ftml-proof-title-expanded"
        } else {
            "ftml-proof-title ftml-proof-title-collapsed"
        }
    });
    FragmentContinuation::wrap(
        &(uri, ParagraphKind::Proof.into()),
        view! {
          <Provider value=Some(value)>
            {children()}
          </Provider>
          {move || value.title.get().map(|html| {
              view!(<div class=cls on:click=move |_| value.expanded.update(|b| *b = !*b)>
                  <DomCont orig=html cont=crate::iterate skip_head=true />
              </div>)
          })}
          {move || value.body.get().map(|html|
              view!{<div style=display><DomCont orig=html cont=crate::iterate skip_head=true /></div>}
          )}
        },
    )
}

pub fn subproof<V: IntoView + 'static>(
    uri: DocumentElementURI,
    initial: bool,
    children: impl FnOnce() -> V + Send + 'static,
) -> impl IntoView {
    let value = ProofOrSubproof {
        sub: true,
        expanded: RwSignal::new(!initial),
        title: Elem::default(),
        body: Elem::default(),
    };
    let display = Memo::new(move |_| {
        if value.expanded.get() {
            "display:contents;"
        } else {
            "display:none;"
        }
    });
    let cls = Memo::new(move |_| {
        if value.expanded.get() {
            "ftml-subproof-title ftml-proof-title-expanded"
        } else {
            "ftml-subproof-title ftml-proof-title-collapsed"
        }
    });
    FragmentContinuation::wrap(
        &(uri, ParagraphKind::Proof.into()),
        view! {
          <Provider value=Some(value)>
            {children()}
          </Provider>
          {move || value.title.get().map(|html| {
              view!(<div class=cls on:click=move |_| value.expanded.update(|b| *b = !*b)>
                  <DomCont orig=html cont=crate::iterate skip_head=true />
              </div>)
          })}
          {move || value.body.get().map(|html|
              view!{<div style=display><DomCont orig=html cont=crate::iterate skip_head=true /></div>}
          )}
        },
    )
}

pub fn proof_title(orig: OriginalNode) -> impl IntoView {
    if let Some(Some(ProofOrSubproof {
        title, sub: false, ..
    })) = use_context::<Option<ProofOrSubproof>>()
    {
        /*
        #[cfg(any(feature = "csr", feature = "hydrate"))]
        let s = {
            let e = &*orig;
            //leptos::web_sys::console::log_1(e);
            let s = e.inner_html();
            //tracing::info!("HTML: {s}");
            s
        };
        #[cfg(not(any(feature = "csr", feature = "hydrate")))]
        let s = String::new(); */
        title.set(orig);
        Either::Left(())
    } else {
        Either::Right(view! {
        <div class="ftml-proof-title"><DomCont orig cont=crate::iterate skip_head=true/></div>
        })
    }
}

pub fn subproof_title(orig: OriginalNode) -> impl IntoView {
    if let Some(Some(ProofOrSubproof {
        title, sub: true, ..
    })) = use_context::<Option<ProofOrSubproof>>()
    {
        /*
        #[cfg(any(feature = "csr", feature = "hydrate"))]
        let s = {
            let e = &*orig;
            //leptos::web_sys::console::log_1(e);
            let s = e.inner_html();
            //tracing::info!("HTML: {s}");
            s
        };
        #[cfg(not(any(feature = "csr", feature = "hydrate")))]
        let s = String::new();
         */
        title.set(orig);
        Either::Left(())
    } else {
        Either::Right(view! {
        <div class="ftml-subproof-title"><DomCont orig cont=crate::iterate skip_head=true/></div>
        })
    }
}

pub fn proof_body(orig: OriginalNode) -> impl IntoView {
    if let Some(Some(ProofOrSubproof { body, .. })) = use_context::<Option<ProofOrSubproof>>() {
        /*tracing::info!("Here 1:");
        #[cfg(any(feature = "csr", feature = "hydrate"))]
        let s = {
            let e = &*orig;
            //leptos::web_sys::console::log_1(e);
            let s = e.inner_html();
            //tracing::info!("HTML: {s}");
            s
        };
        #[cfg(not(any(feature = "csr", feature = "hydrate")))]
        let s = String::new();
        */
        body.set(orig);
        Either::Left(())
    } else {
        Either::Right(view!(<DomCont orig cont=crate::iterate skip_head=true/>))
    }
}
