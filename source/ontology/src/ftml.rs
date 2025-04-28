#[cfg(doc)]
use crate::{
    content::{
        declarations::{
            structures::{Extension, MathStructure},
            symbols::Symbol,
        },
        modules::{Module, NestedModule},
    },
    narration::{
        documents::Document,
        paragraphs::{LogicalParagraph, ParagraphKind},
        problems::Problem,
        sections::{Section, SectionLevel},
        DocumentElement::Slide,
    },
    uris::{Name, PathURI},
};

pub const PREFIX: &str = "data-ftml-";

macro_rules! ftml {
    ($l:literal) => {
        concat!("data-ftml-", $l)
    };
}

macro_rules! do_keys {
    ( $count:literal: $(
        $(#[$meta:meta])*
        $tag:ident
        $(-? $otp:ty;)?
        $(@ $tp:ty)?
        =$val:literal
        $(+ $($other:ident),+ ;)*
        $(- $($req:ident),+ ;)*
        $(! $only:literal;)?
        $(-! $not:literal;)?
    )*
    ) => {

        pub const NUM_RULES: usize = $count;

        #[derive(Copy,Clone,PartialEq, Eq,Hash)]
        pub enum FTMLKey {
            $(
                #[doc = concat!(
                    "<div class=\"flams-syntax\">\n\n`data-ftml-",
                        $val
                        $(,"`[`=\"`<[",stringify!($otp),"]>`\"`]`")?
                        $(,"=\"`<[",stringify!($tp),"]>`\"")?
                        ,"`"
                        $(,
                            "\n\nAdditional attributes: " $(,
                                "[" ,stringify!($other),"](FTMLKey::",stringify!($other), "), "
                            )*
                            ,""
                        )?
                        $(,
                            "\n\nAttribute of: " $(,
                                "[" ,stringify!($req),"](FTMLKey::",stringify!($req), "), "
                            )*
                            ,""
                        )?
                    ,"\n\n</div>\n\n"
                    $(
                        , "<div class=\"warning\">\n\n*Only allowed "
                        , $only,
                        "*\n\n</div>\n\n"
                    )?
                    $(
                        , "<div class=\"warning\">\n\n*Not allowed "
                        , $not,
                        "*\n\n</div>\n\n"
                    )?
                )]
                $(#[$meta])*
                $tag
            ),*
        }

        paste::paste! {
            mod attrstrings {$(
                pub const [<$tag:snake:upper>]:&'static str
                    = ftml!($val);
            )*}
            impl FTMLKey {
                #[must_use]#[inline]
                pub const fn as_str(self) -> &'static str {
                    match self {$(
                        Self::$tag => $val
                    ),*}
                }

                #[must_use]#[inline]
                pub const fn attr_name(self) -> &'static str {
                    match self {$(
                        Self::$tag => attrstrings::[<$tag:snake:upper>]
                    ),*}
                }
            }
        }
    }
}

