use crate::components::SearchState;
use flams_ontology::{
    search::{QueryFilter, SearchResult, SearchResultKind},
    uris::{
        ArchiveId, ArchiveURITrait, ContentURITrait, DocumentElementURI, DocumentURI, NarrativeURI,
        PathURITrait, SymbolURI, URI,
    },
};
use flams_router_base::uris::{URIComponents, URIComponentsTrait};
use flams_router_vscode::{
    VSCode,
    components::{VSCodeButton, VSCodeCheckbox, VSCodeRadio, VSCodeRadioGroup, VSCodeTextbox},
};
use flams_utils::{impossible, unwrap};
use flams_web_utils::{components::wait_and_then_fn, do_css, inject_css};
use ftml_viewer_components::components::omdoc::{comma_sep, doc_name, symbol_name};
use leptos::prelude::*;

#[component]
pub fn VSCodeSearch() -> impl IntoView {
    inject_css("flams-search-block", include_str!("vscode.css"));
    use flams_web_utils::components::Themer;
    use ftml_viewer_components::FTMLGlobalSetup;

    let remote = || leptos_router::hooks::use_query_map().with(|q| q.get_string("remote"));

    let selected_radio = RwSignal::new(Some("doc".to_string()));
    let disabled =
        Memo::new(move |_| selected_radio.with(|s| s.as_ref().is_some_and(|s| s == "symbol")));

    let full_docs = RwSignal::new(false);
    let paras = RwSignal::new(true);
    let defs = RwSignal::new(true);
    let exs = RwSignal::new(true);
    let asss = RwSignal::new(false);
    let probs = RwSignal::new(false);
    let query = RwSignal::new(String::default());
    let opts = Memo::new(move |_| {
        let mut ret = QueryFilter::default();
        ret.allow_documents = full_docs.get();
        ret.allow_paragraphs = paras.get();
        ret.allow_definitions = defs.get();
        ret.allow_examples = exs.get();
        ret.allow_assertions = asss.get();
        ret.allow_problems = probs.get();
        ret
    });
    let local_results = RwSignal::new(SearchState::None);
    let remote_results = RwSignal::new(SearchState::None);
    let local_act = Action::new(move |&()| {
        let query = query.get_untracked();
        local_results.set(SearchState::Loading);
        let opts = opts.get_untracked();
        async move {
            match super::search_query(query, opts, 20).await {
                Ok(r) => local_results.set(SearchState::Results(r)),
                Err(_) => {
                    local_results.set(SearchState::None);
                }
            }
        }
    });
    let remote_act = Action::new(move |&()| {
        let remote = remote();
        let query = query.get_untracked();
        remote_results.set(SearchState::Loading);
        let opts = opts.get_untracked();
        async move {
            let Some(remote) = remote else { return };
            #[cfg(feature = "hydrate")]
            {
                use flams_router_base::ServerFnExt;
                let query = super::SearchQuery {
                    query,
                    opts,
                    num_results: 20,
                }
                .call_remote(remote)
                .await;
                match query {
                    Ok(r) => remote_results.set(SearchState::Results(r)),
                    Err(_) => {
                        remote_results.set(SearchState::None);
                    }
                }
            }
        }
    });
    let local_sym_act = Action::new(move |&()| {
        let query = query.get_untracked();
        local_results.set(SearchState::Loading);
        async move {
            match super::search_symbols(query, 20).await {
                Ok(r) => local_results.set(SearchState::SymResults(r)),
                Err(_) => {
                    local_results.set(SearchState::None);
                }
            }
        }
    });
    let remote_sym_act = Action::new(move |&()| {
        let remote = remote();
        let query = query.get_untracked();
        remote_results.set(SearchState::Loading);
        async move {
            let Some(remote) = remote else { return };
            #[cfg(feature = "hydrate")]
            {
                use flams_router_base::ServerFnExt;
                let query = super::SearchSymbols {
                    query,
                    num_results: 20,
                }
                .call_remote(remote)
                .await;
                match query {
                    Ok(r) => remote_results.set(SearchState::SymResults(r)),
                    Err(_) => {
                        remote_results.set(SearchState::None);
                    }
                };
            }
        }
    });
    Effect::new(move || {
        if query.with(String::is_empty) {
            local_results.set(SearchState::None);
            return;
        }
        if selected_radio.with(|v| v.as_ref().is_some_and(|s| s == "symbol")) {
            local_sym_act.dispatch(());
            remote_sym_act.dispatch(());
        } else {
            let _ = opts.get();
            local_act.dispatch(());
            remote_act.dispatch(());
        }
    });

    view! {
        <div style="display:flex;flex-direction:column;">
            <VSCodeTextbox value=query placeholder="Search"/>
            <VSCodeRadioGroup name="flams-vscode-search" selected=selected_radio>
                <div style="display:flex;flex-direction:row;">
                    <VSCodeRadio id="symbol">"Symbols"</VSCodeRadio>
                    <VSCodeRadio id="doc">"Paragraphs"</VSCodeRadio>
                </div>
            </VSCodeRadioGroup>
            <div style="display:flex;flex-direction:row;flex-wrap:wrap;">
                <VSCodeCheckbox checked=full_docs disabled>"Full Documents"</VSCodeCheckbox>
                <VSCodeCheckbox checked=paras disabled>"Paragraphs"</VSCodeCheckbox>
                <VSCodeCheckbox checked=defs disabled>"Definitions"</VSCodeCheckbox>
                <VSCodeCheckbox checked=exs disabled>"Examples"</VSCodeCheckbox>
                <VSCodeCheckbox checked=asss disabled>"Assertions"</VSCodeCheckbox>
                <VSCodeCheckbox checked=probs disabled>"Problems"</VSCodeCheckbox>
                <Themer><FTMLGlobalSetup>
                {do_results("Local Results",None,local_results)}
                <div style="margin-top:25px;"></div>
                {do_results("Remote Results",Some(remote),remote_results)}
                </FTMLGlobalSetup></Themer>
            </div>
        </div>
    }
}

