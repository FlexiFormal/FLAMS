use std::fmt::{Debug, Display};
use std::io::{Read};
use crate::utils::sourcerefs::SourcePos;

pub trait StringOrStr<'a>:AsRef<str> + From<&'a str> + Debug + Display + Eq + std::hash::Hash +
    Clone + for<'b> PartialEq<&'b str> {
    fn strip_prefix(self,s:&str) -> Result<Self,Self>;
}
impl<'a> StringOrStr<'a> for &'a str {
    fn strip_prefix(self,s: &str) -> Result<Self, Self> {
        str::strip_prefix(self,s).map(|s| s.trim_start()).ok_or(self)
    }
}
impl<'a> StringOrStr<'a> for String {
    fn strip_prefix(self, s: &str) -> Result<Self, Self> {
        match str::strip_prefix(&self,s) {
            Some(s) => Ok(s.trim_start().to_string()),
            None => Err(self)
        }
    }
}

pub trait ParseSource<'a>:'a {
    type Pos:SourcePos;
    type Str:StringOrStr<'a>;
    fn curr_pos(&self) -> &Self::Pos;
    fn pop_head(&mut self) -> Option<char>;
    fn read_until_line_end(&mut self) -> (Self::Str,Self::Pos);
    fn trim_start(&mut self);
    fn starts_with(&mut self,c:char) -> bool;
    fn peek_head(&mut self) -> Option<char>;
    fn read_n(&mut self,i:usize) -> Self::Str;
    fn read_while(&mut self,pred:impl FnMut(char) -> bool) -> Self::Str;
    #[inline]
    fn read_until(&mut self,mut pred:impl FnMut(char) -> bool) -> Self::Str {
        self.read_while(|c| !pred(c))
    }
    fn read_until_str(&mut self,s:&str) -> Self::Str;
    fn read_until_with_brackets<const OPEN:char,const CLOSE:char>(&mut self,pred:impl FnMut(char) -> bool) -> Self::Str;
}

pub struct ParseReader<R:Read,P:SourcePos> {
    inner:R,
    buf:Vec<char>,
    pos:P
}
impl<R:Read,P:SourcePos> ParseReader<R,P> {
    pub fn new(inner:R) -> Self {
        Self { inner, buf:Vec::new(), pos:P::default() }
    }
}

impl<'a,R:Read+'a,P:SourcePos+'a> ParseSource<'a> for ParseReader<R,P> {
    type Pos = P;
    type Str = String;
    #[inline(always)]
    fn curr_pos(&self) -> &P { &self.pos }
    fn pop_head(&mut self) -> Option<char> {
        match self.get_char() {
            Some('\n') => {
                self.pos.update_newline(false);
                Some('\n')
            },
            Some('\r') => {
                match self.get_char() {
                    Some('\n') => {
                        self.pos.update_newline(true);
                        Some('\n')
                    },
                    Some(c) => {
                        self.pos.update_newline(false);
                        self.push_char(c);
                        Some(c)
                    },
                    None => {
                        self.pos.update_newline(false);
                        Some('\n')
                    }
                }
            }
            Some(c) => {
                self.pos.update(c);
                Some(c)
            },
            None => None
        }
    }
    fn read_until_line_end(&mut self) -> (String,P) {
        let (s,rn) = self.find_line_end();
        self.pos.update_str_no_newline(&s);
        let pos = self.pos.clone();
        if let Some(rn) = rn {self.pos.update_newline(rn)}
        (s,pos)
    }
    fn trim_start(&mut self) {
        while let Some(c) = self.get_char() {
            if c == '\n' {
                self.pos.update_newline(false);
            } else if c == '\r' {
                match self.get_char() {
                    Some('\n') => {
                        self.pos.update_newline(true);
                    },
                    Some(c) => {
                        self.push_char(c);
                        self.pos.update_newline(false);
                    },
                    None => {
                        self.pos.update_newline(false);
                        break
                    }
                }
            } else if c.is_whitespace() {
                self.pos.update(c);
            } else {
                self.push_char(c);
                break
            }
        }
    }
    fn starts_with(&mut self, c: char) -> bool {
        if let Some(c2) = self.get_char() {
            self.push_char(c2);
            c2 == c
        } else { false }
    }
    fn read_while(&mut self, mut pred: impl FnMut(char) -> bool) -> Self::Str  {
        let mut ret = String::new();
        while let Some(c) = self.get_char() {
            if !pred(c) {
                self.push_char(c);
                break
            }
            self.pos.update(c);
            ret.push(c);
        }
        ret
    }
    fn read_until_with_brackets<const OPEN: char, const CLOSE: char>(&mut self, mut pred: impl FnMut(char) -> bool) -> Self::Str {
        let mut ret = String::new();
        let mut depth = 0;
        while let Some(c) = self.get_char() {
            if c == OPEN {
                depth += 1;
                self.pos.update(c);
                ret.push(c);
                continue
            } else if c == CLOSE && depth > 0 {
                depth -= 1;
                self.pos.update(c);
                ret.push(c);
                continue
            } else if depth > 0 {
                self.pos.update(c);
                ret.push(c);
                continue
            }
            if pred(c) {
                self.push_char(c);
                break
            }
            self.pos.update(c);
            ret.push(c);
        }
        ret
    }
    fn peek_head(&mut self) -> Option<char> {
        self.get_char().map(|c| {self.push_char(c);c})
    }
    fn read_n(&mut self, i: usize) -> Self::Str {
        let mut ret = String::new();
        for _ in 0..i {
            if let Some(c) = self.pop_head() {
                ret.push(c);
            } else { break }
        }
        ret
    }
    fn read_until_str(&mut self, s: &str) -> Self::Str {
        let mut ret = String::new();
        loop {
            if let Some(c) = self.pop_head() {
                ret.push(c);
            } else {
                break
            }
            if ret.ends_with(s) {
                for _ in 0..s.len() { self.push_char(ret.pop().unwrap()); }
                return ret
            }
        }
        ret
    }
}

