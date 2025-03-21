#![allow(clippy::must_use_candidate)]

mod login_state;
pub use login_state::*;
pub mod uris;
pub mod ws;

use leptos::{either::EitherOf3, prelude::*};

#[component]
pub fn RequireLogin<Ch: IntoView + 'static>(children: TypedChildren<Ch>) -> impl IntoView {
    require_login(children.into_inner())
}

pub fn require_login<Ch: IntoView + 'static>(
    children: impl FnOnce() -> Ch + Send + 'static,
) -> impl IntoView {
    use flams_web_utils::components::{Spinner, display_error};

    let children = std::sync::Arc::new(flams_utils::parking_lot::Mutex::new(Some(children)));
    move || match LoginState::get() {
        LoginState::Loading => EitherOf3::A(view!(<Spinner/>)),
        LoginState::Admin | LoginState::NoAccounts | LoginState::User { is_admin: true, .. } => {
            EitherOf3::B((children.clone().lock().take()).map(|f| f()))
        }
        _ => EitherOf3::C(view!(<div>{display_error("Not logged in".into())}</div>)),
    }
}

#[cfg(feature = "ssr")]
/// #### Errors
pub fn get_oauth() -> Result<(flams_git::gl::auth::GitLabOAuth, String), String> {
    use flams_git::gl::auth::GitLabOAuth;
    use leptos::prelude::*;
    let Some(session) = use_context::<axum_login::AuthSession<flams_database::DBBackend>>() else {
        return Err("Internal Error".to_string());
    };
    let Some(user) = session.user else {
        return Err("Not logged in".to_string());
    };
    let Some(oauth): Option<GitLabOAuth> = expect_context() else {
        return Err("Not Gitlab integration set up".to_string());
    };
    Ok((oauth, user.secret))
}
