use std::path::Path;
use crate::backend::archive_manager::ArchiveManager;
use std::path::PathBuf;
use std::sync::Arc;
use either::Either;
use oxigraph::model::GraphName;
use parking_lot::RwLock;
//use reedline_repl_rs::clap::Command;
//use reedline_repl_rs::{Callback, Error, Repl};
use tracing::{event, info, instrument};
use immt_api::archives::{ArchiveGroupT, ArchiveId};
use immt_api::formats::{Format,FormatStore};
use immt_api::formats::building::Backend;
use crate::ontology::relational::RelationalManager;
use crate::buildqueue::BuildQueue;
use crate::settings::Settings;


pub struct ControllerBuilder {
    main_mh:PathBuf,
    formats:FormatStore,
    settings:Settings,
    //commands:Vec<(Command,Callback<Controller,Error>)>
}

struct ControllerI {
    mgr:ArchiveManager,
    settings:Settings,
    main_mh:PathBuf,
    relman:RelationalManager,
    formats:FormatStore,
    //commands:Vec<(Command,Callback<Controller,Error>)>,
    queue:BuildQueue
}

#[derive(Clone)]
pub struct Controller(Arc<ControllerI>);

impl Controller {
    pub fn builder<S:AsRef<Path>+Into<PathBuf>>(mh:S) -> ControllerBuilder {
        ControllerBuilder {
            main_mh:mh.into(),formats:FormatStore::default(),
            settings:Settings::default(),
            /*commands:vec!(
                (
                    Command::new("exit").about("Exit the program")
                        //.visible_short('q')
                        .visible_alias("quit"),
                    exit
                )
            )*/
        }
    }
    pub fn as_backend<'a>(&'a self) -> impl Fn(&ArchiveId) -> Option<Arc<Path>> + 'a {
        move |id| match self.archives().find(id.clone()) {
            Some(Either::Right(a)) => a.path().clone(),
            _ => None
        }
    }
    pub fn formats(&self) -> &FormatStore { &self.0.formats }
    pub fn build_queue(&self) -> &BuildQueue { &self.0.queue }
    pub fn archives(&self) -> &ArchiveManager { &self.0.mgr }
    pub fn mathhub(&self) -> &Path { &self.0.main_mh }
    pub fn relational_manager(&self) -> &RelationalManager { &self.0.relman }
    pub fn settings(&self) -> &Settings { &self.0.settings }
    /*pub fn run_repl(&self) {
        let mut repl:Repl<Controller,Error> = Repl::new(self.clone())
            .with_name("iMMT")
            .with_version("v0.1.0")
            .with_description("Foo Bar Baz Bla Blubb Wheeeeee")
            .with_banner("Shaaaalalaaaa");
        for (cmd,cb) in &self.0.commands {
            repl = repl.with_command(cmd.clone(),*cb);
        }
        let _ = repl.run();
    }*/
}

impl ControllerBuilder {
    #[instrument(level="info",name="Initializing controller",skip_all)]
    pub fn build(self) -> Controller {
        let mgr = ArchiveManager::new(&self.main_mh,&self.formats);
        let mut queue = BuildQueue::default();
        let mut relman = RelationalManager::default();
        relman.init();
        info!("Controller initialized; base ontology has {} quads",relman.size());
        //relman.load_archives(&mgr);
        let ctrl = Controller(Arc::new(ControllerI{
            mgr,queue,
            settings:self.settings,
            main_mh:self.main_mh,relman,
            formats:self.formats//,commands:self.commands
        }));
        BuildQueue::init(ctrl.clone());
        RelationalManager::load_archives(ctrl.clone());
        ctrl
    }
    pub fn register_format(&mut self,format:Format) {
        self.formats.register(format);
    }
}

/*
//use reedline_repl_rs::{Result as Res,clap::{Arg,ArgMatches}};

fn exit(args: ArgMatches, _context: &mut Controller) -> Res<Option<String>> {
    std::process::exit(1);
    Ok(None)
}

 */