fn do_results(
    pre: &'static str,
    remote: Option<fn() -> Option<String>>,
    results: RwSignal<SearchState>,
) -> impl IntoView {
    use leptos::either::EitherOf5::*;
    let pre_view =
        move || view! {<div style="width:100%;font-weight:bold;text-align:center;">{pre}</div>};
    move || {
        results.with(|r| match r {
            SearchState::None => A(()),
            SearchState::Results(v) if v.is_empty() => B(view!({pre_view}"(No results)")),
            SearchState::Loading => C(view!({pre_view}<flams_web_utils::components::Spinner/>)),
            SearchState::SymResults(v) => D(view!({pre_view}{v
            .iter()
            .map(|(sym, res)| do_sym_result(sym, res.clone(),remote))
            .collect_view()})),
            SearchState::Results(v) => E(view!({pre_view}{v
            .iter()
            .map(|(score, res)| do_result(*score, res,remote))
            .collect_view()})),
        })
    }
}

fn do_result(
    score: f32,
    res: &SearchResult,
    remote: Option<fn() -> Option<String>>,
) -> impl IntoView + use<> {
    use leptos::either::Either::*;
    match res {
        SearchResult::Document(d) => Left(do_doc(score, d.clone(), remote)),
        SearchResult::Paragraph {
            uri, fors, kind, ..
        } => Right(do_para(score, uri.clone(), *kind, fors.clone(), remote)),
    }
}

#[derive(leptos::server_fn::serde::Serialize, Debug, Clone)]
struct Usemodule {
    archive: ArchiveId,
    path: String,
}
impl Usemodule {
    fn make(uri: &SymbolURI) -> Self {
        let module = uri.module();
        let archive = module.archive_id().clone();
        let path = if let Some(p) = module.path() {
            format!("{p}?{}", module.name().first_name())
        } else {
            module.name().first_name().to_string()
        };
        Self { archive, path }
    }
}

