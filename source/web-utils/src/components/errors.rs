use std::borrow::Cow;

use thaw::{ToastOptions, ToastPosition,MessageBar,MessageBarBody,MessageBarIntent};
use leptos::prelude::{view, IntoAny};

pub fn error_toast(err:Cow<'static,str>,toaster:thaw::ToasterInjection) {
  toaster.dispatch_toast(
    view!{
      <MessageBar intent=MessageBarIntent::Error>
        <MessageBarBody>{err}</MessageBarBody>
      </MessageBar>
    }.into_any(),
    ToastOptions::default().with_position(ToastPosition::Top)
  );
}