use git2::{build::CheckoutBuilder, RepositoryOpenFlags};
use std::path::Path;

use crate::GitUrlExt;

macro_rules! in_git {
	(($($tt:tt)*); $b:block) => {{
		let span = ::tracing::debug_span!(parent:&*$crate::REMOTE_SPAN,$($tt)*);
		span.in_scope(|| $b).map_err(|e| {
			span.in_scope(|| ::tracing::error!("Error: {e}"));
			e
		})
	}};
}

pub struct GitRepo(git2::Repository);

impl From<git2::Repository> for GitRepo {
    #[inline]
    fn from(r: git2::Repository) -> Self {
        Self(r)
    }
}
impl From<git2::Commit<'_>> for super::Commit {
    #[allow(clippy::cast_lossless)]
    fn from(commit: git2::Commit) -> Self {
        let time = commit.time();
        let author_name = commit
            .author()
            .name()
            .map(ToString::to_string)
            .unwrap_or_default();
        Self {
            id: commit.id().to_string(),
            created_at: chrono::DateTime::from_timestamp(
                time.seconds() + (time.offset_minutes() as i64 * 60),
                0,
            )
            .unwrap_or_else(|| unreachable!()),
            title: commit
                .summary()
                .map(ToString::to_string)
                .unwrap_or_default(),
            parent_ids: commit.parent_ids().map(|p| p.to_string()).collect(),
            message: commit
                .message()
                .map(ToString::to_string)
                .unwrap_or_default(),
            author_name,
        }
    }
}

const NOTES_NS: &str = "refs/notes/flams";

impl GitRepo {
    /// #### Errors
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, git2::Error> {
        let repo = git2::Repository::open_ext(
            path,
            RepositoryOpenFlags::NO_SEARCH.intersection(RepositoryOpenFlags::NO_DOTGIT),
            std::iter::empty::<&std::ffi::OsStr>(),
        )?;
        Ok(Self(repo))
    }

    #[must_use]
    pub fn is_managed(&self, gl_host: &str) -> Option<git_url_parse::GitUrl> {
        self.get_origin_url().ok().and_then(|url| {
            if url.host.as_ref().is_some_and(|h| h == gl_host) {
                Some(url)
            } else {
                None
            }
        })
    }

    /// #### Errors
    pub fn get_origin_url(&self) -> Result<git_url_parse::GitUrl, git2::Error> {
        let remote = self.0.find_remote("origin")?;
        let Some(url) = remote.url() else {
            return Err(git2::Error::from_str("No origin"));
        };
        let mut url =
            git_url_parse::GitUrl::parse(url).map_err(|e| git2::Error::from_str(&e.to_string()))?;
        // enforce HTTPS (because oauth; for now)
        if matches!(
            url.scheme,
            git_url_parse::Scheme::Ssh | git_url_parse::Scheme::GitSsh
        ) {
            url = url.into_https();
            self.0.remote_set_url("origin", &url.to_string())?;
        }

        Ok(url)
    }

    /// #### Errors
    pub fn add_note(&self, note: &str) -> Result<(), git2::Error> {
        let head = self.0.head()?.peel_to_commit()?.id();
        let sig = self.0.signature()?;
        self.0.note(&sig, &sig, Some(NOTES_NS), head, note, true)?;
        Ok(())
    }

    /// #### Errors
    pub fn with_latest_note<R>(&self, f: impl FnOnce(&str) -> R) -> Result<Option<R>, git2::Error> {
        let head = self.0.head()?.peel_to_commit()?.id();
        self.0
            .find_note(Some(NOTES_NS), head)
            .map(|n| n.message().map(f))
    }

    /// #### Errors
    #[inline]
    pub fn clone_from_oauth(
        token: &str,
        url: &str,
        branch: &str,
        to: &Path,
        shallow: bool,
    ) -> Result<Self, git2::Error> {
        Self::clone("oauth2", token, url, branch, to, shallow)
    }

