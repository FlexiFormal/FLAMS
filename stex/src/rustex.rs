use std::path::Path;
use parking_lot::Mutex;

mod rustex {
    pub use tex_engine::commands::{Macro, MacroSignature, TeXCommand};
    pub use tex_engine::engine::gullet::DefaultGullet;
    pub use tex_engine::engine::mouth::DefaultMouth;
    pub use tex_engine::engine::{DefaultEngine, EngineAux};
    pub use tex_engine::pdflatex::PDFTeXEngine;
    pub use tex_engine::prelude::{CSName, InternedCSName, Token, TokenList};
    pub use tex_engine::tex::tokens::control_sequences::CSInterner;
    pub use tex_engine::tex::tokens::StandardToken;
    pub use tex_engine::{engine::utils::memory::MemoryManager, tex::tokens::CompactToken};
    pub use tracing::{debug, error, info, info_span, instrument, trace, warn};
    pub use RusTeX::engine::{Extension, RusTeXEngine, Types,RusTeXEngineT};
    pub use RusTeX::engine::output::{OutputCont, RusTeXOutput};
    pub use RusTeX::engine::stomach::RusTeXStomach;
    pub use RusTeX::engine::{fonts::Fontsystem, state::RusTeXState};
    pub use RusTeX::engine::files::RusTeXFileSystem;
    pub use RusTeX::engine::commands::{register_primitives_preinit,register_primitives_postinit};
    pub type RTSettings = RusTeX::engine::Settings;
    pub use RusTeX::ImageOptions;
}
use rustex::*;

struct TracingOutput;
impl OutputCont for TracingOutput {
    fn message(&self, text: String) {
        debug!(target:"rustex","{}", text);
    }
    fn errmessage(&self, text: String) {
        debug!(target:"rustex","{}", text);
    }
    fn file_open(&self, text: String) {
        trace!(target:"rustex","({}", text);
    }
    fn file_close(&self, text: String) { trace!(target:"rustex",")"); }
    fn write_18(&self, text: String) {
        trace!(target:"rustex","write18: {}", text);
    }
    fn write_17(&self, text: String) {
        debug!(target:"rustex","{}", text);
    }
    fn write_16(&self, text: String) {
        trace!(target:"rustex","write16: {}", text);
    }
    fn write_neg1(&self, text: String) {
        trace!(target:"rustex","write-1: {}", text);
    }
    fn write_other(&self, text: String) {
        trace!(target:"rustex","write: {}", text);
    }
}

#[derive(Clone)]
struct EngineBase {
    state: RusTeXState,
    memory: MemoryManager<CompactToken>,
    font_system: Fontsystem,
}
static ENGINE_BASE: Mutex<Option<EngineBase>> = Mutex::new(None);
impl EngineBase {
    fn into_engine<I:IntoIterator<Item=(String,String)>>(mut self,envs:I) -> RusTeXEngine {
        use tex_engine::engine::filesystem::FileSystem;
        use tex_engine::engine::gullet::Gullet;
        use tex_engine::engine::stomach::Stomach;
        use tex_engine::engine::EngineExtension;
        use tex_engine::prelude::ErrorHandler;
        use tex_engine::prelude::Mouth;
        let mut aux = EngineAux {
            outputs: RusTeXOutput::Cont(Box::new(TracingOutput)),
            error_handler: ErrorThrower::new(),
            start_time: chrono::Local::now(),
            extension: Extension::new(&mut self.memory),
            memory: self.memory,
            jobname: String::new(),
        };
        let mut mouth = DefaultMouth::new(&mut aux, &mut self.state);
        let gullet = DefaultGullet::new(&mut aux, &mut self.state, &mut mouth);
        let mut stomach = RusTeXStomach::new(&mut aux, &mut self.state);
        stomach.continuous = true;
        DefaultEngine {
            state: self.state,
            aux,
            fontsystem: self.font_system,
            filesystem: RusTeXFileSystem::new_with_envs(tex_engine::utils::PWD.to_path_buf(),envs),
            mouth,
            gullet,
            stomach,
        }
    }
    fn get<R,F:FnOnce(&Self) -> R>(f:F) -> R {
        let mut lock = ENGINE_BASE.lock();
        match &mut *lock {
            Some(engine) => f(&engine),
            o => {
                *o = Some(Self::initialize());
                f(o.as_ref().unwrap())
            }
        }
    }

    #[instrument(level = "info",
        target = "sTeX",
        name = "Initializing RusTeX engine"
    )]
    fn initialize() -> Self {
        let mut engine = DefaultEngine::<Types>::default();
        engine.aux.outputs = RusTeXOutput::Cont(Box::new(TracingOutput));
        register_primitives_preinit(&mut engine);
        match engine.initialize_pdflatex() {
            Ok(_) => {}
            Err(e) => {
                error!("Error initializing RusTeX engine: {}", e);
            }
        };
        register_primitives_postinit(&mut engine);
        EngineBase {
            state: engine.state.clone(),
            memory: engine.aux.memory.clone(),
            font_system: engine.fontsystem.clone(),
        }
    }
}

