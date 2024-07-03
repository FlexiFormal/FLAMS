use std::fmt::Display;
use leptos::*;


#[inline(always)]
pub fn if_logged_in<R>(yes:impl FnOnce() -> R,no: impl FnOnce() -> R) -> R {
    use crate::accounts::LoginState;
    use leptos::use_context;
    let signal = use_context::<LoginState>();
    match signal {
        Some(LoginState::Admin | LoginState::User(_) | LoginState::NoAccounts) => yes(),
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

#[component(transparent)]
pub(crate) fn WithAccount(children:ChildrenFn) -> impl IntoView {
    use thaw::*;
    use leptos::leptos_dom::tracing::Instrument;
    let user_update = create_rw_signal(true);
    let (user,writer) = create_signal(LoginState::Loading);
    let resource = create_resource(move || user_update.get(), move |_| async move {
        login_status().in_current_span().await.unwrap_or_else(|_| LoginState::None)
    });
    provide_context(user);
    view!{
        <Suspense fallback=|| view!(<Spinner/>)>{
            if let Some(login) = resource.get() {
                //console_log!("User login state: {:?}",login);
                writer.set(login.clone());
            }
            //move || view!(<WithAccountClient user = login>{children()}</WithAccountClient>)
            children()
        }</Suspense>
    }
}

#[island]
pub fn WithAccountClient(children:Children,user:LoginState) -> impl IntoView {
    view!(<Provider value=user>{children()}</Provider>)
}

#[cfg(feature="server")]
pub(crate) async fn login_status_with_session(
    session:Option<&axum_login::AuthSession<AccountManager>>,
    db:impl FnOnce() -> Option<sea_orm::DatabaseConnection>
) -> Option<LoginState> {
    use axum_login::AuthUser;
    use crate::server::ADMIN_PWD;
    use sea_orm::prelude::*;
    use leptos::leptos_dom::tracing::Instrument;
    let identity = session.map(|s| s.user.as_ref().map(|u| u.id())).flatten();
    if let Some(user) = identity { return Some(user) }
    let db = db()?;
    let users = immt_web_orm::entities::prelude::User::find().all(&db).in_current_span().await;
    if ADMIN_PWD.is_none() && match users {
        Ok(users) => users.is_empty(),
        _ => true
    } {
        Some(LoginState::Admin)
    }
    else { Some(LoginState::None) }
}

#[server(prefix="/api/users",endpoint="login_status")]
pub async fn login_status() -> Result<LoginState,ServerFnError<IMMTError>> {
    use leptos::leptos_dom::tracing::Instrument;
    login_status_with_session(
        use_context::<axum_login::AuthSession<AccountManager>>().as_ref(),
        || use_context::<sea_orm::DatabaseConnection>()
    ).in_current_span().await.ok_or(IMMTError::ImplementationError.into())
}

#[server(Login,prefix="/api/users",endpoint="login")]//, input=server_fn::codec::Cbor)]
pub async fn login(username:String,password:String) -> Result<(),ServerFnError<IMMTError>> {
    use leptos_axum::redirect;
    use leptos::leptos_dom::tracing::Instrument;
    redirect("/dashboard");
    let mut session = use_context::<axum_login::AuthSession<AccountManager>>().unwrap();
    match session.authenticate((username,password)).in_current_span().await.map_err(|_| IMMTError::ImplementationError)? {
        Some(u) => {
            session.login(&u).in_current_span().await.map_err(|_| IMMTError::InvalidCredentials)?;
            Ok(())
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
    use leptos::use_context;
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
#[cfg(feature="server")]
pub(crate) use server::*;
use crate::console_log;
use crate::utils::errors::{IMMTError, ServerError, ServerResult};

#[derive(serde::Serialize,serde::Deserialize,Clone,Debug,PartialEq,Eq)]
pub struct UserLogin {
    pub id:i32,
    pub name:String
}