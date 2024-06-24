pub use arrayvec::ArrayVec;
use crate::uris::symbols::SymbolURI;

#[derive(Debug, Clone)]
pub struct Constant {
    pub uri:SymbolURI,
    pub arity:ArrayVec<ArgType,9>,
    pub macroname:Option<String>
}


#[derive(Debug, Clone,Copy)]
pub enum ArgType {
    Normal,Sequence,Binding,BindingSequence
}
impl ArgType {
    pub fn parse(s:&str) -> ArrayVec<ArgType,9> {
        let mut ret = ArrayVec::new();
        for c in s.bytes() {
            ret.push(match c {
                b'i' => ArgType::Normal,
                b'a' => ArgType::Sequence,
                b'b' => ArgType::Binding,
                b'B' => ArgType::BindingSequence,
                _ => panic!("Invalid ArgType")
            })
        }
        ret
    }
}