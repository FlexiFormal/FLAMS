use flams_web_utils::components::{Header, LazySubtree, Leaf, Subtree, Tree};
use ftml_components::components::content::FtmlViewable;
use ftml_dom::notations::TermExt;
use ftml_ontology::terms::{ComponentVar, Term, Variable};
pub use ftml_solver_trace::results::DocumentCheckResult;
use ftml_solver_trace::results::{
    CheckResult, ContentCheckResult, ProofStepCheckResult, ProofStepResult, SymbolCheckResult,
};
use ftml_solver_trace::{CheckLog, Displayable};
use ftml_uris::{DocumentElementUri, DocumentUri, Uri};
use leptos::math::mrow;
use leptos::prelude::*;
use thaw::Text;

pub trait ResultExt {
    fn render(self) -> AnyView;
}
impl ResultExt for DocumentCheckResult {
    fn render(self) -> AnyView {
        crate::Views::setup_document(
            DocumentUri::no_doc().clone(),
            ftml_components::SidebarPosition::None,
            false,
            ftml_dom::toc::TocSource::None,
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
                    {e.as_view()}
                </Text></Leaf>
            }
            .into_any(),
            Self::Variable(v, r) => {
                let success = do_success(r.success());
                view! {
                    <Subtree expanded=!r.success()>
                        <Header slot>
                            <b>{success}"Variable "{ftml_dom::utils::math(move || do_variable_uri(v))}</b>
                        </Header>
                        {
                            symbol_result(r)
                        }
                    </Subtree>
                }
                .into_any()
            }
            Self::Content(c) => match c {
                ContentCheckResult::Symbol(u, s) => {
                    let success = s.success();
                    let succ = do_success(success);
                    view! {
                        <Subtree expanded=!success>
                            <Header slot>
                                <b>{succ}"Symbol "{u.as_view()}</b>
                            </Header>
                            {
                                symbol_result(s)
                            }
                        </Subtree>
                    }
                    .into_any()
                }
            },
            Self::Proof(uri, checks) => {
                let success = checks.iter().all(ProofStepResult::success);
                let succ = do_success(success);
                view! {
                    <Subtree expanded=!success>
                        <Header slot>
                            <b>{succ}"Proof "{uri.as_view()}</b>
                        </Header>
                        {
                            checks.into_iter().map(proof).collect_view()
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
                            <b>{succ}"Module "{uri.as_view()}</b>
                        </Header>
                        {
                            checks.into_iter().map(|c| match c {
                                ContentCheckResult::Symbol(u, s) => {
                                    let success = s.success();
                                    let succ = do_success(success);
                                    view! {
                                        <Subtree expanded=!success>
                                            <Header slot>
                                                <b>{succ}"Symbol "{u.as_view()}</b>
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

fn proof(r: ProofStepResult) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    let (prefix, var, result) = match r {
        ProofStepResult::Assumption { var, result } => ("Assumption ", var, result),
        ProofStepResult::Step { var, result } => ("Step ", var, result),
        ProofStepResult::Conclusion { var, result } => ("Conclusion ", var, result),
        ProofStepResult::Subproof { uri, var, results } => {
            return Right({
                let success = results.iter().all(ProofStepResult::success);
                let succ = do_success(success);
                view! {
                    <Subtree expanded=!success>
                        <Header slot>
                            <b>
                                {succ}
                                {"Subproof "}
                                {var.map(|v| ftml_dom::utils::math(move || do_variable_uri(v)))}
                            </b>
                        </Header>
                        {results.into_iter().map(proof).collect_view()}
                    </Subtree>
                }
            });
        }
    };
    let success = result.success();
    let succ = do_success(success);
    Left(view! {
        <Subtree expanded=!success>
            <Header slot>
                <b>
                    {succ}
                    {prefix}
                    {var.map(|v| ftml_dom::utils::math(move || do_variable_uri(v)))}
                </b>
            </Header>
            {proofstep_result(result)}
        </Subtree>
    })
}

fn proofstep_result(r: ProofStepCheckResult) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    match r {
        ProofStepCheckResult::GoalOnly { result }
        | ProofStepCheckResult::Both {
            inhabitable: result,
            matches: None,
        } => Left(do_log(result.log, &mut Vec::new())),
        ProofStepCheckResult::ProofOnly { log, .. } => Left(do_log(log, &mut Vec::new())),
        ProofStepCheckResult::Both {
            inhabitable,
            matches: Some(r),
        } => Right(view! {
            <Subtree expanded=!inhabitable.success>
                <Header slot><Text><b>"Proof goal"</b></Text></Header>
                {do_log(inhabitable.log, &mut Vec::new())}
            </Subtree>
            <Subtree expanded=!r.success>
                <Header slot><Text><b>"Proof"</b></Text></Header>
                {do_log(r.log, &mut Vec::new())}
            </Subtree>
        }),
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

            let mut ctx = ctx.clone();
            tree(success, header, move || {
                steps
                    .iter()
                    .map(|l| do_log(l.clone(), &mut ctx))
                    .collect_view()
            })
            .into_any()
            /*
            let steps = steps.into_iter().map(|l| do_log(l, ctx)).collect_view();
            view! {<Subtree expanded=!success>
                <Header slot>{header}</Header>
                {steps}
            </Subtree>}
            .into_any()
            */
        }
        CheckLog::Simplify {
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
                    //{do_success(success)}
                    "Simplifying "
                    {ftml_dom::utils::math(move || mrow().child(view!{
                        {context}
                        <mo style="font-weight:bold">"⊢"</mo>
                        {do_term(term)}
                        {suffix}
                    }))}
                </Text>
            };

            let mut ctx = ctx.clone();
            tree(success, header, move || {
                steps
                    .iter()
                    .map(|l| do_log(l.clone(), &mut ctx))
                    .collect_view()
            })
            .into_any()
            /*
            let steps = steps.into_iter().map(|l| do_log(l, ctx)).collect_view();
            view! {<Subtree expanded=!success>
                <Header slot>{header}</Header>
                {steps}
            </Subtree>}
            .into_any()
            */
        }),
        CheckLog::Proving {
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
                    //{do_success(success)}
                    "Proving "
                    {ftml_dom::utils::math(move || mrow().child(view!{
                        {context}
                        <mo style="font-weight:bold">"⊢"</mo>
                        {do_term(term)}
                        {suffix}
                    }))}
                </Text>
            };

            let mut ctx = ctx.clone();
            tree(success, header, move || {
                steps
                    .iter()
                    .map(|l| do_log(l.clone(), &mut ctx))
                    .collect_view()
            })
            .into_any()
            /*
            let steps = steps.into_iter().map(|l| do_log(l, ctx)).collect_view();
            view! {<Subtree expanded=!success>
                <Header slot>{header}</Header>
                {steps}
            </Subtree>}
            .into_any()
            */
        }),
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
                    //{do_success(success)}
                    "Inferring type of "
                    {ftml_dom::utils::math(move || mrow().child(view!{
                        {context}
                        <mo style="font-weight:bold">"⊢"</mo>
                        {do_term(term)}
                        {suffix}
                    }))}
                </Text>
            };

            let mut ctx = ctx.clone();
            tree(success, header, move || {
                steps
                    .iter()
                    .map(|l| do_log(l.clone(), &mut ctx))
                    .collect_view()
            })
            .into_any()
            /*
            let steps = steps.into_iter().map(|l| do_log(l, ctx)).collect_view();
            view! {<Subtree expanded=!success>
                <Header slot>{header}</Header>
                {steps}
            </Subtree>}
            .into_any()
            */
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
                    //{do_success(success)}
                    "Inferring type of "
                    {ftml_dom::utils::math(move || mrow().child(view!{
                        {context}
                        <mo style="font-weight:bold">"⊢"</mo>
                        <mtext style="color:gray">{var.into_string()}</mtext>
                        {suffix}
                    }))}
                </Text>
            };

            let mut ctx = ctx.clone();
            tree(success, header, move || {
                steps
                    .iter()
                    .map(|l| do_log(l.clone(), &mut ctx))
                    .collect_view()
            })
            .into_any()
            /*
            let steps = steps.into_iter().map(|l| do_log(l, ctx)).collect_view();
            view! {<Subtree expanded=!success>
                <Header slot>{header}</Header>
                {steps}
            </Subtree>}
            .into_any()
            */
        }),
        CheckLog::Inhabitable {
            term,
            steps,
            context,
            result,
        } => in_context(ctx, context, move |context, ctx| {
            let header = view! {
                <Text>
                    //{do_success(result.unwrap_or(false))}
                    "Checking Inhabitability "
                    {ftml_dom::utils::math(move || mrow().child(view!{
                        {context}
                        <mo style="font-weight:bold">"⊢"</mo>
                        <mtext style="font-weight:bold">"INH"</mtext>
                        <mspace width="5px"/>
                        {do_term(term)}
                    }))}
                </Text>
            };

            let mut ctx = ctx.clone();
            tree(result.unwrap_or(false), header, move || {
                steps
                    .iter()
                    .map(|l| do_log(l.clone(), &mut ctx))
                    .collect_view()
            })
            .into_any()
            /*
            let steps = steps.into_iter().map(|l| do_log(l, ctx)).collect_view();
            view! {<Subtree expanded=result.is_none_or(|b| !b)>
                <Header slot>{header}</Header>
                {steps}
            </Subtree>}
            .into_any()
            */
        }),
        CheckLog::Universe {
            term,
            steps,
            context,
            result,
        } => in_context(ctx, context, move |context, ctx| {
            let header = view! {
                <Text>
                    //{do_success(result.unwrap_or(false))}
                    "Checking Universe "
                    {ftml_dom::utils::math(move || mrow().child(view!{
                        {context}
                        <mo style="font-weight:bold">"⊢"</mo>
                        <mtext style="font-weight:bold">"UNIV"</mtext>
                        <mspace width="5px"/>
                        {do_term(term)}
                    }))}
                </Text>
            };

            let mut ctx = ctx.clone();
            tree(result.unwrap_or(false), header, move || {
                steps
                    .iter()
                    .map(|l| do_log(l.clone(), &mut ctx))
                    .collect_view()
            })
            .into_any()
            /*
            let steps = steps.into_iter().map(|l| do_log(l, ctx)).collect_view();
            view! {<Subtree expanded=result.is_none_or(|b| !b)>
                <Header slot>{header}</Header>
                {steps}
            </Subtree>}
            .into_any()
            */
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
                    //{do_success(result.unwrap_or(false))}
                    "Checking Equality "
                    {ftml_dom::utils::math(move || mrow().child(view!{
                        {context}
                        <mo style="font-weight:bold">"⊢"</mo>
                        {do_term(lhs)}
                        <mo style="font-weight:bold">"=="</mo>
                        {do_term(rhs)}
                    }))}
                </Text>
            };

            let mut ctx = ctx.clone();
            tree(result.unwrap_or(false), header, move || {
                steps
                    .iter()
                    .map(|l| do_log(l.clone(), &mut ctx))
                    .collect_view()
            })
            .into_any()
            /*
            let steps = steps.into_iter().map(|l| do_log(l, ctx)).collect_view();
            view! {<Subtree expanded=result.is_none_or(|b| !b)>
                <Header slot>{header}</Header>
                {steps}
            </Subtree>}
            .into_any()
            */
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
                    //{do_success(result.unwrap_or(false))}
                    "Checking Typing "
                    {ftml_dom::utils::math(move || mrow().child(view!{
                        {context}
                        <mo style="font-weight:bold">"⊢"</mo>
                        {do_term(tm)}
                        <mo style="font-weight:bold">":"</mo>
                        {do_term(tp)}
                    }))}
                </Text>
            };

