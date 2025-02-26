use flams_utils::{logs::{LogFileLine, LogLevel, LogMessage, LogSpan, LogTree, LogTreeElem}, time::Timestamp, vecmap::VecMap};
use flams_web_utils::{inject_css,components::{Tree,Leaf,LazySubtree,Header}};
use leptos::{either::Either, prelude::*};
use thaw::Caption1Strong;
use flams_web_utils::components::Spinner;
use std::num::NonZeroU64;

use crate::utils::{needs_login, ws::WebSocket};

#[cfg(feature="ssr")]
async fn full_log() -> Result<flams_utils::logs::LogTree,()> {
    use tokio::io::AsyncBufReadExt;

    let path = flams_system::logging::logger().log_file();

    let reader = tokio::io::BufReader::new(tokio::fs::File::open(path).await.map_err(|_| ())?);
    let mut lines = reader.lines();
    let mut tree = flams_utils::logs::LogTree::default();
    while let Ok(Some(line)) = lines.next_line().await {
        if !line.is_empty() {
            if let Ok(line) = serde_json::from_str(&line) {
                tree.add_line(line);
            }
        }
    }
    
    Ok(tree)
}

#[component]
pub fn Logger() -> impl IntoView {
    needs_login(|| {
        inject_css("flams-logging", include_str!("logs.css"));
        let signals = LogSignals {
            top: RwSignal::new(Vec::new()),
            open_span_paths: RwSignal::new(VecMap::default()),
            warnings: RwSignal::new(Vec::new())
        };
        Effect::new(move |_| {
            #[cfg(feature="hydrate")]
            {
                use crate::utils::ws::WebSocketClient;
                let _ = LogSocket::start(move |msg| {
                    LogSocket::ws(signals, msg);
                    None
                });
            }
        });
        view!{
            <div class="flams-log-frame">{ move || {
                if signals.top.with(Vec::is_empty) {
                    Either::Left(view!(<div class="flams-spinner-frame"><Spinner/></div>))
                } else {Either::Right(view!{<Tree>
                    {do_ls(signals.top)}
                </Tree>})}
            }}</div>
            <div class="flams-warn-frame">
            <Caption1Strong><span style="color:var(--colorPaletteRedForeground1)">"Warnings"</span></Caption1Strong>{ move || {
                if signals.top.with(Vec::is_empty) {
                    Either::Left(view!(<div class="flams-spinner-frame"><Spinner/></div>))
                } else {Either::Right(view!{<Tree>
                    <For each=move || signals.warnings.get() key=|e| e.0 children=move |e| view!(
                        <Leaf><LogLine e=e.1/></Leaf>
                    )/>
                </Tree>})}
            }}</div>
        }
    })
}

fn do_ls(v:RwSignal<Vec<LogEntrySignal>>) -> impl IntoView {
    view!{
        <For each=move || v.get() key=|e| e.id().to_string() children=|e| {
            match e {
                LogEntrySignal::Simple(_,e) => view!(<Leaf><span class="flams-log-elem"><LogLine e/></span></Leaf>).into_any(),
                LogEntrySignal::Span(_,e) => do_span(e).into_any()
            }
        }/>
    }
}
fn do_span(s:SpanSignal) -> impl IntoView {
    let children = s.children;
    view!{<LazySubtree>
        <Header slot>{move || {let s = s.clone(); match s.message.get() {
            SpanMessage::Open {name,timestamp} => view!(<LogLineHelper message=name timestamp target=s.target level=s.level args=s.args spinner=true/>),
            SpanMessage::Closed(message) => view!(<LogLineHelper message target=s.target level=s.level args=s.args spinner=false />)
        }}}</Header>
        {move || do_ls(children)}
    </LazySubtree>}
}


#[component]
fn LogLine(e:LogMessage) -> impl IntoView {
    let LogMessage {message,timestamp,target,level,args} = e;
    view!(<LogLineHelper message timestamp target level args/>)
}

#[component]
fn LogLineHelper(
    message:String,
    #[prop(optional)] timestamp:Option<Timestamp>,
    target:Option<String>,
    level:LogLevel,
    args:VecMap<String,String>,
    #[prop(optional)] spinner:bool,
) -> impl IntoView {
    use flams_web_utils::components::SpinnerSize;
    use std::fmt::Write;
    let cls = class_from_level(level);
    let mut str = timestamp.map_or_else(
        || format!("<{level}> "),
        |timestamp| format!("{timestamp} <{level}> ")
    );
    if let Some(target) = target {
        write!(str,"[{target}] ").unwrap();
    }
    str.push_str(&message);
    if !args.is_empty() {
        str.push_str(" (");
        for (k,v) in args {
            write!(str,"{k}:{v} ").unwrap();
        }
        str.push(')');
    }
    if spinner {
        Either::Left(view!(<span class=cls>
            <span class="flams-spinner-inline">
            <Spinner size=SpinnerSize::Tiny/>
            </span>{str}
        </span>))
    } else {Either::Right(view!(<span class=cls>{str}</span>))}
}

