pub use oxrdf::{
    BlankNode, GraphName, GraphNameRef, Literal, LiteralRef, NamedNode, NamedNodeRef, Quad,
    QuadRef, Subject, SubjectRef, Term as RDFTerm, TermRef as RDFTermRef, Triple, TripleRef,
    Variable,
};

#[macro_export]
macro_rules! triple {
    (<($sub:expr)> $($tt:tt)*) => {
        triple!(@PRED $crate::rdf::Subject::NamedNode($sub); $($tt)*)
    };

    (($sub:expr)! $($tt:tt)*) => {
        triple!(@PRED $crate::rdf::Subject::BlankNode($sub); $($tt)*)
    };

    (@PRED $sub:expr; : $($tt:tt)*) => {
        triple!(@OBJ $sub;$crate::rdf::ontologies::rdf::TYPE.into_owned(); $($tt)*)
    };
    (@PRED $sub:expr;ulo:$pred:ident $($tt:tt)*) => {
        triple!(@OBJ $sub;$crate::rdf::ontologies::ulo2::$pred.into_owned(); $($tt)*)
    };
    (@PRED $sub:expr;dc:$pred:ident $($tt:tt)*) => {
        triple!(@OBJ $sub;$crate::rdf::ontologies::dc::$pred.into_owned(); $($tt)*)
    };
    (@PRED $sub:expr;rdfs:$pred:ident $($tt:tt)*) => {
        triple!(@OBJ $sub;$crate::rdf::ontologies::rdfs::$pred.into_owned(); $($tt)*)
    };

    (@OBJ $sub:expr;$pred:expr; = ($obj:expr) $($tt:tt)*) => {
        triple!(@MAYBEQUAD $sub;$pred;$crate::rdf::RDFTerm::Literal(
            $crate::rdf::Literal::new_simple_literal($obj)
        ); $($tt)*)
    };
    (@OBJ $sub:expr;$pred:expr; ulo:$obj:ident $($tt:tt)*) => {
        triple!(@MAYBEQUAD $sub;$pred;$crate::rdf::RDFTerm::NamedNode($crate::rdf::ontologies::ulo2::$obj.into_owned()); $($tt)*)
    };
    (@OBJ $sub:expr;$pred:expr; <($obj:expr)> $($tt:tt)*) => {
        triple!(@MAYBEQUAD $sub;$pred;$crate::rdf::RDFTerm::NamedNode($obj); $($tt)*)
    };
    (@OBJ $sub:expr;$pred:expr; ($obj:expr)! $($tt:tt)*) => {
        triple!(@MAYBEQUAD $sub;$pred;$crate::rdf::RDFTerm::BlankNode($obj); $($tt)*)
    };

    (@MAYBEQUAD $sub:expr;$pred:expr;$obj:expr;) => {
        $crate::rdf::Triple {
            subject: $sub,
            predicate: $pred,
            object: $obj
        }
    }
}

