use immt_utils::{logs::{LogFileLine, LogLevel, LogMessage, LogTree}, time::Timestamp, vecmap::VecMap};
use immt_web_utils::{inject_css,components::{Tree,Leaf,LazySubtree,Header}};
use leptos::{either::Either, prelude::*};
use thaw::Caption1Strong;
use immt_web_utils::components::Spinner;

use crate::utils::{needs_login, ws::WebSocket};

#[cfg(feature="ssr")]
async fn full_log() -> Result<immt_utils::logs::LogTree,()> {
    use tokio::io::AsyncBufReadExt;

    let path = immt_system::logger().log_file();

    let reader = tokio::io::BufReader::new(tokio::fs::File::open(path).await.map_err(|_| ())?);
    let mut lines = reader.lines();
    let mut parsed = Vec::new();
    while let Ok(Some(line)) = lines.next_line().await {
        if !line.is_empty() {
            if let Some(line) = LogFileLine::parse(&line) {
                parsed.push(line.to_owned());
            }
        }
    }
    let tree : LogTree = parsed.into();
    Ok(tree)
}

#[component]
pub fn Logger() -> impl IntoView {
    needs_login(|| {
        inject_css("immt-logging", include_str!("logs.css"));
        let signals = LogSignals {
            top: RwSignal::new(Vec::new()),
            open_span_paths: RwSignal::new(VecMap::new()),
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
            <div class="immt-log-frame">{ move || {
                if signals.top.with(Vec::is_empty) {
                    Either::Left(view!(<div class="immt-spinner-frame"><Spinner/></div>))
                } else {Either::Right(view!{<Tree>
                    {do_ls(signals.top)}
                </Tree>})}
            }}</div>
            <div class="immt-warn-frame">
            <Caption1Strong><span style="color:var(--colorPaletteRedForeground1)">"Warnings"</span></Caption1Strong>{ move || {
                if signals.top.get().is_empty() {
                    Either::Left(view!(<div class="immt-spinner-frame"><Spinner/></div>))
                } else {Either::Right(view!{<Tree>
                    <For each=move || signals.warnings.get() key=|e| e.0.clone() children=move |e| view!(
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
                LogEntrySignal::Simple(_,e) => view!(<Leaf><span class="immt-log-elem"><LogLine e/></span></Leaf>).into_any(),
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
    use immt_web_utils::components::SpinnerSize;
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
            <span class="immt-spinner-inline">
            <Spinner size=SpinnerSize::Tiny/>
            </span>{str}
        </span>))
    } else {Either::Right(view!(<span class=cls>{str}</span>))}
}

const fn class_from_level(lvl:LogLevel) -> &'static str {
    match lvl {
        LogLevel::ERROR => "immt-log-error",
        LogLevel::WARN => "immt-log-warn",
        LogLevel::INFO => "immt-log-info",
        LogLevel::DEBUG => "immt-log-debug",
        LogLevel::TRACE => "immt-log-trace",
    }
}
pub struct LogSocket {
    #[cfg(feature="ssr")]
    listener: immt_utils::change_listener::ChangeListener<LogFileLine<String>>,
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
                let listener = immt_system::logger().listener();
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

    fn populate(signals:LogSignals,tree:LogTree) {
        use immt_utils::logs::{LogTreeElem,LogSpan};

        fn add(signal:&mut Vec<LogEntrySignal>,warnings:RwSignal<Vec<(String,LogMessage)>>,e:LogTreeElem) {
            let id = e.id();
            match e {
                LogTreeElem::Message(LogMessage {message,timestamp,target,level,args}) => {
                    if level >= LogLevel::WARN {
                        warnings.try_update(|v| v.push(
                            (id.clone(),LogMessage {message:message.clone(),timestamp,target:target.clone(),level,args:args.clone()})
                        ));
                    }
                    signal.push(LogEntrySignal::Simple(id,LogMessage {message,timestamp,target,level,args}));
                }
                LogTreeElem::Span(LogSpan {name,timestamp,target,level,args,children,closed}) => {
                    let message = RwSignal::new(if let Some(closed) = closed {
                        SpanMessage::Closed(format!("{} (finished after {})",name,closed.since(timestamp)))
                    } else {SpanMessage::Open{name,timestamp}});
                    let mut nchildren = Vec::new();
                    for c in children {
                        add(&mut nchildren,warnings,c);
                    }
                    let e = SpanSignal {
                        message,
                        target,level,args,children:RwSignal::new(nchildren)
                    };
                    signal.push(LogEntrySignal::Span(id,e));
                }
            }
        }

        signals.open_span_paths.try_update_untracked(|v| *v = tree.open_span_paths);
        signals.top.try_update(|v|
            for e in tree.children {
                add(v,signals.warnings,e);
            }
        );
    }
    fn update(signals:LogSignals,update:LogFileLine<String>) {
        match update {
            LogFileLine::Message {message,timestamp,target,level,args,span} => {
                let id = LogFileLine::id_from(&message,&args);
                if level >= LogLevel::WARN {
                    signals.warnings.try_update(|v| v.push((id.clone(), LogMessage { message: message.clone(), timestamp, target: target.clone(), level, args: args.clone() })));
                }
                signals.open_span_paths.try_update_untracked(move |spans| {
                    let mut curr = signals.top;
                    if let Some(v) = span.and_then(|id| spans.get(&id)) {
                        signals.top.try_with_untracked(|nv| 
                            for i in v {
                                if let Some(LogEntrySignal::Span(_,s)) = nv.get(*i) {
                                    curr = s.children;
                                } else {break}
                            }
                        );
                    }
                    curr.try_update(|v| v.push(LogEntrySignal::Simple(id, LogMessage { message, timestamp, target, level, args })));
                });
            }
            LogFileLine::SpanOpen {name,timestamp,target,level,args,parent} => {
                signals.open_span_paths.try_update_untracked(move |spans| {
                    let id = LogFileLine::id_from(&name,&args);
                    let mut curr = signals.top;
                    if let Some(v) = parent.and_then(|id| spans.get(&id)) {
                        signals.top.try_with_untracked(|nv|
                            for i in v {
                                if let Some(LogEntrySignal::Span(_, s)) = nv.get(*i) {
                                    curr = s.children;
                                } else { break }
                            }
                        );
                    }
                    curr.try_update(|parent| {
                        parent.push(LogEntrySignal::Span(id, SpanSignal {
                            message: RwSignal::new(SpanMessage::Open { name, timestamp }),
                            target, level, args, children: RwSignal::new(Vec::new())
                        }));
                    });
                });
            }
            LogFileLine::SpanClose {id,timestamp,..} => {
                signals.open_span_paths.try_update_untracked(move |spans| {
                    if let Some(path) = spans.remove(&id) {
                        fn get(mut iter:std::vec::IntoIter<usize>,ret:&mut Option<(String,RwSignal<SpanMessage>)>,curr:RwSignal<Vec<LogEntrySignal>>) {
                            if let Some(i) = iter.next() {
                                curr.try_with_untracked(|v|
                                    if let Some(LogEntrySignal::Span(id,s)) = v.get(i) {
                                        *ret = Some((id.clone(),s.message));
                                        get(iter,ret,s.children);
                                    } else { *ret = None }
                                );
                            }
                        }
                        let mut ret = None;
                        get(path.into_iter(),&mut ret,signals.top);
                        if let Some((oid,message)) = ret {
                            if oid == id {
                                if let Some(SpanMessage::Open { name, timestamp: old }) = message.try_get_untracked() {
                                    message.try_set(SpanMessage::Closed(format!("{} (finished after {})", name, timestamp.since(old))));
                                }
                            }
                        }
                    }
                });
            }
        }
    }
}


#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub enum Log {
    Initial(LogTree),
    Update(LogFileLine<String>),
}

#[derive(Debug,Copy,Clone,serde::Serialize,serde::Deserialize)]
struct LogSignals {
    top:RwSignal<Vec<LogEntrySignal>>,
    open_span_paths:RwSignal<VecMap<String,Vec<usize>>>,
    warnings:RwSignal<Vec<(String,LogMessage)>>
}
#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
enum LogEntrySignal {
    Simple(String,LogMessage),
    Span(String,SpanSignal)
}
impl LogEntrySignal {
    fn id(&self) -> &str {
        match self {
            Self::Simple(id,_) | Self::Span(id,_) => id
        }
    }
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