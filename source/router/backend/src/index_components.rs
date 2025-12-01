use flams_backend_types::archive_json::{ArchiveIndex, Institution};
use flams_web_utils::components::wait_and_then_fn;
use ftml_dom::utils::css::inject_css;
use ftml_uris::DocumentUri;
use leptos::prelude::*;
use thaw::{
    Body1, Caption1, Card, CardFooter, CardHeader, CardHeaderAction, CardHeaderDescription,
    CardPreview, Scrollbar,
};

#[component]
pub fn Index() -> AnyView {
    inject_css(
        "flams-index-card",
        ".flams-index-card{max-width:400px !important;margin:10px !important;}",
    );
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
        view! {
          {do_books(books)}
          {do_papers(papers)}
          {do_self_studies(self_studies)}
          {do_courses(courses,is)}
          {do_libraries(libraries)}
        }
        .into_any()
    })
}

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

fn wrap_list(ttl: &'static str, i: impl FnOnce() -> AnyView) -> AnyView {
    use thaw::Divider;
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
    view!(<div style="margin:5px;"><Scrollbar style="max-height: 100px;"><Body1>
    <ClientOnly><span inner_html=txt style="font-size:smaller;"/></ClientOnly>
  </Body1></Scrollbar></div>)
    .into_any()
}

fn do_books(books: Vec<ArchiveIndex>) -> AnyView {
    if books.is_empty() {
        return ().into_any();
    }
    client(move || {
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
    view! {<Card class="flams-index-card">
      <CardHeader>
        {link_doc(&file,|| view!(<Body1><b inner_html=title.to_string()/></Body1>).into_any())}
        <CardHeaderDescription slot><Caption1>
          {if authors.is_empty() {None} else {Some(IntoIterator::into_iter(authors).map(|a| view!{{a.to_string()}<br/>}).collect_view())}}
        </Caption1>
        </CardHeaderDescription>
      </CardHeader>
      <CardPreview>
        {thumbnail.map(|t| do_img(t.to_string()))}
        {teaser.map(|t| do_teaser(t.to_string()))}
      </CardPreview>
    </Card>}.into_any()
}

fn do_papers(papers: Vec<ArchiveIndex>) -> AnyView {
    if papers.is_empty() {
        return ().into_any();
    }
    client(move || {
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
    view! {<Card class="flams-index-card">
      <CardHeader>
        {link_doc(&file,|| view!(<Body1><b inner_html=title.to_string()/></Body1>).into_any())}
        <CardHeaderDescription slot><Caption1>
          {if authors.is_empty() {None} else {Some(IntoIterator::into_iter(authors).map(|a| view!{{a.to_string()}<br/>}).collect_view())}}
        </Caption1>
        </CardHeaderDescription>
        <CardHeaderAction slot>
        {venue.map(|v| {
          if let Some(url) = venue_url {
            leptos::either::Either::Left(view!(
              <a target="_blank" href=url.to_string() style="color:var(--colorBrandForeground1)">
                <b>{v.to_string()}</b>
              </a>
            ))
          } else {
            leptos::either::Either::Right(view!(<b>{v.to_string()}</b>))
          }
        })}
        </CardHeaderAction>
      </CardHeader>
      <CardPreview>
        {thumbnail.map(|t| do_img(t.to_string()))}
        {teaser.map(|t| do_teaser(t.to_string()))}
      </CardPreview>
    </Card>}.into_any()
}

fn do_self_studies(sss: Vec<ArchiveIndex>) -> AnyView {
    if sss.is_empty() {
        return ().into_any();
    }
    client(move || {
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
    view! {<Card class="flams-index-card">
      <CardHeader>
        {link_doc(&landing,|| view!(
          <Body1><b><span inner_html=title.to_string()/>{acronym.map(|s| format!(" ({s})"))}</b></Body1>
        ).into_any())}
      </CardHeader>
      <CardPreview>
        {thumbnail.map(|t| do_img(t.to_string()))}
        {teaser.map(|t| do_teaser(t.to_string()))}
      </CardPreview>
      <div style="margin-top:auto;"/>
      <CardFooter>
        <Caption1>
          {link_doc(&notes,|| "Notes".into_any())}
          {slides.map(|s| view!(", "{link_doc(&s,|| "Slides".into_any())}))}
        </Caption1>
      </CardFooter>
    </Card>}.into_any()
}

fn do_courses(courses: Vec<ArchiveIndex>, insts: Vec<Institution>) -> AnyView {
    if courses.is_empty() {
        return ().into_any();
    }
    client(move || {
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
    view! {<Card class="flams-index-card">
      <CardHeader>
        {link_doc(&landing,|| view!(
          <Body1><b><span inner_html=title.to_string()/>{acronym.map(|s| format!(" ({s})"))}</b></Body1>
        ).into_any())}
        <CardHeaderDescription slot><Caption1>
          {if instructors.is_empty() {None} else {Some(IntoIterator::into_iter(instructors).map(|a| view!{{a.to_string()}<br/>}).collect_view())}}
        </Caption1>
        </CardHeaderDescription>
        <CardHeaderAction slot>{
          {inst.map(|inst| view!(
            <img style="max-width:50px;max-height:30px;" src=inst.logo().to_string() title=inst.title().to_string()/>
          ))}
        }</CardHeaderAction>
      </CardHeader>
      <CardPreview>
        {thumbnail.map(|t| do_img(t.to_string()))}
        {teaser.map(|t| do_teaser(t.to_string()))}
      </CardPreview>
      <div style="margin-top:auto;"/>
      <CardFooter>
        <Caption1>
          {link_doc(&notes,|| "Notes".into_any())}
          {slides.map(|s| view!(", "{link_doc(&s,|| "Slides".into_any())}))}
        </Caption1>
      </CardFooter>
    </Card>}.into_any()
}

fn do_libraries(libs: Vec<ArchiveIndex>) -> AnyView {
    if libs.is_empty() {
        return ().into_any();
    }
    client(move || {
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
    view! {<Card class="flams-index-card">
      <CardHeader>
        <Body1><b inner_html=title.to_string()/></Body1>
        <CardHeaderDescription slot><Caption1>
          {archive.to_string()}
        </Caption1></CardHeaderDescription>
        /*{link_doc(&landing,|| view!(
          <Body1><b><span inner_html=title.to_string()/>{acronym.map(|s| format!(" ({s})"))}</b></Body1>
        ))}*/
      </CardHeader>
      <CardPreview>
        {thumbnail.map(|t| do_img(t.to_string()))}
        {teaser.map(|t| do_teaser(t.to_string()))}
      </CardPreview>
      <div style="margin-top:auto;"/>
    </Card>}
    .into_any()
}
