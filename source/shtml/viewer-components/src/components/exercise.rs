use immt_ontology::{narration::exercises::{BlockFeedback, CheckedResult, ExerciseFeedback, ExerciseResponse as OrigResponse, FillinFeedback}, uris::DocumentElementURI};
use leptos::{context::Provider, prelude::*};
use leptos_dyn_dom::OriginalNode;
use serde::Serialize;
use shtml_extraction::prelude::SHTMLElements;
use smallvec::SmallVec;

//use crate::ExerciseOptions;

#[derive(Clone,Debug)]
pub struct CurrentExercise {
  uri:DocumentElementURI,
  solutions:RwSignal<u8>,
  responses:RwSignal<SmallVec<ExerciseResponse,4>>,
  feedback:RwSignal<Option<ExerciseFeedback>>
}
impl CurrentExercise {
  pub fn to_response(uri:&DocumentElementURI,responses:&SmallVec<ExerciseResponse,4>) -> OrigResponse {
    OrigResponse {
      uri:uri.clone(),
      responses:responses.iter().map(|r|
        match r {
          ExerciseResponse::MultipleChoice(_,sigs) =>
            immt_ontology::narration::exercises::ExerciseResponseType::MultipleChoice(sigs.clone()),
          ExerciseResponse::SingleChoice(_,sig,_) =>
            immt_ontology::narration::exercises::ExerciseResponseType::SingleChoice(*sig),
            ExerciseResponse::Fillinsol(s) =>
              immt_ontology::narration::exercises::ExerciseResponseType::Fillinsol(s.clone()),
        }
      ).collect()
    }
  }
}

#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
enum ExerciseResponse {
  MultipleChoice(bool,SmallVec<bool,8>),
  SingleChoice(bool,u16,u16),
  Fillinsol(String)
}

pub(super) fn exercise<V:IntoView+'static>(uri:&DocumentElementURI,autogradable:bool,sub_exercise:bool,children:impl FnOnce() -> V + Send + 'static) -> impl IntoView {
  let ex = CurrentExercise{
    solutions:RwSignal::new(0),
    uri:uri.clone(),
    responses:RwSignal::new(SmallVec::new()),
    feedback:RwSignal::new(None)
  };
  let responses = ex.responses;
  let is_done = false /*with_context(|exopt:&ExerciseOptions| {
    if let Some((s,r)) = exopt.responses.get(uri) {
      if let Some(r) = s.check(r) {
        ex.feedback.update_untracked(|v| *v = Some(r));
      } else {
        tracing::error!("Answer to Exercise does not match solution");
      }
      true
    } else {false }
  }).unwrap_or_default()*/;
  /*let _ = Effect::new(move |_| {
    if let Some(resp) = ex.responses.try_with(|resp| 
      CurrentExercise::to_response(&uri, resp)
    ) {
      tracing::info!("Response: {}",serde_json::to_string(&resp).unwrap());
    }
  });*/
  view!{
    <Provider value=ex>
      <div style="border-left:3px solid red;margin-top:5px;margin-bottom:5px;padding-left:5px;margin-left:-8px">
        <b>"Exercise"</b>
        {//<form>{
          let r = children();
          if is_done || responses.get_untracked().is_empty() {
            leptos::either::Either::Left(r)
          } else {
            leptos::either::Either::Right(view!{
              {r}
              {submit_answer()}
            })
          }
          /* responses.with_untracked(|resp| {
            let r = CurrentExercise::to_response(&uricl, resp);
            let Ok(j) = serde_json::to_string(&r) else {unreachable!()};
            if r.responses.is_empty() {
              tracing::info!("No response :(");
            } else {
              tracing::info!("Has response: {j}");
            }
          });*/
        }//</form>
      </div>
    </Provider>
  }
}

fn submit_answer() -> impl IntoView {
  use thaw::{Button,ButtonSize};
  with_context(|current:&CurrentExercise| {
    let uri = current.uri.clone();
    let responses = current.responses;
    let feedback = current.feedback;
    move || if feedback.with(Option::is_none) {
      let uri = uri.clone();
      let act = Action::new(move |()| {let uri = uri.clone(); async move {
        match crate::remote::server_config.solution(uri.clone()).await {
          Ok(r) => {
            let resp = responses.with_untracked(|responses|
              CurrentExercise::to_response(&uri, responses)
            );
            if let Some(r) = r.check(&resp) {
              feedback.set(Some(r));
            } else {
              tracing::error!("Answer to Exercise does not match solution");
            }
          }
          Err(s) => tracing::error!("{s}")
        }
      }});
      Some(view!{
        <div style="margin:5px 0;"><div style="margin-left:auto;width:fit-content;">
          <Button size=ButtonSize::Small on_click=move |_| {act.dispatch(());}>"Submit Answer"</Button>
        </div></div>
      })
    } else { None }
  })
}

