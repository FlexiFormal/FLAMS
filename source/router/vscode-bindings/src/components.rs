use flams_web_utils::inject_css;
pub use leptos::prelude::*;

#[component]
pub fn VSCodeButton<T: IntoView + 'static>(children: TypedChildren<T>) -> impl IntoView {
    inject_css("flams-vscode-button", include_str!("button.css"));
    let children = children.into_inner();
    view!(<button class="flams-vscode-button">{children()}</button>)
}

struct RadioGroup {
    name: String,
    selected: RwSignal<Option<String>>,
}

#[component]
pub fn VSCodeRadioGroup<T: IntoView + 'static>(
    #[prop(into)] name: String,
    selected: RwSignal<Option<String>>,
    children: TypedChildren<T>,
) -> impl IntoView {
    inject_css("flams-vscode-radio", include_str!("radio.css"));
    let children = children.into_inner();
    provide_context(RadioGroup { name, selected });
    children()
}

#[component]
pub fn VSCodeRadio<T: IntoView + 'static>(
    #[prop(into)] id: String,
    children: TypedChildren<T>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
) -> impl IntoView {
    let children = children.into_inner();
    let Some((name, selected)) = with_context(|g: &RadioGroup| (g.name.clone(), g.selected)) else {
        panic!("VSCodeRadio outside of VSCodeRadioGroup");
    };
    let idc = id.clone();
    let checked = Memo::new(move |_| {
        if selected.with(|s| s.as_ref().is_some_and(|s| *s == idc)) {
            "icon checked"
        } else {
            "icon"
        }
    });
    let top_class = Memo::new(move |_| {
        if disabled.is_some_and(|b| b.get()) {
            "flams-vscode-radio disabled"
        } else {
            "flams-vscode-radio"
        }
    });
    let idc = id.clone();
    let on_click = move |_| {
        if !disabled.is_some_and(|b| b.get()) {
            selected.set(Some(idc.clone()))
        }
    };
    view! {
        <div class=top_class on:click=on_click><div class="wrapper">
            <input type="radio" id=id.clone() name=name checked=checked disabled=disabled/>
            <div class=checked></div>
            <label for=id><div>
                {children()}
            </div></label>
        </div></div>
    }
}

#[component]
pub fn VSCodeCheckbox<T: IntoView + 'static>(
    checked: RwSignal<bool>,
    children: TypedChildren<T>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
) -> impl IntoView {
    inject_css("flams-vscode-checkbox", include_str!("checkbox.css"));
    let children = children.into_inner();

    let top_class = Memo::new(move |_| {
        if disabled.is_some_and(|b| b.get()) {
            "flams-vscode-checkbox disabled"
        } else {
            "flams-vscode-checkbox"
        }
    });
    let on_click = move |_| {
        if !disabled.is_some_and(|b| b.get()) {
            checked.update(|v| *v = !*v)
        }
    };
    view! {
        <div class=top_class on:click=on_click><div class="wrapper">
            <input type="checkbox" checked=checked disabled=disabled/>
            <div class="icon">
                {move || if checked.get() {Some(view!{
                    <svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="currentColor" class="check-icon">
                        <path fill-rule="evenodd" clip-rule="evenodd" d="M14.431 3.323l-8.47 10-.79-.036-3.35-4.77.818-.574 2.978 4.24 8.051-9.506.764.646z"></path>
                    </svg>
                })} else {None}}
            </div>
            <label><div>
                {children()}
            </div></label>
        </div></div>
    }
}

#[component]
pub fn VSCodeTextbox(
    value: RwSignal<String>,
    #[prop(optional)] placeholder: Option<&'static str>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
) -> impl IntoView {
    inject_css("flams-vscode-textbox", include_str!("textbox.css"));
    view! {
        <input type="text" placeholder=placeholder bind:value=value class="flams-vscode-textbox" disabled=disabled></input>
    }
}
