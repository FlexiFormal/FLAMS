#![allow(clippy::must_use_candidate)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(any(
    all(feature = "ssr", feature = "hydrate", not(feature = "docs-only")),
    not(any(feature = "ssr", feature = "hydrate"))
))]
compile_error!("exactly one of the features \"ssr\" or \"hydrate\" must be enabled");

pub mod math;

use flams_router_content::Views;
use ftml_dom::{FtmlViews, utils::css::CssExt};
use ftml_ontology::utils::Css;
use ftml_uris::DocumentUri;
use leptos::prelude::*;

#[component]
pub fn FloDownEditor() -> AnyView {
    #[cfg(feature = "hydrate")]
    math::TeXClient::provide();

    Css::Link("/rustex.css".to_string().into_boxed_str()).inject();
    Css::Link(
        "https://fonts.googleapis.com/css2?family=STIX+Two+Text"
            .to_string()
            .into_boxed_str(),
    )
    .inject();

    let text = RwSignal::new(DEMO.to_string());
    let checked = RwSignal::new(false);

    let csr = RwSignal::new(false);
    #[cfg(feature = "hydrate")]
    let _ = Effect::new(move || csr.set(true));

    ftml_components::config::FtmlConfig::set_toc_source(ftml_dom::structure::TocSource::None);
    Views::setup_document::<flams_router_content::backend::FtmlBackend>(
        DocumentUri::no_doc().clone(),
        ftml_components::SidebarPosition::None,
        false,
        move || {
            view! {
                <div><input type="checkbox" on:change:target=move |ev| {
                      checked.set(ev.target().checked());
                }/>"LaTeX"</div>
                <div style="width:100%;display:flex;flex-direction:row;">
                    <textarea
                        style="width:50%;max-width:50%;min-width:50%;min-height:200px;"
                        on:input:target=move |ev| {
                            text.set(ev.target().value());
                        }
                        prop:value=text
                    />
                    <div
                        style="width:50%;max-width:50%;min-width:50%;text-align:left;border:1px solid black"
                        //inner_html=move || text.with(|txt| flodown::to_html(txt))
                    >
                        {move ||
                            if csr.get() {
                                Some(if checked.get() {
                                    view!(<pre>{text.with(|txt| flodown::to_latex(txt))}</pre>).into_any()
                                } else {
                                    md_html(text)
                                })
                            } else {
                                None
                            }
                        }
                    </div>
                </div>
            }.into_any()
        },
    )
}

fn md_html(md: RwSignal<String>) -> AnyView {
    let owner = leptos::prelude::Owner::current().expect("not in a reactive context");

    let actual = RwSignal::new(String::new());
    let signals = RwSignal::new(Vec::<(usize, RwSignal<Option<Result<String, String>>>)>::new());
    Effect::new(move || {
        #[cfg(feature = "hydrate")]
        {
            signals.update_untracked(Vec::clear);
            math::TeXClient::reset();
            let s = md.with(|txt| {
                flodown::to_html_with_math(
                    txt,
                    |s, out| {
                        use std::fmt::Write;
                        let (i, rs) = owner.with(|| math::TeXClient::inline_math(s));
                        signals.update_untracked(|v| v.push((i, rs)));
                        let _ = write!(out, "<!--math{i}--> ...");
                    },
                    |s, out| {
                        use std::fmt::Write;
                        let (i, rs) = owner.with(|| math::TeXClient::block_math(s));
                        signals.update_untracked(|v| v.push((i, rs)));
                        let _ = write!(out, "<!--math{i}--> ...");
                    },
                )
            });
            actual.set(s);
            signals.notify();
        }
    });
    Effect::new(move || {
        #[cfg(feature = "hydrate")]
        {
            signals.with(|v| {
                for (i, sig) in v {
                    if let Some(v) = sig.get() {
                        match v {
                            Ok(s) => actual.update_untracked(|a| {
                                *a = a.replace(&format!("<!--math{i}--> ..."), &s)
                            }),
                            Err(e) => actual.update_untracked(|a| {
                                *a = a.replace(
                                    &format!("<!--math{i}--> ..."),
                                    &format!("<span style=\"background-color:red;\">{e}</span>"),
                                );
                            }),
                        }
                    }
                }
            });
            actual.notify();
        }
    });
    (move || actual.with(|txt| Views::render_ftml(txt.clone(), None))).into_any()
}

static DEMO: &str = r#"
----
title: Foo Bar Baz
foo: blubb

symbols:
  cs: http://mathhub.info?a=smglom/cs&p=mod&m=computer-science&s=CS
  computer science: http://mathhub.info?a=smglom/cs&p=mod&m=computer-science&s=CS
----

::: definition title="Gödel's Incompleteness Theorem"
  foo @[sym](cs) @[sym](cs,computer science)
  @[def](cs)
:::

@[definition](this is a @[sym](cs) inline definition for @[def](cs,computer science))

a *b* **blubb** \*bla\* blubb _bla_ __blubb__ ~bla~ ^blubb^ ~~bla~~ ==blubb==
$inline math$ and $$block math$$ and such, and `inline code` and
```javascript
some
  code blocks
    with
  indentation
```

go visit [mathhub](https://mathhub.info)

- foo
  - bar

- blubb

1. btw
1. this works
  1. too
  1. and this
1) and this

foo bar

definition list
: foo bar baz lorem ipsum

another
: foo bar baz lorem ipsum

"#;
