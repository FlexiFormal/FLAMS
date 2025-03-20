#![allow(clippy::must_use_candidate)]

mod r#await;
mod binder;
mod popover;
mod trees;
mod drawer;
mod spinner;

mod errors;
pub use errors::*;

#[cfg(any(feature="ssr",feature="hydrate"))]
pub use theming::*;
#[cfg(any(feature="ssr",feature="hydrate"))]
mod theming;
mod anchors;
mod block;

#[cfg(not(any(feature="ssr",feature="hydrate")))]
#[component(transparent)]
pub fn Themer<Ch:IntoView+'static>(children:TypedChildren<Ch>) -> impl IntoView {
    use thaw::ConfigProvider;//,ToasterProvider,Theme};
    let children = children.into_inner();
    view!{
      <ConfigProvider>
        {children()}
        //<ToasterProvider>{children()}</ToasterProvider>
      </ConfigProvider>
    }
}


pub use popover::*;
pub use r#await::*;
pub use trees::*;
pub use drawer::*;
pub use anchors::*;
pub use spinner::*;
pub use block::*;

#[leptos::prelude::slot]
pub struct Header { children:leptos::prelude::Children }
#[leptos::prelude::slot]
pub struct Trigger { children:leptos::prelude::Children }

use leptos::prelude::*;

use crate::inject_css;

#[component]
pub fn Collapsible<Ch:IntoView+'static>(
    #[prop(optional)] header:Option<Header>,
    children:TypedChildren<Ch>
) -> impl IntoView {
  let children = children.into_inner();
    let expanded = RwSignal::new(false);
    view!{<details>
        <summary on:click=move |_| expanded.update(|b| *b = !*b)>{
            header.map(|c| (c.children)())
        }</summary>
        <div>{children()}</div>
    </details>}
}

#[component]
pub fn LazyCollapsible<Ch:IntoView+'static>(
    #[prop(optional)] header:Option<Header>,
    children:TypedChildrenMut<Ch>
) -> impl IntoView {
  let mut children = children.into_inner();
    let expanded = RwSignal::new(false);
    view!{<details>
        <summary on:click=move |_| expanded.update(|b| *b = !*b)>{
            header.map(|c| (c.children)())
        }</summary>
        <div>{move || if expanded.get() {
          Some(children())
        } else { None }}</div>
    </details>}
}

#[component]
pub fn Burger<Ch:IntoView+'static>(children:TypedChildren<Ch>) -> impl IntoView {
  use thaw::{Menu,MenuTriggerType,MenuTrigger,MenuPosition};
  use icondata_ch::ChMenuHamburger;
  inject_css("burger",include_str!("burger.css"));
  let children = children.into_inner();
  view!{<ClientOnly><div class="ftml-burger-outer"><div class="ftml-burger">
    <Menu on_select=|_| () trigger_type=MenuTriggerType::Hover position=MenuPosition::FlexibleBottom>
        <MenuTrigger slot><div><thaw::Icon width="2.5em" height="2.5em" icon=ChMenuHamburger/></div></MenuTrigger>
        {children()}
    </Menu>
  </div></div></ClientOnly>}
}

#[component]
pub fn ClientOnly<Ch:IntoView+'static>(children:TypedChildren<Ch>) -> impl IntoView {
  let children = std::cell::Cell::new(Some(children.into_inner()));
  let sig = RwSignal::new(false);
  let rf = NodeRef::new();
  rf.on_load(move |_| sig.set(true));
  move || if sig.get() {
    leptos::either::Either::Left(children.take().map(|c| c()))
  } else { 
    leptos::either::Either::Right(view!(<div node_ref = rf/>)) 
  }
}