            let mut ctx = ctx.clone();
            tree(result.unwrap_or(false), header, move || {
                steps
                    .iter()
                    .map(|l| do_log(l.clone(), &mut ctx))
                    .collect_view()
            })
            .into_any()
            /*
            let steps = steps.into_iter().map(|l| do_log(l, ctx)).collect_view();
            view! {<Subtree expanded=result.is_none_or(|b| !b)>
                <Header slot>{header}</Header>
                {steps}
            </Subtree>}
            .into_any()
            */
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
                    //{do_success(result.unwrap_or(false))}
                    "Checking Subtyping "
                    {ftml_dom::utils::math(move || mrow().child(view!{
                        {context}
                        <mo style="font-weight:bold">"⊢"</mo>
                        {do_term(sub)}
                        <mo style="font-weight:bold">"<:"</mo>
                        {do_term(sup)}
                    }))}
                </Text>
            };

            let mut ctx = ctx.clone();
            tree(result.unwrap_or(false), header, move || {
                steps
                    .iter()
                    .map(|l| do_log(l.clone(), &mut ctx))
                    .collect_view()
            })
            .into_any()
            /*
            let steps = steps.into_iter().map(|l| do_log(l, ctx)).collect_view();
            view! {<Subtree expanded=true>
                <Header slot>{header}</Header>
                {steps}
            </Subtree>}
            .into_any()
            */
        }),
    }
}

