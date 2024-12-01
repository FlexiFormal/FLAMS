use std::borrow::Cow;

use thaw::{ToastOptions, ToastPosition, ToasterInjection, MessageBar,MessageBarBody,MessageBarIntent};
use leptos::prelude::{view, IntoAny};

pub fn error_toast(err:Cow<'static,str>,toaster:ToasterInjection) {
  toaster.dispatch_toast(
    move || view!{
      <MessageBar intent=MessageBarIntent::Error>
        <MessageBarBody>{err}</MessageBarBody>
      </MessageBar>
    },
    ToastOptions::default().with_position(ToastPosition::Top)
  );
}