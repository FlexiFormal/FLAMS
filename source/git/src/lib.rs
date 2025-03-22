#[cfg(feature = "gitlab")]
pub mod gl;

#[cfg(feature = "git2")]
pub mod repos;

lazy_static::lazy_static! {
    pub(crate) static ref REMOTE_SPAN:tracing::Span = {
            //println!("Here!");
            tracing::info_span!(target:"git",parent:None,"git")
    };
}

#[cfg(any(feature = "git2", feature = "gitlab"))]
pub trait GitUrlExt {
    fn into_https(self) -> Self;
}
#[cfg(any(feature = "git2", feature = "gitlab"))]
impl GitUrlExt for git_url_parse::GitUrl {
    fn into_https(mut self) -> Self {
        self = self.trim_auth();
        self.scheme = git_url_parse::Scheme::Https;
        self.scheme_prefix = true;
        if !self.path.starts_with('/') {
            self.path = format!("/{}", self.path);
        }
        self
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Project {
    pub id: u64,
    pub name: String,
    #[serde(rename = "path_with_namespace")]
    pub path: String,
    #[serde(rename = "http_url_to_repo")]
    pub url: String,
    pub default_branch: Option<String>,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Branch {
    pub name: String,
    pub default: bool,
    pub commit: Commit,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Commit {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub parent_ids: Vec<String>,
    pub title: String,
    pub message: String,
    pub author_name: String,
}

/*
[
    {
        "name":"main",
        "commit":{
            "id":"2c10ef48497cb4af1068e194cfe49ea1444321de",
            "short_id":"2c10ef48",
            "created_at":"2024-12-23T11:44:01.000+01:00",
            "parent_ids":[],
            "title":"init",
            "message":"init\n",
            "author_name":"Jazzpirate",
            "author_email":"raupi@jazzpirate.com",
            "authored_date":"2024-12-23T11:44:01.000+01:00",
            "committer_name":"Jazzpirate",
            "committer_email":"raupi@jazzpirate.com",
            "committed_date":"2024-12-23T11:44:01.000+01:00",
            "trailers":{},
            "extended_trailers":{},
            "web_url":"http://gitlab.example.com/some/third/-/commit/2c10ef48497cb4af1068e194cfe49ea1444321de"
        },
        "merged":false,
        "protected":true,
        "developers_can_push":false,
        "developers_can_merge":false,
        "can_push":true,
        "default":true,
        "web_url":"http://gitlab.example.com/some/third/-/tree/main"
    }
]
     */

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct TreeEntry {
    name: String,
    path: String,
    #[serde(rename = "type")]
    kind: DirOrFile,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
enum DirOrFile {
    #[serde(rename = "tree")]
    Dir,
    #[serde(rename = "blob")]
    File,
}

/*
{
    "id":"2f3cbdf17d7970bda62c7e749b9395295980a5ee",
    "name":"META-INF",
    "type":"tree"|"blob",
    "path":"META-INF",
    "mode":"040000"
}
*/
