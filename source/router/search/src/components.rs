use flams_backend_types::search::{SearchResult, SearchResultKind};
use flams_router_base::maybe_lazy;
use flams_utils::{impossible, vecmap::VecMap};
use flams_web_utils::components::error_with_toaster;
use ftml_components::components::content::{FtmlViewable, symbol_uri};
use ftml_dom::utils::css::inject_css;
use ftml_uris::{
    DocumentElementUri, DocumentUri, IsNarrativeUri, SymbolUri,
    components::{DocumentUriComponents, UriComponents},
};
use leptos::prelude::*;

#[derive(Debug, Clone)]
pub(crate) enum SearchState {
    None,
    Loading,
    Results(Vec<(f32, SearchResult)>),
    SymResults(Vec<(f32, SymbolUri, DocumentElementUri)>),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Filter {
    Doc,
    Def,
    Par,
    Ex,
    Ass,
}
impl Filter {
    const ALL: [Self; 5] = [Self::Doc, Self::Def, Self::Par, Self::Ex, Self::Ass];
    fn from_value(s: &str) -> Self {
        match s {
            "doc" => Self::Doc,
            "def" => Self::Def,
            "par" => Self::Par,
            "ex" => Self::Ex,
            "ass" => Self::Ass,
            _ => impossible!(),
        }
    }
    const fn value_str(self) -> &'static str {
        match self {
            Self::Doc => "doc",
            Self::Def => "def",
            Self::Par => "par",
            Self::Ex => "ex",
            Self::Ass => "ass",
        }
    }
    const fn tag_str(self) -> &'static str {
        match self {
            Self::Doc => "Documents",
            Self::Def => "Definitions",
            Self::Par => "Paragraphs",
            Self::Ex => "Examples",
            Self::Ass => "Assertions",
        }
    }
    const fn long_str(self) -> &'static str {
        match self {
            Self::Doc => "Full Documents",
            Self::Def => "Definitions",
            Self::Par => "Other Paragraphs",
            Self::Ex => "(Counter-)examples",
            Self::Ass => "Assertions (Theorems, Lemmata, etc.)",
        }
    }
}
maybe_lazy!(SearchTop = search_top());

