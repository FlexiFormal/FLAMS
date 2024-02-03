use either::Either;
use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

macro_rules! scomponent {
    ($ident:ident($($i:ident:$tp:tt),*) => $f:block) => {
        #[component]
        fn $ident($($i:$tp),*) -> impl IntoView {
            #[cfg(feature = "ssr")]
            $f
            #[cfg(not(feature = "ssr"))]
            view! { "" }
        }
    }
}
macro_rules! ssr {
    ($($t:tt)*) => {
        #[cfg(feature = "ssr")]
        {$($t)*}
        #[cfg(not(feature = "ssr"))]
        view! { "" }
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/immt-server.css"/>

        // sets the document title
        <Title text="iMMT"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }.into_view()
        }>
            <nav>"what is this??"
            </nav>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="backend" view=Backend/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <h1>"i"<span style="font-variant:small-caps">Mmt</span></h1>
    }
}

#[cfg(feature = "ssr")]
use immt_api::backend::archives::{Archive, ArchiveGroup};

#[cfg(feature = "ssr")]
async fn get_archive_view() -> String {
    use std::fmt::Write;
    //std::thread::sleep(std::time::Duration::from_secs(5));
    let archives:&Vec<Either<ArchiveGroup,Archive>> = crate::controller::CONTROLLER.archives().get_top();
    let mut ls = vec!(archives.iter());
    let mut prev:Vec<(String,String)> = vec!();
    let mut curr = String::new();
    while !ls.is_empty() {
        match ls.last_mut().unwrap().next() {
            None => {
                ls.pop();
                match prev.pop() {
                    None => break,
                    Some(p) => {
                        let old = std::mem::replace(&mut curr,p.1);
                        write!(curr,"<li><div>{}</div><ul>{}</ul></li>",p.0,old).unwrap();
                    }
                }
            },
            Some(e) => match e {
                Either::Left(g) => {
                    ls.push(g.archives().iter());
                    let old = std::mem::replace(&mut curr,String::new());
                    prev.push((g.id().steps().last().unwrap().to_string(),old));
                },
                Either::Right(a) => {
                    write!(curr,"<li>{}</li>",a.id().steps().last().unwrap()).unwrap();
                }
            }
        }
    }
    curr
}

#[component]
fn ArchiveView() -> impl IntoView { ssr!{
    let ls = create_resource(|| (), |_| async { get_archive_view().await });
    view! {
        <div>"Archives:"<Suspense fallback=move || view!{"Loading..."}>
        <div style="column-count: 5;column-gap: 10px;">
            {move || {
                let ul = html::ul();
                ul.inner_html(ls.get().unwrap_or_else(|| "Error".to_string()))
            }}
        </div></Suspense></div>
    }
} }


#[component]
fn MathHubView() -> impl IntoView {ssr!{
    use crate::backend::archives::{archive_ids,mathhub};
    let mh = create_resource(|| (), |_| async { mathhub().await });
    let archives = create_resource(|| (), |_| async { archive_ids().await.unwrap_or_default() });
    view! {
        <div>"MathHub: "<Suspense fallback=move || view!{"Loading..."}>
        <code>{move || mh.get().map(|m| m.unwrap_or_else(|e| "Error".to_string()))}</code>
        " ("{archives.get().map(|a| a.len()).unwrap_or(0)}" archives)"
    </Suspense></div>
    }
}}

#[component]
fn Backend() -> impl IntoView {ssr!{
    view! {
        <h1>"Backend"</h1>
        <MathHubView/>
        <ArchiveView/>
    }
}}

#[island]
fn Counter() -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
        <button on:click=on_click>"Click Me: " {count}</button>
    }
}
