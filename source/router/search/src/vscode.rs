use std::fmt::Write;

use crate::components::SearchState;
use flams_backend_types::search::{QueryFilter, SearchResult, SearchResultKind};
use flams_router_base::maybe_lazy;
use flams_router_vscode::{
    VSCode,
    components::{VSCodeButton, VSCodeCheckbox, VSCodeRadio, VSCodeRadioGroup, VSCodeTextbox},
};
use flams_utils::{impossible, unwrap};
use flams_web_utils::components::wait_and_then_fn;
use ftml_components::components::content::{FtmlViewable, symbol_uri};
use ftml_dom::{FtmlViews, utils::css::inject_css};
use ftml_uris::{
    ArchiveId, DocumentElementUri, DocumentUri, IsDomainUri, IsNarrativeUri, NarrativeUri,
    SymbolUri, UriWithArchive, UriWithPath,
    components::{UriComponents, UriComponentsTrait},
};
use leptos::prelude::*;

maybe_lazy!(VSCSearch = vscode_search());

pub fn vscode_search() -> AnyView {
    use ftml_component_utils::toasts::ToasterProvider;
    // make sure this runs client side rather than server side because of hydration errors
    // I don't understand.
    let sig = RwSignal::new(false);
    Effect::new(move || {
        //sig.track();
        #[cfg(feature = "hydrate")]
        {
            sig.set(true);
        }
    });
    let inner = || {
        let remote = || leptos_router::hooks::use_query_map().with(|q| q.get("remote"));

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
            use flams_backend_types::search::{FragmentQueryFilter, QueryFilterFlags};

            let mut ret = FragmentQueryFilter::default();
            ret.flags = QueryFilterFlags::none();
            if full_docs.get() {
                ret.flags = ret.flags.set_allow_documents();
            }
            if paras.get() {
                ret.flags = ret.flags.set_allow_paragraphs();
            }
            if defs.get() {
                ret.flags = ret.flags.set_allow_definitions();
            }
            if exs.get() {
                ret.flags = ret.flags.set_allow_examples();
            }
            if asss.get() {
                ret.flags = ret.flags.set_allow_assertions();
            }
            if probs.get() {
                ret.flags = ret.flags.set_allow_problems();
            }
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
                #[cfg(all(feature = "hydrate", not(feature = "ssr")))]
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
                #[cfg(all(feature = "hydrate", not(feature = "ssr")))]
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

        inject_css("flams-search-block", include_str!("vscode.css"));
        view! {
            <ToasterProvider>
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
                    /*<Themer>*///{
                        //flams_router_content::Views::top(move || view!{
                            {do_results("Local Results",None,local_results)}
                            <div style="margin-top:25px;"></div>
                            {do_results("Remote Results",Some(remote),remote_results)}
                        // })
                    //}//</Themer>
                </div>
            </div>
            </ToasterProvider>
        }
        .into_any()
    };
    let inner = move || {
        flams_router_content::Views::top(move || {
            flams_router_content::Views::setup_document(
                DocumentUri::no_doc().clone(),
                ftml_components::SidebarPosition::None,
                false,
                ftml_dom::toc::TocSource::None,
                inner,
            )
        })
    };
    (move || if sig.get() { Some(inner()) } else { None }).into_any()
}

fn do_results(
    pre: &'static str,
    remote: Option<fn() -> Option<String>>,
    results: RwSignal<SearchState>,
) -> AnyView {
    use leptos::either::EitherOf6::*;
    let pre_view =
        move || view! {<div style="width:100%;font-weight:bold;text-align:center;">{pre}</div>};
    (move || {
        results.with(|r| match r {
            SearchState::None => A(()),
            SearchState::Results(v) if v.is_empty() => B(view!({pre_view}"(No results)")),
            SearchState::Loading => C(view!({pre_view}<ftml_component_utils::Spinner/>)),
            SearchState::SymResults(v) if remote.is_none() => D(view!({pre_view}{v
            .iter()
            .map(|(_,sym, elem)| do_sym_result_local(sym,elem))
            .collect_view()})),
            SearchState::SymResults(v) => E(view!({pre_view}{v
            .iter()
            .map(|(_,sym, elem)| do_sym_result_remote(sym, elem.clone(),unwrap!(remote)))
            .collect_view()})),
            SearchState::Results(v) => F(view!({pre_view}{v
            .iter()
            .map(|(score, res)| do_result(*score, res,remote))
            .collect_view()})),
        })
    })
    .into_any()
}

fn do_result(score: f32, res: &SearchResult, remote: Option<fn() -> Option<String>>) -> AnyView {
    use leptos::either::Either::*;
    match res {
        SearchResult::Document(d) => do_doc(score, d.clone(), remote),
        SearchResult::Paragraph {
            uri, fors, kind, ..
        } => do_para(score, uri.clone(), *kind, fors.clone(), remote),
    }
}

#[derive(leptos::server_fn::serde::Serialize, Debug, Clone)]
struct Usemodule {
    kind: &'static str,
    archive: ArchiveId,
    path: String,
}
impl Usemodule {
    fn make(uri: &SymbolUri) -> Self {
        let module = uri.module_uri();
        let archive = module.archive_id().clone();
        let path = if let Some(p) = module.path() {
            format!("{p}?{}", module.module_name().first())
        } else {
            module.module_name().first().to_string()
        };
        Self {
            kind: "usemodule",
            archive,
            path,
        }
    }
}

