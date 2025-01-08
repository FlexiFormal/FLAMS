use std::borrow::Cow;

use thaw::{ToastOptions, ToastPosition, ToasterInjection, MessageBar,MessageBarBody,MessageBarIntent};
use leptos::prelude::*;

#[inline]
pub fn display_error(err:Cow<'static,str>) -> impl leptos::IntoView {
  #[cfg(any(feature="hydrate",feature="csr"))]
  { error_toast(err.clone()); }
  view!(<h3 style="color:red">"Error: "{err}</h3>)
}

pub fn message_action<
  I: Clone+Send+Sync+'static,
  O,
  E:std::fmt::Display + Send + 'static,
  V: std::fmt::Display + Send + 'static,
  Fut: std::future::Future<Output = Result<O,E>> + Send
>(
  run:impl Fn(I) -> Fut + Send + Sync + Clone + 'static,
  msg:impl Fn(O) -> V + Send + Sync + Clone + 'static
) -> Action<I,()> {
  let toaster = ToasterInjection::expect_context();
  Action::new(move |args:&I| {
    let run = run.clone();
    let msg = msg.clone();
    let args = args.clone();
    async move {
      match run(args).await {
        Ok(r) => success_with_toaster(msg(r),toaster),
        Err(e) => error_with_toaster(e,toaster),
      }
    }
  })
}

#[inline]
pub fn error_toast(err:impl std::fmt::Display + Send + 'static) {
  let toaster = ToasterInjection::expect_context();
  error_with_toaster(err, toaster);
}

#[inline]
pub fn success_toast(msg:impl std::fmt::Display + Send + 'static) {
  let toaster = ToasterInjection::expect_context();
  success_with_toaster(msg, toaster);
}

pub fn error_with_toaster(err:impl std::fmt::Display + Send + 'static,toaster:ToasterInjection) {
  tracing::error!("{err}");
  toaster.dispatch_toast(
    move || view!{
      <MessageBar intent=MessageBarIntent::Error>
        <MessageBarBody>{err.to_string()}</MessageBarBody>
      </MessageBar>
    },
    ToastOptions::default().with_position(ToastPosition::Top)
  );
}

fn success_with_toaster(msg:impl std::fmt::Display + Send + 'static,toaster:ToasterInjection) {
  tracing::info!("{msg}");
  toaster.dispatch_toast(
    move || view!{
      <MessageBar intent=MessageBarIntent::Success>
        <MessageBarBody>{msg.to_string()}</MessageBarBody>
      </MessageBar>
    },
    ToastOptions::default().with_position(ToastPosition::Top)
  );
}