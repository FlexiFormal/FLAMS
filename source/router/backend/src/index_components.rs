use flams_backend_types::archive_json::{ArchiveIndex, Institution};
use flams_router_base::maybe_lazy;
use flams_web_utils::{client_only, components::wait_and_then_fn};
use ftml_component_utils::{
    Block, Caption, Footer, Header, HeaderLeft, HeaderRight, Scrollbar, Text,
};
use ftml_dom::utils::css::inject_css;
use ftml_uris::DocumentUri;
use leptos::prelude::*;

maybe_lazy!(Index = index());

//#[component]
pub fn index() -> AnyView {
    wait_and_then_fn(super::server_fns::index, |(is, idxs)| {
        let mut libraries = Vec::new();
        let mut books = Vec::new();
        let mut papers = Vec::new();
        let mut courses = Vec::new();
        let mut self_studies = Vec::new();
        for e in idxs {
            match e {
                e @ ArchiveIndex::Library { .. } => libraries.push(e),
                e @ ArchiveIndex::Book { .. } => books.push(e),
                e @ ArchiveIndex::Paper { .. } => papers.push(e),
                e @ ArchiveIndex::Course { .. } => courses.push(e),
                e @ ArchiveIndex::SelfStudy { .. } => self_studies.push(e),
            }
        }
        //leptos::logging::log!("Here: main");
        let r = view! {
          {do_books(books)}
          {do_papers(papers)}
          {do_self_studies(self_studies)}
          {do_courses(courses,is)}
          {do_libraries(libraries)}
        }
        .into_any();
        inject_css(
            "flams-index-card",
            ".flams-index-card{max-width:400px;margin:10px;}",
        );
        r
    })
}
/*
fn client<V: IntoView + Send>(f: impl Fn() -> V + Send + 'static) -> impl IntoView {
    let sig = RwSignal::new(false);
    #[cfg(feature = "hydrate")]
    let _ = Effect::new(move || sig.set(true));
    move || {
        if sig.get() {
            f().into_any()
        } else {
            ().into_any()
        }
    }
}
 */

fn wrap_list(ttl: &'static str, i: impl FnOnce() -> AnyView) -> AnyView {
    use ftml_component_utils::Divider;
    view! {
      <h2 style="color:var(--colorBrandForeground1)">{ttl}</h2>
      <div style="display:flex;flex-flow:wrap;">
      {i()}
      </div>
      <Divider/>
    }
    .into_any()
}

fn link_doc<T: FnOnce() -> AnyView>(uri: &DocumentUri, i: T) -> AnyView {
    view! {
      <a target="_blank" href=format!("/?uri={}",urlencoding::encode(&uri.to_string())) style="color:var(--colorBrandForeground1)">
        {i()}
      </a>
    }.into_any()
}

fn do_img(url: String) -> AnyView {
    view!(<div style="width:100%"><div style="width:min-content;margin:auto;">
    <img src=url style="max-width:350px;max-height:150px;"/>
  </div></div>)
    .into_any()
}

fn do_teaser(txt: String) -> AnyView {
    use flams_web_utils::components::ClientOnly;
    view!(<div style="margin:5px;"><Scrollbar style="max-height: 100px;"><Text>
    <ClientOnly><span inner_html=txt style="font-size:smaller;"/></ClientOnly>
  </Text></Scrollbar></div>)
    .into_any()
}

fn do_books(books: Vec<ArchiveIndex>) -> AnyView {
    if books.is_empty() {
        return ().into_any();
    }
    client_only!({
        wrap_list("Books", || {
            books
                .clone()
                .into_iter()
                .map(book)
                .collect_view()
                .into_any()
        })
    })
    .into_any()
}

fn book(book: ArchiveIndex) -> AnyView {
    let ArchiveIndex::Book {
        title,
        authors,
        file,
        teaser,
        thumbnail,
    } = book
    else {
        unreachable!()
    };
    view! {<Block class="flams-index-card">
      <Header slot>
        {link_doc(&file,|| view!(<Text bold=true><span inner_html=title.to_string()/></Text>).into_any())}
      </Header>
      <HeaderLeft slot><Caption>
        {if authors.is_empty() {None} else {Some(IntoIterator::into_iter(authors).map(|a| view!{{a.to_string()}<br/>}).collect_view())}}
      </Caption>
      </HeaderLeft>
      <div style="margin: 0 -12px;">
        {thumbnail.map(|t| do_img(t.to_string()))}
        {teaser.map(|t| do_teaser(t.to_string()))}
      </div>
    </Block>}.into_any()
}

fn do_papers(papers: Vec<ArchiveIndex>) -> AnyView {
    if papers.is_empty() {
        return ().into_any();
    }
    client_only!({
        wrap_list("Papers", || {
            papers
                .clone()
                .into_iter()
                .map(paper)
                .collect_view()
                .into_any()
        })
    })
    .into_any()
}

