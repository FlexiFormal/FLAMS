use std::fmt::{Debug, Display};

pub trait SourcePos:Clone+Default+std::fmt::Debug {
    fn step(&mut self,s:&mut &str) -> Option<char>;
    /// &str is the line that was read; `Self` is the position of the endline
    /// For a SourceOffset, this is self.offset - (1|2), depending on the line ending
    fn step_until_line_end<'a>(&mut self,s:&mut &'a str) -> (&'a str,Self);
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
    fn step(&mut self,s:&mut &str) -> Option<char> {
        s.chars().next().map(|c|{
            *s = &s[c.len_utf8()..];
            c
        })
    }
    fn step_until_line_end<'a>(&mut self, s: &mut &'a str) -> (&'a str,Self) {
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
}

#[derive(Clone,Copy,PartialEq,Eq)]
pub struct SourceRange<P:SourcePos> {
    pub start:P,
    pub end:P
}
impl Display for SourceRange<SourceOffsetBytes> {
    fn fmt(&self,f:&mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"{}-{}",self.start,self.end)
    }
}
impl Debug for SourceRange<SourceOffsetBytes> {
    fn fmt(&self,f:&mut std::fmt::Formatter) -> std::fmt::Result {
        <Self as Display>::fmt(self,f)
    }
}

#[derive(Clone,Copy,PartialEq,Eq,Default)]
pub struct SourceOffsetBytes {
    pub offset:usize
}
impl Display for SourceOffsetBytes {
    fn fmt(&self,f:&mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"{}",self.offset)
    }
}
impl Debug for SourceOffsetBytes {
    fn fmt(&self,f:&mut std::fmt::Formatter) -> std::fmt::Result {
        <Self as Display>::fmt(self,f)
    }
}
impl SourcePos for SourceOffsetBytes {
    #[inline]
    fn step(&mut self,s:&mut &str) -> Option<char> {
        s.chars().next().map(|c|{
            let len = c.len_utf8();
            *s = &s[len..];
            self.offset += len;
            c
        })
    }
    fn step_until_line_end<'a>(&mut self, s: &mut &'a str) -> (&'a str,Self) {
        match find_line_end(s) {
            None => {
                let l = *s;
                self.offset += s.len();
                *s = "";
                (l,*self)
            },
            Some((l,i,rn)) => {
                self.offset += i;
                (l,Self{ offset:if rn {self.offset - 2} else {self.offset - 1} })
            }
        }
    }
}

#[derive(Clone,Copy,PartialEq,Eq,Default)]
pub struct SourceOffsetLineCol {
    pub line:usize,
    pub col:usize,
    pub offset:usize
}
impl Display for SourceOffsetLineCol {
    fn fmt(&self,f:&mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"l. {} c. {} ({})",self.line,self.col,self.offset)
    }
}
impl Debug for SourceOffsetLineCol {
    fn fmt(&self,f:&mut std::fmt::Formatter) -> std::fmt::Result {
        <Self as Display>::fmt(self,f)
    }
}
impl SourcePos for SourceOffsetLineCol {
    fn step(&mut self,s:&mut &str) -> Option<char> {
        let mut cs = s.chars();
        match cs.next() {
            None => None,
            Some('\n') => {
                self.line += 1;
                self.col = 0;
                self.offset += 1;
                *s = &s[1..];
                Some('\n')
            }
            Some('\r') => {
                self.line += 1;
                self.col = 0;
                self.offset += 1;
                *s = &s[1..];
                if cs.next() == Some('\n') {
                    self.offset += 1;
                    *s = &s[1..];
                }
                Some('\n')
            }
            Some(c) => {
                self.col += 1;
                self.offset += c.len_utf8();
                *s = &s[c.len_utf8()..];
                Some(c)
            }
        }
    }
    fn step_until_line_end<'a>(&mut self, s: &mut &'a str) -> (&'a str,Self) {
        match find_line_end(s) {
            None => {
                let l = *s;
                self.offset += s.len();
                self.col += s.chars().count();
                *s = "";
                (l,*self)
            },
            Some((l,i,rn)) => {
                let off = Self {
                    offset:self.offset + i - if rn { 2 } else { 1 },
                    col:self.col,line:self.line
                };
                self.offset += i;
                self.col = 0;
                self.line += 1;
                (l, off)
            }
        }
    }
}

pub struct Parser<'a,P:SourcePos> {
    input:&'a str,
    curr_pos:P
}

impl<'a,P:SourcePos> Parser<'a, P> {
    #[inline(always)]
    pub fn new(input:&'a str) -> Self {
        Self { input, curr_pos:P::default() }
    }
    #[inline(always)]
    pub fn rest(&self) -> &'a str { self.input }
    #[inline(always)]
    pub fn curr_pos(&self) -> &P { &self.curr_pos }
    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.input.is_empty() }
    #[inline(always)]
    pub fn pop_head(&mut self) -> Option<char> {
        self.curr_pos.step(&mut self.input)
    }
    #[inline(always)]
    pub fn read_until_line_end(&mut self) -> (&'a str,P) {
        self.curr_pos.step_until_line_end(&mut self.input)
    }

    pub fn read_until<Pr:FnMut(&'a str) -> bool>(&mut self,mut p:Pr) -> &'a str {
        let curr = self.input;
        let mut i = 0;
        while !p(self.input) {
            match self.curr_pos.step(&mut self.input) {
                None => break,
                Some(c) => i += c.len_utf8()
            }
        }
        &curr[..i]
    }

    pub fn read_while<Pr:FnMut(char) -> bool>(&mut self,mut p:Pr) -> &'a str {
        self.read_until(|s| !s.starts_with(&mut p))
    }

    pub fn trim_start(&mut self) {
        self.read_while(|c| c.is_whitespace());
    }
}