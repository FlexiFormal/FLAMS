#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]

use leptos::prelude::*;
use thaw_utils::{add_event_listener, get_scroll_parent_node};
use super::popover::DivOrMrowRef;
use thaw_components::{Follower, FollowerPlacement, FollowerWidth, Teleport};
use leptos::web_sys::DomRect;

#[component]
pub fn Binder(
    /// Used to track DOM locations
    #[prop(into)]
    target_ref: DivOrMrowRef,
    #[prop(optional)] mut max_width:u32,
    /// Content for pop-up display
    follower: Follower,
    children: Children,
) -> impl IntoView {
    if max_width == 0 { max_width = 600 };
    crate::inject_css("thaw-id-binder", include_str!("./binder.css"));
    let Follower {
        show: follower_show,
        width: follower_width,
        placement: follower_placement,
        children: follower_children,
    } = follower;

    let scrollable_element_handle_vec = StoredValue::<Vec<thaw_utils::EventListenerHandle>>::new(vec![]);
    let resize_handle = StoredValue::new(None::<WindowListenerHandle>);
    let follower_ref = NodeRef::<leptos::html::Div>::new();
    let content_ref = NodeRef::<leptos::html::Div>::new();
    let content_style = RwSignal::new(String::new());
    let placement_str = RwSignal::new(follower_placement.as_str());
    let sync_position = move || {
        let Some(follower_el) = follower_ref.get_untracked() else {
            return;
        };
        let Some(content_ref) = content_ref.get_untracked() else {
            return;
        };
        let Some(target_ref) = target_ref.get_untracked() else {
            return;
        };
        let follower_rect = follower_el.get_bounding_client_rect();
        let target_rect = target_ref.get_bounding_client_rect();
        let content_rect = content_ref.get_bounding_client_rect();
        let mut style = format!("max-width:{max_width}px;");
        if let Some(width) = follower_width {
            let width = match width {
                FollowerWidth::Target => format!("width: {}px;", target_rect.width()),
                FollowerWidth::MinTarget => format!("min-width: {}px;", target_rect.width()),
                FollowerWidth::Px(width) => format!("width: {width}px;"),
            };
            style.push_str(&width);
        }
        if let Some(FollowerPlacementOffset {
            top,
            left,
            transform,
            placement,
        }) = get_follower_placement_offset(
            max_width,
            follower_placement,
            target_rect,
            follower_rect,
            content_rect,
        ) {
            placement_str.set(placement.as_str());
            style.push_str(&format!(
                "transform-origin: {};",
                placement.transform_origin()
            ));
            style.push_str(&format!(
                "transform: translateX({left}px) translateY({top}px) {transform};"
            ));
        } else {
            leptos::logging::error!("Thaw-Binder: get_follower_placement_style return None");
        }

        content_style.set(style);
    };

    let ensure_listener = move || {
        let target_ref = target_ref.get_untracked();
        let Some(el) = target_ref.as_deref() else {
            return;
        };

        let mut handle_vec = vec![];
        let mut cursor = get_scroll_parent_node(el);
        while let Some(node) = cursor.take() {
            cursor = get_scroll_parent_node(&node);

            let handle = add_event_listener(node, leptos::ev::scroll, move |_| {
                sync_position();
            });
            handle_vec.push(handle);
        }
        scrollable_element_handle_vec.set_value(handle_vec);

        resize_handle.update_value(move |resize_handle| {
            if let Some(handle) = resize_handle.take() {
                handle.remove();
            }
            let handle = window_event_listener(leptos::ev::resize, move |_| {
                sync_position();
            });
            *resize_handle = Some(handle);
        });
    };

    let remove_listener = move || {
        scrollable_element_handle_vec.update_value(|vec| {
            vec.drain(..).for_each(thaw_utils::EventListenerHandle::remove);
        });
        resize_handle.update_value(move |handle| {
            if let Some(handle) = handle.take() {
                handle.remove();
            }
        });
    };

    Effect::new(move |_| {
        if target_ref.get().is_none() {
            return;
        }
        if content_ref.get().is_none() {
            return;
        }
        if follower_show.get() {
            request_animation_frame(move || {
                sync_position();
            });

            remove_listener();
            ensure_listener();
        } else {
            remove_listener();
        }
    });

    Owner::on_cleanup(move || {
        remove_listener();
    });

    //let follower_injection = FollowerInjection(Callback::new(move |()| sync_position()));

    view! {
        {children()}
        <Teleport immediate=follower_show>
            <div class="thaw-binder-follower" node_ref=follower_ref>
                <div
                    class="thaw-binder-follower-content"
                    data-thaw-placement=move || placement_str.get()
                    node_ref=content_ref
                    style=move || content_style.get()
                >
                    //<Provider value=follower_injection>
                        {follower_children()}
                    //</Provider>
                </div>
            </div>
        </Teleport>
    }
}
/*
#[derive(Debug, Clone, Copy)]
pub struct FollowerInjection(Callback<()>);

impl FollowerInjection {
    pub fn expect_context() -> Self {
        expect_context()
    }

    pub fn refresh_position(&self) {
        self.0.run(());
    }
}
 */

