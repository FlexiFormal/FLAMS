#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]

use leptos::prelude::*;
use leptos::{ev, html};
use std::time::Duration;
use thaw_utils::{add_event_listener, class_list,BoxCallback};
use thaw_components::{CSSTransition,Follower};
use super::binder::Binder;

/// Largely copied from [thaw](https://docs.rs/thaw), but modified
/// to work with MathML.
#[component]
pub fn Popover(
    #[prop(optional, into)] class: MaybeProp<String>,
    /// Action that displays the popover.
    #[prop(optional)]
    trigger_type: PopoverTriggerType,
    /// The element or component that triggers popover.
    popover_trigger: PopoverTrigger,
    /// Configures the position of the Popover.
    #[prop(optional)]
    position: PopoverPosition,
    #[prop(optional)] max_width:u32,
    children: Children,
    #[prop(optional, into)]
    appearance: MaybeProp<PopoverAppearance>,
    #[prop(optional, into)] size: Signal<PopoverSize>,
    #[prop(optional, into)] on_open: Option<BoxCallback>,
    #[prop(optional, into)] on_close: Option<BoxCallback>,
    #[prop(optional)] node_type: DivOrMrow,
) -> impl IntoView {
    //#[derive(Copy,Clone)]
    //struct InnerPopover(RwSignal<Option<RwSignal<bool>>>);
    //let previous_popover = use_context::<InnerPopover>();
    //let this_popover = InnerPopover(RwSignal::new(None));
    
    crate::inject_css("thaw-id-popover", include_str!("./popover.css"));
    let config_provider = thaw::ConfigInjection::expect_context();
    let popover_ref = NodeRef::<html::Div>::new();
    let target_ref = node_type.new_ref();
    let is_show_popover = RwSignal::new(false);
    let show_popover_handle = StoredValue::new(None::<TimeoutHandle>);



    if on_open.is_some() || on_close.is_some() {
        Effect::watch(
            move || is_show_popover.get(),
            move |is_shown, prev_is_shown, _| {
                if prev_is_shown != Some(is_shown) {
                    if *is_shown {
                        if let Some(on_open) = &on_open {
                            on_open();
                        }
                    } else if let Some(on_close) = &on_close {
                        on_close();
                    }
                }
            },
            false,
        );
    }

    let on_mouse_enter = move |_| {
        if trigger_type != PopoverTriggerType::Hover {
            return;
        }
        show_popover_handle.update_value(|handle| {
            if let Some(handle) = handle.take() {
                handle.clear();
            }
        });
        is_show_popover.set(true);
    };
    let on_mouse_leave = move |_| {
        if trigger_type != PopoverTriggerType::Hover {
            return;
        }
        show_popover_handle.update_value(|handle| {
            if let Some(handle) = handle.take() {
                handle.clear();
            }
            *handle = set_timeout_with_handle(
                move || {
                    is_show_popover.set(false);
                },
                Duration::from_millis(100),
            )
            .ok();
        });
    };
    #[cfg(any(feature = "csr", feature = "hydrate"))]
    {
        let handle = window_event_listener(ev::click, move |ev| {
            use leptos::wasm_bindgen::__rt::IntoJsResult;
            if trigger_type != PopoverTriggerType::Click {
                return;
            }
            if !is_show_popover.get_untracked() {
                return;
            }
            let el = ev.target();
            let mut el: Option<leptos::web_sys::Element> =
                el.into_js_result().map_or(None, |el| Some(el.into()));
            let body = document().body().unwrap();
            while let Some(current_el) = el {
                if current_el == *body {
                    break;
                };
                let Some(popover_el) = popover_ref.get_untracked() else {
                    break;
                };
                if current_el == **popover_el {
                    return;
                }
                el = current_el.parent_element();
            }
            is_show_popover.set(false);
        });
        on_cleanup(move || handle.remove());
    }
    Effect::new(move |_| {
        let Some(target_el) = target_ref.get() else {
            return;
        };
        let handler = add_event_listener(target_el, ev::click, move |event| {
            if trigger_type != PopoverTriggerType::Click {
                return;
            }
            event.stop_propagation();
            is_show_popover.update(|show| *show = !*show);
        });
        on_cleanup(move || handler.remove());
    });

    let PopoverTrigger {
        class: trigger_class,
        children: trigger_children,
    } = popover_trigger;
    
    view! {
        <Binder target_ref max_width>
            {match target_ref {
                DivOrMrowRef::Div(target_ref) => view!{
                    <div
                        class=class_list![
                            "thaw-popover-trigger",
                            move || is_show_popover.get().then(|| "thaw-popover-trigger--open".to_string()),
                            trigger_class
                        ]
                        node_ref=target_ref
                        on:mouseenter=on_mouse_enter
                        on:mouseleave=on_mouse_leave
                    >
                        {trigger_children()}
                    </div>
                }.into_any(),
                DivOrMrowRef::Mrow(target_ref) => view!{
                    <mrow
                        class=class_list![
                            "thaw-popover-trigger",
                            move || is_show_popover.get().then(|| "thaw-popover-trigger--open".to_string()),
                            trigger_class
                        ]
                        node_ref=target_ref
                        on:mouseenter=on_mouse_enter
                        on:mouseleave=on_mouse_leave
                    >
                        {trigger_children()}
                    </mrow>
                }.into_any()
            }}
            <Follower slot show=is_show_popover placement=position>
                <CSSTransition
                    node_ref=popover_ref
                    name="popover-transition"
                    appear=is_show_popover.get_untracked()
                    show=is_show_popover
                    let:display
                >
                    <div
                        class=class_list![
                            "thaw-config-provider thaw-popover-surface",
                            move || format!("thaw-popover-surface--{}", size.get().as_str()),
                            move || appearance.get().map(|a| format!("thaw-popover-surface--{}", a.as_str())),
                            class
                        ]
                        data-thaw-id=config_provider.id()
                        style=move || display.get().unwrap_or_default()

                        node_ref=popover_ref
                        on:mouseenter=on_mouse_enter
                        on:mouseleave=on_mouse_leave
                    >
                        {children()}
                        <div class="thaw-popover-surface__angle"></div>
                    </div>
                </CSSTransition>
            </Follower>
        </Binder>
    }
}

