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
                      oxrdf::SubjectRef::NamedNode($($subj_n)::*),
                      $($pred_n)::*,
                      $(oxrdf::TermRef::NamedNode($($obj_n)::*))?
                      $(oxrdf::TermRef::Literal(oxrdf::LiteralRef::new_simple_literal($obj_str)))?
                ;)?
                $(  //dict!(@tp $ci;super::owl::DATATYPE_PROPERTY);
                    oxrdf::SubjectRef::NamedNode($di),
                    oxrdf::vocab::rdf::TYPE,
                    oxrdf::TermRef::NamedNode(super::owl::DATATYPE_PROPERTY);
                    $(//dict!(@comment $di;$dcl);
                        oxrdf::SubjectRef::NamedNode($di),
                        oxrdf::vocab::rdfs::COMMENT,
                        oxrdf::TermRef::Literal(oxrdf::LiteralRef::new_simple_literal($dcl));
                    )?
                    $(
                        oxrdf::SubjectRef::NamedNode($di),
                        oxrdf::vocab::rdfs::RANGE,
                        oxrdf::TermRef::NamedNode($($dtp)::*);
                    )?
                    $($(//dict!(@subprop $oi;$($osup)::*);
                        oxrdf::SubjectRef::NamedNode($di),
                        oxrdf::vocab::rdfs::SUB_PROPERTY_OF,
                        oxrdf::TermRef::NamedNode($($dsup)::*);
                    )*)?
                )?
                $(  //dict!(@tp $ci;super::owl::OBJECT_PROPERTY);
                    oxrdf::SubjectRef::NamedNode($oi),
                    oxrdf::vocab::rdf::TYPE,
                    oxrdf::TermRef::NamedNode(super::owl::OBJECT_PROPERTY);
                    $(//dict!(@comment $oi;$ocl);
                        oxrdf::SubjectRef::NamedNode($oi),
                        oxrdf::vocab::rdfs::COMMENT,
                        oxrdf::TermRef::Literal(oxrdf::LiteralRef::new_simple_literal($ocl));
                    )?
                    $(
                        oxrdf::SubjectRef::NamedNode($oi),
                        oxrdf::vocab::rdfs::DOMAIN,
                        oxrdf::TermRef::NamedNode($dom);
                        oxrdf::SubjectRef::NamedNode($oi),
                        oxrdf::vocab::rdfs::RANGE,
                        oxrdf::TermRef::NamedNode($range);
                    )?
                    $(
                        oxrdf::SubjectRef::NamedNode($oi),
                        super::owl::INVERSE_OF,
                        oxrdf::TermRef::NamedNode($inv);
                    )?
                    $(
                        oxrdf::SubjectRef::NamedNode($oi),
                        super::owl::DISJOINT_WITH,
                        oxrdf::TermRef::NamedNode($disj);
                    )?
                    $($(//dict!(@subprop $oi;$($osup)::*);
                        oxrdf::SubjectRef::NamedNode($oi),
                        oxrdf::vocab::rdfs::SUB_PROPERTY_OF,
                        oxrdf::TermRef::NamedNode($($osup)::*);
                    )*)?
                )?
                $(  //dict!(@tp $ci;super::owl::CLASS);
                    oxrdf::SubjectRef::NamedNode($ci),
                    oxrdf::vocab::rdf::TYPE,
                    oxrdf::TermRef::NamedNode(super::owl::CLASS);
                    $(//dict!(@comment $ci;$ccl);
                        oxrdf::SubjectRef::NamedNode($ci),
                        oxrdf::vocab::rdfs::COMMENT,
                        oxrdf::TermRef::Literal(oxrdf::LiteralRef::new_simple_literal($ccl));
                    )?
                    $(
                        oxrdf::SubjectRef::NamedNode($left),
                        super::owl::DISJOINT_WITH,
                        oxrdf::TermRef::NamedNode($right);
                        oxrdf::SubjectRef::NamedNode($left),
                        super::owl::COMPLEMENT_OF,
                        oxrdf::TermRef::NamedNode($right);
                    )?
                    $($(//dict!(@subclass $ci;$($csup)::*);
                        oxrdf::SubjectRef::NamedNode($ci),
                        oxrdf::vocab::rdfs::SUB_CLASS_OF,
                        oxrdf::TermRef::NamedNode($($csup)::*);
                    )*)?
                )?
            )*
        }
    };
    (@triple $($subj:ident)::*;$($pred:ident)::*;$(NAME $($obj_n:ident)::*)? $(STRING $obj_str:literal)? ) => {dict!(@quad
          oxrdf::SubjectRef::NamedNode($($subj)::*);
          $($pred_n)::*;
          $(oxrdf::TermRef::NamedNode($($obj_n)::*))?
          $(oxrdf::TermRef::Literal(oxrdf::LiteralRef::new_simple_literal($obj_str)))?
    )};
    (@tp $i:ident;$($tp:ident)::*) => {dict!(@quad
        oxrdf::SubjectRef::NamedNode($i);
        oxrdf::vocab::rdfs::SUB_CLASS_OF;
        oxrdf::TermRef::NamedNode($($tp)::*)
    )};
    (@subprop $i:ident;$($sup:ident)::*) => {dict!(@quad
        oxrdf::SubjectRef::NamedNode($i);
        oxrdf::vocab::rdfs::SUB_PROPERTY_OF;
        oxrdf::TermRef::NamedNode($($sup)::*)
    )};
    (@subclass $i:ident;$($sup:ident)::*) => {dict!(@quad
        oxrdf::SubjectRef::NamedNode($i);
        oxrdf::vocab::rdfs::SUB_CLASS_OF;
        oxrdf::TermRef::NamedNode($($sup)::*)
    )};
    (@class $i:ident;) => {dict!(@quad
        oxrdf::SubjectRef::NamedNode($ci);
        oxrdf::vocab::rdf::TYPE;
        oxrdf::TermRef::NamedNode(super::owl::CLASS)
    )};
    (@comment $i:ident;$c:literal;) => {dict!(@quad
        oxrdf::SubjectRef::NamedNode($i);
        oxrdf::vocab::rdfs::COMMENT;
        oxrdf::TermRef::Literal(oxrdf::LiteralRef::new_simple_literal($c))
    )};
    (@quad $sub:expr;$pred:expr;$obj:expr) => {
        QuadRef{ subject:$sub,predicate:$pred,object:$obj,graph_name:oxrdf::GraphNameRef::NamedNode(NS) }
    };
    (@final $name:ident = $uri:literal;
        $($i:ident = $l:literal,)*;
        $($($quad:tt)*;)*
    ) => {
        pub mod $name {
            use oxrdf::{NamedNodeRef, QuadRef};
            pub static NS : NamedNodeRef = NamedNodeRef::new_unchecked($uri);
            $(
                pub static $i : NamedNodeRef = NamedNodeRef::new_unchecked(concat!($uri,"#",$l));
            )*

            pub static QUADS :&[QuadRef;count!($( $($quad)*; )*)] = &[$( $($quad)* ),*];
        }
    };
    (@old $name:ident = $uri:literal;
        $($i:ident = $l:literal,)*;
        $($sub:expr,$pred:expr,$obj:expr;)*
    ) => {
        pub mod $name {
            use oxrdf::{NamedNodeRef, QuadRef};
            pub static NS : NamedNodeRef = NamedNodeRef::new_unchecked($uri);
            $(
                pub static $i : NamedNodeRef = NamedNodeRef::new_unchecked(concat!($uri,"#",$l));
            )*

            pub static QUADS :&[QuadRef;count!($($sub;)*)] = &[$(QuadRef{
                subject:$sub,predicate:$pred,object:$obj,graph_name:oxrdf::GraphNameRef::NamedNode(NS)
            }),*];
        }
    }
}

