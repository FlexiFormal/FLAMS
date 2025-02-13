use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;

use crate::hashstr;
use crate::parsing::{ParseSource, ParseStr};
use crate::time::Timestamp;
use crate::vecmap::VecMap;

#[derive(Debug,Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize,serde::Deserialize))]
pub enum LogTreeElem {
    Span(LogSpan),
    Message(LogMessage)
}
impl LogTreeElem {
    #[must_use]
    pub fn id(&self) -> String {
        match self {
            Self::Span(LogSpan { name,args,.. }) =>
                LogFileLine::id_from(name,args),
            Self::Message(LogMessage { message,args,.. }) =>
                LogFileLine::id_from(message,args)
        }
    }
    #[must_use]
    pub const fn timestamp(&self) -> Timestamp {
        match self {
            Self::Span(LogSpan {timestamp,..}) |
            Self::Message(LogMessage {timestamp,..}) => *timestamp
        }
    }
    #[must_use]
    pub const fn level(&self) -> LogLevel {
        match self {
            Self::Span(LogSpan {level,..}) |
            Self::Message(LogMessage {level,..}) => *level
        }
    }
    #[must_use]
    pub fn target(&self) -> Option<&str> {
        match self {
            Self::Span(LogSpan {target,..}) |
            Self::Message(LogMessage {target,..}) => target.as_deref()
        }
    }
    #[must_use]
    pub const fn args(&self) -> &VecMap<String, String> {
        match self {
            Self::Span(LogSpan {args,..}) |
            Self::Message(LogMessage {args,..}) => args
        }
    }
}

#[derive(Debug,Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize,serde::Deserialize))]
pub struct LogMessage {
    pub message:String,
    pub timestamp:Timestamp,
    pub target:Option<String>,
    pub level:LogLevel,
    pub args:VecMap<String, String>,
}

#[derive(Debug,Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize,serde::Deserialize))]
pub struct LogSpan {
    pub name:String,
    pub timestamp:Timestamp,
    pub target:Option<String>,
    pub level:LogLevel,
    pub args:VecMap<String, String>,
    pub children:Vec<LogTreeElem>,
    pub closed:Option<Timestamp>
}

#[derive(Debug,Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize,serde::Deserialize))]
pub struct LogTree {
    pub children:Vec<LogTreeElem>,
    pub open_span_paths:VecMap<String,Vec<usize>>
}
impl LogTree {
    fn merge(&mut self,line:LogTreeElem, parent:Option<String>) {
        let mut path = Vec::new();
        let p = if let Some(p) = parent.as_ref().and_then(|p| self.open_span_paths.get(p)) {
            path.clone_from(p);
            let mut ls = &mut self.children;
            for i in p {
                ls = match &mut ls[*i] {
                    LogTreeElem::Span(LogSpan {children,..}) => children,
                    LogTreeElem::Message { .. } => unreachable!()
                };
            }
            ls
        } else {
           /*  if parent.is_some() {
                println!("Parent not found: {line:?}!");
            }*/
            &mut self.children
        };
        if let LogTreeElem::Span(LogSpan { .. }) = &line {
            path.push(p.len());
            self.open_span_paths.insert(line.id(),path);
        }
        p.push(line);
    }
    fn close(&mut self,id:&str,timestamp:Timestamp) {
        let e = if let Some(mut path) = self.open_span_paths.remove(id) {
            let mut ls = &mut self.children;
            let last = path.pop().unwrap_or_else(|| unreachable!());
            for i in path {
                ls = match &mut ls[i] {
                    LogTreeElem::Span(LogSpan {children,..}) => children,
                    LogTreeElem::Message { .. } => unreachable!()
                };
            }
            &mut ls[last]
        } else { return };
        if let LogTreeElem::Span(e) = e { e.closed = Some(timestamp); }
    }
    fn add_line<S:ToString+PartialEq<String>>(&mut self,line:LogFileLine<S>) {
        match line {
            LogFileLine::SpanOpen { name, timestamp, target, level, args, parent } => {
                let span = LogSpan {
                    name: name.to_string(),
                    timestamp,
                    target: target.map(|s| s.to_string()),
                    level,
                    args: args.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
                    children: Vec::new(),
                    closed: None
                };
                self.merge(LogTreeElem::Span(span), parent);
            }
            LogFileLine::Message { message, timestamp, target, level, args, span } => {
                let message = LogMessage {
                    message: message.to_string(),
                    timestamp,
                    target: target.map(|s| s.to_string()),
                    level,
                    args: args.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
                };
                self.merge(LogTreeElem::Message(message), span);
            }
            LogFileLine::SpanClose { timestamp,id, .. } => {
                self.close(&id,timestamp);
            }
        }
    }
}
impl<S:ToString+PartialEq<String>,I:IntoIterator<Item = LogFileLine<S>>> From<I> for LogTree {
    fn from(iter: I) -> Self {
        let mut s = Self{children:Vec::new(),open_span_paths:VecMap::default()};
        for e in iter {
            s.add_line(e);
        }
        s
    }
}

