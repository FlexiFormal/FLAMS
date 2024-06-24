use leptos::*;
use crate::accounts::LoginState;
use crate::console_log;
use crate::utils::{if_logged_in_client, target};

#[component]
pub fn Settings() -> impl IntoView {
    move || view!(<Test>"Settings"</Test>)
}

#[island]
fn Test(children:Children) -> impl IntoView {
    let login = expect_context::<RwSignal<LoginState>>();
    crate::console_log!("Here: Checking login in {} mode: {:?}",target(),login.get_untracked());
    match login.get() {
        LoginState::Admin => children(),
        _ => view!{<div>"Please log in to view this content"</div><span/>}.into()
    }
}