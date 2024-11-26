#![allow(clippy::must_use_candidate)]
const MML_TAGS: [&str; 31] = [
    "math",
    "mi",
    "mn",
    "mo",
    "ms",
    "mspace",
    "mtext",
    "menclose",
    "merror",
    "mfenced",
    "mfrac",
    "mpadded",
    "mphantom",
    "mroot",
    "mrow",
    "msqrt",
    "mstyle",
    "mmultiscripts",
    "mover",
    "mprescripts",
    "msub",
    "msubsup",
    "msup",
    "munder",
    "munderover",
    "mtable",
    "mtd",
    "mtr",
    "maction",
    "annotation",
    "semantics",
];

#[must_use]
pub fn is(tag: &str) -> Option<&'static str> {
    MML_TAGS
        .iter()
        .find(|e| tag.eq_ignore_ascii_case(e))
        .copied()
}

use leptos::prelude::*;

#[component]
pub fn MathMLTag(tag: &'static str, children: Children) -> impl IntoView {
    match tag {
        "math" => view!(<math>{children()}</math>).into_any(),
        "mi" => view!(<mi>{children()}</mi>).into_any(),
        "mn" => view!(<mn>{children()}</mn>).into_any(),
        "mo" => view!(<mo>{children()}</mo>).into_any(),
        "ms" => view!(<ms>{children()}</ms>).into_any(),
        "mspace" => view!(<mspace>{children()}</mspace>).into_any(),
        "mtext" => view!(<mtext>{children()}</mtext>).into_any(),
        "menclose" => view!(<menclose>{children()}</menclose>).into_any(),
        "merror" => view!(<merror>{children()}</merror>).into_any(),
        "mfenced" => view!(<mfenced>{children()}</mfenced>).into_any(),
        "mfrac" => view!(<mfrac>{children()}</mfrac>).into_any(),
        "mpadded" => view!(<mpadded>{children()}</mpadded>).into_any(),
        "mphantom" => view!(<mphantom>{children()}</mphantom>).into_any(),
        "mroot" => view!(<mroot>{children()}</mroot>).into_any(),
        "mrow" => view!(<mrow>{children()}</mrow>).into_any(),
        "msqrt" => view!(<msqrt>{children()}</msqrt>).into_any(),
        "mstyle" => view!(<mstyle>{children()}</mstyle>).into_any(),
        "mmultiscripts" => view!(<mmultiscripts>{children()}</mmultiscripts>).into_any(),
        "mover" => view!(<mover>{children()}</mover>).into_any(),
        "mprescripts" => view!(<mprescripts>{children()}</mprescripts>).into_any(),
        "msub" => view!(<msub>{children()}</msub>).into_any(),
        "msubsup" => view!(<msubsup>{children()}</msubsup>).into_any(),
        "msup" => view!(<msup>{children()}</msup>).into_any(),
        "munder" => view!(<munder>{children()}</munder>).into_any(),
        "munderover" => view!(<munderover>{children()}</munderover>).into_any(),
        "mtable" => view!(<mtable>{children()}</mtable>).into_any(),
        "mtd" => view!(<mtd>{children()}</mtd>).into_any(),
        "mtr" => view!(<mtr>{children()}</mtr>).into_any(),
        "maction" => view!(<maction>{children()}</maction>).into_any(),
        "annotation" => view!(<annotation>{children()}</annotation>).into_any(),
        "semantics" => view!(<semantics>{children()}</semantics>).into_any(),
        _ => view!(<mrow>{children()}</mrow>).into_any(),
    }
}