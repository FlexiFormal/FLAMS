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
    //inject_css("immt-drawer",include_str!("./drawer.css"));
    let (cls,csstr) = match self {
      Self::Small => ("immt-drawer-absolute-small", ".immt-drawer-absolute-small {--thaw-drawer--size:20vw !important;z-index:5;}"),
      Self::Medium => ("immt-drawer-absolute-medium", ".immt-drawer-absolute-medium {--thaw-drawer--size:50vw !important;z-index:5;}"),
      Self::Wide => ("immt-drawer-absolute-wide", ".immt-drawer-absolute-wide {--thaw-drawer--size:80vw !important;z-index:5;}"),
    };
    inject_css(cls,csstr);
    cls
  }
  /*
  const fn css(self) -> &'static str {
    // --thaw-drawer--size:80vw !important;
    match self {
      Self::RelativeSmall => "--thaw-drawer--size:20% !important",
      Self::AbsoluteSmall => "--thaw-drawer--size:20vw !important",
      Self::RelativeMedium => "--thaw-drawer--size:50% !important",
      Self::AbsoluteMedium => "--thaw-drawer--size:50vw !important",
      Self::RelativeWide => "--thaw-drawer--size:80% !important",
      Self::AbsoluteWide => "--thaw-drawer--size:80vw !important",
    }
  }
   */
}
/*
#[cfg(not(feature="themed"))]
#[component]
pub fn Drawer(
    lazy:bool,
    trigger:super::Trigger,
    #[prop(optional)] header:Option<Header>,
    #[prop(optional)] size:DrawerSize,
    mut children:ChildrenFnMut
) -> impl IntoView {
  use thaw_components::{Teleport,FocusTrap,CSSTransition};
  use crate::{Button,ButtonAppearance,Scrollbar};
  size.css();
  let open_drawer = RwSignal::new(false);
  let drawer_ref = NodeRef::<leptos::html::Div>::new();
  let mask_ref = NodeRef::<leptos::html::Div>::new();
  let is_lock = RwSignal::new(false);
  Effect::new(move |_| {
    if open_drawer.get() { is_lock.set(true); }
  });
  thaw_utils::use_lock_html_scroll(is_lock.into());

  view!{
    <span on:click=move |_| open_drawer.set(true)>{(trigger.children)()}</span>
    <Teleport immediate=open_drawer>
      <FocusTrap disabled=false active=open_drawer on_esc=move |_| open_drawer.set(false)>
        <div class="immt-drawer-container">
          <CSSTransition node_ref=mask_ref appear=open_drawer.get_untracked() show=open_drawer name="fade-in-transition" let:display>
            <div 
              class="immt-drawer__backdrop" 
              style=move || display.get().unwrap_or_default()
              on:click=move |_| open_drawer.set(false)
              node_ref=mask_ref
            />
          </CSSTransition>
          <CSSTransition
            node_ref=drawer_ref
            appear=open_drawer.get_untracked()
            show=open_drawer
            name="slide-in-from-right-transition"
            on_after_leave=move || is_lock.set(false)
            let:display
          >
            <div
              class="immt-drawer"
              style = move || {
                display.get().map_or_else(|| size.css(),|d| d)
              }
              node_ref=drawer_ref
              role="dialog"
              aria-modal="true"
            >
              <header class="immt-drawer-header">
                <div class="immt-drawer-header-title">
                  <h2 class="immt-drawer-header-title-heading">{header.map(|h| (h.children)())}</h2>
                  <div class="immt-drawer-header-title-right">
                    <Button
                      appearance=ButtonAppearance::Subtle
                      on_click=move |_| open_drawer.set(false)>
                      "x"
                    </Button>
                  </div>
                </div>
              </header>
              <Scrollbar><div class="immt-drawer-body">{
                if lazy {(move || if open_drawer.get() { Some(children())} else {None}).into_any()}
                else {children()}
              }</div></Scrollbar>
            </div>
          </CSSTransition>
        </div>
      </FocusTrap>
    </Teleport>
  }
}
*/

#[component]
pub fn Drawer(
  lazy:bool,
  trigger:super::Trigger,
  #[prop(optional)] header:Option<Header>,
  #[prop(optional)] size:DrawerSize,
  mut children:ChildrenFnMut
) -> impl IntoView {
  use thaw::{Button,ButtonAppearance,DrawerBody,OverlayDrawer,DrawerHeaderTitle,DrawerHeader,DrawerPosition,DrawerHeaderTitleAction};
  //inject_css("immt-drawer", ".immt-wide-drawer { z-index:5;}");
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
        if !lazy || open.get() { children().into_any()}
        else {"".into_any()}
      }</DrawerBody>
    </OverlayDrawer>
  }
}