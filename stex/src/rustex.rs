use std::path::Path;
use tex_engine::commands::{Macro, MacroSignature, TeXCommand};
use tex_engine::engine::gullet::DefaultGullet;
use tex_engine::engine::mouth::DefaultMouth;
use tex_engine::engine::{DefaultEngine, EngineAux};
use tex_engine::pdflatex::PDFTeXEngine;
use tex_engine::prelude::{CSName, InternedCSName, Token, TokenList};
use tex_engine::tex::tokens::control_sequences::CSInterner;
use tex_engine::tex::tokens::StandardToken;
use tex_engine::{engine::utils::memory::MemoryManager, tex::tokens::CompactToken};
use tracing::{debug, error, info, info_span, trace, warn};
use RusTeX::engine::{Extension, RusTeXEngine, Types};
use RusTeX::output::{OutputCont, RusTeXOutput};
use RusTeX::stomach::RusTeXStomach;
use RusTeX::{fonts::Fontsystem, state::RusTeXState};

pub struct TracingOutput;
impl OutputCont for TracingOutput {
    fn message(&self, text: String) {
        debug!("{}", text);
    }
    fn errmessage(&self, text: String) {
        trace!("{}", text);
    }
    fn file_open(&self, text: String) {
        trace!("({}", text);
    }
    fn file_close(&self, text: String) {
        trace!(")");
    }
    fn write_18(&self, text: String) {
        trace!("write18: {}", text);
    }
    fn write_17(&self, text: String) {
        trace!("write17: {}", text);
    }
    fn write_16(&self, text: String) {
        trace!("write16: {}", text);
    }
    fn write_neg1(&self, text: String) {
        trace!("write-1: {}", text);
    }
    fn write_other(&self, text: String) {
        trace!("write: {}", text);
    }
}

#[derive(Clone)]
struct EngineBase {
    state: RusTeXState,
    memory: MemoryManager<CompactToken>,
    font_system: Fontsystem,
}
impl EngineBase {
    fn into_engine(mut self) -> RusTeXEngine {
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
            filesystem: RusTeX::files::RusTeXFileSystem::new(tex_engine::utils::PWD.to_path_buf()),
            mouth,
            gullet,
            stomach,
        }
    }
}
static ENGINE_BASE: parking_lot::Mutex<Option<EngineBase>> = parking_lot::Mutex::new(None);

pub fn initialize() {
    let _ = get_engine();
}

fn get_engine() -> RusTeXEngine {
    let mut guard = ENGINE_BASE.lock();
    if let Some(e) = &*guard {
        e.clone().into_engine()
    } else {
        let _s = info_span!(target:"rustex","Initializing RusTeX engine");
        let _e = _s.enter();
        let start = std::time::Instant::now();
        let mut engine = DefaultEngine::<Types>::new();
        engine.aux.outputs = RusTeXOutput::Cont(Box::new(TracingOutput));
        RusTeX::commands::register_primitives_preinit(&mut engine);
        match engine.initialize_pdflatex() {
            Ok(_) => {}
            Err(e) => {
                error!("Error initializing RusTeX engine: {}", e);
            }
        };
        RusTeX::commands::register_primitives_postinit(&mut engine);
        let e = EngineBase {
            state: engine.state.clone(),
            memory: engine.aux.memory.clone(),
            font_system: engine.fontsystem.clone(),
        };
        *guard = Some(e.clone());
        info!("RusTeX engine initialized in {:?}", start.elapsed());
        e.into_engine()
    }
}

pub fn run_rustex(file: &Path) -> Result<String, String> {
    use RusTeX::engine::RusTeXEngineT;
    let mut engine = get_engine();
    let res = engine.run(file.to_str().unwrap(), false);
    if let Some(e) = res.error {
        Err(e.to_string())
    } else {
        give_back(engine);
        Ok(res.out)
    }
}

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

fn give_back(engine: RusTeXEngine) {
    let mut guard = ENGINE_BASE.lock();
    let EngineBase {
        state,
        memory,
        font_system,
    } = guard.as_mut().unwrap();
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