dict! { dc = "http://purl.org/dc/elements/1.1":
    + RIGHTS = "rights";
}

dict!{ owl = "http://www.w3.org/2002/07/owl":
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

dict!{ ulo2 = "http://mathhub.info/ulo":
    { NS <super::dc::RIGHTS> S "This ontology is licensed under the CC-BY-SA license."};
    DATAPROP ORGANIZATIONAL = "organizational";


    CLASS PHYSICAL = "physical" @ "An organizational unit for the physical organization of \
        mathematical knowledge into documents or document collections.";
    CLASS FILE = "file" <: PHYSICAL @ "A document in a file system.";
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
    CLASS PROPOSITION = "proposition" <: PARA @ "A statement of a mathematical object or some relation between some." ;

    // -----------------------------------------------------------------------------

    CLASS LOGICAL = "logical" = PRIMITIVE u LOGICAL @ "A logical classification of mathematical \
        knowledge items.";
    CLASS PRIMITIVE = "primitive" <: LOGICAL @ "This knowledge item does not have a definition in \
        terms of (more) primitive items." ;
    CLASS DERIVED = "derived" <: LOGICAL;
    CLASS THEORY = "theory" <: LOGICAL @ "A semantically meaningful block of declarations that can \
        be referred to globally. Examples include MMT theories, Mizar articles, Isabelle locales \
        and Coq sections.";
    CLASS DECLARATION = "declaration" <: LOGICAL @ "Declarations are named objects. They can also \
        have a type and a definiens.";
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

    // -----------------------------------------------------------------------------

    OBJPROP CONTAINS = "contains" (PHYSICAL => PHYSICAL);
    OBJPROP DECLARES = "declares" (LOGICAL => LOGICAL);

    OBJPROP SPECIFIES = "specifies" (PHYSICAL => LOGICAL) -SPECIFIED_IN @ "The physical organizational \
        item S specifies a knowledge item O, i.e. S is represented in O.";
    OBJPROP SPECIFIED_IN = "specified-in" (LOGICAL => PHYSICAL) -SPECIFIES;
    OBJPROP CROSSREFS = "crossrefs";
    OBJPROP ALIGNED_WITH = "aligned-with" <: CROSSREFS;
    { ALIGNED_WITH <oxrdf::vocab::rdf::TYPE> <super::owl::SYMMETRIC_PROPERTY>};
    OBJPROP ALTERNATIVE_FOR = "alternative-for" <: CROSSREFS;
    OBJPROP INSPIRED_BY = "inspired-by" <: CROSSREFS;
    OBJPROP SAME_AS = "same-as" <: CROSSREFS;
    { SAME_AS <oxrdf::vocab::rdf::TYPE> <super::owl::SYMMETRIC_PROPERTY>};
    OBJPROP SEE_ALSO = "see-also" <: CROSSREFS;
    OBJPROP SIMILAR_TO = "similar-to" <: CROSSREFS;
    { SIMILAR_TO <oxrdf::vocab::rdf::TYPE> <super::owl::SYMMETRIC_PROPERTY>};

    OBJPROP INTER_STATEMENT = "inter-statement";
    OBJPROP CONSTRUCTS = "constructs" <: INTER_STATEMENT @ "S is a constructor for an inductive type or predicate O";
    OBJPROP EXAMPLE_FOR = "example-for" <: INTER_STATEMENT !COUNTER_EXAMPLE_FOR;
    OBJPROP COUNTER_EXAMPLE_FOR = "counter-example-for" <: INTER_STATEMENT !EXAMPLE_FOR;

    OBJPROP DEFINES = "defines" <: INTER_STATEMENT (DEFINITION => FUNCTION) @ "A definition defines various objects.";
    OBJPROP GENERATED_BY = "generated-by" <: INTER_STATEMENT (FUNCTION => FUNCTION);
    OBJPROP INDUCTIVE_ON = "inductive-on" <: INTER_STATEMENT;
    OBJPROP JUSTIFIES = "justifies" <: INTER_STATEMENT;
    { JUSTIFIES <oxrdf::vocab::rdfs::DOMAIN> <PROOF>};

    // -----------------------------------------------------------------------------

    OBJPROP NYMS = "nyms";
    OBJPROP ANTONYM = "antonym" <: NYMS;
    OBJPROP HYPONYM = "hyponym" <: NYMS;
    OBJPROP HYPERNYM = "hypernym" <: NYMS -HYPONYM;

    // -----------------------------------------------------------------------------

    OBJPROP FORMALIZES = "formalizes";
    OBJPROP USES = "uses" (STATEMENT => FUNCTION);
    { USES <oxrdf::vocab::rdfs::RANGE> <TYPE>};
    { USES <oxrdf::vocab::rdf::TYPE> <super::owl::TRANSITIVE_PROPERTY>};

    OBJPROP INSTANCE_OF = "instance-of" @ "S is an instance of O iff it is a model of O, iniherits \
        from O, interprets O, etc.";

    OBJPROP SUPERSEDED_BY = "superseded-by" @ "S (a deprecated knowledge item) is superseded by another.";
    { SUPERSEDED_BY <oxrdf::vocab::rdf::TYPE> <super::owl::TRANSITIVE_PROPERTY>};

    // -----------------------------------------------------------------------------

    DATAPROP SIZE_PROPERTIES = "size-properties";
    { SIZE_PROPERTIES <oxrdf::vocab::rdfs::DOMAIN> <super::owl::THING>};
    { SIZE_PROPERTIES <oxrdf::vocab::rdf::TYPE> <super::owl::FUNCTIONAL_PROPERTY>};

    DATAPROP AUTOMATICALLY_PROVED = "automatically-proved" <: ORGANIZATIONAL : oxrdf::vocab::xsd::STRING
        @ "S is automatically proven by a theorem prover, O is an explanatory string.";
    DATAPROP CHECK_TIME = "check-time" <: SIZE_PROPERTIES : oxrdf::vocab::xsd::DAY_TIME_DURATION
        @ "time it took to check the declaration that introduced the subject.";
    { CHECK_TIME <oxrdf::vocab::rdfs::DOMAIN> <FUNCTION>};
    { CHECK_TIME <oxrdf::vocab::rdfs::DOMAIN> <TYPE>};
    DATAPROP DEPRECATED = "deprecated" <: ORGANIZATIONAL : oxrdf::vocab::xsd::STRING
        @ "S is deprecated (do not use any longer), O is an explanatory string.";
    DATAPROP LAST_CHECKED_AT = "last-checked-at" <: SIZE_PROPERTIES : oxrdf::vocab::xsd::DATE_TIME_STAMP
        @ "The time stamp of when the subject was last checked.";
    { LAST_CHECKED_AT <oxrdf::vocab::rdfs::DOMAIN> <FUNCTION>};
    { LAST_CHECKED_AT <oxrdf::vocab::rdfs::DOMAIN> <TYPE>};
    DATAPROP SOURCEREF = "sourceref" : oxrdf::vocab::xsd::ANY_URI @ "The URI of the physical \
        location (e.g. file/URI, line, column) of the source code that introduced the subject.";
}
