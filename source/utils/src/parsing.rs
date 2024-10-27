use crate::sourcerefs::{ByteOffset, SourcePos};
use std::fmt::{Debug, Display};
use std::io::Read;

pub trait StringOrStr<'a>:
    AsRef<str>
    + From<&'a str>
    + Debug
    + Display
    + Eq
    + std::hash::Hash
    + Clone
    + for<'b> PartialEq<&'b str>
{
    /// # Errors
    ///
    /// Will return `Err` if self does not start with prefix.
    fn strip_prefix(self, s: &str) -> Result<Self, Self>;
    fn split_noparens<const OPEN: char, const CLOSE: char>(
        &'a self,
        split_char: char,
    ) -> impl Iterator<Item = &'a str>;
}
impl<'a> StringOrStr<'a> for &'a str {
    fn strip_prefix(self, s: &str) -> Result<Self, Self> {
        str::strip_prefix(self, s).map(str::trim_start).ok_or(self)
    }
    fn split_noparens<const OPEN: char, const CLOSE: char>(
        &'a self,
        split_char: char,
    ) -> impl Iterator<Item = &'a str> {
        let mut depth = 0;
        self.split(move |c: char| {
            if c == OPEN {
                depth += 1;
                false
            } else if c == CLOSE && depth > 0 {
                depth -= 1;
                false
            } else if depth > 0 {
                false
            } else {
                c == split_char
            }
        })
    }
}
impl<'a> StringOrStr<'a> for String {
    #[allow(clippy::option_if_let_else)]
    fn strip_prefix(self, s: &str) -> Result<Self, Self> {
        match str::strip_prefix(&self, s) {
            Some(s) => Ok(s.trim_start().to_string()),
            None => Err(self),
        }
    }
    fn split_noparens<const OPEN: char, const CLOSE: char>(
        &'a self,
        split_char: char,
    ) -> impl Iterator<Item = &'a str> {
        let mut depth = 0;
        self.split(move |c: char| {
            if c == OPEN {
                depth += 1;
                false
            } else if c == CLOSE && depth > 0 {
                depth -= 1;
                false
            } else if depth > 0 {
                false
            } else {
                c == split_char
            }
        })
    }
}

pub trait ParseSource<'a>: 'a {
    type Pos: SourcePos;
    type Str: StringOrStr<'a>;
    fn curr_pos(&self) -> &Self::Pos;
    fn pop_head(&mut self) -> Option<char>;
    fn read_until_line_end(&mut self) -> (Self::Str, Self::Pos);
    fn trim_start(&mut self);
    fn starts_with(&mut self, c: char) -> bool;
    fn peek_head(&mut self) -> Option<char>;
    fn read_n(&mut self, i: usize) -> Self::Str;
    fn read_while(&mut self, pred: impl FnMut(char) -> bool) -> Self::Str;
    #[inline]
    fn read_until(&mut self, mut pred: impl FnMut(char) -> bool) -> Self::Str {
        self.read_while(|c| !pred(c))
    }
    fn read_until_str(&mut self, s: &str) -> Self::Str;
    fn read_until_with_brackets<const OPEN: char, const CLOSE: char>(
        &mut self,
        pred: impl FnMut(char) -> bool,
    ) -> Self::Str;
    fn skip(&mut self, i: usize);
}

pub struct ParseReader<R: Read, P: SourcePos> {
    inner: R,
    buf: Vec<char>,
    pos: P,
}
impl<R: Read, P: SourcePos> ParseReader<R, P> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            buf: Vec::new(),
            pos: P::default(),
        }
    }
}

