use gitlab::api::AsyncQuery;
use flams_ontology::uris::ArchiveId;
use oauth2::{url::Url, TokenResponse};
pub use oauth2::AccessToken;
use tracing::{Instrument,instrument};
use super::GitLab;

#[derive(Debug,Clone)]
pub struct GitLabOAuth(
  oauth2::basic::BasicClient,
  GitLab
);

#[derive(Debug,Clone,serde::Deserialize)]
pub struct AuthRequest {
    pub code: String,
    //state: String
}

impl GitLabOAuth {
  pub fn login_url(&self) -> Url {
    let (url,_) = self.0.authorize_url(oauth2::CsrfToken::new_random)
          .add_scope(oauth2::Scope::new("read_user".to_string()))
          .add_scope(oauth2::Scope::new("api".to_string()))
          .url();
    url
  }
  /// #### Errors
  pub async fn callback(&self,request:AuthRequest) -> Result<
    AccessToken,
    impl std::error::Error
  > {
    use oauth2::AuthorizationCode;
    let code = AuthorizationCode::new(request.code);
    self.0.exchange_code(code).request_async(oauth2::reqwest::async_http_client).instrument(crate::REMOTE_SPAN.clone()).await
      .map(|token| token.access_token().clone())
  }

  /// #### Errors
  pub async fn get_projects(&self,token:String) -> Result<Vec<crate::Project>,super::Err> {
    self.get_projects_i(token).instrument(crate::REMOTE_SPAN.clone()).await
  }
  #[instrument(level = "debug",
    target = "git",
    name = "getting all projects for user",
    skip_all
  )]
  async fn get_projects_i(&self,token:String) -> Result<Vec<crate::Project>,super::Err> {
    let mut client = gitlab::ImpersonationClient::new(&self.1.0.inner, token);
    client.oauth2_token();
    let r = gitlab::api::projects::Projects::builder()
      .simple(true).min_access_level(gitlab::api::common::AccessLevel::Developer)
      //.membership(false)
      .build().unwrap_or_else(|_| unreachable!());
    let r:Vec<crate::Project> = gitlab::api::paged(r, gitlab::api::Pagination::All)
      .query_async(&client).await?;
    let mut vs = self.1.0.projects.lock();
    for p in &r {
      if !vs.contains(&p.id) {
        vs.insert(super::ProjectWithId { project: p.clone(), id: None });
      }
    }
    Ok(r)
  }

  /// #### Errors
  pub async fn get_archive_id(&self,id:u64,token:String,branch:&str) -> Result<Option<ArchiveId>,super::Err> {
    self.get_archive_id_i(id,token,branch).instrument(crate::REMOTE_SPAN.clone()).await
  }
  #[instrument(level = "debug",
    target = "git",
    name = "getting archive id",
    skip(self,token)
  )]
  pub async fn get_archive_id_i(&self,id:u64,token:String,branch:&str) -> Result<Option<ArchiveId>,super::Err> {
    {
      let vs = self.1.0.projects.lock();
      if let Some(super::ProjectWithId{id:Some(id),..}) = vs.get(&id) {
        return Ok(id.clone())
      }
    }

    macro_rules! ret {
      ($v:expr) => {{
        tracing::info!("Found {:?}",$v);
        let mut lock= self.1.0.projects.lock();
        if let Some(mut v) = lock.take(&id) {
          v.id = Some($v.clone());
          lock.insert(v);
        }
        return Ok($v);
      }}
    }
    let mut client = gitlab::ImpersonationClient::new(&self.1.0.inner, token);
    client.oauth2_token();
    let r = gitlab::api::projects::repository::TreeBuilder::default()
      .project(id).ref_(branch).recursive(false).build().unwrap_or_else(|_| unreachable!());
    let r:Vec<crate::TreeEntry> = r.query_async(&client).await?;
    let Some(p) = r.into_iter().find_map(|e| if e.path.eq_ignore_ascii_case("meta-inf") && matches!(e.kind, crate::DirOrFile::Dir) { Some(e.path) } else {None}) else {
      ret!(None::<ArchiveId>)
    };

    let r = gitlab::api::projects::repository::TreeBuilder::default()
      .project(id).ref_(branch).path(p).recursive(false).build().unwrap_or_else(|_| unreachable!());
    let r:Vec<crate::TreeEntry> = r.query_async(&client).await?;
    let Some(p) = r.into_iter().find_map(|e| if e.name.eq_ignore_ascii_case("manifest.mf") && matches!(e.kind,crate::DirOrFile::File) { Some(e.path) } else {None}) else {
      ret!(None::<ArchiveId>)
    };

    let blob = gitlab::api::projects::repository::files::FileRaw::builder()
      .project(id).file_path(p).ref_(branch).build().unwrap_or_else(|_| unreachable!());
    let r  = gitlab::api::raw(blob).query_async(&client).await?;
    let r = std::str::from_utf8(&r)?;
    let r = r.split('\n').find_map(|line| {
      let line = line.trim();
      line.strip_prefix("id:").map(|rest| {
        ArchiveId::new(rest.trim())
      })
    });
    ret!(r)
  }

  /// #### Errors
  pub async fn get_branches(&self,id:u64,token:String) -> Result<Vec<crate::Branch>,super::Err> {
    self.get_branches_i(id,token).instrument(crate::REMOTE_SPAN.clone()).await
  }
  #[instrument(level = "debug",
    target = "git",
    name = "getting branches for archive",
    skip(self,token)
  )]
  pub async fn get_branches_i(&self,id:u64,token:String) -> Result<Vec<crate::Branch>,super::Err> {
    let mut client = gitlab::ImpersonationClient::new(&self.1.0.inner, token);
    client.oauth2_token();
    let r = gitlab::api::projects::repository::branches::Branches::builder().project(id)
      .build().unwrap_or_else(|_| unreachable!())
      .query_async(&client).await?;
    Ok(r)
  }
}

