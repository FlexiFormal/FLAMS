use std::{fmt::{Display, Write}, path::{Path, PathBuf}};

use flams_utils::{change_listener::{ChangeListener, ChangeSender}, logs::LogFileLine, time::Timestamp, triomphe::Arc, vecmap::VecMap};
use parking_lot::RwLock;
use tracing::span::Id;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt::format::FmtSpan, layer::Context, Layer};


pub(crate) fn tracing(logdir:&Path,level: tracing::Level,rotation: tracing_appender::rolling::Rotation) -> LogStore {
  use tracing::level_filters::LevelFilter;
  use tracing_subscriber::fmt::writer::MakeWriterExt;
  use tracing_subscriber::layer::SubscriberExt;

  let filename = chrono::Local::now().format("%Y-%m-%d-%H.%M.%S.log").to_string();
  let path = logdir.join(&filename);

  let file_appender = tracing_appender::rolling::RollingFileAppender::builder()
      .rotation(rotation)
      .filename_prefix(filename)
      .build(logdir)
      .expect("failed to initialize file logging");
  let (file_layer,guard) = tracing_appender::non_blocking(file_appender);
  let file_layer = file_layer.with_max_level(level);
  /*
  let l = Logger(Arc::new(RwLock::new(LoggerI {
      _guard: guard,
      layers : vec![]
  })));
  
   */
  
  let logger = LogStore::new(guard,path);

  let subscriber = tracing_subscriber::registry()
      .with(
          tracing_subscriber::fmt::Layer::default()
              .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
              .with_writer(file_layer)
              .with_ansi(false)
              .with_file(false)
              .with_line_number(false)
              .json()
              .flatten_event(true)
              .with_current_span(true)
              .with_span_list(true)
              //.map_fmt_fields(|f| )
          ,
      )
      .with(logger.clone().with_filter(LevelFilter::from(level)))
      .with(tracing_error::ErrorLayer::default());
  let _ = tracing::subscriber::set_global_default(subscriber).expect(
    "Error initializing tracing subscriber"
  );
  logger
}

#[derive(Debug)]
struct LogStoreI {
  notifier:ChangeSender<LogFileLine<String>>,
  _guard:WorkerGuard,
  log_file:PathBuf,
  open_spans:RwLock<VecMap<Id,(String,Option<Id>)>>
}
#[derive(Clone,Debug)]
pub struct LogStore(Arc<LogStoreI>);
impl LogStore {
  fn new<P:Into<PathBuf>>(guard:WorkerGuard,log_file:P) -> Self {
      let store = Arc::new(LogStoreI {
          notifier: ChangeSender::new(1024),
          _guard: guard,
          open_spans:RwLock::new(VecMap::default()),
          log_file: log_file.into()
      });
      Self(store)
  }
  #[must_use]
  pub fn listener(&self) -> ChangeListener<LogFileLine<String>> {
      self.0.notifier.listener()
  }
  #[must_use]
  pub fn log_file(&self) -> &std::path::Path { &self.0.log_file }
}

impl LogStore {
  fn parents(&self,id:&Id) -> Vec<String> {
      let mut ret = Vec::new();
      let spans = self.0.open_spans.read();
      let mut id = id;
      while let Some((span,nid)) = spans.get(id) {
          ret.push(span.clone());
          if let Some(nid) = nid {
              id = nid;
          } else {break}
      }
      ret
  }
  fn open_span(&self,id:&Id,name:&str,parent:Option<&Id>) {
    //println!("Here:{id:?}={name},{parent:?}");
      let mut spans = self.0.open_spans.write();
      spans.insert(id.clone(),(name.to_string(),parent.cloned()));
  }
  fn close_span(&self,id:&Id) -> Option<(String,Option<Id>)> {
      let mut spans = self.0.open_spans.write();
      spans.remove(id)
  }
}

impl<S: tracing::Subscriber> Layer<S> for LogStore {
  fn on_event(&self, event: &tracing::Event<'_>, ctx: Context<'_, S>) {
      self.0.notifier.lazy_send(move || {
          let target: Option<String> = {
              let tg = event.metadata().target();
              if tg.starts_with("flams") { None } else { Some(tg.into()) }
          };
          let mut visitor = StringVisitor::default();
          event.record(&mut visitor);
          let mut args: VecMap<String, String> = visitor.0.into_iter().map(|(a,b)| (a.into(),b)).collect();
          let message = args.remove("message");
          let timestamp = Timestamp::now();
          let parent = if event.is_root() { None }
          else {event.parent().cloned().or_else(|| ctx.current_span().id().cloned())};
          let parent = parent.and_then(|id| self.parents(&id).pop());
          LogFileLine::Message {
              message: message.unwrap_or_default(),
              timestamp,
              target,
              level: (*event.metadata().level()).into(),
              args,
              span:parent,
          }
      });
  }
  fn on_new_span(&self, md: &tracing::span::Attributes<'_>, thisid: &Id, ctx: Context<'_, S>) {
      let name = md.metadata().name().to_string();
      let mut visitor = StringVisitor::default();
      md.record(&mut visitor);
      let args: VecMap<String, String> = visitor.0.into_iter().map(|(a,b)| (a.into(),b)).collect();
      let id =  LogFileLine::id_from(&name,&args);
      self.open_span(thisid,&id,md.parent());
      self.0.notifier.lazy_send(move || {
          let target: Option<String> = {
              let tg = md.metadata().target();
              if tg.starts_with("flams") { None } else { Some(tg.into()) }
          };
          let timestamp = Timestamp::now();
          let parent = if md.is_root() { None }
          else {md.parent().cloned().or_else(|| ctx.current_span().id().cloned())};
          let parent = parent.and_then(|id| self.parents(&id).pop());
          LogFileLine::SpanOpen {
              name,
              timestamp,
              target,
              level: (*md.metadata().level()).into(),
              args,
              parent,
          }
      });
  }
  
  fn on_close(&self, id: Id, _ctx: Context<'_, S>) {
      if let Some((span,parent)) = self.close_span(&id) {
          let parent = parent.and_then(|id| self.parents(&id).pop());
          self.0.notifier.lazy_send(move || {
              LogFileLine::SpanClose {
                  id:span,
                  parent,
                  timestamp: Timestamp::now(),
              }
          });
      }
  }
}


#[derive(Default)]
struct StringVisitor(VecMap<&'static str, String>);
impl Display for StringVisitor {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      if self.0.is_empty() {return Ok(())}
      f.write_char('{')?;
      let mut had = false;
      for (k,v) in self.0.iter() {
          if had { f.write_str(", ")?;}
          f.write_str(k)?;
          f.write_char('=')?;
          f.write_str(v)?;
          had = true;
      }
      f.write_char('}')
  }
}

impl tracing::field::Visit for StringVisitor {
  fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
      self.0
          .insert(field.name(), value.to_string());
  }

  fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
      self.0
          .insert(field.name(), value.to_string());
  }

  fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
      self.0
          .insert(field.name(), value.to_string());
  }

  fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
      self.0
          .insert(field.name(), value.to_string());
  }

  fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
      self.0
          .insert(field.name(), value.to_string());
  }

  fn record_error(
      &mut self,
      field: &tracing::field::Field,
      value: &(dyn std::error::Error + 'static),
  ) {
      self.0
          .insert(field.name(), value.to_string());
  }

  fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
      self.0
          .insert(field.name(), format!("{value:?}"));
  }
}