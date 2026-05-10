#![allow(clippy::must_use_candidate)]

mod r#await;
mod drawer;
mod trees;

mod errors;
pub use errors::*;
use leptos::prelude::*;

mod anchors;
//mod block;
//pub use block::*;
//mod popover;
//pub use popover::*;
mod layout;
pub use layout::*;

pub use anchors::*;
pub use drawer::*;
pub use r#await::*;
pub use trees::*;

pub use ftml_component_utils::Header;
#[leptos::prelude::slot]
pub struct Trigger {
    children: leptos::prelude::Children,
}

#[macro_export]
macro_rules! client_only {
    ($b:expr) => {{
        let sig = ::leptos::prelude::RwSignal::new(false);
        ::leptos::prelude::Effect::new(move || {
            #[cfg(feature = "hydrate")]
            {
                sig.set(true);
            }
        });
        move || if sig.get() { Some($b) } else { None }
    }};
}

#[component]
pub fn ClientOnly(children: Children) -> impl IntoView {
    let children = std::cell::Cell::new(Some(children));
    let sig = RwSignal::new(false);
    let rf = NodeRef::new();
    rf.on_load(move |_| sig.set(true));
    move || {
        if sig.get() {
            leptos::either::Either::Left(children.take().map(|c| c()))
        } else {
            leptos::either::Either::Right(view!(<div node_ref = rf/>))
        }
    }
}
