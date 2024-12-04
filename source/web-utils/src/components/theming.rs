#![allow(clippy::must_use_candidate)]

use leptos::prelude::*;

#[derive(Copy,Clone,PartialEq,Eq,Debug,Default,serde::Serialize,serde::Deserialize)]
pub enum ThemeType { #[default] Light,Dark }
impl<'a> From<&'a thaw::Theme> for ThemeType {
    fn from(theme: &'a thaw::Theme) -> Self {
        if theme.name == "dark" {
            Self::Dark
        } else {
            Self::Light
        }
    }
}
impl From<ThemeType> for thaw::Theme {
  fn from(tp:ThemeType) -> Self {
      match tp {
          ThemeType::Light => Self::light(),
          ThemeType::Dark => Self::dark()
      }
  }
}

#[component(transparent)]
pub fn Themer<Ch:IntoView+'static>(children:TypedChildren<Ch>) -> impl IntoView {
    use thaw::{ConfigProvider,ToasterProvider,Theme};
    #[cfg(feature = "hydrate")]
    use gloo_storage::Storage;
    #[cfg(feature="ssr")]
    let signal = RwSignal::<thaw::Theme>::new(Theme::light());
    #[cfg(feature = "hydrate")]
    let signal = {
        let sig = gloo_storage::LocalStorage::get("theme")
            .map_or_else(|_| RwSignal::<thaw::Theme>::new(Theme::light()),
            |theme:ThemeType| RwSignal::<thaw::Theme>::new(theme.into())
        );
        Effect::new(move || {
            sig.with(move |theme|
                {let _ = gloo_storage::LocalStorage::set("theme",ThemeType::from(theme));}
            );
        });
        sig
    };
    let children = children.into_inner();
    provide_context(signal);
    view!{
      <ConfigProvider theme=signal>
        <ToasterProvider>{children()}</ToasterProvider>
      </ConfigProvider>
    }
}