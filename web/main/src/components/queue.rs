use leptos::*;
use immt_core::building::formats::{BuildJobSpec, FormatOrTarget, SourceFormatId};
use immt_core::uris::archives::ArchiveId;
use crate::accounts::{if_logged_in, login_status, LoginState};
use crate::utils::errors::IMMTError;

#[server(
    prefix="/api/buildqueue",
    endpoint="enqueue",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
)]
pub async fn enqueue(archive:Option<String>,group:Option<String>,target:SourceFormatId,path:Option<String>,all:bool) -> Result<(),ServerFnError<IMMTError>> {
    use immt_controller::{controller,ControllerTrait};
    use immt_api::backend::archives::{Archive, Storage};
    use immt_core::utils::filetree::{FileLike, SourceDirEntry};
    use immt_core::prelude::DirLike;
    match login_status().await? {
        LoginState::Admin => {
            println!("building [{group:?}][{archive:?}]/{path:?}: {target} ({all})");
            let spec = match (archive,group,path) {
                (None,Some(_),Some(_)) | (None,None,_) => {
                    return Err(ServerFnError::MissingArg("Must specify either an archive with optional path or a group".into()))
                },
                (Some(a),_,Some(p)) => {
                    let id : ArchiveId = a.parse().map_err(|_| IMMTError::InvalidArgument("archive"))?;
                    BuildJobSpec::Path {id,rel_path:p.into(),target:FormatOrTarget::Format(target),stale_only:!all}
                },
                (Some(a),_,_) => {
                    let id : ArchiveId = a.parse().map_err(|_| IMMTError::InvalidArgument("archive"))?;
                    BuildJobSpec::Archive {id,target:FormatOrTarget::Format(target),stale_only:!all}
                },
                (_,Some(a),_) => {
                    let id : ArchiveId = a.parse().map_err(|_| IMMTError::InvalidArgument("group"))?;
                    BuildJobSpec::Group {id,target:FormatOrTarget::Format(target),stale_only:!all}
                }
            };
            let controller = controller();
            controller.build_queue().enqueue(spec);
            Ok(())
        },
        _ => Err(IMMTError::AccessForbidden.into())
    }
}

#[island]
pub fn Queue() -> impl IntoView {
    move || if_logged_in(
        || template!{
            <div>"Queue"</div>
        },
        || template!{
            <div>"Please log in to view the queue"</div>
        }
    )
}