fn do_sym_result(
    sym: &SymbolURI,
    res: Vec<(f32, SearchResult)>,
    remote: Option<fn() -> Option<String>>,
) -> impl IntoView + use<> {
    use thaw::Scrollbar;
    let vs = unwrap!(VSCode::get());
    let usemodule = if remote.is_none() {
        Some(Usemodule::make(sym))
    } else {
        None
    };
    let name = ftml_viewer_components::components::omdoc::symbol_name(sym, &sym.to_string());
    view! {
        <div class="flams-search-block">
            <div><b>{name}</b>
                {
                    usemodule.map(|u| {
                        let on_click = move |_| {
                            let _ = vs.post_message(u.clone());
                        };
                        view!{
                            <div style="width:100%"><div style="margin-left:auto;width:fit-content;" on:click=on_click>
                                <VSCodeButton>"\\usemodule"</VSCodeButton>
                            </div></div>
                        }
                    })
                }
            </div>
            <div style="display:block">
            <div style="padding:0 5px;max-width:100%">
                <div style="width:100%;color:black;background-color:white;">
                  <Scrollbar style="max-height: 100px;width:100%;max-width:100%;">{
                    res.into_iter().map(|(_,r)| {
                      let SearchResult::Paragraph { uri, .. } = r else { impossible!()};
                      fragment(uri.into(),remote)
                    }).collect_view()
                  }
                  </Scrollbar>
                </div>
              </div>
            </div>
        </div>
    }
}

fn do_doc(score: f32, uri: DocumentURI, remote: Option<fn() -> Option<String>>) -> impl IntoView {
    use thaw::Scrollbar;
    let name = doc_name(&uri, uri.name().to_string());
    view! {
        <div class="flams-search-block">
            <div><b>"Document "{name}</b>
                <div style="width:100%"><div style="margin-left:auto;width:fit-content;">"Score: "{score}</div></div>
            </div>
            <div style="display:block">
            <div style="padding:0 5px;max-width:100%">
                <div style="width:100%;color:black;background-color:white;">
                  <Scrollbar style="max-height: 100px;width:100%;max-width:100%;">
                    {fragment(uri.into(),remote)}
                  </Scrollbar>
                </div>
              </div>
            </div>
        </div>
    }
}

fn do_para(
    score: f32,
    uri: DocumentElementURI,
    kind: SearchResultKind,
    fors: Vec<SymbolURI>,
    remote: Option<fn() -> Option<String>>,
) -> impl IntoView {
    use thaw::Scrollbar;
    let uristr = uri.to_string();
    let name = uristr;
    let desc = comma_sep(
        "For",
        fors.into_iter()
            .map(|s| symbol_name(&s, s.name().last_name().as_ref())),
    );
    view! {
        <div class="flams-search-block">
            <div><b>{kind.as_str()}" "{name}</b>
                <div style="width:100%"><div style="margin-left:auto;width:fit-content;">"Score: "{score}</div></div>
            </div>
            <div style="display:block">
            <div style="padding:0 5px;max-width:100%">
                <div style="width:100%;color:black;background-color:white;">
                  <Scrollbar style="max-height: 100px;width:100%;max-width:100%;">
                    {fragment(uri.into(),remote)}
                  </Scrollbar>
                </div>
              </div>
            </div>
        </div>
    }
}

fn fragment(uri: NarrativeURI, remote: Option<fn() -> Option<String>>) -> impl IntoView {
    use flams_router_content::components::Fragment;
    use leptos::either::Either;
    move || {
        let uri = uri.clone();
        if let Some(remote) = remote.and_then(|f| f()) {
            Either::Left({
                #[cfg(feature = "hydrate")]
                {
                    use flams_router_base::ServerFnExt;
                    use ftml_viewer_components::components::documents::{
                        FragmentString, FragmentStringProps,
                    };
                    wait_and_then_fn(
                        move || {
                            flams_router_content::server_fns::Fragment {
                                uri: Some(URI::Narrative(uri.clone())),
                                rp: None,
                                a: None,
                                p: None,
                                l: None,
                                d: None,
                                e: None,
                                s: None,
                                m: None,
                            }
                            .call_remote(remote.clone())
                        },
                        move |(uri, css, html)| {
                            let uri = if let URI::Narrative(NarrativeURI::Element(uri)) = uri {
                                Some(uri)
                            } else {
                                None
                            };
                            view! {<div>{
                              for css in css { do_css(css); }
                              FragmentString(FragmentStringProps{html,uri})
                            }</div>}
                        },
                    )
                }
                #[cfg(not(feature = "hydrate"))]
                {
                    ""
                }
            })
        } else {
            Either::Right(view!(<Fragment uri=URIComponents::Uri(URI::Narrative(uri)) />))
        }
    }
}