#[derive(Debug, Default, Clone)]
pub enum PopoverSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl PopoverSize {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, Default)]
pub enum DivOrMrow {
    #[default]
    Div,
    Mrow,
}
impl DivOrMrow {
    #[inline]
    fn new_ref(self) -> DivOrMrowRef {
        match self {
            Self::Div => DivOrMrowRef::Div(NodeRef::new()),
            Self::Mrow => DivOrMrowRef::Mrow(NodeRef::new()),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum DivOrMrowRef {
    Div(NodeRef::<html::Div>),
    Mrow(NodeRef::<leptos::tachys::mathml::Mrow>)
}
impl DivOrMrowRef {
    #[inline]
    pub fn get(&self) -> Option<DivOrMrowElem> {
        match self {
            Self::Div(r) => r.get().map(DivOrMrowElem::Div),
            Self::Mrow(r) => r.get().map(DivOrMrowElem::Mrow)
        }
    }
    #[inline]
    pub fn get_untracked(&self) -> Option<DivOrMrowElem> {
        match self {
            Self::Div(r) => r.get_untracked().map(DivOrMrowElem::Div),
            Self::Mrow(r) => r.get_untracked().map(DivOrMrowElem::Mrow)
        }
    }
}

pub(crate) enum DivOrMrowElem {
    Div(leptos::web_sys::HtmlDivElement),
    Mrow(leptos::web_sys::Element)
}
impl std::ops::Deref for DivOrMrowElem {
    type Target = leptos::web_sys::Element;
    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Div(e) => &**e,
            Self::Mrow(e) => e
        }
    }
}
impl DivOrMrowElem {
    #[inline]
    pub fn get_bounding_client_rect(&self) -> leptos::web_sys::DomRect {
        match self {
            Self::Div(r) => r.get_bounding_client_rect(),
            Self::Mrow(r) => r.get_bounding_client_rect()
        }
    }
}
impl Into<leptos::web_sys::EventTarget> for DivOrMrowElem {
    #[inline]
    fn into(self) -> leptos::web_sys::EventTarget {
        match self {
            Self::Div(e) => e.into(),
            Self::Mrow(e) => e.into()
        }
    }
}

#[slot]
pub struct PopoverTrigger {
    #[prop(optional, into)]
    class: MaybeProp<String>,
    children: Children,
}

pub use thaw::{PopoverPosition,PopoverAppearance};


#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum PopoverTriggerType {
    #[default]
    Hover,
    Click,
}