pub(super) fn hint<V:IntoView+'static>(children:impl FnOnce() -> V + Send + 'static) -> impl IntoView {
  use immt_web_utils::components::{Collapsible,Header};
  view!{
    <Collapsible>
      <Header slot><span style="font-style:italic;color:gray">"Hint"</span></Header>
      {children()}
    </Collapsible>
  }
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn solution(_skip:usize,_elements:SHTMLElements,orig:OriginalNode,_id:Option<Box<str>>) -> impl IntoView {
  let Some((solutions,feedback)) = with_context::<CurrentExercise,_>(|e| (e.solutions,e.feedback)) else {
    tracing::error!("solution outside of exercise!");
    return None
  };
  let idx = solutions.get_untracked();
  solutions.update_untracked(|i| *i += 1);
  #[cfg(any(feature="csr",feature="hydrate"))]
  {
    if orig.child_element_count() == 0 {
      tracing::debug!("Solution removed!");
    } else {
      tracing::debug!("Solution exists!");
    }
    Some(move || feedback.with(|f| f.as_ref().and_then(|f| {
      let Some(f) = f.solutions.get(idx as usize) else {
        tracing::error!("No solution!");
        return None
      };
      Some(view!{
        <div style="background-color:lawngreen;">
          <span inner_html=f.to_string()/>
        </div>
      })
    })))
    // TODO
  }
  #[cfg(not(any(feature="csr",feature="hydrate")))]
  {Some(())}
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn gnote(_skip:usize,_elements:SHTMLElements,orig:OriginalNode) -> impl IntoView {
  #[cfg(any(feature="csr",feature="hydrate"))]
  {
    if orig.child_element_count() == 0 {
      tracing::debug!("Grading note removed!");
    } else {
      tracing::debug!("Grading note exists!");
    }
    // TODO
  }
  #[cfg(not(any(feature="csr",feature="hydrate")))]
  {()}
}

#[derive(Clone)]
struct CurrentChoice(usize);

pub(super) fn choice_block<V:IntoView+'static>(multiple:bool,inline:bool,children:impl FnOnce() -> V + Send + 'static) -> impl IntoView {
  let response = if multiple {
    ExerciseResponse::MultipleChoice(inline,SmallVec::new())
  } else {
    ExerciseResponse::SingleChoice(inline,0,0)
  };
  let Some(i) = with_context::<CurrentExercise,_>(|ex|
    ex.responses.try_update_untracked(|ex| {
      let i = ex.len();
      ex.push(response);
      i
    })
  ).flatten() else {
    tracing::error!("{} choice block outside of an exercise!",if multiple {"multiple"} else {"single"});
    return None
  };
  Some(view!{<Provider value=CurrentChoice(i)>{children()}</Provider>})
}


pub(super) fn problem_choice<V:IntoView+'static>(children:impl Fn() -> V + Send + 'static + Clone) -> impl IntoView {
  use leptos::either::Either;
  let Some(CurrentChoice(block)) = use_context() else {
    tracing::error!("choice outside of choice block!");
    return None
  };
  let Some(ex) = use_context::<CurrentExercise>() else {
    tracing::error!("choice outside of exercise!");
    return None
  };
  let Some((multiple,inline)) = ex.responses.try_update_untracked(|resp| resp.get_mut(block).map(|l| match l {
      ExerciseResponse::MultipleChoice(inline,sigs) => {
        let idx = sigs.len();
        sigs.push(false);
        Some((Either::Left(idx),*inline))
      }
      ExerciseResponse::SingleChoice(inline,sig,total) => {
        let val = *total;
        *total += 1;
        Some((Either::Right(val),*inline))
      }
      ExerciseResponse::Fillinsol(_) => None
    })
  ).flatten().flatten() else {
    tracing::error!("choice outside of choice block!");
    return None
  };
  Some(match multiple {
    Either::Left(idx) => Either::Left(multiple_choice(idx,block,inline,ex.responses,ex.feedback,children)),
    Either::Right(idx) => Either::Right(single_choice(idx,block,inline,ex.responses,ex.uri,ex.feedback,children))
  })
}

