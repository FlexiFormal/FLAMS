#![cfg_attr(docsrs, feature(doc_cfg))]

use flams_router_content::backend::FtmlBackend;
use flams_web_utils::components::{Header, Leaf, Subtree, Tree};
use ftml_components::components::content::FtmlViewable;
use ftml_dom::notations::TermExt;
use ftml_ontology::terms::{ComponentVar, Term, Variable};
pub use ftml_solver_trace::results::DocumentCheckResult;
use ftml_solver_trace::results::{CheckResult, ContentCheckResult, SymbolCheckResult};
use ftml_solver_trace::{CheckLog, Displayable};
use ftml_uris::{DocumentElementUri, DocumentUri, Uri};
use leptos::prelude::*;
use thaw::Text;

pub trait ResultExt {
    fn render(self) -> AnyView;
}
impl ResultExt for DocumentCheckResult {
    fn render(self) -> AnyView {
        flams_router_content::Views::setup_document::<FtmlBackend>(
            DocumentUri::no_doc().clone(),
            ftml_components::SidebarPosition::None,
            false,
            move || {
                let inner = self
                    .checks
                    .into_iter()
                    .map(CheckResult::render)
                    .collect_view();
                view! {<Tree>{inner}</Tree>}.into_any()
            },
        )
    }
}

impl ResultExt for CheckResult {
    fn render(self) -> AnyView {
        match self {
            Self::Missing(e) => view! {
                <Leaf><Text style="color:red;">
                    {do_success(false)}"Module not found: "
                    {e.as_view::<FtmlBackend>()}
                </Text></Leaf>
            }
            .into_any(),
            Self::Variable(v, r) => {
                let success = do_success(r.success());
                view! {
                    <Subtree expanded=!r.success()>
                        <Header slot>
                            <b>{success}"Variable "<math>{do_variable_uri(v)}</math></b>
                        </Header>
                        {
                            symbol_result(r)
                        }
                    </Subtree>
                }
                .into_any()
            }
            Self::Module { uri, checks } => {
                let success = checks.iter().all(ContentCheckResult::success);
                let succ = do_success(success);
                view! {
                    <Subtree expanded=!success>
                        <Header slot>
                            <b>{succ}"Module "{uri.as_view::<FtmlBackend>()}</b>
                        </Header>
                        {
                            checks.into_iter().map(|c| match c {
                                ContentCheckResult::Symbol(u, s) => {
                                    let success = s.success();
                                    let succ = do_success(success);
                                    view! {
                                        <Subtree expanded=!success>
                                            <Header slot>
                                                <b>{succ}"Symbol "{u.as_view::<FtmlBackend>()}</b>
                                            </Header>
                                            {
                                                symbol_result(s)
                                            }
                                        </Subtree>
                                    }
                                }
                            }).collect_view()
                        }
                    </Subtree>
                }
                .into_any()
            }
            Self::Term { uri, inferred, log } => {
                let success = do_success(inferred.is_some());
                view! {
                    <Subtree expanded=inferred.is_none()>
                        <Header slot>
                            <b>{success}"Document term "{uri.name().to_string()}</b>
                        </Header>
                        {
                            do_log(log, &mut Vec::new())
                        }
                    </Subtree>
                }
                .into_any()
            }
        }
    }
}

fn symbol_result(r: SymbolCheckResult) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    match r {
        SymbolCheckResult::TypeOnly { result }
        | SymbolCheckResult::Both {
            inhabitable: result,
            matches: None,
        } => Left(do_log(result.log, &mut Vec::new())),
        SymbolCheckResult::DefiniensOnly { log, .. } => Left(do_log(log, &mut Vec::new())),
        SymbolCheckResult::Both {
            inhabitable,
            matches: Some(r),
        } => Right(view! {
            <Subtree expanded=!inhabitable.success>
                <Header slot><Text><b>"Type"</b></Text></Header>
                {do_log(inhabitable.log, &mut Vec::new())}
            </Subtree>
            <Subtree expanded=!r.success>
                <Header slot><Text><b>"Definiens"</b></Text></Header>
                {do_log(r.log, &mut Vec::new())}
            </Subtree>
        }),
    }
}