#[macro_export]
macro_rules! rdft {
    (> ($sub:expr) : $tp:ident) => {
        rdft!(($crate::rdf::NamedNode::new($sub).unwrap()) : $tp)
    };
    (($sub:expr) : $tp:ident) => {
        rdft!(@TRIPLE $sub; $crate::rdf::ontologies::rdf::TYPE.into_owned(); $crate::rdf::ontologies::ulo2::$tp.into_owned())
    };
    (($sub:expr) : ($tp:expr)) => {
        rdft!(@TRIPLE $sub; $crate::rdf::ontologies::rdf::TYPE.into_owned(); $tp)
    };
    (>($sub:expr) : $tp:ident Q) => {
        rdft!(($crate::rdf::NamedNode::new($sub).unwrap()) : $tp Q)
    };
    (($sub:expr) : $tp:ident Q) => {
        rdft!(@QUAD $sub; $crate::rdf::ontologies::rdf::TYPE.into_owned(); $crate::rdf::ontologies::ulo2::$tp.into_owned())
    };
    (>($sub:expr) : $tp:ident IN $graph:expr) => {
        rdft!(($crate::rdf::NamedNode::new($sub).unwrap()) : $tp IN $graph)
    };
    (($sub:expr) : >($tp:expr) IN $graph:expr) => {
        rdft!(@QUAD_IN $sub; $crate::rdf::ontologies::rdf::TYPE.into_owned(); $tp; $graph)
    };
    (($sub:expr) : $tp:ident IN $graph:expr) => {
        rdft!(@QUAD_IN $sub; $crate::rdf::ontologies::rdf::TYPE.into_owned(); $crate::rdf::ontologies::ulo2::$tp.into_owned(); $graph)
    };
    (($sub:expr) !($tp:expr) ($obj:expr) IN $graph:expr) => {
        rdft!(@QUAD_IN $sub; $tp.into_owned(); $obj; $graph)
    };
    (($sub:expr) !($tp:expr) ($obj:expr)) => {
        rdft!(@TRIPLE $sub; $tp.into_owned(); $obj)
    };

    (>($sub:expr) $tp:ident >($obj:expr) Q) => {
        rdft!(($crate::rdf::NamedNode::new($sub).unwrap()) $tp ($crate::rdf::NamedNode::new($obj).unwrap()) Q)
    };
    (($sub:expr) $tp:ident >($obj:expr) Q) => {
        rdft!(($sub) $tp ($crate::rdf::NamedNode::new($obj).unwrap()) Q)
    };
    (>($sub:expr) $tp:ident ($obj:expr) Q) => {
        rdft!(($crate::rdf::NamedNode::new($sub).unwrap()) $tp ($obj) Q)
    };
    (($sub:expr) $tp:ident ($obj:expr) Q) => {
        rdft!(@QUAD $sub; $crate::rdf::ontologies::ulo2::$tp.into_owned(); $obj)
    };
    (($sub:expr) $tp:ident ($obj:expr) IN $graph:expr) => {
        rdft!(@QUAD_IN $sub; $crate::rdf::ontologies::ulo2::$tp.into_owned(); $obj; $graph)
    };
    (($sub:expr) $tp:ident >>($obj:expr) IN $graph:expr) => {
        rdft!(@QUAD_IN $sub; $crate::rdf::ontologies::ulo2::$tp.into_owned(); >>$obj; $graph)
    };
    (($sub:expr) $tp:ident >>($obj:expr)) => {
        rdft!(@TRIPLE $sub; $crate::rdf::ontologies::ulo2::$tp.into_owned(); >>$obj)
    };
    (>>($sub:expr) $tp:ident ($obj:expr) IN $graph:expr) => {
        rdft!(@QUAD_IN >>$sub; $crate::rdf::ontologies::ulo2::$tp.into_owned(); $obj; $graph)
    };
    (>>($sub:expr) $tp:ident ($obj:expr)) => {
        rdft!(@TRIPLE >>$sub; $crate::rdf::ontologies::ulo2::$tp.into_owned(); $obj)
    };
    (>($sub:expr) $tp:ident >($obj:expr)) => {
        rdft!(($crate::rdf::NamedNode::new($sub).unwrap()) $tp ($crate::rdf::NamedNode::new($obj).unwrap()))
    };
    (($sub:expr) $tp:ident >($obj:expr)) => {
        rdft!(($sub) $tp ($crate::rdf::NamedNode::new($obj).unwrap()))
    };
    (>($sub:expr) $tp:ident ($obj:expr)) => {
        rdft!(($crate::rdf::NamedNode::new($sub).unwrap()) $tp ($obj))
    };
    (($sub:expr) $tp:ident ($obj:expr)) => {
        rdft!(@TRIPLE $sub; $crate::rdf::ontologies::ulo2::$tp.into_owned(); $obj)
    };
    (($sub:expr) ($tp:expr) = ($obj:expr) IN $graph:expr) => {
        $crate::rdf::Quad {
            subject: $crate::rdf::Subject::NamedNode($sub),
            predicate: $tp.into(),
            object: $crate::rdf::Term::Literal(
                $crate::rdf::Literal::new_simple_literal($obj)
            ),
            graph_name: $crate::rdf::GraphName::NamedNode($graph)
        }
    };

    (@TRIPLE >>$sub:expr; $pred:expr; $obj:expr) => {
        $crate::rdf::Triple {
            subject: $sub,
            predicate: $pred,
            object: $crate::rdf::Term::NamedNode($obj)
        }
    };
    (@TRIPLE $sub:expr; $pred:expr; >>$obj:expr) => {
        $crate::rdf::Triple {
            subject: $crate::rdf::Subject::NamedNode($sub),
            predicate: $pred,
            object: $obj
        }
    };
    (@TRIPLE $sub:expr; $pred:expr; $obj:expr) => {
        $crate::rdf::Triple {
            subject: $crate::rdf::Subject::NamedNode($sub),
            predicate: $pred,
            object: $crate::rdf::Term::NamedNode($obj)
        }
    };
    (@QUAD $sub:expr; $pred:expr; $obj:expr) => {
        $crate::rdf::Quad {
            subject: $crate::rdf::Subject::NamedNode($sub),
            predicate: $pred,
            object: $crate::rdf::Term::NamedNode($obj),
            graph_name: $crate::rdf::GraphName::DefaultGraph
        }
    };
    (@QUAD_IN $sub:expr; $pred:expr; >>$obj:expr; $graph:expr) => {
        $crate::rdf::Quad {
            subject: $crate::rdf::Subject::NamedNode($sub),
            predicate: $pred,
            object: $obj,
            graph_name: $crate::rdf::GraphName::NamedNode($graph)
        }
    };
    (@QUAD_IN >>$sub:expr; $pred:expr; $obj:expr; $graph:expr) => {
        $crate::rdf::Quad {
            subject: $sub,
            predicate: $pred,
            object: $crate::rdf::Term::NamedNode($obj),
            graph_name: $crate::rdf::GraphName::NamedNode($graph)
        }
    };
    (@QUAD_IN $sub:expr; $pred:expr; $obj:expr; $graph:expr) => {
        $crate::rdf::Quad {
            subject: $crate::rdf::Subject::NamedNode($sub),
            predicate: $pred,
            object: $crate::rdf::Term::NamedNode($obj),
            graph_name: $crate::rdf::GraphName::NamedNode($graph)
        }
    };
}

