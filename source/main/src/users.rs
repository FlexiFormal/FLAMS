#![allow(clippy::must_use_candidate)]

use std::{fmt::Display, str::FromStr};
use leptos::prelude::*;
#[cfg(feature="hydrate")]
use std::borrow::Cow;
//#[cfg(feature="hydrate")]
//use flams_web_utils::components::error_toast;

#[server(prefix="/api",endpoint="login")]
pub async fn login(admin_pwd:Option<String>) -> Result<Option<LoginState>,ServerFnError<LoginError>> {
    use flams_git::gl::auth::GitLabOAuth;
    use argon2::PasswordVerifier;
    let Some(mut session)= use_context::<axum_login::AuthSession<crate::server::db::DBBackend>>() else {
        return Ok(Some(LoginState::None))
    };
    if session.backend.admin.is_none() { return Ok(Some(LoginState::NoAccounts)) }
    if let Some(password) = admin_pwd {
        let (pass_hash,salt) = session.backend.admin.as_ref().unwrap_or_else(|| unreachable!());
        let hash = password_hash::PasswordHash::parse(pass_hash, password_hash::Encoding::B64)
            .map_err(|e| ServerFnError::WrappedServerError(e.into()))?;
        let hasher = argon2::Argon2::default();
        return if hasher.verify_password(password.as_bytes(), &hash).is_ok() {
            session.login(&User::admin(salt.as_bytes().to_owned())).await
                .map_err(|e| {
                    leptos::logging::log!("Wut: {e}");
                    ServerFnError::WrappedServerError(e.into())
                })?;
            leptos_axum::redirect("/dashboard");
            return Ok(Some(LoginState::Admin))
        } else {
            return Err(LoginError::WrongUsernameOrPassword.into())
        }
    }
    let oauth:Option<GitLabOAuth> = expect_context();
    if let Some(oauth) = oauth.as_ref() {
        leptos_axum::redirect(oauth.login_url().as_str())
    } else {
        leptos_axum::redirect("/dashboard")
    }
    Ok(None)
}

#[server(prefix="/api",endpoint="logout")]
pub async fn logout() -> Result<(),ServerFnError<LoginError>> {
    let Some(mut session)= use_context::<axum_login::AuthSession<crate::server::db::DBBackend>>() else {
        return Ok(());
    };
    let _ = session.logout().await;
    leptos_axum::redirect("/dashboard");
    Ok(())
}

#[server(LoginStateFn,prefix="/api",endpoint="login_state")]
#[allow(clippy::unused_async)]
pub async fn login_state() -> Result<LoginState,ServerFnError<String>> {
    Ok(LoginState::get_server())
}

#[derive(Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub session_auth_hash: Vec<u8>,
    pub avatar_url:Option<String>,
    pub is_admin:bool,
    pub secret:String
}
impl User {
    pub(crate) fn admin(hash:Vec<u8>) -> Self {
        Self {
            id:0,
            username:"admin".to_string(),
            session_auth_hash:hash,
            secret:String::new(),
            is_admin:true,
            avatar_url:None
        }
    }
}


#[derive(Clone,serde::Serialize,serde::Deserialize,Debug,PartialEq,Eq)]
pub enum LoginState {
    Loading,Admin,User{
        name:String,
        avatar:String,
        is_admin:bool
    },None,NoAccounts
}
#[cfg(feature="ssr")]
impl LoginState {
    #[must_use]
    pub fn get_server() -> Self {
        let Some(session) = use_context::<axum_login::AuthSession<crate::server::db::DBBackend>>() else {
            return Self::None;
        };
        match &session.backend.admin {
            None => Self::NoAccounts,
            Some(_) => match session.user {
                None => Self::None,
                Some(User{id:0,username,..}) if username == "admin" => Self::Admin,
                Some(u) => Self::User{name:u.username,avatar:u.avatar_url.unwrap_or_default(),is_admin:u.is_admin}
            }
        }
    }
}

impl Display for LoginState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Loading => write!(f,"Loading"),
            Self::Admin => write!(f,"Admin"),
            Self::User{name,..} => write!(f,"User: {name}"),
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

impl LoginState {
    #[inline]
    pub fn get() -> Self {
        let ctx:RwSignal<Self> = expect_context();
        ctx.get()
    }
}

#[component(transparent)]
pub(crate) fn Login<Ch:IntoView+'static>(children:TypedChildren<Ch>) -> impl IntoView {
    use flams_web_utils::components::Spinner;
    let children = children.into_inner();
    let res = Resource::new_blocking(|| (),|()| async {
        login_state().await.unwrap_or_else(|e| {
            leptos::logging::error!("Error getting login state: {e}");
        //error_toast(Cow::Owned(format!("Error: {e}")),toaster);
            LoginState::None
        })
    });
    let sig = RwSignal::new(LoginState::Loading);
    let _ = view!{<Suspense>{move || {res.get();()}}</Suspense>};
    //if let Some(v) = res.get_untracked() {sig.set(v)}
    let _ = Effect::new(move |_| if let Some(r) = res.get() {sig.set(r)});
    provide_context(sig);
    children()
}