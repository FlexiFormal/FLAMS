use std::fmt::Display;

pub struct Depth(pub u8, pub bool);
impl Display for Depth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 == 0 { return Ok(()) }
        for _ in 1..self.0 { write!(f,"│ ")?; }
        if self.1 { write!(f,"├─ ") } else { write!(f,"└─ ") }
    }
}