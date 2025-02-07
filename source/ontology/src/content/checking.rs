use crate::{
    uris::{ModuleURI, SymbolURI}, LocalBackend, MaybeResolved, Unchecked
};

use super::{
    declarations::{
        morphisms::Morphism,
        structures::{Extension, MathStructure},
        Declaration, OpenDeclaration
    },
    modules::NestedModule,
};

pub trait ModuleChecker: LocalBackend {
    fn open(&mut self, elem: &mut OpenDeclaration<Unchecked>);
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
        uri: SymbolURI,
        domain: ModuleURI,
        total: bool,
    },
}

impl Elem {
    fn close(self, v: Vec<Declaration>, checker: &mut impl ModuleChecker) -> Declaration {
        match self {
            Self::Extension(uri, target) => {
                //println!("Require declaration {target}");
                let target = MaybeResolved::resolve(target,|m| checker.get_declaration(m));
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
                let domain = MaybeResolved::resolve(domain,|d| checker.get_module(d));
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
        std::vec::IntoIter<OpenDeclaration<Unchecked>>,
        Vec<Declaration>,
    )>,
    curr_in: std::vec::IntoIter<OpenDeclaration<Unchecked>>,
    curr_out: Vec<Declaration>,
    checker: &'a mut Check,
    uri: &'a ModuleURI,
}
impl<Check: ModuleChecker> ModuleCheckIter<'_, Check> {
    pub fn go(
        elems: Vec<OpenDeclaration<Unchecked>>,
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

    fn do_elem(&mut self, mut e: OpenDeclaration<Unchecked>) {
        self.checker.open(&mut e);
        match e {
            OpenDeclaration::Import(uri) => {
                //println!("Require import {uri}");
                let m = if !uri.clone() == *self.uri {
                    MaybeResolved::unresolved(uri)
                } else {
                    MaybeResolved::resolve(uri,|u| self.checker.get_module(u))
                };
                let mut m = Declaration::Import(m);
                self.checker.close(&mut m);
                self.curr_out.push(m);
            }
            OpenDeclaration::Symbol(s) => {
                let mut m = Declaration::Symbol(s);
                self.checker.close(&mut m);
                self.curr_out.push(m);
            }
            OpenDeclaration::Extension(Extension {
                target,
                uri,
                elements,
            }) => {
                let old_in = std::mem::replace(&mut self.curr_in, elements.into_iter());
                let old_out = std::mem::take(&mut self.curr_out);
                self.stack
                    .push((Elem::Extension(uri, target), old_in, old_out));
            }
            OpenDeclaration::NestedModule(NestedModule { uri, elements }) => {
                let old_in = std::mem::replace(&mut self.curr_in, elements.into_iter());
                let old_out = std::mem::take(&mut self.curr_out);
                self.stack.push((Elem::NestedModule(uri), old_in, old_out));
            }
            OpenDeclaration::MathStructure(MathStructure {
                uri,
                macroname,
                elements,
            }) => {
                let old_in = std::mem::replace(&mut self.curr_in, elements.into_iter());
                let old_out = std::mem::take(&mut self.curr_out);
                self.stack
                    .push((Elem::MathStructure { uri, macroname }, old_in, old_out));
            }
            OpenDeclaration::Morphism(Morphism {
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