#[derive(Clone,Debug)]
#[cfg_attr(feature="serde", derive(serde::Serialize,serde::Deserialize))]
pub enum LogFileLine<S> {
    SpanOpen {
        name:S,
        timestamp:Timestamp,
        target:Option<S>,
        level:LogLevel,
        args:VecMap<S, S>,
        parent:Option<String>
    },
    SpanClose {
        id:String,
        timestamp:Timestamp,
        parent:Option<String>
    },
    Message {
        message:S,
        timestamp:Timestamp,
        target:Option<S>,
        level:LogLevel,
        args:VecMap<S, S>,
        span: Option<String>
    },
}
impl<S:AsRef<str> + std::fmt::Debug + Hash> LogFileLine<S> {

    #[must_use]
    pub fn id_from(msg:&str,args:&VecMap<S,S>) -> String {
        hashstr("", &(msg,args))
    }
    #[must_use]
    pub fn id(&self) -> String {
        match self {
            Self::SpanOpen { name, args, .. } => {
                Self::id_from(name.as_ref(),args)
            }
            Self::SpanClose { id, .. } => id.clone(),
            Self::Message { message, args, .. } => {
                Self::id_from(message.as_ref(),args)
            }
        }
    }
}

impl<'a> LogFileLine<&'a str> {
    fn read_string(parser:&mut ParseStr<'a,()>) -> Option<&'a str> {
        if !parser.drop_prefix("\"") {return None}
        let s = parser.read_until_escaped('\"','\\');
        parser.pop_head();
        Some(s)
    }
    fn read_elem(parser:&mut ParseStr<'a,()>) -> Option<&'a str> {
        if parser.peek_head().is_some_and(|c| c == '\"') {
            Self::read_string(parser)
        } else if parser.peek_head().is_some_and(|c| c.is_ascii_digit()) {
            Some(parser.read_while(|c| c.is_ascii_digit() || c == '.'))
        } else {
            None
        }
    }
    fn read_span(parser:&mut ParseStr<'a,()>) -> Option<(&'a str,VecMap<&'a str,&'a str>)> {
        let mut name = "";
        if !parser.drop_prefix("{") {return None}
        let mut vec = VecMap::default();
        while parser.drop_prefix("\"") {
            let key = parser.read_until(|c| c == '"');
            if !parser.drop_prefix("\":") {return None}
            if key == "name" {
                name = Self::read_string(parser)?;
            } else {
                vec.insert(key,Self::read_elem(parser)?);
            }
            parser.drop_prefix(",");
        }
        if !parser.drop_prefix("}") {return None}
        Some((name,vec))
    }

    #[must_use]
    pub fn parse(s: &'a str) -> Option<Self> {
        let mut parser = ParseStr::<()>::new(s);
        if !parser.drop_prefix("{\"timestamp\":") {return None}
        let ts = Self::read_string(&mut parser)?;
        let timestamp =ts.parse().ok()?;
        if !parser.drop_prefix(",\"level\":") {return None}
        let level = Self::read_string(&mut parser)?;
        let level : LogLevel = level.parse().ok()?;
        if !parser.drop_prefix(",\"message\":") {return None}
        let message = Self::read_string(&mut parser)?;
        let mut target = None;
        let mut span = None;
        let mut spans = Vec::new();
        let mut args = VecMap::default();
        while parser.drop_prefix(",\"") {
            let key = parser.read_until(|c| c == '"');
            if !parser.drop_prefix("\":") {return None}
            match key {
                "target" => {
                    target = Some(Self::read_string(&mut parser)?);
                }
                "span" => span = Some(Self::read_span(&mut parser)?),
                "spans" => {
                    if !parser.drop_prefix("[") {return None}
                    if parser.peek_head() == Some('{') {
                        spans.push(Self::read_span(&mut parser)?);
                        while parser.drop_prefix(",") {
                            spans.push(Self::read_span(&mut parser)?);
                        }
                    }
                    if !parser.drop_prefix("]") {return None}
                }
                "time.busy" | "time.idle" => {
                    Self::read_string(&mut parser)?;
                }
                k => {
                    args.insert(k,Self::read_elem(&mut parser)?);
                }
            }
        }
        if !parser.drop_prefix("}") {return None}
        if message == "new" {
            let (name,args) = span?;
            Some(LogFileLine::SpanOpen {
                name,
                timestamp,
                target,
                level,
                args,
                parent:spans.into_iter().map(|(name,args)| hashstr("",&(name,args))).next_back()
            })
        } else if message == "close" {
            let (name,args) = span?;
            let id = LogFileLine::id_from(name,&args);
            Some(LogFileLine::SpanClose {
                id,
                timestamp,
                parent:spans.into_iter().map(|(name,args)| hashstr("",&(name,args))).next_back()
            })
        } else {
            Some(LogFileLine::Message {
                message,
                timestamp,
                target,
                level,
                args,
                span:spans.into_iter().map(|(name,args)| hashstr("",&(name,args))).next_back()
            })
        }
    }

    #[must_use]
    pub fn to_owned(self) -> LogFileLine<String> {
        match self {
            LogFileLine::SpanOpen { name, timestamp, target, level, args, parent } => LogFileLine::SpanOpen {
                name: name.into(),
                timestamp,
                target: target.map(Into::into),
                level,
                args: args.into_iter().map(|(k,v)| (k.into(),v.into())).collect(),
                parent
            },
            LogFileLine::SpanClose { id, timestamp, parent } => LogFileLine::SpanClose {
                id, timestamp,
                parent
            },
            LogFileLine::Message { message, timestamp, target, level, args, span } => LogFileLine::Message {
                message: message.into(),
                timestamp,
                target: target.map(Into::into),
                level,
                args: args.into_iter().map(|(k,v)| (k.into(),v.into())).collect(),
                span
            }
        }
    }
}

#[derive(Debug,Copy,Clone,PartialEq,Eq)]
#[cfg_attr(feature="serde", derive(serde::Serialize,serde::Deserialize))]
pub enum LogLevel { TRACE, DEBUG, INFO, WARN, ERROR }
impl FromStr for LogLevel {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TRACE" => Ok(Self::TRACE),
            "DEBUG" => Ok(Self::DEBUG),
            "INFO" => Ok(Self::INFO),
            "WARN" => Ok(Self::WARN),
            "ERROR" => Ok(Self::ERROR),
            _ => Err(())
        }
    }
}
impl From<tracing::Level> for LogLevel {
    fn from(l:tracing::Level) -> Self {
        match l {
            tracing::Level::TRACE => Self::TRACE,
            tracing::Level::DEBUG => Self::DEBUG,
            tracing::Level::INFO => Self::INFO,
            tracing::Level::WARN => Self::WARN,
            tracing::Level::ERROR => Self::ERROR
        }
    }
}
impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for LogLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (a,b) if a == b => std::cmp::Ordering::Equal,
            (Self::TRACE, _) => std::cmp::Ordering::Less,
            (_, Self::TRACE) => std::cmp::Ordering::Greater,
            (Self::DEBUG, _) => std::cmp::Ordering::Less,
            (_, Self::DEBUG) => std::cmp::Ordering::Greater,
            (Self::INFO, _) => std::cmp::Ordering::Less,
            (_, Self::INFO) => std::cmp::Ordering::Greater,
            (Self::WARN, _) => std::cmp::Ordering::Less,
            (_, Self::WARN) => std::cmp::Ordering::Greater,
            _ => unreachable!()
        }
    }
}
impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TRACE => write!(f,"TRACE"),
            Self::DEBUG => write!(f,"DEBUG"),
            Self::INFO =>  write!(f,"INFO "),
            Self::WARN =>  write!(f,"WARN "),
            Self::ERROR => write!(f,"ERROR")
        }
    }
}