/*
#[allow(clippy::too_many_lines)]
fn do_log2(log: CheckLog, ctx: &mut Vec<ComponentVar>) -> impl IntoView + use<> + 'static {
    fn do_log_i(
        log: CheckLog,
        ctx: &mut Vec<ComponentVar>,
    ) -> (impl IntoView + use<> + 'static, bool, Vec<CheckLog>, usize) {
        use leptos::either::EitherOf13::{A, B, C, D, E, F, G, H, I, J, K, L, M};
        match log {
            CheckLog::Comment(s) => (
                A(view! {<Leaf><Text style="color:cadetblue"><i>{s}</i></Text></Leaf>}),
                true,
                Vec::new(),
                0,
            ),
            CheckLog::Emph(s) => (
                B(view! {<Leaf><Text><b>{s}</b></Text></Leaf>}),
                true,
                Vec::new(),
                0,
            ),
            CheckLog::Header(s) => (
                B(view! {<Leaf><Text><b>{s}</b></Text></Leaf>}),
                true,
                Vec::new(),
                0,
            ),
            CheckLog::Fail(s) => (
                C(view! {<Leaf><Text style="color:red">{s}</Text></Leaf>}),
                true,
                Vec::new(),
                0,
            ),
            CheckLog::Rule { header, steps } => (
                D(view! {
                    <Text><i style="color:blueviolet;">
                        "Trying rule: "
                        {header.into_iter().map(do_displayable).collect_view()}
                    </i></Text>
                }),
                true,
                steps,
                0,
            ),
            CheckLog::Strategy {
                name,
                steps,
                success,
            } => (
                E(view! {
                    <Text>
                        {do_success(success)}
                        <i>"Strategy: "<span style="color:darkgoldenrod;">{name}</span></i>
                    </Text>
                }),
                !success,
                steps,
                0,
            ),
            CheckLog::Simplify {
                term,
                steps,
                context,
                result,
            } => {
                let clen = ctx.len();
                ctx.extend(context);
                let context = do_context(ctx);
                let success = result.is_some();
                let suffix = result.map(|result| {
                    view! {
                        <mo>":"</mo>
                        {do_term(result)}
                    }
                });
                (
                    F(view! {
                        <Text>
                            {do_success(success)}
                            "Simplifying "
                            {ftml_dom::utils::math(move || mrow().child(view!{
                                {context}
                                <mo style="font-weight:bold">"⊢"</mo>
                                {do_term(term)}
                                {suffix}
                            }))}
                        </Text>
                    }),
                    !success,
                    steps,
                    clen,
                )
            }
            CheckLog::Proving {
                term,
                steps,
                context,
                result,
            } => {
                let clen = ctx.len();
                ctx.extend(context);
                let context = do_context(ctx);
                let success = result.is_some();
                let suffix = result.map(|result| {
                    view! {
                        <mo>":"</mo>
                        {do_term(result)}
                    }
                });
                (
                    F(view! {
                        <Text>
                            {do_success(success)}
                            "Proving "
                            {ftml_dom::utils::math(move || mrow().child(view!{
                                {context}
                                <mo style="font-weight:bold">"⊢"</mo>
                                {do_term(term)}
                                {suffix}
                            }))}
                        </Text>
                    }),
                    !success,
                    steps,
                    clen,
                )
            }
            CheckLog::Inference {
                term,
                steps,
                context,
                result,
            } => {
                let clen = ctx.len();
                ctx.extend(context);
                let context = do_context(ctx);
                let success = result.is_some();
                let suffix = result.map(|result| {
                    view! {
                        <mo>":"</mo>
                        {do_term(result)}
                    }
                });
                (
                    G(view! {
                        <Text>
                            {do_success(success)}
                            "Inferring type of "
                            {ftml_dom::utils::math(move || mrow().child(view!{
                                {context}
                                <mo style="font-weight:bold">"⊢"</mo>
                                {do_term(term)}
                                {suffix}
                            }))}
                        </Text>
                    }),
                    !success,
                    steps,
                    clen,
                )
            }
            CheckLog::VariableInference {
                var,
                steps,
                context,
                result,
            } => {
                let clen = ctx.len();
                ctx.extend(context);
                let context = do_context(ctx);
                let success = result.is_some();
                let suffix = result.map(|result| {
                    view! {
                        <mo>":"</mo>
                        {do_term(result)}
                    }
                });
                (
                    H(view! {
                        <Text>
                            {do_success(success)}
                            "Inferring type of "
                            {ftml_dom::utils::math(move || mrow().child(view!{
                                {context}
                                <mo style="font-weight:bold">"⊢"</mo>
                                <mtext style="color:gray">{var.into_string()}</mtext>
                                {suffix}
                            }))}
                        </Text>
                    }),
                    !success,
                    steps,
                    clen,
                )
            }
            CheckLog::Inhabitable {
                term,
                steps,
                context,
                result,
            } => {
                let clen = ctx.len();
                ctx.extend(context);
                let context = do_context(ctx);
                let success = result.unwrap_or(false);
                (
                    I(view! {
                        <Text>
                            {do_success(success)}
                            "Checking Inhabitability "
                            {ftml_dom::utils::math(move || mrow().child(view!{
                                {context}
                                <mo style="font-weight:bold">"⊢"</mo>
                                <mtext style="font-weight:bold">"INH"</mtext>
                                <mspace width="5px"/>
                                {do_term(term)}
                            }))}
                        </Text>
                    }),
                    !success,
                    steps,
                    clen,
                )
            }
            CheckLog::Universe {
                term,
                steps,
                context,
                result,
            } => {
                let clen = ctx.len();
                ctx.extend(context);
                let context = do_context(ctx);
                let success = result.unwrap_or(false);
                (
                    J(view! {
                        <Text>
                            {do_success(success)}
                            "Checking Universe "
                            {ftml_dom::utils::math(move || mrow().child(view!{
                                {context}
                                <mo style="font-weight:bold">"⊢"</mo>
                                <mtext style="font-weight:bold">"UNIV"</mtext>
                                <mspace width="5px"/>
                                {do_term(term)}
                            }))}
                        </Text>
                    }),
                    !success,
                    steps,
                    clen,
                )
            }
            CheckLog::Equality {
                lhs,
                rhs,
                steps,
                context,
                result,
            } => {
                let clen = ctx.len();
                ctx.extend(context);
                let context = do_context(ctx);
                let success = result.unwrap_or(false);
                (
                    K(view! {
                        <Text>
                            {do_success(success)}
                            "Checking Equality "
                            {ftml_dom::utils::math(move || mrow().child(view!{
                                {context}
                                <mo style="font-weight:bold">"⊢"</mo>
                                {do_term(lhs)}
                                <mo style="font-weight:bold">"=="</mo>
                                {do_term(rhs)}
                            }))}
                        </Text>
                    }),
                    !success,
                    steps,
                    clen,
                )
            }
            CheckLog::HasType {
                tm,
                tp,
                steps,
                context,
                result,
            } => {
                let clen = ctx.len();
                ctx.extend(context);
                let context = do_context(ctx);
                let success = result.unwrap_or(false);
                (
                    L(view! {
                        <Text>
                            {do_success(success)}
                            "Checking Typing "
                            {ftml_dom::utils::math(move || mrow().child(view!{
                                {context}
                                <mo style="font-weight:bold">"⊢"</mo>
                                {do_term(tm)}
                                <mo style="font-weight:bold">":"</mo>
                                {do_term(tp)}
                            }))}
                        </Text>
                    }),
                    !success,
                    steps,
                    clen,
                )
            }
            CheckLog::Subtype {
                sub,
                sup,
                steps,
                context,
                result,
            } => {
                let clen = ctx.len();
                ctx.extend(context);
                let context = do_context(ctx);
                let success = result.unwrap_or(false);
                (
                    M(view! {
                        <Text>
                            {do_success(result.unwrap_or(false))}
                            "Checking Subtyping "
                            {ftml_dom::utils::math(move || mrow().child(view!{
                                {context}
                                <mo style="font-weight:bold">"⊢"</mo>
                                {do_term(sub)}
                                <mo style="font-weight:bold">"<:"</mo>
                                {do_term(sup)}
                            }))}
                        </Text>
                    }),
                    !success,
                    steps,
                    clen,
                )
            }
        }
    }
    let mut stack: Vec<(
        _,
        _,
        _,
        Vec<leptos::either::Either<_, AnyView>>,
        std::vec::IntoIter<CheckLog>,
    )> = Vec::new();
    let mut current = log;
    let mut ret = Vec::new();
    'outer: loop {
        let (v, expanded, steps, vars) = do_log_i(current, ctx);
        if steps.is_empty() {
            ctx.truncate(vars);
            if let Some(mut s) = stack.last_mut() {
                let mut v = leptos::either::Either::Left(v);
                loop {
                    s.3.push(v);
                    if let Some(next) = s.4.next() {
                        current = next;
                        break;
                    }
                    // SAFETY: we just had Some(last_mut())
                    let (nv, nexp, nvars, nret, _) = unsafe { stack.pop().unwrap_unchecked() };
                    ctx.truncate(nvars);
                    v = leptos::either::Either::Right(
                        view! {<Subtree expanded=nexp>
                            <Header slot>{nv}</Header>
                            {nret}
                        </Subtree>}
                        .into_any(),
                    );
                    if let Some(ns) = stack.last_mut() {
                        s = ns;
                    } else {
                        ret.push(v);
                        break 'outer;
                    }
                }
            } else {
                ret.push(leptos::either::Either::Left(v));
                break;
            }
        } else {
            let mut steps = steps.into_iter();
            // SAFETY: !steps.is_empty()
            current = unsafe { steps.next().unwrap_unchecked() };
            stack.push((v, expanded, vars, Vec::new(), steps));
        }
    }
    ret
}

fn do_context(ctx: &[ComponentVar]) -> impl IntoView + use<> + 'static {
    fn component_var(c: &ComponentVar) -> impl IntoView + use<> + 'static {
        let head = do_term(Term::Var {
            variable: c.var.clone(),
            presentation: None,
        });
        if c.tp.is_none() && c.df.is_none() {
            return leptos::either::Either::Left(head);
        }
        let tp = c.tp.clone().map(|t| view! {<mo>":"</mo>{do_term(t)}});
        let df = c.df.clone().map(|t| view! {<mo>":="</mo>{do_term(t)}});
        leptos::either::Either::Right(view! {<mrow>{head}{tp}{df}</mrow>})
    }
    if ctx.is_empty() {
        return None;
    }
    Some({
        let mut iter = ctx.iter();
        let first = unsafe { iter.next().unwrap_unchecked() };
        view! {
            <mo style="font-weight:bold">"{"</mo>
            {component_var(first)}
            {iter.map(|v| view!{<mo>","</mo>{component_var(v)}}).collect_view()}
            <mo style="font-weight:bold">"}"</mo>
        }
    })
}
 */

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