#[allow(clippy::too_many_lines)]
fn do_log(log: CheckLog, ctx: &mut Vec<ComponentVar>) -> AnyView {
    match log {
        CheckLog::Comment(s) => {
            view! {<Leaf><Text style="color:cadetblue"><i>{s}</i></Text></Leaf>}.into_any()
        }
        CheckLog::Emph(s) => view! {<Leaf><Text><b>{s}</b></Text></Leaf>}.into_any(),
        CheckLog::Header(s) => view! {<Leaf><Text><b>{s}</b></Text></Leaf>}.into_any(),
        CheckLog::Fail(s) => view! {<Leaf><Text style="color:red">{s}</Text></Leaf>}.into_any(),
        CheckLog::Rule { header, steps } => {
            let header = view! {
                <Text><i style="color:blueviolet;">
                    "Trying rule: "
                    {header.into_iter().map(do_displayable).collect_view()}
                </i></Text>
            };
            let steps = steps.into_iter().map(|l| do_log(l, ctx)).collect_view();
            view! {<Subtree expanded=true>
                <Header slot>{header}</Header>
                {steps}
            </Subtree>}
            .into_any()
        }
        CheckLog::Strategy {
            name,
            steps,
            success,
        } => {
            let header = view! {
                <Text>
                    {do_success(success)}
                    <i>"Strategy: "<span style="color:darkgoldenrod;">{name}</span></i>
                </Text>
            };
            let steps = steps.into_iter().map(|l| do_log(l, ctx)).collect_view();
            view! {<Subtree expanded=!success>
                <Header slot>{header}</Header>
                {steps}
            </Subtree>}
            .into_any()
        }
        CheckLog::Inference {
            term,
            steps,
            context,
            result,
        } => in_context(ctx, context, move |context, ctx| {
            let success = result.is_some();
            let suffix = result.map(|result| {
                view! {
                    <mo>":"</mo>
                    {do_term(result)}
                }
            });
            let header = view! {
                <Text>
                    {do_success(success)}
                    "Inferring type of "
                    <math><mrow>
                        {context}
                        <mo style="font-weight:bold">"⊢"</mo>
                        {do_term(term)}
                        {suffix}
                    </mrow></math>
                </Text>
            };
            let steps = steps.into_iter().map(|l| do_log(l, ctx)).collect_view();
            view! {<Subtree expanded=!success>
                <Header slot>{header}</Header>
                {steps}
            </Subtree>}
            .into_any()
        }),
        CheckLog::VariableInference {
            var,
            steps,
            context,
            result,
        } => in_context(ctx, context, move |context, ctx| {
            let success = result.is_some();
            let suffix = result.map(|result| {
                view! {
                    <mo>":"</mo>
                    {do_term(result)}
                }
            });
            let header = view! {
                <Text>
                    {do_success(success)}
                    "Inferring type of "
                    <math><mrow>
                        {context}
                        <mo style="font-weight:bold">"⊢"</mo>
                        <mtext style="color:gray">{var.into_string()}</mtext>
                        {suffix}
                    </mrow></math>
                </Text>
            };
            let steps = steps.into_iter().map(|l| do_log(l, ctx)).collect_view();
            view! {<Subtree expanded=!success>
                <Header slot>{header}</Header>
                {steps}
            </Subtree>}
            .into_any()
        }),
        CheckLog::Inhabitable {
            term,
            steps,
            context,
            result,
        } => in_context(ctx, context, move |context, ctx| {
            let header = view! {
                <Text>
                    {do_success(result.unwrap_or(false))}
                    "Checking Inhabitability "
                    <math><mrow>
                        {context}
                        <mo style="font-weight:bold">"⊢"</mo>
                        <mtext style="font-weight:bold">"INH"</mtext>
                        <mspace width="5px"/>
                        {do_term(term)}
                    </mrow></math>
                </Text>
            };
            let steps = steps.into_iter().map(|l| do_log(l, ctx)).collect_view();
            view! {<Subtree expanded=result.is_none_or(|b| !b)>
                <Header slot>{header}</Header>
                {steps}
            </Subtree>}
            .into_any()
        }),
        CheckLog::Universe {
            term,
            steps,
            context,
            result,
        } => in_context(ctx, context, move |context, ctx| {
            let header = view! {
                <Text>
                    {do_success(result.unwrap_or(false))}
                    "Checking Universe "
                    <math><mrow>
                        {context}
                        <mo style="font-weight:bold">"⊢"</mo>
                        <mtext style="font-weight:bold">"UNIV"</mtext>
                        <mspace width="5px"/>
                        {do_term(term)}
                    </mrow></math>
                </Text>
            };
            let steps = steps.into_iter().map(|l| do_log(l, ctx)).collect_view();
            view! {<Subtree expanded=result.is_none_or(|b| !b)>
                <Header slot>{header}</Header>
                {steps}
            </Subtree>}
            .into_any()
        }),
        CheckLog::Equality {
            lhs,
            rhs,
            steps,
            context,
            result,
        } => in_context(ctx, context, move |context, ctx| {
            let header = view! {
                <Text>
                    {do_success(result.unwrap_or(false))}
                    "Checking Equality "
                    <math><mrow>
                        {context}
                        <mo style="font-weight:bold">"⊢"</mo>
                        {do_term(lhs)}
                        <mo style="font-weight:bold">"=="</mo>
                        {do_term(rhs)}
                    </mrow></math>
                </Text>
            };
            let steps = steps.into_iter().map(|l| do_log(l, ctx)).collect_view();
            view! {<Subtree expanded=result.is_none_or(|b| !b)>
                <Header slot>{header}</Header>
                {steps}
            </Subtree>}
            .into_any()
        }),
        CheckLog::HasType {
            tm,
            tp,
            steps,
            context,
            result,
        } => in_context(ctx, context, move |context, ctx| {
            let header = view! {
                <Text>
                    {do_success(result.unwrap_or(false))}
                    "Checking Typing "
                    <math><mrow>
                        {context}
                        <mo style="font-weight:bold">"⊢"</mo>
                        {do_term(tm)}
                        <mo style="font-weight:bold">":"</mo>
                        {do_term(tp)}
                    </mrow></math>
                </Text>
            };
            let steps = steps.into_iter().map(|l| do_log(l, ctx)).collect_view();
            view! {<Subtree expanded=result.is_none_or(|b| !b)>
                <Header slot>{header}</Header>
                {steps}
            </Subtree>}
            .into_any()
        }),
        CheckLog::Subtype {
            sub,
            sup,
            steps,
            context,
            result,
        } => in_context(ctx, context, move |context, ctx| {
            let header = view! {
                <Text>
                    {do_success(result.unwrap_or(false))}
                    "Checking Subtyping "
                    <math><mrow>
                        {context}
                        <mo style="font-weight:bold">"⊢"</mo>
                        {do_term(sub)}
                        <mo style="font-weight:bold">"<:"</mo>
                        {do_term(sup)}
                    </mrow></math>
                </Text>
            };
            let steps = steps.into_iter().map(|l| do_log(l, ctx)).collect_view();
            view! {<Subtree expanded=true>
                <Header slot>{header}</Header>
                {steps}
            </Subtree>}
            .into_any()
        }),
    }
}

