use std::fmt::{Debug, Display};

pub trait SourcePos: Clone + Copy + Default + Debug + PartialOrd + Ord + 'static {
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

#[derive(Clone, Copy, PartialEq, Eq, Default,PartialOrd,Ord)]
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

#[derive(Clone, Copy, PartialEq, Eq,Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LSPLineCol {
    pub line: u32,
    pub col: u32,
}
impl Ord for LSPLineCol {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.line.cmp(&other.line).then(self.col.cmp(&other.col))
    }
}
impl PartialOrd for LSPLineCol {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for LSPLineCol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "l. {} c. {}", self.line, self.col)
    }
}
impl Debug for LSPLineCol {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl SourcePos for LSPLineCol {
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn update(&mut self, c: char) {
        self.col += c.len_utf16() as u32;
    }
    #[inline]
    fn update_newline(&mut self, _: bool) {
        self.line += 1;
        self.col = 0;
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn update_str_no_newline(&mut self, s: &str) {
        self.col += s.chars().map(|c| char::len_utf16(c) as u32).sum::<u32>();
    }

    #[allow(clippy::cast_possible_truncation)]
    fn update_str_maybe_newline(&mut self, s: &str) {
        let s = s.split("\r\n").flat_map(|s| s.split(['\n', '\r']));
        let mut last = "";
        let mut first = true;
        for l in s {
            if first {
                first = false;
            } else {
                self.line += 1;
                self.col = 0;
            }
            last = l;
        }
        self.col += last.chars().map(|c| char::len_utf16(c) as u32).sum::<u32>();
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
impl<P: SourcePos> Display for SourceRange<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:#?}-{:#?}", self.start, self.end)
    }
}
impl<P: SourcePos> Debug for SourceRange<P> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}
impl<P:SourcePos> SourceRange<P> {
    pub fn contains(&self, pos: P) -> bool {
        self.start <= pos && pos <= self.end
    }
}

#[test]
fn test() {
    let str = "\n\n";
    let len = str.split("\r\n").flat_map(|s| s.split(['\r', '\n'])).count();
    assert_eq!(len, 3);
}