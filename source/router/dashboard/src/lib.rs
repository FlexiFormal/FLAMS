#![recursion_limit = "512"]

#[cfg(any(
    all(feature = "ssr", feature = "hydrate", not(feature = "docs-only")),
    not(any(feature = "ssr", feature = "hydrate"))
))]
compile_error!("exactly one of the features \"ssr\" or \"hydrate\" must be enabled");

pub mod query;
mod settings;

pub mod ws {
    pub use flams_flodown::math::MathSocket;
    #[cfg(feature = "ssr")]
    pub use flams_flodown::math::TeXSocket;
    pub use flams_router_base::ws::*;
    pub use flams_router_buildqueue_components::QueueSocket;
    pub use flams_router_logging::LogSocket;
}

pub mod server_fns {
    pub mod content {
        pub use flams_router_content::server_fns::*;
    }
    pub mod backend {
        pub use flams_router_backend::server_fns::*;
    }
    pub mod buildqueue {
        pub use flams_router_buildqueue_base::server_fns::*;
    }
    pub mod git {
        pub use flams_router_git_base::server_fns::*;
    }
    pub mod login {
        pub use flams_router_login::server_fns::*;
    }
    pub mod search {
        pub use flams_router_search::{search_query, search_symbols};
    }
    pub use super::query::query_api as query;
    pub use super::settings::{get_settings as settings, reload};
}

pub use flams_router_base::LoginState;
use ftml_dom::FtmlViews;
use leptos::{
    either::{Either, EitherOf4},
    prelude::*,
};
use leptos_meta::{Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Outlet, ParentRoute, Redirect, Route, Router, Routes},
    hooks::use_query_map,
    path,
};
use thaw::{Divider, Grid, GridItem, Layout, LayoutHeader, LayoutPosition, LayoutSider};

#[component]
pub fn Main() -> AnyView {
    provide_meta_context();
    view! {
        <Title text="𝖥𝖫∀𝖬∫"/>
        <Router>{
            let has_params = Memo::new(move |_| use_query_map().with(|p| p.get_str("a").is_some() || p.get_str("uri").is_some()));
            view!{<Routes fallback=|| NotFound()>
                <ParentRoute/* ssr=SsrMode::InOrder*/ path=() view=Top>
                    <ParentRoute path=path!("/dashboard") view=Dashboard>
                        <Route path=path!("mathhub") view=|| view!(<MainPage page=Page::MathHub/>).into_any()/>
                        //<Route path="graphs" view=|| view!(<MainPage page=Page::Graphs/>)/>
                        <Route path=path!("log") view=|| view!(<MainPage page=Page::Log/>).into_any()/>
                        <Route path=path!("queue") view=|| view!(<MainPage page=Page::Queue/>).into_any()/>
                        <Route path=path!("settings") view=|| view!(<MainPage page=Page::Settings/>).into_any()/>
                        <Route path=path!("query") view=|| view!(<MainPage page=Page::Query/>).into_any()/>
                        <Route path=path!("archives") view=|| view!(<MainPage page=Page::MyArchives/>).into_any()/>
                        <Route path=path!("users") view=|| view!(<MainPage page=Page::Users/>).into_any()/>
                        <Route path=path!("search") view=|| view!(<MainPage page=Page::Search/>).into_any()/>
                        <Route path=path!("flodown") view=|| view!(<MainPage page=Page::FloDown/>).into_any()/>
                        <Route path=path!("") view=|| view!(<MainPage page=Page::Home/>).into_any()/>
                        <Route path=path!("*any") view=|| view!(<MainPage page=Page::NotFound/>).into_any()/>
                    </ParentRoute>
                    <ParentRoute path=path!("/vscode") view= flams_router_vscode::VSCodeWrap>
                        <Route path=path!("search") view=flams_router_search::vscode::VSCodeSearch/>
                    </ParentRoute>
                    <Route path=path!("/document") view={move || {
                        use flams_router_content::components::{DocumentOfTop,DocumentOfTopProps};
                        let params = use_query_map().get_untracked();
                        if let Some(p) = params.get_str("uri") {
                            let Ok(uri) = <ftml_uris::Uri as std::str::FromStr>::from_str(p) else {
                                return view! { <Redirect path="/dashboard"/> }.into_any()
                            };
                            DocumentOfTop(DocumentOfTopProps{uri}).into_any()
                        } else {
                            view! { <Redirect path="/dashboard"/> }.into_any()
                        }
                    }.into_any()}/>
                    <Route path=path!("/") view={move || if has_params.get() {
                            view! { <flams_router_content::components::URITop/> }.into_any()
                        } else {
                            view! { <Redirect path="/dashboard"/> }.into_any()
                        }}
                    />
                </ParentRoute>
            </Routes>}
        }</Router>
    }.into_any()
}

