#![allow(clippy::must_use_candidate)]
use crate::inject_css;
use leptos::prelude::*;

#[component]
pub fn Tree<T:IntoView+'static>(children:TypedChildren<T>) -> impl IntoView {
    let children = children.into_inner();
    inject_css("flams-treeview",include_str!("trees.css"));
    view!{
        <ul class="flams-treeview">{children()}</ul>
    }
}

#[component]
pub fn Leaf<T:IntoView+'static>(children:TypedChildren<T>) -> impl IntoView {
    let children = children.into_inner();
    view!{
        <li class="flams-treeview-li">{children()}</li>
    }
}

#[component]
pub fn Subtree<T:IntoView+'static>(
    header:super::Header,
    children:TypedChildren<T>
) -> impl IntoView {
    let children = children.into_inner();
    let expanded = RwSignal::new(false);
    view!{
        <li class="flams-treeview-li"><details>
            <summary class="flams-treeview-summary" on:click=move |_| {expanded.update(|b| *b = !*b)}>
                {(header.children)()}
            </summary>
        <Tree>{children()}</Tree>
        </details></li>
    }
}

#[component]
pub fn LazySubtree<T:IntoView+'static>(
    header:super::Header,
    children:TypedChildrenMut<T>
) -> impl IntoView {
    let mut children = children.into_inner();
    let expanded = RwSignal::new(false);
    view!{
        <li class="flams-treeview-li"><details>
            <summary class="flams-treeview-summary" on:click=move |_| {expanded.update(|b| *b = !*b)}>
                {(header.children)()}
            </summary>
        <Tree>{move || if expanded.get() {
            let children = children();
            Some(children)
        } else {None}
        }</Tree>
        </details></li>
    }
}