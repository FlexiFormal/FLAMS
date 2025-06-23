use std::future::Future;

use crate::components::{display_error, Spinner};
use flams_utils::parking_lot;
use leptos::{
    either::{Either, EitherOf3},
    prelude::*,
};

pub fn wait_local<
    V: IntoView + 'static,
    Out: 'static + Send + Sync + Clone,
    Fut: Future<Output = Option<Out>> + 'static + Send,
    F: Fn() -> Fut + 'static,
>(
    future: F,
    children: impl Fn(Out) -> V + 'static + Send,
    err: String,
) -> impl IntoView {
    let res = LocalResource::new(future);
    view! {
      <Suspense fallback = || view!(<Spinner/>)>{move || {
        res.get().and_then(|mut r| r.take()).map_or_else(
          || Either::Left(view!(<div>{err.clone()}</div>)),
          |res| Either::Right(children(res))
        )
      }}</Suspense>
    }
}

pub fn wait_and_then<E, Fut, F, T, V: IntoView + 'static>(
    f: F,
    r: impl FnOnce(T) -> V + Send + 'static,
) -> impl IntoView
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
                EitherOf3::A(r.lock().take().map(|r| r(t))),
              Some(Err(e)) => EitherOf3::B(display_error(e.to_string().into())),
              None => EitherOf3::C(view!(<Spinner/>)),
            }
        }</Suspense>
    }
}

pub fn wait_and_then_fn<E, Fut, F, T, V: IntoView + 'static>(
    f: F,
    r: impl Fn(T) -> V + 'static + Send,
) -> impl IntoView
where
    Fut: Future<Output = Result<T, ServerFnError<E>>> + Send + 'static,
    F: Fn() -> Fut + 'static + Send + Sync,
    T: Send + Sync + Clone + 'static + serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: std::fmt::Display
        + Clone
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + Send
        + Sync
        + 'static,
{
    let res = Resource::new(|| (), move |()| f());
    view! {
        <Suspense fallback = || view!(<Spinner/>)>{move ||
            match res.get() {
              Some(Ok(t)) =>
                EitherOf3::A(r(t)),
              Some(Err(e)) => EitherOf3::B(display_error(e.to_string().into())),
              None => EitherOf3::C(view!(<Spinner/>)),
            }
        }</Suspense>
    }
}
