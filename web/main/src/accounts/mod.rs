use std::time::Duration;
use actix_session::SessionExt;
use leptos::*;
use leptos_actix::ResponseOptions;

#[component(transparent)]
pub(crate) fn WithAccount(children:ChildrenFn) -> impl IntoView {
    use thaw::*;
    let user_update = create_rw_signal(true);
    let (user,writer) = create_signal(LoginState::Loading);
    let resource = create_resource(move || user_update.get(), move |_| async move {
        login_status().await.unwrap_or_else(|_| LoginState::None)
    });
    provide_context(user);
    view!{
        <Suspense fallback=|| view!(<Spinner/>)>{
            if let Some(login) = resource.get() {
                console_log!("User login state: {:?}",login);
                writer.set(login);
            }
            children()
        }</Suspense>
    }
}

#[cfg(feature="accounts")]
mod accounts {
    pub trait ToID {
        fn into_login_state(self) -> Option<crate::accounts::LoginState>;
    }
    impl ToID for actix_identity::Identity {
        fn into_login_state(self) -> Option<crate::accounts::LoginState> {
            leptos::serde_json::from_str(&self.id().ok()?).ok()
        }
    }
}
#[cfg(feature="accounts")]
pub use accounts::*;
use crate::console_log;

#[cfg(feature="accounts")]
#[server(prefix="/api/users",endpoint="login_status")]
pub async fn login_status() -> Result<LoginState,ServerFnError> {
    use crate::server::ADMIN_PWD;
    use sea_orm::prelude::*;
    use actix_session::SessionExt;

    let session = leptos_actix::extract::<actix_session::Session>().await?;
    let identity = session.get::<LoginState>("user").ok().flatten();

    //let identity = leptos_actix::extract::<actix_identity::Identity>().await.ok();
    println!("Identity: {:?}",identity.as_ref());
    //println!("Identity: {:?}",identity.as_ref().and_then(|i| i.id().ok()));
    if let Some(val) = identity {
        Ok(val)
    }
    /*if let Some(id) = identity {
        match id.into_login_state() {
            Some(val) => Ok(val),
            _ => Err(UserValidation::NoUser.into())
        }
    }*/ else {
        let db = leptos_actix::extract::<actix_web::web::Data<sea_orm::DatabaseConnection>>().await?;
        let users = immt_web_orm::entities::prelude::User::find().all(&**db).await;
        if ADMIN_PWD.is_none() && match users {
            Ok(users) => users.is_empty(),
            _ => true
        } {
            Ok(LoginState::Admin)
        }
        else { Ok(LoginState::None) }
    }
}
#[cfg(not(feature="accounts"))]
#[server(prefix="/api/users",endpoint="login_status")]
pub async fn login_status() -> Result<LoginState,ServerFnError> {
    Ok(LoginState::Admin)
}

#[server(Login,prefix="/api/users",endpoint="login")]
pub async fn login(username:String,password:String) -> Result<(),ServerFnError> {
    #[cfg(feature="accounts")]
    {
        use sea_orm::{EntityTrait,QueryFilter,ColumnTrait};

        use argon2::{
            password_hash::{PasswordHash, PasswordVerifier},
            Argon2,
        };
        use actix_web::HttpMessage;
        println!("Logging in: {username}@{password}");
        let (state,compare) = match (crate::server::ADMIN_PWD.as_ref(),username) {
            (Some(pwd),r) if r == "admin" => (LoginState::Admin,pwd.clone()),
            (_,username) => {
                let db = leptos_actix::extract::<actix_web::web::Data<sea_orm::DatabaseConnection>>().await?;
                let user = immt_web_orm::entities::prelude::User::find().filter(
                    <immt_web_orm::entities::prelude::User as EntityTrait>::Column::Name.eq(&username)
                ).one(&**db).await?;
                let user = user.ok_or(UserValidation::NoUser)?;
                (LoginState::User(UserLogin{id:user.id,name:user.name}),user.password)
            }
        };
        println!("Returning {state:?}");

        let compare = PasswordHash::new(&compare).unwrap();
        Argon2::default()
            .verify_password(password.as_bytes(), &compare).map_err(|_| UserValidation::NoUser)?;

        let session = leptos_actix::extract::<actix_session::Session>().await?;
        session.insert("user",state).unwrap();
        //actix_identity::Identity::login(&request.extensions(),serde_json::to_string(&state)?)?;
    }
    Ok(())
}

#[derive(Clone,serde::Serialize,serde::Deserialize,Debug,PartialEq,Eq)]
pub enum LoginState {
    Loading,Admin,User(UserLogin),None,NoAccounts
}
#[derive(serde::Serialize,serde::Deserialize,Clone,Debug,PartialEq,Eq)]
pub struct UserLogin {
    pub id:i32,
    pub name:String
}

#[derive(Debug,serde::Serialize,serde::Deserialize)]
enum UserValidation {
    NoUser,
    SerializationError
}
impl std::fmt::Display for UserValidation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"User validation error: {:?}",self)
    }
}
impl std::error::Error for UserValidation {

}
impl From<serde_json::Error> for UserValidation {
    fn from(value: serde_json::Error) -> Self { UserValidation::SerializationError }
}
