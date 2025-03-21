#[derive(Clone, serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub enum LoginState {
    Loading,
    Admin,
    User {
        name: String,
        avatar: String,
        is_admin: bool,
    },
    None,
    NoAccounts,
}
impl LoginState {
    #[cfg(feature = "ssr")]
    #[must_use]
    pub fn get_server() -> Self {
        let Some(session) =
            leptos::prelude::use_context::<axum_login::AuthSession<crate::db::DBBackend>>()
        else {
            return Self::None;
        };
        match &session.backend.admin {
            None => Self::NoAccounts,
            Some(_) => match session.user {
                None => Self::None,
                Some(super::users::ServerUser {
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

    #[inline]
    pub fn get() -> Self {
        use leptos::prelude::*;
        let ctx: RwSignal<Self> = expect_context();
        ctx.get()
    }
}

impl std::fmt::Display for LoginState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Loading => write!(f, "Loading"),
            Self::Admin => write!(f, "Admin"),
            Self::User { name, .. } => write!(f, "User: {name}"),
            Self::None => write!(f, "None"),
            Self::NoAccounts => write!(f, "No accounts"),
        }
    }
}