pub mod ontologies {
    /*! # RDF Ontology Summary
     *
     * #### [`Document`](crate::narration::documents::Document) `D`
     * | struct | field | triple |
     * | -----  | ----- | ------ |
     * |   |    | `D` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#document>`](ulo2::DOCUMENT) |
     * |   | language `l` | `D` [`<dc:#language>`](dc::LANGUAGE) `l` |
     * |   | in archive `A`  | `A` [`<ulo:#contains>`](ulo2::CONTAINS) `D` |
     * | [`DocumentReference`](crate::narration::DocumentElement::DocumentReference) | [`.target`](crate::narration::DocumentElement::DocumentReference::target)`=D2` | `D` [`<dc:#hasPart>`](dc::HAS_PART) `D2` |
     * | [`UseModule`](crate::narration::DocumentElement::UseModule) | `(M)` | `D` [`<dc:#requires>`](dc::REQUIRES) `M` |
     * | [`Paragraph`](crate::narration::paragraphs::LogicalParagraph) |   | `D` [`<ulo:#contains>`](ulo2::CONTAINS) `P` |
     * |   | `P`[`.kind`](crate::narration::paragraphs::LogicalParagraph::kind)`=`[`Definition`](crate::narration::paragraphs::ParagraphKind::Definition) | `P` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#definition>`](ulo2::DEFINITION) |
     * |   | `P`[`.kind`](crate::narration::paragraphs::LogicalParagraph::kind)`=`[`Assertion`](crate::narration::paragraphs::ParagraphKind::Assertion) | `P` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#proposition>`](ulo2::PROPOSITION) |
     * |   | `P`[`.kind`](crate::narration::paragraphs::LogicalParagraph::kind)`=`[`Paragraph`](crate::narration::paragraphs::ParagraphKind::Paragraph) | `P` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#para>`](ulo2::PARA) |
     * |   | `P`[`.kind`](crate::narration::paragraphs::LogicalParagraph::kind)`=`[`Example`](crate::narration::paragraphs::ParagraphKind::Example) | `P` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#example>`](ulo2::EXAMPLE) |
     * |   | is [`Example`](crate::narration::paragraphs::ParagraphKind::Example) and `_`[`.fors`](crate::narration::paragraphs::LogicalParagraph::fors)`.contains(S)`  | `P` [`<ulo:#example-for>`](ulo2::EXAMPLE_FOR) `S` |
     * |   | [`is_definition_like`](crate::narration::paragraphs::ParagraphKind::is_definition_like) and  `_`[`.fors`](crate::narration::paragraphs::LogicalParagraph::fors)`.contains(S)`  | `P` [`<ulo:#defines>`](ulo2::DEFINES) `S` |
     * | [`Problem`](crate::narration::problems::Problem) `E` |   | `D` [`<ulo:#contains>`](ulo2::CONTAINS) `E` |
     * |   | [`.sub_problem`](crate::narration::problems::Problem::sub_problem)`==false`   | `E` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#problem>`](ulo2::PROBLEM) |
     * |   | [`.sub_problem`](crate::narration::problems::Problem::sub_problem)`==true`   | `E` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#subproblem>`](ulo2::SUBPROBLEM) |
     * |   | `_`[`.preconditions`](crate::narration::problems::Problem::preconditions)`.contains(d,S)`  | `E` [`<ulo:#precondition>`](ulo2::PRECONDITION) `<BLANK>` |
     * |   |    | `<BLANK>` [`<ulo:#cognitive-dimension>`](ulo2::COGDIM) `d`, where `d=`[`<ulo:#cs-remember>`](ulo2::REMEMBER)⏐[`<ulo:#cs-understand>`](ulo2::UNDERSTAND)⏐[`<ulo:#cs-apply>`](ulo2::APPLY)⏐[`<ulo:#cs-analyze>`](ulo2::ANALYZE)⏐[`<ulo:#cs-evaluate>`](ulo2::EVALUATE)⏐[`<ulo:#cs-create>`](ulo2::CREATE) |
     * |   |    | `<BLANK>` [`<ulo:#po-symbol>`](ulo2::POSYMBOL) `S` |
     * |   | `_`[`.objectives`](crate::narration::problems::Problem::objectives)`.contains(d,S)`  | `E` [`<ulo:#objective>`](ulo2::OBJECTIVE) `<BLANK>` |
     * |   |    | `<BLANK>` [`<ulo:#cognitive-dimension>`](ulo2::COGDIM) `d`, where `d=`[`<ulo:#cs-remember>`](ulo2::REMEMBER)⏐[`<ulo:#cs-understand>`](ulo2::UNDERSTAND)⏐[`<ulo:#cs-apply>`](ulo2::APPLY)⏐[`<ulo:#cs-analyze>`](ulo2::ANALYZE)⏐[`<ulo:#cs-evaluate>`](ulo2::EVALUATE)⏐[`<ulo:#cs-create>`](ulo2::CREATE) |
     * |   |    | `<BLANK>` [`<ulo:#po-symbol>`](ulo2::POSYMBOL) `S` |
     *
     * #### [`Module`](crate::content::modules::Module) `M`
     * | struct | field | triple |
     * | -----  | ----- | ------ |
     * |   |    | `D` [`<ulo:#contains>`](ulo2::CONTAINS) `M` |
     * |   |    | `M` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#theory>`](ulo2::THEORY) |
     * | [`Import`](crate::content::declarations::OpenDeclaration::Import) | `(M2)` | `M` [`<ulo:#imports>`](ulo2::IMPORTS) `M2` |
     * | [`NestedModule`](crate::content::declarations::OpenDeclaration::NestedModule) | `(M2)` | `D` [`<ulo:#contains>`](ulo2::CONTAINS) `M2` |
     * |   |    | `M` [`<ulo:#contains>`](ulo2::CONTAINS) `M2` |
     * |   |    | `M2` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#theory>`](ulo2::THEORY) |
     * | [`MathStructure`](crate::content::declarations::OpenDeclaration::MathStructure) | `(S)` | `M` [`<ulo:#contains>`](ulo2::CONTAINS) `S` |
     * |   |    | `S` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#structure>`](ulo2::STRUCTURE) |
     * |   | [`Import`](crate::content::declarations::OpenDeclaration::Import)(`S2`)   | `S` [`<ulo:#extends>`](ulo2::EXTENDS) `S2` |
     * | [`Morphism`](crate::content::declarations::OpenDeclaration::Morphism) | `(F)` | `M` [`<ulo:#contains>`](ulo2::CONTAINS) `F` |
     * |   |    | `F` [`<rdf:#type>`](rdf::TYPE) [`<ulo:#morphism>`](ulo2::MORPHISM) |
     * |   | [`.domain`](crate::content::declarations::morphisms::Morphism)`=M2`   | `F` [`<rdfs:#domain>`](rdfs::DOMAIN) `M2` |
     *
     *
     *
     * # Some Example Queries
     *
     * #### Unused files in `ARCHIVE`:
     * All elements contained in the archive that are neither inputrefed elsewhere
     * nor (transitively) contain an element that is required or imported (=> is a module)
     * by another document:
     * ```sparql
     * SELECT DISTINCT ?f WHERE {
     *   <ARCHIVE> ulo:contains ?f .
     *   MINUS { ?d dc:hasPart ?f }
     *   MINUS {
     *     ?f ulo:contains+ ?m.
     *     ?d (dc:requires|ulo:imports) ?m.
     *   }
     * }
     * ```
     *
     * #### All referenced symbols in `DOCUMENT`:
     * All symbols referenced in an element that is transitively contained or inputrefed in
     * the document:
     * ```sparql
     * SELECT DISTINCT ?s WHERE {
     *   <DOCUMENT> (ulo:contains|dc:hasPart)* ?p.
     *   ?p ulo:crossrefs ?s.
     * }
     * ```
     *
     * #### All symbols defined in a `DOCUMENT`:
     * All symbols defined by a paragraph `p` that is transitively contained or inputrefed in
     * the document:
     * ```sparql
     * SELECT DISTINCT ?s WHERE {
     *   <DOCUMENT> (ulo:contains|dc:hasPart)* ?p.
     *   ?p ulo:defines ?s.
     * }
     * ```
     *
     * #### All "prerequisite" concepts in a `DOCUMENT`:
     * All symbols references in the document that are not also defined in it:
     * ```sparql
     * SELECT DISTINCT ?s WHERE {
     *   <DOCUMENT> (ulo:contains|dc:hasPart)* ?p.
     *   ?p ulo:crossrefs ?s.
     *   MINUS {
     *     <DOCUMENT> (ulo:contains|dc:hasPart)* ?p.
     *     ?p ulo:defines ?s.
     *   }
     * }
     * ```
     *
     *
     */

