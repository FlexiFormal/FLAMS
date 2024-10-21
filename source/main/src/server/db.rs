use std::fmt::Display;

use argon2::{PasswordHash, PasswordVerifier,PasswordHasher};
use axum_login::{tracing::Instrument, AuthUser, AuthnBackend};
use immt_system::settings::Settings;
use leptos::prelude::ServerFnError;
use password_hash::{rand_core::OsRng, SaltString};
use sqlx::{prelude::FromRow, SqlitePool};

use crate::users::{LoginError, User};

#[derive(Clone, Debug)]
pub struct DBBackend {
    pub pool: SqlitePool,
    pub admin:Option<(String,String)>
}

impl DBBackend {
    /// ### Panics
    pub async fn new() -> Self {
        let settings = Settings::get();
        let db_path = &settings.database;
        let admin = settings.admin_pwd.as_ref().map(|pwd| {
            let argon2 = argon2::Argon2::default();
            let salt = SaltString::generate(&mut OsRng);
            let pass_hash = argon2.hash_password(pwd.as_bytes(), &salt)
                .expect("Failed to hash password");
            let pass_hash_str = pass_hash.to_string();
            let salt_str = salt.as_str().to_string();
            (pass_hash_str,salt_str)
        });
        let db_path = db_path
            .as_os_str()
            .to_str()
            .expect("Failed to connect to database");
        let pool = SqlitePool::connect(db_path)
            .in_current_span()
            .await
            .expect("Failed to connect to database");
        sqlx::migrate!("../../resources/migrations")
            .run(&pool)
            .in_current_span()
            .await
            .expect("Failed to run migrations");
        Self { pool,admin }
    }

    /// #### Errors
    pub async fn add_user(&self,username:String,password:String) -> Result<Option<User>,UserError> {
        #[derive(Debug)]
        struct InsertUser { pub id:i64 }
        
        if username.len() < 2 { return Err(UserError::InvalidUserName)}
        if password.len() < 2 { return Err(UserError::InvalidPassword)}
        let argon2 = argon2::Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let pass_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let pass_hash_str = pass_hash.to_string();
        let salt = salt.as_str();
        let new_id:InsertUser = sqlx::query_as!(InsertUser,
            "INSERT INTO users (username,pass_hash,salt) VALUES ($1,$2,$3) RETURNING id",
            username,pass_hash_str,salt
        ).fetch_one(&self.pool).await?;
        let hash_bytes = pass_hash.hash.unwrap_or_else(|| unreachable!()).as_bytes().to_owned();
        Ok(Some(User {
            id: new_id.id,
            username,
            session_auth_hash: hash_bytes
        }))
    }
}

#[async_trait::async_trait]
impl AuthnBackend for DBBackend {
    type User = User;
    type Credentials = (String, String);
    type Error = UserError;

    async fn authenticate(
        &self,
        (username, password): Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let Some(user) =
            sqlx::query_as!(SqlUser, "SELECT * FROM users WHERE username=$1", username)
                .fetch_optional(&self.pool)
                .in_current_span()
                .await?
        else {
            return Ok(None);
        };
        let hash = PasswordHash::parse(&user.pass_hash, password_hash::Encoding::B64)?;
        let hasher = argon2::Argon2::default();
        if hasher.verify_password(password.as_bytes(), &hash).is_ok() {
            Ok(Some(user.into_user()?))
        } else {
            Ok(None)
        }
    }

    async fn get_user(&self, user_id: &i64) -> Result<Option<Self::User>, Self::Error> {
        if *user_id == 0 { return Ok(Some(User {id:0,username:"admin".to_string(),session_auth_hash:self.admin.as_ref().unwrap_or_else(|| unreachable!()).1.as_bytes().to_owned()})) }
        let Some(res) = sqlx::query_as!(SqlUser, "SELECT * FROM users WHERE id=$1", *user_id)
            .fetch_optional(&self.pool)
            .in_current_span()
            .await?
        else {
            return Ok(None);
        };
        Ok(Some(res.into_user()?))
    }
}

impl AuthUser for User {
    type Id = i64;

    #[inline]
    fn id(&self) -> Self::Id {
        self.id
    }

    #[inline]
    fn session_auth_hash(&self) -> &[u8] {
        &self.session_auth_hash
    }
}

#[derive(Clone, PartialEq, Eq, Debug, FromRow)]
struct SqlUser {
    id: i64,
    username: String,
    pass_hash: String,
    salt: String,
}
impl SqlUser {
    fn into_user(self) -> Result<User, UserError> {
        let PasswordHash { hash, .. } =
            PasswordHash::parse(&self.pass_hash, password_hash::Encoding::B64)?;
        let Some(hash) = hash.map(|h| h.as_bytes().to_owned()) else {
            return Err(UserError::PasswordHashNone);
        };
        Ok(User {
            id: self.id,
            username: self.username,
            session_auth_hash: hash,
        })
    }
}

#[derive(Debug)]
pub enum UserError {
    PasswordHashNone,
    PasswordHash(password_hash::errors::Error),
    Sqlx(sqlx::Error),
    InvalidUserName,
    InvalidPassword
}
impl std::error::Error for UserError {}
impl Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PasswordHashNone => f.write_str("Invalid password hash"),
            Self::PasswordHash(e) => e.fmt(f),
            Self::Sqlx(e) => e.fmt(f),
            Self::InvalidUserName => f.write_str("Invalid username: needs to be at least two characters"),
            Self::InvalidPassword => f.write_str("Invalid password: needs to be at least two characters"),
        }
    }
}
impl From<password_hash::errors::Error> for UserError {
    #[inline]
    fn from(e: password_hash::errors::Error) -> Self {
        Self::PasswordHash(e)
    }
}

impl From<sqlx::Error> for UserError {
    #[inline]
    fn from(e: sqlx::Error) -> Self {
        Self::Sqlx(e)
    }
}

impl From<UserError> for ServerFnError<LoginError> {
    #[inline]
    fn from(_: UserError) -> Self {
        Self::WrappedServerError(
            LoginError::WrongUsernameOrPassword
        )
    }
}
impl From<password_hash::Error> for LoginError {
    #[inline]
    fn from(_: password_hash::Error) -> Self {
        Self::WrongUsernameOrPassword
    }
}
impl From<axum_login::Error<DBBackend>> for LoginError {
    fn from(_ : axum_login::Error<DBBackend>) -> Self {
        Self::InternalError
    }
}