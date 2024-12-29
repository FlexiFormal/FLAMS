use http::StatusCode;
use immt_system::settings::Settings;
use leptos::config::ConfFile;
use oauth2::TokenResponse;
use axum::{extract, response::{IntoResponse, Redirect, Response}};
use super::ServerState;

pub(super) async fn gl_login(extract::State(state):extract::State<ServerState>) -> Redirect {
  if let Some(oauth) = state.oauth.as_ref() {
      let (url,_) = oauth.client.authorize_url(oauth2::CsrfToken::new_random)
          .add_scope(oauth2::Scope::new("read_user".to_string()))
          .url();
      Redirect::to(url.as_str())
  } else {
      Redirect::to("/dashboard")
  }
}

#[derive(Debug,serde::Deserialize)]
pub(super) struct AuthRequest {
    code: String,
    state: String
}

pub(super) async fn gl_cont(
    extract::Query(params): extract::Query<AuthRequest>,
    extract::State(state):extract::State<ServerState>,
) -> Result<Response,StatusCode> {
    use oauth2::AuthorizationCode;
    let code = AuthorizationCode::new(params.code);
    let token = state.oauth.as_ref().unwrap_or_else(|| unreachable!())
        .client.exchange_code(code).request_async(oauth2::reqwest::async_http_client).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let url = Settings::get().gitlab_url.as_ref().unwrap_or_else(|| unreachable!());
    let client = reqwest::Client::new();
    let resp = client.get(format!("{url}/api/v4/user"))
        .bearer_auth(token.access_token().secret())
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user = resp.json::<GitlabUser>().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    println!("Result: {:?}",user);
    Ok(Redirect::to("/dashboard").into_response())
}

#[derive(Debug,serde::Deserialize,serde::Serialize)]
pub struct GitlabUser {
  username:String,
  name:String,
  state:String,
  avatar_url:String,
  pronouns:Option<String>,
  email:Option<String>,
  commit_email:Option<String>,
  can_create_group:bool,
  can_create_project:bool,
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

#[derive(Debug,Clone)]
pub struct OAuthConfig {
    client:oauth2::basic::BasicClient
}
impl OAuthConfig {
    #[inline]
    pub(super) fn new(leptos:&ConfFile) -> Option<Self> {
        use oauth2::{ClientId,ClientSecret,AuthUrl,TokenUrl,RedirectUrl};
        let _span = tracing::info_span!("GitLab");
        let _e = _span.enter();
        let settings = Settings::get();
        let url = settings.gitlab_url.as_ref()?;
        let app_id = settings.gitlab_app_id.as_ref()?;
        let app_secret = settings.gitlab_app_secret.as_ref()?;
        let redirect_url = settings.gitlab_redirect_url.as_ref()?;
        let Ok(auth_url) = AuthUrl::new(format!("{url}/oauth/authorize")) else {
            tracing::error!("Invalid auth URL: {url}/oauth/authorize");
            return None
        };
        let Ok(token_url) = TokenUrl::new(format!("{url}/oauth/token")) else {
            tracing::error!("Invalid Token URL: {url}/oauth/token");
            return None
        };
        let Ok(redirect_url) = RedirectUrl::new(format!("{redirect_url}/gitlab_login")) else {
            tracing::error!("Invalid Redirect URL: {redirect_url}/gitlab_login");
            return None
        };
        let client = oauth2::basic::BasicClient::new(
            ClientId::new(app_id.to_string()),
            Some(ClientSecret::new(app_secret.to_string())),
            auth_url,
            Some(token_url)
        ).set_redirect_uri(redirect_url);
        tracing::info!("OAuth config initialized");
        Some(Self {client})
    }
}