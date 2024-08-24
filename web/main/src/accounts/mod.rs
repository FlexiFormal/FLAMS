use std::fmt::Display;
use leptos::prelude::*;
#[cfg(feature="server")]
pub(crate) use server::*;
use crate::utils::errors::IMMTError;
use leptos::context::Provider;


#[inline(always)]
pub fn if_logged_in<R>(mut yes:impl (FnMut() -> R)+Clone+'static,mut no: impl (FnMut() -> R)+Clone+'static) -> impl FnMut() -> R {
    use crate::accounts::LoginState;
    move || match get_account() {
        LoginState::Admin | LoginState::User(_) | LoginState::NoAccounts => yes(),
        _ => no()
    }
}

#[derive(Clone,serde::Serialize,serde::Deserialize,Debug,PartialEq,Eq)]
pub enum LoginState {
    Loading,Admin,User(UserLogin),None,NoAccounts
}
impl Display for LoginState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoginState::Loading => write!(f,"Loading"),
            LoginState::Admin => write!(f,"Admin"),
            LoginState::User(u) => write!(f,"User: {}",u.name),
            LoginState::None => write!(f,"None"),
            LoginState::NoAccounts => write!(f,"No accounts")
        }
    }
}
#[inline(always)]
pub fn get_account() -> LoginState {
    //#[cfg(feature="server")]
    {expect_context::<RwSignal<LoginState>>().get()}
    /*#[cfg(feature="client")]
    {
        let r = use_context::<LoginState>();
        r.unwrap_or_else(|| {
            crate::console_log!("No user found!");
            panic!("No user found!")
        })
    }*/
}

macro_rules! fut {
    ($e:expr) => {{
        #[cfg(feature="server")]
        {
            use tracing::Instrument;
            $e.in_current_span()
        }
        #[cfg(feature="client")]
        { $e }
    }}
}

//#[cfg(feature="server")]
#[component(transparent)]
pub(crate) fn WithAccount(children:ChildrenFn) -> impl IntoView {
    use thaw::*;
    let user = RwSignal::new(LoginState::Loading);
    provide_context(user);
    crate::components::wait_blocking(|| fut!(login_status()),move |res|
        if let Ok(u) = res {
            let children = children.clone();
            user.set(u);
            Some(children())
        } else {
            println!("Wut!");
            None
        }
    )
}
/*
#[island]
pub(crate) fn WithAccountClient(children:Children,user:LoginState) -> impl IntoView {
    provide_context(user);
    view!{ {children()} }
}

 */

#[cfg(feature="server")]
pub(crate) async fn login_status_with_session(
    session:Option<&axum_login::AuthSession<AccountManager>>,
    _db:impl FnOnce() -> Option<sea_orm::DatabaseConnection>
) -> Option<LoginState> {
    use axum_login::AuthUser;
    use crate::server::ADMIN_PWD;
    use sea_orm::prelude::*;
    use tracing::Instrument;

    if ADMIN_PWD.is_none() { return Some(LoginState::Admin)}
    let identity = session.and_then(|session| session.user.as_ref().map(|u| u.id()));
    identity.or_else(|| Some(LoginState::None))
    /*
    let state: axum::extract::State<crate::server::AppState> = expect_context();
    let db = &state.db;
    let users = immt_web_orm::entities::prelude::User::find().all(&db).in_current_span().await;
    if ADMIN_PWD.is_none() && match users {
        Ok(users) => users.is_empty(),
        _ => true
    } {
        Some(LoginState::Admin)
    }
    else { Some(LoginState::None) }*/
}

#[server(prefix="/api/users",endpoint="login_status")]
#[inline]
pub async fn login_status() -> Result<LoginState,ServerFnError<IMMTError>> {
    use tracing::Instrument;
    login_status_with_session(
        use_context::<axum_login::AuthSession<AccountManager>>().as_ref(),
        || use_context::<sea_orm::DatabaseConnection>()
    ).in_current_span().await.ok_or(IMMTError::ImplementationError.into())
}

#[server(prefix="/api/users",endpoint="login")]//, input=server_fn::codec::Cbor)]
pub async fn login(username:String,password:String) -> Result<LoginState,ServerFnError<IMMTError>> {
    use leptos_axum::redirect;
    use tracing::Instrument;
    //redirect("/dashboard");
    let mut session = use_context::<axum_login::AuthSession<AccountManager>>().unwrap();
    match session.authenticate((username,password)).in_current_span().await.map_err(|_| IMMTError::ImplementationError)? {
        Some(u) => {
            session.login(&u).in_current_span().await.map_err(|_| IMMTError::InvalidCredentials)?;
            Ok(u)
        },
        None => return Err(IMMTError::InvalidCredentials.into())
    }
}


#[cfg(feature="server")]
pub(crate) mod server {
    use std::fmt::{Debug, Display, Formatter};
    use axum_login::UserId;
    use super::{LoginState, UserLogin};

    #[derive(Clone)]
    pub(crate) struct AccountManager(pub(crate) sea_orm::DatabaseConnection);

    impl axum_login::AuthUser for LoginState {
        type Id = Self;

        fn id(&self) -> Self::Id { self.clone() }

        fn session_auth_hash(&self) -> &[u8] { &[] }
    }

    use async_trait::async_trait;
    use tracing::Instrument;
    use crate::utils::errors::IMMTError;

    #[async_trait]
    impl axum_login::AuthnBackend for AccountManager {
        type User = LoginState;
        type Credentials = (String,String);
        type Error = IMMTError;

        async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
            use sea_orm::{EntityTrait,QueryFilter,ColumnTrait};
            use argon2::{
                password_hash::{PasswordHash, PasswordVerifier},
                Argon2,
            };
            let (username,password) = creds;
            let (state,compare) = match (crate::server::ADMIN_PWD.as_ref(),username) {
                (Some(pwd),r) if r == "admin" => (LoginState::Admin,pwd.clone()),
                (_,username) => {
                    let db = &self.0;
                    let user = immt_web_orm::entities::prelude::User::find().filter(
                        immt_web_orm::entities::user::Column::Name.eq(&username)
                    ).one(db).in_current_span().await.map_err(|_| IMMTError::InvalidCredentials)?;
                    let user = user.ok_or(IMMTError::InvalidCredentials)?;
                    (LoginState::User(UserLogin{id:user.id,name:user.name}),user.password)
                }
            };
            let compare = PasswordHash::new(&compare).unwrap();
            Argon2::default()
                .verify_password(password.as_bytes(), &compare).map_err(|_| IMMTError::InvalidCredentials)?;
            Ok(Some(state))
        }

        async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
            Ok(Some(user_id.clone()))
        }
    }
}

#[derive(serde::Serialize,serde::Deserialize,Clone,Debug,PartialEq,Eq)]
pub struct UserLogin {
    pub id:i32,
    pub name:String
}