#[derive(leptos::server_fn::serde::Serialize, Debug, Clone)]
struct Preview<'u> {
    kind: &'static str,
    uri: &'u SymbolUri,
}
impl Preview<'_> {
    fn make(uri: &SymbolUri) -> Preview<'_> {
        Preview {
            kind: "preview",
            uri,
        }
    }
}

#[derive(Copy, Clone)]
struct Short<'u>(&'u SymbolUri);
impl std::fmt::Display for Short<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]{{", self.0.archive_id())?;
        if let Some(p) = self.0.path() {
            p.fmt(f)?;
            f.write_char('?')?;
        }
        write!(f, "{}}} {}", self.0.module_name(), self.0.name())
    }
}

fn do_sym_result_local(sym: &SymbolUri, elem: &DocumentElementUri) -> AnyView {
    let vs = unwrap!(VSCode::get());
    let name = symbol_uri(
        format!("{}?{}", sym.module.short_id_string(), sym.name()),
        sym,
    ); //ftml_viewer_components::components::omdoc::symbol_name(sym, &Short(sym).to_string());
    view! {
        <div class="flams-search-block">
            <div><b>{name}</b>
                {
                    let sym_a = sym.clone();
                    let vs_a = vs.clone();
                    let on_use = move |_| {
                        let _ = vs_a.post_message(Usemodule::make(&sym_a));
                    };
                    let sym = sym.clone();
                    let on_preview = move |_| {
                        let _ = vs.post_message(Preview::make(&sym));
                    };
                    view!{
                        <div style="width:100%">
                            <div style="margin-left:auto;width:fit-content;display:flex;flex-direction:row;">
                                <div style="width:fit-content;margin-right:5px;" on:click=on_preview>
                                    <VSCodeButton>"preview"</VSCodeButton>
                                </div>
                                <div style="width:fit-content;" on:click=on_use>
                                    <VSCodeButton>"\\usemodule"</VSCodeButton>
                                </div>
                            </div>
                        </div>
                    }
                }
            </div>
        </div>
    }.into_any()
}

fn do_sym_result_remote(
    sym: &SymbolUri,
    elem: DocumentElementUri,
    remote: fn() -> Option<String>,
) -> AnyView {
    use ftml_component_utils::Scrollbar;
    let name = sym.as_view(); //ftml_viewer_components::components::omdoc::symbol_name(sym, &sym.to_string());
    view! {
        <div class="flams-search-block">
            <div><b>{name}</b>
            </div>
            <div style="display:block">
            <div style="padding:0 5px;max-width:100%">
                <div style="width:100%;color:black;background-color:white;">
                  <Scrollbar style="max-height: 100px;width:100%;max-width:100%;">{
                      fragment(elem.into(),Some(remote))
                  }
                  </Scrollbar>
                </div>
              </div>
            </div>
        </div>
    }
    .into_any()
}

fn do_doc(score: f32, uri: DocumentUri, remote: Option<fn() -> Option<String>>) -> AnyView {
    use ftml_component_utils::Scrollbar;
    let name = uri.as_view(); //doc_name(&uri, uri.document_name().to_string());
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
    }.into_any()
}

fn do_para(
    score: f32,
    uri: DocumentElementUri,
    kind: SearchResultKind,
    fors: Vec<SymbolUri>,
    remote: Option<fn() -> Option<String>>,
) -> AnyView {
    use ftml_component_utils::Scrollbar;
    let uristr = uri.to_string();
    let name = uristr;
    /*let desc = ftml_components::components::content::CommaSep(
        "For",
        fors.into_iter()
            .map(|s| s.as_view::<flams_router_content::backend::FtmlBackend>()),
    );*/
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
    }.into_any()
}

fn fragment(uri: NarrativeUri, remote: Option<fn() -> Option<String>>) -> AnyView {
    use flams_router_content::components::Fragment;
    (move || {
        let uri = uri.clone();
        if let Some(remote) = remote.and_then(|f| f()) {
            {
                #[cfg(all(feature = "hydrate", not(feature = "ssr")))]
                {
                    use flams_router_base::ServerFnExt;
                    wait_and_then_fn(
                        move || {
                            flams_router_content::server_fns::Fragment {
                                uri: Some(uri.clone().into()),
                                rp: None,
                                a: None,
                                p: None,
                                l: None,
                                d: None,
                                e: None,
                                s: None,
                                m: None,
                                context: None,
                            }
                            .call_remote(remote.clone())
                        },
                        move |(uri, css, html)| {
                            use ftml_dom::utils::css::CssExt;
                            use ftml_uris::Uri;

                            let uri = if let Uri::DocumentElement(uri) = uri {
                                Some(uri)
                            } else {
                                None
                            };
                            view! {<div>{
                              for css in css { css.inject(); }
                              flams_router_content::Views::render_ftml(html.into_string(),None)
                              //FragmentString(FragmentStringProps{html,uri})
                            }</div>}.into_any()
                        },
                    )
                }
                #[cfg(not(feature = "hydrate"))]
                {
                    ""
                }
            }.into_any()
        } else {
            view!(<Fragment uri=UriComponents::Full(uri.into()) position=ftml_components::SidebarPosition::None/>).into_any()
        }
    }).into_any()
}
