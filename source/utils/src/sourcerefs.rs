use std::fmt::{Debug, Display};
use std::num::NonZeroUsize;

pub trait SourcePos: Clone + Default + Debug {
    fn update(&mut self, c: char);
    fn update_newline(&mut self, rn: bool);
    fn update_str_no_newline(&mut self, s: &str);
    fn update_str_maybe_newline(&mut self, s: &str);
}
impl SourcePos for () {
    #[inline]
    fn update(&mut self, _: char) {}
    #[inline]
    fn update_newline(&mut self, _: bool) {}
    #[inline]
    fn update_str_no_newline(&mut self, _: &str) {}
    #[inline]
    fn update_str_maybe_newline(&mut self, _: &str) {}
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ByteOffset {
    pub offset: usize,
}
impl Display for ByteOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.offset)
    }
}
impl Debug for ByteOffset {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}
impl SourcePos for ByteOffset {
    #[inline]
    fn update(&mut self, c: char) {
        self.offset += c.len_utf8();
    }
    #[inline]
    fn update_newline(&mut self, rn: bool) {
        self.offset += if rn { 2 } else { 1 };
    }
    #[inline]
    fn update_str_no_newline(&mut self, s: &str) {
        self.offset += s.len();
    }
    #[inline]
    fn update_str_maybe_newline(&mut self, s: &str) {
        self.update_str_no_newline(s);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceOffsetLineCol {
    pub line: NonZeroUsize,
    pub col: NonZeroUsize,
}
impl Default for SourceOffsetLineCol {
    fn default() -> Self {
        Self {
            line: NonZeroUsize::new(1).unwrap_or_else(|| unreachable!()),
            col: NonZeroUsize::new(1).unwrap_or_else(|| unreachable!()),
        }
    }
}
impl Display for SourceOffsetLineCol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "l. {} c. {}", self.line, self.col)
    }
}
impl Debug for SourceOffsetLineCol {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl SourcePos for SourceOffsetLineCol {
    #[inline]
    fn update(&mut self, _: char) {
        self.col = self.col.saturating_add(1);
    }
    #[inline]
    fn update_newline(&mut self, _: bool) {
        self.line = self.line.saturating_add(1);
        self.col = NonZeroUsize::new(1).unwrap_or_else(|| unreachable!());
    }
    #[inline]
    fn update_str_no_newline(&mut self, s: &str) {
        self.col = self.col.saturating_add(s.chars().count());
    }

    fn update_str_maybe_newline(&mut self, s: &str) {
        let s = s.split("\r\n").flat_map(|s| s.split(['\n', '\r']));
        let mut had_newline = false;
        for l in s {
            if had_newline {
                self.line = self.line.saturating_add(1);
                self.col = NonZeroUsize::new(1).unwrap_or_else(|| unreachable!());
            }
            self.col = self.col.saturating_add(l.chars().count());
            had_newline = true;
        }
    }
}

impl<A: SourcePos, B: SourcePos> SourcePos for (A, B) {
    #[inline]
    fn update(&mut self, c: char) {
        self.0.update(c);
        self.1.update(c);
    }
    #[inline]
    fn update_newline(&mut self, rn: bool) {
        self.0.update_newline(rn);
        self.1.update_newline(rn);
    }
    #[inline]
    fn update_str_no_newline(&mut self, s: &str) {
        self.0.update_str_no_newline(s);
        self.1.update_str_no_newline(s);
    }
    #[inline]
    fn update_str_maybe_newline(&mut self, s: &str) {
        self.0.update_str_maybe_newline(s);
        self.1.update_str_maybe_newline(s);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceRange<P: SourcePos> {
    pub start: P,
    pub end: P,
}
impl Display for SourceRange<ByteOffset> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}
impl Debug for SourceRange<ByteOffset> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}
