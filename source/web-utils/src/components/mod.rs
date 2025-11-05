#![allow(clippy::must_use_candidate)]

mod r#await;
mod binder;
mod drawer;
mod popover;
mod spinner;
mod trees;

mod errors;
pub use errors::*;
use ftml_dom::utils::css::inject_css;
use leptos::prelude::*;

//#[cfg(any(feature = "ssr", feature = "hydrate"))]
//pub use theming::*;
mod anchors;
mod block;
//#[cfg(any(feature = "ssr", feature = "hydrate"))]
//mod theming;

/*
#[cfg(not(any(feature = "ssr", feature = "hydrate")))]
#[component(transparent)]
pub fn Themer<Ch: IntoView + 'static>(children: TypedChildren<Ch>) -> impl IntoView {
    use thaw::ConfigProvider; //,ToasterProvider,Theme};
    let children = children.into_inner();
    view! {
      <ConfigProvider>
        {children()}
        //<ToasterProvider>{children()}</ToasterProvider>
      </ConfigProvider>
    }
}
*/

pub use anchors::*;
pub use block::*;
pub use drawer::*;
pub use popover::*;
pub use r#await::*;
pub use spinner::*;
pub use trees::*;

#[leptos::prelude::slot]
pub struct Header {
    children: leptos::prelude::Children,
}
#[leptos::prelude::slot]
pub struct Trigger {
    children: leptos::prelude::Children,
}

#[component]
pub fn Collapsible(
    #[prop(optional)] header: Option<Header>,
    children: Children,
    #[prop(optional, into)] expanded: Option<RwSignal<bool>>,
) -> impl IntoView {
    let expanded = expanded.unwrap_or_else(|| RwSignal::new(false));
    view! {<details open=move || expanded.get()>
        <summary on:click=move |_| expanded.update(|b| *b = !*b)>{
            header.map(|c| (c.children)())
        }</summary>
        <div>{children()}</div>
    </details>}
}

#[component]
pub fn LazyCollapsible(
    #[prop(optional)] header: Option<Header>,
    mut children: ChildrenFnMut,
) -> impl IntoView {
    let expanded = RwSignal::new(false);
    view! {<details>
        <summary on:click=move |_| expanded.update(|b| *b = !*b)>{
            header.map(|c| (c.children)())
        }</summary>
        <div>{move || if expanded.get() {
          Some(children())
        } else { None }}</div>
    </details>}
}

#[component]
pub fn Burger(children: Children) -> impl IntoView {
    use icondata_ch::ChMenuHamburger;
    use thaw::{Menu, MenuPosition, MenuTrigger, MenuTriggerType};
    inject_css("burger", include_str!("burger.css"));
    view! {<ClientOnly><div class="ftml-burger-outer"><div class="ftml-burger">
      <Menu on_select=|_:String| () trigger_type=MenuTriggerType::Hover position=MenuPosition::Bottom>
          <MenuTrigger slot><div><thaw::Icon width="2.5em" height="2.5em" icon=ChMenuHamburger/></div></MenuTrigger>
          {children()}
      </Menu>
    </div></div></ClientOnly>}
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
