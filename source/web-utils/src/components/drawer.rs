use leptos::prelude::*;
use crate::inject_css;

use super::{Header,Trigger};

#[component]
pub fn Drawer(
    lazy:bool,
    trigger:Trigger,
    #[prop(optional)] header:Option<Header>,
    mut children:ChildrenFnMut
) -> impl IntoView {
  use thaw::{Button,ButtonAppearance,DrawerSize,DrawerBody,OverlayDrawer,DrawerHeaderTitle,DrawerHeader,DrawerPosition,DrawerHeaderTitleAction};
  inject_css("immt-drawer", ".immt-wide-drawer { z-index:5; --thaw-drawer--size:80vw !important; }");
  let open = RwSignal::new(false);
  view!{
    <span on:click=move |_| open.set(true)>{(trigger.children)()}</span>
    <OverlayDrawer class="immt-wide-drawer" open position=DrawerPosition::Right>
      <DrawerHeader><DrawerHeaderTitle>
        <DrawerHeaderTitleAction slot>
          <Button
            appearance=ButtonAppearance::Subtle
            on_click=move |_| open.set(false)>
            "x"
          </Button>
        </DrawerHeaderTitleAction>
        {header.map(|h| (h.children)())}
      </DrawerHeaderTitle></DrawerHeader>
      <DrawerBody>{move ||
        if lazy || open.get() { children().into_any()}
        else {"".into_any()}
      }</DrawerBody>
    </OverlayDrawer>
  }
}