use leptos::prelude::*;

use crate::inject_css;

#[derive(Default, Clone)]
pub enum SpinnerSize {
    ExtraTiny,
    Tiny,
    ExtraSmall,
    Small,
    #[default]
    Medium,
    Large,
    ExtraLarge,
    Huge,
}

impl SpinnerSize {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ExtraTiny => "extra-tiny",
            Self::Tiny => "tiny",
            Self::ExtraSmall => "extra-small",
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
            Self::ExtraLarge => "extra-large",
            Self::Huge => "huge",
        }
    }
}

#[component]
pub fn Spinner(
    /// An optional label for the Spinner.
    #[prop(optional, into)]
    label: MaybeProp<String>,
    /// The size of the spinner.
    #[prop(optional, into)]
    size: Signal<SpinnerSize>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    inject_css("immt-spinner", include_str!("./spinner.css"));

    let cls = format!("thaw-spinner thaw-spinner--{}", size.get().as_str());

    view! {
        <div
            class=cls
            role="progressbar"
        >
            <span class="thaw-spinner__spinner">
                <span class="thaw-spinner__spinner-tail"></span>
            </span>
            {children.map_or_else(
              || {
                move || label.get().map(|label|
                  view! {
                      <label class="thaw-spinner__label">
                          {label}
                      </label>
                  }.into_any()
                )
              }.into_any(),
              |children|  view! {
                <label class="thaw-spinner__label">
                    {children()}
                </label>
            }.into_any()
            )}
        </div>
    }
}
