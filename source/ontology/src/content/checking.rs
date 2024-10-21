use crate::{
    uris::{ModuleURI, SymbolURI},
    LocalBackend,
};

use super::{
    declarations::{
        morphisms::{Morphism, UncheckedMorphism},
        structures::{Extension, MathStructure, UncheckedExtension, UncheckedMathStructure},
        Declaration, UncheckedDeclaration,
    },
    modules::NestedModule,
};

pub trait ModuleChecker: LocalBackend {
    fn open(&mut self, elem: &mut UncheckedDeclaration);
    fn close(&mut self, elem: &mut Declaration);
}

enum Elem {
    Extension(SymbolURI, SymbolURI),
    NestedModule(SymbolURI),
    MathStructure {
        uri: SymbolURI,
        macroname: Option<Box<str>>,
    },
    Morphism {
        uri: Option<SymbolURI>,
        domain: ModuleURI,
        total: bool,
    },
}

impl Elem {
    fn close(self, v: Vec<Declaration>, checker: &mut impl ModuleChecker) -> Declaration {
        match self {
            Self::Extension(uri, target) => {
                //println!("Require declaration {target}");
                let target = checker
                    .get_declaration(&target)
                    .map_or_else(|| Err(target), Ok);
                Declaration::Extension(Extension {
                    uri,
                    target,
                    elements: v.into_boxed_slice(),
                })
            }
            Self::NestedModule(uri) => {
                //println!("Closing nested module {uri}");
                Declaration::NestedModule(NestedModule {
                    uri,
                    elements: v.into_boxed_slice(),
                })
            }
            Self::MathStructure { uri, macroname } => {
                //println!("Closing structure {uri}");
                Declaration::MathStructure(MathStructure {
                    uri,
                    macroname,
                    elements: v.into_boxed_slice(),
                })
            }
            Self::Morphism { uri, domain, total } => {
                //println!("Require domain {domain}");
                let domain = checker.get_module(&domain).map_or_else(|| Err(domain), Ok);
                Declaration::Morphism(Morphism {
                    uri,
                    domain,
                    total,
                    elements: v.into_boxed_slice(),
                })
            }
        }
    }
}

pub(super) struct ModuleCheckIter<'a, Check: ModuleChecker> {
    stack: Vec<(
        Elem,
        std::vec::IntoIter<UncheckedDeclaration>,
        Vec<Declaration>,
    )>,
    curr_in: std::vec::IntoIter<UncheckedDeclaration>,
    curr_out: Vec<Declaration>,
    checker: &'a mut Check,
    uri: &'a ModuleURI,
}
impl<Check: ModuleChecker> ModuleCheckIter<'_, Check> {
    pub fn go(
        elems: Vec<UncheckedDeclaration>,
        checker: &mut Check,
        uri: &ModuleURI,
    ) -> Vec<Declaration> {
        let mut slf = ModuleCheckIter {
            stack: Vec::new(),
            curr_in: elems.into_iter(),
            curr_out: Vec::new(),
            checker,
            uri,
        };
        loop {
            while let Some(next) = slf.curr_in.next() {
                slf.do_elem(next);
            }
            if let Some((e, curr_in, curr_out)) = slf.stack.pop() {
                slf.curr_in = curr_in;
                let dones = std::mem::replace(&mut slf.curr_out, curr_out);
                let mut closed = e.close(dones, slf.checker);
                slf.checker.close(&mut closed);
                slf.curr_out.push(closed);
            } else {
                return slf.curr_out;
            }
        }
    }

    fn do_elem(&mut self, mut e: UncheckedDeclaration) {
        self.checker.open(&mut e);
        match e {
            UncheckedDeclaration::Import(uri) => {
                //println!("Require import {uri}");
                let m = if !uri.clone() == *self.uri {
                    Err(uri)
                } else {
                    self.checker.get_module(&uri).map_or_else(|| Err(uri), Ok)
                };
                let mut m = Declaration::Import(m);
                self.checker.close(&mut m);
                self.curr_out.push(m);
            }
            UncheckedDeclaration::Symbol(s) => {
                let mut m = Declaration::Symbol(s);
                self.checker.close(&mut m);
                self.curr_out.push(m);
            }
            UncheckedDeclaration::Extension(UncheckedExtension {
                target,
                uri,
                elements,
            }) => {
                let old_in = std::mem::replace(&mut self.curr_in, elements.into_iter());
                let old_out = std::mem::take(&mut self.curr_out);
                self.stack
                    .push((Elem::Extension(uri, target), old_in, old_out));
            }
            UncheckedDeclaration::NestedModule { uri, elements } => {
                let old_in = std::mem::replace(&mut self.curr_in, elements.into_iter());
                let old_out = std::mem::take(&mut self.curr_out);
                self.stack.push((Elem::NestedModule(uri), old_in, old_out));
            }
            UncheckedDeclaration::MathStructure(UncheckedMathStructure {
                uri,
                macroname,
                elements,
            }) => {
                let old_in = std::mem::replace(&mut self.curr_in, elements.into_iter());
                let old_out = std::mem::take(&mut self.curr_out);
                self.stack
                    .push((Elem::MathStructure { uri, macroname }, old_in, old_out));
            }
            UncheckedDeclaration::Morphism(UncheckedMorphism {
                uri,
                domain,
                total,
                elements,
            }) => {
                let old_in = std::mem::replace(&mut self.curr_in, elements.into_iter());
                let old_out = std::mem::take(&mut self.curr_out);
                self.stack
                    .push((Elem::Morphism { uri, domain, total }, old_in, old_out));
            }
        }
    }
}
