use ftml_ontology::utils::time::measure;
use ftml_uris::ModuleUri;
use rustex_lib::engine::RusTeXEngineExt;

use crate::{rustex::EngineBase, RusTeX};

static PREAMBLE: &str = r"
\documentclass{stex}
\usepackage[T1]{fontenc}
\usepackage[utf8]{inputenc}
\usepackage[hide]{ed}
\usepackage[hyperref=auto,style=alphabetic,backend=bibtex]{biblatex}
\usepackage{url,amstext,amsfonts,amsmath,bbm,amssymb,stix2,csquotes,listings}
\lstset{columns=fullflexible,basicstyle=\ttfamily}
\usepackage[hidelinks]{hyperref}
\usepackage[dvipsnames]{xcolor}
\usepackage{stex-highlighting,stexthm}
";

pub struct RusTeXMath {
    engine: RusTeX,
    modules: parking_lot::Mutex<String>, //preamble: std::sync::Mutex<String>,
}
impl std::fmt::Debug for RusTeXMath {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RusTeXMath")
    }
}

static BASE: std::sync::LazyLock<RusTeX> = std::sync::LazyLock::new(|| {
    RusTeX::initialize();
    tracing::info_span!("initializing math engine").in_scope(move || {
        tracing::info!("Getting RusTeX");
        let mut engine = RusTeX::get().expect("wut").0.into_inner().into_engine();
        tracing::info!("Compiling preamble");
        engine.run_string(std::path::PathBuf::from("/rustex_math.tex"), PREAMBLE);
        tracing::info!("Packaging");
        let r = RusTeX(parking_lot::Mutex::new(EngineBase::from_engine(engine)));
        tracing::info!("Done");
        r
    })
});

impl Default for RusTeXMath {
    fn default() -> Self {
        Self {
            engine: RusTeX(parking_lot::Mutex::new(BASE.0.lock().clone())),
            modules: parking_lot::Mutex::new(String::new()),
        }
    }
}

impl RusTeXMath {
    #[inline]
    pub fn initialize() {
        std::sync::LazyLock::force(&BASE);
    }

    pub fn add_usemodule(&self, module: &ModuleUri) {
        use std::fmt::Write;
        let mut lock = self.modules.lock();
        let Some(rs) = self
            .engine
            .builder()
            .set_sourcerefs(false)
            .set_font_debug_info(false)
            .set_string_noaux(
                std::path::Path::new("./rustex_math.tex"),
                &format!(
                    "\\begin{{document}}\\usemodule{}\\end{{document}}",
                    module.short_id_string()
                ),
            )
        else {
            return;
        };
        let (_, res) = rs.run();
        let _ = write!(lock, "\\usemodule{}", module.short_id_string());
        res.memorize(&self.engine);
        drop(lock);
    }

    /// ### Errors
    pub fn run(&self, math: &str) -> Result<String, String> {
        tracing::info_span!("running {}", math).in_scope(move || {
            let rs = self
                .engine
                .builder()
                .set_sourcerefs(false)
                .set_font_debug_info(false);
            let Some(rs) = rs.set_string_noaux(
                std::path::Path::new("./rustex_math.tex"),
                &format!(
                    "\\begin{{document}}\n{}\n{math}\\end{{document}}",
                    &*self.modules.lock()
                ),
            ) else {
                return Err(format!("Could not add file with content \"{math}\""));
            };
            let (out, _) = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| rs.run()))
            {
                Ok(v) => v,
                Err(e) => {
                    return Err({
                        e.downcast_ref::<&str>().map_or_else(
                            || {
                                e.downcast_ref::<String>()
                                    .map_or_else(|| "Unknown error".to_string(), String::clone)
                            },
                            ToString::to_string,
                        )
                    })
                }
            };
            if out.error.is_some() {
                // SAFETY: we just checked is_some()
                return unsafe { Err(out.error.unwrap_unchecked().0.to_string()) };
            }
            let html = out.to_string();
            let Some(start) = html.find("<math") else {
                return Err("No math node found".to_string());
            };
            let Some(end) = html.rfind("</math>") else {
                return Err("No math node found".to_string());
            };
            Ok(html[start..end + "</math>".len()].to_string())
        })
    }
}

#[test]
fn math_test() {
    use ftml_ontology::utils::time::measure;

    tracing_subscriber::fmt().init();
    let (engine, t) = measure(RusTeXMath::default);
    tracing::info!("Initialized after {t}");
    let ((), t) = measure(|| {
        engine.add_usemodule(
            &"http://mathhub.info?a=smglom/arithmetics&p=mod&m=realarith"
                .parse()
                .expect("foo"),
        );
    });
    tracing::info!("Added module in {t}");
    let ((), t) = measure(|| {
        engine.add_usemodule(
            &"http://mathhub.info?a=smglom/arithmetics&p=mod&m=ratarith"
                .parse()
                .expect("foo"),
        );
    });
    tracing::info!("Added redundant module in {t}");
    let (out, t) = measure(|| {
        engine
            .run("$\\realabsval{\\realplus{a,b,c,d,e}}$")
            .expect("works")
    });
    tracing::info!("Converted first math in {t} to:\n{out}");
    let (out, t) = measure(|| {
        engine
            .run("$$\\realabsval{\\realplus{f,g,h}}$$")
            .expect("works")
    });
    tracing::info!("Converted second math in {t} to:\n{out}");
}