#[component(transparent)]
fn Top() -> AnyView {
    use flams_router_login::components::LoginProvider;
    view!(<LoginProvider><leptos_router::components::Outlet/></LoginProvider>).into_any()
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    Search,
    FloDown,
    MyArchives,
    Users,
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
            Search => "search",
            FloDown => "flodown",
            Users => "users",
            NotFound => "notfound",
        }
    }
}
impl std::fmt::Display for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.key())
    }
}

#[component(transparent)]
pub fn Dashboard() -> AnyView {
    view! {
      <Outlet/>
    }
    .into_any()
}

#[component]
fn MainPage(page: Page) -> AnyView {
    //use flams_web_utils::components::Themer;
    /*view! {
    <Themer>{*/
    //flams_router_content::Views::top(move || {
    ftml_dom::global_setup(move || flams_router_content::Views::top(move || {
    view! {
          <Layout position=LayoutPosition::Absolute>
            //<Login>
              <LayoutHeader class="flams-header">
                <div style="width:100%">
                  <Grid cols=3>
                    <GridItem>""</GridItem>
                    <GridItem>
                    <svg xmlns="http://www.w3.org/2000/svg" width="120px" height="60px" viewBox="0 -805.5 2248.7 1111" xmlns:xlink="http://www.w3.org/1999/xlink" aria-hidden="true" style="color:var(--colorBrandForeground1)"><defs><path id="MJX-5-TEX-SS-1D5A5" d="M86 0V691H526V611H358L190 612V384H485V308H190V0H86Z"></path><path id="MJX-5-TEX-SS-1D5AB" d="M87 0V694H191V79L297 80H451L499 81V0H87Z"></path><path id="MJX-5-TEX-N-2200" d="M0 673Q0 684 7 689T20 694Q32 694 38 680T82 567L126 451H430L473 566Q483 593 494 622T512 668T519 685Q524 694 538 694Q556 692 556 674Q556 670 426 329T293 -15Q288 -22 278 -22T263 -15Q260 -11 131 328T0 673ZM414 410Q414 411 278 411T142 410L278 55L414 410Z"></path><path id="MJX-5-TEX-SS-1D5AC" d="M92 0V694H228L233 680Q236 675 284 547T382 275T436 106Q446 149 497 292T594 558L640 680L645 694H782V0H689V305L688 606Q688 577 500 78L479 23H392L364 96Q364 97 342 156T296 280T246 418T203 544T186 609V588Q185 568 185 517T185 427T185 305V0H92Z"></path><path id="MJX-5-TEX-SO-222B" d="M113 -244Q113 -246 119 -251T139 -263T167 -269Q186 -269 199 -260Q220 -247 232 -218T251 -133T262 -15T276 155T297 367Q300 390 305 438T314 512T325 580T340 647T361 703T390 751T428 784T479 804Q481 804 488 804T501 805Q552 802 581 769T610 695Q610 669 594 657T561 645Q542 645 527 658T512 694Q512 705 516 714T526 729T538 737T548 742L552 743Q552 745 545 751T525 762T498 768Q475 768 460 756T434 716T418 652T407 559T398 444T387 300T369 133Q349 -38 337 -102T303 -207Q256 -306 169 -306Q119 -306 87 -272T55 -196Q55 -170 71 -158T104 -146Q123 -146 138 -159T153 -195Q153 -206 149 -215T139 -230T127 -238T117 -242L113 -244Z"></path></defs><g stroke="currentcolor" fill="currentcolor" stroke-width="0" transform="scale(1,-1)"><g data-mml-node="math"><g data-mml-node="mstyle"><g data-mml-node="TeXAtom" data-mjx-texclass="ORD"><g data-mml-node="mi"><use data-c="1D5A5" xlink:href="#MJX-5-TEX-SS-1D5A5"></use></g></g><g data-mml-node="mspace" transform="translate(569,0)"></g><g data-mml-node="TeXAtom" data-mjx-texclass="ORD" transform="translate(469,0)"><g data-mml-node="mi"><use data-c="1D5AB" xlink:href="#MJX-5-TEX-SS-1D5AB"></use></g></g><g data-mml-node="mspace" transform="translate(1011,0)"></g><g data-mml-node="mpadded" transform="translate(651,0)"><g transform="translate(0,23)"><g data-mml-node="mi"><use data-c="2200" xlink:href="#MJX-5-TEX-N-2200"></use></g></g></g><g data-mml-node="mspace" transform="translate(1207,0)"></g><g data-mml-node="TeXAtom" data-mjx-texclass="ORD" transform="translate(1097,0)"><g data-mml-node="mi"><use data-c="1D5AC" xlink:href="#MJX-5-TEX-SS-1D5AC"></use></g></g><g data-mml-node="mspace" transform="translate(1972,0)"></g><g data-mml-node="mo" transform="translate(1638.7,0) translate(0 0.5)"><use data-c="222B" xlink:href="#MJX-5-TEX-SO-222B"></use></g></g></g></g></svg>
                      /*<h1 style="font-family:serif;color:var(--colorBrandForeground1)">
                        "𝖥𝖫∀𝖬∫"
                      </h1>*/
                    </GridItem>
                    <GridItem>
                      <div style="width:calc(100% - 20px);text-align:right;padding:10px">
                        {user_field().into_any()}
                      </div>
                    </GridItem>
                  </Grid>
                  <Divider/>
                </div>
              </LayoutHeader>
              <Layout position=LayoutPosition::Absolute class="flams-main" content_style="height:100%" has_sider=true>
                  <LayoutSider class="flams-menu" content_style="width:100%;height:100%">
                    {side_menu(page).into_any()}
                  </LayoutSider>
                  <Layout>
                    <div style="width:calc(100% - 10px);padding-left:5px;height:calc(100vh - 67px)">
                      {do_main(page).into_any()}
                      </div>
                  </Layout>
                </Layout>
            //</Login>
          </Layout>
        }
    })).into_any()
}

