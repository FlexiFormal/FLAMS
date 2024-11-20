use std::fmt::{Debug, Display};

use crate::extractor::SHTMLExtractor;
use crate::rules::SHTMLExtractionRule;
use paste::paste;

macro_rules! shtml { ($l:literal) => {concat!("data-shtml-",$l)} }


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
                            SHTMLExtractionRule::new(self,tagstrings::[<$tag:snake:upper>],do_tags!(@FUN $tag $($f)?))
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
    (@FUN $tag:ident None) => {Self::no_op};
    (@FUN $tag:ident $i:ident) => {Self::$i};
    (@FUN $tag:ident ) => {|a,b,c| SHTMLTag::todo(a,b,c,SHTMLTag::$tag)}
}

do_tags!{115:
    Module                  = "theory"              @ module,
    MathStructure           = "feature-structure"   @ mathstructure,
    Morphism                = "feature-morphism"    @ morphism,   
    Section                 = "section"             @ section,

    Definition              = "definition"          @ definition,     
    Paragraph               = "paragraph"           @ paragraph,
    Assertion               = "assertion"           @ assertion,
    Example                 = "example"             @ example,
    Problem                 = "problem"             @ exercise,
    SubProblem              = "subproblem"          @ subexercise,

    DocTitle                = "doctitle"            @ doctitle,
    Title                   = "title"               @ title,

    Symdecl                 = "symdecl"             @ symdecl,
    Vardef                  = "vardef"              @ vardecl,
    Varseq                  = "varseq"              @ varseq,

    Notation                = "notation"            @ notation,
    NotationComp            = "notationcomp"        @ notationcomp,
    NotationOpComp          = "notationopcomp"      @ notationopcomp,
    Definiendum             = "definiendum"         @ definiendum,

    Type                    = "type"                @ r#type,
    Conclusion              = "conclusion"          @ conclusion,
    Definiens               = "definiens"           @ definiens,
    Rule                    = "rule"                @ mmtrule,

    ArgSep                  = "argsep"              @ argsep,
    ArgMap                  = "argmap"              @ argmap,
    ArgMapSep               = "argmap-sep"          @ argmapsep,

    Term                    = "term"                @ term,
    Arg                     = "arg"                 @ arg,
    HeadTerm                = "headterm"            @ headterm,

    ImportModule            = "import"              @ importmodule,
    UseModule               = "usemodule"           @ usemodule,
    InputRef                = "inputref"            @ inputref,

    SetSectionLevel         = "sectionlevel"        @ setsectionlevel,
    SkipSection             = "skipsection"         @ no_op /* TODO */,


    Proof                   = "proof"               @ proof,
    SubProof                = "subproof"            @ subproof,
    ProofMethod             = "proofmethod"         @ no_op /* TODO */,
    ProofSketch             = "proofsketch"         @ no_op /* TODO */,
    ProofTerm               = "proofterm"           @ no_op /* TODO */,
    ProofBody               = "proofbody"           @ no_op /* TODO */,
    ProofAssumption         = "spfassumption"       @ no_op /* TODO */,
    ProofHide               = "proofhide"           @ no_op /* TODO */,
    ProofStep               = "spfstep"             @ no_op /* TODO */,
    ProofStepName           = "stepname"            @ no_op /* TODO */,
    ProofEqStep             = "spfeqstep"           @ no_op /* TODO */,
    ProofPremise            = "premise"             @ no_op /* TODO */,
    ProofConclusion         = "spfconclusion"       @ no_op /* TODO */,

    PreconditionDimension   = "preconditiondimension" @no_op /* TODO */,
    PreconditionSymbol      = "preconditionsymbol"  @no_op /* TODO */,
    ObjectiveDimension      = "objectivedimension"  @no_op /* TODO */,
    ObjectiveSymbol         = "objectivesymbol"     @no_op /* TODO */,
    AnswerClass             = "answerclass"         @no_op /* TODO */,
    AnswerClassPts          = "answerclass-pts"     @no_op /* TODO */,
    AnswerclassFeedback     = "answerclass-feedback" @no_op /* TODO */,
    ProblemMinutes          = "problemminutes"      @no_op /* TODO */,
    ProblemMultipleChoiceBlock = "multiple-choice-block"  @no_op /* TODO */,
    ProblemSingleChoiceBlock = "single-choice-block" @no_op /* TODO */,
    ProblemMCC              = "mcc"                 @no_op /* TODO */,
    ProblemMCCSolution      = "mcc-solution"        @no_op /* TODO */,
    ProblemSCC              = "scc"                 @no_op /* TODO */,
    ProblemSCCSolution      = "scc-solution"        @no_op /* TODO */,
    ProblemFillinsol        = "fillinsol"           @no_op /* TODO */,
    ProblemFillinsolCase    = "fillin-case"         @no_op /* TODO */,
    ProblemFillinsolCaseValue = "fillin-case-value" @no_op /* TODO */,
    ProblemFillinsolCaseVerdict = "fillin-case-verdict" @no_op /* TODO */,
    ProblemFillinsolValue   = "fillin-value"        @no_op /* TODO */,
    ProblemVillinsolVerdict = "fillin-verdict"      @no_op /* TODO */,
    Solution                = "solution"            @no_op /* TODO */,
    ProblemHint             = "problemhint"         @no_op /* TODO */,
    ProblemNote             = "problemnote"         @no_op /* TODO */,
    ProblemGradingNote      = "problemgnote"        @no_op /* TODO */,

    Comp                    = "comp"                @ comp,
    VarComp                 = "varcomp"             @ comp,
    MainComp                = "maincomp"            @ maincomp,


    Invisible               = "visible"             @ invisible,

    IfInputref              = "ifinputref"          @ ifinputref,
    ReturnType              = "returntype"          @no_op /* TODO */,
    ArgTypes                = "argtypes"            @no_op /* TODO */,

    SRef                    = "sref"                @no_op /* TODO */,
    SRefIn                  = "srefin"              @no_op /* TODO */,
    Frame                   = "frame"               @no_op /* TODO */,
    FrameNumber             = "framenumber"         @no_op /* TODO */,
    Slideshow               = "slideshow"           @no_op /* TODO */,
    SlideshowSlide          = "slideshow-slide"     @no_op /* TODO */,
    CurrentSectionLevel     = "currentsectionlevel" @no_op /* TODO */,
    Capitalize              = "capitalize"          @no_op /* TODO */,
    
    Assign                  = "assign"              @ assign,
    Rename                  = "rename"              @no_op /* TODO */,
    RenameTo                = "to"                  @no_op /* TODO */,
    AssignMorphismFrom      = "assignmorphismfrom"  @no_op /* TODO */,
    AssignMorphismTo        = "assignmorphismto"    @no_op /* TODO */,

    AssocType               = "assoctype"           @ no_op,
    ArgumentReordering      = "reorderargs"         @ no_op,
    ArgNum                  = "argnum"              @ no_op,
    Bind                    = "bind"                @ no_op,
    ProblemPoints           = "problempoints"       @ no_op,
    Autogradable            = "autogradable"        @ no_op,
    MorphismDomain          = "domain"              @ no_op,
    MorphismTotal           = "total"               @ no_op,
    ArgMode                 = "argmode"             @ no_op,
    NotationId              = "notationid"          @ no_op,
    Head                    = "head"                @ no_op,
    Language                = "language"            @ no_op,
    Metatheory              = "metatheory"          @ no_op,
    Signature               = "signature"           @ no_op,
    Args                    = "args"                @ no_op,
    Macroname               = "macroname"           @ no_op,
    Inline                  = "inline"              @ no_op,
    Fors                    = "fors"                @ no_op,
    Id                      = "id"                  @ no_op,
    NotationFragment        = "notationfragment"    @ no_op,
    Precedence              = "precedence"          @ no_op,
    Role                    = "role"                @ no_op,
    Styles                  = "styles"              @ no_op,
    Argprecs                = "argprecs"            @ no_op
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