    /*

     use crate::content::declarations::OpenDeclaration::Symbol;
     use crate::narration::problems::Problem;
     use crate::narration::paragraphs::ParagraphKind::Definition;
     use crate::content::declarations::morphisms::Morphism;

    section:


    symdecl:
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

    vardecl:
    #[cfg(feature="rdf")]
    if E::RDF {
        let iri = uri.to_iri();
        extractor.add_triples([
            triple!(<(iri.clone())> : ulo:VARIABLE),
            triple!(<(extractor.get_document_iri())> ulo:DECLARES <(iri)>),
        ]);
    }

    notation:
    #[cfg(feature="rdf")]
    if E::RDF {
        let iri = uri.to_iri();
        extractor.add_triples([
            triple!(<(iri.clone())> : ulo:NOTATION),
            triple!(<(iri.clone())> ulo:NOTATION_FOR <(symbol.to_iri())>),
            triple!(<(extractor.get_document_iri())> ulo:DECLARES <(iri)>),
        ]);
    }

    symref:
    #[cfg(feature="rdf")]
    if E::RDF {
        let iri = extractor.get_document_iri();
        extractor.add_triples([
            triple!(<(iri)> ulo:CROSSREFS <(uri.to_iri())>)
        ]);
    }

    varref:
    #[cfg(feature="rdf")]
    if E::RDF {
        let iri = extractor.get_document_iri();
        extractor.add_triples([
            triple!(<(iri)> ulo:CROSSREFS <(uri.to_iri())>)
        ]);
    }





     */

    pub mod rdf {
        pub use oxrdf::vocab::rdf::*;
    }
    pub mod rdfs {
        pub use oxrdf::vocab::rdfs::*;
    }
    pub mod xsd {
        pub use oxrdf::vocab::xsd::*;
    }
    macro_rules! count {
        () => (0usize);
        ( $e:expr; $($n:expr;)* ) => (1usize + count!($($n;)*));
    }

