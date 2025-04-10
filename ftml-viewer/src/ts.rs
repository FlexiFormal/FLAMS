#![allow(non_local_definitions)]

use flams_ontology::{
    narration::problems::{ProblemFeedback, ProblemResponse, Solutions},
    uris::{DocumentElementURI, DocumentURI},
};
use flams_web_utils::try_catch;
use ftml_viewer_components::{
    components::{
        documents::{
            DocumentFromURI, DocumentString, FragmentFromURI, FragmentString, FragmentStringProps,
        },
        Gotto, TOCElem, TOCSource,
    },
    ts::{
        InputRefContinuation, JInputRefCont, JOnSectTtl, JParaCont, JSectCont, JSlideCont,
        LeptosContext, NamedJsFunction, OnSectionTitle, ParagraphContinuation, SectionContinuation,
        SlideContinuation, TsTopCont,
    },
    AllowHovers, ProblemOptions,
};
use leptos::{either::Either, prelude::*};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen] //(js_name = setDebugLog)]
/// activates debug logging
pub fn set_debug_log() {
    let _ = tracing_wasm::try_set_as_global_default();
    console_error_panic_hook::set_once();
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
    on_section: Option<JSectCont>,
    on_section_title: Option<JOnSectTtl>,
    on_paragraph: Option<JParaCont>,
    on_inputref: Option<JInputRefCont>,
    on_slide: Option<JSlideCont>,
    problem_opts: Option<ProblemOption>,
    on_problem: Option<JProblemCont>,
) -> FTMLMountHandle {
    let allow_hovers = allow_hovers.unwrap_or(true);
    let children = children.to_cont();
    let on_section = on_section.map(|f| SectionContinuation(f.get().into()));
    let on_section_title = on_section_title.map(|f| OnSectionTitle(f.get().into()));
    let on_paragraph = on_paragraph.map(|f| f.get().into());
    let on_inputref = on_inputref.map(|f| f.get().into());
    let on_slide = on_slide.map(|f| f.get().into());

    FTMLMountHandle::new(to, move || {
        view! {
          <GlobalSetup allow_hovers on_section on_section_title on_paragraph on_inputref on_slide problem_opts on_problem>{
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
    on_section: Option<JSectCont>,
    on_section_title: Option<JOnSectTtl>,
    on_paragraph: Option<JParaCont>,
    on_inputref: Option<JInputRefCont>,
    on_slide: Option<JSlideCont>,
    problem_opts: Option<ProblemOption>,
    on_problem: Option<JProblemCont>,
) -> Result<FTMLMountHandle, String> {
    fn inner(
        to: leptos::web_sys::HtmlElement,
        document: DocumentOptions,
        allow_hovers: bool,
        on_section: Option<JSectCont>,
        on_section_title: Option<JOnSectTtl>,
        on_paragraph: Option<JParaCont>,
        on_inputref: Option<JInputRefCont>,
        on_slide: Option<JSlideCont>,
        problem_opts: Option<ProblemOption>,
        on_problem: Option<JProblemCont>,
    ) -> Result<FTMLMountHandle, String> {
        let on_section = on_section.map(|f| SectionContinuation(f.get().into()));
        let on_section_title = on_section_title.map(|f| OnSectionTitle(f.get().into()));
        let on_paragraph = on_paragraph.map(|f| f.get().into());
        let on_inputref = on_inputref.map(|f| f.get().into());
        let on_slide = on_slide.map(|f| f.get().into());

        let comp = move || match document {
            DocumentOptions::HtmlString { html, gottos, toc } => {
                let toc = toc.map_or(TOCSource::None, TOCSource::Ready);
                let gottos = gottos.unwrap_or_default();
                Either::Left(
                    view! {<GlobalSetup allow_hovers on_section on_section_title on_paragraph on_inputref on_slide problem_opts on_problem><DocumentString html gottos toc/></GlobalSetup>},
                )
            }
            DocumentOptions::FromBackend { uri, gottos, toc } => {
                let toc = toc.map_or(TOCSource::None, Into::into);
                let gottos = gottos.unwrap_or_default();
                Either::Right(
                    view! {<GlobalSetup allow_hovers on_section on_section_title on_paragraph on_inputref on_slide problem_opts on_problem><DocumentFromURI uri gottos toc/></GlobalSetup>},
                )
            }
        };

        Ok(FTMLMountHandle::new(to, move || comp()))
    }
    let allow_hovers = allow_hovers.unwrap_or(true);
    if let Some(context) = context {
        context.with(move || {
            inner(
                to,
                document,
                allow_hovers,
                on_section,
                on_section_title,
                on_paragraph,
                on_inputref,
                on_slide,
                problem_opts,
                on_problem,
            )
        })
    } else {
        inner(
            to,
            document,
            allow_hovers,
            on_section,
            on_section_title,
            on_paragraph,
            on_inputref,
            on_slide,
            problem_opts,
            on_problem,
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
    on_section: Option<JSectCont>,
    on_section_title: Option<JOnSectTtl>,
    on_paragraph: Option<JParaCont>,
    on_inputref: Option<JInputRefCont>,
    on_slide: Option<JSlideCont>,
    problem_opts: Option<ProblemOption>,
    on_problem: Option<JProblemCont>,
) -> Result<FTMLMountHandle, String> {
    fn inner(
        to: leptos::web_sys::HtmlElement,
        fragment: FragmentOptions,
        allow_hovers: bool,
        on_section: Option<JSectCont>,
        on_section_title: Option<JOnSectTtl>,
        on_paragraph: Option<JParaCont>,
        on_inputref: Option<JInputRefCont>,
        on_slide: Option<JSlideCont>,
        problem_opts: Option<ProblemOption>,
        on_problem: Option<JProblemCont>,
    ) -> Result<FTMLMountHandle, String> {
        let on_section = on_section.map(|f| SectionContinuation(f.get().into()));
        let on_section_title = on_section_title.map(|f| OnSectionTitle(f.get().into()));
        let on_paragraph = on_paragraph.map(|f| f.get().into());
        let on_inputref = on_inputref.map(|f| f.get().into());
        let on_slide = on_slide.map(|f| f.get().into());

        let comp = move || match fragment {
            FragmentOptions::HtmlString { html, uri } => {
                Either::Left(FragmentString(FragmentStringProps { html, uri }))
            }
            FragmentOptions::FromBackend { uri } => Either::Right(view! {<FragmentFromURI uri/>}),
        };
        Ok(FTMLMountHandle::new(
            to,
            move || view! {<GlobalSetup allow_hovers on_section on_section_title on_paragraph on_inputref on_slide problem_opts on_problem>{comp()}</GlobalSetup>},
        ))
    }
    let allow_hovers = allow_hovers.unwrap_or(true);
    if let Some(context) = context {
        context.with(move || {
            inner(
                to,
                fragment,
                allow_hovers,
                on_section,
                on_section_title,
                on_paragraph,
                on_inputref,
                on_slide,
                problem_opts,
                on_problem,
            )
        })
    } else {
        inner(
            to,
            fragment,
            allow_hovers,
            on_section,
            on_section_title,
            on_paragraph,
            on_inputref,
            on_slide,
            problem_opts,
            on_problem,
        )
    }
}

#[component]
fn GlobalSetup<V: IntoView + 'static>(
    #[prop(default = true)] allow_hovers: bool,
    #[prop(default=None)] on_section: Option<SectionContinuation>,
    #[prop(default=None)] on_section_title: Option<OnSectionTitle>,
    #[prop(default=None)] on_paragraph: Option<ParagraphContinuation>,
    #[prop(default=None)] on_inputref: Option<InputRefContinuation>,
    #[prop(default=None)] on_slide: Option<SlideContinuation>,
    #[prop(default=None)] problem_opts: Option<ProblemOption>,
    #[prop(default=None)] on_problem: Option<JProblemCont>,
    children: TypedChildren<V>,
) -> impl IntoView {
    use flams_web_utils::components::Themer;
    use ftml_viewer_components::FTMLGlobalSetup;
    //use leptos::either::Either as E;
    use leptos::either::Either::{Left, Right};
    console_error_panic_hook::set_once();
    let problem_opts = if let Some(on_problem) = on_problem {
        Some(ProblemOptions::OnResponse(on_problem.get().into()))
    } else {
        problem_opts.map(Into::into)
    };
    let children = children.into_inner();
    let children = move || {
        if let Some(on_section) = on_section {
            provide_context(Some(on_section));
        }
        if let Some(on_section_title) = on_section_title {
            provide_context(Some(on_section_title));
        }
        if let Some(on_paragraph) = on_paragraph {
            provide_context(Some(on_paragraph));
        }
        if let Some(on_inputref) = on_inputref {
            provide_context(Some(on_inputref));
        }
        if let Some(on_slide) = on_slide {
            provide_context(Some(on_slide));
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
#[serde(untagged)]
/// Options for rendering an FTML document
/// - `FromBackend`: calls the backend for the document
///     uri: the URI of the document (as string)
///     toc: if defined, will render a table of contents for the document
// - Prerendered: Take the existent DOM HTMLElement as is
/// - `HtmlString`: render the provided HTML String
///     html: the HTML String
///     toc: if defined, will render a table of contents for the document
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
#[serde(untagged)]
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
    #[serde(untagged)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, tsify_next::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum ProblemOption {
    //OnResponse( #[tsify(type = "(r:ProblemResponse) => void")] ProblemCont),
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

/*
pub mod server {
  use wasm_bindgen::prelude::wasm_bindgen;
  use ftml_viewer_components::remote::{ServerConfig,server_config};
  pub use flams_utils::CSS;
  use tsify_next::Tsify;

  #[derive(Tsify, serde::Serialize, serde::Deserialize)]
  #[tsify(into_wasm_abi, from_wasm_abi)]
  pub struct HTMLFragment {
    pub css: Vec<CSS>,
    pub html: String
  }

  #[wasm_bindgen]
  /// #### Errors
  pub async fn get_document_html(doc:&str) -> Result<HTMLFragment,String> {
    let doc = doc.parse().map_err(|e| "invalid document URI".to_string())?;
    server_config.inputref(doc).await.map(|(_,css,html)|
      HTMLFragment {css, html}
    )
  }

  #[wasm_bindgen]
  /// #### Errors
  pub async fn get_paragraph_html(elem:&str) -> Result<HTMLFragment,String> {
    let doc = elem.parse().map_err(|e| "invalid document URI".to_string())?;
    server_config.paragraph(doc).await.map(|(_,css,html)|
      HTMLFragment {css, html}
    )
  }
}
   */

/*
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
/// render an FTML document to the provided element
/// #### Errors
pub fn render_document_b(
  to:leptos::web_sys::HtmlElement,
  document:DocumentOptions,
  on_section_start: Option<JSectContB>,
  on_section_end: Option<JSectContB>,
  context:Option<LeptosContext>
) -> Result<FTMLMountHandle,String> {
  fn inner(
    to:leptos::web_sys::HtmlElement,
    document:DocumentOptions,
    on_section_start: Option<JSectContB>,
    on_section_end: Option<JSectContB>
  ) -> Result<FTMLMountHandle,String> {
    use ftml_viewer_components::FTMLGlobalSetup;
    use flams_web_utils::components::Themer;

    let comp = move || match document {
      DocumentOptions::HtmlString{html,toc} => {
        let toc = toc.map_or(TOCSource::None,TOCSource::Ready);
        Either::Left(view!{<GlobalSetup><DocumentString html toc/></GlobalSetup>})
      }
      DocumentOptions::FromBackend{uri,toc} => {
        let toc = toc.map_or(TOCSource::None,Into::into);
        Either::Right(view!{<GlobalSetup><DocumentFromURI uri toc/></GlobalSetup>})
      }
    };

    Ok(FTMLMountHandle::new(to,move || {
      if let Some(start) = on_section_start {
        ftml_viewer_components::components::OnSectionBegin::set(start.get().into());
      };
      if let Some(end) = on_section_end {
        ftml_viewer_components::components::OnSectionEnd::set(end.get().into());
      };
      comp()
    }))
  }
  if let Some(context) = context {
    context.with(move || inner(to,document,on_section_start,on_section_end))
  } else {
    inner(to,document,on_section_start,on_section_end)
  }
}


#[wasm_bindgen]
/// sets up a leptos context for rendering FTML documents or fragments.
/// If a context already exists, does nothing, so is cheap to call
/// [render_document] and [render_fragment] also inject a context
/// iff none already exists, so this is optional in every case.
pub fn ftml_setup_b(to:leptos::web_sys::HtmlElement,cont:TsTopCont) -> FTMLMountHandle {
  let cont = cont.to_cont();
  FTMLMountHandle::new(to,move || view! {
    <GlobalSetup>{cont.view()}</GlobalSetup>
  })
}


#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
/// render an FTML document fragment to the provided element
/// #### Errors
pub fn render_fragment_with_cont_b(
  to:leptos::web_sys::HtmlElement,
  fragment:FragmentOptions,
  context:Option<LeptosContext>,
  problem_cont:JProblemCont
) -> Result<FTMLMountHandle,String> {
  let cont = ProblemOptions::OnResponse(problem_cont.get().into());
  fn inner(
    to:leptos::web_sys::HtmlElement,
    fragment:FragmentOptions,
    cont:ProblemOptions
  ) -> Result<FTMLMountHandle,String> {
    use ftml_viewer_components::FTMLGlobalSetup;
    use flams_web_utils::components::Themer;

    let comp = move || match fragment {
      FragmentOptions::HtmlString{html,uri} => {
        Either::Left(FragmentString(FragmentStringProps {html,uri}))
      }
      FragmentOptions::FromBackend{uri} => {
        Either::Right(view!{<FragmentFromURI uri/>})
      }
    };

    Ok(FTMLMountHandle::new(to,move || view!{
      <GlobalSetup><Provider value=cont>{comp()}</Provider></GlobalSetup>
    }))
  }
  if let Some(context) = context {
    context.with(move || inner(to,fragment,cont))
  } else {
    inner(to,fragment,cont)
  }
}

#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
/// render an FTML document fragment to the provided element
/// #### Errors
pub fn render_fragment_b(
  to:leptos::web_sys::HtmlElement,
  fragment:FragmentOptions,
  context:Option<LeptosContext>,
  problem_options:Option<ProblemOption>
) -> Result<FTMLMountHandle,String> {
  fn inner(
    to:leptos::web_sys::HtmlElement,
    fragment:FragmentOptions,
    problem_options:Option<ProblemOption>
  ) -> Result<FTMLMountHandle,String> {
    use ftml_viewer_components::FTMLGlobalSetup;
    use flams_web_utils::components::Themer;

    let comp = move || match fragment {
      FragmentOptions::HtmlString{html,uri} => {
        Either::Left(FragmentString(FragmentStringProps {html,uri}))
      }
      FragmentOptions::FromBackend{uri} => {
        Either::Right(view!{<FragmentFromURI uri/>})
      }
    };

    let comp = move || if let Some(opt) = problem_options {
      let opt: ProblemOptions = opt.into();
      Either::Left(view!{<GlobalSetup><Provider value=opt>{comp()}</Provider></GlobalSetup>})
    } else {
      Either::Right(view!{<GlobalSetup>{comp()}</GlobalSetup>})
    };

    Ok(FTMLMountHandle::new(to,move || comp()))
  }
  if let Some(context) = context {
    context.with(move || inner(to,fragment,problem_options))
  } else {
    inner(to,fragment,problem_options)
  }
}
 */
