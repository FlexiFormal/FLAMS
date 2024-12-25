use std::borrow::Cow;

use either::Either;
use immt_ontology::{content::{declarations::{morphisms::Morphism, structures::{Extension, MathStructure}, symbols::{ArgSpec, AssocType, Symbol}, OpenDeclaration}, modules::{NestedModule, OpenModule}, terms::{Term, Var}}, languages::Language, narration::{exercises::Exercise, notations::Notation, paragraphs::{LogicalParagraph, ParagraphKind}, sections::{Section, SectionLevel}, variables::Variable, DocumentElement}, uris::{ContentURI, DocumentElementURI, DocumentURI, ModuleURI, SymbolURI, URIOrRefTrait}};
use smallvec::SmallVec;
use terms::{OpenArg, PreVar, VarOrSym};

#[cfg(feature="rdf")]
use immt_ontology::triple;

use crate::{errors::SHTMLError, prelude::{ExerciseState, NotationState, ParagraphState, SHTMLExtractor, SHTMLNode}, rules::SHTMLElements};

pub mod terms;
#[allow(clippy::large_enum_variant)]
#[derive(Debug,Clone)]
pub enum OpenSHTMLElement {
    Invisible,
    SetSectionLevel(SectionLevel),
    ImportModule(ModuleURI),
    UseModule(ModuleURI),
    Module {
        uri:ModuleURI,
        meta:Option<ModuleURI>,
        signature:Option<Language>,
    },
    MathStructure {
        uri: SymbolURI,
        macroname: Option<Box<str>>,
    },
    Morphism {
        uri: Option<SymbolURI>,
        domain: ModuleURI,
        total: bool
    },
    Assign(SymbolURI),
    Section {
        lvl:SectionLevel,
        uri:DocumentElementURI
    },
    Paragraph {
        uri:DocumentElementURI,
        kind: ParagraphKind,
        inline: bool,
        styles: Box<[Box<str>]>,
    },
    Exercise {
        uri:DocumentElementURI,
        styles: Box<[Box<str>]>,
        autogradable: bool,
        points: Option<f32>,
        sub_exercise:bool
    },
    Doctitle,
    Title,
    Symdecl {
        uri: SymbolURI,
        arity: ArgSpec,
        macroname: Option<Box<str>>,
        role: Box<[Box<str>]>,
        assoctype: Option<AssocType>,
        reordering: Option<Box<str>>,
    },
    Vardecl {
        uri: DocumentElementURI,
        arity: ArgSpec,
        bind:bool,
        macroname: Option<Box<str>>,
        role: Box<[Box<str>]>,
        assoctype: Option<AssocType>,
        reordering: Option<Box<str>>,
        is_seq:bool
    },
    Notation {
        id:Box<str>,
        symbol:VarOrSym,
        precedence:isize,
        argprecs:SmallVec<isize,9>
    },
    NotationComp,
    NotationOpComp,
    Definiendum(SymbolURI),
    Type,
    Conclusion{uri:SymbolURI,in_term:bool},
    Definiens{uri:Option<SymbolURI>,in_term:bool},
    OpenTerm{term:terms::OpenTerm,is_top:bool},
    ClosedTerm(Term),
    MMTRule(Box<str>),
    ArgSep,
    ArgMap,
    ArgMapSep,
    HeadTerm,



    Inputref{uri:DocumentURI,id:Box<str>},
    IfInputref(bool),


    Comp,
    MainComp,
    Arg(OpenArg),
}

