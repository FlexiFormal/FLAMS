use flams_database::{LoginError, UserData};
use flams_router_base::LoginState;
use leptos::prelude::*;

#[server(prefix = "/api", endpoint = "login")]
pub async fn login(
    admin_pwd: Option<String>,
) -> Result<Option<LoginState>, ServerFnError<LoginError>> {
    ssr::login(admin_pwd).await
}

#[server(prefix = "/api", endpoint = "logout")]
pub async fn logout() -> Result<(), ServerFnError<LoginError>> {
    ssr::logout().await
}

#[server(LoginStateFn, prefix = "/api", endpoint = "login_state")]
#[allow(clippy::unused_async)]
pub async fn login_state() -> Result<LoginState, ServerFnError<String>> {
    Ok(LoginState::get_server())
}

#[server(prefix = "/api", endpoint = "get_users")]
pub async fn get_users() -> Result<Vec<UserData>, ServerFnError<LoginError>> {
    ssr::get_users().await
}

#[server(prefix = "/api", endpoint = "set_admin")]
pub async fn set_admin(user_id: i64, is_admin: bool) -> Result<(), ServerFnError<LoginError>> {
    ssr::set_admin(user_id, is_admin).await
}

#[cfg(feature = "ssr")]
mod ssr {
    use flams_database::{DBBackend, LoginError, UserData};
    use flams_git::gl::auth::GitLabOAuth;
    use flams_router_base::LoginState;
    use leptos::prelude::*;

    pub(super) async fn login(
        admin_pwd: Option<String>,
    ) -> Result<Option<LoginState>, ServerFnError<LoginError>> {
        let Some(session) = use_context::<axum_login::AuthSession<DBBackend>>() else {
            return Ok(Some(LoginState::None));
        };
        if session.backend.admin.is_none() {
            return Ok(Some(LoginState::NoAccounts));
        }
        if let Some(password) = admin_pwd {
            return if DBBackend::login_as_admin(&password, session).await.is_ok() {
                Ok(Some(LoginState::Admin))
            } else {
                Err(LoginError::WrongUsernameOrPassword.into())
            };
        }
        let oauth: Option<GitLabOAuth> = expect_context();
        if let Some(oauth) = oauth.as_ref() {
            leptos_axum::redirect(oauth.login_url().as_str());
        } else {
            leptos_axum::redirect("/dashboard");
        }
        Ok(None)
    }

    pub async fn logout() -> Result<(), ServerFnError<LoginError>> {
        let Some(mut session) = use_context::<axum_login::AuthSession<DBBackend>>() else {
            return Ok(());
        };
        let _ = session.logout().await;
        leptos_axum::redirect("/dashboard");
        Ok(())
    }

    pub(super) async fn get_users() -> Result<Vec<UserData>, ServerFnError<LoginError>> {
        match LoginState::get_server() {
            LoginState::Admin | LoginState::NoAccounts => (),
            _ => return Err(ServerFnError::WrappedServerError(LoginError::NotLoggedIn)),
        }
        let Some(session) = use_context::<axum_login::AuthSession<DBBackend>>() else {
            return Ok(Vec::new());
        };
        let users = session
            .backend
            .all_users()
            .await
            .map_err(|_| ServerFnError::WrappedServerError(LoginError::NotLoggedIn))?;
        let mut users: Vec<_> = users.into_iter().map(UserData::from).collect();
        users.sort_by_key(|e| e.id);
        Ok(users)
    }

    pub async fn set_admin(user_id: i64, is_admin: bool) -> Result<(), ServerFnError<LoginError>> {
        match LoginState::get_server() {
            LoginState::Admin | LoginState::NoAccounts => (),
            _ => return Err(ServerFnError::WrappedServerError(LoginError::NotLoggedIn)),
        }
        let Some(session) = use_context::<axum_login::AuthSession<DBBackend>>() else {
            return Ok(());
        };
        session
            .backend
            .set_admin(user_id, is_admin)
            .await
            .map_err(|_| ServerFnError::WrappedServerError(LoginError::NotLoggedIn))?;
        Ok(())
    }
}
