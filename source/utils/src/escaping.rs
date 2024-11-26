use std::fmt::Display;

pub struct Escaper<C, const N: usize>(pub [(C, &'static str); N]);
impl<const N: usize> Escaper<char, N> {
    pub fn escape<'a, D: Display>(&'a self, display: &'a D) -> impl Display + 'a {
        EscaperI {
            display,
            replacements: &self.0,
        }
    }
}
impl<const N: usize> Escaper<u8, N> {
    pub fn escape<'a, D: Display>(&'a self, display: &'a D) -> impl Display + 'a {
        EscaperI {
            display,
            replacements: &self.0,
        }
    }    
    pub fn unescape<'a, D: Display>(&'a self, display: &'a D) -> impl Display + 'a {
        UnEscaperI {
            display,
            replacements: &self.0,
        }
    }
}

struct EscaperI<'a, D: Display, C, const N: usize> {
    display: &'a D,
    replacements: &'a [(C, &'static str); N],
}
impl<D: Display, const N: usize> Display for EscaperI<'_, D, char, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = Replacer {
            writer: f,
            replacements: self.replacements,
        };
        std::fmt::Write::write_fmt(&mut r, format_args!("{}", self.display))
    }
}
impl<D: Display, const N: usize> Display for EscaperI<'_, D, u8, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = Replacer {
            writer: f,
            replacements: self.replacements,
        };
        std::fmt::Write::write_fmt(&mut r, format_args!("{}", self.display))
    }
}

struct UnEscaperI<'a, D: Display, C, const N: usize> {
    display: &'a D,
    replacements: &'a [(C, &'static str); N],
}
impl<D: Display, const N: usize> Display for UnEscaperI<'_, D, u8, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = RevReplacer {
            writer: f,
            replacements: self.replacements,
        };
        std::fmt::Write::write_fmt(&mut r, format_args!("{}", self.display))
    }
}

struct Replacer<'a, W: std::fmt::Write, C, const N: usize> {
    writer: W,
    replacements: &'a [(C, &'static str); N],
}
impl<W: std::fmt::Write, const N: usize> std::fmt::Write for Replacer<'_, W, char, N> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        for c in s.chars() {
            self.write_char(c)?;
        }
        Ok(())
    }
    fn write_char(&mut self, c: char) -> std::fmt::Result {
        for (r, s) in self.replacements {
            if c == *r {
                return self.writer.write_str(s);
            }
        }
        self.writer.write_char(c)
    }
}
impl<W: std::fmt::Write, const N: usize> std::fmt::Write for Replacer<'_, W, u8, N> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        for c in s.as_bytes() {
            self.write_char(*c as char)?;
        }
        Ok(())
    }
    fn write_char(&mut self, c: char) -> std::fmt::Result {
        for (r, s) in self.replacements {
            if c == *r as char {
                return self.writer.write_str(s);
            }
        }
        self.writer.write_char(c)
    }
}

struct RevReplacer<'a, W: std::fmt::Write, C, const N: usize> {
    writer: W,
    replacements: &'a [(C, &'static str); N],
}

impl<W: std::fmt::Write, const N: usize> std::fmt::Write for RevReplacer<'_, W, u8, N> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let bytes = self.replacements.map(|(a,b)| (a,b.as_bytes()));
        let mut s = s.as_bytes();
        'outer: loop {
            let mut i = 0;
            while i < s.len() {
                if let Some((a,b)) = bytes.iter().find_map(|(b,needle)|
                    s.strip_prefix(*needle).map(|r| (*b,r) )
                ) {
                    self.writer.write_str(std::str::from_utf8(&s[..i]).map_err(|_|std::fmt::Error)?)?;
                    self.writer.write_char(a as char)?;
                    s = b;
                    continue 'outer
                }
                i += 1;
            }
            return self.writer.write_str(std::str::from_utf8(s).map_err(|_| std::fmt::Error)?);
        }
    }
    fn write_char(&mut self, c: char) -> std::fmt::Result {
        self.writer.write_char(c)
    }
}

pub static IRI_ESCAPE: Escaper<u8, 5> = Escaper([
    (b' ', "%20"),
    (b'\\', "%5C"),
    (b'^', "%5E"),
    (b'[', "%5B"),
    (b']', "%5D"),
]);
