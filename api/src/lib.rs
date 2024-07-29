use std::path::{Path, PathBuf};
use lazy_static::lazy_static;
pub static API_VERSION: &str = env!("CARGO_PKG_VERSION");
pub static RUSTC_VERSION: &str = env!("RUSTC_VERSION");

pub mod async_trait { pub use async_trait::*; }

pub mod backend {
    pub mod archives;
    pub mod manager;
    #[cfg(feature="oxigraph")]
    pub mod relational;
}
pub mod extensions;
pub mod controller;
pub mod checking;

pub mod building {
    pub mod targets;
    pub mod buildqueue;
    pub mod queue;
    pub mod tasks;
}
pub mod utils {
    use std::ffi::OsStr;
    use std::path::Path;
    use lazy_static::lazy_static;
    use immt_core::utils::triomphe::Arc;

    pub mod asyncs;
    pub mod circular_buffer;
    pub mod settings;

    
    pub fn time<A,F:FnOnce() -> A>(f:F,s:&str) -> A {
        let start = std::time::Instant::now();
        let ret = f();
        let dur = start.elapsed();
        tracing::info!("{s} took {:?}", dur);
        ret
    }
    pub fn run_command<'a,A:AsRef<OsStr>,S:AsRef<OsStr>,
        Env:Iterator<Item=(S,S)>,
        Args:Iterator<Item=A>
    >(cmd:&str,args:Args,in_path:&Path,mut with_envs:Env) -> Result<std::process::Output,()> {
        let mut proc = std::process::Command::new(cmd);
        let mut process = proc
            .args(args)
            .current_dir(in_path)
            .env("IMMT_ADMIN_PWD","NOPE");
        for (k,v) in with_envs {
            process = process.env(k,v);
        }
        match process
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn() {
            Ok(c) => c.wait_with_output().map_err(|e|{
                tracing::error!("Error executing command {cmd}: {e}");
                ()
            }),
            Err(e) => {
                tracing::error!("Error executing command {cmd}: {e}");
                Err(())
            },
        }

    }
}

pub mod core { pub use immt_core::*; }

pub type HMap<A,B> = rustc_hash::FxHashMap<A,B>;

lazy_static! {
    pub static ref MATHHUB_PATHS: Box<[PathBuf]> = mathhubs().into();
    static ref CONFIG_DIR: Option<PathBuf> = {
        simple_home_dir::home_dir().map(|d| d.join(".immt"))
    };
}

pub fn config_dir() -> Option<&'static Path> { CONFIG_DIR.as_ref().map(|a| a.as_path())}
pub fn exe_dir() -> Option<PathBuf> { std::env::current_exe().ok().map(|p| p.parent().map(|p| p.to_path_buf())).flatten() }

fn mathhubs() -> Vec<PathBuf> {
    if let Ok(f) = std::env::var("MATHHUB") {
        return f.split(',').map(|s| PathBuf::from(s.trim())).collect()
    }
    if let Some(d) = simple_home_dir::home_dir() {
        let p = d.join(".mathhub").join("mathhub.path");
        if let Ok(f) = std::fs::read_to_string(p) {
            return f.split('\n').map(|s| PathBuf::from(s.trim())).collect()
        }
        return vec![d.join("MathHub")];
    }
    panic!(
        "No MathHub directory found and default ~/MathHub not accessible!\n\
    Please set the MATHHUB environment variable or create a file ~/.mathhub/mathhub.path containing \
    the path to the MathHub directory."
    )
}

#[cfg(test)]
pub mod tests {
    pub use rstest::{fixture,rstest};
    pub use tracing::{info,warn,error};

    #[fixture]
    pub fn setup() {
        let _ = tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).try_init();
    }
}