impl OpenSHTMLElement {
    #[allow(clippy::too_many_lines)]
    pub(crate) fn close<E:SHTMLExtractor,N:SHTMLNode>(self,previous:&mut SHTMLElements,next:&mut SHTMLElements,extractor:&mut E,node:&N) -> Option<Self> {
        //println!("{self:?}}}");
        match self {
            Self::Invisible => {
                if !extractor.in_term() && !extractor.in_notation() {
                    node.delete();
                }
            }
            Self::SetSectionLevel(lvl) => 
                extractor.add_document_element(
                    DocumentElement::SetSectionLevel(lvl)
                ),
            Self::ImportModule(uri) => Self::close_importmodule(extractor, uri),
            Self::UseModule(uri) => Self::close_usemodule(extractor, uri),
            Self::Module { uri, meta, signature } => Self::close_module(extractor, node, uri, meta, signature),
            Self::MathStructure { uri,macroname} => Self::close_structure(extractor, node, uri, macroname),
            Self::Morphism { uri,domain,total } => Self::close_morphism(extractor, node, uri, domain, total),

            Self::Assign(_sym) => {
                if let Some(tm) = extractor.close_complex_term() {

                }
                // TODO
            }

            Self::Section { lvl,  uri } => Self::close_section(extractor, node, lvl, uri),
            Self::Paragraph { kind, inline, styles, uri } => Self::close_paragraph(extractor, node, kind, inline, styles, uri),
            Self::Exercise { uri, styles, autogradable, points, sub_exercise } => Self::close_exercise(extractor, node, uri, styles, autogradable, points, sub_exercise),

            Self::Doctitle => {
                extractor.set_document_title(node.inner_string().into_boxed_str());
            }

            Self::Title =>
                if extractor.add_title(node.inner_range()).is_err() {
                    extractor.add_error(SHTMLError::NotInNarrative);
                },
            Self::Symdecl { uri, arity, macroname, role, assoctype, reordering } => 
                Self::close_symdecl(extractor, uri, arity, macroname, role, assoctype, reordering),
            Self::Vardecl { uri, arity, bind, macroname, role, assoctype, reordering, is_seq } =>
                Self::close_vardecl(extractor, uri, bind,arity, macroname, role, assoctype, reordering, is_seq),
            Self::Notation { id, symbol, precedence, argprecs } => 
                Self::close_notation(extractor, id, symbol, precedence, argprecs),
            Self::NotationComp => {
                if let Some(n) = node.as_notation() {
                    if extractor.add_notation(n).is_err() {
                        extractor.add_error(SHTMLError::NotInNarrative);
                    }
                } else {
                    extractor.add_error(SHTMLError::NotInNarrative);
                }
            }
            Self::NotationOpComp => {
                if let Some(n) = node.as_op_notation() {
                    if extractor.add_op_notation(n).is_err() {
                        extractor.add_error(SHTMLError::NotInNarrative);
                    }
                } else {
                    extractor.add_error(SHTMLError::NotInNarrative);
                }
            }
            Self::Type => {
                extractor.set_in_term(false);
                let tm = Self::as_term(next,node);
                if extractor.add_type(tm).is_err() {
                    extractor.add_error(SHTMLError::NotInContent);
                }
            }
            Self::Conclusion { uri, in_term } => {
                extractor.set_in_term(in_term);
                let tm = Self::as_term(next,node);
                if extractor.add_term(Some(uri), tm).is_err() {
                    extractor.add_error(SHTMLError::NotInContent);
                }
            }
            Self::Definiens { uri, in_term } => {
                extractor.set_in_term(in_term);
                let tm = Self::as_term(next,node);
                if extractor.add_term(uri, tm).is_err() {
                    extractor.add_error(SHTMLError::NotInContent);
                }
            }
            Self::OpenTerm { term, is_top:true } => {
                let term = term.close(extractor);
                let uri = match extractor.get_narrative_uri() & &*extractor.new_id(Cow::Borrowed("term")) {
                    Ok(uri) => uri,
                    Err(e) => {
                        extractor.add_error(SHTMLError::InvalidURI("(should be impossible)".to_string()));
                        return None
                    }
                };
                extractor.set_in_term(false);
                if !matches!(term,Term::OMID{..}|Term::OMV{..}) {
                    extractor.add_document_element(DocumentElement::TopTerm { uri, term });
                }
            }
            Self::OpenTerm{term,is_top:false } => {
                let term = term.close(extractor);
                return Some(Self::ClosedTerm(term));
            }
            Self::MMTRule(_id) => {
                let _ = extractor.close_args();
                // TODO
            }
            Self::ArgSep => {
                return Some(Self::ArgSep);
            }
            Self::ArgMap => {
                return Some(Self::ArgMap);
            }
            Self::ArgMapSep => {
                return Some(Self::ArgMapSep);
            }
            Self::Arg(a) => {
                if extractor.in_notation() {
                    return Some(self)
                }
                let t = node.as_term();
                let pos = match a.index {
                    Either::Left(u) => (u,None),
                    Either::Right((a,b)) => (a,Some(b))
                };
                if extractor.add_arg(pos, t, a.mode).is_err() {
                    //println!("HERE 1");
                    extractor.add_error(SHTMLError::IncompleteArgs);
                }
            }
            Self::HeadTerm => {
                let tm = node.as_term();
                if extractor.add_term(None,tm).is_err() {
                    //println!("HERE 2");
                    extractor.add_error(SHTMLError::IncompleteArgs);
                }
            }

            Self::Comp | Self::MainComp if extractor.in_notation() => {
                return Some(self);
            }
            Self::ClosedTerm(_) => return Some(self),

            Self::Inputref { uri, id } => {
                let top = extractor.get_narrative_uri();
                #[cfg(feature="rdf")]
                if E::RDF {
                    extractor.add_triples([
                        triple!(<(top.to_iri())> dc:HAS_PART <(uri.to_iri())>)
                    ]);
                }
                extractor.add_document_element(DocumentElement::DocumentReference { 
                    id: match top & &*id {
                        Ok(id) => id,
                        Err(e) => {
                            extractor.add_error(SHTMLError::InvalidURI(id.to_string()));
                            return None
                        }
                    },
                    range: node.range(), 
                    target: uri
                });
                previous.elems.retain(|e| !matches!(e,Self::Invisible));
            }

            Self::IfInputref(_) | Self::Definiendum(_) | Self::Comp | Self::MainComp => (),
        }
        None
    }

