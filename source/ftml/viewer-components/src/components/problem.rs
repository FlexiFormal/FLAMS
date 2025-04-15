use flams_ontology::{
    narration::problems::{
        BlockFeedback, CheckedResult, FillinFeedback, FillinFeedbackKind, ProblemFeedback,
        ProblemResponse as OrigResponse, ProblemResponseType, Solutions,
    },
    uris::{DocumentElementURI, Name},
};
use flams_utils::prelude::HMap;
use flams_web_utils::inject_css;
use ftml_extraction::prelude::FTMLElements;
use leptos::{
    context::Provider,
    either::Either::{Left, Right},
    prelude::*,
};
use leptos_dyn_dom::OriginalNode;
use smallvec::SmallVec;

use crate::{
    components::{counters::SectionCounters, documents::ForcedName},
    ts::{FragmentContinuation, JsOrRsF},
    FragmentKind,
};

//use crate::ProblemOptions;

#[derive(Debug, Clone)]
pub enum ProblemState {
    Interactive {
        current_response: Option<OrigResponse>,
        solution: Option<Solutions>,
    },
    Finished {
        current_response: Option<OrigResponse>,
    },
    Graded {
        feedback: ProblemFeedback,
    },
}

pub struct ProblemOptions {
    pub on_response: Option<JsOrRsF<OrigResponse, ()>>,
    pub states: HMap<DocumentElementURI, ProblemState>,
}