pub fn search_top() -> AnyView {
    use flams_web_utils::components::ClientOnly;
    use ftml_component_utils::{
        Divider, Flex, FlexAlign, Input, InputPrefix, Layout, LayoutHeader, Radio, RadioGroup, Tag,
        TagPicker, TagPickerControl, TagPickerGroup, TagPickerInput, TagPickerOption, Text,
        toasts::ToasterInjection,
    }; //,Combobox,ComboboxOption
    let query = RwSignal::new(String::new());
    let in_doc_str = RwSignal::new(String::new());
    let search_kind = RwSignal::new(vec![
        Filter::Def.value_str().to_string(),
        Filter::Par.value_str().to_string(),
    ]);
    let query_opts = Memo::new(move |_| {
        search_kind.with(|v| {
            use flams_backend_types::search::FragmentQueryFilter;

            let mut ret = FragmentQueryFilter::default();
            ret.flags = ret.flags.unset_allow_documents();
            ret.flags = ret.flags.unset_allow_paragraphs();
            ret.flags = ret.flags.unset_allow_definitions();
            ret.flags = ret.flags.unset_allow_examples();
            ret.flags = ret.flags.unset_allow_assertions();
            ret.flags = ret.flags.unset_allow_problems();
            for s in v {
                match Filter::from_value(s.as_str()) {
                    Filter::Doc => ret.flags = ret.flags.set_allow_documents(),
                    Filter::Def => ret.flags = ret.flags.set_allow_definitions(),
                    Filter::Par => ret.flags = ret.flags.set_allow_paragraphs(),
                    Filter::Ex => ret.flags = ret.flags.set_allow_examples(),
                    Filter::Ass => ret.flags = ret.flags.set_allow_assertions(),
                }
            }
            in_doc_str.with(|s| {
                if let Ok(uri) = s.parse() {
                    ret.in_documents.push(uri);
                }
            });
            ret
        })
    });
    let color = Memo::new(move |_| {
        use std::str::FromStr;
        if in_doc_str.with(|s| s.is_empty() || DocumentUri::from_str(s).is_ok()) {
            "background-color:green"
        } else {
            "background-color:red"
        }
    });
    let results = RwSignal::new(SearchState::None);
    let toaster = ToasterInjection::expect_context();
    let action = Action::new(move |&()| {
        results.set(SearchState::Loading);
        let s = query.get_untracked();
        let opts = query_opts.get_untracked();
        async move {
            match super::search_query(s, opts, 20).await {
                Ok(r) => results.set(SearchState::Results(r)),
                Err(e) => {
                    results.set(SearchState::None);
                    error_with_toaster(e, toaster);
                }
            }
        }
    });
    let sym_action = Action::new(move |&()| {
        results.set(SearchState::Loading);
        let s = query.get_untracked();
        async move {
            match super::search_symbols(s, 20).await {
                Ok(r) => results.set(SearchState::SymResults(r)),
                Err(e) => {
                    results.set(SearchState::None);
                    error_with_toaster(e, toaster);
                }
            }
        }
    });
    let radio_value = RwSignal::new("X".to_string());
    Effect::new(move || {
        if query.with(|q| q.is_empty()) {
            return;
        };
        if radio_value.with(|s| s == "S") {
            sym_action.dispatch(());
        } else {
            let _ = query_opts.get(); // register dependency
            action.dispatch(());
        }
    });
    inject_css(
        "flams-search-picker",
        ".flams-search-picker{} .flams-search-picker-disabled { display:none; }",
    );
    let cls = Memo::new(move |_| match radio_value.get().as_str() {
        "X" => "flams-search-picker".to_string(),
        "S" => "flams-search-picker-disabled".to_string(),
        _ => impossible!(),
    });
    view! {
      <Layout>
        <LayoutHeader><Flex>
          <Input value=query placeholder="search...">
              <InputPrefix slot>
                  <ftml_component_utils::icons::SearchIcon/>
              </InputPrefix>
          </Input>
          <RadioGroup value=radio_value>
            <Radio value="S" label="Symbols"/>
            <Radio value="X" label="Documents/Paragraphs"/>
          </RadioGroup>
          <div>
          <Text>"In document: "</Text>
          <Input value=in_doc_str attr:style=color/>

          </div>
          <ClientOnly>
            <TagPicker selected_options=search_kind class=cls>
                <TagPickerControl slot>
                <TagPickerGroup>
                  {move ||
                    search_kind.get().into_iter().map(|option| view!{
                      <Tag value=option.clone() attr:style="background-color:var(--colorBrandBackground2)">
                          {Filter::from_value(option.as_str()).tag_str()}
                      </Tag>
                    }).collect_view()
                  }
                  </TagPickerGroup>
                  <TagPickerInput />
                </TagPickerControl>
                {
                  move ||
                      search_kind.with(|opts| {
                          Filter::ALL.iter().filter_map(|option| {
                              if opts.iter().any(|o| o == option.value_str()) {
                                  return None
                              } else {
                                  Some(view! {
                                      <TagPickerOption value=option.value_str().to_string() text=option.long_str() />
                                  })
                              }
                          }).collect_view()
                      })
                }
            </TagPicker>
          </ClientOnly>
        </Flex></LayoutHeader>
        <Layout>
          <Divider/>
          <div style="width:fit-content;padding:10px;"><Flex vertical=true align=FlexAlign::Start>{move || do_results(results)}</Flex></div>
        </Layout>
      </Layout>
    }.into_any()
}

fn do_results(results: RwSignal<SearchState>) -> AnyView {
    results.with(|r| match r {
        SearchState::None => ().into_any(),
        SearchState::Results(v) if v.is_empty() => "(No results)".into_any(),
        SearchState::Loading => view!(<flams_web_utils::components::Spinner/>).into_any(),
        SearchState::SymResults(v) => v
            .iter()
            .map(|(score, sym, elem)| do_sym_result(sym, *score, elem))
            .collect_view()
            .into_any(),
        SearchState::Results(v) => v
            .iter()
            .map(|(score, res)| do_result(*score, res))
            .collect_view()
            .into_any(),
    })
}

