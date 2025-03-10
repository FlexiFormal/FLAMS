use flams_ontology::uris::ArchiveId;
use flams_utils::prelude::HSet;
use gitlab::api::AsyncQuery;
use tracing::{Instrument,instrument};

pub mod auth;

lazy_static::lazy_static!{
  static ref GITLAB: GLInstance = GLInstance::default();
}

#[derive(Debug)]
struct ProjectWithId {
  pub project:super::Project,
  #[allow(clippy::option_option)]
  pub id:Option<Option<ArchiveId>>
}
impl std::borrow::Borrow<u64> for ProjectWithId {
  #[inline]
  fn borrow(&self) -> &u64 {
    &self.project.id
  }
}
impl std::hash::Hash for ProjectWithId {
  #[inline]
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.project.id.hash(state);
  }
}
impl PartialEq for ProjectWithId {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.project.id == other.project.id
  }
}
impl Eq for ProjectWithId {}

#[derive(Debug)]
struct GitLabI {
  inner: gitlab::AsyncGitlab,
	url:Box<str>,
	id:Option<Box<str>>,
	secret:Option<Box<str>>,
  projects:parking_lot::Mutex<HSet<ProjectWithId>>
}

#[derive(Debug,Clone)]
pub struct GitLab(std::sync::Arc<GitLabI>);

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
  #[inline]#[must_use]
  pub fn global() -> &'static Self {
    &GITLAB
  }
	pub fn load(self,cfg:GitlabConfig) {
		*self.inner.write() = MaybeGitlab::Loading;
    let span = tracing::info_span!(target:"git","loading gitlab");
		tokio::spawn(async move {
			match GitLab::new(cfg).in_current_span().await {
				Ok(gl) => {
					*self.inner.write() = MaybeGitlab::Loaded(gl.clone());
          let Ok(ps) = gl.get_projects().in_current_span().await else {
            tracing::error!("Failed to load projects");
            return 
          };
          tracing::info!("Loaded {} projects",ps.len());
          let span2 = tracing::info_span!("loading archive IDs");
          let mut js = tokio::task::JoinSet::new();
          for p in ps {
            if let Some(d) = p.default_branch {
              let gl = gl.clone();
              let span = span2.clone();
              let f = async move {gl.get_archive_id(p.id, &d).instrument(span).await};
              let _ = js.spawn(f);
            }
          }
          let _ = js.join_all().in_current_span().await;
				}
				Err(e) => {
					tracing::error!("Failed to load gitlab: {e}");
					*self.inner.write() = MaybeGitlab::Failed;
				}
			}
		}.instrument(span));
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
		Ok(Self(std::sync::Arc::new(GitLabI {
			inner: builder.build_async().in_current_span().await?,
			url:url.into(),
			id:app_id.map(Into::into),
			secret:app_secret.map(Into::into),
      projects:parking_lot::Mutex::new(HSet::default())
		})))
	}

  #[must_use]
  pub fn new_background(cfg:GitlabConfig) -> GLInstance {
    let r = GLInstance {inner:std::sync::Arc::new(parking_lot::RwLock::new(MaybeGitlab::Loading))};
    r.clone().load(cfg);
    r
  }

  /// #### Errors
#[instrument(level = "debug",
  target = "git",
  name = "getting all gitlab projects",
  skip_all
)]
  pub async fn get_projects(&self) -> Result<Vec<crate::Project>,Err> {
    use gitlab::api::AsyncQuery;
    let q = gitlab::api::projects::Projects::builder().simple(true).build().unwrap_or_else(|_| unreachable!());
    let v: Vec<super::Project> = gitlab::api::paged(q,gitlab::api::Pagination::All).query_async(&self.0.inner).await
      .map_err(|e| {tracing::error!("Failed to load projects: {e}"); e})?;
    let mut prs = self.0.projects.lock();
    for p in &v {
      if !prs.contains(&p.id) {
        prs.insert(ProjectWithId { project: p.clone(), id: None });
      }
    }
    drop(prs);
    Ok(v)
    //let raw = gitlab::api::raw(q).query_async(&self.inner).await?;
    //Ok(std::str::from_utf8(raw.as_ref())?.to_string())
  }

  /// #### Errors
#[instrument(level = "debug",
  target = "git",
  name = "getting archive id",
  skip(self),
)]
  pub async fn get_archive_id(&self,id:u64,branch:&str) -> Result<Option<ArchiveId>,Err> {
    {
      let vs = self.0.projects.lock();
      if let Some(ProjectWithId{id:Some(id),..}) = vs.get(&id) {
        return Ok(id.clone())
      }
    }

    macro_rules! ret {
      ($v:expr) => {{
        tracing::info!("Found {:?}",$v);
        let mut lock = self.0.projects.lock();
        if let Some(mut v) = lock.take(&id) {
          v.id = Some($v.clone());
          lock.insert(v);
        }
        drop(lock);
        return Ok($v);
      }}
    }
    let r = gitlab::api::projects::repository::TreeBuilder::default()
      .project(id).ref_(branch).recursive(false).build().unwrap_or_else(|_| unreachable!());
    let r:Vec<crate::TreeEntry> = r.query_async(&self.0.inner).await?;
    let Some(p) = r.into_iter().find_map(|e| if e.path.eq_ignore_ascii_case("meta-inf") && matches!(e.kind, crate::DirOrFile::Dir) { Some(e.path) } else {None}) else {
      ret!(None::<ArchiveId>)
    };

    let r = gitlab::api::projects::repository::TreeBuilder::default()
      .project(id).ref_(branch).path(p).recursive(false).build().unwrap_or_else(|_| unreachable!());
    let r:Vec<crate::TreeEntry> = r.query_async(&self.0.inner).await?;
    let Some(p) = r.into_iter().find_map(|e| if e.name.eq_ignore_ascii_case("manifest.mf") && matches!(e.kind,crate::DirOrFile::File) { Some(e.path) } else {None}) else {
      ret!(None::<ArchiveId>)
    };

    let blob = gitlab::api::projects::repository::files::FileRaw::builder()
      .project(id).file_path(p).ref_(branch).build().unwrap_or_else(|_| unreachable!());
    let r  = gitlab::api::raw(blob).query_async(&self.0.inner).await?;
    let r = std::str::from_utf8(&r)?;
    let r = r.split('\n').find_map(|line| {
      let line = line.trim();
      line.strip_prefix("id:").map(|rest| {
        ArchiveId::new(rest.trim())
      })
    });
    ret!(r)
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