    /// #### Errors
    pub fn clone(
        user: &str,
        password: &str,
        url: &str,
        branch: &str,
        to: &Path,
        shallow: bool,
    ) -> Result<Self, git2::Error> {
        use git2::{build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks};
        in_git!(("git clone",url=url,branch=branch); {
          let _ = std::fs::create_dir_all(to);
          let mut cbs = RemoteCallbacks::new();
          cbs.credentials(|_,_,_| Cred::userpass_plaintext(user, password));

          let mut fetch = FetchOptions::new();
          fetch.remote_callbacks(cbs);
          if shallow { fetch.depth(1); }

          let repo = RepoBuilder::new()
            .fetch_options(fetch)
            .bare(false)
            .branch(branch)
            .clone(url,to)?;
          Ok(repo.into())
        })
    }

    /// #### Errors
    #[inline]
    pub fn fetch_branch_from_oauth(
        &self,
        token: &str,
        branch: &str,
        shallow: bool,
    ) -> Result<(), git2::Error> {
        self.fetch_branch("oauth2", token, branch, shallow)
    }

    /// #### Errors
    pub fn fetch_branch(
        &self,
        user: &str,
        password: &str,
        branch: &str,
        shallow: bool,
    ) -> Result<(), git2::Error> {
        in_git!(("git fetch",path=%self.0.path().display(),branch=branch); {
          let mut cbs = git2::RemoteCallbacks::new();
          cbs.credentials(|_,_,_| git2::Cred::userpass_plaintext(user, password));
          let mut fetch = git2::FetchOptions::new();
          fetch.remote_callbacks(cbs);
          if shallow { fetch.depth(1); }
          self.0.find_remote("origin")?
            .fetch(&[branch,&format!("+{NOTES_NS}:{NOTES_NS}")], Some(&mut fetch), None)?;
          /*
          let remote = self.0.find_branch(&format!("origin/{branch}"), git2::BranchType::Remote)?;
          let commit = remote.get().peel_to_commit()?;
          if let Ok(mut local) = self.0.find_branch(branch, git2::BranchType::Local) {
            local.get_mut().set_target(commit.id(), "fast forward")?;
            Ok(())
          } else {
            self.0.branch(branch, &commit, false)?.set_upstream(Some(&format!("origin/{branch}")))
          }*/
          Ok(())
        })
    }

    /// #### Errors
    #[inline]
    pub fn push_with_oauth(&self, secret: &str) -> Result<(), git2::Error> {
        self.push("oauth2", secret)
    }

    /// #### Errors
    pub fn push(&self, user: &str, password: &str) -> Result<(), git2::Error> {
        in_git!(("git push",path=%self.0.path().display()); {
          let head = self.0.head()?;
          if !head.is_branch() { return Err(git2::Error::from_str("no branch checked out")); }
          let Some(branch_name) = head.shorthand() else {
            return Err(git2::Error::from_str("no branch checked out"));
          };
          let mut remote = self.0.find_remote("origin")?;
          let mut cbs = git2::RemoteCallbacks::new();
          cbs.credentials(|_,_,_| git2::Cred::userpass_plaintext(user, password));
          let mut opts = git2::PushOptions::new();
          opts.remote_callbacks(cbs);
          remote.push(&[
            format!("+refs/heads/{branch_name}:refs/heads/{branch_name}").as_str(),
            NOTES_NS
          ],Some(&mut opts))?;
          Ok(())
        })
    }

    /// #### Errors
    #[inline]
    pub fn get_new_commits_with_oauth(
        &self,
        token: &str,
    ) -> Result<Vec<(String, super::Commit)>, git2::Error> {
        self.get_new_commits("oauth2", token)
    }