fn do_main(page: Page) -> AnyView {
    //leptos::logging::log!("Here!");
    let inner = || match page {
        Page::Home => view!(<flams_router_backend::index_components::Index/>).into_any(),
        Page::MathHub => view! {<flams_router_backend::components::ArchivesTop/>}.into_any(),
        //Page::Graphs => view!{<GraphTest/>},
        Page::Log => view! {<flams_router_logging::Logger/>}.into_any(),
        Page::Queue => view! {<flams_router_buildqueue_components::QueuesTop/>}.into_any(),
        Page::Query => view! {<query::Query/>}.into_any(),
        Page::Settings => view! {<settings::Settings/>}.into_any(),
        Page::MyArchives => view! {<flams_router_git_components::Archives/>}.into_any(),
        Page::Search => view! {<flams_router_search::components::SearchTop/>}.into_any(),
        Page::FloDown => view! {<flams_flodown::FloDownEditor/>}.into_any(),
        Page::Users => view! {<flams_router_login::components::Users/>}.into_any(),
        _ => view!(<span>"TODO"</span>).into_any(),
        //Page::Login => view!{<LoginPage/>}
    };
    view!(<main style="height:100%">{inner()}</main>).into_any()
}

#[component]
fn NotFound() -> AnyView {
    #[cfg(feature = "ssr")]
    {
        let resp = expect_context::<leptos_axum::ResponseOptions>();
        resp.set_status(axum::http::StatusCode::NOT_FOUND);
    }

    view! {
        <h3>"Not Found"</h3>
    }
    .into_any()
}

fn side_menu(page: Page) -> AnyView {
    use thaw::{NavDrawer, NavItem};
    view! {
        <NavDrawer selected_value=page.to_string() class="flams-menu-inner">
            <NavItem value="home" href="/">"Home"</NavItem>
            <NavItem value="mathhub" href="/dashboard/mathhub">"MathHub"</NavItem>
            <NavItem value="query" href="/dashboard/query">"Queries"</NavItem>
            <NavItem value="search" href="/dashboard/search">"Search Content"</NavItem>
            {move || {let s = LoginState::get(); match s {
                LoginState::NoAccounts => view!{
                    <NavItem value="log" href="/dashboard/log">"Logs"</NavItem>
                    <NavItem value="settings" href="/dashboard/settings">"Settings"</NavItem>
                    <NavItem value="queue" href="/dashboard/queue">"Queue"</NavItem>
                    <NavItem value="flodown" href="/dashboard/flodown">"FloDown"</NavItem>
                }.into_any(),
                LoginState::Admin  => view!{
                  <NavItem value="log" href="/dashboard/log">"Logs"</NavItem>
                  <NavItem value="settings" href="/dashboard/settings">"Settings"</NavItem>
                  <NavItem value="queue" href="/dashboard/queue">"Queue"</NavItem>
                  <NavItem value="flodown" href="/dashboard/flodown">"FloDown"</NavItem>
                  <NavItem value="users" href="/dashboard/users">"Manage Users"</NavItem>
                }.into_any(),
                LoginState::User{is_admin:true,..} => view!{
                  <NavItem value="log" href="/dashboard/log">"Logs"</NavItem>
                  <NavItem value="settings" href="/dashboard/settings">"Settings"</NavItem>
                  <NavItem value="queue" href="/dashboard/queue">"Queue"</NavItem>
                  <NavItem value="archives" href="/dashboard/archives">"My Archives"</NavItem>
                  <NavItem value="flodown" href="/dashboard/flodown">"FloDown"</NavItem>
                }.into_any(),
                LoginState::User{..} => view!{
                    <NavItem value="queue" href="/dashboard/queue">"Queue"</NavItem>
                    <NavItem value="archives" href="/dashboard/archives">"My Archives"</NavItem>
                    <NavItem value="flodown" href="/dashboard/flodown">"FloDown"</NavItem>
                }.into_any(),
                LoginState::None | LoginState::Loading => ().into_any()
            }}}
        </NavDrawer>
    }
    .into_any()
}

