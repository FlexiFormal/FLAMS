use leptos::{context::Provider, html, prelude::*};
use leptos::web_sys::{DomRect, Element};

use crate::inject_css;
use super::Header;

#[component]
pub fn AnchorLink(
    header:Header,
    /// The target of link.
    #[prop(into)]
    href: String,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let anchor = AnchorInjection::expect_context();
    let title_ref = NodeRef::<html::A>::new();
    let href_id = StoredValue::new(None::<String>);
    let is_active = Memo::new(move |_| {
        href_id.with_value(|href_id| {
            if href_id.is_none() {
                false
            } else {
                anchor.active_id.with(|active_id| active_id == href_id)
            }
        })
    });

    if !href.is_empty() && href.starts_with('#') {
      let id = href[1..].to_string();
      href_id.set_value(Some(id.clone()));
      anchor.append_id(id);

      on_cleanup(move || {
          href_id.with_value(|id| {
              if let Some(id) = id {
                  anchor.remove_id(id);
              }
          });
      });

      Effect::new(move |_| {
          let Some(title_el) = title_ref.get() else {
              return;
          };

          if is_active.get() {
              let title_rect = title_el.get_bounding_client_rect();
              anchor.update_background_position(&title_rect);
          }
      });
    }
    let on_click = move |_| {
        href_id.with_value(move |href_id| {
            if let Some(href_id) = href_id {
                AnchorInjection::scroll_into_view(href_id);
            }
        });
    };

    view! {
        <div class="thaw-anchor-link" class:thaw-anchor-link--active= move || is_active.get()>
            <a
                href=href
                class="thaw-anchor-link__title"
                on:click=on_click
                node_ref=title_ref
            >
                {(header.children)()}
            </a>
            {children.map(|c| c())}
        </div>
    }
}


#[component]
pub fn Anchor(
  children: Children,
) -> impl IntoView {
  inject_css("anchor",include_str!("./anchor.css"));

  let anchor_ref = NodeRef::new();
  let bar_ref = NodeRef::new();
  let element_ids = RwSignal::new(Vec::<String>::new());
  let active_id = RwSignal::new(None::<String>);

    #[cfg(any(feature = "csr", feature = "hydrate"))]
    {
        use leptos::ev;
        use std::cmp::Ordering;
        use thaw_utils::{add_event_listener_with_bool, throttle};

        struct LinkInfo {
            top: f64,
            id: String,
        }

        let offset_target : send_wrapper::SendWrapper<Option<OffsetTarget>>  = send_wrapper::SendWrapper::new(None);

        let on_scroll = move || {
            element_ids.with(|ids| {
                let offset_target_top = if let Some(offset_target) = offset_target.as_ref() {
                    if let Some(rect) = offset_target.get_bounding_client_rect() {
                        rect.top()
                    } else {
                        return;
                    }
                } else {
                    0.0
                };

                let mut links: Vec<LinkInfo> = vec![];
                for id in ids {
                    if let Some(link_el) = document().get_element_by_id(id) {
                        let link_rect = link_el.get_bounding_client_rect();
                        links.push(LinkInfo {
                            top: link_rect.top() - offset_target_top,
                            id: id.clone(),
                        });
                    }
                }
                links.sort_by(|a, b| {
                    if a.top > b.top {
                        Ordering::Greater
                    } else {
                        Ordering::Less
                    }
                });

                let mut temp_link = None::<LinkInfo>;
                for link in links {
                    if link.top >= 0.0 {
                        if link.top <= 12.0 {
                            temp_link = Some(link);
                            break;
                        } else if temp_link.is_some() {
                            break;
                        } 
                        temp_link = None;
                    } else {
                        temp_link = Some(link);
                    }
                }
                active_id.set(temp_link.map(|link| link.id));
            });
        };
        let cb = throttle(
            move || {
                on_scroll();
            },
            std::time::Duration::from_millis(200),
        );
        let scroll_handle = add_event_listener_with_bool(
            document(),
            ev::scroll,
            move |_| {
                cb();
            },
            true,
        );
        on_cleanup(move || {
            scroll_handle.remove();
        });
    }

    view! {
      <div class="thaw-anchor" node_ref=anchor_ref>
          <div class="thaw-anchor-rail">
              <div
                  class="thaw-anchor-rail__bar"
                  class=(
                      "thaw-anchor-rail__bar--active",
                      move || active_id.with(Option::is_some),
                  )

                  node_ref=bar_ref
              ></div>
          </div>
          <Provider value=AnchorInjection::new(
              anchor_ref,
              bar_ref,
              element_ids,
              active_id,
          )>{children()}</Provider>
      </div>
  }
}

#[derive(Clone,Copy)]
struct AnchorInjection {
    anchor_ref: NodeRef<html::Div>,
    bar_ref: NodeRef<html::Div>,
    element_ids: RwSignal<Vec<String>>,
    active_id: RwSignal<Option<String>>,
}


impl AnchorInjection {
  pub fn expect_context() -> Self {
      expect_context()
  }

  const fn new(
      anchor_ref: NodeRef<html::Div>,
      bar_ref: NodeRef<html::Div>,
      element_ids: RwSignal<Vec<String>>,
      active_id: RwSignal<Option<String>>,
  ) -> Self {
      Self {
          anchor_ref,
          bar_ref,
          element_ids,
          active_id,
      }
  }

  pub fn scroll_into_view(id: &str) {
      let Some(link_el) = document().get_element_by_id(id) else {
          return;
      };
      link_el.scroll_into_view();
  }

  pub fn append_id(&self, id: String) {
      self.element_ids.update(|ids| {
          ids.push(id);
      });
  }

  pub fn remove_id(&self, id: &String) {
      self.element_ids.update(|ids| {
          if let Some(index) = ids.iter().position(|item_id| item_id == id) {
              ids.remove(index);
          }
      });
  }

  pub fn update_background_position(&self, title_rect: &DomRect) {
      if let Some(anchor_el) = self.anchor_ref.get_untracked() {
          let bar_el = self.bar_ref.get_untracked().expect("This should not happen");
          let anchor_rect = anchor_el.get_bounding_client_rect();

          let offset_top = title_rect.top() - anchor_rect.top();
          // let offset_left = title_rect.left() - anchor_rect.left();

          bar_el.style(("top", format!("{offset_top}px")));
          bar_el.style(("height", format!("{}px", title_rect.height())));
      }
  }
}

pub enum OffsetTarget {
  Selector(String),
  Element(Element),
}

#[cfg(any(feature = "csr", feature = "hydrate"))]
impl OffsetTarget {
  fn get_bounding_client_rect(&self) -> Option<DomRect> {
      match self {
          Self::Selector(selector) => {
              let el = document().query_selector(selector).ok().flatten()?;
              Some(el.get_bounding_client_rect())
          }
          Self::Element(el) => Some(el.get_bounding_client_rect()),
      }
  }
}

impl From<&'static str> for OffsetTarget {
  fn from(value: &'static str) -> Self {
      Self::Selector(value.to_string())
  }
}

impl From<String> for OffsetTarget {
  fn from(value: String) -> Self {
      Self::Selector(value)
  }
}

impl From<Element> for OffsetTarget {
  fn from(value: Element) -> Self {
      Self::Element(value)
  }
}