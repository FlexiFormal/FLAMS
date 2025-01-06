pub mod auth;

#[derive(Debug,Clone)]
pub struct GitLab {
  inner: gitlab::AsyncGitlab,
	url:Box<str>,
	id:Option<Box<str>>,
	secret:Option<Box<str>>
}

#[derive(Debug,Clone,Default)]
enum MaybeGitlab {
  #[default]
  None,
  Loading,
  Loaded(GitLab),
  Failed
}
#[derive(Clone,Debug,Default)]
pub struct GLInstance{
  inner:std::sync::Arc<parking_lot::RwLock<MaybeGitlab>>
}

impl GLInstance {
	pub fn load(self,cfg:GitlabConfig) {
		*self.inner.write() = MaybeGitlab::Loading;
		tokio::spawn(async move {
			match GitLab::new(cfg).await {
				Ok(gl) => {
					*self.inner.write() = MaybeGitlab::Loaded(gl);
				}
				Err(e) => {
					tracing::error!("Failed to load gitlab: {e}");
					*self.inner.write() = MaybeGitlab::Failed;
				}
			}
		});
	}

  pub async fn get(&self) -> Option<GitLab> {
    loop {
      match &*self.inner.read() {
        MaybeGitlab::None | MaybeGitlab::Failed => return None,
        MaybeGitlab::Loading => (),
        MaybeGitlab::Loaded(gl) => return Some(gl.clone())
      }
			tokio::time::sleep(std::time::Duration::from_secs_f32(0.1)).await;
    }
  }

  #[inline]#[must_use]
  pub fn exists(&self) -> bool {
   
	  !matches!(&*self.inner.read(),MaybeGitlab::None)
  }

  #[inline]#[must_use]
  pub fn has_loaded(&self) -> bool {
    matches!(&*self.inner.read(),MaybeGitlab::Loaded(_))
  }
}

#[derive(Debug,Clone)]
pub struct GitlabConfig {
	url:String,
	token:Option<String>,
	app_id:Option<String>,
	app_secret:Option<String>
}
impl GitlabConfig {
	#[inline]#[must_use]
	pub const fn new(url:String,token:Option<String>,app_id:Option<String>,app_secret:Option<String>) -> Self {
		Self { url, token, app_id, app_secret }
	}

  fn split(url:&str) -> (&str,bool) {
    if let Some(r) = url.strip_prefix("https://") {
      return (r,false)
    }
    if let Some(r) = url.strip_prefix("http://") {
      return (r,true)
    }
    (url,false)
  }
}

impl GitLab {
  /// #### Errors
	pub async fn new(cfg:GitlabConfig) -> Result<Self,gitlab::GitlabError> {
		let GitlabConfig { url, token, app_id, app_secret } = cfg;
		let (split_url,http) = GitlabConfig::split(&url);
		let mut builder = token.map_or_else(
			|| gitlab::GitlabBuilder::new_unauthenticated(split_url),
			|token| gitlab::GitlabBuilder::new(split_url,token)
		);
		if http { builder.insecure(); }
		Ok(Self {
			inner: builder.build_async().await?,
			url:url.into(),
			id:app_id.map(Into::into),
			secret:app_secret.map(Into::into)
		})
	}

  #[must_use]
  pub fn new_background(cfg:GitlabConfig) -> GLInstance {
    let r = GLInstance {inner:std::sync::Arc::new(parking_lot::RwLock::new(MaybeGitlab::Loading))};
    r.clone().load(cfg);
    r
  }

  /// #### Errors
  pub async fn get_projects(&self) -> Result<Vec<crate::Project>,Err> {
    use gitlab::api::AsyncQuery;
    let q = gitlab::api::projects::Projects::builder().simple(true).build().unwrap_or_else(|_| unreachable!());
    Ok(q.query_async(&self.inner).await?)
    //let raw = gitlab::api::raw(q).query_async(&self.inner).await?;
    //Ok(std::str::from_utf8(raw.as_ref())?.to_string())
  }
}

#[derive(Debug)]
pub enum Err {
  Api(gitlab::api::ApiError<gitlab::RestError>),
  Str(std::str::Utf8Error),
	Gitlab(gitlab::GitlabError)
}
impl From<gitlab::api::ApiError<gitlab::RestError>> for Err {
  #[inline]
  fn from(e: gitlab::api::ApiError<gitlab::RestError>) -> Self {
    Self::Api(e)
  }
}
impl From<std::str::Utf8Error> for Err {
  #[inline]
  fn from(e: std::str::Utf8Error) -> Self {
    Self::Str(e)
  }
}
impl From<gitlab::GitlabError> for Err {
	#[inline]
	fn from(e: gitlab::GitlabError) -> Self {
		Self::Gitlab(e)
	}
}
impl std::fmt::Display for Err {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Api(e) => e.fmt(f),
			Self::Str(e) => e.fmt(f),
			Self::Gitlab(e) => e.fmt(f)
		}
	}
}