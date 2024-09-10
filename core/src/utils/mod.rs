use std::fmt::{Debug, Display, Formatter, Write};

pub mod arrayvec {
    pub use arrayvec::*;
}
pub mod ignore_regex;
pub mod filetree;
pub mod logs;
pub mod asyncs;
pub mod settings;


pub struct NestingFormatter<'b,'a:'b> {
    f: &'b mut std::fmt::Formatter<'a>,
    level: usize
}
impl<'b,'a:'b> NestingFormatter<'b,'a> {
    #[inline]
    pub fn inner(&mut self) -> &mut std::fmt::Formatter<'a> {
        self.f
    }
    #[inline]
    pub fn new(f: &'b mut std::fmt::Formatter<'a>) -> Self {
        Self { f, level: 0 }
    }
    #[inline]
    pub fn nest(&mut self,f: impl FnOnce(&mut Self) -> std::fmt::Result) -> std::fmt::Result {
        self.level += 1;
        self.f.write_str(" {")?;
        f(self)?;
        self.level -= 1;
        self.f.write_char('\n')?;
        for _ in 0..self.level {
            self.f.write_str("  ")?;
        }
        self.f.write_char('}')
    }
    #[inline]
    pub fn inc(&mut self) {
        self.level += 1;
    }
    #[inline]
    pub fn dec(&mut self) {
        self.level -= 1;
    }
    #[inline]
    pub fn next(&mut self) -> std::fmt::Result {
        self.f.write_str("\n")?;
        for _ in 0..self.level {
            self.f.write_str("  ")?;
        }
        self.f.write_str("- ")
    }
}

pub trait NestedDisplay {
    fn fmt_nested(&self, f: &mut NestingFormatter) -> std::fmt::Result;

    #[inline]
    fn in_display<'a,'b>(&self, f: &'b mut Formatter<'a>) -> std::fmt::Result where 'a:'b {
        let mut nf = NestingFormatter::new(f);
        let r = self.fmt_nested(&mut nf);
        r
    }
}

pub fn hashstr<A:std::hash::Hash>(a:&A) -> String {
    use std::hash::BuildHasher;
    let h = rustc_hash::FxBuildHasher::default().hash_one(a);
    format!("{:02x}",h)
}