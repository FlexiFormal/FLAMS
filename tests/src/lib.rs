#[cfg(test)]
mod ontology;

#[cfg(test)]
pub mod tests {
    use std::path::Path;
    use criterion::Criterion;
    pub use rstest::{fixture, rstest};
    pub use tracing::{info,warn,error};

    #[fixture]
    pub fn setup() {
        tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();
    }
    
    #[rstest]
    pub fn main_dir(setup:()) {
        let main = Path::new(crate::MAIN_DIR).join("target/x86_64-unknown-linux-gnu/release/libimmt_stex.so");
        info!("main dir: {:?}",main);
        info!("exists: {:?}",main.exists());
    }
}

pub static MAIN_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/..");