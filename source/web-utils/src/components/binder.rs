#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::must_use_candidate)]

pub use get_placement_style::FollowerPlacement;
use leptos::wasm_bindgen::JsCast;

use get_placement_style::{get_follower_placement_offset, FollowerPlacementOffset};
use leptos::{
    context::Provider,
    ev,
    html::{self, ElementType},
    leptos_dom::helpers::WindowListenerHandle,
    prelude::*,
};
use thaw_components::Teleport;
use thaw_utils::{add_event_listener, get_scroll_parent_node, EventListenerHandle};

#[slot]
pub struct Follower {
    #[prop(into)]
    show: Signal<bool>,
    #[prop(optional)]
    width: Option<FollowerWidth>,
    #[prop(into)]
    placement: FollowerPlacement,
    children: Children,
}

#[derive(Clone)]
pub enum FollowerWidth {
    /// The popup width is the same as the target DOM width.
    Target,
    /// The popup min width is the same as the target DOM width.
    MinTarget,
    /// Customize the popup width.
    Px(u32),
}

impl Copy for FollowerWidth {}

#[component]
pub fn Binder<E>(
    /// Used to track DOM locations
    #[prop(into)]
    target_ref: NodeRef<E>,
    /// Content for pop-up display
    follower: Follower,
    children: Children,
) -> impl IntoView
where
    E: ElementType + 'static,
    E::Output: JsCast + Clone + AsRef<leptos::web_sys::Element> + 'static,
{
    crate::inject_css("thaw-id-binder", include_str!("./binder.css"));
    let Follower {
        show: follower_show,
        width: follower_width,
        placement: follower_placement,
        children: follower_children,
    } = follower;

    let scrollable_element_handle_vec = StoredValue::<Vec<EventListenerHandle>>::new(vec![]);
    let resize_handle = StoredValue::new(None::<WindowListenerHandle>);
    let content_ref = NodeRef::<html::Div>::new();
    let content_style = RwSignal::new(String::new());
    let placement_str = RwSignal::new(follower_placement.as_str());
    let sync_position = move || {
        let Some(_) = content_ref.get_untracked() else {
            return;
        };
        let Some(target_ref) = target_ref.get_untracked() else {
            return;
        };
        let tr: &leptos::web_sys::Element = target_ref.as_ref();
        let target_rect = tr.get_bounding_client_rect();
        let content_rect = tr.get_bounding_client_rect();
        let mut style = String::new();
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
        }) = get_follower_placement_offset(follower_placement, &target_rect, &content_rect)
        {
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
        let Some(el) = target_ref.as_ref() else {
            return;
        };
        let el = AsRef::<leptos::web_sys::Element>::as_ref(el);

        let mut handle_vec = vec![];
        let mut cursor = get_scroll_parent_node(el);
        while let Some(el) = cursor.take() {
            cursor = get_scroll_parent_node(&el);

            let handle = add_event_listener(el, ev::scroll, move |_| {
                sync_position();
            });
            handle_vec.push(handle);
        }
        scrollable_element_handle_vec.set_value(handle_vec);

        resize_handle.update_value(move |resize_handle| {
            if let Some(handle) = resize_handle.take() {
                handle.remove();
            }
            let handle = window_event_listener(ev::resize, move |_| {
                sync_position();
            });
            *resize_handle = Some(handle);
        });
    };

    let remove_listener = move || {
        scrollable_element_handle_vec.update_value(|vec| {
            vec.drain(..).for_each(EventListenerHandle::remove);
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

    let follower_injection = FollowerInjection();

    view! {
        {children()}
        <Teleport immediate=follower_show>
            <Provider value=follower_injection>
                <div class="thaw-binder-follower-container">
                    <div
                        class="thaw-binder-follower-content"
                        data-thaw-placement=move || placement_str.get()
                        node_ref=content_ref
                        style=move || content_style.get()
                    >
                        {follower_children()}
                    </div>
                </div>
            </Provider>
        </Teleport>
    }
}

#[derive(Debug, Clone)]
pub struct FollowerInjection();

mod get_placement_style {
    use leptos::prelude::window;
    use leptos::web_sys::DomRect;
    use thaw::PopoverPosition;

    #[derive(Clone)]
    pub enum FollowerPlacement {
        Top,
        Bottom,
        Left,
        Right,
        TopStart,
        TopEnd,
        LeftStart,
        LeftEnd,
        RightStart,
        RightEnd,
        BottomStart,
        BottomEnd,
    }

    impl Copy for FollowerPlacement {}

    impl FollowerPlacement {
        pub const fn as_str(self) -> &'static str {
            match self {
                Self::Top => "top",
                Self::Bottom => "bottom",
                Self::Left => "left",
                Self::Right => "right",
                Self::TopStart => "top-start",
                Self::TopEnd => "top-end",
                Self::LeftStart => "left-start",
                Self::LeftEnd => "left-end",
                Self::RightStart => "right-start",
                Self::RightEnd => "right-end",
                Self::BottomStart => "bottom-start",
                Self::BottomEnd => "bottom-end",
            }
        }

        pub const fn transform_origin(self) -> &'static str {
            match self {
                Self::Top => "bottom center",
                Self::Bottom => "top center",
                Self::Left => "center right",
                Self::Right => "center left",
                Self::TopStart | Self::RightEnd => "bottom left",
                Self::TopEnd | Self::LeftEnd => "bottom right",
                Self::LeftStart | Self::BottomEnd => "top right",
                Self::RightStart | Self::BottomStart => "top left",
            }
        }
    }

    pub struct FollowerPlacementOffset {
        pub top: f64,
        pub left: f64,
        pub transform: String,
        pub placement: FollowerPlacement,
    }

    pub fn get_follower_placement_offset(
        placement: FollowerPlacement,
        target_rect: &DomRect,
        follower_rect: &DomRect,
    ) -> Option<FollowerPlacementOffset> {
        match placement {
            FollowerPlacement::Top => {
                let left = target_rect.x() + target_rect.width() / 2.0;
                let (top, placement) = {
                    let follower_height = follower_rect.height();
                    let target_y = target_rect.y();
                    let target_height = target_rect.height();
                    let top = target_y - follower_height;

                    let inner_height = window_inner_height()?;

                    if top < 0.0 && target_y + target_height + follower_height <= inner_height {
                        (target_y + target_height, FollowerPlacement::Bottom)
                    } else {
                        (top, FollowerPlacement::Top)
                    }
                };
                Some(FollowerPlacementOffset {
                    top,
                    left,
                    transform: String::from("translateX(-50%)"),
                    placement,
                })
            }
            FollowerPlacement::TopStart => {
                let left = target_rect.x();
                let (top, placement) = {
                    let follower_height = follower_rect.height();
                    let target_y = target_rect.y();
                    let target_height = target_rect.height();
                    let top = target_y - follower_height;

                    let inner_height = window_inner_height()?;

                    if top < 0.0 && target_y + target_height + follower_height <= inner_height {
                        (target_y + target_height, FollowerPlacement::BottomStart)
                    } else {
                        (top, FollowerPlacement::TopStart)
                    }
                };
                Some(FollowerPlacementOffset {
                    top,
                    left,
                    transform: String::new(),
                    placement,
                })
            }
            FollowerPlacement::TopEnd => {
                let left = target_rect.x() + target_rect.width();
                let (top, placement) = {
                    let follower_height = follower_rect.height();
                    let target_y = target_rect.y();
                    let target_height = target_rect.height();
                    let top = target_y - follower_height;

                    let inner_height = window_inner_height()?;

                    if top < 0.0 && target_y + target_height + follower_height <= inner_height {
                        (target_y + target_height, FollowerPlacement::BottomEnd)
                    } else {
                        (top, FollowerPlacement::TopEnd)
                    }
                };
                Some(FollowerPlacementOffset {
                    top,
                    left,
                    transform: String::from("translateX(-100%)"),
                    placement,
                })
            }
            FollowerPlacement::Left => {
                let top = target_rect.y() + target_rect.height() / 2.0;
                let (left, placement) = {
                    let follower_width = follower_rect.width();
                    let target_x = target_rect.x();
                    let target_width = target_rect.width();
                    let left = target_x - follower_width;

                    let inner_width = window_inner_width()?;

                    if left < 0.0 && target_x + target_width + follower_width > inner_width {
                        (target_x + follower_width, FollowerPlacement::Right)
                    } else {
                        (left, FollowerPlacement::Left)
                    }
                };
                Some(FollowerPlacementOffset {
                    top,
                    left,
                    transform: String::from("translateY(-50%)"),
                    placement,
                })
            }
            FollowerPlacement::LeftStart => {
                let top = target_rect.y();
                let (left, placement) = {
                    let follower_width = follower_rect.width();
                    let target_x = target_rect.x();
                    let target_width = target_rect.width();
                    let left = target_x - follower_width;

                    let inner_width = window_inner_width()?;

                    if left < 0.0 && target_x + target_width + follower_width > inner_width {
                        (target_x + follower_width, FollowerPlacement::RightStart)
                    } else {
                        (left, FollowerPlacement::LeftStart)
                    }
                };
                Some(FollowerPlacementOffset {
                    top,
                    left,
                    transform: String::new(),
                    placement,
                })
            }
            FollowerPlacement::LeftEnd => {
                let top = target_rect.y() + target_rect.height();
                let (left, placement) = {
                    let follower_width = follower_rect.width();
                    let target_x = target_rect.x();
                    let target_width = target_rect.width();
                    let left = target_x - follower_width;

                    let inner_width = window_inner_width()?;

                    if left < 0.0 && target_x + target_width + follower_width > inner_width {
                        (target_x + follower_width, FollowerPlacement::RightEnd)
                    } else {
                        (left, FollowerPlacement::LeftEnd)
                    }
                };
                Some(FollowerPlacementOffset {
                    top,
                    left,
                    transform: String::from("translateY(-100%)"),
                    placement,
                })
            }
            FollowerPlacement::Right => {
                let top = target_rect.y() + target_rect.height() / 2.0;
                let (left, placement) = {
                    let follower_width = follower_rect.width();
                    let target_x = target_rect.x();
                    let target_width = target_rect.width();
                    let left = target_x + target_width;

                    let inner_width = window_inner_width()?;

                    if left + follower_width > inner_width && target_x - follower_width >= 0.0 {
                        (target_x - follower_width, FollowerPlacement::Left)
                    } else {
                        (left, FollowerPlacement::Right)
                    }
                };
                Some(FollowerPlacementOffset {
                    top,
                    left,
                    transform: String::from("translateY(-50%)"),
                    placement,
                })
            }
            FollowerPlacement::RightStart => {
                let top = target_rect.y();
                let (left, placement) = {
                    let follower_width = follower_rect.width();
                    let target_x = target_rect.x();
                    let target_width = target_rect.width();
                    let left = target_x + target_width;

                    let inner_width = window_inner_width()?;

                    if left + follower_width > inner_width && target_x - follower_width >= 0.0 {
                        (target_x - follower_width, FollowerPlacement::LeftStart)
                    } else {
                        (left, FollowerPlacement::RightStart)
                    }
                };
                Some(FollowerPlacementOffset {
                    top,
                    left,
                    transform: String::new(),
                    placement,
                })
            }
            FollowerPlacement::RightEnd => {
                let top = target_rect.y() + target_rect.height();
                let (left, placement) = {
                    let follower_width = follower_rect.width();
                    let target_x = target_rect.x();
                    let target_width = target_rect.width();
                    let left = target_x + target_width;

                    let inner_width = window_inner_width()?;

                    if left + follower_width > inner_width && target_x - follower_width >= 0.0 {
                        (target_x - follower_width, FollowerPlacement::LeftEnd)
                    } else {
                        (left, FollowerPlacement::RightEnd)
                    }
                };
                Some(FollowerPlacementOffset {
                    top,
                    left,
                    transform: String::from("translateY(-100%)"),
                    placement,
                })
            }
            FollowerPlacement::Bottom => {
                let left = target_rect.x() + target_rect.width() / 2.0;
                let (top, placement) = {
                    let follower_height = follower_rect.height();
                    let target_y = target_rect.y();
                    let target_height = target_rect.height();
                    let top = target_y + target_height;

                    let inner_height = window_inner_height()?;

                    if top + follower_height > inner_height && target_y - follower_height >= 0.0 {
                        (target_y - follower_height, FollowerPlacement::Top)
                    } else {
                        (top, FollowerPlacement::Bottom)
                    }
                };
                Some(FollowerPlacementOffset {
                    top,
                    left,
                    transform: String::from("translateX(-50%)"),
                    placement,
                })
            }
            FollowerPlacement::BottomStart => {
                let left = target_rect.x();
                let (top, placement) = {
                    let follower_height = follower_rect.height();
                    let target_y = target_rect.y();
                    let target_height = target_rect.height();
                    let top = target_y + target_height;

                    let inner_height = window_inner_height()?;

                    if top + follower_height > inner_height && target_y - follower_height >= 0.0 {
                        (target_y - follower_height, FollowerPlacement::TopStart)
                    } else {
                        (top, FollowerPlacement::BottomStart)
                    }
                };
                Some(FollowerPlacementOffset {
                    top,
                    left,
                    transform: String::new(),
                    placement,
                })
            }
            FollowerPlacement::BottomEnd => {
                let left = target_rect.x() + target_rect.width();
                let (top, placement) = {
                    let follower_height = follower_rect.height();
                    let target_y = target_rect.y();
                    let target_height = target_rect.height();
                    let top = target_y + target_height;

                    let inner_height = window_inner_height()?;

                    if top + follower_height > inner_height && target_y - follower_height >= 0.0 {
                        (target_y - follower_height, FollowerPlacement::TopEnd)
                    } else {
                        (top, FollowerPlacement::BottomEnd)
                    }
                };
                Some(FollowerPlacementOffset {
                    top,
                    left,
                    transform: String::from("translateX(-100%)"),
                    placement,
                })
            }
        }
    }

    fn window_inner_width() -> Option<f64> {
        window().inner_width().ok()?.as_f64()
    }

    fn window_inner_height() -> Option<f64> {
        window().inner_height().ok()?.as_f64()
    }
    impl From<thaw::PopoverPosition> for FollowerPlacement {
        fn from(value: PopoverPosition) -> Self {
            match value {
                PopoverPosition::Top => Self::Top,
                PopoverPosition::Bottom => Self::Bottom,
                PopoverPosition::Left => Self::Left,
                PopoverPosition::Right => Self::Right,
                PopoverPosition::TopStart => Self::TopStart,
                PopoverPosition::TopEnd => Self::TopEnd,
                PopoverPosition::LeftStart => Self::LeftStart,
                PopoverPosition::LeftEnd => Self::LeftEnd,
                PopoverPosition::RightStart => Self::RightStart,
                PopoverPosition::RightEnd => Self::RightEnd,
                PopoverPosition::BottomStart => Self::BottomStart,
                PopoverPosition::BottomEnd => Self::BottomEnd,
            }
        }
    }
}