fn user_field() -> AnyView {
    use flams_web_utils::components::ClientOnly;
    use flams_web_utils::components::{Spinner, SpinnerSize};
    use thaw::{Menu, MenuItem, MenuPosition, MenuTrigger, MenuTriggerType};

    view! {//<ClientOnly>
        <div class="flams-user-menu-trigger">{
        let theme = expect_context::<RwSignal<thaw::Theme>>();
        let on_select = move |key: &'static str| match key {
            "theme" => {
                theme.update(|v| {
                    if v.name == "dark" {
                        *v = thaw::Theme::light();
                    } else {
                        *v = thaw::Theme::dark();
                    }
                });
            }
            _ => unreachable!(),
        };
        let src = Memo::new(|_| match LoginState::get() {
            LoginState::User { avatar, .. } => Some(avatar),
            LoginState::Admin => Some("/admin.png".to_string()),
            _ => None,
        });
        let icon = Memo::new(move |_| if theme.with(|v| v.name == "dark")
            {icondata_bi::BiSunRegular} else {icondata_bi::BiMoonSolid}
        );
        let text = Memo::new(move |_| if theme.with(|v| v.name == "dark")
            {"Light Mode"} else {"Dark Mode"}
        );
        view!{
        <Menu on_select trigger_type=MenuTriggerType::Hover position=MenuPosition::Bottom>
            <MenuTrigger slot>
                <thaw::Avatar src />
            </MenuTrigger>
            // AiGitlabFilled
            <MenuItem value="theme" icon=icon>{text}</MenuItem>
            <Divider/>
            {move || match LoginState::get() {
                LoginState::None => login_form().into_any(),
                LoginState::NoAccounts => view!(<span>"Admin"</span>).into_any(),
                LoginState::Admin => logout_form("admin".to_string()).into_any(),
                LoginState::User{name,..} => logout_form(name).into_any(),
                LoginState::Loading => view!(<Spinner size=SpinnerSize::Tiny/>).into_any()
            }}
        </Menu>
        }
    }</div>
    //</ClientOnly>
    }
    .into_any()
}

fn logout_form(user: String) -> AnyView {
    use thaw::Button;
    let login = expect_context::<RwSignal<LoginState>>();
    let action = Action::new(move |_| {
        login.set(LoginState::None);
        flams_router_login::server_fns::logout()
    });
    view!(<span>{user}" "<Button on_click=move |_| {action.dispatch(());}>Logout</Button></span>)
        .into_any()
}

fn login_form() -> AnyView {
    use thaw::{Button, Input, InputType};
    let login = expect_context();
    let action = Action::new(move |pwd: &String| do_login(pwd.clone(), login));
    let value = RwSignal::<String>::new(String::new());
    view! {
      <Button on_click=move |_| {action.dispatch(value.get_untracked());}>Login</Button>
      <Input placeholder="admin pwd" value input_type=InputType::Password/>
    }
    .into_any()
}

#[allow(unused_variables)]
async fn do_login(pw: String, login: RwSignal<LoginState>) {
    let pwd = if pw.is_empty() { None } else { Some(pw) };
    match flams_router_login::server_fns::login(pwd).await {
        Ok(Some(u @ (LoginState::Admin | LoginState::User { .. }))) => login.set(u),
        Ok(_) => (),
        Err(e) => {
            #[cfg(feature = "hydrate")]
            flams_web_utils::components::display_error(std::borrow::Cow::Owned(format!(
                "Error: {e}"
            )));
        }
    }
    let _ = view!(<Redirect path="/dashboard"/>);
}
