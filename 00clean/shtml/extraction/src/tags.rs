use std::fmt::{Debug, Display};

use crate::extractor::SHTMLExtractor;
use crate::rules::SHTMLExtractionRule;
use paste::paste;

macro_rules! shtml { ($l:literal) => {concat!("shtml:",$l)} }


macro_rules! do_tags {
    ($count:literal: $($tag:ident =$val:literal $(@$f:ident)?),*) => {
        #[derive(Copy,Clone,PartialEq, Eq,Hash)]
        pub enum SHTMLTag {
            $(
                #[doc = shtml!($val)]
                $tag
            ),*
        }
        paste! {
            pub mod tagstrings {$(
                pub const [<$tag:snake:upper>]:&'static str
                    = shtml!($val);
            )*}

            impl SHTMLTag {
                #[must_use]#[inline]
                pub const fn all_rules<E:SHTMLExtractor>() -> [SHTMLExtractionRule<E>;$count] {[$(
                    Self::$tag.rule()
                ),*]}
                #[must_use]#[inline]
                pub const fn rule<E:SHTMLExtractor>(self) -> SHTMLExtractionRule<E> {
                    match self {$(
                        Self::$tag =>
                            SHTMLExtractionRule::new(self,tagstrings::[<$tag:snake:upper>],do_tags!(@FUN $($f)?))
                    ),*}
                }

                #[must_use]#[inline]
                pub const fn as_str(self) -> &'static str {
                    match self {$(
                        Self::$tag => $val
                    ),*}
                }

                #[must_use]#[inline]
                pub const fn attr_name(self) -> &'static str {
                    match self {$(
                        Self::$tag => tagstrings::[<$tag:snake:upper>]
                    ),*}
                }

            }
        }

    };
    (@FUN None) => {Self::no_op};
    (@FUN $i:ident) => {Self::$i};
    (@FUN ) => {SHTMLTag::todo}
}

do_tags!{117:
    Module = "theory" @ module,
    MathStructure = "feature-structure",
    Morphism = "feature-morphism",
    Section = "section",

    Definition = "definition",
    Paragraph = "paragraph",
    Assertion = "assertion",
    Example = "example",
    Problem = "problem",
    SubProblem = "subproblem",

    DocTitle = "doctitle",
    SectionTitle = "sectiontitle",
    StatementTitle = "statementtitle",
    ProofTitle = "prooftitle",
    ProblemTitle = "problemtitle",

    Symdecl = "symdecl",
    Vardef = "vardef",
    Varseq = "varseq",

    Notation = "notation",
    NotationComp = "notationcomp",
    NotationOpComp = "notationopcomp",
    Definiendum = "definiendum",

    Type = "type",
    Conclusion = "conclusion",
    Definiens = "definiens",
    Rule = "rule",

    ArgSep = "argsep",
    ArgMap = "argmap",
    ArgMapSep = "argmap-sep",

    Term = "term" @ term,
    Arg = "arg" @ arg,
    HeadTerm = "headterm",

    ImportModule = "import",
    UseModule = "usemodule",
    InputRef = "inputref" @ inputref,

    SetSectionLevel = "sectionlevel",
    SkipSection = "skipsection",

    ArgNume = "argnum",

    Proof = "proof",
    SubProof = "subproof",
    ProofMethod = "proofmethod",
    ProofSketch = "proofsketch",
    ProofTerm = "proofterm",
    ProofBody = "proofbody",
    ProofAssumption = "spfassumption",
    ProofHide = "proofhide",
    ProofStep = "spfstep",
    ProofStepName = "stepname",
    ProofEqStep = "spfeqstep",
    ProofPremise = "premise",
    ProofConclusion = "spfconclusion",

    PreconditionDimension = "preconditiondimension",
    PreconditionSymbol = "preconditionsymbol",
    ObjectiveDimension = "objectivedimension",
    ObjectiveSymbol = "objectivesymbol",
    AnswerClass = "answerclass",
    AnswerClassPts = "answerclass-pts",
    AnswerclassFeedback = "answerclass-feedback",
    ProblemMinutes = "problemminutes",
    ProblemMultipleChoiceBlock = "multiple-choice-block",
    ProblemSingleChoiceBlock = "single-choice-block",
    ProblemMCC = "mcc",
    ProblemMCCSolution = "mcc-solution",
    ProblemSCC = "scc",
    ProblemSCCSolution = "scc-solution",
    ProblemFillinsol = "fillinsol",
    ProblemFillinsolCase = "fillin-case",
    ProblemFillinsolCaseValue = "fillin-case-value",
    ProblemFillinsolCaseVerdict = "fillin-case-verdict",
    ProblemFillinsolValue = "fillin-value",
    ProblemVillinsolVerdict = "fillin-verdict",
    Solution = "solution",
    ProblemHint = "problemhint",
    ProblemNote = "problemnote",
    ProblemGradingNote = "problemgnote",
    Autogradable = "autogradable",

    Comp = "comp" @ comp,
    VarComp = "varcomp" @ comp,
    MainComp = "maincomp" @ maincomp,

    AssocType = "assoctype",
    ArgumentReordering = "reorderargs",
    Bind = "bind",

    Invisible = "visible",

    IfInputref = "ifinputref" @ ifinputref,
    ReturnType = "returntype",
    ArgTypes = "argtypes",

    SRef = "sref",
    SRefIn = "srefin",
    Frame = "frame",
    FrameNumber = "framenumber",
    Slideshow = "slideshow",
    SlideshowSlide = "slideshow-slide",
    CurrentSectionLevel = "currentsectionlevel",
    Capitalize = "capitalize",
    Styles = "styles",
    
    Assign = "assign",
    MorphismDomain = "domain",
    MorphismTotal = "total",
    Rename = "rename",
    RenameTo = "to",
    AssignMorphismFrom = "assignmorphismfrom",
    AssignMorphismTo = "assignmorphismto",

    ArgMode = "argmode" @ no_op,
    NotationId = "notationid" @no_op,
    Head = "head" @no_op,
    Language = "language" @no_op,
    Metatheory = "metatheory" @no_op,
    Signature = "signature" @no_op,
    Args = "args" @no_op,
    Macroname = "macroname" @no_op,
    Inline = "inline" @no_op,
    Fors = "fors" @no_op,
    Id = "id" @no_op,
    NotationFragment = "notationfragment" @no_op,
    Precedence = "precedence" @no_op,
    Role = "role" @no_op,
    Argprecs = "argprecs" @no_op
}

impl SHTMLTag {
    pub const fn ignore<E:SHTMLExtractor>(self) -> SHTMLExtractionRule<E> {
        SHTMLExtractionRule::new(self,self.attr_name(),Self::no_op)
    }
}

impl Display for SHTMLTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Debug for SHTMLTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.attr_name())
    }
}