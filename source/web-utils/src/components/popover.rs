#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::must_use_candidate)]

use leptos::prelude::*;
use leptos::{ev, html};
use std::time::Duration;
pub use thaw::{PopoverAppearance, PopoverPosition, PopoverTrigger};
use thaw_utils::{add_event_listener, class_list};
//use thaw_components::{Follower, CSSTransition, Binder, FollowerWidth, FollowerInjection};
use crate::components::binder::{Binder, Follower};
use thaw_components::CSSTransition;

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
