#![allow(clippy::must_use_candidate)]

mod dashboard;
pub mod content;
pub mod settings;
pub mod backend;
pub mod query;
pub mod git;
pub mod index;
pub(crate) mod buildqueue;
pub(crate) mod logging;
pub(crate) mod users;

use dashboard::{Dashboard,MainPage};

use leptos::{either::Either, prelude::*};
use leptos_meta::{provide_meta_context, Title};
use leptos_router::{components::{ParentRoute, Redirect, Route, Router, Routes}, hooks::use_query_map, SsrMode, StaticSegment};

//#[derive(Copy,Clone,Debug,serde::Serialize,serde::Deserialize)]
//pub struct UseLSP(pub bool);



#[component]
pub fn Main() -> impl IntoView {
    provide_meta_context();
    #[cfg(feature = "ssr")]
    provide_context(flams_web_utils::CssIds::default());
    view! {
        <Title text="𝖥𝖫∀𝖬∫"/>
        <Router>{
            let params = use_query_map();
            let has_params = move || params.with(|p| p.get_str("a").is_some() || p.get_str("uri").is_some());
            //provide_context(UseLSP(params.with_untracked(|p|)))
            view!{<Routes fallback=|| NotFound()>
                <ParentRoute/* ssr=SsrMode::InOrder*/ path=() view=Top>
                    <ParentRoute path=StaticSegment("/dashboard") view=Dashboard>
                        <Route path=StaticSegment("mathhub") view=|| view!(<MainPage page=Page::MathHub/>)/>
                        //<Route path="graphs" view=|| view!(<MainPage page=Page::Graphs/>)/>
                        <Route path=StaticSegment("log") view=|| view!(<MainPage page=Page::Log/>)/>
                        <Route path=StaticSegment("queue") view=|| view!(<MainPage page=Page::Queue/>)/>
                        <Route path=StaticSegment("settings") view=|| view!(<MainPage page=Page::Settings/>)/>
                        <Route path=StaticSegment("query") view=|| view!(<MainPage page=Page::Query/>)/>
                        <Route path=StaticSegment("archives") view=|| view!(<MainPage page=Page::MyArchives/>)/>
                        <Route path=StaticSegment("users") view=|| view!(<MainPage page=Page::Users/>)/>
                        <Route path=StaticSegment("") view=|| view!(<MainPage page=Page::Home/>)/>
                        <Route path=StaticSegment("*any") view=|| view!(<MainPage page=Page::NotFound/>)/>
                    </ParentRoute>
                    <Route path=StaticSegment("/") view={move || if has_params() {
                            Either::Left(view! { <content::URITop/> })
                        } else {
                            Either::Right(view! { <Redirect path="/dashboard"/> })
                        }}
                    />
                </ParentRoute>
            </Routes>}
        }</Router>
    }
}

#[component(transparent)]
fn Top() -> impl IntoView {
    use crate::users::Login;
    //use flams_web_utils::components::Themer;
    //use ftml_viewer_components::FTMLGlobalSetup;
    //use crate::users::Login;
    view!{<Login><leptos_router::components::Outlet/></Login>}
}

#[derive(Copy,Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
enum Page {
    Home,
    MathHub,
    //Graphs,
    Log,
    NotFound,
    Queue,
    Settings,
    Login,
    Query,
    MyArchives,
    Users
}
impl Page {
    pub const fn key(self) -> &'static str {
        use Page::*;
        match self {
            Home => "home",
            MathHub => "mathhub",
            //Graphs => "graphs",
            Log => "log",
            Login => "login",
            Queue => "queue",
            Settings => "settings",
            Query => "query",
            MyArchives => "archives",
            Users => "users",
            NotFound => "notfound"
        }
    }
}
impl std::fmt::Display for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.key())
    }
}

#[component]
fn NotFound() -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        let resp = expect_context::<leptos_axum::ResponseOptions>();
        resp.set_status(http::StatusCode::NOT_FOUND);
    }

    view! {
        <h3>"Not Found"</h3>
    }
}