fn paper(paper: ArchiveIndex) -> AnyView {
    let ArchiveIndex::Paper {
        title,
        authors,
        file,
        teaser,
        thumbnail,
        venue,
        venue_url,
    } = paper
    else {
        unreachable!()
    };
    view! {<Block class="flams-index-card">
      <Header slot>
        {link_doc(&file,|| view!(<Text bold=true><span inner_html=title.to_string()/></Text>).into_any())}
      </Header>
      <HeaderLeft slot><Caption>
        {if authors.is_empty() {None} else {Some(IntoIterator::into_iter(authors).map(|a| view!{{a.to_string()}<br/>}).collect_view())}}
      </Caption>
      </HeaderLeft>
      <HeaderRight slot>
      {venue.map(|v| venue_url.map_or_else(|| leptos::either::Either::Right(view!(<b>{v.to_string()}</b>)),
          |url| {
            leptos::either::Either::Left(view!(
              <a target="_blank" href=url.to_string() style="color:var(--colorBrandForeground1)">
                <b>{v.to_string()}</b>
              </a>
            ))
          }
      ))}
      </HeaderRight>
      <div style="margin: 0 -12px;">
        {thumbnail.map(|t| do_img(t.to_string()))}
        {teaser.map(|t| do_teaser(t.to_string()))}
      </div>
    </Block>}.into_any()
}

fn do_self_studies(sss: Vec<ArchiveIndex>) -> AnyView {
    if sss.is_empty() {
        return ().into_any();
    }
    client_only!({
        wrap_list("Self-Study Courses", || {
            sss.clone()
                .into_iter()
                .map(self_study)
                .collect_view()
                .into_any()
        })
    })
    .into_any()
}

fn self_study(ss: ArchiveIndex) -> AnyView {
    let ArchiveIndex::SelfStudy {
        title,
        landing,
        acronym,
        notes,
        slides,
        thumbnail,
        teaser,
        ..
    } = ss
    else {
        unreachable!()
    };
    view! {<Block class="flams-index-card">
      <Header slot>
        {link_doc(&landing,|| view!(
          <Text bold=true><span inner_html=title.to_string()/>{acronym.map(|s| format!(" ({s})"))}</Text>
        ).into_any())}
      </Header>
      <div style="margin: 0 -12px;">
        {thumbnail.map(|t| do_img(t.to_string()))}
        {teaser.map(|t| do_teaser(t.to_string()))}
      </div>
      <div style="margin-top:auto;"/>
      <Footer slot>
        <Caption>
          {link_doc(&notes,|| "Notes".into_any())}
          {slides.map(|s| view!(", "{link_doc(&s,|| "Slides".into_any())}))}
        </Caption>
      </Footer>
    </Block>}.into_any()
}

fn do_courses(courses: Vec<ArchiveIndex>, insts: Vec<Institution>) -> AnyView {
    if courses.is_empty() {
        return ().into_any();
    }
    client_only!({
        wrap_list("Courses", || {
            courses
                .clone()
                .into_iter()
                .map(|c| course(c, &insts))
                .collect_view()
                .into_any()
        })
    })
    .into_any()
}

fn course(course: ArchiveIndex, insts: &[Institution]) -> AnyView {
    let ArchiveIndex::Course {
        title,
        landing,
        acronym,
        authors: instructors,
        institution,
        notes,
        slides,
        thumbnail,
        teaser,
        //quizzes,
        //homeworks,
        //instances,
        ..
    } = course
    else {
        unreachable!()
    };
    let inst = institution
        .and_then(|inst| insts.iter().find(|i| i.acronym() == &*inst))
        .cloned();
    view! {<Block class="flams-index-card">
      <Header slot>
        {link_doc(&landing,|| view!(
          <Text bold=true><span inner_html=title.to_string()/>{acronym.map(|s| format!(" ({s})"))}</Text>
        ).into_any())}
      </Header>
      <HeaderLeft slot><Caption>
        {if instructors.is_empty() {None} else {Some(IntoIterator::into_iter(instructors).map(|a| view!{{a.to_string()}<br/>}).collect_view())}}
      </Caption>
      </HeaderLeft>
      <HeaderRight slot>{
        {inst.map(|inst| view!(
          <img style="max-width:50px;max-height:30px;" src=inst.logo().to_string() title=inst.title().to_string()/>
        ))}
      }</HeaderRight>
      <div style="margin: 0 -12px;">
        {thumbnail.map(|t| do_img(t.to_string()))}
        {teaser.map(|t| do_teaser(t.to_string()))}
      </div>
      <div style="margin-top:auto;"/>
      <Footer slot>
        <Caption>
          {link_doc(&notes,|| "Notes".into_any())}
          {slides.map(|s| view!(", "{link_doc(&s,|| "Slides".into_any())}))}
        </Caption>
      </Footer>
    </Block>}.into_any()
}

fn do_libraries(libs: Vec<ArchiveIndex>) -> AnyView {
    if libs.is_empty() {
        return ().into_any();
    }
    client_only!({
        wrap_list("Libraries", || {
            libs.clone()
                .into_iter()
                .map(library)
                .collect_view()
                .into_any()
        })
    })
    .into_any()
}

fn library(lib: ArchiveIndex) -> AnyView {
    let ArchiveIndex::Library {
        archive,
        title,
        teaser,
        thumbnail,
    } = lib
    else {
        unreachable!()
    };
    view! {<Block class="flams-index-card">
      <Header slot>
        <Text bold=true><span inner_html=title.to_string()/></Text>
        /*{link_doc(&landing,|| view!(
          <BodyText><b><span inner_html=title.to_string()/>{acronym.map(|s| format!(" ({s})"))}</b></BodyText>
        ))}*/
      </Header>
      <HeaderLeft slot><Caption>
        {archive.to_string()}
      </Caption></HeaderLeft>
      <div style="margin: 0 -12px;">
        {thumbnail.map(|t| do_img(t.to_string()))}
        {teaser.map(|t| do_teaser(t.to_string()))}
      </div>
      <div style="margin-top:auto;"/>
    </Block>}
    .into_any()
}
