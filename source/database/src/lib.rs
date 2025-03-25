#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(feature = "backend")]
mod db;
#[cfg(feature = "backend")]
pub use db::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserData {
    pub id: i64,
    pub name: String,
    pub username: String,
    pub email: String,
    pub avatar_url: String,
    pub is_admin: bool,
}

#[derive(
    Debug,
    Copy,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    strum::Display,
    strum::EnumString,
)]
pub enum LoginError {
    #[strum(to_string = "Wrong username or password")]
    WrongUsernameOrPassword,
    #[strum(to_string = "Internal error")]
    InternalError,
    #[strum(to_string = "Not logged in")]
    NotLoggedIn,
}