const fn class_from_level(lvl:LogLevel) -> &'static str {
    match lvl {
        LogLevel::ERROR => "flams-log-error",
        LogLevel::WARN => "flams-log-warn",
        LogLevel::INFO => "flams-log-info",
        LogLevel::DEBUG => "flams-log-debug",
        LogLevel::TRACE => "flams-log-trace",
    }
}
pub struct LogSocket {
    #[cfg(feature="ssr")]
    listener: flams_utils::change_listener::ChangeListener<LogFileLine>,
    #[cfg(all(feature="hydrate",not(doc)))]
    socket: leptos::web_sys::WebSocket,
    #[cfg(all(feature="hydrate",doc))]
    socket: ()
}

//#[async_trait]
impl WebSocket<(),Log> for LogSocket {
    const SERVER_ENDPOINT: &'static str = "/ws/log";
}

#[cfg(feature="ssr")]
#[async_trait::async_trait]
impl crate::utils::ws::WebSocketServer<(),Log> for LogSocket {
    async fn new(account:crate::users::LoginState,_db:crate::server::db::DBBackend) -> Option<Self> {
        use crate::users::LoginState;
        match account {
            LoginState::Admin | LoginState::NoAccounts | LoginState::User{is_admin:true,..} => {
                let listener = flams_system::logging::logger().listener();
                Some(Self {
                    listener,
                    #[cfg(feature="hydrate")] socket:unreachable!()
                })
            }
            _ => None
        }
    }
    async fn next(&mut self) -> Option<Log> {
        self.listener.read().await.map(Log::Update)
    }
    async fn handle_message(&mut self,_msg:()) -> Option<Log> {None}
    async fn on_start(&mut self,socket:&mut axum::extract::ws::WebSocket) {
        if let Ok(init) = full_log().await {
            let _ = socket.send(axum::extract::ws::Message::Text({
                let Ok(s) = serde_json::to_string(&Log::Initial(init)) else {return};
                s
            })).await;
        }
    }
}

#[cfg(feature="hydrate")]
impl crate::utils::ws::WebSocketClient<(),Log> for LogSocket {
    fn new(ws: leptos::web_sys::WebSocket) -> Self { Self{
        #[cfg(not(doc))]
        socket:ws,
        #[cfg(doc)]
        socket:(),
        #[cfg(feature="ssr")] listener:unreachable!()
    } }
    fn socket(&mut self) -> &mut leptos::web_sys::WebSocket {&mut self.socket }
}

#[cfg(feature="hydrate")]
impl LogSocket {
    fn ws(signals:LogSignals,l:Log) {
        match l {
            Log::Initial(tree) => Self::populate(signals,tree),
            Log::Update(up) => Self::update(signals,up)
        }
    }

    fn convert(e:LogTreeElem,warnings:&mut Vec<(NonZeroU64,LogMessage)>) -> LogEntrySignal {
        match e {
            LogTreeElem::Message(e) => {
                let id = next_id();
                if e.level >= LogLevel::WARN {
                    warnings.push((id,e.clone()));
                }
                LogEntrySignal::Simple(next_id(),e)
            },
            LogTreeElem::Span(e@LogSpan{ closed:None,..}) => LogEntrySignal::Span(next_id(),
                SpanSignal {
                    message:RwSignal::new(SpanMessage::Open {name:e.name,timestamp:e.timestamp}),
                    target:e.target,level:e.level,args:e.args,children:RwSignal::new(
                        e.children.into_iter().map(|e| Self::convert(e,warnings)).collect()
                    )
                }
            ),
            LogTreeElem::Span(e) => {
                let closed = e.closed.unwrap_or_else(|| unreachable!());
                LogEntrySignal::Span(next_id(),
                    SpanSignal {
                        message:RwSignal::new(
                            SpanMessage::Closed(format!("{} (finished after {})",e.name,closed.since(e.timestamp)))
                        ),
                        target:e.target,level:e.level,args:e.args,children:RwSignal::new(
                            e.children.into_iter().map(|e| Self::convert(e,warnings)).collect()
                        )
                    }
                )
            }
        }
    }

