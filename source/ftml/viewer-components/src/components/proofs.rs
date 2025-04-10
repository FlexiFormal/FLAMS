use flams_web_utils::components::{Collapsible, Header};
use leptos::{context::Provider, either::Either, prelude::*};
use leptos_dyn_dom::{DomChildrenCont, DomCont, DomStringCont, OriginalNode};

#[derive(Copy, Clone)]
struct ProofHidable {
    //expanded: RwSignal<bool>,
    body: RwSignal<Option<String>>,
    set: RwSignal<bool>,
}

pub fn proof_hide<V: IntoView + 'static>(
    initial: bool,
    children: impl FnOnce() -> V + Send + 'static,
) -> impl IntoView {
    let expanded = RwSignal::new(!initial);
    let body = RwSignal::new(None);
    let set = RwSignal::new(false);
    let hidable = ProofHidable {
        //expanded,
        body,
        set,
    };

    return view!(<Provider value=Some(hidable)>{children()}</Provider>);
    /*
    view! {
        <Collapsible expanded=expanded >
            <Header slot><Provider value=Some(hidable)>{children()}</Provider></Header>
            {move || if set.get() {
                body.update_untracked(|e| e.take()).map(|html| {
                    tracing::info!("Here 2: {html}");
                    view!(<DomStringCont html cont=crate::iterate />)
                })
            } else { None }}
        </Collapsible>
    }
     */
}

pub fn proof_body(orig: OriginalNode) -> impl IntoView {
    if let Some(Some(ProofHidable { body, set })) = use_context::<Option<ProofHidable>>() {
        //tracing::info!("Here 1:");
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

        let none = None::<ProofHidable>;
        Either::Left(
            view!(<Provider value=none><DomCont orig cont=crate::iterate skip_head=true/></Provider>),
        )
        /*
        body.set(Some(s));
        set.set(true);
        Either::Left(())
        */
    } else {
        let none = None::<ProofHidable>;
        Either::Right(
            view!(<Provider value=none><DomChildrenCont orig cont=crate::iterate /></Provider>),
        )
    }
}
