use std::fmt::Display;

pub struct Escaper<C,const N:usize>(pub [(C,&'static str);N]);
impl<const N:usize> Escaper<char,N> {
    pub fn escape<'a,D:Display>(&'a self, display:&'a D) -> impl Display+'a {
        EscaperI { display, replacements: &self.0 }
    }
}
impl<const N:usize> Escaper<u8,N> {
    pub fn escape<'a,D:Display>(&'a self, display:&'a D) -> impl Display+'a {
        EscaperI { display, replacements: &self.0 }
    }
}

struct EscaperI<'a,D:Display,C,const N:usize> {
    display:&'a D,
    replacements:&'a [(C,&'static str);N],
}
impl<'a,D:Display,const N:usize> Display for EscaperI<'a,D,char,N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = Replacer{writer:f,replacements:self.replacements};
        std::fmt::Write::write_fmt(&mut r,format_args!("{}",self.display))
    }
}
impl<'a,D:Display,const N:usize> Display for EscaperI<'a,D,u8,N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = Replacer{writer:f,replacements:self.replacements};
        std::fmt::Write::write_fmt(&mut r,format_args!("{}",self.display))
    }
}


struct Replacer<'a,W:std::fmt::Write,C,const N:usize> {
    writer:W,
    replacements:&'a [(C,&'static str);N],
}
impl<'a,W:std::fmt::Write,const N:usize> std::fmt::Write for Replacer<'a,W,char,N> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        for c in s.chars() {
            self.write_char(c)?;
        }
        Ok(())
    }
    fn write_char(&mut self, c: char) -> std::fmt::Result {
        for (r,s) in self.replacements {
            if c == *r {
                return self.writer.write_str(s)
            }
        }
        self.writer.write_char(c)
    }
}
impl<'a,W:std::fmt::Write,const N:usize> std::fmt::Write for Replacer<'a,W,u8,N> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        for c in s.as_bytes() {
            self.write_char(*c as char)?;
        }
        Ok(())
    }
    fn write_char(&mut self, c: char) -> std::fmt::Result {
        for (r,s) in self.replacements {
            if c == *r as char {
                return self.writer.write_str(s)
            }
        }
        self.writer.write_char(c)
    }
}

pub static IRI_ESCAPE:Escaper<u8,5> = Escaper([
    (b' ', "%20"),
    (b'\\', "%5C"),
    (b'^', "%5E"),
    (b'[', "%5B"),
    (b']', "%5D"),
]);