#![allow(non_local_definitions)]

use std::collections::HashMap;

use flams_ontology::{
    narration::problems::{
        ProblemFeedback, ProblemFeedbackJson, ProblemResponse, SolutionData, Solutions,
    },
    uris::{DocumentElementURI, DocumentURI},
};
use ftml_viewer_components::{
    components::{
        documents::{
            DocumentFromURI, DocumentString, FragmentFromURI, FragmentString, FragmentStringProps,
        },
        problem::ProblemState as OrigState,
        Gotto, TOCElem, TOCSource,
    },
    ts::{
        FragmentContinuation, InputRefContinuation, JFragCont, JInputRefCont, JOnSectTtl, JsOrRsF,
        LeptosContext, NamedJsFunction, OnSectionTitle, TsTopCont,
    },
    AllowHovers, ProblemOptions,
};
use leptos::{either::Either, prelude::*};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(js_name = injectCss)]
pub fn inject_css(mut css: flams_utils::CSS) {
    if let flams_utils::CSS::Link(lnk) = &mut css {
        if let Some(r) = lnk.strip_prefix("srv:") {
            *lnk = format!("{}{r}", ftml_viewer_components::remote::get_server_url());
        }
    }
    flams_web_utils::do_css(css)
}

#[wasm_bindgen] //(js_name = setDebugLog)]
/// activates debug logging
pub fn set_debug_log() {
    let _ = tracing_wasm::try_set_as_global_default();
    console_error_panic_hook::set_once();
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, tsify_next::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "type")]
/// State of a particular problem
pub enum ProblemState {
    /// Users can provide/change their answers
    Interactive {
        /// initial response (if a user has already selected an answer)
        #[serde(default)]
        current_response: Option<ProblemResponse>,
        /// The solution ( => ftml-viewer will take care of matching a response to this solution and compute feedback accordingly )
        #[serde(default)]
        solution: Option<Box<[SolutionData]>>,
    },
    /// No change to the response possible anymore
    Finished {
        #[serde(default)]
        current_response: Option<ProblemResponse>,
    },
    /// Fully graded; feedback provided
    Graded { feedback: ProblemFeedbackJson },
}
impl From<ProblemState> for OrigState {
    fn from(value: ProblemState) -> Self {
        match value {
            ProblemState::Interactive {
                current_response,
                solution,
            } => Self::Interactive {
                current_response,
                solution: solution.map(Solutions::from_solutions),
            },
            ProblemState::Finished { current_response } => Self::Finished { current_response },
            ProblemState::Graded { feedback } => Self::Graded {
                feedback: ProblemFeedback::from_json(feedback),
            },
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, tsify_next::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(transparent)]
pub struct ProblemStates(pub HashMap<DocumentElementURI, ProblemState>);

fn convert(
    on_response: Option<JProblemCont>,
    state: Option<ProblemStates>,
) -> Option<ProblemOptions> {
    if on_response.is_some() || state.is_some() {
        Some(ProblemOptions {
            on_response: on_response.map(|j| JsOrRsF::Js(j.get().into())),
            states: state
                .map(|e| e.0.into_iter().map(|(k, v)| (k, v.into())).collect())
                .unwrap_or_default(),
        })
    } else {
        None
    }
}

#[wasm_bindgen]
/// sets up a leptos context for rendering FTML documents or fragments.
/// If a context already exists, does nothing, so is cheap to call
/// [render_document] and [render_fragment] also inject a context
/// iff none already exists, so this is optional in every case.
pub fn ftml_setup(
    to: leptos::web_sys::HtmlElement,
    children: TsTopCont,
    allow_hovers: Option<bool>,
    on_section_title: Option<JOnSectTtl>,
    on_fragment: Option<JFragCont>,
    on_inputref: Option<JInputRefCont>,
    on_problem: Option<JProblemCont>,
    problem_states: Option<ProblemStates>,
) -> FTMLMountHandle {
    let allow_hovers = allow_hovers.unwrap_or(true);
    let children = children.to_cont();
    let on_section_title = on_section_title.map(|f| OnSectionTitle(f.get().into()));
    let on_fragment = on_fragment.map(|f| f.get().into());
    let on_inputref = on_inputref.map(|f| f.get().into());
    let problem_opts = convert(on_problem, problem_states);

    FTMLMountHandle::new(to, move || {
        view! {
          <GlobalSetup allow_hovers on_fragment on_section_title on_inputref problem_opts>{
            let ret = NodeRef::new();
            ret.on_load(move |e| {
              let owner = Owner::current().expect("Not in a leptos reactive context!");
              if let Err(e) = children.apply(&(e,owner.into())) {
                tracing::error!("Error calling continuation: {e}");
              }
            });
            view!(<div node_ref = ret/>)
          }</GlobalSetup>
        }
    })
}

#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
/// render an FTML document to the provided element
/// #### Errors
pub fn render_document(
    to: leptos::web_sys::HtmlElement,
    document: DocumentOptions,
    context: Option<LeptosContext>,
    allow_hovers: Option<bool>,
    on_section_title: Option<JOnSectTtl>,
    on_fragment: Option<JFragCont>,
    on_inputref: Option<JInputRefCont>,
    on_problem: Option<JProblemCont>,
    problem_states: Option<ProblemStates>,
) -> Result<FTMLMountHandle, String> {
    fn inner(
        to: leptos::web_sys::HtmlElement,
        document: DocumentOptions,
        allow_hovers: bool,
        on_section_title: Option<JOnSectTtl>,
        on_fragment: Option<JFragCont>,
        on_inputref: Option<JInputRefCont>,
        problem_opts: Option<ProblemOptions>,
    ) -> Result<FTMLMountHandle, String> {
        let on_section_title = on_section_title.map(|f| OnSectionTitle(f.get().into()));
        let on_fragment = on_fragment.map(|f| f.get().into());
        let on_inputref = on_inputref.map(|f| f.get().into());

        let comp = move || match document {
            DocumentOptions::HtmlString { html, gottos, toc } => {
                let toc = toc.map_or(TOCSource::None, TOCSource::Ready);
                let gottos = gottos.unwrap_or_default();
                Either::Left(
                    view! {<GlobalSetup allow_hovers on_section_title on_fragment on_inputref problem_opts>
                        <DocumentString html gottos toc/>
                    </GlobalSetup>},
                )
            }
            DocumentOptions::FromBackend { uri, gottos, toc } => {
                let toc = toc.map_or(TOCSource::None, Into::into);
                let gottos = gottos.unwrap_or_default();
                Either::Right(
                    view! {<GlobalSetup allow_hovers on_section_title on_fragment on_inputref problem_opts>
                        <DocumentFromURI uri gottos toc/>
                    </GlobalSetup>},
                )
            }
        };

        Ok(FTMLMountHandle::new(to, move || comp()))
    }
    let allow_hovers = allow_hovers.unwrap_or(true);
    let problem_opts = convert(on_problem, problem_states);
    if let Some(context) = context {
        context.with(move || {
            inner(
                to,
                document,
                allow_hovers,
                on_section_title,
                on_fragment,
                on_inputref,
                problem_opts,
            )
        })
    } else {
        inner(
            to,
            document,
            allow_hovers,
            on_section_title,
            on_fragment,
            on_inputref,
            problem_opts,
        )
    }
}

#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
/// render an FTML document fragment to the provided element
/// #### Errors
pub fn render_fragment(
    to: leptos::web_sys::HtmlElement,
    fragment: FragmentOptions,
    context: Option<LeptosContext>,
    allow_hovers: Option<bool>,
    on_section_title: Option<JOnSectTtl>,
    on_fragment: Option<JFragCont>,
    on_inputref: Option<JInputRefCont>,
    on_problem: Option<JProblemCont>,
    problem_states: Option<ProblemStates>,
) -> Result<FTMLMountHandle, String> {
    fn inner(
        to: leptos::web_sys::HtmlElement,
        fragment: FragmentOptions,
        allow_hovers: bool,
        on_section_title: Option<JOnSectTtl>,
        on_fragment: Option<JFragCont>,
        on_inputref: Option<JInputRefCont>,
        problem_opts: Option<ProblemOptions>,
    ) -> Result<FTMLMountHandle, String> {
        let _ = to.style().set_property(
            "--rustex-this-width",
            "var(--rustex-curr-width,min(800px,100%))",
        );
        let on_section_title = on_section_title.map(|f| OnSectionTitle(f.get().into()));
        let on_fragment = on_fragment.map(|f| f.get().into());
        let on_inputref = on_inputref.map(|f| f.get().into());

        let comp = move || match fragment {
            FragmentOptions::HtmlString { html, uri } => {
                Either::Left(FragmentString(FragmentStringProps { html, uri }))
            }
            FragmentOptions::FromBackend { uri } => Either::Right(view! {<FragmentFromURI uri/>}),
        };
        Ok(FTMLMountHandle::new(to, move || {
            view! {<GlobalSetup allow_hovers on_section_title on_fragment on_inputref problem_opts>
                {comp()}
            </GlobalSetup>}
        }))
    }
    let allow_hovers = allow_hovers.unwrap_or(true);
    let problem_opts = convert(on_problem, problem_states);
    if let Some(context) = context {
        context.with(move || {
            inner(
                to,
                fragment,
                allow_hovers,
                on_section_title,
                on_fragment,
                on_inputref,
                problem_opts,
            )
        })
    } else {
        inner(
            to,
            fragment,
            allow_hovers,
            on_section_title,
            on_fragment,
            on_inputref,
            problem_opts,
        )
    }
}

#[component]
fn GlobalSetup<V: IntoView + 'static>(
    #[prop(default = true)] allow_hovers: bool,
    #[prop(default=None)] on_section_title: Option<OnSectionTitle>,
    #[prop(default=None)] on_fragment: Option<FragmentContinuation>,
    #[prop(default=None)] on_inputref: Option<InputRefContinuation>,
    #[prop(default=None)] problem_opts: Option<ProblemOptions>,
    children: TypedChildren<V>,
) -> impl IntoView {
    use flams_web_utils::components::Themer;
    use ftml_viewer_components::FTMLGlobalSetup;
    //use leptos::either::Either as E;
    use leptos::either::Either::{Left, Right};
    console_error_panic_hook::set_once();
    let children = children.into_inner();
    let children = move || {
        if let Some(on_section_title) = on_section_title {
            provide_context(Some(on_section_title));
        }
        if let Some(on_fragment) = on_fragment {
            provide_context(Some(on_fragment));
        }
        if let Some(on_inputref) = on_inputref {
            provide_context(Some(on_inputref));
        }
        if let Some(problem_opts) = problem_opts {
            provide_context(problem_opts);
        }
        provide_context(AllowHovers(allow_hovers));
        children()
    };

    let children = move || {
        if ftml_viewer_components::is_in_ftml() {
            Left(children())
        } else {
            Right(view!(<FTMLGlobalSetup>{children()}</FTMLGlobalSetup>))
        }
    };

    //let r = owner.with(move || {
    if with_context::<thaw::ConfigInjection, _>(|_| ()).is_none() {
        Left(view!(<Themer>{children()}</Themer>))
    } else {
        Right(children())
    }
    //});
    //on_cleanup(move || drop(owner));
    //r
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, tsify_next::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
/// Options for rendering an FTML document
/// - `FromBackend`: calls the backend for the document
///     uri: the URI of the document (as string)
///     toc: if defined, will render a table of contents for the document
// - Prerendered: Take the existent DOM HTMLElement as is
/// - `HtmlString`: render the provided HTML String
///     html: the HTML String
///     toc: if defined, will render a table of contents for the document
#[serde(tag = "type")]
pub enum DocumentOptions {
    FromBackend {
        uri: DocumentURI,
        #[serde(default)]
        gottos: Option<Vec<Gotto>>,
        //#[serde(default)] <- this breaks toc:"GET" for some reason
        toc: Option<TOCOptions>,
    },
    //Prerendered,
    HtmlString {
        html: String,
        #[serde(default)]
        gottos: Option<Vec<Gotto>>,
        //#[serde(default)] <- this breaks toc:"GET" for some reason
        toc: Option<Vec<TOCElem>>,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, tsify_next::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "type")]
/// Options for rendering an FTML document fragment
/// - `FromBackend`: calls the backend for the document fragment
///     uri: the URI of the document fragment (as string)
// - Prerendered: Take the existent DOM HTMLElement as is
/// - `HtmlString`: render the provided HTML String
///     html: the HTML String
pub enum FragmentOptions {
    FromBackend {
        uri: DocumentElementURI,
    },
    //Prerendered,
    HtmlString {
        html: String,
        #[serde(default)]
        uri: Option<DocumentElementURI>,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, tsify_next::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
/// Options for rendering a table of contents
/// `GET` will retrieve it from the remote backend
/// `TOCElem[]` will render the provided TOC
pub enum TOCOptions {
    #[serde(rename = "GET")]
    GET,
    Predefined(Vec<TOCElem>),
}

impl From<TOCOptions> for TOCSource {
    fn from(value: TOCOptions) -> Self {
        match value {
            TOCOptions::GET => Self::Get,
            TOCOptions::Predefined(toc) => Self::Ready(toc),
        }
    }
}

ftml_viewer_components::ts_function! {
  JProblemCont ProblemCont @ "(r:ProblemResponse) => void"
  = ProblemResponse => ()
}

/*
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, tsify_next::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum ProblemOption {
    WithFeedback(Vec<(DocumentElementURI, ProblemFeedback)>),
    WithSolutions(Vec<(DocumentElementURI, Solutions)>),
}
impl Into<ProblemOptions> for ProblemOption {
    fn into(self) -> ProblemOptions {
        match self {
            Self::WithFeedback(v) => ProblemOptions::WithFeedback(v.into()),
            Self::WithSolutions(s) => ProblemOptions::WithSolutions(s.into()),
        }
    }
}
 */

#[wasm_bindgen]
pub struct FTMLMountHandle {
    mount: std::cell::Cell<
        Option<leptos::prelude::UnmountHandle<leptos::tachys::view::any_view::AnyViewState>>,
    >,
}

#[wasm_bindgen]
impl FTMLMountHandle {
    /// unmounts the view and cleans up the reactive system.
    /// Not calling this is a memory leak
    pub fn unmount(&self) -> Result<(), wasm_bindgen::JsError> {
        if let Some(mount) = self.mount.take() {
            drop(mount); //try_catch(move || drop(mount))?;
        }
        Ok(())
    }
    fn new<V: IntoView + 'static>(
        div: leptos::web_sys::HtmlElement,
        f: impl FnOnce() -> V + 'static,
    ) -> Self {
        let handle = leptos::prelude::mount_to(div, move || f().into_any());
        Self {
            mount: std::cell::Cell::new(Some(handle)),
        }
    }
}
