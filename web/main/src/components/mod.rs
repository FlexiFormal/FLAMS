pub mod mathhub_tree;
pub mod graph_viewer;
pub mod logging;
pub mod queue;
pub mod settings;
pub mod content;
pub mod query;
mod thaws;
pub use thaws::*;

use std::future::Future;
pub use mathhub_tree::ArchiveOrGroups;
//pub use graph_viewer::GraphTest;
pub use logging::FullLog;
pub use queue::QueuesTop;
pub use settings::Settings;
pub use query::Query;

use leptos::prelude::*;
use thaw::{BadgeAppearance,BadgeSize,BadgeColor};


pub(crate) fn icon(icon:icondata_core::Icon) -> impl IntoView {
    icon_with_options(icon,Some("18px"),Some("18px"),None,None)
}

pub(crate) fn icon_with_options(icon:icondata_core::Icon,width:Option<&str>,height:Option<&str>,style:Option<&str>,class:Option<&str>) -> impl IntoView {
    let style = match (style,icon.style) {
        (Some(a),Some(b)) => format!("{b} {}",a),
        (Some(a),None) => a.to_string(),
        (None,Some(b)) => b.to_string(),
        (None,None) => "vertical-align:sub;".to_string(),
    };
    view! {
        <div style="display:inline-block;margin:auto">
        <svg
            x=icon.x y=icon.y style=style
            width=width.map(|w| w.to_string()) height=height.map(|w| w.to_string())
            viewBox=icon.view_box.map(|view_box| view_box.to_string())
            stroke-linecap=icon.stroke_linecap.map(|a| a.to_string())
            stroke-linejoin=icon.stroke_linejoin.map(|a| a.to_string())
            stroke-width=icon.stroke_width.map(|a| a.to_string())
            stroke=icon.stroke.map(|a| a.to_string())
            fill=icon.fill.unwrap_or("currentColor").to_string()
            inner_html=icon.data.to_string()
        ></svg></div>
    }
}

pub(crate) fn wait_blocking<T,Fut,V:IntoView + 'static>(
                        fetcher: impl Fn() -> Fut + Send + Sync + 'static,
                          f: impl (FnMut(T) -> V) + Clone + Send + 'static
) -> impl IntoView
    where
        T: Send + Sync + Clone + serde::Serialize + for<'de>serde::Deserialize<'de> + 'static,
        Fut: Future<Output = T> + Send + 'static {
    let resource = Resource::new_blocking(|| (),move |_| fetcher());
    view!{
        <Suspense fallback= || view!(<thaw::Spinner/>)>{move || {
            resource.get().map(f.clone())
        }}</Suspense>
    }
}

pub(crate) fn wait<T,Fut,V:IntoView + 'static>(
    fetcher: impl Fn() -> Fut + Send + Sync + 'static,
    f: impl (FnMut(T) -> V) + Clone + Send + 'static
) -> impl IntoView
    where
        T: Send + Sync + Clone + serde::Serialize + for<'de>serde::Deserialize<'de> + 'static,
        Fut: Future<Output = T> + Send + 'static {
    let resource = Resource::new(|| (),move |_| fetcher());
    view!{
        <Suspense fallback= || view!(<thaw::Spinner/>)>{move || {
            resource.get().map(f.clone())
        }}</Suspense>
    }
}

#[component]
pub(crate) fn IFrame(src:String,#[prop(optional,into)] ht:String) -> impl IntoView {
    view!(<iframe src=format!("/{src}") style=if ht.is_empty() {
        "width:100%;border: 0;".to_string()
    } else {
        format!("width:100%;height:{ht};border: 0;")
    }></iframe>)
}