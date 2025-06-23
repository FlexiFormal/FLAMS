#[derive(Clone, serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, strum::Display)]
pub enum LoginState {
    #[strum(to_string = "Loading")]
    Loading,
    #[strum(to_string = "Admin")]
    Admin,
    #[strum(to_string = "User {name}")]
    User {
        name: String,
        avatar: String,
        is_admin: bool,
    },
    #[strum(to_string = "None")]
    None,
    #[strum(to_string = "No accounts")]
    NoAccounts,
}
impl LoginState {
    #[inline]
    #[must_use]
    pub fn get() -> Self {
        use leptos::prelude::*;
        let ctx: RwSignal<Self> = expect_context();
        ctx.get()
    }
}

#[cfg(feature = "ssr")]
mod server {
    use flams_database::{DBBackend, DBUser};

    use super::LoginState;

    impl LoginState {
        #[must_use]
        pub fn get_server() -> Self {
            let Some(session) =
                leptos::prelude::use_context::<axum_login::AuthSession<DBBackend>>()
            else {
                return Self::None;
            };
            match &session.backend.admin {
                None => Self::NoAccounts,
                Some(_) => match session.user {
                    None => Self::None,
                    Some(DBUser {
                        id: 0, username, ..
                    }) if username == "admin" => Self::Admin,
                    Some(u) => Self::User {
                        name: u.username,
                        avatar: u.avatar_url.unwrap_or_default(),
                        is_admin: u.is_admin,
                    },
                },
            }
        }
    }
}
