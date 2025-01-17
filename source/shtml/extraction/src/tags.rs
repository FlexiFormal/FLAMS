use std::fmt::{Debug, Display};

use crate::extractor::SHTMLExtractor;
use crate::open::OpenSHTMLElement;
use crate::rules::SHTMLExtractionRule;
use immt_ontology::shtml::SHTMLKey;
use paste::paste;

macro_rules! do_tags {
    ($($tag:ident $(@$f:ident)?),*) => {
        paste! {
            //impl SHTMLTagExt for SHTMLKey {
                #[must_use]#[inline]
                pub const fn all_rules<E:SHTMLExtractor>() -> [SHTMLExtractionRule<E>;immt_ontology::shtml::NUM_RULES] {[$(
                    rule(SHTMLKey::$tag)
                ),*]}
                #[must_use]#[inline]
                pub const fn rule<E:SHTMLExtractor>(key:SHTMLKey) -> SHTMLExtractionRule<E> {
                    match key {$(
                        SHTMLKey::$tag =>
                            SHTMLExtractionRule::new(key,SHTMLKey::$tag.attr_name(),do_tags!(@FUN $tag $($f)?))
                    ),*}
                }
            //}
        }

    };
    (@FUN $tag:ident None) => {no_op};
    (@FUN $tag:ident $i:ident) => {super::rules::rules::$i};
    (@FUN $tag:ident ) => {|a,b,c| todo(a,b,c,SHTMLTag::$tag)}
}


do_tags!{
    Module                      @ module,
    MathStructure               @ mathstructure,
    Morphism                    @ morphism,   
    Section                     @ section,

    Definition                  @ definition,     
    Paragraph                   @ paragraph,
    Assertion                   @ assertion,
    Example                     @ example,
    Problem                     @ exercise,
    SubProblem                  @ subexercise,

    DocTitle                    @ doctitle,
    Title                       @ title,

    Symdecl                     @ symdecl,
    Vardef                      @ vardecl,
    Varseq                      @ varseq,

    Notation                    @ notation,
    NotationComp                @ notationcomp,
    NotationOpComp              @ notationopcomp,
    Definiendum                 @ definiendum,

    Type                        @ r#type,
    Conclusion                  @ conclusion,
    Definiens                   @ definiens,
    Rule                        @ mmtrule,

    ArgSep                      @ argsep,
    ArgMap                      @ argmap,
    ArgMapSep                   @ argmapsep,

    Term                        @ term,
    Arg                         @ arg,
    HeadTerm                    @ headterm,

    ImportModule                @ importmodule,
    UseModule                   @ usemodule,
    InputRef                    @ inputref,

    SetSectionLevel             @ setsectionlevel,
    SkipSection                 @ no_op /* TODO */,


    Proof                       @ proof,
    SubProof                    @ subproof,
    ProofMethod                 @ no_op /* TODO */,
    ProofSketch                 @ no_op /* TODO */,
    ProofTerm                   @ no_op /* TODO */,
    ProofBody                   @ no_op /* TODO */,
    ProofAssumption             @ no_op /* TODO */,
    ProofHide                   @ no_op /* TODO */,
    ProofStep                   @ no_op /* TODO */,
    ProofStepName               @ no_op /* TODO */,
    ProofEqStep                 @ no_op /* TODO */,
    ProofPremise                @ no_op /* TODO */,
    ProofConclusion             @ no_op /* TODO */,

    PreconditionDimension       @ precondition,
    PreconditionSymbol          @ no_op,
    ObjectiveDimension          @ objective,
    ObjectiveSymbol             @ no_op,
    ProblemMinutes              @ no_op /* TODO */,
    ProblemFillinsol            @ no_op /* TODO */,
    ProblemFillinsolCase        @ no_op /* TODO */,
    ProblemFillinsolCaseValue   @ no_op /* TODO */,
    ProblemFillinsolCaseVerdict @ no_op /* TODO */,
    ProblemFillinsolValue       @ no_op /* TODO */,
    ProblemFillinsolVerdict     @ no_op /* TODO */,

    ProblemNote                 @ no_op /* TODO */,
    ExerciseSolution            @ solution,
    ExerciseHint                @ problem_hint,
    ExerciseGradingNote         @ gnote,

    ProblemMultipleChoiceBlock  @ multiple_choice_block,
    ProblemSingleChoiceBlock    @ single_choice_block,
    ProblemChoice               @ problem_choice,
    ProblemChoiceVerdict        @ problem_choice_verdict,
    ProblemChoiceFeedback       @ problem_choice_feedback,

    AnswerClass                 @ answer_class,
    AnswerclassFeedback         @ ac_feedback,
    AnswerClassPts              @ no_op,

    Comp                        @ comp,
    VarComp                     @ comp,
    MainComp                    @ maincomp,
    DefComp                     @ defcomp,

    Invisible                   @ invisible,

    IfInputref                  @ ifinputref,
    ReturnType                  @ no_op /* TODO */,
    ArgTypes                    @ no_op /* TODO */,

    SRef                        @ no_op /* TODO */,
    SRefIn                      @ no_op /* TODO */,
    Frame                       @ no_op /* TODO */,
    FrameNumber                 @ no_op /* TODO */,
    Slideshow                   @ no_op /* TODO */,
    SlideshowSlide              @ no_op /* TODO */,
    CurrentSectionLevel         @ no_op /* TODO */,
    Capitalize                  @ no_op /* TODO */,
    
    Assign                      @ assign,
    Rename                      @ no_op /* TODO */,
    RenameTo                    @ no_op /* TODO */,
    AssignMorphismFrom          @ no_op /* TODO */,
    AssignMorphismTo            @ no_op /* TODO */,

    AssocType                   @ no_op,
    ArgumentReordering          @ no_op,
    ArgNum                      @ no_op,
    Bind                        @ no_op,
    ProblemPoints               @ no_op,
    Autogradable                @ no_op,
    MorphismDomain              @ no_op,
    MorphismTotal               @ no_op,
    ArgMode                     @ no_op,
    NotationId                  @ no_op,
    Head                        @ no_op,
    Language                    @ no_op,
    Metatheory                  @ no_op,
    Signature                   @ no_op,
    Args                        @ no_op,
    Macroname                   @ no_op,
    Inline                      @ no_op,
    Fors                        @ no_op,
    Id                          @ no_op,
    NotationFragment            @ no_op,
    Precedence                  @ no_op,
    Role                        @ no_op,
    Styles                      @ no_op,
    Argprecs                    @ no_op
}


pub const fn ignore<E:SHTMLExtractor>(key:SHTMLKey) -> SHTMLExtractionRule<E> {
    SHTMLExtractionRule::new(key,key.attr_name(),super::rules::rules::no_op)
}
pub const fn no_op<E:SHTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut super::rules::rules::SV<E>) -> Option<OpenSHTMLElement> { None }

pub fn todo<E:SHTMLExtractor>(_extractor:&mut E,_attrs:&mut E::Attr<'_>,_nexts:&mut super::rules::rules::SV<E>,tag:SHTMLKey) -> Option<OpenSHTMLElement> {
    todo!("Tag {}",tag.as_str()) 
}