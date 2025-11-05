#![allow(clippy::must_use_candidate)]

use ftml_dom::utils::css::inject_css;
use leptos::prelude::*;

#[component]
pub fn Tree(children: Children) -> impl IntoView {
    inject_css("flams-treeview", include_str!("trees.css"));
    view! {
        <ul class="flams-treeview">{children()}</ul>
    }
}

#[component]
pub fn Leaf(children: Children) -> impl IntoView {
    view! {
        <li class="flams-treeview-li">{children()}</li>
    }
}

#[component]
pub fn Subtree(
    header: super::Header,
    children: Children,
    #[prop(default = false)] expanded: bool,
) -> impl IntoView {
    let children = move || {
        view! {
            <summary class="flams-treeview-summary">
                {(header.children)()}
            </summary>
            <Tree>{children()}</Tree>
        }
    };
    let spread = if expanded {
        leptos::either::Either::Left(view!(<{..} open="true"/>))
    } else {
        leptos::either::Either::Right(view!(<{..}/>))
    };
    view! {
        <li class="flams-treeview-li">
            <details {..spread}>{children()}</details>
        </li>
    }
}

#[component]
pub fn LazySubtree(
    header: super::Header,
    mut children: ChildrenFnMut,
) -> impl IntoView {
    let expanded = RwSignal::new(false);
    let children = move || {
        view! {
            <summary class="flams-treeview-summary" on:click=move |_| {expanded.update(|b| *b = !*b)}>
                {(header.children)()}
            </summary>
        <Tree>{move || if expanded.get() {
            let children = children();
            Some(children)
        } else {None}
        }</Tree>
        }
    };
    view! {
        <li class="flams-treeview-li"><details>
            {children()}
        </details></li>
    }
}