    macro_rules! dict {
        ($name:ident = $uri:literal: $(
            $(+ $i:ident = $l:literal)?
            $(DATAPROP $di:ident = $dl:literal $(<: $( $($dsup:ident)::*  ),* )? $(: $($dtp:ident)::* )? $(@ $dcl:literal)? )?
            $(OBJPROP $oi:ident = $ol:literal $(<: $( $($osup:ident)::*  ),* )? $(( $dom:ident => $range:ident ))? $(- $inv:ident)? $(! $disj:ident)? $(@ $ocl:literal)? )?
            $(CLASS $ci:ident = $cl:literal $(<: $( $($csup:ident)::*  ),* )? $(= $left:ident u $right:ident)? $(@ $ccl:literal)? )?
            $({
              $($subj_n:ident)::*
              <$($pred_n:ident)::*>
              $(<$($obj_n:ident)::*>)?
              $(S $obj_str:literal)?
            })?
        ;)*) => {
            dict!{@old $name = $uri;
                $($($i = $l,)?)* $($($di = $dl,)?)* $($($oi = $ol,)?)* $($($ci = $cl,)?)*;
                $(
                    $( // dict!(@triple $($subj_n)::*;$($pred_n)::*; $(NAME $($obj_n)::*)? $(STRING $obj_str)? );
                          SubjectRef::NamedNode($($subj_n)::*),
                          $($pred_n)::*,
                          $(RDFTermRef::NamedNode($($obj_n)::*))?
                          $(RDFTermRef::Literal(LiteralRef::new_simple_literal($obj_str)))?
                    ;)?
                    $(  //dict!(@tp $ci;super::owl::DATATYPE_PROPERTY);
                        SubjectRef::NamedNode($di),
                        super::rdf::TYPE,
                        RDFTermRef::NamedNode(super::owl::DATATYPE_PROPERTY);
                        $(//dict!(@comment $di;$dcl);
                            SubjectRef::NamedNode($di),
                            super::rdfs::COMMENT,
                            RDFTermRef::Literal(LiteralRef::new_simple_literal($dcl));
                        )?
                        $(
                            SubjectRef::NamedNode($di),
                            super::rdfs::RANGE,
                            RDFTermRef::NamedNode($($dtp)::*);
                        )?
                        $($(//dict!(@subprop $oi;$($osup)::*);
                            SubjectRef::NamedNode($di),
                            super::rdfs::SUB_PROPERTY_OF,
                            RDFTermRef::NamedNode($($dsup)::*);
                        )*)?
                    )?
                    $(  //dict!(@tp $ci;super::owl::OBJECT_PROPERTY);
                        SubjectRef::NamedNode($oi),
                        super::rdf::TYPE,
                        RDFTermRef::NamedNode(super::owl::OBJECT_PROPERTY);
                        $(//dict!(@comment $oi;$ocl);
                            SubjectRef::NamedNode($oi),
                            super::rdfs::COMMENT,
                            RDFTermRef::Literal(LiteralRef::new_simple_literal($ocl));
                        )?
                        $(
                            SubjectRef::NamedNode($oi),
                            super::rdfs::DOMAIN,
                            RDFTermRef::NamedNode($dom);
                            SubjectRef::NamedNode($oi),
                            super::rdfs::RANGE,
                            RDFTermRef::NamedNode($range);
                        )?
                        $(
                            SubjectRef::NamedNode($oi),
                            super::owl::INVERSE_OF,
                            RDFTermRef::NamedNode($inv);
                        )?
                        $(
                            SubjectRef::NamedNode($oi),
                            super::owl::DISJOINT_WITH,
                            RDFTermRef::NamedNode($disj);
                        )?
                        $($(//dict!(@subprop $oi;$($osup)::*);
                            SubjectRef::NamedNode($oi),
                            super::rdfs::SUB_PROPERTY_OF,
                            RDFTermRef::NamedNode($($osup)::*);
                        )*)?
                    )?
                    $(  //dict!(@tp $ci;super::owl::CLASS);
                        SubjectRef::NamedNode($ci),
                        super::rdf::TYPE,
                        RDFTermRef::NamedNode(super::owl::CLASS);
                        $(//dict!(@comment $ci;$ccl);
                            SubjectRef::NamedNode($ci),
                            super::rdfs::COMMENT,
                            RDFTermRef::Literal(LiteralRef::new_simple_literal($ccl));
                        )?
                        $(
                            SubjectRef::NamedNode($left),
                            super::owl::DISJOINT_WITH,
                            RDFTermRef::NamedNode($right);
                            SubjectRef::NamedNode($left),
                            super::owl::COMPLEMENT_OF,
                            RDFTermRef::NamedNode($right);
                        )?
                        $($(//dict!(@subclass $ci;$($csup)::*);
                            SubjectRef::NamedNode($ci),
                            super::rdfs::SUB_CLASS_OF,
                            RDFTermRef::NamedNode($($csup)::*);
                        )*)?
                    )?
                )*
            }
        };
        (@triple $($subj:ident)::*;$($pred:ident)::*;$(NAME $($obj_n:ident)::*)? $(STRING $obj_str:literal)? ) => {dict!(@quad
              SubjectRef::NamedNode($($subj)::*);
              $($pred_n)::*;
              $(TermRef::NamedNode($($obj_n)::*))?
              $(TermRef::Literal(LiteralRef::new_simple_literal($obj_str)))?
        )};
        (@tp $i:ident;$($tp:ident)::*) => {dict!(@quad
            SubjectRef::NamedNode($i);
            super::rdfs::SUB_CLASS_OF;
            TermRef::NamedNode($($tp)::*)
        )};
        (@subprop $i:ident;$($sup:ident)::*) => {dict!(@quad
            SubjectRef::NamedNode($i);
            super::rdfs::SUB_PROPERTY_OF;
            TermRef::NamedNode($($sup)::*)
        )};
        (@subclass $i:ident;$($sup:ident)::*) => {dict!(@quad
            SubjectRef::NamedNode($i);
            super::rdfs::SUB_CLASS_OF;
            TermRef::NamedNode($($sup)::*)
        )};
        (@class $i:ident;) => {dict!(@quad
            SubjectRef::NamedNode($ci);
            super::rdf::TYPE;
            TermRef::NamedNode(super::owl::CLASS)
        )};
        (@comment $i:ident;$c:literal;) => {dict!(@quad
            SubjectRef::NamedNode($i);
            super::rdfs::COMMENT;
            TermRef::Literal(LiteralRef::new_simple_literal($c))
        )};
        (@quad $sub:expr;$pred:expr;$obj:expr) => {
            QuadRef{ subject:$sub,predicate:$pred,object:$obj,graph_name:GraphNameRef::NamedNode(NS) }
        };
        (@final $name:ident = $uri:literal;
            $($i:ident = $l:literal,)*;
            $($($quad:tt)*;)*
        ) => {
            pub mod $name {
                #![doc=concat!("`",$uri,"`")]
                use super::super::terms::*;
                #[doc=concat!("`",$uri,"`")]
                pub const NS : NamedNodeRef = NamedNodeRef::new_unchecked($uri);
                $(
                    #[doc=concat!("`",$uri,"#",$l,"`")]
                    pub const $i : NamedNodeRef = NamedNodeRef::new_unchecked(concat!($uri,"#",$l));
                )*

                pub static QUADS :&[QuadRef;count!($( $($quad)*; )*)] = &[$( $($quad)* ),*];
            }
        };
        (@old $name:ident = $uri:literal;
            $($i:ident = $l:literal,)*;
            $($sub:expr,$pred:expr,$obj:expr;)*
        ) => {
            pub mod $name {
                #![doc=concat!("`",$uri,"`")]
                use super::super::*;
                #[doc=concat!("`",$uri,"`")]
                pub const NS : NamedNodeRef = NamedNodeRef::new_unchecked($uri);
                $(
                    #[doc=concat!("`",$uri,"#",$l,"`")]
                    pub const $i : NamedNodeRef = NamedNodeRef::new_unchecked(concat!($uri,"#",$l));
                )*

                pub static QUADS :&[QuadRef;count!($($sub;)*)] = &[$(QuadRef{
                    subject:$sub,predicate:$pred,object:$obj,graph_name:GraphNameRef::NamedNode(NS)
                }),*];
            }
        }
    }

