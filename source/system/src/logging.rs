use flams_utils::{
    change_listener::{ChangeListener, ChangeSender},
    logs::LogFileLine,
    time::Timestamp,
    triomphe::Arc,
    vecmap::VecMap,
};
use std::{fmt::Display, path::PathBuf};
use tracing::span::Id;
use tracing_subscriber::{layer::Context, Layer};

#[derive(Clone, Debug)]
pub struct LogStore(Arc<LogStoreI>);

static LOG: std::sync::OnceLock<LogStore> = std::sync::OnceLock::new();

impl LogStore {
    ///#### Panics
    pub fn initialize() {
        use tracing_subscriber::layer::SubscriberExt;
        if LOG.get().is_some() {
            return;
        }
        let logger = Self::new();
        let level = if crate::settings::Settings::get().debug {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        };
        let subscriber = tracing_subscriber::registry()
            .with(logger.with_filter(tracing::level_filters::LevelFilter::from(level)))
            .with(tracing_error::ErrorLayer::default());
        tracing::subscriber::set_global_default(subscriber)
            .expect("Error initializing tracing subscriber");
    }

    #[allow(clippy::new_without_default)]
    ///#### Panics
    pub fn new() -> Self {
        assert!(LOG.get().is_none(), "Logger already initialized");
        let settings = crate::settings::Settings::get();
        let logdir = &settings.log_dir;
        let filename = chrono::Local::now()
            .format("%Y-%m-%d-%H.%M.%S.log")
            .to_string();
        let path = logdir.join(&filename);
        let ret = Self::new_i(/*guard,file_layer,*/ path);
        LOG.set(ret.clone()).expect("Error initializing logger");
        ret
    }
}

/// ### Panics
pub fn logger() -> &'static LogStore {
    LOG.get().expect("log should be initialized")
}

#[derive(Clone)]
enum Msg {
    Line(LogFileLine),
    Kill,
}

#[derive(Debug)]
struct LogStoreI {
    notifier: ChangeListener<LogFileLine>,
    sender: crossbeam_channel::Sender<Msg>,
    log_file: PathBuf,
}

impl Drop for LogStoreI {
    fn drop(&mut self) {
        let _ = self.sender.send(Msg::Kill);
    }
}

impl LogStore {
    #[allow(clippy::let_underscore_future)]
    fn new_i<P: Into<PathBuf>>(log_file: P) -> Self {
        let (sender, recv) = crossbeam_channel::unbounded();
        let cs = ChangeSender::new(1024);
        let store = Arc::new(LogStoreI {
            notifier: cs.listener(),
            sender,
            log_file: log_file.into(),
        });
        let file = store.log_file.clone();
        let _ = tokio::task::spawn_blocking(move || {
            if let Some(p) = file.parent() {
                let _ = std::fs::create_dir_all(p);
            };
            let f = std::fs::File::create(file).expect("Failed to create log file");
            let mut f = std::io::BufWriter::new(f); //std::fs::File::create_buffered(file).expect("Failed to create log file");
            loop {
                match recv.recv() {
                    Err(_) | Ok(Msg::Kill) => break,
                    Ok(Msg::Line(msg)) => {
                        let _ = serde_json::to_writer(&mut f, &msg);
                        let _ = std::io::Write::write(&mut f, b"\n");
                        cs.send(msg);
                    }
                }
            }
        });
        Self(store)
    }
    #[must_use]
    pub fn listener(&self) -> ChangeListener<LogFileLine> {
        self.0.notifier.clone()
    }
    #[must_use]
    pub fn log_file(&self) -> &std::path::Path {
        &self.0.log_file
    }
}

impl<S: tracing::Subscriber> Layer<S> for LogStore {
    fn on_event(&self, event: &tracing::Event<'_>, ctx: Context<'_, S>) {
        let mut visitor = StringVisitor::default();
        event.record(&mut visitor);
        let args = visitor.0;
        let message = visitor.1;
        let timestamp = Timestamp::now();
        let parent = if event.is_root() {
            None
        } else {
            event.parent().map_or_else(
                || ctx.current_span().id().map(tracing::Id::into_non_zero_u64),
                |i| Some(i.into_non_zero_u64()),
            )
        };
        let target: Option<String> = {
            let tg = event.metadata().target();
            if tg.starts_with("flams") {
                None
            } else {
                Some(tg.into())
            }
        };
        let msg = LogFileLine::Message {
            message,
            timestamp,
            target,
            level: (*event.metadata().level()).into(),
            args,
            span: parent,
        };
        let _ = self.0.sender.send(Msg::Line(msg));
    }

    fn on_new_span(&self, md: &tracing::span::Attributes<'_>, thisid: &Id, ctx: Context<'_, S>) {
        let mut visitor = StringVisitor::default();
        md.record(&mut visitor);
        let args = visitor.0;
        let name = md.metadata().name().to_string();
        let target: Option<String> = {
            let tg = md.metadata().target();
            if tg.starts_with("flams") {
                None
            } else {
                Some(tg.into())
            }
        };
        let parent = if md.is_root() {
            None
        } else {
            md.parent().map_or_else(
                || ctx.current_span().id().map(tracing::Id::into_non_zero_u64),
                |i| Some(i.into_non_zero_u64()),
            )
        };
        let id = thisid.into_non_zero_u64();
        let level = (*md.metadata().level()).into();
        let _ = self.0.sender.send(Msg::Line(LogFileLine::SpanOpen {
            name,
            id,
            timestamp: Timestamp::now(),
            target,
            level,
            args,
            parent,
        }));
    }

    fn on_close(&self, id: Id, _ctx: Context<'_, S>) {
        let _ = self.0.sender.send(Msg::Line(LogFileLine::SpanClose {
            id: id.into_non_zero_u64(),
            timestamp: Timestamp::now(),
        }));
    }
}

#[derive(Default)]
struct StringVisitor(VecMap<String, String>, String);
impl Display for StringVisitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        if self.0.is_empty() {
            return Ok(());
        }
        f.write_char('{')?;
        let mut had = false;
        for (k, v) in self.0.iter() {
            if had {
                f.write_str(", ")?;
            }
            f.write_str(k)?;
            f.write_char('=')?;
            f.write_str(v)?;
            had = true;
        }
        f.write_char('}')
    }
}

impl tracing::field::Visit for StringVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.1 = format!("{value:?}");
        } else {
            self.0
                .insert(field.name().to_string(), format!("{value:?}"));
        }
    }
}
