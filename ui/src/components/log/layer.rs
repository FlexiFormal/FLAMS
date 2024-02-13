use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Write};
use std::sync::Arc;
use chrono::Local;
use tracing::Id;
use tracing_subscriber::layer::Context;
use immt_api::utils::circular_buffer::CircularBuffer;
use immt_api::utils::HMap;

pub enum LogLine {
    Simple(SimpleLogLine),
    Span(SpanLine)
}

pub struct SimpleLogLine {
    pub level: tracing::Level,
    pub message:String,
    pub timestamp:chrono::DateTime<Local>,
    pub attrs:StringVisitor
}
pub struct SpanLine {
    pub level: tracing::Level,
    pub name:String,
    pub attrs:StringVisitor,
    pub timestamp:chrono::DateTime<Local>,
    pub children: Vec<LogLine>,
    id: Id,
    pub open:Option<OpenSpan>
}

pub struct OpenSpan {
    subspans:Vec<Id>,
    pub spinner:&'static[&'static str]
}

pub type LogStore = Arc<parking_lot::RwLock<(CircularBuffer<LogLine>,bool)>>;
pub struct Layer {
    store: LogStore
}
impl Layer {
    pub fn new() -> (Self,LogStore) {
        let store = Arc::new(parking_lot::RwLock::new((CircularBuffer::new(10000),false)));
        (Self { store: store.clone() }, store)
    }

    fn sort<S:FnOnce(&mut SpanLine),P:FnMut(&mut SpanLine)>(&self,pid:Id,mut p:P,s:S) {
        let mut store = self.store.write();
        store.1 = true;
        for e in store.0.iter_mut().rev(){ match e {
            LogLine::Span(span) if span.id == pid => {
                s(span);return
            },
            LogLine::Span(span) if span.open.as_ref().is_some_and(|s| s.subspans.contains(&pid)) => {
                p(span);
                let mut current = span.children.iter_mut().rev();
                loop {
                    let next = current.next();
                    match next {
                        Some(LogLine::Span(span)) if span.id == pid => {
                            s(span);return;
                        }
                        Some(LogLine::Span(span)) if span.open.as_ref().is_some_and(|s| s.subspans.contains(&pid)) => {
                            p(span);
                            current = span.children.iter_mut().rev();continue
                        }
                        Some(_) => continue,
                        None => return
                    }
                }
            },
            _ => ()
        }}

    }
    fn sort_into(&self,pid:Id,line:LogLine) {
        self.sort(pid,|_| (),|span| span.children.push(line))
    }
    fn sort_span_into(&self,pid:Id,tid:Id,line:LogLine) {
        self.sort(pid,
                  |span| span.open.as_mut().unwrap().subspans.push(tid.clone()),
                  |span| {
                        span.open.as_mut().unwrap().subspans.push(tid.clone());
                        span.children.push(line)
                    }
        )
    }
}
impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for Layer {
    fn on_event(&self, event: &tracing::Event<'_>, ctx: Context<'_, S>) {
        let parent = if event.is_root() { None }
        else {event.parent().cloned().or_else(|| ctx.current_span().id().cloned())};
        let mut visitor = StringVisitor::default();
        event.record(&mut visitor);
        let message = visitor.0.remove("message").unwrap_or_default();
        let line = LogLine::Simple(SimpleLogLine{
            level: *event.metadata().level(),
            timestamp:Local::now(),
            message,attrs:visitor
        });
        match parent {
            None => {
                let mut store = self.store.write();
                store.1 = true;
                store.0.push(line);
            }
            Some(id) => self.sort_into(id,line)
        }
    }
    fn on_new_span(&self, attrs: &tracing::span::Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let parent = if attrs.is_root() { None }
        else {attrs.parent().cloned().or_else(|| ctx.current_span().id().cloned())};
        let mut visitor = StringVisitor::default();
        attrs.record(&mut visitor);
        let line = LogLine::Span(SpanLine {
            level: *attrs.metadata().level(),
            name: attrs.metadata().name().to_string(),
            attrs: visitor,
            timestamp:Local::now(),
            children: Vec::new(),
            id: id.clone(),
            open:Some(OpenSpan {
                subspans:Vec::new(),
                spinner:&["▹▹▹▹▹", "▹▹▹▹▹","▹▹▹▹▹","▸▹▹▹▹", "▸▹▹▹▹", "▸▹▹▹▹", "▹▸▹▹▹", "▹▸▹▹▹", "▹▸▹▹▹", "▹▹▸▹▹", "▹▹▸▹▹", "▹▹▸▹▹", "▹▹▹▸▹","▹▹▹▸▹","▹▹▹▸▹", "▹▹▹▹▸","▹▹▹▹▸","▹▹▹▹▸"]
                //immt_system::utils::progress::spinners::ARROW3
            })
        });
        match parent {
            None => {
                let mut store = self.store.write();
                store.1 = true;
                store.0.push(line);
            }
            Some(pid) => self.sort_span_into(pid,id.clone(),line)
        }
    }
    fn on_close(&self, id: Id, _ctx: Context<'_, S>) {
        self.sort(id,|_| (),|span| {
            span.open = None
        })
    }
}

#[derive(Default)]
pub struct StringVisitor(HMap<String, String>);
impl Display for StringVisitor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {return Ok(())}
        f.write_char('[')?;
        let mut had = false;
        for (k,v) in &self.0 {
            if had { f.write_str(", ")?;}
            f.write_str(k)?;
            f.write_char(':')?;
            f.write_str(v)?;
        }
        f.write_char(']')
    }
}

impl tracing::field::Visit for StringVisitor {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.0
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.0
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.0
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.0
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.0
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        self.0
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name().to_string(), format!("{:?}", value));
    }
}