impl std::fmt::Debug for ProblemOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProblemOptions")
            .field("on_response", &self.on_response.is_some())
            .field("states", &self.states)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct CurrentProblem {
    uri: DocumentElementURI,
    solutions: RwSignal<u8>,
    initial: Option<OrigResponse>,
    responses: RwSignal<SmallVec<ProblemResponse, 4>>,
    interactive: bool,
    feedback: RwSignal<Option<ProblemFeedback>>,
}
impl CurrentProblem {
    fn to_response(
        uri: &DocumentElementURI,
        responses: &SmallVec<ProblemResponse, 4>,
    ) -> OrigResponse {
        OrigResponse {
            uri: uri.clone(),
            responses: responses
                .iter()
                .map(|r| match r {
                    ProblemResponse::MultipleChoice(_, sigs) => {
                        flams_ontology::narration::problems::ProblemResponseType::MultipleChoice(
                            sigs.clone(),
                        )
                    }
                    ProblemResponse::SingleChoice(_, sig, _) => {
                        flams_ontology::narration::problems::ProblemResponseType::SingleChoice(*sig)
                    }
                    ProblemResponse::Fillinsol(s) => {
                        flams_ontology::narration::problems::ProblemResponseType::Fillinsol(
                            s.clone(),
                        )
                    }
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
enum ProblemResponse {
    MultipleChoice(bool, SmallVec<bool, 8>),
    SingleChoice(bool, u16, u16),
    Fillinsol(String),
}

pub(super) fn problem<V: IntoView + 'static>(
    uri: &DocumentElementURI,
    autogradable: bool,
    sub_problem: bool,
    styles: Box<[Name]>,
    children: impl FnOnce() -> V + Send + 'static,
) -> impl IntoView {
    let kind = FragmentKind::Problem {
        is_sub_problem: sub_problem,
        is_autogradable: autogradable,
    };
    inject_css("ftml-sections", include_str!("sections.css"));
    let mut counters: SectionCounters = expect_context();
    let style = counters.get_problem(&styles);
    let cls = {
        let mut s = String::new();
        s.push_str("ftml-problem");
        for style in styles {
            s.push(' ');
            s.push_str("ftml-problem-");
            s.push_str(style.first_name().as_ref());
        }
        s
    };

    let uri = with_context::<ForcedName, _>(|n| n.update(uri)).unwrap_or_else(|| uri.clone());
    let mut ex = CurrentProblem {
        solutions: RwSignal::new(0),
        uri,
        initial: None,
        interactive: true,
        responses: RwSignal::new(SmallVec::new()),
        feedback: RwSignal::new(None),
    };
    let responses = ex.responses;
    let is_done = with_context(|opt: &ProblemOptions| {
        match opt.states.get(&ex.uri) {
            Some(ProblemState::Graded { feedback }) => {
                ex.feedback
                    .update_untracked(|v| *v = Some(feedback.clone()));
                return Left(true);
            }
            Some(ProblemState::Interactive {
                current_response: Some(resp),
                ..
            }) => ex.initial = Some(resp.clone()),
            Some(ProblemState::Finished {
                current_response: Some(resp),
            }) => {
                ex.initial = Some(resp.clone());
                ex.interactive = false;
            }
            _ => (),
        }
        if let Some(f) = &opt.on_response {
            tracing::debug!("Problem: Using onResponse callback");
            Right(f.clone())
        } else {
            Left(false)
        }
    })
    .unwrap_or(Left(false));
    let uri = ex.uri.clone();
    FragmentContinuation::wrap(
        &(uri.clone(), kind),
        view! {
          <Provider value=ex><Provider value=counters><div class=cls style=style>
              {//<form>{
                let r = children();
                match is_done {
                  Left(true) => Left(r),
                  Right(f) => {
                    let _ = Effect::new(move |_| {
                      if let Some(resp) = responses.try_with(|resp|
                        CurrentProblem::to_response(&uri, resp)
                      ) {
                        let _ = f.apply(&resp);
                      }
                    });
                    Left(r)
                  }
                  _ if responses.get_untracked().is_empty() =>
                    Left(r),
                  _ => Right(view!{
                    {r}
                    {submit_answer()}
                  })
                }
              }//</form>
          </div></Provider></Provider>
        },
    )
}

fn submit_answer() -> impl IntoView {
    use thaw::{Button, ButtonSize};
    with_context(|current: &CurrentProblem| {
        let uri = current.uri.clone();
        let responses = current.responses;
        let feedback = current.feedback;
        move || {
            if feedback.with(Option::is_none) {
                let do_solution = move |uri: &_, r: &Solutions| {
                    let resp = responses
                        .with_untracked(|responses| CurrentProblem::to_response(uri, responses));
                    if let Some(r) = r.check(&resp) {
                        feedback.set(Some(r));
                    } else {
                        tracing::error!("Answer to Problem does not match solution");
                    }
                };
                let uri = uri.clone();
                let foract = if let Some(s) = with_context(|opt: &ProblemOptions| {
                    if let Some(ProblemState::Interactive {
                        solution: Some(sol),
                        ..
                    }) = opt.states.get(&uri)
                    {
                        Some(sol.clone())
                    } else {
                        None
                    }
                })
                .flatten()
                {
                    leptos::either::Either::Left(move || do_solution(&uri, &s))
                } else {
                    leptos::either::Either::Right(Action::new(move |()| {
                        let uri = uri.clone();
                        let do_solution = do_solution.clone();
                        async move {
                            match crate::remote::server_config.solution(uri.clone()).await {
                                Ok(r) => do_solution(&uri, &r),
                                Err(s) => tracing::error!("{s}"),
                            }
                        }
                    }))
                };
                let foract = move || match &foract {
                    leptos::either::Either::Right(act) => {
                        act.dispatch(());
                    }
                    leptos::either::Either::Left(sol) => sol(),
                };
                Some(view! {
                  <div style="margin:5px 0;"><div style="margin-left:auto;width:fit-content;">
                    <Button size=ButtonSize::Small on_click=move |_| {foract()}>"Submit Answer"</Button>
                  </div></div>
                })
            } else {
                None
            }
        }
    })
}

pub(super) fn hint<V: IntoView + 'static>(
    children: impl FnOnce() -> V + Send + 'static,
) -> impl IntoView {
    use flams_web_utils::components::{Collapsible, Header};
    view! {
      <Collapsible>
        <Header slot><span style="font-style:italic;color:gray">"Hint"</span></Header>
        {children()}
      </Collapsible>
    }
}

#[allow(clippy::needless_pass_by_value)]
#[allow(unused_variables)]
pub(super) fn solution(
    _skip: usize,
    _elements: FTMLElements,
    orig: OriginalNode,
    _id: Option<Box<str>>,
) -> impl IntoView {
    let Some((solutions, feedback)) =
        with_context::<CurrentProblem, _>(|e| (e.solutions, e.feedback))
    else {
        tracing::error!("solution outside of problem!");
        return None;
    };
    let idx = solutions.get_untracked();
    solutions.update_untracked(|i| *i += 1);
    #[cfg(any(feature = "csr", feature = "hydrate"))]
    {
        if orig.child_element_count() == 0 {
            tracing::debug!("Solution removed!");
        } else {
            tracing::debug!("Solution exists!");
        }
        Some(move || {
            feedback.with(|f| {
                f.as_ref().and_then(|f| {
                    let Some(f) = f.solutions.get(idx as usize) else {
                        tracing::error!("No solution!");
                        return None;
                    };
                    Some(view! {
                      <div style="background-color:lawngreen;">
                        <span inner_html=f.to_string()/>
                      </div>
                    })
                })
            })
        })
        // TODO
    }
    #[cfg(not(any(feature = "csr", feature = "hydrate")))]
    {
        Some(())
    }
}

#[allow(clippy::needless_pass_by_value)]
#[allow(unused_variables)]
pub(super) fn gnote(_skip: usize, _elements: FTMLElements, orig: OriginalNode) -> impl IntoView {
    #[cfg(any(feature = "csr", feature = "hydrate"))]
    {
        if orig.child_element_count() == 0 {
            tracing::debug!("Grading note removed!");
        } else {
            tracing::debug!("Grading note exists!");
        }
        // TODO
    }
    #[cfg(not(any(feature = "csr", feature = "hydrate")))]
    {
        ()
    }
}

#[derive(Clone)]
struct CurrentChoice(usize);

pub(super) fn choice_block<V: IntoView + 'static>(
    multiple: bool,
    inline: bool,
    children: impl FnOnce() -> V + Send + 'static,
) -> impl IntoView {
    let response = if multiple {
        ProblemResponse::MultipleChoice(inline, SmallVec::new())
    } else {
        ProblemResponse::SingleChoice(inline, 0, 0)
    };
    let Some(i) = with_context::<CurrentProblem, _>(|ex| {
        ex.responses.try_update_untracked(|ex| {
            let i = ex.len();
            ex.push(response);
            i
        })
    })
    .flatten() else {
        tracing::error!(
            "{} choice block outside of a problem!",
            if multiple { "multiple" } else { "single" }
        );
        return None;
    };
    Some(view! {<Provider value=CurrentChoice(i)>{children()}</Provider>})
}

pub(super) fn problem_choice<V: IntoView + 'static>(
    children: impl Fn() -> V + Send + 'static + Clone,
) -> impl IntoView {
    let Some(CurrentChoice(block)) = use_context() else {
        tracing::error!("choice outside of choice block!");
        return None;
    };
    let Some(ex) = use_context::<CurrentProblem>() else {
        tracing::error!("choice outside of problem!");
        return None;
    };
    let Some((multiple, inline)) = ex
        .responses
        .try_update_untracked(|resp| {
            resp.get_mut(block).map(|l| match l {
                ProblemResponse::MultipleChoice(inline, sigs) => {
                    let idx = sigs.len();
                    sigs.push(false);
                    Some((Left(idx), *inline))
                }
                ProblemResponse::SingleChoice(inline, _, total) => {
                    let val = *total;
                    *total += 1;
                    Some((Right(val), *inline))
                }
                ProblemResponse::Fillinsol(_) => None,
            })
        })
        .flatten()
        .flatten()
    else {
        tracing::error!("choice outside of choice block!");
        return None;
    };
    let selected = if let Some(init) = ex.initial.as_ref().and_then(|i| i.responses.get(block)) {
        match (init, multiple) {
            (ProblemResponseType::MultipleChoice(v), Left(idx)) => {
                v.get(idx).copied().unwrap_or_default()
            }
            (ProblemResponseType::SingleChoice(v), Right(val)) => *v == val,
            _ => false,
        }
    } else {
        false
    };
    let disabled = !ex.interactive;
    Some(match multiple {
        Left(idx) => Left(multiple_choice(
            idx,
            block,
            inline,
            selected,
            disabled,
            ex.responses,
            ex.feedback,
            children,
        )),
        Right(idx) => Right(single_choice(
            idx,
            block,
            inline,
            selected,
            disabled,
            ex.responses,
            ex.uri,
            ex.feedback,
            children,
        )),
    })
}

fn multiple_choice<V: IntoView + 'static>(
    idx: usize,
    block: usize,
    inline: bool,
    orig_selected: bool,
    disabled: bool,
    responses: RwSignal<SmallVec<ProblemResponse, 4>>,
    feedback: RwSignal<Option<ProblemFeedback>>,
    children: impl Fn() -> V + Send + 'static + Clone,
) -> impl IntoView {
    use leptos::either::{Either::Left, Either::Right, EitherOf3 as Either};
    use thaw::Icon;
    move || {
        feedback.with(|v|
      if let Some(feedback) = v.as_ref() {
        let err = || {
          tracing::error!("Answer to problem does not match solution:");
          Either::C(view!(<div style="color:red;">"ERROR"</div>))
        };
        let Some(CheckedResult::MultipleChoice{selected,choices}) = feedback.data.get(block) else {return err()};
        let Some(selected) = selected.get(idx).copied() else { return err() };
        let Some(BlockFeedback{is_correct,verdict_str,feedback}) = choices.get(idx) else { return err() };
        let icon = if selected == *is_correct {
          view!(<Icon icon=icondata_ai::AiCheckCircleOutlined style="color:green;"/>)
        } else {
          view!(<Icon icon=icondata_ai::AiCloseCircleOutlined style="color:red;"/>)
        };
        let bx = if selected {
          Left(view!(<input type="checkbox" checked disabled/>))
        } else {
          Right(view!(<input type="checkbox" disabled/>))
        };
        let verdict = if *is_correct {
          Left(view!(<span style="color:green;" inner_html=verdict_str.clone()/>))
        } else {
          Right(view!(<span style="color:red;" inner_html=verdict_str.clone()/>))
        };
        Either::B(view!{
          {icon}{bx}{children()}" "{verdict}" "
          {if inline {None} else {Some(view!(<br/>))}}
          <span style="background-color:lightgray;" inner_html=feedback.clone()/>
        })
      } else {
        let sig = create_write_slice(responses,
          move |resp,val| {
            let resp = resp.get_mut(block).expect("Signal error in problem");
            let ProblemResponse::MultipleChoice(_,v) = resp else { panic!("Signal error in problem")};
            v[idx] = val;
          }
        );
        sig.set(orig_selected);
        let rf = NodeRef::<leptos::html::Input>::new();
        let on_change = move |_| {
          let Some(ip) = rf.get_untracked() else {return};
          let nv = ip.checked();
          sig.set(nv);
        };
        Either::A(
          view!{
            <div style="display:inline;margin-right:5px;"><input node_ref=rf type="checkbox" on:change=on_change checked=orig_selected disabled=disabled/>{children()}</div>
          }
        )
      }
    )
    }
}