pub struct RusTeX(parking_lot::RwLock<EngineBase>);
impl RusTeX {
    pub fn get() -> Self {
        Self(ENGINE_BASE.lock().as_ref().unwrap().clone().into())
    }
    pub fn initialize() { EngineBase::get(|_| ()); }
    pub fn run(&self,file:&Path,memorize:bool) -> Result<String,String> {
        self.run_with_envs(file,memorize,std::iter::once(("IMMT_ADMIN_PWD".to_string(),"NOPE".to_string())))
    }
    pub fn run_with_envs<I:IntoIterator<Item=(String,String)>>(&self,file:&Path,memorize:bool,envs:I) -> Result<String,String> {
        let mut engine = self.0.read().clone().into_engine(envs);
        let settings = RTSettings {
            verbose:false,
            sourcerefs:false,
            log:true,
            image_options:ImageOptions::AsIs
        };
        let res = engine.run(file.to_str().unwrap(), settings);
        if let Some(e) = res.error {
            Err(e.to_string())
        } else {
            if memorize {
                let mut base = self.0.write();
                give_back(engine, &mut base);
            }
            Ok(res.to_string())
        }
    }
}
/*
fn run_rustex(file: &Path) -> Result<String, String> {
    let mut engine = EngineBase::get(|e| e.clone()).into_engine();
    let res = engine.run(file.to_str().unwrap(), false);
    if let Some(e) = res.error {
        Err(e.to_string())
    } else {
        give_back(engine);
        Ok(res.out)
    }
}

 */

fn save_macro(
    name: InternedCSName<u8>,
    m: &Macro<CompactToken>,
    oldmem: &CSInterner<u8>,
    newmem: &mut CSInterner<u8>,
    state: &mut RusTeXState,
) {
    let newname = convert_name(name, oldmem, newmem);

    let exp = &m.expansion;
    let newexp: TokenList<_> = exp
        .0
        .iter()
        .map(|x| convert_token(*x, oldmem, newmem))
        .collect();
    let newsig = MacroSignature {
        arity: m.signature.arity,
        params: m
            .signature
            .params
            .0
            .iter()
            .map(|x| convert_token(*x, oldmem, newmem))
            .collect(),
    };
    let newmacro = Macro {
        protected: m.protected,
        long: m.long,
        outer: m.outer,
        signature: newsig,
        expansion: newexp,
    };
    state.set_command_direct(newname, Some(TeXCommand::Macro(newmacro)))
}

fn convert_name(
    oldname: InternedCSName<u8>,
    oldmem: &CSInterner<u8>,
    newmem: &mut CSInterner<u8>,
) -> InternedCSName<u8> {
    newmem.intern(oldmem.resolve(oldname))
}

fn convert_token(
    old: CompactToken,
    oldmem: &CSInterner<u8>,
    newmem: &mut CSInterner<u8>,
) -> CompactToken {
    match old.to_enum() {
        StandardToken::ControlSequence(cs) => {
            CompactToken::from_cs(convert_name(cs, oldmem, newmem))
        }
        _ => old,
    }
}

use tex_engine::tex::tokens::control_sequences::CSNameMap;
use tex_engine::utils::errors::ErrorThrower;

fn give_back(engine: RusTeXEngine,base:&mut EngineBase) {
    let EngineBase {
        state,
        memory,
        font_system,
    } = base;
    *font_system = engine.fontsystem;
    let oldinterner = engine.aux.memory.cs_interner();
    let iter = CommandIterator {
        prefix: b"c_stex_module_",
        cmds: engine.state.destruct().into_iter(),
        interner: oldinterner,
    };
    for (n, c) in iter.filter_map(|(a, b)| match b {
        TeXCommand::Macro(m) => Some((a, m)),
        _ => None,
    }) {
        save_macro(n, &c, oldinterner, memory.cs_interner_mut(), state);
    }
}

pub struct CommandIterator<'a, I: Iterator<Item = (InternedCSName<u8>, TeXCommand<Types>)>> {
    prefix: &'static [u8],
    cmds: I,
    interner: &'a <InternedCSName<u8> as CSName<u8>>::Handler,
}
impl<I: Iterator<Item = (InternedCSName<u8>, TeXCommand<Types>)>> Iterator
    for CommandIterator<'_, I>
{
    type Item = (InternedCSName<u8>, TeXCommand<Types>);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((name, cmd)) = self.cmds.next() {
                let bname = self.interner.resolve(name);
                if bname.starts_with(self.prefix) {
                    return Some((name, cmd));
                }
            } else {
                return None;
            }
        }
    }
}