do_keys! {119:

    /// Denotes a new [Section]. The given [SectionLevel] is only a sanity check;
    /// the actual level is determined by the occurrence within a [Document].
    Section                     @ SectionLevel  = "section"         + Id; -!"in [LogicalParagraph]s, [Problem]s or [Slide]s";

    /// Denotes a new [LogicalParagraph] of [ParagraphKind::Definition]
    /// for the given [Symbol]s using the given styles.
    Definition                              = "definition"          + Id,Inline,Fors,Styles;
    /// Denotes a new [LogicalParagraph] of [ParagraphKind::Assertion] (Theorems, Lemmata,
    /// Axioms, etc.) for the given [Symbol]s using the given styles.
    Assertion                               = "assertion"           + Id,Inline,Fors,Styles;
    /// Denotes a new [LogicalParagraph] of [ParagraphKind::Example] (this includes counterexamples)
    /// for the given [Symbol]s using the given styles.
    Example                                 = "example"             + Id,Inline,Fors,Styles;
    /// Denotes a new [LogicalParagraph] of [ParagraphKind::Paragraph]
    /// for the given [Symbol]s using the given styles.
    Paragraph                               = "paragraph"           + Id,Inline,Fors,Styles;

    /// Denotes a new [Problem] with [`sub_problem`](Problem::sub_problem)`=false`
    Problem                                 = "problem"             + Id,Styles,Autogradable,ProblemPoints ;
    /// Denotes a new [Problem] with [`sub_problem`](Problem::sub_problem)`=true`
    SubProblem                              = "subproblem"          + Id,Styles,Autogradable,ProblemPoints ;

    /// Denotes a [Slide], implying that the [Document] is (or contains in some sense)
    /// a presentation.
    Slide                                   = "slide"               + Id;    -!"in [LogicalParagraph]s, [Problem]s or [Slide]s";


    // --------------------------------------------------------------------------------

    /// A (possibly empty) node that, when being rendered, should be replaced by the
    /// current slide number.
    SlideNumber                 = "slide-number"            !"in [Slide]s";

    // ------------------------------------------------------------------------------------

    /// Denotes a new [Module] (or [NestedModule]) with the given [Name] in the
    /// current [Namespace](PathURI).
    Module                      @ String        = "module"              + Metatheory, Signature;

    /// Denotes a new [MathStructure] or [Extension] with the given [Name].
    MathStructure               @ String        = "feature-structure"   + Macroname; !"in [Module]s";

    /// <div class="flams-wip">TODO</div>
    Morphism                                    = "feature-morphism"

    Proof                                   = "proof"               + Id,Inline,Fors,Styles,ProofHide;
    SubProof                                = "subproof"            + Id,Inline,Fors,Styles,ProofHide;



    Style                       = "style"
    Counter                     = "counter"
    CounterParent               = "counter-parent"

    DocTitle                    = "doctitle"
    Title                       = "title"
    ProofTitle                  = "prooftitle"
    SubproofTitle               = "subprooftitle"

    Symdecl                     = "symdecl"
    Vardef                      = "vardef"
    Varseq                      = "varseq"

    Notation                    = "notation"
    NotationComp                = "notationcomp"
    NotationOpComp              = "notationopcomp"
    Definiendum                 = "definiendum"

    Type                        = "type"
    Conclusion                  = "conclusion"
    Definiens                   = "definiens"
    Rule                        = "rule"

    ArgSep                      = "argsep"
    ArgMap                      = "argmap"
    ArgMapSep                   = "argmap-sep"

    Term                        = "term"
    Arg                         = "arg"
    HeadTerm                    = "headterm"

    ImportModule                = "import"
    UseModule                   = "usemodule"
    InputRef                    = "inputref"

    SetSectionLevel             = "sectionlevel"
    SkipSection                 = "skipsection"


    ProofMethod                 = "proofmethod"
    ProofSketch                 = "proofsketch"
    ProofTerm                   = "proofterm"
    ProofBody                   = "proofbody"
    ProofAssumption             = "spfassumption"
    ProofStep                   = "spfstep"
    ProofStepName               = "stepname"
    ProofEqStep                 = "spfeqstep"
    ProofPremise                = "premise"
    ProofConclusion             = "spfconclusion"

    PreconditionDimension       = "preconditiondimension"
    PreconditionSymbol          = "preconditionsymbol"
    ObjectiveDimension          = "objectivedimension"
    ObjectiveSymbol             = "objectivesymbol"
    AnswerClass                 = "answerclass"
    AnswerClassPts              = "answerclass-pts"
    AnswerclassFeedback         = "answerclass-feedback"
    ProblemMinutes              = "problemminutes"
    ProblemMultipleChoiceBlock  = "multiple-choice-block"
    ProblemSingleChoiceBlock    = "single-choice-block"
    ProblemChoice               = "problem-choice"
    ProblemChoiceVerdict        = "problem-choice-verdict"
    ProblemChoiceFeedback       = "problem-choice-feedback"
    ProblemFillinsol            = "fillinsol"
    ProblemFillinsolWidth       = "fillinsol-width"
    ProblemFillinsolCase        = "fillin-case"
    ProblemFillinsolCaseValue   = "fillin-case-value"
    ProblemFillinsolCaseVerdict = "fillin-case-verdict"
    ProblemSolution            = "solution"
    ProblemHint                = "problemhint"
    ProblemNote                 = "problemnote"
    ProblemGradingNote         = "problemgnote"

    Comp                        = "comp"
    VarComp                     = "varcomp"
    MainComp                    = "maincomp"
    DefComp                     = "defcomp"

    Invisible                   = "invisible"

    IfInputref                  = "ifinputref"
    ReturnType                  = "returntype"
    ArgTypes                    = "argtypes"

    SRef                        = "sref"
    SRefIn                      = "srefin"
    Slideshow                   = "slideshow"
    SlideshowSlide              = "slideshow-slide"
    CurrentSectionLevel         = "currentsectionlevel"
    Capitalize                  = "capitalize"

    Assign                      = "assign"
    Rename                      = "rename"
    RenameTo                    = "to"
    AssignMorphismFrom          = "assignmorphismfrom"
    AssignMorphismTo            = "assignmorphismto"

    AssocType                   = "assoctype"
    ArgumentReordering          = "reorderargs"
    ArgNum                      = "argnum"
    Bind                        = "bind"
    MorphismDomain              = "domain"
    MorphismTotal               = "total"
    ArgMode                     = "argmode"
    NotationId                  = "notationid"
    Head                        = "head"
    Language                    = "language"
    /// The metatheory of a module, that provides the formal "language" the module
    /// is in
    Metatheory                              = "metatheory"      - Module;
    Signature                               = "signature"       - Module;
    Args                        = "args"
    ProblemPoints               = "problempoints"               - Problem, SubProblem;
    Autogradable                = "autogradable"                - Problem, SubProblem;
    ProofHide                   = "proofhide"                   - Proof,SubProof;
    Macroname                   = "macroname"                   - MathStructure;
    Inline                      = "inline"                      - Definition, Paragraph, Assertion, Example, Problem, SubProblem;
    Fors                        = "fors"                        - Definition, Paragraph, Assertion, Example, Proof, SubProof;
    Id                          = "id"                          - Section,Definition, Paragraph, Assertion, Example, Proof, SubProof, Problem, SubProblem, Slide;
    NotationFragment            = "notationfragment"
    Precedence                  = "precedence"
    Role                        = "role"
    Styles                      = "styles"
    Argprecs                    = "argprecs"
}

impl std::fmt::Display for FTMLKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::fmt::Debug for FTMLKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.attr_name())
    }
}