    fn as_term<N:SHTMLNode>(next:&mut SHTMLElements,node:&N) -> Term {
        if let Some(i) = next.iter().position(|e| matches!(e,Self::ClosedTerm(_))) {
            let Self::ClosedTerm(t) = next.elems.remove(i) else {unreachable!()};
            return t
        }
        node.as_term()
    }

    fn close_importmodule<E:SHTMLExtractor>(extractor:&mut E,uri:ModuleURI) {
        #[cfg(feature="rdf")]
        if E::RDF {
            if let Some(m) = extractor.get_content_iri() {
                extractor.add_triples([
                    triple!(<(m)> ulo:IMPORTS <(uri.to_iri())>)
                ]);
            }
        }
        extractor.add_document_element(DocumentElement::ImportModule(uri.clone()));
        if extractor.add_content_element(OpenDeclaration::Import(uri),).is_err() {
            extractor.add_error(SHTMLError::NotInContent);
        }
    }

    fn close_usemodule<E:SHTMLExtractor>(extractor:&mut E,uri:ModuleURI) {
        #[cfg(feature="rdf")]
        if E::RDF {
            extractor.add_triples([
                triple!(<(extractor.get_document_iri())> dc:REQUIRES <(uri.to_iri())>)
            ]);
            
        }
        extractor.add_document_element(DocumentElement::UseModule(uri));
    }

    fn close_module<E:SHTMLExtractor,N:SHTMLNode>(extractor:&mut E,node:&N,uri:ModuleURI,meta:Option<ModuleURI>,signature:Option<Language>) {
        let Some((_,narrative)) = extractor.close_narrative() else {
            extractor.add_error(SHTMLError::NotInNarrative);
            return
        };
        let Some((_,mut content)) = extractor.close_content() else {
            extractor.add_error(SHTMLError::NotInContent);
            return
        };

        #[cfg(feature="rdf")]
        if E::RDF {
            let iri = uri.to_iri();
            extractor.add_triples([
                triple!(<(iri.clone())> : ulo:THEORY),
                triple!(<(extractor.get_document_iri())> ulo:CONTAINS <(iri)>)
            ]);
        }

        extractor.add_document_element(DocumentElement::Module { 
            range: node.range(), 
            module: uri.clone(), 
            children: narrative
        });

        if uri.name().is_simple() {
            extractor.add_module(OpenModule {
                uri,meta,signature,elements:content
            });
        } else { // NestedModule
            let Some(sym) = uri.into_symbol() else {unreachable!()};
            #[cfg(feature="rdf")]
            if E::RDF {
                if let Some(m) = extractor.get_content_iri() {
                    extractor.add_triples([
                        triple!(<(m)> ulo:CONTAINS <(sym.to_iri())>)
                    ]);
                }
            }
            if extractor.add_content_element(OpenDeclaration::NestedModule(NestedModule { 
                uri:sym,
                elements:std::mem::take(&mut content)
            })).is_err() {
                extractor.add_error(SHTMLError::NotInContent);

            }
        }
    }