impl<'a, R: Read + 'a, P: SourcePos + 'a> ParseSource<'a> for ParseReader<R, P> {
    type Pos = P;
    type Str = String;
    #[inline]
    fn curr_pos(&self) -> &P {
        &self.pos
    }
    fn skip(&mut self, i: usize) {
        for _ in 0..i {
            self.pop_head();
        }
    }
    fn pop_head(&mut self) -> Option<char> {
        match self.get_char() {
            Some('\n') => {
                self.pos.update_newline(false);
                Some('\n')
            }
            Some('\r') => { 
                match self.get_char() {
                    Some('\n') => {
                        self.pos.update_newline(true);
                    }
                    Some(c) => {
                        self.pos.update_newline(false);
                        self.push_char(c);
                    }
                    None => {
                        self.pos.update_newline(false);
                    }
                }
                Some('\n')
            }
            Some(c) => {
                self.pos.update(c);
                Some(c)
            }
            None => None,
        }
    }
    fn read_until_line_end(&mut self) -> (String, P) {
        let (s, rn) = self.find_line_end();
        self.pos.update_str_no_newline(&s);
        let pos = self.pos.clone();
        if let Some(rn) = rn {
            self.pos.update_newline(rn);
        }
        (s, pos)
    }
    fn trim_start(&mut self) {
        while let Some(c) = self.get_char() {
            if c == '\n' {
                self.pos.update_newline(false);
            } else if c == '\r' {
                match self.get_char() {
                    Some('\n') => {
                        self.pos.update_newline(true);
                    }
                    Some(c) => {
                        self.push_char(c);
                        self.pos.update_newline(false);
                    }
                    None => {
                        self.pos.update_newline(false);
                        break;
                    }
                }
            } else if c.is_whitespace() {
                self.pos.update(c);
            } else {
                self.push_char(c);
                break;
            }
        }
    }
    fn starts_with(&mut self, c: char) -> bool {
        self.get_char().map_or(false, |c2| {
            self.push_char(c2);
            c2 == c
        })
    }
    fn read_while(&mut self, mut pred: impl FnMut(char) -> bool) -> Self::Str {
        let mut ret = String::new();
        while let Some(c) = self.get_char() {
            if !pred(c) {
                self.push_char(c);
                break;
            }
            self.pos.update(c);
            ret.push(c);
        }
        ret
    }
    fn read_until_with_brackets<const OPEN: char, const CLOSE: char>(
        &mut self,
        mut pred: impl FnMut(char) -> bool,
    ) -> Self::Str {
        let mut ret = String::new();
        let mut depth = 0;
        while let Some(c) = self.get_char() {
            if c == OPEN {
                depth += 1;
                self.pos.update(c);
                ret.push(c);
                continue;
            } else if c == CLOSE && depth > 0 {
                depth -= 1;
                self.pos.update(c);
                ret.push(c);
                continue;
            } else if depth > 0 {
                self.pos.update(c);
                ret.push(c);
                continue;
            }
            if pred(c) {
                self.push_char(c);
                break;
            }
            self.pos.update(c);
            ret.push(c);
        }
        ret
    }
    fn peek_head(&mut self) -> Option<char> {
        self.get_char().inspect(|c| {
            self.push_char(*c);
        })
    }
    fn read_n(&mut self, i: usize) -> Self::Str {
        let mut ret = String::new();
        for _ in 0..i {
            if let Some(c) = self.pop_head() {
                ret.push(c);
            } else {
                break;
            }
        }
        ret
    }
    fn read_until_str(&mut self, s: &str) -> Self::Str {
        let mut ret = String::new();
        while let Some(c) = self.pop_head() {
            ret.push(c);
            if ret.ends_with(s) {
                for _ in 0..s.len() {
                    self.push_char(ret.pop().unwrap_or_else(|| unreachable!()));
                }
                return ret;
            }
        }
        ret
    }
}

impl<R: Read, P: SourcePos> ParseReader<R, P> {
    fn get_char(&mut self) -> Option<char> {
        self.buf.pop().or_else(|| self.read_char())
    }
    fn read_char(&mut self) -> Option<char> {
        let mut byte = [0u8];
        self.inner.read_exact(&mut byte).ok()?;
        let byte = byte[0];
        if byte & 224u8 == 192u8 {
            // a two byte unicode character
            let mut buf = [byte, 0];
            self.inner.read_exact(&mut buf[1..]).ok()?;
            Self::char_from_utf8(&buf)
        } else if byte & 240u8 == 224u8 {
            // a three byte unicode character
            let mut buf = [byte, 0, 0];
            self.inner.read_exact(&mut buf[1..]).ok()?;
            Self::char_from_utf8(&buf)
        } else if byte & 248u8 == 240u8 {
            // a four byte unicode character
            let mut buf = [byte, 0, 0, 0];
            self.inner.read_exact(&mut buf[1..]).ok()?;
            Self::char_from_utf8(&buf)
        } else {
            Some(byte as char)
        }
    }
    fn push_char(&mut self, c: char) {
        self.buf.push(c);
    }
    fn char_from_utf8(buf: &[u8]) -> Option<char> {
        std::str::from_utf8(buf).ok().and_then(|s| s.chars().next())
    }
    fn find_line_end(&mut self) -> (String, Option<bool>) {
        let mut ret = String::new();
        while let Some(c) = self.get_char() {
            if c == '\n' {
                return (ret, Some(false));
            }
            if c == '\r' {
                match self.get_char() {
                    Some('\n') => return (ret, Some(true)),
                    Some(c) => self.push_char(c),
                    None => (),
                }
                return (ret, Some(true));
            }
            ret.push(c);
        }
        (ret, None)
    }
}

pub struct ParseStr<'a, P: SourcePos> {
    input: &'a str,
    pos: P,
}
impl<'a, P: SourcePos> ParseStr<'a, P> {
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: P::default(),
        }
    }
    #[inline]
    pub fn starts_with_str(&self, s: &str) -> bool {
        self.input.starts_with(s)
    }
    #[inline]
    pub const fn rest(&self) -> &'a str {
        self.input
    }

    pub fn read_until_inclusive(&mut self, pred: impl FnMut(char) -> bool) -> &'a str {
        let i = self.input.find(pred).unwrap_or(self.input.len());
        let (l, r) = self.input.split_at(i + 1);
        self.input = r;
        self.pos.update_str_maybe_newline(l);
        l
    }
    pub fn drop_prefix(&mut self, s: &str) -> bool {
        self.input.starts_with(s) && {
            self.input = &self.input[s.len()..];
            self.pos.update_str_maybe_newline(s);
            true
        }
    }
    pub fn read_until_escaped(&mut self, find: char, escape: char) -> &'a str {
        let mut chars = self.input.chars();
        let mut i: usize = 0;
        while let Some(c) = chars.next() {
            if c == escape {
                if let Some(c) = chars.next() {
                    i += c.len_utf8();
                }
            } else if c == find {
                let (l, r) = self.input.split_at(i);
                self.input = r;
                self.pos.update_str_maybe_newline(l);
                return l;
            }
            i += c.len_utf8();
        }
        let ret = self.input;
        self.input = "";
        self.pos.update_str_maybe_newline(ret);
        ret
    }
}
impl<'a> ParseStr<'a, ByteOffset> {
    #[inline]
    pub fn offset(&mut self) -> &mut ByteOffset {
        &mut self.pos
    }
}

