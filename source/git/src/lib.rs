#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(feature = "gitlab")]
pub mod gl;
#[cfg(feature = "git2")]
pub mod repos;

#[cfg(any(feature = "git2", feature = "gitlab"))]
pub use git_url_parse::GitUrl;


#[cfg(any(feature = "git2", feature = "gitlab"))]
pub(crate) static REMOTE_SPAN: std::sync::LazyLock<tracing::Span> =
    std::sync::LazyLock::new(|| tracing::info_span!(target:"git",parent:None,"git"));

#[cfg(any(feature = "git2", feature = "gitlab"))]
pub trait GitUrlExt {
    #[must_use]
    fn into_https(self) -> Self;
}
#[cfg(any(feature = "git2", feature = "gitlab"))]
impl GitUrlExt for GitUrl {
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
