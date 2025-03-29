use flams_ontology::{
    search::{QueryFilter, SearchResult, SearchResultKind},
    uris::{DocumentElementURI, DocumentURI, SymbolURI, URI},
};
use flams_router_base::uris::{DocURIComponents, URIComponents};
use flams_utils::{impossible, vecmap::VecMap};
use flams_web_utils::{components::error_with_toaster, inject_css};
use leptos::prelude::*;

#[derive(Debug, Clone)]
pub(crate) enum SearchState {
    None,
    Loading,
    Results(Vec<(f32, SearchResult)>),
    SymResults(VecMap<SymbolURI, Vec<(f32, SearchResult)>>),
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
    const ALL: [Filter; 5] = [
        Filter::Doc,
        Filter::Def,
        Filter::Par,
        Filter::Ex,
        Filter::Ass,
    ];
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
    fn value_str(self) -> &'static str {
        match self {
            Self::Doc => "doc",
            Self::Def => "def",
            Self::Par => "par",
            Self::Ex => "ex",
            Self::Ass => "ass",
        }
    }
    fn tag_str(self) -> &'static str {
        match self {
            Self::Doc => "Documents",
            Self::Def => "Definitions",
            Self::Par => "Paragraphs",
            Self::Ex => "Examples",
            Self::Ass => "Assertions",
        }
    }
    fn long_str(self) -> &'static str {
        match self {
            Self::Doc => "Full Documents",
            Self::Def => "Definitions",
            Self::Par => "Other Paragraphs",
            Self::Ex => "(Counter-)examples",
            Self::Ass => "Assertions (Theorems, Lemmata, etc.)",
        }
    }
}

