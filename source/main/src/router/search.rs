use flams_ontology::{search::{FragmentQueryOpts, QueryFilter, SearchResult, SearchResultKind}, uris::{ArchiveURITrait, DocumentElementURI, DocumentURI, SymbolURI, URI}};
use flams_web_utils::components::error_with_toaster;
use leptos::prelude::*;

use crate::router::content::uris::{DocURIComponents, URIComponents};


#[server(prefix="/api",endpoint="search")]
#[allow(clippy::unused_async)]
pub async fn search_query(query:String,opts:QueryFilter,num_results:usize) -> Result<Vec<(f32,SearchResult)>,ServerFnError<String>> {
  use flams_system::search::Searcher;
  tokio::task::spawn_blocking(move || {
    Searcher::get().query(&query, opts, num_results).ok_or_else(
      || ServerFnError::ServerError("Search error".to_string())
    )
  }).await.map_err(|e| ServerFnError::ServerError(e.to_string()))?
}

#[derive(Debug,Clone)]
enum SearchState {
  None,
  Loading,
  Results(bool,Vec<(f32,SearchResult)>)
}

#[derive(Copy,Clone,PartialEq,Eq)]
enum Filter {
  Doc,Def,Par,Ex,Ass
}
impl Filter {
  const ALL: [Filter;5] = [Filter::Doc,Filter::Def,Filter::Par,Filter::Ex,Filter::Ass];
  fn from_value(s:&str) -> Self {
    match s {
      "doc" => Self::Doc,
      "def" => Self::Def,
      "par" => Self::Par,
      "ex" => Self::Ex,
      "ass" => Self::Ass,
      _ => unreachable!()
    }
  }
  fn value_str(self) -> &'static str {
    match self {
      Self::Doc => "doc",
      Self::Def => "def",
      Self::Par => "par",
      Self::Ex => "ex",
      Self::Ass => "ass"
    }
  }
  fn tag_str(self) -> &'static str {
    match self {
      Self::Doc => "Documents",
      Self::Def => "Definitions",
      Self::Par => "Paragraphs",
      Self::Ex => "Examples",
      Self::Ass => "Assertions"
    }
  }
  fn long_str(self) -> &'static str {
    match self {
      Self::Doc => "Full Documents",
      Self::Def => "Definitions",
      Self::Par => "Other Paragraphs",
      Self::Ex => "(Counter-)examples",
      Self::Ass => "Assertions (Theorems, Lemmata, etc.)"
    }
  }
}

#[component]
pub fn SearchTop() -> impl IntoView {
  use thaw::{Layout,LayoutHeader,Flex,Input,InputPrefix,Icon,Divider,ToasterInjection,FlexAlign,Tag,TagPicker,TagPickerControl,TagPickerGroup,TagPickerInput,TagPickerOption};//,Combobox,ComboboxOption
  use flams_web_utils::components::ClientOnly;
  
  let query = RwSignal::new(String::new());
  let search_kind = RwSignal::new(vec![
    Filter::Def.value_str().to_string(),
    Filter::Par.value_str().to_string()
  ]);
  let query_opts = Memo::new(move |_| search_kind.with(|v| {
    let mut ret = FragmentQueryOpts::default();
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
        _ => ()
      }
    }
    ret
  }));
  let results = RwSignal::new(SearchState::None);
  let toaster = ToasterInjection::expect_context();
  let action = Action::new(move |args:&()| {
    results.set(SearchState::Loading);
    let s = query.get_untracked();
    let opts = query_opts.get_untracked();
    let symbols = false;
    async move {
      match search_query(s,QueryFilter::Fragments(opts),20).await {
        Ok(r) => results.set(SearchState::Results(symbols,r)),
        Err(e) => {
          results.set(SearchState::None);
          error_with_toaster(e,toaster);
        }
      }
    }
  });
  Effect::new(move || {
    if query.with(|q| q.is_empty()) { return };
    let opts = query_opts.get();
    action.dispatch(());
  });
  view!{
    <Layout>
      <LayoutHeader><Flex>
        <Input value=query placeholder="search...">
            <InputPrefix slot>
                <Icon icon=icondata_ai::AiSearchOutlined/>
            </InputPrefix>
        </Input>
        <ClientOnly>
          <TagPicker selected_options=search_kind>
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
          /*<Combobox selected_options=search_kind>
            <ComboboxOption value="doc" text="Full Documents"/>
            <ComboboxOption value="def" text="Definitions"/>
            <ComboboxOption value="ex" text="Examples"/>
            <ComboboxOption value="ass" text="Assertions (Theorems, Lemmata, etc.)"/>
            <ComboboxOption value="par" text="Other Paragraphs"/>
          </Combobox>*/
        </ClientOnly>
      </Flex></LayoutHeader>
      <Layout>
        <Divider/>
        <div style="width:fit-content;padding:10px;"><Flex vertical=true align=FlexAlign::Start>{move || do_results(results)}</Flex></div>
      </Layout>
    </Layout>
  }
}

fn do_results(results:RwSignal<SearchState>) -> impl IntoView {
  use leptos::either::EitherOf5::*;
  results.with(|r| match r {
    SearchState::None => A(()),
    SearchState::Results(_,v) if v.is_empty() => B("(No results)"),
    SearchState::Loading => C(view!(<flams_web_utils::components::Spinner/>)),
    SearchState::Results(true,v) => {
      D(view!{"TODO: "{format!("{v:?}")}})
    }
    SearchState::Results(symbols,v) => {
      E(v.iter().map(|(score,res)| do_result(*score,res)).collect_view())
    }
  })
}

fn do_result(score:f32,res:&SearchResult) -> impl IntoView {
  use thaw::{};
  use leptos::either::Either::*;
  match res {
    SearchResult::Document(d) => Left(do_doc(score,d.clone())),
    SearchResult::Paragraph { uri, fors,kind,.. } => 
      Right(do_para(score,uri.clone(),*kind,fors.clone()))
  }
  
}

fn do_doc(score:f32,uri:DocumentURI) -> impl IntoView {
  use thaw::{Card,CardHeader,Body1,CardHeaderDescription,CardHeaderAction,CardPreview,CardFooter,Scrollbar};
  use ftml_viewer_components::components::omdoc::doc_name;
  use super::content::DocumentInner;
  let name = doc_name(&uri,uri.name().to_string());
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

fn do_para(score:f32,uri:DocumentElementURI,kind:SearchResultKind,fors:Vec<SymbolURI>) -> impl IntoView {
  use thaw::{Card,CardHeader,Body1,Caption1,CardHeaderDescription,CardHeaderAction,CardPreview,CardFooter,Scrollbar};
  use ftml_viewer_components::components::omdoc::{symbol_name,doc_elem_name,comma_sep};
  use flams_web_utils::components::{Popover,PopoverTrigger};
  use super::content::Fragment;
  let uristr = uri.to_string();
  let namestr = uri.name().to_string();
  let name = view!{
    <div style="display:inline-block;"><Popover>
    <PopoverTrigger slot><span class="ftml-comp">{namestr}</span></PopoverTrigger>
    <div style="font-size:small;">{uristr}</div>
    </Popover></div>
  };
  let desc = comma_sep("For",fors.into_iter().map(|s| symbol_name(&s,s.name().last_name().as_ref())));
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