impl<'a, P: SourcePos + 'a> ParseSource<'a> for ParseStr<'a, P> {
    type Pos = P;
    type Str = &'a str;
    #[inline]
    fn curr_pos(&self) -> &P {
        &self.pos
    }
    fn pop_head(&mut self) -> Option<char> {
        if let Some(c) = self.input.chars().next() {
            if c == '\n' {
                self.pos.update_newline(false);
                self.input = &self.input[1..];
                Some('\n')
            } else if c == '\r' {
                if self.input.chars().nth(1) == Some('\n') {
                    self.input = &self.input[2..];
                    self.pos.update_newline(true);
                } else {
                    self.input = &self.input[1..];
                    self.pos.update_newline(false);
                }
                Some('\n')
            } else {
                self.pos.update(c);
                self.input = &self.input[c.len_utf8()..];
                Some(c)
            }
        } else {
            None
        }
    }
    fn read_until_line_end(&mut self) -> (&'a str, P) {
        if let Some(i) = self.input.find(['\r', '\n']) {
            if self.input.as_bytes()[i] == b'\r' && self.input.as_bytes().get(i + 1) == Some(&b'\n')
            {
                let (l, r) = self.input.split_at(i);
                self.input = &r[2..];
                self.pos.update_str_no_newline(l);
                let pos = self.pos.clone();
                self.pos.update_newline(true);
                return (l, pos);
            }
            let (l, r) = self.input.split_at(i);
            self.input = &r[1..];
            self.pos.update_str_no_newline(l);
            let pos = self.pos.clone();
            self.pos.update_newline(false);
            (l, pos)
        } else {
            let ret = self.input;
            self.pos.update_str_no_newline(ret);
            self.input = "";
            (ret, self.pos.clone())
        }
    }
    fn trim_start(&mut self) {
        while let Some(c) = self.input.chars().next() {
            if c == '\n' {
                self.input = &self.input[1..];
                self.pos.update_newline(false);
            } else if c == '\r' {
                self.input = &self.input[1..];
                if self.input.starts_with('\n') {
                    self.input = &self.input[1..];
                    self.pos.update_newline(true);
                } else {
                    self.pos.update_newline(false);
                }
            } else if c.is_whitespace() {
                self.input = &self.input[c.len_utf8()..];
                self.pos.update(c);
            } else {
                break;
            }
        }
    }
    fn starts_with(&mut self, c: char) -> bool {
        self.input.starts_with(c)
    }
    fn read_while(&mut self, mut pred: impl FnMut(char) -> bool) -> Self::Str {
        let i = self.input.find(|c| !pred(c)).unwrap_or(self.input.len());
        let (l, r) = self.input.split_at(i);
        self.input = r;
        self.pos.update_str_maybe_newline(l);
        l
    }
    fn read_until_with_brackets<const OPEN: char, const CLOSE: char>(
        &mut self,
        mut pred: impl FnMut(char) -> bool,
    ) -> Self::Str {
        let mut depth = 0;
        let i = self
            .input
            .find(|c| {
                if c == OPEN {
                    depth += 1;
                    false
                } else if c == CLOSE && depth > 0 {
                    depth -= 1;
                    false
                } else {
                    depth == 0 && pred(c)
                }
            })
            .unwrap_or(self.input.len());
        let (l, r) = self.input.split_at(i);
        self.input = r;
        self.pos.update_str_maybe_newline(l);
        l
    }
    fn peek_head(&mut self) -> Option<char> {
        self.input.chars().next()
    }
    fn read_n(&mut self, i: usize) -> Self::Str {
        let (l, r) = self.input.split_at(i);
        self.input = r;
        self.pos.update_str_maybe_newline(l);
        l
    }
    fn read_until_str(&mut self, s: &str) -> Self::Str {
        if let Some(i) = self.input.find(s) {
            let (l, r) = self.input.split_at(i);
            self.input = r;
            self.pos.update_str_maybe_newline(l);
            l
        } else {
            let ret = self.input;
            self.input = "";
            self.pos.update_str_maybe_newline(ret);
            ret
        }
    }
    fn skip(&mut self, i: usize) {
        let (a, b) = self.input.split_at(i);
        self.input = b;
        self.pos.update_str_maybe_newline(a);
    }
}