/*
use crate::components::binder::{Binder, Follower};



impl PopoverAppearance {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Brand => "brand",
            Self::Inverted => "inverted",
        }
    }
}



#[derive(Debug, Copy, Clone, Hash, Default)]
pub enum DivOrMrow {
    #[default]
    Div,
    Mrow,
}

#[derive(Default, PartialEq, Clone, Copy, Eq)]
pub enum PopoverTriggerType {
    #[default]
    Hover,
    Click,
    HoverSignal(RwSignal<bool>),
    ClickSignal(RwSignal<bool>),
}

#[component]
pub fn Popover(
    #[prop(optional, into)] class: MaybeProp<String>,
    /// Action that displays the popover.
    #[prop(optional)]
    trigger_type: PopoverTriggerType,
    /// The element or component that triggers popover.
    popover_trigger: PopoverTrigger,
    /// Configures the position of the Popover.
    #[prop(optional)]
    position: PopoverPosition,
    children: Children,
    #[prop(optional)] node_type: DivOrMrow,
) -> impl IntoView {
    crate::inject_css("thaw-id-popover", include_str!("./popover.css"));

    let popover_ref = NodeRef::<html::Div>::new(); // TODO math

    let is_show_popover = match trigger_type {
        PopoverTriggerType::HoverSignal(s) | PopoverTriggerType::ClickSignal(s) => s,
        _ => RwSignal::new(false),
    };

    let show_popover_handle = StoredValue::new(None::<TimeoutHandle>);

    let on_mouse_enter = move |_| {
        match trigger_type {
            PopoverTriggerType::Hover | PopoverTriggerType::HoverSignal(_) => (),
            _ => return,
        }
        show_popover_handle.update_value(|handle| {
            if let Some(handle) = handle.take() {
                handle.clear();
            }
        });
        is_show_popover.set(true);
    };
    let on_mouse_leave = move |_| {
        match trigger_type {
            PopoverTriggerType::Hover | PopoverTriggerType::HoverSignal(_) => (),
            _ => return,
        }
        show_popover_handle.update_value(|handle| {
            if let Some(handle) = handle.take() {
                handle.clear();
            }
            *handle = set_timeout_with_handle(
                move || {
                    is_show_popover.set(false);
                },
                Duration::from_millis(100),
            )
            .ok();
        });
    };
    #[cfg(feature = "hydrate")]
    {
        let handle = window_event_listener(ev::click, move |ev| {
            use leptos::wasm_bindgen::__rt::IntoJsResult;
            if trigger_type != PopoverTriggerType::Click {
                return;
            }
            if !is_show_popover.get_untracked() {
                return;
            }
            let el = ev.target();
            let mut el: Option<leptos::web_sys::Element> =
                el.into_js_result().map_or(None, |el| Some(el.into()));
            let Some(body) = document().body() else {
                leptos::logging::log!("ERROR: body does not exist");
                return;
            };
            while let Some(current_el) = el {
                if current_el == *body {
                    break;
                };
                let Some(popover_el) = popover_ref.get_untracked() else {
                    break;
                };
                if current_el == **popover_el {
                    return;
                }
                el = current_el.parent_element();
            }
            is_show_popover.set(false);
        });
        on_cleanup(move || handle.remove());
    }

    let PopoverTrigger {
        class: trigger_class,
        children: trigger_children,
    } = popover_trigger;

    // ----------------------------------

    macro_rules! go {
        ($node_type:ty;$node_fun:ident) => {{
            let target_ref = NodeRef::<$node_type>::new(); // TODO math
            Effect::new(move |_| {
                let Some(target_el) = target_ref.get() else {
                    return;
                };
                let handler = add_event_listener(target_el, ev::click, move |event| {
                    if trigger_type != PopoverTriggerType::Click {
                        return;
                    }
                    event.stop_propagation();
                    is_show_popover.update(|show| *show = !*show);
                });
                on_cleanup(move || handler.remove());
            });
            // ---------------------------------------

            view! {
                <Binder target_ref>
                    <$node_fun
                        class=class_list!["thaw-patched-popover-trigger", trigger_class]
                        node_ref=target_ref
                        on:mouseenter=on_mouse_enter
                        on:mouseleave=on_mouse_leave
                    >
                        {trigger_children()}
                    </$node_fun>
                    <Follower slot show=is_show_popover placement=position>
                        <CSSTransition
                            node_ref=popover_ref
                            name="popover-transition"
                            appear=is_show_popover.get_untracked()
                            show=is_show_popover
                            let:display
                        >
                            <div
                                class=class_list![
                                    "thaw-patched-popover-surface",
                                    class
                                ]
                                style=move || display.get().unwrap_or_default()

                                node_ref=popover_ref
                                on:mouseenter=on_mouse_enter
                                on:mouseleave=on_mouse_leave
                            >
                                {children()}
                                <div class="thaw-patched-popover-surface__angle"></div>
                            </div>
                        </CSSTransition>
                    </Follower>
                </Binder>
            }
        }};
    }
    match node_type {
        DivOrMrow::Div => go!(html::Div;div).into_any(),
        DivOrMrow::Mrow => go!(leptos::tachys::mathml::Mrow;mrow).into_any(),
    }
}
 */