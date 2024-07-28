use std::fmt::Display;
use std::str::FromStr;
use leptos::ServerFnError;

#[derive(Debug,serde::Serialize,serde::Deserialize,Clone,PartialEq,Eq)]
pub enum IMMTError {
    InvalidCredentials,
    ImplementationError,
    AccessForbidden,
    InvalidArgument(String)
}
impl Display for IMMTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use IMMTError::*;
        match self {
            IMMTError::InvalidCredentials => write!(f,"User does not exist or password is incorrect"),
            IMMTError::ImplementationError => write!(f,"An error occurred in the server implementation"),
            IMMTError::AccessForbidden => write!(f,"Access to the requested resource is forbidden"),
            IMMTError::InvalidArgument(s) => write!(f,"Invalid argument: {}",s)
        }
    }
}

impl FromStr for IMMTError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use IMMTError::*;
        match s {
            "InvalidCredentials" => Ok(InvalidCredentials),
            "ImplementationError" => Ok(ImplementationError),
            "AccessForbidden" => Ok(AccessForbidden),
            "InvalidArgument" => Ok(InvalidArgument(String::new())),
            _ => Err(())
        }
    }
}
impl std::error::Error for IMMTError {}
pub type ServerError = ServerFnError<IMMTError>;
pub type ServerResult<A> = Result<A,ServerError>;