struct FollowerPlacementOffset {
    pub top: f64,
    pub left: f64,
    pub transform: String,
    pub placement: FollowerPlacement,
}

#[allow(clippy::cognitive_complexity)]
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::cast_lossless)]
fn get_follower_placement_offset(
    max_width:u32,
    placement: FollowerPlacement,
    target_rect: DomRect,
    follower_rect: DomRect,
    content_rect: DomRect,
) -> Option<FollowerPlacementOffset> {
    let barrier_left = (max_width / 2) as f64;
    let barrier_right = window_inner_width().map(|w| w - barrier_left)?;
    let (left, placement, top, transform) = match placement {
        FollowerPlacement::Top | FollowerPlacement::TopStart | FollowerPlacement::TopEnd => {
            let window_inner_height = window_inner_height()?;
            let content_height = content_rect.height();
            let target_top = target_rect.top();
            let target_bottom = target_rect.bottom();
            let top = target_top - content_height;
            let (top, new_placement) =
                if top < 0.0 && target_bottom + content_height <= window_inner_height {
                    let new_placement = if placement == FollowerPlacement::Top {
                        FollowerPlacement::Bottom
                    } else if placement == FollowerPlacement::TopStart {
                        FollowerPlacement::BottomStart
                    } else if placement == FollowerPlacement::TopEnd {
                        FollowerPlacement::BottomEnd
                    } else {
                        unreachable!()
                    };
                    (target_bottom, new_placement)
                } else {
                    (top, placement)
                };

            if placement == FollowerPlacement::Top {
                let left = (target_rect.left() + target_rect.width() / 2.0).max(barrier_left).min(barrier_right);
                //leptos::logging::log!("Here: {left} {top}");
                let transform = String::from("translateX(-50%)");
                (left, new_placement, top, transform)
            } else if placement == FollowerPlacement::TopStart {
                let left = target_rect.left().max(barrier_left).min(barrier_right);
                //leptos::logging::log!("Here: {left} {top}");
                let transform = String::new();
                (left, new_placement, top, transform)
            } else if placement == FollowerPlacement::TopEnd {
                let left = target_rect.right().max(barrier_left).min(barrier_right);
                //leptos::logging::log!("Here: {left} {top}");
                let transform = String::from("translateX(-100%)");
                (left, new_placement, top, transform)
            } else {
                unreachable!()
            }
        }
        FollowerPlacement::Bottom
        | FollowerPlacement::BottomStart
        | FollowerPlacement::BottomEnd => {
            let window_inner_height = window_inner_height()?;
            let content_height = content_rect.height();
            let target_top = target_rect.top();
            let target_bottom = target_rect.bottom();
            let top = target_bottom;
            let (top, new_placement) = if top + content_height > window_inner_height
                && target_top - content_height >= 0.0
            {
                let new_placement = if placement == FollowerPlacement::Bottom {
                    FollowerPlacement::Top
                } else if placement == FollowerPlacement::BottomStart {
                    FollowerPlacement::TopStart
                } else if placement == FollowerPlacement::BottomEnd {
                    FollowerPlacement::TopEnd
                } else {
                    unreachable!()
                };
                (target_top - content_height, new_placement)
            } else {
                (top, placement)
            };
            if placement == FollowerPlacement::Bottom {
                let left = (target_rect.left() + target_rect.width() / 2.0).max(barrier_left).min(barrier_right);
                let transform = String::from("translateX(-50%)");
                (left, new_placement, top, transform)
            } else if placement == FollowerPlacement::BottomStart {
                let left = target_rect.left().max(barrier_left).min(barrier_right);
                let transform = String::new();
                (left, new_placement, top, transform)
            } else if placement == FollowerPlacement::BottomEnd {
                let left = target_rect.right().max(barrier_left).min(barrier_right);
                let transform = String::from("translateX(-100%)");
                (left, new_placement, top, transform)
            } else {
                unreachable!()
            }
        }
        FollowerPlacement::Left | FollowerPlacement::LeftStart | FollowerPlacement::LeftEnd => {
            let window_inner_width = window_inner_width()?;
            let content_width = content_rect.width();
            let target_left = target_rect.left();
            let target_right = target_rect.right();
            let left = target_left - content_width;

            let (left, new_placement) =
                if left < 0.0 && target_right + content_width <= window_inner_width {
                    let new_placement = if placement == FollowerPlacement::Left {
                        FollowerPlacement::Right
                    } else if placement == FollowerPlacement::LeftStart {
                        FollowerPlacement::RightStart
                    } else if placement == FollowerPlacement::LeftEnd {
                        FollowerPlacement::RightEnd
                    } else {
                        unreachable!()
                    };
                    (target_right, new_placement)
                } else {
                    (left, placement)
                };
            if placement == FollowerPlacement::Left {
                let top = target_rect.top() + target_rect.height() / 2.0;
                let transform = String::from("translateY(-50%)");
                (left, new_placement, top, transform)
            } else if placement == FollowerPlacement::LeftStart {
                let top = target_rect.top();
                let transform = String::new();
                (left, new_placement, top, transform)
            } else if placement == FollowerPlacement::LeftEnd {
                let top = target_rect.bottom();
                let transform = String::from("translateY(-100%)");
                (left, new_placement, top, transform)
            } else {
                unreachable!()
            }
        }
        FollowerPlacement::Right | FollowerPlacement::RightStart | FollowerPlacement::RightEnd => {
            let window_inner_width = window_inner_width()?;
            let content_width = content_rect.width();
            let target_left = target_rect.left();
            let target_right = target_rect.right();
            let left = target_right;
            let (left, new_placement) = if left + content_width > window_inner_width
                && target_left - content_width >= 0.0
            {
                let new_placement = if placement == FollowerPlacement::Right {
                    FollowerPlacement::Left
                } else if placement == FollowerPlacement::RightStart {
                    FollowerPlacement::LeftStart
                } else if placement == FollowerPlacement::RightEnd {
                    FollowerPlacement::LeftEnd
                } else {
                    unreachable!()
                };
                (target_left - content_width, new_placement)
            } else {
                (left, placement)
            };

            if placement == FollowerPlacement::Right {
                let top = target_rect.top() + target_rect.height() / 2.0;
                let transform = String::from("translateY(-50%)");
                (left, new_placement, top, transform)
            } else if placement == FollowerPlacement::RightStart {
                let top = target_rect.top();
                let transform = String::new();
                (left, new_placement, top, transform)
            } else if placement == FollowerPlacement::RightEnd {
                let top = target_rect.bottom();
                let transform = String::from("translateY(-100%)");
                (left, new_placement, top, transform)
            } else {
                unreachable!()
            }
        }
    };

    Some(FollowerPlacementOffset {
        top: top - follower_rect.top(),
        left: left - follower_rect.left(),
        placement,
        transform,
    })
}

fn window_inner_width() -> Option<f64> {
    let inner_width = window().inner_width().ok()?;
    let inner_width = inner_width.as_f64()?;
    Some(inner_width)
}

fn window_inner_height() -> Option<f64> {
    let inner_height = window().inner_height().ok()?;
    let inner_height = inner_height.as_f64()?;
    Some(inner_height)
}
