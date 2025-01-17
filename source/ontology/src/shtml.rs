pub const PREFIX:&str = "data-shtml-";

macro_rules! shtml { ($l:literal) => {concat!("data-shtml-",$l)} }

macro_rules! do_keys {
    ($count:literal: $($tag:ident =$val:literal)*) => {

        pub const NUM_RULES: usize = $count;

        #[derive(Copy,Clone,PartialEq, Eq,Hash)]
        pub enum SHTMLKey {
            $(
                #[doc = shtml!($val)]
                $tag
            ),*
        }

        paste::paste! {
            mod attrstrings {$(
                pub const [<$tag:snake:upper>]:&'static str
                    = shtml!($val);
            )*}
            impl SHTMLKey {
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


do_keys!{115:
    Module                      = "theory"
    MathStructure               = "feature-structure"
    Morphism                    = "feature-morphism"
    Section                     = "section"

    Definition                  = "definition"
    Paragraph                   = "paragraph"
    Assertion                   = "assertion"
    Example                     = "example"
    Problem                     = "problem"
    SubProblem                  = "subproblem"

    DocTitle                    = "doctitle"
    Title                       = "title"

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


    Proof                       = "proof"
    SubProof                    = "subproof"
    ProofMethod                 = "proofmethod"
    ProofSketch                 = "proofsketch"
    ProofTerm                   = "proofterm"
    ProofBody                   = "proofbody"
    ProofAssumption             = "spfassumption"
    ProofHide                   = "proofhide"
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
    ProblemFillinsolCase        = "fillin-case"
    ProblemFillinsolCaseValue   = "fillin-case-value"
    ProblemFillinsolCaseVerdict = "fillin-case-verdict"
    ProblemFillinsolValue       = "fillin-value"
    ProblemFillinsolVerdict     = "fillin-verdict"
    ExerciseSolution            = "solution"
    ExerciseHint                = "problemhint"
    ProblemNote                 = "problemnote"
    ExerciseGradingNote         = "problemgnote"

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
    Frame                       = "frame"
    FrameNumber                 = "framenumber"
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
    ProblemPoints               = "problempoints"
    Autogradable                = "autogradable"
    MorphismDomain              = "domain"
    MorphismTotal               = "total"
    ArgMode                     = "argmode"
    NotationId                  = "notationid"
    Head                        = "head"
    Language                    = "language"
    Metatheory                  = "metatheory"
    Signature                   = "signature"
    Args                        = "args"
    Macroname                   = "macroname"
    Inline                      = "inline"
    Fors                        = "fors"
    Id                          = "id"
    NotationFragment            = "notationfragment"
    Precedence                  = "precedence"
    Role                        = "role"
    Styles                      = "styles"
    Argprecs                    = "argprecs"
}

impl std::fmt::Display for SHTMLKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::fmt::Debug for SHTMLKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.attr_name())
    }
}