    fn close_structure<E:SHTMLExtractor,N:SHTMLNode>(extractor:&mut E,node:&N,uri:SymbolURI,macroname:Option<Box<str>>) {                
        let Some((_,narrative)) = extractor.close_narrative() else {
            extractor.add_error(SHTMLError::NotInNarrative);
            return
        };
        let Some((_,content)) = extractor.close_content() else {
            extractor.add_error(SHTMLError::NotInContent);
            return
        };

        #[cfg(feature="rdf")]
        if E::RDF {
            if let Some(cont) = extractor.get_content_iri() {
                let iri = uri.to_iri();
                extractor.add_triples([
                    triple!(<(iri.clone())> : ulo:STRUCTURE),
                    triple!(<(cont)> ulo:CONTAINS <(iri)>)
                ]);
            }
        }

        if uri.name().last_name().as_ref().starts_with("EXTSTRUCT") {
            let Some(target) = content.iter().find_map(|d| match d {
                OpenDeclaration::Import(uri) if !uri.name().last_name().as_ref().starts_with("EXTSTRUCT") => Some(uri),
                _ => None
            }) else {
                extractor.add_error(SHTMLError::NotInContent);
                return
            };
            let Some(target) = target.clone().into_symbol() else {
                extractor.add_error(SHTMLError::NotInContent);
                return
            };

            #[cfg(feature="rdf")]
            if E::RDF {
                extractor.add_triples([
                    triple!(<(uri.to_iri())> ulo:EXTENDS <(target.to_iri())>)
                ]);
            }
            extractor.add_document_element(DocumentElement::Extension { 
                range: node.range(), extension:uri.clone(), target: target.clone(), children: narrative
            });
            if extractor.add_content_element(OpenDeclaration::Extension(Extension {
                uri,elements:content,target
            })).is_err() {
                extractor.add_error(SHTMLError::NotInContent);
            }
        } else {
            extractor.add_document_element(DocumentElement::MathStructure { 
                range: node.range(), structure: uri.clone(), children: narrative
            });
            if extractor.add_content_element(OpenDeclaration::MathStructure(MathStructure {
                uri,elements:content,macroname
            })).is_err() {
                extractor.add_error(SHTMLError::NotInContent);
            }
        }
    }

    fn close_morphism<E:SHTMLExtractor,N:SHTMLNode>(extractor:&mut E,node:&N,uri:Option<SymbolURI>,domain:ModuleURI,total:bool) {
        let Some((_,narrative)) = extractor.close_narrative() else {
            extractor.add_error(SHTMLError::NotInNarrative);
            return
        };
        let Some((_,content)) = extractor.close_content() else {
            extractor.add_error(SHTMLError::NotInContent);
            return
        };

        #[cfg(feature="rdf")]
        if E::RDF {
            if let Some(cont) = extractor.get_content_iri() {
                let iri = uri.as_ref().expect("TODO").to_iri(); // TODO
                extractor.add_triples([
                    triple!(<(iri.clone())> : ulo:MORPHISM),
                    triple!(<(iri.clone())> rdfs:DOMAIN <(domain.to_iri())>),
                    triple!(<(cont)> ulo:CONTAINS <(iri)>)
                ]);
            }
        }
        
        extractor.add_document_element(DocumentElement::Morphism { 
            range: node.range(), morphism: uri.clone().expect("TODO") /* TODO */, children: narrative
        });
        if extractor.add_content_element(OpenDeclaration::Morphism(Morphism {
            uri,domain,total,elements:content
        })).is_err() {
            extractor.add_error(SHTMLError::NotInContent);
        }
    }

