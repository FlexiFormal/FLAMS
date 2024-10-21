#![allow(clippy::must_use_candidate)]
use crate::inject_css;
use leptos::prelude::*;

#[component]
pub fn Tree(children:Children) -> impl IntoView {
    inject_css("immt-treeview",include_str!("trees.css"));
    view!{
        <ul class="immt-treeview">{children()}</ul>
    }
}

#[component]
pub fn Leaf(children:Children) -> impl IntoView {
    view!{
        <li class="immt-treeview-li">{children()}</li>
    }
}

#[component]
pub fn Subtree(
    #[prop(optional)] lazy:bool,
    header:super::Header,
    mut children:ChildrenFnMut
) -> impl IntoView {
    let expanded = RwSignal::new(false);
    view!{
        <li class="immt-treeview-li"><details>
            <summary class="immt-treeview-summary" on:click=move |_| {expanded.update(|b| *b = !*b)}>
                {(header.children)()}
            </summary>
        <Tree>{if lazy {
            (move || if expanded.get() {
                let children = children();
                Some(children)
            } else {None}).into_any()
        } else {children().into_any()}}</Tree>
        </details></li>
    }
}