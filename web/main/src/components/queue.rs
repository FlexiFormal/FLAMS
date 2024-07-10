use std::collections::HashMap;
use async_trait::async_trait;
use leptos::*;
use immt_core::building::buildstate::{QueueEntry, QueueMessage};
use immt_core::building::formats::{BuildJobSpec, FormatOrTarget, SourceFormatId};
use immt_core::uris::archives::ArchiveId;
use crate::accounts::{if_logged_in, login_status, LoginState};
use crate::components::logging::Log;
use crate::console_log;
use crate::utils::errors::IMMTError;
use crate::utils::WebSocket;

#[server(
    prefix="/api/buildqueue",
    endpoint="enqueue",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
)]
pub async fn enqueue(archive:Option<String>,group:Option<String>,target:SourceFormatId,path:Option<String>,all:bool) -> Result<(),ServerFnError<IMMTError>> {
    use immt_controller::{controller,ControllerTrait};
    match login_status().await? {
        LoginState::Admin => {
            //println!("building [{group:?}][{archive:?}]/{path:?}: {target} ({all})");
            let spec = match (archive,group,path) {
                (None,Some(_),Some(_)) | (None,None,_) => {
                    return Err(ServerFnError::MissingArg("Must specify either an archive with optional path or a group".into()))
                },
                (Some(a),_,Some(p)) => {
                    let id : ArchiveId = a.parse().map_err(|_| IMMTError::InvalidArgument("archive".to_string()))?;
                    BuildJobSpec::Path {id,rel_path:p.into(),target:FormatOrTarget::Format(target),stale_only:!all}
                },
                (Some(a),_,_) => {
                    let id : ArchiveId = a.parse().map_err(|_| IMMTError::InvalidArgument("archive".to_string()))?;
                    BuildJobSpec::Archive {id,target:FormatOrTarget::Format(target),stale_only:!all}
                },
                (_,Some(a),_) => {
                    let id : ArchiveId = a.parse().map_err(|_| IMMTError::InvalidArgument("group".to_string()))?;
                    BuildJobSpec::Group {id,target:FormatOrTarget::Format(target),stale_only:!all}
                }
            };
            let controller = controller();
            {controller.build_queue().enqueue(spec,controller)}
            Ok(())
        },
        _ => Err(IMMTError::AccessForbidden.into())
    }
}

#[server(
    prefix="/api/buildqueue",
    endpoint="get_queues",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
)]
pub async fn get_queues() -> Result<Vec<String>,ServerFnError<IMMTError>> {
    use immt_controller::{controller,ControllerTrait};
    match login_status().await? {
        LoginState::Admin => {
            Ok(controller().build_queue().queues()
                .iter().map(|q| q.id().to_string()).collect())
        },
        _ => Err(IMMTError::AccessForbidden.into())
    }
}

#[server(
    prefix="/api/buildqueue",
    endpoint="run",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
)]
pub async fn run(id:String) -> Result<(),ServerFnError<IMMTError>> {
    use immt_controller::{controller,ControllerTrait};
    //println!("Running queue {id}");
    match login_status().await? {
        LoginState::Admin => {
            controller().build_queue().start(&id);
            Ok(())
        },
        _ => Err(IMMTError::AccessForbidden.into())
    }
}

pub(crate) struct QueueSocket {
    #[cfg(feature="client")]
    socket: web_sys::WebSocket,
    #[cfg(feature="server")]
    listener: immt_api::utils::asyncs::ChangeListener<QueueMessage>
}
#[async_trait]
impl WebSocket<(),QueueMessage> for QueueSocket {
    const SERVER_ENDPOINT: &'static str = "/dashboard/queue/ws";
    #[cfg(feature="server")]
    async fn new(account:LoginState,db:sea_orm::DatabaseConnection) -> Option<Self> {
        use immt_api::controller::Controller;
        if account == LoginState::Admin {
            let listener = immt_controller::controller().build_queue().listener();
            Some(Self {listener})
        } else {None}
    }
    #[cfg(feature="client")]
    fn new(ws: leptos::web_sys::WebSocket) -> Self { Self{socket:ws} }
    #[cfg(feature="server")]
    async fn next(&mut self) -> Option<QueueMessage> {
        self.listener.read().await
    }
    #[cfg(feature="server")]
    async fn handle_message(&mut self,msg:()) -> Option<QueueMessage> {None}
    #[cfg(feature="server")]
    async fn on_start(&mut self,socket:&mut axum::extract::ws::WebSocket) {
        use immt_controller::{controller,ControllerTrait};
        let msgs = controller().build_queue().queues().iter().flat_map(|q| {
            if q.running() {
                serde_json::to_string(&q.state()).ok()
            } else {
                let entries = q.get_list();
                serde_json::to_string(&QueueMessage::Ls{id:q.id().to_string(),entries}).ok()
            }
        }).collect::<Vec<_>>();
        //println!("HERE: {msg}");
        for msg in msgs {
            match socket.send(axum::extract::ws::Message::Text(msg)).await {
                Err(e) => tracing::info!("Error sending message: {e}"),
                _ => ()
            }
        }
        /*
        for q in controller().build_queue().queues().iter() {
            if q.running() {
                todo!()
            } else {
                let entries = q.get_list();
                let msg = serde_json::to_string(&QueueMessage::Ls{id:q.id().to_string(),entries}).unwrap();
                //println!("HERE: {msg}");
                match socket.send(axum::extract::ws::Message::Text(msg)).await {
                    Err(e) => tracing::info!("Error sending message: {e}"),
                    _ => ()
                }
            }
        }*/
    }
    #[cfg(feature="client")]
    fn socket(&mut self) -> &mut leptos::web_sys::WebSocket {&mut self.socket }
}