fn in_context(
    current: &mut Vec<ComponentVar>,
    new: Box<[ComponentVar]>,
    then: impl FnOnce(Option<AnyView>, &mut Vec<ComponentVar>) -> AnyView,
) -> AnyView {
    fn component_var(c: ComponentVar) -> impl IntoView {
        let head = do_term(Term::Var {
            variable: c.var,
            presentation: None,
        });
        if c.tp.is_none() && c.df.is_none() {
            return leptos::either::Either::Left(head);
        }
        let tp = c.tp.map(|t| view! {<mo>":"</mo>{do_term(t)}});
        let df = c.df.map(|t| view! {<mo>":="</mo>{do_term(t)}});
        leptos::either::Either::Right(view! {<mrow>{head}{tp}{df}</mrow>})
    }
    let news = new.len();
    for n in new {
        current.push(n);
    }
    let ctx = if current.is_empty() {
        None
    } else {
        let mut iter = current.iter().cloned();
        // SAFETY: !current.is_empty()
        let first = unsafe { iter.next().unwrap_unchecked() };
        Some(
            view! {
                <mo style="font-weight:bold">"{"</mo>
                {component_var(first)}
                {iter.map(|v| view!{<mo>","</mo>{component_var(v)}}).collect_view()}
                <mo style="font-weight:bold">"}"</mo>
            }
            .into_any(),
        )
    };
    let r = then(ctx, current);
    for _ in 0..news {
        current.pop();
    }
    r
}

fn do_displayable(d: Displayable) -> AnyView {
    match d {
        Displayable::Num(i) => i.into_any(),
        Displayable::String(s) => s.into_any(),
        Displayable::Space => " ".into_any(),
        Displayable::Term(t) => view!(<math>{do_term(t)}</math>).into_any(),
        Displayable::Var(v) => {
            view!(<math>{do_term(Term::Var{variable:v,presentation:None})}</math>).into_any()
        }
        Displayable::Uri(Uri::Symbol(s)) => s.as_view::<FtmlBackend>(),
        Displayable::Uri(Uri::Module(m)) => m.as_view::<FtmlBackend>(),
        Displayable::Uri(Uri::DocumentElement(e)) => e.as_view::<FtmlBackend>(),
        Displayable::Uri(u) => u.to_string().into_any(),
    }
}

fn do_success(s: bool) -> impl IntoView {
    if s {
        leptos::either::Either::Left(view! {<span style="color:green">"✔ "</span>})
    } else {
        leptos::either::Either::Right(view! {<span style="color:red">"✗ "</span>})
    }
}

const ADD_INFO: bool = true;

fn do_term(t: Term) -> AnyView {
    if ADD_INFO {
        use thaw::{Popover, PopoverSize, PopoverTrigger};
        let s = format!("{:#?}", t.debug_short());
        view! {<msup>
            {t.into_view::<flams_router_content::Views, FtmlBackend>(false)}
            <Popover size=PopoverSize::Small>
                <PopoverTrigger slot><mi>"🛈"</mi></PopoverTrigger>
                <pre>{s}</pre>
            </Popover>
        </msup>}
        .into_any()
    } else {
        t.into_view::<flams_router_content::Views, FtmlBackend>(false)
    }
}

fn do_variable_uri(v: DocumentElementUri) -> AnyView {
    let t = Term::Var {
        variable: Variable::Ref {
            declaration: v,
            is_sequence: None,
        },
        presentation: None,
    };
    t.into_view::<flams_router_content::Views, FtmlBackend>(false)
}
