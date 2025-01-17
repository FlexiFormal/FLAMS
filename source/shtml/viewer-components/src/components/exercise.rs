use immt_ontology::uris::DocumentElementURI;
use leptos::{context::Provider, prelude::*};
use leptos_dyn_dom::OriginalNode;
use shtml_extraction::prelude::SHTMLElements;
use smallvec::SmallVec;

#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature="ts",derive(tsify_next::Tsify))]
#[cfg_attr(feature="ts",tsify(into_wasm_abi, from_wasm_abi))]
pub struct ResponseWrapper(immt_ontology::narration::exercises::ExerciseResponse);


#[derive(Clone,Debug)]
pub(crate) struct CurrentExercise {
  uri:DocumentElementURI,
  responses:SmallVec<ExerciseResponse,4>
}
impl CurrentExercise {
  pub fn to_response(&self) -> ResponseWrapper {
    ResponseWrapper(immt_ontology::narration::exercises::ExerciseResponse {
      uri:self.uri.clone(),
      responses:self.responses.iter().map(|r|
        match r {
          ExerciseResponse::MultipleChoice(sigs) =>
            immt_ontology::narration::exercises::ExerciseResponseType::MultipleChoice(sigs.clone()),
          ExerciseResponse::SingleChoice(sig,_) =>
            immt_ontology::narration::exercises::ExerciseResponseType::SingleChoice(*sig)
        }
      ).collect()
    })
  }
}

#[derive(Clone,Debug)]
enum ExerciseResponse {
  MultipleChoice(SmallVec<bool,8>),
  SingleChoice(u16,u16),
}

pub(super) fn exercise<V:IntoView+'static>(uri:DocumentElementURI,autogradable:bool,sub_exercise:bool,children:impl FnOnce() -> V + Send + 'static) -> impl IntoView {
  let ex = RwSignal::new(CurrentExercise{uri,responses:SmallVec::new()});
  let _ = Effect::new(move |_| {
    if let Some(resp) = ex.try_with(|ex| ex.to_response()) {
      tracing::info!("Response: {:?}",resp.0);
    }
  });
  view!{
    <Provider value=ex>
      <div style="border:1px solid red;margin:3px;">
        <form>{children()}</form>
      </div>
    </Provider>
  }
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

pub(super) fn solution(skip:usize,elements:SHTMLElements,orig:OriginalNode,on_load:RwSignal<bool>,_id:Option<Box<str>>) -> impl IntoView {
  #[cfg(any(feature="csr",feature="hydrate"))]
  {
    if orig.child_element_count() == 0 {
      tracing::debug!("Solution removed!");
    } else {
      tracing::debug!("Solution exists!");
    }
    on_load.set(true);
    // TODO
  }
  #[cfg(not(any(feature="csr",feature="hydrate")))]
  {()}
}

pub(super) fn gnote(skip:usize,elements:SHTMLElements,orig:OriginalNode,on_load:RwSignal<bool>) -> impl IntoView {
  #[cfg(any(feature="csr",feature="hydrate"))]
  {
    if orig.child_element_count() == 0 {
      tracing::debug!("Grading note removed!");
    } else {
      tracing::debug!("Grading note exists!");
    }
    on_load.set(true);
    // TODO
  }
  #[cfg(not(any(feature="csr",feature="hydrate")))]
  {()}
}

#[derive(Clone)]
struct CurrentChoice(usize);

pub(super) fn choice_block<V:IntoView+'static>(multiple:bool,children:impl FnOnce() -> V + Send + 'static) -> impl IntoView {
  let response = if multiple {
    ExerciseResponse::MultipleChoice(SmallVec::new())
  } else {
    ExerciseResponse::SingleChoice(0,0)
  };
  let Some(i) = with_context::<RwSignal<CurrentExercise>,_>(|ex|
    ex.try_update_untracked(|ex| {
      let i = ex.responses.len();
      ex.responses.push(response);
      i
    })
  ).flatten() else {
    tracing::error!("{} choice block outside of an exercise!",if multiple {"multiple"} else {"single"});
    return None
  };
  Some(view!{<Provider value=CurrentChoice(i)>{children()}</Provider>})
}


pub(super) fn problem_choice<V:IntoView+'static>(children:impl FnOnce() -> V + Send + 'static) -> impl IntoView {
  use leptos::either::Either;
  let Some(CurrentChoice(choice)) = use_context() else {
    tracing::error!("choice outside of choice block!");
    return None
  };
  let Some(ex) = use_context::<RwSignal<CurrentExercise>>() else {
    tracing::error!("choice outside of exercise!");
    return None
  };
  tracing::info!("New Choice: {}@{choice}; has {} responses",ex.with_untracked(|ex| ex.uri.to_string()),ex.with_untracked(|ex| ex.responses.len()));
  let Some(multiple) = ex.try_update_untracked(|ex| ex.responses.get_mut(choice).map(|l| match l {
      ExerciseResponse::MultipleChoice(sigs) => {
        let idx = sigs.len();
        sigs.push(false);
        Either::Left(idx)
      }
      ExerciseResponse::SingleChoice(sig,total) => {
        let val = *total;
        *total += 1;
        Either::Right((val,ex.uri.clone()))
      }
    })
  ).flatten() else {
    tracing::error!("choice outside of choice block!");
    return None
  };
  Some(match multiple {
    Either::Left(idx) => Either::Left({
      let sig = create_write_slice(ex, 
        move |ex,val| {
          let resp = ex.responses.get_mut(choice).expect("Signal error in exercise");
          let ExerciseResponse::MultipleChoice(v) = resp else { panic!("Signal error in exercise")};
          v[idx] = val;
        }
      );
      let rf = NodeRef::<leptos::html::Input>::new();
      let on_change = move |ev| {
        let Some(ip) = rf.get_untracked() else {return};
        let nv = ip.checked();
        sig.set(nv);
      };
      view!{
        <div style="display:inline;margin-right:5px;"><input node_ref=rf type="checkbox" on:change=on_change/>{children()}</div>
      }
    }),
    Either::Right((idx,uri)) => Either::Right({
      let sig = create_write_slice(ex, 
        move |ex,()| {
          let resp = ex.responses.get_mut(choice).expect("Signal error in exercise");
          let ExerciseResponse::SingleChoice(i,_) = resp else { panic!("Signal error in exercise")};
          *i = idx;
        }
      );
      let name = format!("{uri}{choice}");
      let rf = NodeRef::<leptos::html::Input>::new();
      let on_change = move |ev| {
        let Some(ip) = rf.get_untracked() else {return};
        if ip.checked() { sig.set(()); }
      };
      view!{
        <div style="display:inline;margin-right:5px;"><input node_ref=rf type="radio" name=name on:change=on_change/>{children()}</div>
      }

    })
  })
}

