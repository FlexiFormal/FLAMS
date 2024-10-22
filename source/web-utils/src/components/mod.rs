#![allow(clippy::must_use_candidate)]

mod r#await;
mod binder;
mod popover;
mod trees;
mod drawer;

#[cfg(feature="hydrate")]
mod errors;
#[cfg(feature="hydrate")]
pub use errors::*;

#[cfg(any(feature="ssr",feature="hydrate"))]
pub use theming::*;
#[cfg(any(feature="ssr",feature="hydrate"))]
mod theming;
mod anchors;

pub use popover::*;
pub use r#await::*;
pub use trees::*;
pub use drawer::*;
pub use anchors::*;



#[leptos::prelude::slot]
pub struct Header { children:leptos::prelude::Children }
#[leptos::prelude::slot]
pub struct Trigger { children:leptos::prelude::Children }

use leptos::prelude::*;

#[component]
pub fn Collapsible(
    #[prop(optional)] lazy:bool,
    #[prop(optional)] header:Option<Header>,
    mut children:ChildrenFnMut
) -> impl IntoView {
    let expanded = RwSignal::new(false);
    view!{<details>
        <summary on:click=move |_| expanded.update(|b| *b = !*b)>{
            header.map_or_else(|| view!(<span/>).into_any(),|c| (c.children)())
        }</summary>
        <div>{
            if lazy { (move || if expanded.get() {
                Some(children())
                } else { None }
            ).into_any()} else {children().into_any()}
        }</div>
    </details>}
}