fn single_choice<V: IntoView + 'static>(
    idx: u16,
    block: usize,
    inline: bool,
    orig_selected: bool,
    disabled: bool,
    responses: RwSignal<SmallVec<ProblemResponse, 4>>,
    uri: DocumentElementURI,
    feedback: RwSignal<Option<ProblemFeedback>>,
    children: impl Fn() -> V + Send + 'static + Clone,
) -> impl IntoView {
    use leptos::either::{Either::Left, Either::Right, EitherOf3 as Either};
    use thaw::Icon;
    move || {
        feedback.with(|v| {
      if let Some(feedback) = v.as_ref() {
        let err = || {
          tracing::error!("Answer to problem does not match solution!");
          Either::C(view!(<div style="color:red;">"ERROR"</div>))
        };
        let Some(CheckedResult::SingleChoice{selected,choices}) = feedback.data.get(block) else {return err()};
        let Some(BlockFeedback{is_correct,verdict_str,feedback}) = choices.get(idx as usize) else { return err() };
        let icon = if *selected == idx && *is_correct {
          Some(Left(view!(<Icon icon=icondata_ai::AiCheckCircleOutlined style="color:green;"/>)))
        } else if *selected == idx {
          Some(Right(view!(<Icon icon=icondata_ai::AiCloseCircleOutlined style="color:red;"/>)))
        } else {None};
        let bx = if *selected == idx {
          Left(view!(<input type="radio" checked disabled/>))
        } else {
          Right(view!(<input type="radio" disabled/>))
        };
        let verdict = if *is_correct {
          Left(view!(<span style="color:green;" inner_html=verdict_str.clone()/>))
        } else {
          Right(view!(<span style="color:red;" inner_html=verdict_str.clone()/>))
        };
        Either::B(view!{
          {icon}{bx}{children()}" "{verdict}" "
          {if inline {None} else {Some(view!(<br/>))}}
          <span style="background-color:lightgray;" inner_html=feedback.clone()/>
        })
      } else {
        let name = format!("{uri}_{block}");
        let sig = create_write_slice(responses,
          move |resp,()| {
            let resp = resp.get_mut(block).expect("Signal error in problem");
            let ProblemResponse::SingleChoice(_,i,_) = resp else { panic!("Signal error in problem")};
            *i = idx;
          }
        );
        if orig_selected {sig.set(());}
        let rf = NodeRef::<leptos::html::Input>::new();
        let on_change = move |_| {
          let Some(ip) = rf.get_untracked() else {return};
          if ip.checked() { sig.set(()); }
        };
        Either::A(view!{
          <div style="display:inline;margin-right:5px;"><input node_ref=rf type="radio" name=name on:change=on_change checked=orig_selected disabled=disabled/>{children()}</div>
        })
      }
    })
    }
}

