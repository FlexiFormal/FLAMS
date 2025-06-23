use argon2::PasswordHasher;
use axum_login::{AuthUser, AuthnBackend, tower_sessions, tracing::Instrument};
use flams_git::gl::auth::GitlabUser;
use flams_system::settings::Settings;
use flams_utils::unwrap;
use password_hash::{SaltString, rand_core::OsRng};
use sqlx::{SqlitePool, prelude::FromRow};

#[derive(Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct DBUser {
    pub id: i64,
    pub username: String,
    pub session_auth_hash: Vec<u8>,
    pub avatar_url: Option<String>,
    pub is_admin: bool,
    pub secret: String,
}

impl DBUser {
    fn admin(hash: Vec<u8>) -> Self {
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

#[derive(Clone, Debug)]
pub struct DBBackend {
    pub pool: SqlitePool,
    pub admin: Option<(String, String)>,
}

impl DBBackend {
    /// ### Panics
    pub async fn new() -> Self {
        let settings = Settings::get();
        let db_path = &settings.database;
        let admin = settings.admin_pwd.as_ref().map(|pwd| {
            let argon = argon2::Argon2::default();
            let salt = SaltString::generate(&mut OsRng);
            let pass_hash = argon
                .hash_password(pwd.as_bytes(), &salt)
                .expect("Failed to hash password");
            let pass_hash_str = pass_hash.to_string();
            let salt_str = salt.as_str().to_string();
            (pass_hash_str, salt_str)
        });
        if !db_path.exists() {
            tokio::fs::create_dir_all(db_path.parent().expect("Invalid database path"))
                .await
                .expect("Failed to create database directory");
            tokio::fs::File::create(db_path)
                .await
                .expect("Failed to create database file");
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
        Self { pool, admin }
    }

    /// #### Errors
    pub async fn all_users(&self) -> Result<Vec<SqlUser>, UserError> {
        sqlx::query_as!(SqlUser, "SELECT * FROM users")
            .fetch_all(&self.pool)
            //.in_current_span()
            .await
            .map_err(Into::into)
    }

    /// #### Errors
    pub async fn set_admin(&self, id: i64, is_admin: bool) -> Result<(), UserError> {
        sqlx::query!("UPDATE users SET is_admin=$2 WHERE id=$1", id, is_admin)
            .execute(&self.pool)
            //.in_current_span()
            .await
            .map_err(Into::into)
            .map(|_| ())
    }

    /// #### Errors
    pub async fn add_user(
        &self,
        user: GitlabUser,
        secret: String,
    ) -> Result<Option<DBUser>, UserError> {
        #[derive(Debug)]
        struct InsertUser {
            pub id: i64,
            is_admin: bool,
        }
        let GitlabUser {
            id: gitlab_id,
            name,
            username,
            avatar_url,
            email,
            can_create_group,
            can_create_project,
        } = user;

        if username.len() < 2 {
            return Err(UserError::InvalidUserName);
        }
        if secret.len() < 2 {
            return Err(UserError::InvalidPassword);
        }
        let argon2 = argon2::Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let pass_hash = argon2.hash_password(secret.as_bytes(), &salt)?;
        let hash_bytes = pass_hash
            .hash
            .unwrap_or_else(|| unreachable!())
            .as_bytes()
            .to_owned();
        //let salt = salt.as_str();
        let new_id:InsertUser = sqlx::query_as!(InsertUser,
            "INSERT INTO users (gitlab_id,name,username,email,avatar_url,can_create_group,can_create_project,secret,secret_hash,is_admin)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            ON CONFLICT (gitlab_id) DO UPDATE
            SET name = excluded.name, username = excluded.username, email = excluded.email, avatar_url = excluded.avatar_url, can_create_group = excluded.can_create_group, can_create_project = excluded.can_create_project, secret = excluded.secret, secret_hash = excluded.secret_hash
            RETURNING id,is_admin",
            gitlab_id,name,username,email,avatar_url,can_create_group,can_create_project,secret,hash_bytes,false//,salt
        ).fetch_one(&self.pool).await?;
        let u = DBUser {
            id: new_id.id,
            username,
            session_auth_hash: hash_bytes,
            secret,
            avatar_url: Some(avatar_url),
            is_admin: new_id.is_admin,
        };
        Ok(Some(u))
    }

    /// #### Errors
    pub async fn login_as_admin(
        pwd: &str,
        mut session: axum_login::AuthSession<Self>,
    ) -> Result<(), UserError> {
        use argon2::PasswordVerifier;
        let (pass_hash, salt) = unwrap!(session.backend.admin.as_ref());
        let hash = password_hash::PasswordHash::parse(pass_hash, password_hash::Encoding::B64)?;
        let hasher = argon2::Argon2::default();
        hasher.verify_password(pwd.as_bytes(), &hash)?;
        session
            .login(&DBUser::admin(salt.as_bytes().to_owned()))
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl AuthnBackend for DBBackend {
    type User = DBUser;
    type Credentials = (i64, String);
    type Error = UserError;

    async fn authenticate(
        &self,
        (gitlab_id, secret): Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let Some(user) =
            sqlx::query_as!(SqlUser, "SELECT * FROM users WHERE gitlab_id=$1", gitlab_id)
                .fetch_optional(&self.pool)
                .await?
        else {
            return Ok(None);
        };
        if user.secret == secret {
            Ok(Some(user.try_into()?))
        } else {
            Ok(None)
        }
    }

    async fn get_user(&self, user_id: &i64) -> Result<Option<Self::User>, Self::Error> {
        if *user_id == 0 {
            return Ok(Some(DBUser::admin(
                self.admin
                    .as_ref()
                    .unwrap_or_else(|| unreachable!())
                    .1
                    .as_bytes()
                    .to_owned(),
            )));
        }
        let Some(res) = sqlx::query_as!(SqlUser, "SELECT * FROM users WHERE id=$1", *user_id)
            .fetch_optional(&self.pool)
            //.in_current_span()
            .await?
        else {
            return Ok(None);
        };
        Ok(Some(res.try_into()?))
    }
}

impl AuthUser for DBUser {
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
pub struct SqlUser {
    id: i64,
    gitlab_id: i64,
    name: String,
    username: String,
    email: String,
    avatar_url: String,
    can_create_group: bool,
    can_create_project: bool,
    secret: String,
    secret_hash: Vec<u8>,
    is_admin: bool, //salt: String,
}

impl TryFrom<SqlUser> for DBUser {
    type Error = UserError;
    fn try_from(value: SqlUser) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            username: value.username,
            session_auth_hash: value.secret_hash,
            secret: value.secret,
            is_admin: value.is_admin,
            avatar_url: Some(value.avatar_url),
        })
    }
}

impl From<SqlUser> for super::UserData {
    fn from(u: SqlUser) -> Self {
        Self {
            id: u.id,
            name: u.name,
            username: u.username,
            email: u.email,
            avatar_url: u.avatar_url,
            is_admin: u.is_admin,
        }
    }
}

#[derive(Debug, strum::Display)]
pub enum UserError {
    #[strum(to_string = "Invalid password hash")]
    PasswordHashNone,
    #[strum(to_string = "{0}")]
    PasswordHash(password_hash::errors::Error),
    #[strum(to_string = "{0}")]
    Sqlx(sqlx::Error),
    #[strum(to_string = "{0}")]
    Session(tower_sessions::session::Error),
    #[strum(to_string = "Invalid username: needs to be at least two characters")]
    InvalidUserName,
    #[strum(to_string = "Invalid password: needs to be at least two characters")]
    InvalidPassword,
}
impl std::error::Error for UserError {}
impl From<password_hash::errors::Error> for UserError {
    #[inline]
    fn from(e: password_hash::errors::Error) -> Self {
        Self::PasswordHash(e)
    }
}

impl From<axum_login::Error<DBBackend>> for UserError {
    #[inline]
    fn from(e: axum_login::Error<DBBackend>) -> Self {
        match e {
            axum_login::Error::Session(e) => Self::Session(e),
            axum_login::Error::Backend(e) => e,
        }
    }
}

impl From<sqlx::Error> for UserError {
    #[inline]
    fn from(e: sqlx::Error) -> Self {
        Self::Sqlx(e)
    }
}

impl From<UserError> for super::LoginError {
    #[inline]
    fn from(_: UserError) -> Self {
        Self::WrongUsernameOrPassword
    }
}
impl From<password_hash::Error> for super::LoginError {
    #[inline]
    fn from(_: password_hash::Error) -> Self {
        Self::WrongUsernameOrPassword
    }
}
impl From<axum_login::Error<DBBackend>> for super::LoginError {
    fn from(_: axum_login::Error<DBBackend>) -> Self {
        Self::InternalError
    }
}
