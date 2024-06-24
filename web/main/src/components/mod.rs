pub mod mathhub_tree;
pub mod graph_viewer;
pub mod logging;
pub mod queue;
pub mod settings;

pub use mathhub_tree::ArchiveOrGroups;
pub use graph_viewer::GraphTest;
pub use logging::FullLog;
pub use queue::Queue;
pub use settings::Settings;

use std::future::Future;
use leptos::*;
use leptos::error::*;

use thaw::{Spinner, SpinnerSize};

pub fn with_spinner<S, T, E, Fu,A:Clone+'static,V:IntoView + 'static>(
    source: impl Fn() -> S + 'static,
    fetcher: impl Fn(S) -> Fu + 'static,
    args: A,
    then: impl (Fn(A,T) -> V) + Copy + 'static
) -> impl IntoView
    where
        S: PartialEq + Clone + 'static,
        E: Clone + Serializable + serde::Serialize + for<'a> serde::Deserialize<'a> + 'static + std::fmt::Debug,
        T: Serializable + Clone + 'static + serde::Serialize + for<'a> serde::Deserialize<'a> + std::fmt::Debug,
        Fu: Future<Output = Result<T,ServerFnError<E>>> + 'static,
{
    let resource = create_resource(source, fetcher);
    template! {<Suspense fallback=|| template! {<Spinner size=SpinnerSize::Tiny/>}>{
            let res = resource.get();
            let args = args.clone();
            template!{<ErrorBoundary fallback=|_| {template! {<p>"Something went wrong"</p>}}>{
                res.map(move |x| x.ok().map(|t| then(args,t)) )
            }</ErrorBoundary>}
        }</Suspense>}
}

pub fn with_spinner_simple<E,T,Fu,V:IntoView + 'static>(
    fut: impl Fn() -> Fu + 'static,
    then: impl (Fn(T) -> V) + Copy + 'static
) -> impl IntoView where
    E: Clone + Serializable + serde::Serialize + for<'a> serde::Deserialize<'a> + 'static + std::fmt::Debug,
    T: Serializable + Clone + 'static + serde::Serialize + for<'a> serde::Deserialize<'a> + std::fmt::Debug,
    Fu:Future<Output = Result<T,E>> + 'static
    {
    let resource = create_resource(|| (),move |_| fut());
    template! {<Suspense fallback=|| template! {<Spinner size=SpinnerSize::Tiny/>}>{
        let res = resource.get();
        template!{<ErrorBoundary fallback=|_| {template! {<p>"Something went wrong"</p>}}>{
            res.map(move |x| x.ok().map(|t| then(t)) )
        }</ErrorBoundary>}
    }</Suspense>}
}