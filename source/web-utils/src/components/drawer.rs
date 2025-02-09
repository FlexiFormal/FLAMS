use leptos::prelude::*;
use crate::inject_css;

use super::Header;

#[derive(Copy,Clone,Default)]
pub enum DrawerSize {
  Small,Medium,
  #[default]
  Wide
}

impl DrawerSize {
  fn css(self) -> &'static str {
    //inject_css("flams-drawer",include_str!("./drawer.css"));
    let (cls,csstr) = match self {
      Self::Small => ("flams-drawer-absolute-small", ".flams-drawer-absolute-small {--thaw-drawer--size:20vw !important;z-index:5;}"),
      Self::Medium => ("flams-drawer-absolute-medium", ".flams-drawer-absolute-medium {--thaw-drawer--size:50vw !important;z-index:5;}"),
      Self::Wide => ("flams-drawer-absolute-wide", ".flams-drawer-absolute-wide {--thaw-drawer--size:80vw !important;z-index:5;}"),
    };
    inject_css(cls,csstr);
    cls
  }
}

#[component]
pub fn Drawer<Ch:IntoView+'static>(
  lazy:bool,
  trigger:super::Trigger,
  #[prop(optional)] header:Option<Header>,
  #[prop(optional)] size:DrawerSize,
  children:TypedChildrenMut<Ch>
) -> impl IntoView {
  use thaw::{Button,ButtonAppearance,DrawerBody,OverlayDrawer,DrawerHeaderTitle,DrawerHeader,DrawerPosition,DrawerHeaderTitleAction};
  let mut children = children.into_inner();
  //inject_css("flams-drawer", ".flams-wide-drawer { z-index:5;}");
  let open = RwSignal::new(false);
  view!{
    <span on:click=move |_| open.set(true)>{(trigger.children)()}</span>
    <OverlayDrawer class=size.css() open position=DrawerPosition::Right>
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
        if !lazy || open.get() { Some(children())}
        else { None }
      }</DrawerBody>
    </OverlayDrawer>
  }
}