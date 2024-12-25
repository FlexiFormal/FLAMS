use gitlab::api::AsyncQuery;

#[derive(Debug,Clone)]
pub struct GitLab {
  inner: gitlab::AsyncGitlab
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
  pub fn load<T:Into<String>+Send+'static>(self,url:&str,token:T) {
    *self.inner.write() = MaybeGitlab::Loading;
    let (url,http) = GitLab::split(url);
    tokio::spawn(async move {
      match GitLab::new(url,token,http).await {
        Ok(gl) => {
          *self.inner.write() = MaybeGitlab::Loaded(gl);
          /*tokio::spawn(async move {
            println!("{:?}",gl.get_projects().await);
          });*/
        }
        Err(e) => {
          tracing::error!("Failed to load gitlab: {e}");
          *self.inner.write() = MaybeGitlab::Failed;
        }
      };
    });
  }
  #[allow(clippy::future_not_send)]
  pub async fn get(&self) -> Option<GitLab> {
    loop {
      let r = self.inner.read();
      match &*r {
        MaybeGitlab::None | MaybeGitlab::Failed => return None,
        MaybeGitlab::Loading => {
          drop(r);
          tokio::time::sleep(std::time::Duration::from_secs_f32(0.1)).await;
        }
        MaybeGitlab::Loaded(gl) => return Some(gl.clone())
      }
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

impl GitLab {
  fn split(s:&str) -> (String,bool) {
    if let Some(r) = s.strip_prefix("https://") {
      return (r.to_string(),false)
    }
    if let Some(r) = s.strip_prefix("http://") {
      return (r.to_string(),true)
    }
    (s.to_string(),false)
  }
  /// #### Errors
  pub async fn new<T:Into<String>>(url:String,token:T,http:bool) -> Result<Self,gitlab::GitlabError> {
    let mut gl = gitlab::GitlabBuilder::new(url,token);
    if http { gl.insecure(); }
    Ok(Self {
      inner: gl.build_async().await?
    })
  }
  #[must_use]
  pub fn new_background<T:Into<String>+Send+'static>(url:&str,token:T) -> GLInstance {
    let r = GLInstance {inner:std::sync::Arc::new(parking_lot::RwLock::new(MaybeGitlab::Loading))};
    r.clone().load(url,token);
    r
  }

  pub async fn get_projects(&self) -> Result<Vec<Project>,Err> {
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
  Str(std::str::Utf8Error)
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

#[derive(Debug,serde::Serialize,serde::Deserialize)]
pub struct Project {
  pub name: String,
  pub path_with_namespace:String
}

/*
[{
	"id":4,
	"description":null,
	"name":"third",
	"name_with_namespace":"some / third",
	"path":"third",
	"path_with_namespace":"some/third",
	"created_at":"2024-12-23T10:43:32.858Z",
	"default_branch":"main",
	"tag_list":[],
	"topics":[],
	"ssh_url_to_repo":"git@gitlab.example.com:some/third.git",
	"http_url_to_repo":"http://gitlab.example.com/some/third.git",
	"web_url":"http://gitlab.example.com/some/third",
	"readme_url":null,
	"forks_count":0,
	"avatar_url":null,
	"star_count":0,
	"last_activity_at":
	"2024-12-23T10:43:32.822Z",
	"namespace":{
		"id":3,
		"name":"some",
		"path":"some",
		"kind":"group",
		"full_path":"some",
		"parent_id":null,
		"avatar_url":null,
		"web_url":"http://gitlab.example.com/groups/some"
	},
	"repository_storage":"default",
	"_links":{
		"self":"http://gitlab.example.com/api/v4/projects/4",
		"issues":"http://gitlab.example.com/api/v4/projects/4/issues",
		"merge_requests":"http://gitlab.example.com/api/v4/projects/4/merge_requests",
		"repo_branches":"http://gitlab.example.com/api/v4/projects/4/repository/branches",
		"labels":"http://gitlab.example.com/api/v4/projects/4/labels",
		"events":"http://gitlab.example.com/api/v4/projects/4/events",
		"members":"http://gitlab.example.com/api/v4/projects/4/members",
		"cluster_agents":"http://gitlab.example.com/api/v4/projects/4/cluster_agents"
	},
	"packages_enabled":true,
	"empty_repo":false,
	"archived":false,
	"visibility":"public",
	"resolve_outdated_diff_discussions":false,
	"container_expiration_policy":{
		"cadence":"1d","enabled":false,"keep_n":10,"older_than":"90d","name_regex":".*",
		"name_regex_keep":null,"next_run_at":"2024-12-24T10:43:32.880Z"
	},
	"repository_object_format":"sha1",
	"issues_enabled":true,
	"merge_requests_enabled":true,
	"wiki_enabled":true,
	"jobs_enabled":true,
	"snippets_enabled":true,
	"container_registry_enabled":true,
	"service_desk_enabled":false,
	"service_desk_address":null,
	"can_create_merge_request_in":true,
	"issues_access_level":"enabled",
	"repository_access_level":"enabled",
	"merge_requests_access_level":"enabled",
	"forking_access_level":"enabled",
	"wiki_access_level":"enabled",
	"builds_access_level":"enabled",
	"snippets_access_level":"enabled",
	"pages_access_level":"private",
	"analytics_access_level":"enabled",
	"container_registry_access_level":"enabled",
	"security_and_compliance_access_level":"private",
	"releases_access_level":"enabled",
	"environments_access_level":"enabled",
	"feature_flags_access_level":"enabled",
	"infrastructure_access_level":"enabled",
	"monitor_access_level":"enabled",
	"model_experiments_access_level":"enabled",
	"model_registry_access_level":"enabled",
	"emails_disabled":false,
	"emails_enabled":true,
	"shared_runners_enabled":true,
	"lfs_enabled":true,
	"creator_id":2,
	"import_url":null,
	"import_type":null,
	"import_status":"none",
	"open_issues_count":0,
	"description_html":"",
	"updated_at":"2024-12-23T10:44:06.647Z",
	"ci_default_git_depth":20,
	"ci_delete_pipelines_in_seconds":null,
	"ci_forward_deployment_enabled":true,
	"ci_forward_deployment_rollback_allowed":true,
	"ci_job_token_scope_enabled":false,
	"ci_separated_caches":true,
	"ci_allow_fork_pipelines_to_run_in_parent_project":true,
	"ci_id_token_sub_claim_components":["project_path","ref_type","ref"],
	"build_git_strategy":"fetch",
	"keep_latest_artifact":true,
	"restrict_user_defined_variables":true,
	"ci_pipeline_variables_minimum_override_role":"developer",
	"runners_token":"GR1348941Q6BMWPWzxwoMbcceaybe",
	"runner_token_expiration_interval":null,
	"group_runners_enabled":true,
	"auto_cancel_pending_pipelines":"enabled",
	"build_timeout":3600,
	"auto_devops_enabled":true,
	"auto_devops_deploy_strategy":"continuous",
	"ci_push_repository_for_job_token_allowed":false,
	"ci_config_path":null,
	"public_jobs":true,
	"shared_with_groups":[],
	"only_allow_merge_if_pipeline_succeeds":false,
	"allow_merge_on_skipped_pipeline":null,
	"request_access_enabled":true,
	"only_allow_merge_if_all_discussions_are_resolved":false,
	"remove_source_branch_after_merge":true,
	"printing_merge_request_link_enabled":true,
	"merge_method":"merge",
	"squash_option":"default_off",
	"enforce_auth_checks_on_uploads":true,
	"suggestion_commit_message":null,
	"merge_commit_template":null,
	"squash_commit_template":null,
	"issue_branch_template":null,
	"warn_about_potentially_unwanted_characters":true,
	"autoclose_referenced_issues":true,
	"permissions":{"project_access":null,"group_access":null}
}]
*/