fn do_sym_result(sym: &SymbolUri, score: f32, elem: &DocumentElementUri) -> AnyView {
    use flams_router_content::components::Fragment;
    use flams_web_utils::components::ClientOnly;
    use ftml_component_utils::{BodyText, Card, CardHeader, CardPreview, Scrollbar};
    use ftml_uris::Uri;

    let name = symbol_uri(
        format!("{}?{}", sym.module.short_id_string(), sym.name()),
        sym,
    ); // ftml_viewer_components::components::omdoc::symbol_name(sym, &sym.to_string());
    let elem = elem.clone();
    view! {
      <Card>
          <CardHeader>
              <BodyText><b>{name}</b></BodyText>
          </CardHeader>
          <CardPreview>
            <div style="padding:0 5px;max-width:100%">
              <div style="width:100%;color:black;background-color:white;">
                <Scrollbar style="max-height: 100px;width:100%;max-width:100%;">
          <Fragment uri=UriComponents::Full(Uri::DocumentElement(elem)) position=ftml_components::SidebarPosition::None/>
                </Scrollbar>
              </div>
            </div>
          </CardPreview>
      </Card>
    }.into_any()
}

fn do_result(score: f32, res: &SearchResult) -> AnyView {
    use leptos::either::Either::*;
    match res {
        SearchResult::Document(d) => do_doc(score, d.clone()),
        SearchResult::Paragraph {
            uri, fors, kind, ..
        } => do_para(score, uri.clone(), *kind, fors.clone()),
    }
}

fn do_doc(score: f32, uri: DocumentUri) -> AnyView {
    use flams_router_content::components::DocumentInner;
    use ftml_component_utils::{
        BodyText, Card, CardHeader, CardHeaderAction, CardPreview, Scrollbar,
    };

    let name = uri.as_view(); //doc_name(&uri, uri.document_name().to_string());
    view! {
      <Card>
          <CardHeader>
              <BodyText>
                  <b>"Document "{name}</b>
              </BodyText>
              <CardHeaderAction slot>
                  <span>"Score: "{score}</span>
              </CardHeaderAction>
          </CardHeader>
          <CardPreview>
              <div style="padding:0 5px;max-width:100%">
                <div style="width:100%;color:black;background-color:white;">
                    <Scrollbar style="max-height: 100px;;width:100%;max-width:100%;"><DocumentInner doc=DocumentUriComponents::Full(uri) /></Scrollbar>
                </div>
              </div>
          </CardPreview>
      </Card>
    }.into_any()
}

fn do_para(
    score: f32,
    uri: DocumentElementUri,
    kind: SearchResultKind,
    fors: Vec<SymbolUri>,
) -> AnyView {
    use flams_router_content::components::Fragment;
    use flams_web_utils::components::{Popover, PopoverTrigger};
    use ftml_component_utils::{
        BodyText, Caption, Card, CardHeader, CardHeaderAction, CardHeaderDescription, CardPreview,
        Scrollbar,
    };
    let uristr = uri.to_string();
    let namestr = uri.name().to_string();
    let name = view! {
      <div style="display:inline-block;"><Popover>
      <PopoverTrigger slot>{view!(<span class="ftml-comp">{namestr}</span>).into_any()}</PopoverTrigger>
      <div style="font-size:small;">{uristr}</div>
      </Popover></div>
    };

    let desc = ftml_components::components::content::CommaSep(
        "For",
        fors.into_iter().map(|s| s.as_view()),
    )
    .into_view();
    view! {
      <Card>
          <CardHeader>
              <BodyText>
                  <b>{kind.as_str()}" "{name}</b>
              </BodyText>
              <CardHeaderDescription slot>
                  <Caption>{desc}</Caption>
              </CardHeaderDescription>
              <CardHeaderAction slot>
                  <span>"Score: "{score}</span>
              </CardHeaderAction>
          </CardHeader>
          <CardPreview>
            <div style="padding:0 5px;max-width:100%">
              <div style="width:100%;color:black;background-color:white;">
                <Scrollbar style="max-height: 100px;width:100%;max-width:100%;"><Fragment uri=UriComponents::Full(uri.into()) position=ftml_components::SidebarPosition::None /></Scrollbar>
              </div>
            </div>
          </CardPreview>
          /*<CardFooter>
              "sTeX:"<pre></pre>
          </CardFooter>*/
      </Card>
    }.into_any()
}