    fn close_section<E:SHTMLExtractor,N:SHTMLNode>(extractor:&mut E,node:&N,lvl:SectionLevel,uri:DocumentElementURI) {
        let Some((_,title,children)) = extractor.close_section() else {
            extractor.add_error(SHTMLError::NotInNarrative);
            return
        };
        extractor.add_document_element(
            DocumentElement::Section(Section {
                range:node.range(),
                level:lvl,
                title,
                uri,
                children
            })
        );
    }

    fn close_paragraph<E:SHTMLExtractor,N:SHTMLNode>(extractor:&mut E,node:&N,kind:ParagraphKind,inline:bool,styles:Box<[Box<str>]>,uri:DocumentElementURI) {
        let Some(ParagraphState{children,fors,title,..}) = extractor.close_paragraph() else {
            extractor.add_error(SHTMLError::NotInParagraph);
            return
        };

        #[cfg(feature="rdf")]
        if E::RDF {
            let doc =  extractor.get_document_iri();
            let iri = uri.to_iri();
            if kind.is_definition_like(&styles) {
                for (f,_) in fors.iter() {
                    extractor.add_triples([
                        triple!(<(iri.clone())> ulo:DEFINES <(f.to_iri())>)
                    ]);

                }
            } else if kind == ParagraphKind::Example {
                for (f,_) in fors.iter() {
                    extractor.add_triples([
                        triple!(<(iri.clone())> ulo:EXAMPLE_FOR <(f.to_iri())>)
                    ]);

                }
            }
            extractor.add_triples([
                triple!(<(iri.clone())> : <(kind.rdf_type().into_owned())>),
                triple!(<(doc)> ulo:CONTAINS <(iri)>)
            ]);
        }

        extractor.add_document_element(DocumentElement::Paragraph(
            LogicalParagraph {
                range: node.range(),kind,inline,styles,
                fors,uri,children,title
            }
        ));
    }

    fn close_exercise<E:SHTMLExtractor,N:SHTMLNode>(extractor:&mut E,node:&N,uri:DocumentElementURI,styles:Box<[Box<str>]>,autogradable:bool,points:Option<f32>,sub_exercise:bool) {
        let Some(ExerciseState{solutions,hints,notes,gnotes,title,children,preconditions,objectives,..}) = extractor.close_exercise() else {
            extractor.add_error(SHTMLError::NotInExercise);
            return
        };

        #[cfg(feature="rdf")]
        if E::RDF {
            let doc =  extractor.get_document_iri();
            let iri = uri.to_iri();
            for (d,s) in &preconditions {
                let b = immt_ontology::rdf::BlankNode::default();
                extractor.add_triples([
                    triple!(<(iri.clone())> ulo:PRECONDITION (b.clone())!),
                    triple!((b.clone())! ulo:COGDIM <(d.to_iri().into_owned())>),
                    triple!((b)! ulo:POSYMBOL <(s.to_iri())>)
                ]);
            }
            for (d,s) in &objectives {
                let b = immt_ontology::rdf::BlankNode::default();
                extractor.add_triples([
                    triple!(<(iri.clone())> ulo:OBJECTIVE (b.clone())!),
                    triple!((b.clone())! ulo:COGDIM <(d.to_iri().into_owned())>),
                    triple!((b)! ulo:POSYMBOL <(s.to_iri())>)
                ]);
            }

            extractor.add_triples([
                if sub_exercise {
                    triple!(<(iri.clone())> : ulo:SUBPROBLEM)
                } else {
                    triple!(<(iri.clone())> : ulo:PROBLEM)
                },
                triple!(<(doc)> ulo:CONTAINS <(iri)>)
            ]);
        }

        extractor.add_document_element(DocumentElement::Exercise(
            Exercise {
                range: node.range(),uri,styles,autogradable,points,sub_exercise,
                solutions,hints,notes,grading_notes: gnotes,title,children,preconditions,objectives
            }
        ));
    }