#[component]
pub fn SearchTop() -> impl IntoView {
    use flams_web_utils::components::ClientOnly;
    use thaw::{
        Divider, Flex, FlexAlign, Icon, Input, InputPrefix, Layout, LayoutHeader, Radio,
        RadioGroup, Tag, TagPicker, TagPickerControl, TagPickerGroup, TagPickerInput,
        TagPickerOption, ToasterInjection,
    }; //,Combobox,ComboboxOption
    let query = RwSignal::new(String::new());
    let search_kind = RwSignal::new(vec![
        Filter::Def.value_str().to_string(),
        Filter::Par.value_str().to_string(),
    ]);
    let query_opts = Memo::new(move |_| {
        search_kind.with(|v| {
            let mut ret = QueryFilter::default();
            ret.allow_documents = false;
            ret.allow_paragraphs = false;
            ret.allow_definitions = false;
            ret.allow_examples = false;
            ret.allow_assertions = false;
            ret.allow_exercises = false;
            for s in v {
                match Filter::from_value(s.as_str()) {
                    Filter::Doc => ret.allow_documents = true,
                    Filter::Def => ret.allow_definitions = true,
                    Filter::Par => ret.allow_paragraphs = true,
                    Filter::Ex => ret.allow_examples = true,
                    Filter::Ass => ret.allow_assertions = true,
                }
            }
            ret
        })
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
                  <Icon icon=icondata_ai::AiSearchOutlined/>
              </InputPrefix>
          </Input>
          <RadioGroup value=radio_value>
            <Radio value="S" label="Symbols"/>
            <Radio value="X" label="Documents/Paragraphs"/>
          </RadioGroup>
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
    }
}

fn do_results(results: RwSignal<SearchState>) -> impl IntoView {
    use leptos::either::EitherOf5::*;
    results.with(|r| match r {
        SearchState::None => A(()),
        SearchState::Results(v) if v.is_empty() => B("(No results)"),
        SearchState::Loading => C(view!(<flams_web_utils::components::Spinner/>)),
        SearchState::SymResults(v) => D(v
            .iter()
            .map(|(sym, res)| do_sym_result(sym, res.clone()))
            .collect_view()),
        SearchState::Results(v) => E(v
            .iter()
            .map(|(score, res)| do_result(*score, res))
            .collect_view()),
    })
}

fn do_sym_result(sym: &SymbolURI, res: Vec<(f32, SearchResult)>) -> impl IntoView + use<> {
    use flams_router_content::components::Fragment;
    use thaw::{Body1, Card, CardHeader, CardPreview, Scrollbar};

    let name = ftml_viewer_components::components::omdoc::symbol_name(sym, &sym.to_string());
    view! {
      <Card>
          <CardHeader>
              <Body1><b>{name}</b></Body1>
          </CardHeader>
          <CardPreview>
            <div style="padding:0 5px;max-width:100%">
              <div style="width:100%;color:black;background-color:white;">
                <Scrollbar style="max-height: 100px;width:100%;max-width:100%;">{
                  res.into_iter().map(|(_,r)| {
                    let SearchResult::Paragraph { uri, .. } = r else { impossible!()};
                    view!(<Fragment uri=URIComponents::Uri(URI::Narrative(uri.into())) />)
                  }).collect_view()
                }
                </Scrollbar>
              </div>
            </div>
          </CardPreview>
      </Card>
    }
}

fn do_result(score: f32, res: &SearchResult) -> impl IntoView + use<> {
    use leptos::either::Either::*;
    match res {
        SearchResult::Document(d) => Left(do_doc(score, d.clone())),
        SearchResult::Paragraph {
            uri, fors, kind, ..
        } => Right(do_para(score, uri.clone(), *kind, fors.clone())),
    }
}

fn do_doc(score: f32, uri: DocumentURI) -> impl IntoView {
    use flams_router_content::components::DocumentInner;
    use ftml_viewer_components::components::omdoc::doc_name;
    use thaw::{Body1, Card, CardHeader, CardHeaderAction, CardPreview, Scrollbar};
    let name = doc_name(&uri, uri.name().to_string());
    view! {
      <Card>
          <CardHeader>
              <Body1>
                  <b>"Document "{name}</b>
              </Body1>
              /*<CardHeaderDescription slot>
                  <Caption1>"Description"</Caption1>
              </CardHeaderDescription>*/
              <CardHeaderAction slot>
                  <span>"Score: "{score}</span>
              </CardHeaderAction>
          </CardHeader>
          <CardPreview>
              <div style="padding:0 5px;max-width:100%">
                <div style="width:100%;color:black;background-color:white;">
                    <Scrollbar style="max-height: 100px;;width:100%;max-width:100%;"><DocumentInner doc=DocURIComponents::Uri(uri) /></Scrollbar>
                </div>
              </div>
          </CardPreview>
          /*<CardFooter>
              "sTeX:"<pre></pre>
          </CardFooter>*/
      </Card>
    }
}

fn do_para(
    score: f32,
    uri: DocumentElementURI,
    kind: SearchResultKind,
    fors: Vec<SymbolURI>,
) -> impl IntoView {
    use flams_router_content::components::Fragment;
    use flams_web_utils::components::{Popover, PopoverTrigger};
    use ftml_viewer_components::components::omdoc::{comma_sep, symbol_name};
    use thaw::{
        Body1, Caption1, Card, CardHeader, CardHeaderAction, CardHeaderDescription, CardPreview,
        Scrollbar,
    };
    let uristr = uri.to_string();
    let namestr = uri.name().to_string();
    let name = view! {
      <div style="display:inline-block;"><Popover>
      <PopoverTrigger slot><span class="ftml-comp">{namestr}</span></PopoverTrigger>
      <div style="font-size:small;">{uristr}</div>
      </Popover></div>
    };
    let desc = comma_sep(
        "For",
        fors.into_iter()
            .map(|s| symbol_name(&s, s.name().last_name().as_ref())),
    );
    view! {
      <Card>
          <CardHeader>
              <Body1>
                  <b>{kind.as_str()}" "{name}</b>
              </Body1>
              <CardHeaderDescription slot>
                  <Caption1>{desc}</Caption1>
              </CardHeaderDescription>
              <CardHeaderAction slot>
                  <span>"Score: "{score}</span>
              </CardHeaderAction>
          </CardHeader>
          <CardPreview>
            <div style="padding:0 5px;max-width:100%">
              <div style="width:100%;color:black;background-color:white;">
                <Scrollbar style="max-height: 100px;width:100%;max-width:100%;"><Fragment uri=URIComponents::Uri(URI::Narrative(uri.into())) /></Scrollbar>
              </div>
            </div>
          </CardPreview>
          /*<CardFooter>
              "sTeX:"<pre></pre>
          </CardFooter>*/
      </Card>
    }
}