impl<R:Read,P:SourcePos> ParseReader<R,P> {
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
            let mut buf = [byte, 0,0];
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
    fn push_char(&mut self,c:char) {
        self.buf.push(c);
    }
    fn char_from_utf8(buf:&[u8]) -> Option<char> {
        std::str::from_utf8(buf).ok().and_then(|s| s.chars().next())
    }
    fn find_line_end(&mut self) -> (String,Option<bool>) {
        let mut ret = String::new();
        while let Some(c) = self.get_char() {
            if c == '\n' {
                return (ret, Some(false))
            }
            if c == '\r' {
                match self.get_char() {
                    Some('\n') => {
                        return (ret, Some(true))
                    },
                    Some(c) => self.push_char(c),
                    None => ()
                }
                return (ret, Some(true))
            }
            ret.push(c);
        }
        (ret, None)
    }
}

pub struct ParseStr<'a,P:SourcePos> {
    input:&'a str,
    pos:P
}
impl<'a,P:SourcePos> ParseStr<'a,P> {
    pub fn new(input:&'a str) -> Self {
        Self { input, pos:P::default() }
    }
}

impl<'a,P:SourcePos+'a> ParseSource<'a> for ParseStr<'a,P> {
    type Pos = P;
    type Str = &'a str;
    #[inline(always)]
    fn curr_pos(&self) -> &P { &self.pos }
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
                    Some('\n')
                } else {
                    self.input = &self.input[1..];
                    self.pos.update_newline(false);
                    Some('\n')
                }
            } else {
                self.pos.update(c);
                self.input = &self.input[c.len_utf8()..];
                Some(c)
            }
        } else { None }
    }
    fn read_until_line_end(&mut self) -> (&'a str, P) {
        if let Some(i) = self.input.find(['\r','\n']) {
            if self.input.as_bytes()[i] == b'\r' &&
                self.input.as_bytes().get(i+1) == Some(&b'\n') {
                let (l,r) = self.input.split_at(i);
                self.input = &r[2..];
                self.pos.update_str_no_newline(l);
                let pos = self.pos.clone();
                self.pos.update_newline(true);
                return (l, pos)
            }
            let (l,r) = self.input.split_at(i);
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
            } else { break }
        }
    }
    fn starts_with(&mut self, c: char) -> bool {
        self.input.starts_with(c)
    }
    fn read_while(&mut self, mut pred: impl FnMut(char) -> bool) -> Self::Str {
        let i = self.input.find(|c| !pred(c)).unwrap_or(self.input.len());
        let (l,r) = self.input.split_at(i);
        self.input = r;
        self.pos.update_str_maybe_newline(l);
        l
    }
    fn read_until_with_brackets<const OPEN: char, const CLOSE: char>(&mut self, mut pred: impl FnMut(char) -> bool) -> Self::Str {
        let mut depth = 0;
        let i = self.input.find(|c| {
            if c == OPEN {
                depth += 1;
                false
            } else if c == CLOSE && depth > 0 {
                depth -= 1;
                false
            } else {
                depth == 0 && pred(c)
            }
        }).unwrap_or(self.input.len());
        let (l,r) = self.input.split_at(i);
        self.input = r;
        self.pos.update_str_maybe_newline(l);
        l
    }
    fn peek_head(&mut self) -> Option<char> {
        self.input.chars().next()
    }
    fn read_n(&mut self, i: usize) -> Self::Str {
        let (l,r) = self.input.split_at(i);
        self.input = r;
        self.pos.update_str_maybe_newline(l);
        l
    }
    fn read_until_str(&mut self, s: &str) -> Self::Str {
        if let Some(i) = self.input.find(s) {
            let (l,r) = self.input.split_at(i);
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
}

// -----------------------------------------------------------------------------
/*
struct PeekWrapper<R:Read> {
    inner:R,//BufReader<R>,
    buf:Vec<char>
}



pub trait SourcePos:Clone+Default+std::fmt::Debug {
    fn step_str(&mut self, s:&mut &str) -> Option<char>;
    fn step_read(&mut self,r:&mut PeekWrapper<impl Read>) -> Option<char>;

    /// &str is the line that was read; `Self` is the position of the endline
    /// For a SourceOffset, this is self.offset - (1|2), depending on the line ending
    fn step_until_line_end_str<'a>(&mut self, s:&mut &'a str) -> (&'a str, Self);
    fn step_until_line_end_read(&mut self, r:&mut PeekWrapper<impl Read>) -> (String, Self);
}

fn find_line_end<'a>(s:&mut &'a str) -> Option<(&'a str,usize,bool)> {
    let mut wasr = false;
    s.find(|c| c == '\n' || (c == '\r' && {wasr = true;true}) ).map(|i| {
        let (l,r) = s.split_at(i);
        *s = &r[1..];
        if wasr && !r.is_empty() && r.as_bytes()[0] == b'\n' {
            *s = &r[1..];
            (l,i+2,true)
        } else { (l,i+1,false) }
    })
}

impl SourcePos for () {
    #[inline]
    fn step_str(&mut self, s:&mut &str) -> Option<char> {
        s.chars().next().map(|c|{
            *s = &s[c.len_utf8()..];
            c
        })
    }
    fn step_read(&mut self, r: &mut PeekWrapper<impl Read>) -> Option<char> {
        r.get_char()
    }
    fn step_until_line_end_str<'a>(&mut self, s: &mut &'a str) -> (&'a str, Self) {
        match find_line_end(s) {
            None => {
                let l = *s;
                *s = "";
                (l,())
            },
            Some((l,i,_)) => {
                *s = &s[i..];
                (l,())
            }
        }
    }
    fn step_until_line_end_read(&mut self, r: &mut PeekWrapper<impl Read>) -> (String, Self) {
        (r.find_line_end().0,())
    }
}


impl ByteOffset {
    #[inline]
    fn step(&mut self,c:char,s:Option<&mut &str>) -> char {
        let len = c.len_utf8();
        self.offset += len;
        if let Some(s) = s { *s = &s[len..];}
        c
    }
}
impl SourcePos for ByteOffset {
    fn step_str(&mut self, s:&mut &str) -> Option<char> {
        s.chars().next().map(|c|{
            self.step(c,Some(s))
        })
    }
    fn step_read(&mut self, r: &mut PeekWrapper<impl Read>) -> Option<char> {
        r.get_char().map(|c|{
            self.step(c,None)
        })
    }
    fn step_until_line_end_str<'a>(&mut self, s: &mut &'a str) -> (&'a str, Self) {
        match find_line_end(s) {
            None => {
                let l = *s;
                self.offset += s.len();*s = "";
                (l,*self)
            },
            Some((l,i,rn)) => {
                self.offset += i;
                (l,Self{ offset:self.offset - if rn {2} else {1} })
            }
        }
    }
    fn step_until_line_end_read(&mut self, r: &mut PeekWrapper<impl Read>) -> (String, Self) {
        let (l,i,rn) = r.find_line_end();
        self.offset += i;
        (l,Self{ offset:self.offset - if rn {2} else {1} })
    }
}

impl SourcePos for SourceOffsetLineCol {
    fn step_str(&mut self, s:&mut &str) -> Option<char> {
        let mut cs = s.chars();
        match cs.next() {
            None => None,
            Some('\n') => {
                self.line += 1;self.col = 0;self.offset += 1;*s = &s[1..];
                Some('\n')
            }
            Some('\r') => {
                self.line += 1;self.col = 0;self.offset += 1;*s = &s[1..];
                if cs.next() == Some('\n') {
                    self.offset += 1;*s = &s[1..];
                }
                Some('\n')
            }
            Some(c) => {
                self.col += 1;self.offset += c.len_utf8();*s = &s[c.len_utf8()..];
                Some(c)
            }
        }
    }
    fn step_read(&mut self, r: &mut PeekWrapper<impl Read>) -> Option<char> {
        match r.get_char() {
            None => None,
            Some('\n') => {
                self.line += 1;self.col = 0;self.offset += 1;
                Some('\n')
            }
            Some('\r') => {
                self.line += 1;self.col = 0;self.offset += 1;
                match r.get_char() {
                    Some('\n') => self.offset += 1,
                    Some(c) => r.push_char(c),
                    None => {}
                }
                Some('\n')
            }
            Some(c) => {
                self.col += 1;self.offset += c.len_utf8();
                Some(c)
            }
        }
    }
    fn step_until_line_end_str<'a>(&mut self, s: &mut &'a str) -> (&'a str, Self) {
        match find_line_end(s) {
            None => {
                let l = *s;
                self.offset += s.len();self.col += s.chars().count();*s = "";
                (l,*self)
            },
            Some((l,i,rn)) => {
                let off = Self {
                    offset:self.offset + i - if rn { 2 } else { 1 },
                    col:self.col,line:self.line
                };
                self.offset += i;self.col = 0;self.line += 1;
                (l, off)
            }
        }
    }
    fn step_until_line_end_read(&mut self, r: &mut PeekWrapper<impl Read>) -> (String, Self) {
        let (l,i,rn) = r.find_line_end();
        let off = Self {
            offset:self.offset + i - if rn { 2 } else { 1 },
            col:self.col,line:self.line
        };
        self.offset += i;self.col = 0;self.line += 1;
        (l, off)
    }
}

pub trait ParseReader<P:SourcePos> {
    type Str;
    fn curr_pos(&self) -> &P;
    fn pop_head(&mut self) -> Option<char>;
    fn read_until_line_end(&mut self) -> (Self::Str,P);
    fn trim_start(&mut self);
    fn starts_with(&mut self,c:char) -> bool;
}

pub struct ReadReader<P:SourcePos,R:Read> {
    curr_pos:P,
    input:PeekWrapper<R>
}
impl <P:SourcePos,R:Read> ParseReader<P> for ReadReader<P,R> {
    type Str = String;
    #[inline]
    fn curr_pos(&self) -> &P { &self.curr_pos }
    #[inline]
    fn pop_head(&mut self) -> Option<char> {
        self.curr_pos.step_read(&mut self.input)
    }
    #[inline]
    fn read_until_line_end(&mut self) -> (String,P) {
        self.curr_pos.step_until_line_end_read(&mut self.input)
    }
    fn trim_start(&mut self) {
        TODOOO
    }
}
impl <P:SourcePos,R:std::io::Read> ReadReader<P,R> {
    pub fn new(input:R) -> Self {
        Self { curr_pos:P::default(), input:PeekWrapper{inner:input,buf:Vec::new()} }
    }
}

pub struct StrReader<'a,P:SourcePos> {
    input:&'a str,
    curr_pos:P
}

impl<'a,P:SourcePos> ParseReader<P> for StrReader<'a,P> {
    type Str = &'a str;
    #[inline]
    fn curr_pos(&self) -> &P { &self.curr_pos }
    #[inline]
    fn pop_head(&mut self) -> Option<char> {
        self.curr_pos.step_str(&mut self.input)
    }
    #[inline]
    fn read_until_line_end(&mut self) -> (&'a str,P) {
        self.curr_pos.step_until_line_end_str(&mut self.input)
    }
    #[inline]
    fn trim_start(&mut self) {
        self.read_while(|c| c.is_whitespace());
    }
    #[inline]
    fn starts_with(&mut self,c:char) -> bool { self.input.starts_with(c) }
}

impl<'a,P:SourcePos> StrReader<'a, P> {
    #[inline(always)]
    pub fn new(input:&'a str) -> Self {
        Self { input, curr_pos:P::default() }
    }

    pub fn read_until<Pr:FnMut(char) -> bool>(&mut self,mut p:Pr) -> &'a str {
        let curr = self.input;
        let mut i = 0;
        while let Some(c) = self.input.chars().next() {
            if p(c) { break }
            let c = self.curr_pos.step_str(&mut self.input).unwrap();
            i += c.len_utf8();
        }
        &curr[..i]
    }

    pub fn read_until_str<Pr:FnMut(&'a str) -> bool>(&mut self,mut p:Pr) -> &'a str {
        let curr = self.input;
        let mut i = 0;
        while !p(self.input) {
            match self.curr_pos.step_str(&mut self.input) {
                None => break,
                Some(c) => i += c.len_utf8()
            }
        }
        &curr[..i]
    }

    #[inline]
    pub fn read_while<Pr:FnMut(char) -> bool>(&mut self,mut p:Pr) -> &'a str {
        self.read_until_str(|s| !s.starts_with(&mut p))
    }
}

 */