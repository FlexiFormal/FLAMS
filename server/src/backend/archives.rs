use either::Either;
use leptos::{server, ServerFnError};

#[cfg(feature = "ssr")]
use immt_api::backend::archives::{Archive, ArchiveGroup};
#[cfg(feature = "ssr")]
use crate::controller::CONTROLLER;

#[server(MathHub,"/api/backend","GetJson","mathhub")]
pub async fn mathhub() -> Result<String,ServerFnError> {
    Ok(CONTROLLER.mathhub().display().to_string())
}

#[server(ArchiveIds,"/api/backend","GetJson","archive_ids")]
pub async fn archive_ids() -> Result<Vec<String>,ServerFnError> {
    let archs = CONTROLLER.archives().into_iter().map(|a| a.id().to_string()).collect::<Vec<_>>();
    Ok(archs)
}

#[server(GroupAt,"/api/backend","GetJson","archives_at")]
pub async fn archives_at_srv(id:Option<String>) -> Result<Vec<ArchiveLike>,ServerFnError> {
    Ok(match archives_at(id) {
        None => Vec::new(),
        Some(v) => v.iter().map(|e| e.into()).collect()
    })
}

#[cfg(feature = "ssr")]
pub fn archives_at<'a>(id:Option<String>) -> Option<&'a Vec<Either<ArchiveGroup,Archive>>> {
    match id {
        None => Some(CONTROLLER.archives().get_top()),
        Some(id) if id.is_empty() => Some(CONTROLLER.archives().get_top()),
        Some(id) => match CONTROLLER.archives().find(id) {
            Some(arch) => match arch {
                Either::Left(g) => Some(g.archives()),
                Either::Right(_) => None
            },
            None => None
        }
    }
}

#[derive(serde::Serialize,serde::Deserialize)]
pub enum ArchiveLike {
    Archive(String),
    Group(String)
}
#[cfg(feature = "ssr")]
impl From<&Either<ArchiveGroup,Archive>> for ArchiveLike {
    fn from(e:&Either<ArchiveGroup,Archive>) -> Self {
        match e {
            Either::Left(g) => Self::Group(g.id().to_string()),
            Either::Right(a) => Self::Archive(a.id().to_string())
        }
    }
}