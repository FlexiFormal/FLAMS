use std::borrow::Cow;

use thaw::{ToastOptions, ToastPosition, ToasterInjection, MessageBar,MessageBarBody,MessageBarIntent};
use leptos::prelude::*;

pub fn error_toast(err:Cow<'static,str>) -> impl leptos::IntoView {
  let err = err.to_string();
  #[cfg(any(feature="hydrate",feature="csr"))]
  {
    let s = err.clone();
    let toaster = ToasterInjection::expect_context();
    toaster.dispatch_toast(
      move || view!{
        <MessageBar intent=MessageBarIntent::Error>
          <MessageBarBody>{s}</MessageBarBody>
        </MessageBar>
      },
      ToastOptions::default().with_position(ToastPosition::Top)
    );
  }
  view!(<h3 style="color:red">"Error: "{err}</h3>)
}