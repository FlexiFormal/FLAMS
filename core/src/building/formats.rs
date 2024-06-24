use std::fmt::Display;

#[derive(Copy, Clone, Debug,PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ShortId([u8;8]);
impl ShortId {
    pub const CHECK: ShortId = ShortId::new("check");
    pub const fn new(id: &str) -> Self {
        assert!(id.len() > 0 && id.len() <= 8);
        let mut ret = [0,0,0,0,0,0,0,0];
        ret[0] = id.as_bytes()[0];
        if id.len() == 1 {return Self(ret)}
        ret[1] = id.as_bytes()[1];
        if id.len() == 2 {return Self(ret)}
        ret[2] = id.as_bytes()[2];
        if id.len() == 3 {return Self(ret)}
        ret[3] = id.as_bytes()[3];
        if id.len() == 4 {return Self(ret)}
        ret[4] = id.as_bytes()[4];
        if id.len() == 5 {return Self(ret)}
        ret[5] = id.as_bytes()[5];
        if id.len() == 6 {return Self(ret)}
        ret[6] = id.as_bytes()[6];
        if id.len() == 7 {return Self(ret)}
        ret[7] = id.as_bytes()[7];
        Self(ret)
    }
    fn len(&self) -> u8 {
        let mut i = 8u8;
        while i > 0 && self.0[i as usize - 1] == 0 { i -= 1 }
        i
    }
}
impl<'s> TryFrom<&'s str> for ShortId {
    type Error = ();
    fn try_from(s: &'s str) -> Result<Self, Self::Error> {
        if s.len() > 8 { return Err(()); }
        Ok(Self::new(s))
    }
}
impl Display for ShortId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str = std::str::from_utf8(&self.0[0..self.len() as usize])
            .map_err(|_| std::fmt::Error)?;
        f.write_str(str)
    }
}