fn do_displayable(d: Displayable) -> impl IntoView {
    use leptos::either::EitherOf5::{A, B, C, D, E};
    match d {
        Displayable::String(s) => A(s),
        Displayable::Num(i) => B(i),
        //Displayable::Space => C(" "),
        Displayable::Term(t) => C(ftml_dom::utils::math(move || do_term(t))),
        Displayable::Var(v) => D(ftml_dom::utils::math(move || {
            do_term(Term::Var {
                variable: v,
                presentation: None,
            })
        })),
        Displayable::Uri(Uri::Symbol(s)) => E(s.as_view()),
        Displayable::Uri(Uri::Module(m)) => E(m.as_view()),
        Displayable::Uri(Uri::DocumentElement(e)) => E(e.as_view()),
        Displayable::Uri(u) => A(u.to_string()),
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

fn do_term(t: Term) -> impl IntoView {
    if ADD_INFO {
        use thaw::{Popover, PopoverSize, PopoverTrigger};
        let s = format!("{:#?}", t.debug_short());
        leptos::either::Either::Left(view! {<msup>
            {t.into_view::<crate::Views>(ftml_components::backend(),false)}
            <Popover size=PopoverSize::Small>
                <PopoverTrigger slot><mi>"🛈"</mi></PopoverTrigger>
                <pre>{s}</pre>
            </Popover>
        </msup>})
    } else {
        leptos::either::Either::Right(
            t.into_view::<crate::Views>(ftml_components::backend(), false),
        )
    }
}

fn do_variable_uri(v: DocumentElementUri) -> impl IntoView {
    let t = Term::Var {
        variable: Variable::Ref {
            declaration: v,
            is_sequence: None,
        },
        presentation: None,
    };
    t.into_view::<crate::Views>(ftml_components::backend(), false)
}

fn tree<V: IntoView + 'static>(
    success: bool,
    head: impl IntoView + 'static,
    mut children: impl FnMut() -> V + Send + 'static,
) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    if success {
        Left(LazySubtree(flams_web_utils::components::LazySubtreeProps {
            header: flams_web_utils::components::Header {
                children: Box::new(move || view! {{do_success(true)}{head}}.into_any()),
            },
            children: Box::new(move || children().into_any()),
        }))
        /*Left(view! {
            <LazySubtree>
                <Header slot>{do_success(true)}{head}</Header>
                {children()}
            </LazySubtree>
        })*/
    } else {
        Right(view! {
            <Subtree expanded=true>
                <Header slot>{do_success(false)}{head}</Header>
                {children()}
            </Subtree>
        })
    }
}
