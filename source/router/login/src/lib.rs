#[cfg(any(
    all(feature = "ssr", feature = "hydrate", not(doc)),
    not(any(feature = "ssr", feature = "hydrate"))
))]
compile_error!("exactly one of the features \"ssr\" or \"hydrate\" must be enabled");

#[cfg(feature = "ssr")]
pub mod db;
mod login_state;
pub use login_state::*;
pub mod components;
pub mod server_fns;
pub mod users;

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum LoginError {
    WrongUsernameOrPassword,
    InternalError,
    NotLoggedIn,
}
impl std::fmt::Display for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WrongUsernameOrPassword => write!(f, "Wrong username or password"),
            Self::InternalError => write!(f, "Internal error"),
            Self::NotLoggedIn => write!(f, "Not logged in"),
        }
    }
}
impl std::str::FromStr for LoginError {
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
