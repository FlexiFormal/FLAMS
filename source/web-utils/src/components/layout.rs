use std::borrow::Cow;

use ftml_component_utils::inject_css;
use leptos::prelude::*;

#[component]
pub fn Layout(
    #[prop(optional)] layout_header: Option<LayoutHeader>,
    #[prop(optional)] layout_sider: Option<LayoutSider>,
    #[prop(optional)] layout_footer: Option<LayoutFooter>,
    children: Children,
    #[prop(optional)] class: Option<&'static str>,
    #[prop(optional)] style: Option<&'static str>,
) -> impl IntoView {
    use ftml_component_utils::Scrollbar;
    use leptos::either::Either::{Left, Right};
    inject_css("flams-layout", include_str!("layout.css"));

    if layout_header.is_none() && layout_footer.is_none() {
        return if let Some(sider) = layout_sider {
            let class: Cow<str> = class.map_or_else(
                || "flams-layout-with-sider".into(),
                |cls| format!("flams-layout-with-sider {cls}").into(),
            );
            view! {
                <div class=class style=style>
                    <div class="flams-layout-sider">
                        {sider.into_view()}
                    </div>
                    <div>
                        <Scrollbar>{children()}</Scrollbar>
                    </div>
                </div>
            }
            .into_any()
        } else {
            leptos::html::div()
                .class(class)
                .style(style)
                .child(children())
                .into_any()
        };
    }

    let class: Cow<str> = class.map_or_else(
        || "flams-layout".into(),
        |cls| format!("flams-layout {cls}").into(),
    );
    let inner = move || {
        if let Some(sider) = layout_sider {
            Left(view! {
                <div class="flams-layout-with-sider">
                    <div class="flams-layout-sider">
                        <Scrollbar>{sider.into_view()}</Scrollbar>
                    </div>
                    <div>
                        <Scrollbar>{children()}</Scrollbar>
                    </div>
                </div>
            })
        } else {
            Right(view!(<div><Scrollbar>{children()}</Scrollbar></div>))
        }
    };
    view! {
        <div class=class style=style>
            {layout_header.map(LayoutHeader::into_view)}
            {inner()}
            {layout_footer.map(LayoutFooter::into_view)}
        </div>
    }
    .into_any()
}

#[slot]
pub struct LayoutHeader {
    children: Children,
    #[prop(optional)]
    class: Option<&'static str>,
    #[prop(optional)]
    style: Option<&'static str>,
}
impl LayoutHeader {
    fn into_view(self) -> impl IntoView {
        let class: Cow<str> = self.class.map_or_else(
            || "flams-layout-tf".into(),
            |cls| format!("flams-layout-tf {cls}").into(),
        );
        leptos::html::div()
            .class(class)
            .style(self.style)
            .child((self.children)())
    }
}

#[slot]
pub struct LayoutFooter {
    children: Children,
    #[prop(optional)]
    class: Option<&'static str>,
    #[prop(optional)]
    style: Option<&'static str>,
}
impl LayoutFooter {
    fn into_view(self) -> impl IntoView {
        let class: Cow<str> = self.class.map_or_else(
            || "flams-layout-tf".into(),
            |cls| format!("flams-layout-tf {cls}").into(),
        );
        leptos::html::div()
            .class(class)
            .style(self.style)
            .child((self.children)())
    }
}

#[slot]
pub struct LayoutSider {
    children: Children,
    #[prop(optional)]
    class: Option<&'static str>,
    #[prop(optional)]
    style: Option<&'static str>,
}
impl LayoutSider {
    fn into_view(self) -> impl IntoView {
        leptos::html::div()
            .class(self.class)
            .style(self.style)
            .child((self.children)())
    }
}