    fn populate(signals:LogSignals,tree:LogTree) {
        use flams_utils::logs::{LogTreeElem,LogSpan};
        signals.open_span_paths.try_update_untracked(|v| *v = tree.open_span_paths);
        signals.warnings.try_update(|ws| 
            signals.top.try_update(|v|
                *v = tree.children.into_iter().map(|e| Self::convert(e,ws)).collect()
            )
        );
    }
    fn update(signals:LogSignals,update:LogFileLine) {
        let (msg,parent,is_span) = match update {
            LogFileLine::Message { message, timestamp, target, level, args, span } => {
                let id = next_id();
                let message = LogMessage {
                    message, timestamp, target, level,args
                };
                if level >= LogLevel::WARN {
                    signals.warnings.try_update(|v| v.push((id.clone(), message.clone())));
                }
                (LogEntrySignal::Simple(id, message),span,None)
            }
            LogFileLine::SpanOpen { name, id, timestamp, target, level, args, parent } => {
                let span = SpanSignal {
                    message:RwSignal::new(SpanMessage::Open {name,timestamp}),
                    target,level,args,
                    children: RwSignal::new(Vec::new())
                };
                (LogEntrySignal::Span(next_id(), span),parent,Some(id))
            }
            LogFileLine::SpanClose { timestamp,id, .. } => {
                signals.close(id,timestamp);
                return
            }
        };
        signals.merge(msg,parent,is_span);
    }
}


#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub enum Log {
    Initial(LogTree),
    Update(LogFileLine),
}

#[derive(Debug,Copy,Clone,serde::Serialize,serde::Deserialize)]
struct LogSignals {
    top:RwSignal<Vec<LogEntrySignal>>,
    open_span_paths:RwSignal<VecMap<NonZeroU64,Vec<usize>>>,
    warnings:RwSignal<Vec<(NonZeroU64,LogMessage)>>
}
impl LogSignals {
    fn close(&self,id:NonZeroU64,timestamp:Timestamp) {
        if let Some(path) = self.open_span_paths.try_update_untracked(|v| v.remove(&id)).flatten() {
            Self::close_i(self.top,&path,timestamp);
        };
    }
    fn close_i(sigs:RwSignal<Vec<LogEntrySignal>>,path:&[usize],timestamp:Timestamp) {
        if path.is_empty() { return }
        let i = path[0];
        let path = &path[1..];
        
        sigs.try_with_untracked(|v| if let Some(LogEntrySignal::Span(_,s)) = v.get(i) {
            if path.is_empty() {
                s.message.try_update(|m|
                    if let SpanMessage::Open { name, timestamp: old } = &m {
                        let msg = format!("{} (finished after {})", name, timestamp.since(*old));
                        *m = SpanMessage::Closed(msg);
                    }
                );
            } else {
                Self::close_i(s.children,path,timestamp);
            }
        });
    }

    fn merge(&self,line:LogEntrySignal, parent:Option<NonZeroU64>,is_span:Option<NonZeroU64>) {
        match parent {
            None => {
                if let Some(id) = is_span {
                    self.top.try_update(|ch| {
                        self.open_span_paths.try_update_untracked(|v| v.insert(id,vec![ch.len()]));
                        ch.push(line);
                    });
                } else {
                    self.top.try_update(|ch| ch.push(line));
                }
            }
            Some(parent) => {
                let mut path = Vec::new();
                self.open_span_paths.try_update_untracked(|v| {
                    if let Some(p) = v.get(&parent) {
                        path.clone_from(p);
                    } /*else {
                        leptos::logging::log!("Parent not found: {line:?}!");
                    }*/
                });
                self.merge_i(self.top,0,path,is_span,line);
            }
        }
    }
    fn merge_i(&self,sigs:RwSignal<Vec<LogEntrySignal>>,i:usize,mut path:Vec<usize>,is_span:Option<NonZeroU64>,line:LogEntrySignal) {
        if i == path.len() { return }
        let e = path[i];
        if path.len() == i + 1 {
            sigs.try_update(|v| {
                if let Some(s) = is_span {
                    path.push(v.len());
                    self.open_span_paths.try_update_untracked(|v| v.insert(s,path));
                }
                v.push(line);
            });
        } else {
            sigs.try_with_untracked(|v| {
                let LogEntrySignal::Span(_,s) = &v[e] else {unreachable!()};
                self.merge_i(s.children,i+1,path,is_span,line);
            });
        }
    }
}

#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
enum LogEntrySignal {
    Simple(NonZeroU64,LogMessage),
    Span(NonZeroU64,SpanSignal)
}
impl LogEntrySignal {
    #[inline]
    fn id(&self) -> NonZeroU64 { 
        match self {
            Self::Simple(id,_) |
            Self::Span(id,_) => *id
        }
     }
}

const COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
fn next_id() -> NonZeroU64 {
    NonZeroU64::new(COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)).unwrap_or_else(|| unreachable!())
}

#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
enum SpanMessage {
    Open{ name:String, timestamp:Timestamp },
    Closed(String)
}
#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
struct SpanSignal {
    pub message:RwSignal<SpanMessage>,
    pub target:Option<String>,
    pub level:LogLevel,
    pub args:VecMap<String, String>,
    pub children:RwSignal<Vec<LogEntrySignal>>
}