    dict! { dc = "http://purl.org/dc/elements/1.1":
        + RIGHTS = "rights";
        + LANGUAGE = "language";
        + HAS_PART = "hasPart";
        + REQUIRES = "requires";
    }

    dict! { owl = "http://www.w3.org/2002/07/owl":
        + OBJECT_PROPERTY = "ObjectProperty";
        + DATATYPE_PROPERTY = "DatatypeProperty";
        + CLASS = "Class";
        + DISJOINT_WITH = "disjointWith";
        + DISJOINT_UNION_OF = "disjointUnionOf";
        + COMPLEMENT_OF = "complementOf";
        + INVERSE_OF = "inverseOf";
        + SYMMETRIC_PROPERTY = "SymmetricProperty";
        + ASYMMETRIC_PROPERTY = "AsymmetricProperty";
        + TRANSITIVE_PROPERTY = "TransitiveProperty";
        + THING = "Thing";
        + FUNCTIONAL_PROPERTY = "FunctionalProperty";
    }

    dict! { ulo2 = "http://mathhub.info/ulo":
        { NS <super::dc::RIGHTS> S "This ontology is licensed under the CC-BY-SA license."};
        DATAPROP ORGANIZATIONAL = "organizational";


        CLASS PHYSICAL = "physical" @ "An organizational unit for the physical organization of \
            mathematical knowledge into documents or document collections.";
        CLASS FILE = "file" <: PHYSICAL @ "A document in a file system.";
        CLASS DOCUMENT = "document" <: PHYSICAL @ "A document; typically corresponding to a file.";
        CLASS FOLDER = "folder" <: PHYSICAL @ "A grouping of files and other folder, i.e. above the document level.";
        CLASS LIBRARY = "library" <: PHYSICAL @ "A grouping of mathematical documents. Usually in the \
            form of a repository.";
        CLASS LIBRARY_GROUP = "library-group" <: PHYSICAL @ "A group of libraries, usually on a \
            repository server like GitHub.";
        CLASS PARA = "para" <: PHYSICAL @ "A document paragraph with mathematical meaning.";
        CLASS PHRASE = "phrase" <: PHYSICAL @ "Phrasal structures in mathematical texts and formulae, \
            these include symbols, declarations, and quantifications.";
        CLASS SECTION = "section" <: PHYSICAL @ "A physical grouping inside a document. These can be nested.";
        CLASS DEFINITION = "definition" <: PARA @ "A logical paragraph that defines a new concept.";
        CLASS EXAMPLE = "example" <: PARA @ "A logical paragraph that introduces a mathematical example.";
        CLASS PROOF = "proof" <: PARA @ "A logical paragraph that serves as a justification of a proposition.";
        CLASS SUBPROOF = "subproof" <: PARA @ "A logical paragraph that serves as a justification of an\
         intermediate proposition within a proof.";
        CLASS PROPOSITION = "proposition" <: PARA @ "A statement of a mathematical object or some relation between some." ;
        CLASS PROBLEM = "problem" <: PARA @ "A logical paragraph posing an exercise/question/problem for the reader.";
        CLASS SUBPROBLEM = "subproblem" <: PARA @ "A logical paragraph posing a subproblem in some problem/question/problem for the reader.";


        // -----------------------------------------------------------------------------

        CLASS LOGICAL = "logical" = PRIMITIVE u LOGICAL @ "A logical classification of mathematical \
            knowledge items.";
        CLASS PRIMITIVE = "primitive" <: LOGICAL @ "This knowledge item does not have a definition in \
            terms of (more) primitive items." ;
        CLASS DERIVED = "derived" <: LOGICAL;
        CLASS THEORY = "theory" <: LOGICAL @ "A semantically meaningful block of declarations that can \
            be referred to globally. Examples include MMT theories, Mizar articles, Isabelle locales \
            and Coq sections.";
        CLASS STRUCTURE = "structure" <: LOGICAL @ "A semantically meaningful block of declarations that can \
            be instantiated by providing definientia for all (undefined) declarations.";
        CLASS MORPHISM = "morphism" <: LOGICAL @ "A semantically meaningful block of declarations that map \
            map declarations in the domain to expressions over the containing module";
        CLASS DECLARATION = "declaration" <: LOGICAL @ "Declarations are named objects. They can also \
            have a type and a definiens.";
        CLASS VARIABLE = "variable" <: LOGICAL @ "A local variable with optional type and definiens";
        CLASS NOTATION = "notation" <: LOGICAL @ "A way of representing (an application of) a symbol\
            for parsing or presentation.";
        CLASS STATEMENT = "statement" <: DECLARATION = AXIOM u THEOREM @ "Statements are declarations of \
            objects that can in principle have proofs.";
        CLASS AXIOM = "axiom" <: STATEMENT @ "Logically (using the Curry-Howard isomorphism), an axiom \
            is a primitive statement, i.e. a declaration without a definiens.";
        CLASS THEOREM = "theorem" <: STATEMENT @ "Logically (using the Curry-Howard isomorphism), a \
            theorem is a derived statement, i.e. a declaration with a definiens (this is the proof of \
            the theorem given in the type)";
        CLASS FUNCTION_DECL = "function-declaration" <: DECLARATION, FUNCTION;
        CLASS FUNCTION = "function" <: LOGICAL @ "Functions that construct objects, possibly from other \
            objects, for example in first-order logic the successor function.";
        CLASS TYPE_DECL = "type-declaration" <: DECLARATION, TYPE;
        CLASS TYPE = "type" <: LOGICAL @ "Types divide their universe into named subsets.";
        CLASS UNIVERSE_DECL = "universe-declaration" <: DECLARATION, UNIVERSE;
        CLASS UNIVERSE = "universe" <: LOGICAL @ "A universe, used e.g. in strong logics like Coq.";
        CLASS PREDICATE = "predicate" <: FUNCTION @ "A predicate is a mathematical object that \
            evaluates to true/false when applied to enough arguments.";
        CLASS RULE = "rule" <: STATEMENT @  "Rules are statements that can be used for computation, \
            e.g. theorems that can be used for simplification.";

        OBJPROP IMPORTS = "imports" (LOGICAL => LOGICAL);
        { IMPORTS <super::rdf::TYPE> <super::owl::TRANSITIVE_PROPERTY>};

        // -----------------------------------------------------------------------------

        OBJPROP CONTAINS = "contains" (PHYSICAL => PHYSICAL);
        OBJPROP DECLARES = "declares" (LOGICAL => LOGICAL);

        OBJPROP SPECIFIES = "specifies" (PHYSICAL => LOGICAL) -SPECIFIED_IN @ "The physical organizational \
            item S specifies a knowledge item O, i.e. S is represented in O.";
        OBJPROP SPECIFIED_IN = "specified-in" (LOGICAL => PHYSICAL) -SPECIFIES;
        OBJPROP CROSSREFS = "crossrefs";
        OBJPROP ALIGNED_WITH = "aligned-with" <: CROSSREFS;
        { ALIGNED_WITH <super::rdf::TYPE> <super::owl::SYMMETRIC_PROPERTY>};
        OBJPROP ALTERNATIVE_FOR = "alternative-for" <: CROSSREFS;
        OBJPROP INSPIRED_BY = "inspired-by" <: CROSSREFS;
        OBJPROP SAME_AS = "same-as" <: CROSSREFS;
        { SAME_AS <super::rdf::TYPE> <super::owl::SYMMETRIC_PROPERTY>};
        OBJPROP SEE_ALSO = "see-also" <: CROSSREFS;
        OBJPROP SIMILAR_TO = "similar-to" <: CROSSREFS;
        { SIMILAR_TO <super::rdf::TYPE> <super::owl::SYMMETRIC_PROPERTY>};

        OBJPROP INTER_STATEMENT = "inter-statement";
        OBJPROP CONSTRUCTS = "constructs" <: INTER_STATEMENT @ "S is a constructor for an inductive type or predicate O";
        OBJPROP EXTENDS = "extends" <: INTER_STATEMENT @ "S is a conservative extension of O";

        OBJPROP EXAMPLE_FOR = "example-for" <: INTER_STATEMENT !COUNTER_EXAMPLE_FOR;
        OBJPROP COUNTER_EXAMPLE_FOR = "counter-example-for" <: INTER_STATEMENT !EXAMPLE_FOR;
        OBJPROP DEFINES = "defines" <: INTER_STATEMENT (DEFINITION => FUNCTION) @ "A definition defines various objects.";

        OBJPROP GENERATED_BY = "generated-by" <: INTER_STATEMENT (FUNCTION => FUNCTION);
        OBJPROP INDUCTIVE_ON = "inductive-on" <: INTER_STATEMENT;
        OBJPROP JUSTIFIES = "justifies" <: INTER_STATEMENT;
        { JUSTIFIES <super::rdfs::DOMAIN> <PROOF>};
        OBJPROP NOTATION_FOR = "notation-for" <: INTER_STATEMENT;
        { NOTATION_FOR <super::rdfs::DOMAIN> <NOTATION>};

        OBJPROP PRECONDITION = "precondition";
        OBJPROP OBJECTIVE = "objective";

        OBJPROP COGDIM = "cognitive-dimension";
        OBJPROP POSYMBOL = "po-symbol";

        // -----------------------------------------------------------------------------

        OBJPROP NYMS = "nyms";
        OBJPROP ANTONYM = "antonym" <: NYMS;
        OBJPROP HYPONYM = "hyponym" <: NYMS;
        OBJPROP HYPERNYM = "hypernym" <: NYMS -HYPONYM;

        // -----------------------------------------------------------------------------

        OBJPROP FORMALIZES = "formalizes";
        OBJPROP USES = "uses" (STATEMENT => FUNCTION);
        { USES <super::rdfs::RANGE> <TYPE>};
        { USES <super::rdf::TYPE> <super::owl::TRANSITIVE_PROPERTY>};

        OBJPROP INSTANCE_OF = "instance-of" @ "S is an instance of O iff it is a model of O, iniherits \
            from O, interprets O, etc.";

        OBJPROP SUPERSEDED_BY = "superseded-by" @ "S (a deprecated knowledge item) is superseded by another.";
        { SUPERSEDED_BY <super::rdf::TYPE> <super::owl::TRANSITIVE_PROPERTY>};

        // -----------------------------------------------------------------------------

        DATAPROP SIZE_PROPERTIES = "size-properties";
        { SIZE_PROPERTIES <super::rdfs::DOMAIN> <super::owl::THING>};
        { SIZE_PROPERTIES <super::rdf::TYPE> <super::owl::FUNCTIONAL_PROPERTY>};

        DATAPROP AUTOMATICALLY_PROVED = "automatically-proved" <: ORGANIZATIONAL : super::xsd::STRING
            @ "S is automatically proven by a theorem prover, O is an explanatory string.";
        DATAPROP CHECK_TIME = "check-time" <: SIZE_PROPERTIES : super::xsd::DAY_TIME_DURATION
            @ "time it took to check the declaration that introduced the subject.";
        { CHECK_TIME <super::rdfs::DOMAIN> <FUNCTION>};
        { CHECK_TIME <super::rdfs::DOMAIN> <TYPE>};
        DATAPROP DEPRECATED = "deprecated" <: ORGANIZATIONAL : super::xsd::STRING
            @ "S is deprecated (do not use any longer), O is an explanatory string.";
        DATAPROP LAST_CHECKED_AT = "last-checked-at" <: SIZE_PROPERTIES : super::xsd::DATE_TIME_STAMP
            @ "The time stamp of when the subject was last checked.";
        { LAST_CHECKED_AT <super::rdfs::DOMAIN> <FUNCTION>};
        { LAST_CHECKED_AT <super::rdfs::DOMAIN> <TYPE>};
        DATAPROP SOURCEREF = "sourceref" : super::xsd::ANY_URI @ "The URI of the physical \
            location (e.g. file/URI, line, column) of the source code that introduced the subject.";

        + REMEMBER = "cd-remember";
        + UNDERSTAND = "cd-understand";
        + APPLY = "cd-apply";
        + ANALYZE = "cd-analyze";
        + EVALUATE = "cd-evaluate";
        + CREATE = "cd-create";
    }
}
