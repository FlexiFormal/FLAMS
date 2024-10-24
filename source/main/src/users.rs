#![allow(clippy::must_use_candidate)]

use std::{fmt::Display, str::FromStr};
use leptos::prelude::*;
#[cfg(feature="hydrate")]
use std::borrow::Cow;
#[cfg(feature="hydrate")]
use immt_web_utils::components::error_toast;

#[server(prefix="/api",endpoint="login")]//, input=server_fn::codec::Cbor)]
pub async fn login(username:String,password:String) -> Result<LoginState,ServerFnError<LoginError>> {
    use argon2::PasswordVerifier;
    use axum_login::AuthnBackend;
    let mut session: axum_login::AuthSession<crate::server::db::DBBackend> = expect_context();
    if session.backend.admin.is_none() { return Ok(LoginState::NoAccounts) }
    if username == "admin" {
        let (pass_hash,salt) = session.backend.admin.as_ref().unwrap_or_else(|| unreachable!());
        let hash = password_hash::PasswordHash::parse(pass_hash, password_hash::Encoding::B64)
            .map_err(|e| ServerFnError::WrappedServerError(e.into()))?;
        let hasher = argon2::Argon2::default();
        return if hasher.verify_password(password.as_bytes(), &hash).is_ok() {
            session.login(&User {id:0,username:"admin".to_string(),session_auth_hash:salt.as_bytes().to_owned()}).await
                .map_err(|e| {
                    leptos::logging::log!("Wut: {e}");
                    ServerFnError::WrappedServerError(e.into())
                })?;
            Ok(LoginState::Admin)
        } else {
            Err(LoginError::WrongUsernameOrPassword.into())
        }
    }
    match session.backend.authenticate((username,password)).await? {
        Some(user) => {
            session.login(&user).await
                .map_err(|e| ServerFnError::WrappedServerError(e.into()))?;
            Ok(LoginState::User(user.username))
        }
        _ => Ok(LoginState::None)
    }
}


#[server(LoginStateFn,prefix="/api",endpoint="login_state")]
#[allow(clippy::unused_async)]
pub async fn login_state() -> Result<LoginState,ServerFnError<String>> {
    Ok(LoginState::get())
}

#[derive(Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub session_auth_hash: Vec<u8>,
}


#[derive(Clone,serde::Serialize,serde::Deserialize,Debug,PartialEq,Eq)]
pub enum LoginState {
    Loading,Admin,User(String),None,NoAccounts
}
#[cfg(feature="ssr")]
impl LoginState {
    #[must_use]
    pub fn get() -> Self {
        let session: axum_login::AuthSession<crate::server::db::DBBackend> = expect_context();
        match &session.backend.admin {
            None => Self::NoAccounts,
            Some(_) => match session.user {
                None => Self::None,
                Some(User{id:0,username,..}) if username == "admin" => Self::Admin,
                Some(u) => Self::User(u.username)
            }
        }
    }
}

impl Display for LoginState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Loading => write!(f,"Loading"),
            Self::Admin => write!(f,"Admin"),
            Self::User(u) => write!(f,"User: {u}"),
            Self::None => write!(f,"None"),
            Self::NoAccounts => write!(f,"No accounts")
        }
    }
}

#[derive(Debug,Copy,Clone,serde::Serialize,serde::Deserialize,PartialEq,Eq)]
pub enum LoginError {
    WrongUsernameOrPassword,
    InternalError,
    NotLoggedIn
}
impl Display for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WrongUsernameOrPassword => write!(f,"Wrong username or password"),
            Self::InternalError => write!(f,"Internal error"),
            Self::NotLoggedIn => write!(f,"Not logged in"),
        }
    }
}
impl FromStr for LoginError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Wrong username or password" => Ok(Self::WrongUsernameOrPassword),
            "Internal error" => Ok(Self::InternalError),
            "Not logged in" => Ok(Self::NotLoggedIn),
            _ => Err(()),
        }
    }
}

#[component]
pub(crate) fn Login(children:Children) -> impl IntoView {
    use immt_web_utils::components::Spinner;
    let user = RwSignal::new(LoginState::Loading);
    provide_context(user);
    #[cfg(feature="hydrate")]
    let toaster = thaw::ToasterInjection::expect_context();
    let res = Resource::new_blocking(|| (),move |()| async move {
      #[cfg(feature="ssr")]
      { LoginState::get() }
      #[cfg(feature="hydrate")]
      {
        login_state().await.unwrap_or_else(|e| {
          error_toast(Cow::Owned(format!("Error: {e}")),toaster);
          LoginState::None
        })
      }
    });

    view!{
        {move || res.get().map_or_else(|| Some(view!(<Spinner/>)),|u| { user.set(u); None}) }
        {children()}
    }
}