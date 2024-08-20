
#[cfg(all(feature="client",not(feature="server")))]
pub fn main() {
    use crate::home::*;
    crate::hydrate();
}

#[cfg(feature="server")]
fn main() {}