impl GitLab {
  /// #### Errors
  pub fn new_oauth(&self,redirect:&str) -> Result<GitLabOAuth,OAuthError> {
    use oauth2::{ClientId,ClientSecret,AuthUrl,TokenUrl,RedirectUrl};
    let url = &self.0.url;
    let Some(app_id) = &self.0.id else {
      return Err(OAuthError::MissingAppID)
    };
    let Some(app_secret) = &self.0.secret else {
      return Err(OAuthError::MissingAppSecret)
    };
    let Ok(auth_url) = AuthUrl::new(format!("{url}/oauth/authorize")) else {
      return Err(OAuthError::InvalidAuthURL(format!("{url}/oauth/authorize")))
    };
    let Ok(token_url) = TokenUrl::new(format!("{url}/oauth/token")) else {
      return Err(OAuthError::InvalidTokenURL(format!("{url}/oauth/token")))
    };
    let Ok(redirect_url) = RedirectUrl::new(redirect.to_string()) else {
      return Err(OAuthError::InvalidRedirectURL(redirect.to_string()))
    };
    Ok(GitLabOAuth(oauth2::basic::BasicClient::new(
      ClientId::new(app_id.to_string()),
      Some(ClientSecret::new(app_secret.to_string())),
      auth_url,
      Some(token_url)
    ).set_redirect_uri(redirect_url),self.clone()))
  }

  /// #### Errors
  pub async fn get_oauth_user(&self,token:&oauth2::AccessToken) -> Result<GitlabUser,reqwest::Error> {
    let client = reqwest::Client::new();
    let resp = client.get(format!("{}/api/v4/user",self.0.url))
        .bearer_auth(token.secret())
        .send()
        .instrument(crate::REMOTE_SPAN.clone()).await?;
    resp.json().instrument(crate::REMOTE_SPAN.clone()).await
  }
}

#[derive(Debug)]
pub enum OAuthError {
  InvalidAuthURL(String),
  InvalidTokenURL(String),
  InvalidRedirectURL(String),
  MissingAppID,
  MissingAppSecret
}
impl std::fmt::Display for OAuthError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::InvalidAuthURL(s) => write!(f,"Invalid auth url: {s}"),
      Self::InvalidTokenURL(s) => write!(f,"Invalid token url: {s}"),
      Self::InvalidRedirectURL(s) => write!(f,"Invalid redirect url: {s}"),
      Self::MissingAppID => write!(f,"Missing app id"),
      Self::MissingAppSecret => write!(f,"Missing secret"),
    }
  }
}

impl std::error::Error for OAuthError {}


#[derive(Debug,serde::Deserialize,serde::Serialize)]
pub struct GitlabUser {
  pub id:i64,
  pub username:String,
  pub name:String,
  //state:String,
  pub avatar_url:String,
  //pronouns:Option<String>,
  pub email:Option<String>,
  //commit_email:Option<String>,
  pub can_create_group:bool,
  pub can_create_project:bool,
}

/*
{
  "id":2,
  "username":"jazzpirate",
  "name":"Dennis Test",
  "state":"active",
  "locked":false,
  "avatar_url":"https://www.gravatar.com/avatar/46a2a1db127c1d43862f7313d137b5ed3bdac8ffe356c7dcc868f25e413e83c0?s=80\\u0026d=identicon",
  "web_url":"http://gitlab.example.com/jazzpirate",
  "created_at":"2024-12-23T10:35:40.099Z",
  "bio":"",
  "location":"",
  "public_email":null,
  "skype":"",
  "linkedin":"",
  "twitter":"",
  "discord":"",
  "website_url":"",
  "organization":"",
  "job_title":"",
  "pronouns":null,
  "bot":false,
  "work_information":null,
  "local_time":null,
  "last_sign_in_at":"2024-12-23T10:36:24.234Z",
  "confirmed_at":"2024-12-23T10:35:39.994Z",
  "last_activity_on":"2024-12-29",
  "email":"d.mueller@kwarc.info",
  "theme_id":3,
  "color_scheme_id":1,
  "projects_limit":100000,
  "current_sign_in_at":
  "2024-12-29T08:48:51.974Z",
  "identities":[],
  "can_create_group":true,
  "can_create_project":true,
  "two_factor_enabled":false,
  "external":false,
  "private_profile":false,
  "commit_email":"d.mueller@kwarc.info"
}
 */