use std::path::Path;
use crate::backend::archive_manager::ArchiveManager;
use std::path::PathBuf;
use std::sync::Arc;
use oxigraph::model::GraphName;
use parking_lot::RwLock;
use reedline_repl_rs::clap::Command;
use reedline_repl_rs::{Callback, Error, Repl};
use tracing::{event, info, instrument};
use immt_api::archives::ArchiveGroupT;
use immt_api::formats::{Format,FormatStore};
use crate::ontology::relational::RelationalManager;
use crate::utils::problems::ProblemHandler;


pub struct ControllerBuilder {
    main_mh:PathBuf,handler:Option<ProblemHandler>,
    formats:FormatStore,
    commands:Vec<(Command,Callback<Controller,Error>)>
}

struct ControllerI {
    mgr:ArchiveManager,
    main_mh:PathBuf,
    handler:ProblemHandler,
    relman:RelationalManager,
    formats:FormatStore,
    commands:Vec<(Command,Callback<Controller,Error>)>,
    queue:BuildQueue
}

#[derive(Clone)]
pub struct Controller(Arc<ControllerI>);

impl Controller {
    pub fn builder<S:AsRef<Path>+Into<PathBuf>>(mh:S) -> ControllerBuilder {
        ControllerBuilder {
            main_mh:mh.into(),handler:None,formats:FormatStore::default(),
            commands:vec!(
                (
                    Command::new("exit").about("Exit the program")
                        //.visible_short('q')
                        .visible_alias("quit"),
                    exit
                )
            )
        }
    }
    pub fn build_queue(&self) -> &BuildQueue { &self.0.queue }
    pub fn archives(&self) -> &ArchiveManager { &self.0.mgr }
    pub fn mathhub(&self) -> &Path { &self.0.main_mh }
    pub fn relational_manager(&self) -> &RelationalManager { &self.0.relman }
    pub fn run_repl(&self) {
        let mut repl:Repl<Controller,Error> = Repl::new(self.clone())
            .with_name("iMMT")
            .with_version("v0.1.0")
            .with_description("Foo Bar Baz Bla Blubb Wheeeeee")
            .with_banner("Shaaaalalaaaa");
        for (cmd,cb) in &self.0.commands {
            repl = repl.with_command(cmd.clone(),*cb);
        }
        let _ = repl.run();
    }
}

impl ControllerBuilder {
    #[instrument(level="info",name="Initializing controller",skip_all)]
    pub fn build(self) -> Controller {
        let handler = self.handler.unwrap_or_default();
        let mgr = ArchiveManager::new(&self.main_mh,&handler,&self.formats);
        let mut queue = BuildQueue::default();
        queue.init(&mgr);
        let mut relman = RelationalManager::default();
        relman.init();
        info!("Controller initialized; base ontology has {} quads",relman.size());
        //relman.load_archives(&mgr);
        let ctrl = Controller(Arc::new(ControllerI{
            mgr,queue,
            main_mh:self.main_mh,handler,relman,
            formats:self.formats,commands:self.commands
        }));
        RelationalManager::load_archives(ctrl.clone());
        ctrl
    }
    pub fn with_handler(mut self,handler:ProblemHandler) -> Self {
        self.handler = Some(handler);
        self
    }
    pub fn register_format(&mut self,format:Format) {
        self.formats.register(format);
    }
}


use reedline_repl_rs::{Result as Res,clap::{Arg,ArgMatches}};
use crate::buildqueue::BuildQueue;

fn exit(args: ArgMatches, _context: &mut Controller) -> Res<Option<String>> {
    std::process::exit(1);
    Ok(None)
}