    fn close_symdecl<E:SHTMLExtractor>(extractor:&mut E,uri:SymbolURI,arity:ArgSpec,macroname:Option<Box<str>>,role: Box<[Box<str>]>,assoctype:Option<AssocType>,reordering:Option<Box<str>>) {
        let Some((tp,df)) = extractor.close_decl() else {
            extractor.add_error(SHTMLError::NotInContent);
            return
        };
        #[cfg(feature="rdf")]
        if E::RDF {
            if let Some(m) = extractor.get_content_iri() {
                let iri = uri.to_iri();
                extractor.add_triples([
                    triple!(<(iri.clone())> : ulo:DECLARATION),
                    triple!(<(m)> ulo:DECLARES <(iri)>),
                ]);
            }
        }
        extractor.add_document_element(
            DocumentElement::SymbolDeclaration(uri.clone())
        );
        if extractor.add_content_element(OpenDeclaration::Symbol(Symbol {
            uri,arity,macroname,role,tp,df,assoctype,reordering
        })).is_err() {
            extractor.add_error(SHTMLError::NotInContent);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn close_vardecl<E:SHTMLExtractor>(extractor:&mut E,uri:DocumentElementURI,bind:bool,arity:ArgSpec,macroname:Option<Box<str>>,role: Box<[Box<str>]>,assoctype:Option<AssocType>,reordering:Option<Box<str>>,is_seq:bool) {
        let Some((tp,df)) = extractor.close_decl() else {
            extractor.add_error(SHTMLError::NotInContent);
            return
        };
        
        #[cfg(feature="rdf")]
        if E::RDF {
            let iri = uri.to_iri();
            extractor.add_triples([
                triple!(<(iri.clone())> : ulo:VARIABLE),
                triple!(<(extractor.get_document_iri())> ulo:DECLARES <(iri)>),
            ]);
        }
        
        extractor.add_document_element(
            DocumentElement::Variable(Variable {
                uri,arity,macroname,bind,role,tp,df,assoctype,reordering,is_seq
            })
        );
    }

    fn close_notation<E:SHTMLExtractor>(extractor:&mut E,id:Box<str>,symbol:VarOrSym,precedence:isize,argprecs:SmallVec<isize,9>) {
        let Some(NotationState {
            attribute_index,inner_index,is_text,components,op
        }) = extractor.close_notation() else {
            extractor.add_error(SHTMLError::NotInNarrative);
            return
        };
        if attribute_index == 0 {
            extractor.add_error(SHTMLError::NotInNarrative);
            return
        }
        let uri = match extractor.get_narrative_uri() & &*id {
            Ok(uri) => uri,
            Err(e) => {
                extractor.add_error(SHTMLError::InvalidURI(id.to_string()));
                return
            }
        };
        let notation = extractor.add_resource(&Notation {
            attribute_index,
            is_text,
            inner_index,
            components,
            op,
            precedence,id,argprecs
        });
        match symbol {
            VarOrSym::S(ContentURI::Symbol(symbol)) => {
                #[cfg(feature="rdf")]
                if E::RDF {
                    let iri = uri.to_iri();
                    extractor.add_triples([
                        triple!(<(iri.clone())> : ulo:NOTATION),
                        triple!(<(iri.clone())> ulo:NOTATION_FOR <(symbol.to_iri())>),
                        triple!(<(extractor.get_document_iri())> ulo:DECLARES <(iri)>),
                    ]);
                }
                extractor.add_document_element(
                    DocumentElement::Notation { symbol, id:uri, notation }
                );
            }
            VarOrSym::S(_) => unreachable!(),
            VarOrSym::V(PreVar::Resolved(variable)) => 
                extractor.add_document_element(
                    DocumentElement::VariableNotation { variable, id:uri, notation }
                ),
            VarOrSym::V(PreVar::Unresolved(name)) => match extractor.resolve_variable_name(name) {
                Var::Name(name) => extractor.add_error(SHTMLError::UnresolvedVariable(name)),
                Var::Ref{declaration,..} => 
                extractor.add_document_element(
                    DocumentElement::VariableNotation { variable:declaration, id:uri, notation }
                ),
            }
        }
    }

}