    #[allow(dead_code)]
    fn print_history(&self, commit: &git2::Commit) {
        println!("commit {:.8}", commit.id());
        self.walk(commit.clone(), |id| {
            println!(" - {id:.8}");
            true
        });
        /*
        let Ok(mut revwalk) = self.0.revwalk() else {return};
        /*let Ok(_) = revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME) else {
          println!("ERROR SORTING");
          return
        };*/
        revwalk.push(commit.id());
        println!("commit {}",commit.id());
        for oid in revwalk {
          let Ok(id) = oid else {continue};
          println!(" - {}",id);
          let Ok(commit) = self.0.find_commit(id) else {
            println!("NOT FOUND!"); continue
          };
          if commit.parent_count() > 1 {
            print!("Merge:");
            for i in 0..commit.parent_count() {
              let Ok(p)  = commit.parent_id(i) else {
                print!("(MISSING)"); continue
              };
              print!(" {:.8}",p);
            }
            println!();
          }
        }
         */
    }

    fn walk(&self, commit: git2::Commit, mut f: impl FnMut(git2::Oid) -> bool) {
        const MAX_DEPTH: u16 = 500;
        // safe walking over commit history; compatible with missing commits,
        // unlike self.0.revwalk()
        let mut todos = smallvec::SmallVec::<_, 4>::new();
        let mut checked = 0;
        todos.push(commit);
        while let Some(next) = todos.pop() {
            checked += 1;
            if checked > MAX_DEPTH {
                return;
            }
            //tracing::info!("Walking {} {}",next.id(),todos.len());
            //let num = next.parent_count();
            for i in 0..next.parent_count() {
                let Ok(id) = next.parent_id(i) else { continue };
                if !f(id) {
                    return;
                }
                if let Ok(commit) = self.0.find_commit(id) {
                    todos.push(commit);
                }
            }
        }
    }

    /// #### Errors
    pub fn get_new_commits(
        &self,
        user: &str,
        password: &str,
    ) -> Result<Vec<(String, super::Commit)>, git2::Error> {
        in_git!(("get new commits",path=%self.0.path().display()); {
          let mut remote = self.0.find_remote("origin")?;
          let mut cbs = git2::RemoteCallbacks::new();
          cbs.credentials(|_,_,_| git2::Cred::userpass_plaintext(user,password));
          tracing::debug!("Fetching new commits");
          remote.fetch(&[
              "+refs/heads/*:refs/remotes/origin/*",
              &format!("{NOTES_NS}:{NOTES_NS}")
            ],Some(
              git2::FetchOptions::new().remote_callbacks(cbs)
            ),None)?;
          tracing::debug!("Fetching done.");
          let head = self.0.head()?.peel_to_commit()?;
          /*let Some(s) = self.get_managed()? else {
            return Ok(Vec::new())
          };
          let Some((_,managed)) = s.split_once(';') else {
            return Err(git2::Error::from_str("unexpected git note on release branch"))
          };
          let managed_id = git2::Oid::from_str(managed)?;
          let managed = self.0.find_commit(managed_id)?;*/
            let head_id = head.id();
          let mut new_commits = Vec::new();
          for branch in self.0.branches(Some(git2::BranchType::Remote))? {
            let (branch,_) = branch?;
            let Some(branch_name) = branch.name()? else {continue};
            if branch_name == "origin/HEAD" /*|| branch_name == "origin/release"*/ {continue}
            let branch_name = branch_name.strip_prefix("origin/").unwrap_or(branch_name);
            let tip_commit = branch.get().peel_to_commit()?;
            if tip_commit.id() == head_id/*managed_id*/ { continue }
            let mut found = false;
            self.walk(tip_commit.clone(),|id|
              if id == head_id {found = true;false} else {true}
            );
            if found {
              new_commits.push((branch_name.to_string(),tip_commit.into()));
            }
          }
          Ok(new_commits)
        })

        /*

        let mut history = HSet::default();
        history.insert(head.id());
        self.walk(head.clone(),|id| {history.insert(id);true});

        let mut new_commits = Vec::new();
        for branch in self.0.branches(Some(git2::BranchType::Remote))? {
          let (branch,_) = branch?;
          let Some(branch_name) = branch.name()? else {continue};
          if branch_name == "origin/HEAD" {continue}
          let branch_name = branch_name.strip_prefix("origin/").unwrap_or(branch_name);
          let tip_commit = branch.get().peel_to_commit()?;
          if history.contains(&tip_commit.id()) { continue }
          let mut found = false;
          self.walk(tip_commit.clone(),|id|
            if history.contains(&id) {found = true;false} else {true}
          );
          if found {
            new_commits.push((branch_name.to_string(),tip_commit.into()));
          }
        }

        Ok(new_commits)
         */
    }

