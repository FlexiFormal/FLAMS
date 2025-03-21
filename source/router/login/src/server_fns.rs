use crate::{LoginError, LoginState, users::UserData};
use leptos::prelude::*;

#[server(prefix = "/api", endpoint = "login")]
pub async fn login(
    admin_pwd: Option<String>,
) -> Result<Option<LoginState>, ServerFnError<LoginError>> {
    ssr::login(admin_pwd).await
}

#[server(prefix = "/api", endpoint = "logout")]
pub async fn logout() -> Result<(), ServerFnError<LoginError>> {
    let Some(mut session) = use_context::<axum_login::AuthSession<crate::db::DBBackend>>() else {
        return Ok(());
    };
    let _ = session.logout().await;
    leptos_axum::redirect("/dashboard");
    Ok(())
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
    use crate::{
        LoginError, LoginState,
        db::DBBackend,
        users::{ServerUser, UserData},
    };
    use argon2::PasswordVerifier;
    use flams_git::gl::auth::GitLabOAuth;
    use leptos::prelude::*;

    pub(super) async fn login(
        admin_pwd: Option<String>,
    ) -> Result<Option<LoginState>, ServerFnError<LoginError>> {
        let Some(mut session) = use_context::<axum_login::AuthSession<DBBackend>>() else {
            return Ok(Some(LoginState::None));
        };
        if session.backend.admin.is_none() {
            return Ok(Some(LoginState::NoAccounts));
        }
        if let Some(password) = admin_pwd {
            let (pass_hash, salt) = session
                .backend
                .admin
                .as_ref()
                .unwrap_or_else(|| unreachable!());
            let hash = password_hash::PasswordHash::parse(pass_hash, password_hash::Encoding::B64)
                .map_err(|e| ServerFnError::WrappedServerError(e.into()))?;
            let hasher = argon2::Argon2::default();
            if hasher.verify_password(password.as_bytes(), &hash).is_ok() {
                session
                    .login(&ServerUser::admin(salt.as_bytes().to_owned()))
                    .await
                    .map_err(|e| ServerFnError::WrappedServerError(e.into()))?;
                leptos_axum::redirect("/dashboard");
                return Ok(Some(LoginState::Admin));
            } else {
                return Err(LoginError::WrongUsernameOrPassword.into());
            };
        }
        let oauth: Option<GitLabOAuth> = expect_context();
        if let Some(oauth) = oauth.as_ref() {
            leptos_axum::redirect(oauth.login_url().as_str())
        } else {
            leptos_axum::redirect("/dashboard")
        }
        Ok(None)
    }

    pub(super) async fn get_users() -> Result<Vec<UserData>, ServerFnError<LoginError>> {
        match LoginState::get_server() {
            LoginState::Admin | LoginState::NoAccounts => (),
            _ => return Err(ServerFnError::WrappedServerError(LoginError::NotLoggedIn)),
        }
        let Some(session) = use_context::<axum_login::AuthSession<DBBackend>>() else {
            return Ok(Vec::new());
        };
        let mut users = session
            .backend
            .all_users()
            .await
            .map_err(|_| ServerFnError::WrappedServerError(LoginError::NotLoggedIn))?;
        users.sort_by_key(|e| e.id);
        Ok(users
            .into_iter()
            .map(|u| UserData {
                id: u.id,
                name: u.name,
                username: u.username,
                email: u.email,
                avatar_url: u.avatar_url,
                is_admin: u.is_admin,
            })
            .collect())
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
