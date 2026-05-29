use std::future::Future;

use crate::components::display_error;
use flams_utils::parking_lot;
use ftml_component_utils::Spinner;
use leptos::{
    either::{Either, EitherOf3},
    prelude::*,
};

pub fn wait_local<
    Out: 'static + Send + Sync + Clone,
    Fut: Future<Output = Option<Out>> + 'static + Send,
    F: Fn() -> Fut + 'static,
>(
    future: F,
    children: impl Fn(Out) -> AnyView + 'static + Send,
    err: String,
) -> AnyView {
    let res = LocalResource::new(future);
    view! {
      <Suspense fallback = || view!(<Spinner/>)>{move || {
        res.get().and_then(|mut r| r.take()).map_or_else(
          || view!(<div>{err.clone()}</div>).into_any(),
          |res| children(res)
        )
      }}</Suspense>
    }
    .into_any()
}

pub fn wait_and_then<E, Fut, F, T>(f: F, r: impl FnOnce(T) -> AnyView + Send + 'static) -> AnyView
where
    Fut: Future<Output = Result<T, ServerFnError<E>>> + Send + 'static,
    F: Fn() -> Fut + Send + Sync + 'static,
    T: Send + Sync + Clone + 'static + serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: std::fmt::Display
        + Clone
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + Send
        + Sync
        + 'static,
{
    let r = std::sync::Arc::new(parking_lot::Mutex::new(Some(r)));
    let res = Resource::new(|| (), move |()| f());
    view! {
        <Suspense fallback = || view!(<Spinner/>)>{move ||
            match res.get() {
              Some(Ok(t)) =>
                r.lock().take().map(|r| r(t)).into_any(),
              Some(Err(e)) => display_error(e.to_string().into()),
              None => view!(<Spinner/>).into_any(),
            }
        }</Suspense>
    }
    .into_any()
}

pub fn wait_and_then_fn<E, Fut, F, T>(f: F, r: impl Fn(T) -> AnyView + Send + 'static) -> AnyView
where
    Fut: Future<Output = Result<T, E>> + Send + 'static,
    F: Fn() -> Fut + 'static + Send + Sync,
    T: Send + Sync + Clone + 'static + serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: std::fmt::Display
        + Clone
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + Send
        + Sync
        + FromServerFnError
        + 'static,
{
    let res = Resource::new(|| (), move |()| f());
    view! {
        <Suspense fallback = || view!(<Spinner/>)>{move ||
            match res.get() {
              Some(Ok(t)) =>
                r(t),
              Some(Err(e)) => display_error(e.to_string().into()),
              None => view!(<Spinner/>).into_any(),
            }
        }</Suspense>
    }
    .into_any()
}
