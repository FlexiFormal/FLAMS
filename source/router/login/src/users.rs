#[cfg(feature = "ssr")]
#[derive(Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct ServerUser {
    pub id: i64,
    pub username: String,
    pub session_auth_hash: Vec<u8>,
    pub avatar_url: Option<String>,
    pub is_admin: bool,
    pub secret: String,
}

#[cfg(feature = "ssr")]
impl ServerUser {
    pub(crate) fn admin(hash: Vec<u8>) -> Self {
        Self {
            id: 0,
            username: "admin".to_string(),
            session_auth_hash: hash,
            secret: String::new(),
            is_admin: true,
            avatar_url: None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserData {
    pub id: i64,
    pub name: String,
    pub username: String,
    pub email: String,
    pub avatar_url: String,
    pub is_admin: bool,
}
