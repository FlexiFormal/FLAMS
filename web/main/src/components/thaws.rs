use leptos::{ev, html};
use leptos::prelude::*;
use thaw::{BadgeAppearance, BadgeColor, BadgeSize, DrawerPosition, DrawerSize};

#[component]
pub(crate) fn Badge(
    #[prop(optional,into)] appearance:Option<BadgeAppearance>,
    #[prop(optional,into)] size:Option<BadgeSize>,
    #[prop(optional,into)] color:Option<BadgeColor>,
    children: Children
) -> impl IntoView {
    use leptos::either::Either;
    let mut classes = "thaw-badge".to_string();
    if let Some(a) = appearance {
        classes.push_str(&format!(" thaw-badge--{}",a.as_str()));
    }
    if let Some(s) = size {
        classes.push_str(&format!(" thaw-badge--{}",s.as_str()));
    }
    if let Some(c) = color {
        classes.push_str(&format!(" thaw-badge--{}",c.as_str()));
    }
    view! {
        <div class=classes>{children()}</div>
    }
}

pub(crate) fn drawer(open:RwSignal<bool>,header:Option<impl IntoView + 'static>,children: impl IntoView + 'static) -> impl IntoView {
    use thaw_components::{CSSTransition, FocusTrap, Teleport};
    let drawer_ref = NodeRef::<html::Div>::new();
    let mask_ref = NodeRef::<html::Div>::new();
    let is_lock = RwSignal::new(false);

    let position = DrawerPosition::Right;
    let size = DrawerSize::Full;

    thaw_utils::use_lock_html_scroll(is_lock.into());
    let on_after_leave = move || {
        is_lock.set(false);
    };
    let on_esc = move |_: leptos::ev::KeyboardEvent| {
        open.set(false);
    };
    let on_mask_click = move |_| {
        open.set(false);
    };

    view! {
        //<Teleport immediate=open>
            <FocusTrap disabled=false active=open on_esc>
                <div
                    class="thaw-overlay-drawer-container"
                >
                    <CSSTransition
                        node_ref=mask_ref
                        appear=open.get_untracked()
                        show=open
                        name="fade-in-transition"
                        let:display
                    >
                        <div
                            class="thaw-overlay-drawer__backdrop"
                            style=move || display.get().unwrap_or_default()
                            on:click=on_mask_click
                            node_ref=mask_ref
                        ></div>
                    </CSSTransition>
                    <CSSTransition
                        node_ref=drawer_ref
                        appear=open.get_untracked()
                        show=open
                        name=Memo::new(move |_| {
                            format!("slide-in-from-{}-transition", position.as_str())
                        })

                        on_after_leave
                        let:display
                    >
                        <div
                            class=move || format!("thaw-overlay-drawer thaw-overlay-drawer--position-{}", position.as_str())
                            style=move || {
                                let size = move || {
                                    format!(
                                        "--thaw-drawer--size: {}",
                                        as_size_str(size,position),
                                    )
                                };
                                display.get().map_or_else(size, |d| d.to_string())
                            }
                            node_ref=drawer_ref
                            role="dialog"
                            aria-modal="true"
                        >
                            {header.map(|h| view!(<header class="thaw-drawer-header">{h}</header>))}
                            {children}
                        </div>
                    </CSSTransition>
                </div>
            </FocusTrap>
        //</Teleport>
    }
}


fn as_size_str(size:DrawerSize, position: DrawerPosition) -> &'static str {
    match size {
        DrawerSize::Small => "320px",
        DrawerSize::Medium => "592px",
        DrawerSize::Large => "940px",
        DrawerSize::Full => match position {
            DrawerPosition::Top | DrawerPosition::Bottom => "80vh",
            DrawerPosition::Left | DrawerPosition::Right => "80vw",
        },
    }
}