fn multiple_choice<V:IntoView+'static>(
  idx:usize,
  block:usize,
  inline:bool,
  responses:RwSignal<SmallVec<ExerciseResponse,4>>,
  feedback:RwSignal<Option<ExerciseFeedback>>,
  children:impl Fn() -> V + Send + 'static + Clone) -> impl IntoView {
    use leptos::either::{EitherOf3 as Either,Either::Left,Either::Right};
    use thaw::Icon;
    move || feedback.with(|v| 
      if let Some(feedback) = v.as_ref() {
        let err = || {
          tracing::error!("Answer to exercise does not match solution!");
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
            let resp = resp.get_mut(block).expect("Signal error in exercise");
            let ExerciseResponse::MultipleChoice(_,v) = resp else { panic!("Signal error in exercise")};
            v[idx] = val;
          }
        );
        let rf = NodeRef::<leptos::html::Input>::new();
        let on_change = move |ev| {
          let Some(ip) = rf.get_untracked() else {return};
          let nv = ip.checked();
          sig.set(nv);
        };
        Either::A(
          view!{
            <div style="display:inline;margin-right:5px;"><input node_ref=rf type="checkbox" on:change=on_change/>{children()}</div>
          }
        )
      }
    )
}

fn single_choice<V:IntoView+'static>(
  idx:u16,
  block:usize,
  inline:bool,
  responses:RwSignal<SmallVec<ExerciseResponse,4>>,
  uri:DocumentElementURI,
  feedback:RwSignal<Option<ExerciseFeedback>>,
  children:impl Fn() -> V + Send + 'static + Clone) -> impl IntoView {
    use leptos::either::{EitherOf3 as Either,Either::Left,Either::Right};
    use thaw::Icon;
    move || feedback.with(|v| {
      if let Some(feedback) = v.as_ref() {
        let err = || {
          tracing::error!("Answer to exercise does not match solution!");
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
            let resp = resp.get_mut(block).expect("Signal error in exercise");
            let ExerciseResponse::SingleChoice(_,i,_) = resp else { panic!("Signal error in exercise")};
            *i = idx;
          }
        );
        let rf = NodeRef::<leptos::html::Input>::new();
        let on_change = move |ev| {
          let Some(ip) = rf.get_untracked() else {return};
          if ip.checked() { sig.set(()); }
        };
        Either::A(view!{
          <div style="display:inline;margin-right:5px;"><input node_ref=rf type="radio" name=name on:change=on_change/>{children()}</div>
        })
      }
    })

}

/* 
  let feedback = ex.feedback;
  move || {
    if feedback.with(|f| f.is_some()) {}
    else {
      
    }
  }
*/

pub(super) fn fillinsol(wd:Option<f32>) -> impl IntoView {
  use leptos::either::{EitherOf3 as Either,Either::Left,Either::Right};
  use thaw::Icon;
  let Some(ex) = use_context::<CurrentExercise>() else {
    tracing::error!("choice outside of exercise!");
    return None
  };
  let Some(choice) = ex.responses.try_update_untracked(|resp| {
      let i = resp.len();
      resp.push(ExerciseResponse::Fillinsol(String::new()));
      i
    }
  ) else {
    tracing::error!("fillinsol outside of an exercise!");
    return None
  };
  let feedback = ex.feedback;
  Some(move || feedback.with(|v|
    if let Some(feedback) = v.as_ref() {
      let err = || {
        tracing::error!("Answer to exercise does not match solution!");
        Either::C(view!(<div style="color:red;">"ERROR"</div>))
      };
      let Some(CheckedResult::FillinSol { matching, text, options }) = feedback.data.get(choice) else {return err()};
      let (correct,feedback) = if let Some(m) = matching {
        let Some(FillinFeedback{is_correct,kind,feedback}) = options.get(*m) else {return err()};

        (*is_correct,Some(feedback.clone()))
      } else {(false,None)};
      let icon = if correct {
        view!(<Icon icon=icondata_ai::AiCheckCircleOutlined style="color:green;"/>)
      } else {
        view!(<Icon icon=icondata_ai::AiCloseCircleOutlined style="color:red;"/>)
      };
      Either::B(view!{
        {icon}" "
        <input type="text" disabled value=text.clone()/>
        {feedback.map(|s| view!(" "<span style="background-color:lightgray;" inner_html=s/>))}
      })
    } else {
      let sig = create_write_slice(ex.responses, 
        move |resps,val| {
          let resp = resps.get_mut(choice).expect("Signal error in exercise");
          let ExerciseResponse::Fillinsol(s) = resp else { panic!("Signal error in exercise")};
          *s = val;
        }
      );
      let style = wd.map(|wd| format!("width:{wd}px;"));
      Either::A(view!{
        <input type="text" style=style on:input:target=move |ev| {sig.set(ev.target().value());}/>
      })
    }
  ))
}