#[component]
pub fn QueuesTop() -> impl IntoView {
    view!{<Await future = || get_queues() let:queues blocking=true>{
        match queues {
            Ok(q) if q.is_empty() => view!(<div>"No running queues"</div>).into_view(),
            Ok(q) => view!(<QueueTabs queues=q.clone()/>).into_view(),
            _ => view!(<div>"Error"</div>).into_view()
        }
    }</Await> }
}

#[island]
fn QueueTabs(queues:Vec<String>) -> impl IntoView {
    use thaw::*;
    use wasm_bindgen::JsCast;
    let value = create_rw_signal(queues.first().unwrap().clone());
    let queues = queues.into_iter().map(|id| (id,create_node_ref::<html::Div>())).collect::<Vec<_>>();
    let mut divs = HashMap::new();
    for (k,v) in queues.iter() {
        divs.insert(k.clone(), v.clone());
    }
    QueueSocket::force_start(move |msg| {
        //console_log!("Here: {msg:?}");
        match msg {
            QueueMessage::Ls {id,entries} => {
                if let Some(div) = divs.get(&id) {
                    if let Some(div) = div.get_untracked() {
                        div.replace_children_with_node_1(view!(<div><IdleQueue id=id.clone() ls=entries/></div>).dyn_ref().unwrap());
                        /*for e in entries {
                            ul.append_child(view!(<li><b>{e.target.to_string()}</b>{format!(" [{}]/{}",e.archive,e.rel_path)}</li>).dyn_ref().unwrap());
                        }*/
                    }
                }
            }
            QueueMessage::Start {id, queue,blocked,failed,done } => {
                if let Some(div) = divs.get(&id) {
                    if let Some(div) = div.get_untracked() {
                        div.replace_children_with_node_1(view!(<div><RunningQueue id=id.clone() queue=queue.clone() blocked=blocked.clone() failed=failed.clone() done=done.clone()/></div>).dyn_ref().unwrap());
                    }
                }
            }
        }
        None
    });
    view!(<Tabs value>{
        queues.into_iter().map(|(q,ls)| view!(<Tab key=&q label=q><div node_ref=ls>
        </div></Tab>)).collect::<Vec<_>>()
    }</Tabs>)
}

#[island]
pub fn IdleQueue(id:String, ls:Vec<QueueEntry>) -> impl IntoView {
    use thaw::*;
    let act = create_action(move |_| run(id.clone()));
    view!{
        <Space justify=SpaceJustify::End>{move ||
            view!(<Button on_click=move |_| act.dispatch(())>"Run"</Button>)
        }</Space>
        <ul>{let ls = ls.clone(); view!(<For each=move ||ls.clone() key=|e| e.id() children=move|e| view!(
            <li><b>{e.target.to_string()}</b>{format!(" [{}]/{}",e.archive,e.rel_path)}</li>
        )/>)}</ul>
    }
}

#[island]
pub fn RunningQueue(id:String, queue:Vec<QueueEntry>,blocked:Vec<QueueEntry>,failed:Vec<QueueEntry>,done:Vec<QueueEntry>) -> impl IntoView {
    use thaw::*;
    view!{<Layout has_sider=true>
        <LayoutSider style="background-color: #0078ff99; padding: 20px;">
            <div style="position:sticky"><Anchor>
                <AnchorLink title="Queue" href="#queued"/>
                <AnchorLink title="Blocked" href="#blocked"/>
                <AnchorLink title="Failed" href="#failed"/>
                <AnchorLink title="Finished" href="#done"/>
            </Anchor></div>
        </LayoutSider>
        <Layout>
        <h3 id="queued">Queued</h3>
        <ul>{let queue = queue.clone(); view!(<For each=move ||queue.clone() key=|e| e.id() children=move|e| view!(
            <li><b>{e.target.to_string()}</b>{format!(" [{}]/{}",e.archive,e.rel_path)}</li>
        )/>)}</ul>
        <h3 id="blocked">Blocked</h3>
        <ul>{let queue = blocked.clone(); view!(<For each=move ||queue.clone() key=|e| e.id() children=move|e| view!(
            <li><b>{e.target.to_string()}</b>{format!(" [{}]/{}",e.archive,e.rel_path)}</li>
        )/>)}</ul>
        <h3 id="failed">Failed</h3>
        <ul>{let queue = failed.clone(); view!(<For each=move ||queue.clone() key=|e| e.id() children=move|e| view!(
            <li><b>{e.target.to_string()}</b>{format!(" [{}]/{}",e.archive,e.rel_path)}</li>
        )/>)}</ul>
        <h3 id="done">Done</h3>
        <ul>{let queue = done.clone(); view!(<For each=move ||queue.clone() key=|e| e.id() children=move|e| view!(
            <li><b>{e.target.to_string()}</b>{format!(" [{}]/{}",e.archive,e.rel_path)}</li>
        )/>)}</ul>
        </Layout>
    </Layout>}
}