    /*
    /// #### Errors
    pub fn release_commit_id(&self) -> Result<String,git2::Error> {
      let head = self.0.head()?.peel_to_commit()?;
      let release = self.0.find_branch("release", git2::BranchType::Local)?.get().peel_to_commit()?;
      if head.id() == release.id() { Ok(head.id().to_string()) }
      else { Err(git2::Error::from_str("not on release branch")) }
    }
     */

    /// #### Errors
    pub fn current_commit(&self) -> Result<super::Commit, git2::Error> {
        let commit = self.0.head()?.peel_to_commit()?;
        Ok(commit.into())
    }

    /// #### Errors
    pub fn current_commit_on(&self, branch: &str) -> Result<super::Commit, git2::Error> {
        let commit = self
            .0
            .find_branch(branch, git2::BranchType::Local)?
            .get()
            .peel_to_commit()?;
        Ok(commit.into())
    }
    /// #### Errors
    pub fn current_remote_commit_on(&self, branch: &str) -> Result<super::Commit, git2::Error> {
        let commit = self
            .0
            .find_branch(&format!("origin/{branch}"), git2::BranchType::Remote)?
            .get()
            .peel_to_commit()?;
        Ok(commit.into())
    }

    /// #### Errors
    pub fn commit_all(&self, message: &str) -> Result<super::Commit, git2::Error> {
        in_git!(("commit all",path=%self.0.path().display(),commit_message=message); {
          let mut index = self.0.index()?;
          //let managed = self.get_managed()?;
          let id = index.write_tree()?;
          let tree = self.0.find_tree(id)?;
          let parent = self.0.head()?.peel_to_commit()?;
          let sig = self.0.signature()?;
          let commit = self.0.commit(
            Some("HEAD"),
            &sig, &sig,
            message, &tree, &[&parent]
          )?;
          let commit = self.0.find_commit(commit)?;
          /*if let Some(mg) = managed {
            self.add_note(&mg)?
          }*/
          Ok(commit.into())
        })
    }

    /// #### Errors
    pub fn new_branch(&self, name: &str) -> Result<(), git2::Error> {
        in_git!(("new branch",path=%self.0.path().display(),branch=name); {
          let head = self.0.head()?.peel_to_commit()?;
          let mut branch = self.0.branch(name,&head,false)?;
          let _ = self.0.find_remote("origin")?;
          let _ = self.0.reference(
            &format!("refs/remotes/origin/{name}"),
            head.id(),
            false,
            "create remote branch"
          )?;
          branch.set_upstream(Some(&format!("origin/{name}")))?;
          let Some(name) = branch.get().name() else {
            return Err(git2::Error::from_str("failed to create branch"));
          };
          self.0.set_head(name)?;
          self.0.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
        })
    }

