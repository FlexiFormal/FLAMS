use std::fmt::Display;

use argon2::{PasswordHash, PasswordVerifier,PasswordHasher};
use axum_login::{tracing::Instrument, AuthUser, AuthnBackend};
use immt_git::gl::auth::GitlabUser;
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
        if !db_path.exists() {
            tokio::fs::create_dir_all(db_path.parent().expect("Invalid database path"))
                .await.expect("Failed to create database directory");
            tokio::fs::File::create(db_path).await.expect("Failed to create database file");
        }
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

    pub async fn all_users(&self) -> Result<Vec<SqlUser>,UserError> {
        sqlx::query_as!(SqlUser, "SELECT * FROM users")
            .fetch_all(&self.pool)
            //.in_current_span()
            .await.map_err(Into::into)
    }

    pub async fn set_admin(&self,id:i64,is_admin:bool) -> Result<(),UserError> {
        sqlx::query!("UPDATE users SET is_admin=$2 WHERE id=$1",id,is_admin)
            .execute(&self.pool)
            //.in_current_span()
            .await.map_err(Into::into).map(|_| ())
    }

    /// #### Errors
    pub async fn add_user(&self,user:GitlabUser,secret:String) -> Result<Option<User>,UserError> {
        #[derive(Debug)]
        struct InsertUser { pub id:i64, is_admin:bool }
        let GitlabUser {id:gitlab_id,name,username,avatar_url,email,can_create_group,can_create_project} = user;
        
        if username.len() < 2 { return Err(UserError::InvalidUserName)}
        if secret.len() < 2 { return Err(UserError::InvalidPassword)}
        let argon2 = argon2::Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let pass_hash = argon2.hash_password(secret.as_bytes(), &salt)?;
        let hash_bytes = pass_hash.hash.unwrap_or_else(|| unreachable!()).as_bytes().to_owned();
        //let salt = salt.as_str();
        let new_id:InsertUser = sqlx::query_as!(InsertUser,
            "INSERT INTO users (gitlab_id,name,username,email,avatar_url,can_create_group,can_create_project,secret,secret_hash,is_admin) 
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) 
            ON CONFLICT (gitlab_id) DO UPDATE 
            SET name = excluded.name, username = excluded.username, email = excluded.email, avatar_url = excluded.avatar_url, can_create_group = excluded.can_create_group, can_create_project = excluded.can_create_project, secret = excluded.secret, secret_hash = excluded.secret_hash
            RETURNING id,is_admin",
            gitlab_id,name,username,email,avatar_url,can_create_group,can_create_project,secret,hash_bytes,false//,salt
        ).fetch_one(&self.pool).await?;
        let u = User {
            id: new_id.id,
            username,
            session_auth_hash: hash_bytes,
            secret,
            avatar_url:Some(avatar_url),
            is_admin:new_id.is_admin
        };
        Ok(Some(u))
    }
}

#[async_trait::async_trait]
impl AuthnBackend for DBBackend {
    type User = User;
    type Credentials = (i64, String);
    type Error = UserError;

    async fn authenticate(
        &self,
        (gitlab_id, secret): Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let Some(user) =
            sqlx::query_as!(SqlUser, "SELECT * FROM users WHERE gitlab_id=$1", gitlab_id)
                .fetch_optional(&self.pool)
                //.in_current_span()
                .await?
        else {
            return Ok(None);
        };
        //let hash = PasswordHash::parse(&user.secret_hash, password_hash::Encoding::B64)?;
        //let hasher = argon2::Argon2::default();
        if user.secret == secret {//hasher.verify_password(secret.as_bytes(), &hash).is_ok() {
            Ok(Some(user.into_user()?))
        } else {
            Ok(None)
        }
    }

    async fn get_user(&self, user_id: &i64) -> Result<Option<Self::User>, Self::Error> {
        if *user_id == 0 { return Ok(Some(User::admin(self.admin.as_ref().unwrap_or_else(|| unreachable!()).1.as_bytes().to_owned()))) }
        let Some(res) = sqlx::query_as!(SqlUser, "SELECT * FROM users WHERE id=$1", *user_id)
            .fetch_optional(&self.pool)
            //.in_current_span()
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
pub(crate) struct SqlUser {
    pub(crate) id: i64,
    gitlab_id:i64,
    pub(crate) name:String,
    pub(crate) username: String,
    pub(crate) email: String,
    pub(crate) avatar_url: String,
    can_create_group: bool,
    can_create_project: bool,
    secret:String,
    secret_hash: Vec<u8>,
    pub(crate) is_admin:bool
    //salt: String,
}
impl SqlUser {
    fn into_user(self) -> Result<User, UserError> {
        Ok(User {
            id: self.id,
            username: self.username,
            session_auth_hash: self.secret_hash,
            secret: self.secret,
            is_admin:self.is_admin,
            avatar_url:Some(self.avatar_url)
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