/*
  let feedback = ex.feedback;
  move || {
    if feedback.with(|f| f.is_some()) {}
    else {

    }
  }
*/

pub(super) fn fillinsol(wd: Option<f32>) -> impl IntoView {
    use leptos::either::EitherOf3 as Either;
    use thaw::Icon;
    let Some(ex) = use_context::<CurrentProblem>() else {
        tracing::error!("choice outside of problem!");
        return None;
    };
    let Some(choice) = ex.responses.try_update_untracked(|resp| {
        let i = resp.len();
        resp.push(ProblemResponse::Fillinsol(String::new()));
        i
    }) else {
        tracing::error!("fillinsol outside of an problem!");
        return None;
    };
    let feedback = ex.feedback;
    Some(move || {
        let style = wd.map(|wd| format!("width:{wd}px;"));
        feedback.with(|v|
    if let Some(feedback) = v.as_ref() {
      let err = || {
        tracing::error!("Answer to problem does not match solution!");
        Either::C(view!(<div style="color:red;">"ERROR"</div>))
      };
      let Some(CheckedResult::FillinSol { matching, text, options }) = feedback.data.get(choice) else {return err()};
      let (correct,feedback) = if let Some(m) = matching {
        let Some(FillinFeedback{is_correct,feedback,..}) = options.get(*m) else {return err()};

        (*is_correct,Some(feedback.clone()))
      } else {(false,None)};
      let solution = if correct { None } else {
        options.iter().find_map(|f| match f{
          FillinFeedback{is_correct:true,kind:FillinFeedbackKind::Exact(s),..} => Some(s.clone()),
          _ => None
        })
      };
      let icon = if correct {
        view!(<Icon icon=icondata_ai::AiCheckCircleOutlined style="color:green;"/>)
      } else {
        view!(<Icon icon=icondata_ai::AiCloseCircleOutlined style="color:red;"/>)
      };
      Either::B(view!{
        {icon}" "
        <input type="text" style=style disabled value=text.clone()/>
        {solution.map(|s| view!(" "<pre style="color:green">{s}</pre>))}
        {feedback.map(|s| view!(" "<span style="background-color:lightgray;" inner_html=s/>))}
      })
    } else {
      let sig = create_write_slice(ex.responses,
        move |resps,val| {
          let resp = resps.get_mut(choice).expect("Signal error in problem");
          let ProblemResponse::Fillinsol(s) = resp else { panic!("Signal error in problem")};
          *s = val;
        }
      );
      let txt = if let Some(ProblemResponseType::Fillinsol(s)) = ex.initial.as_ref().and_then(|i| i.responses.get(choice)) {
          sig.set(s.clone());
          s.clone()
      } else {String::new()};
      let disabled = !ex.interactive;
      Either::A(view!{
        <input type="text" style=style value=txt disabled=disabled on:input:target=move |ev| {sig.set(ev.target().value());}/>
      })
    }
  )
    })
}
