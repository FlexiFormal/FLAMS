#![allow(clippy::must_use_candidate)]

mod dashboard;
pub(crate) mod content;
mod settings;
mod backend;
mod query;
pub(crate) mod buildqueue;
pub(crate) mod logging;

use dashboard::{Dashboard,MainPage};

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::{components::{ParentRoute, Redirect, Route, Router, Routes}, hooks::use_query_map, SsrMode, StaticSegment};

#[component]
pub fn Main() -> impl IntoView {
    provide_meta_context();
    view! {
        <Title text="iMᴍᴛ"/>
        <Router>{
            let params = use_query_map();
            let has_params = move || params.with(|p| p.get_str("a").is_some() || p.get_str("uri").is_some());
            view!{<Routes fallback=|| NotFound()>
                <ParentRoute ssr=SsrMode::PartiallyBlocked path=StaticSegment("/dashboard") view=Dashboard>
                    <Route path=StaticSegment("mathhub") view=|| view!(<MainPage page=Page::MathHub/>)/>
                    //<Route path="graphs" view=|| view!(<MainPage page=Page::Graphs/>)/>
                    <Route path=StaticSegment("log") view=|| view!(<MainPage page=Page::Log/>)/>
                    <Route path=StaticSegment("queue") view=|| view!(<MainPage page=Page::Queue/>)/>
                    <Route path=StaticSegment("settings") view=|| view!(<MainPage page=Page::Settings/>)/>
                    <Route path=StaticSegment("query") view=|| view!(<MainPage page=Page::Query/>)/>
                    <Route path=StaticSegment("") view=|| view!(<MainPage page=Page::Home/>)/>
                    <Route path=StaticSegment("*any") view=|| view!(<MainPage page=Page::NotFound/>)/>
                </ParentRoute>
                <Route ssr=SsrMode::PartiallyBlocked path=StaticSegment("/") view={move || if has_params() {
                        view! { <content::URITop/> }.into_any()
                    } else {
                        view! { <Redirect path="/dashboard"/> }.into_any()
                    }}
                />
            </Routes>}
        }</Router>
    }
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
    Query
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