    /// #### Errors
    pub fn force_checkout(&self, commit: &str) -> Result<(), git2::Error> {
        in_git!(("checkout",path=%self.0.path().display(),commit=commit); {
          let id = git2::Oid::from_str(commit)?;
          let a_commit = self.0.find_annotated_commit(id)?;
          let commit = self.0.find_commit(id)?;
          let head = self.0.head()?.peel_to_commit()?;
          if /*head.id() == commit.id() ||*/ self.0.graph_descendant_of(head.id(), commit.id())? {
              tracing::debug!("HEAD is descendant of commit!");
              return Ok(())
          }
          let (analysis,_) = self.0.merge_analysis(&[&a_commit])?;
          if analysis.is_up_to_date() { return Ok(())}
          if analysis.is_fast_forward() {
              let head = self.0.head()?;
              let name = head.name().ok_or_else(|| git2::Error::from_str("No name for HEAD reference"))?;
              let _ = self.0.reference(name,id,true,"Fast-forward");
              return self.0.checkout_head(Some(CheckoutBuilder::new().force()));
          }

          let mut merge_options = git2::MergeOptions::new();
          merge_options
            .file_favor(git2::FileFavor::Theirs)
            .fail_on_conflict(false);
          tracing::debug!("Merging");
          self.0.merge(
            &[&a_commit],
            Some(&mut merge_options),
            Some(&mut git2::build::CheckoutBuilder::new()),
          )?;
          tracing::debug!("Checking out HEAD");
          self.0.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
          let mut index = self.0.index()?;
          if index.has_conflicts() {
            tracing::debug!("Index has conflicts now");
            let mut entries = Vec::new();
            for conflict in index.conflicts()? {
              let conflict = conflict?;
              if let Some(entry) = conflict.their {
                entries.push(entry);
              }
            }
            for e in entries {
              index.add(&e)?;
            }
            index.write()?;
          } {
              tracing::debug!("No conflicts");
          }

          if index.has_conflicts() || self.0.state() == git2::RepositoryState::Merge {
            tracing::debug!("In merge state; commit necessary");
            let sig = self.0.signature()?;
            let tree_id = index.write_tree()?;
            let tree = self.0.find_tree(tree_id)?;
            self.0.commit(
              Some("HEAD"),
              &sig, &sig,
              &format!("Merge commit {}",commit.id()),
              &tree,
              &[&head,&commit]
            )?;
          }
          self.0.cleanup_state()?;
          Ok(())
        })
    }

    /// #### Errors
    pub fn merge(&self, commit: &str) -> Result<(), git2::Error> {
        in_git!(("merge",path=%self.0.path().display(),commit=commit); {
          let id = git2::Oid::from_str(commit)?;
          let a_commit = self.0.find_annotated_commit(id)?;
          let commit = self.0.find_commit(id)?;
          let mut merge_options = git2::MergeOptions::new();
          merge_options.file_favor(git2::FileFavor::Theirs);
          let mut checkout_options = git2::build::CheckoutBuilder::new();
          let parent = self.0.head()?.peel_to_commit()?;
          self.0.merge(
            &[&a_commit],
            Some(&mut merge_options),
            Some(&mut checkout_options),
          )?;
          let sig = self.0.signature()?;
          let tree_id = self.0.index()?.write_tree()?;
          let tree = self.0.find_tree(tree_id)?;
          self.0.commit(
            Some("HEAD"),
            &sig, &sig,
            &format!("Merge commit {}",commit.id()),
            &tree,
            &[&parent,&commit]
          ).map(|_| ())
        })
    }

    /// #### Errors
    pub fn add_dir(&self, path: &Path, force: bool) -> Result<(), git2::Error> {
        in_git!(("git add",path=%self.0.path().display(),dir=%path.display()); {
          let mut index = self.0.index()?;
          for entry in walkdir::WalkDir::new(path)
            .min_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file()) {
              let relative_path = entry.path().strip_prefix(self.0.path().parent().unwrap_or_else(|| unreachable!()))
                .map_err(|e| git2::Error::from_str(&e.to_string()))?;
              if force || !self.0.is_path_ignored(relative_path)? {
                index.add_path(relative_path)?;
              }
            }
          index.write()?;
          Ok(())
        })
    }
}

/*
#[test]
fn test_new_commits() {
  tracing_subscriber::fmt().init();
  let repo = GitRepo::open(Path::new("/home/jazzpirate/work/coursetest")).unwrap();
  let commit = repo.0.find_branch("main", git2::BranchType::Local).unwrap().get().peel_to_commit().unwrap();
  repo.walk(commit,|_| true);
}
   */
