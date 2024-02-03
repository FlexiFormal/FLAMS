/*
use std::str::pattern::{Pattern,ReverseSearcher};
pub trait Parser {
    fn str(&self) -> &str;
    fn skip(&mut self, n: usize);
    fn read_digit(&mut self) -> u32;


    fn is_empty(&self) -> bool { self.str().is_empty() }
    fn ends_with<'a, P, S>(&'a self, pat: P) -> bool where P: Pattern<'a, Searcher= S>, S:ReverseSearcher<'a> {
        self.str().ends_with(pat)
    }
    fn starts_with<'a, P: Pattern<'a>>(&'a self, pat: P) -> bool { self.str().starts_with(pat) }
    fn starts_with_digit(&self) -> bool { self.str().starts_with(|c:char| c.is_ascii_digit()) }
}

pub struct NoRefParser<'str>(&'str str);
impl<'str> NoRefParser<'str> {
    pub fn new(s:&'str str) -> Self { Self(s.trim_start()) }
}
impl<'str> Parser for NoRefParser<'str> {
    fn str(&self) -> &str { self.0 }
    fn skip(&mut self, n: usize) { self.0 = &self.0[n..].trim_start(); }
    fn read_digit(&mut self) -> u32 {
        let mut ret = 0u32;
        while !self.is_empty() && self.starts_with_digit() {
            ret = ret*10 + self.0.chars().next().unwrap().to_digit(10).unwrap();
            self.0 = &self.0[1..];
        }
        self.0 = self.0.trim_start();
        ret
    }
}

 */
/*
pub(crate) struct Parser<'a>(pub(crate) &'a str);
impl<'a> Parser<'a> {
    pub fn drop_right(&mut self, n: usize) {
        self.0 = &self.0[..self.0.len() - n];
    }

    pub fn read_until_ws(&mut self) -> &str {
        match self.0.find(char::is_whitespace) {
            Some(i) => {
                let (ret,rest) = self.0.split_at(i);
                self.0 = rest.trim_start();
                ret
            }
            None => {
                let ret = self.0;
                self.0 = "";
                ret
            }
        }
    }
    pub fn read_until_ws_or(&mut self,c:char) -> &str {
        match self.0.find(|x| x == c || char::is_whitespace(x)) {
            Some(i) => {
                let (ret,rest) = self.0.split_at(i);
                self.0 = rest.trim_start();
                ret
            }
            None => {
                let ret = self.0;
                self.0 = "";
                ret
            }
        }
    }
    pub fn read_until(&mut self,c:char) -> &str {
        match self.0.find(c) {
            Some(i) => {
                let (ret,rest) = self.0.split_at(i);
                self.0 = rest[1..].trim_start();
                ret
            }
            None => {
                let ret = self.0;
                self.0 = "";
                ret
            }
        }
    }
    pub fn read_until_str(&mut self,s:&str) -> &str {
        match self.0.find(s) {
            Some(i) => {
                let (ret,rest) = self.0.split_at(i);
                self.0 = rest[s.len()..].trim_start();
                ret
            }
            None => {
                let ret = self.0;
                self.0 = "";
                ret
            }
        }
    }
    pub fn read_until_strs(&mut self,s:&[&str]) -> &str {
        for s in s {
            match self.0.find(s) {
                Some(i) => {
                    let (ret, rest) = self.0.split_at(i);
                    self.0 = rest[s.len()..].trim_start();
                    return ret
                }
                None => continue
            }
        }
        let ret = self.0;
        self.0 = "";
        ret
    }

    pub fn drop(&mut self,s:&str) -> bool {
        if self.0.starts_with(s) {
            self.skip(s.len());
            true
        } else {false }
    }

    pub fn skip_until_endline(&mut self) {
        match self.0.find('\n') {
            Some(i) => {
                self.skip(i+1);
            }
            None => {
                self.0 = "";
            }
        }
    }

    pub fn read_until_parens(&mut self) {
        let mut parens = 0usize;
        loop {
            match self.0.find(|x| x == '(' || x == ')') {
                Some(i) => {
                    if self.0.chars().nth(i).unwrap() == '(' {
                        parens += 1;
                    } else if parens == 0 {
                        self.skip(i+1);
                        return
                    } else {
                        parens -= 1;
                    }
                    self.skip(i+1);
                }
                None => {
                    self.0 = "";
                    return
                }
            }
        }
    }

    pub fn read_enc_num(&mut self) -> u8 {
        if self.starts_with('D') {
            self.skip(1);
            let d = self.read_digit();
            return d as u8
        }
        if self.starts_with('O') {
            self.skip(1);
            let d = self.read_digit();
            return u8::from_str_radix(&format!("{}",d),8).unwrap()
        }
        if self.starts_with('H') {
            self.skip(1);
            let d = self.read_digit();
            return u8::from_str_radix(&format!("{}",d),16).unwrap()
        }
        if self.starts_with('C') {
            self.skip(1);
            let c = self.0.chars().next().unwrap();
            self.skip(1);
            return c as u8
        }
        todo!()
    }
}

 */