#![allow(clippy::must_use_candidate)]

use std::{fmt::Display, str::FromStr};
use leptos::prelude::*;
#[cfg(feature="hydrate")]
use std::borrow::Cow;
//#[cfg(feature="hydrate")]
//use immt_web_utils::components::error_toast;

#[server(prefix="/api",endpoint="login")]//, input=server_fn::codec::Cbor)]
pub async fn login(username:String,password:String) -> Result<LoginState,ServerFnError<LoginError>> {
    use argon2::PasswordVerifier;
    use axum_login::AuthnBackend;
    let Some(mut session)= use_context::<axum_login::AuthSession<crate::server::db::DBBackend>>() else {
        return Ok(LoginState::None)
    };
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
    Ok(LoginState::get_server())
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
    pub fn get_server() -> Self {
        let Some(session) = use_context::<axum_login::AuthSession<crate::server::db::DBBackend>>() else {
            return Self::None;
        };
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

impl LoginState {
    #[inline]
    pub fn get() -> Self {
        let ctx:RwSignal<Self> = expect_context();
        ctx.get()
    }
}

#[component(transparent)]
pub(crate) fn Login<Ch:IntoView+'static>(children:TypedChildren<Ch>) -> impl IntoView {
    use immt_web_utils::components::Spinner;
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
/*


    #[cfg(feature="ssr")]
    let user = RwSignal::new(LoginState::get());
    #[cfg(not(feature="ssr"))]
    let user = RwSignal::new(LoginState::Loading);
    provide_context(user);
    //#[cfg(feature="hydrate")]
    //let toaster = thaw::ToasterInjection::expect_context();
    let res = Resource::new_blocking(|| (),move |()| async move {
        match user.get() {
            LoginState::Loading => (),
            u => return ()
        }
        #[cfg(feature="ssr")]
        #[allow(clippy::needless_return)]
        { user.set(LoginState::get()) }
        #[cfg(feature="hydrate")]
        {
            let r = login_state().await.unwrap_or_else(|e| {
                leptos::logging::error!("Error getting login state: {e}");
            //error_toast(Cow::Owned(format!("Error: {e}")),toaster);
                LoginState::None
            });
            user.set(r);
        }
    });

    view!{
        {move || match user.get() {
            LoginState::Loading => Some(view!{
                <Suspense fallback = || view!(<Spinner/>)>
                    {move || {res.get();match user.get() {
                        LoginState::Loading => Some(view!(<Spinner/>)),
                        _ => None
                    }}}
                </Suspense>
            }),
            _ => None
        }}
        {children()}
    }
     */
}