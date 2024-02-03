//mod backend;
pub mod backend;

pub mod utils {
    pub mod parsing;
    pub mod problems;

    pub fn measure<R,F:FnOnce() -> R>(prefix:&str,f:F) -> R {
        let start = std::time::Instant::now();
        let r = f();
        tracing::info!("{}: Finished after {:?}",prefix, start.elapsed());
        r
    }
    pub type MMTURI = Box<str>;
}

#[allow(non_camel_case_types)]
#[derive(Debug,Clone,Copy,Hash,PartialEq,Eq)]
pub enum InputFormat {
    sTeX
}
impl InputFormat {
    pub fn from_str(s:&str) -> Option<Self> {
        if s.eq_ignore_ascii_case("stex") { return Some(InputFormat::sTeX) }
        None
    }
}