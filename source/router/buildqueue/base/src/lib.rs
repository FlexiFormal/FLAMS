use flams_ontology::uris::ArchiveId;
use flams_router_base::LoginState;
use flams_utils::unwrap;
use flams_web_utils::inject_css;
use std::num::NonZeroU32;

pub mod server_fns;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum FormatOrTarget {
    Format(String),
    Targets(Vec<String>),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueueInfo {
    pub id: NonZeroU32,
    pub name: String,
    pub archives: Option<Vec<RepoInfo>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RepoInfo {
    Copy(ArchiveId),
    Git {
        id: ArchiveId,
        remote: String,
        branch: String,
        commit: flams_git::Commit,
        //updates:Vec<(String,flams_git::Commit)>
    },
}

#[cfg(feature = "ssr")]
mod login {
    use std::num::NonZeroU32;

    use flams_router_base::LoginState;

    pub trait LoginQueue {
        /// #### Errors
        fn with_queue<R>(
            &self,
            id: NonZeroU32,
            f: impl FnOnce(&flams_system::building::Queue) -> R,
        ) -> Result<R, String>;

        /// #### Errors
        fn with_opt_queue<R>(
            &self,
            id: Option<NonZeroU32>,
            f: impl FnOnce(
                flams_system::building::queue_manager::QueueId,
                &flams_system::building::Queue,
            ) -> R,
        ) -> Result<R, String>;
    }
    #[cfg(feature = "ssr")]
    impl LoginQueue for LoginState {
        fn with_queue<R>(
            &self,
            id: NonZeroU32,
            f: impl FnOnce(&flams_system::building::Queue) -> R,
        ) -> Result<R, String> {
            use flams_system::building::QueueName;
            let qm = flams_system::building::queue_manager::QueueManager::get();
            match self {
                Self::None | Self::Loading => {
                    return Err(format!("Not logged in: {self:?}"));
                }
                Self::Admin | Self::NoAccounts | Self::User { is_admin: true, .. } => (),
                Self::User { name, .. } => {
                    return qm.with_queue(id.into(), move |q| q.map_or_else(
                        || Err(format!("Queue {id} not found")),
                        |q| if matches!(q.name(),QueueName::Sandbox{name:qname,..} if &**qname == name)
                        {
                            Ok(f(q))
                        } else {
                            Err(format!("Not allowed to run queue {id}"))
                        }
                    ));
                }
            }
            qm.with_queue(id.into(), move |q| {
                q.map_or_else(|| Err(format!("Queue {id} not found")), |q| Ok(f(q)))
            })
        }

        fn with_opt_queue<R>(
            &self,
            id: Option<NonZeroU32>,
            f: impl FnOnce(
                flams_system::building::queue_manager::QueueId,
                &flams_system::building::Queue,
            ) -> R,
        ) -> Result<R, String> {
            use flams_system::building::QueueName;
            let qm = flams_system::building::queue_manager::QueueManager::get();
            match (self, id) {
                (Self::None | Self::Loading, _) =>
                    Err(format!("Not logged in: {self:?}")),
                (Self::User { name, .. }, Some(id)) => qm.with_queue(id.into(), move |q|
                   q.map_or_else(
                       || Err(format!("Queue {id} not found")),
                       |q| if matches!(q.name(),QueueName::Sandbox{name:qname,..} if &**qname == name)
                       {
                           Ok(f(id.into(), q))
                       } else {
                           Err(format!("Not allowed to run queue {id}"))
                       }
                   )),
                (Self::Admin, Some(id)) => qm.with_queue(id.into(), |q|
                    q.map_or_else(
                        || Err(format!("Queue {id} not found")),
                        |q| Ok(f(id.into(), q))
                    )),
                (Self::User { name, .. }, _) => {
                    let queue = qm.new_queue(name);
                    qm.with_queue(queue, |q| {
                        let Some(q) = q else { unreachable!() };
                        Ok(f(queue, q))
                    })
                }
                (Self::Admin, _) => {
                    let queue = qm.new_queue("admin");
                    qm.with_queue(queue, |q| {
                        let Some(q) = q else { unreachable!() };
                        Ok(f(queue, q))
                    })
                }
                (Self::NoAccounts, _) => qm.with_global(|q| {
                    Ok(f(
                        flams_system::building::queue_manager::QueueId::global(),
                        q,
                    ))
                }),
            }
        }
    }
}
#[cfg(feature = "ssr")]
pub use login::*;

use leptos::prelude::*;
pub fn select_queue(queue_id: RwSignal<Option<NonZeroU32>>) -> impl IntoView {
    use flams_web_utils::components::{Spinner, display_error};
    move || {
        let user = LoginState::get();
        if matches!(user, LoginState::NoAccounts) {
            return None;
        }
        let r = Resource::new(|| (), move |()| server_fns::get_queues());
        Some(view! {<Suspense fallback = || view!(<Spinner/>)>{move || {
          match r.get() {
            None => leptos::either::EitherOf3::A(view!(<Spinner/>)),
            Some(Err(e)) => leptos::either::EitherOf3::B(display_error(e.to_string().into())),
            Some(Ok(queues)) => leptos::either::EitherOf3::C(view!{<div><div style="width:fit-content;margin-left:auto;">{
                do_queues(queue_id,queues)
            }</div></div>})
          }
        }}</Suspense>})
    }
}

fn do_queues(queue_id: RwSignal<Option<NonZeroU32>>, v: Vec<QueueInfo>) -> impl IntoView {
    use thaw::Select;
    inject_css("flams-select-queue", include_str!("select_queue.css"));
    let queues = if v.is_empty() {
        vec![(0u32, "New Build Queue".to_string())]
    } else {
        v.into_iter()
            .map(|q| (q.id.get(), q.name))
            .chain(std::iter::once((0u32, "New Build Queue".to_string())))
            .collect()
    };
    let value = RwSignal::new(unwrap!(queues.first()).clone().1);
    let qc = queues.clone();
    let _ = Effect::new(move |_| {
        let queue = value.get();
        if queue == "New Build Queue" {
            queue_id.update_untracked(|v| *v = None);
        } else {
            let idx = unwrap!(? qc.iter().find_map(|(id,name)| if *name == queue {Some(*id)} else {None}));
            queue_id.update_untracked(|v| *v = Some(unwrap!(NonZeroU32::new(idx))));
        }
    });
    view! {
      <span style="font-style:italic;">"Build Queue: "
      <Select value class="flams-select-queue">{
        queues.into_iter().map(|(_,name)| view!{
          <option value=name>{name.clone()}</option>
        }).collect_view()
      }</Select></span>
    }
}
