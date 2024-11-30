use std::future::Future;

use leptos::prelude::*;
use crate::components::Spinner;

pub fn wait<
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
        res.get().and_then(|r| r.take()).map_or_else(
          || view!(<div>{err.clone()}</div>).into_any(),
          |res| children(res).into_any()
        )
      }}</Suspense>
    }
}