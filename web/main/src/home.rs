use leptos::prelude::*;
//use leptos::server_fn::ServerFn;
use leptos_meta::*;
use leptos_router::{*,components::*,SsrMode,hooks::use_query_map};
use crate::components::*;
use thaw::*;
use crate::accounts::LoginState;
use crate::components::mathhub_tree::ArchivesTop;
use leptos::form::ActionForm;
#[cfg(feature = "server")]
use crate::accounts::WithAccount;

pub(crate) fn shell(options:LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options islands=true/>
                <MetaTags/>
            </head>
            <body>
                <Main/>
            </body>
        </html>
    }
}

#[derive(Copy,Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
pub enum Page {
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
    pub fn key(self) -> &'static str {
        match self {
            Page::Home => "home",
            Page::MathHub => "mathhub",
            //Page::Graphs => "graphs",
            Page::Log => "log",
            Page::Login => "login",
            Page::Queue => "queue",
            Page::Settings => "settings",
            Page::Query => "query",
            Page::NotFound => "notfound"
        }
    }
}
impl std::fmt::Display for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.key())
    }
}


#[component]
pub fn Main() -> impl IntoView {
    provide_meta_context();
    view! {
        <Title text="iMᴍᴛ"/>
        <Router>{
            let params = use_query_map();
            let has_a_param = Memo::new(move |_| params.with(|p| p.get("a").is_some() || p.get("uri").is_some()));
            view!{<Routes fallback=|| view!(<NotFound/>)>
            <Route ssr=SsrMode::PartiallyBlocked path=StaticSegment("/") view=move || {if has_a_param.get() {
                    view! { <content::URITop/> }.into_any()
                } else {
                    view! { <Redirect path="/dashboard"/> }.into_any()
                }}
            />
            <ParentRoute path=StaticSegment("/dashboard") view=Dashboard>
                <Route path=StaticSegment("mathhub") view=|| view!(<MainPage page=Page::MathHub/>)/>
                //<Route path="graphs" view=|| view!(<MainPage page=Page::Graphs/>)/>
                <Route path=StaticSegment("log") view=|| view!(<MainPage page=Page::Log/>)/>
                <Route path=StaticSegment("queue") view=|| view!(<MainPage page=Page::Queue/>)/>
                <Route path=StaticSegment("settings") view=|| view!(<MainPage page=Page::Settings/>)/>
                <Route path=StaticSegment("query") view=|| view!(<MainPage page=Page::Query/>)/>
                <Route path=StaticSegment("") view=|| view!(<MainPage page=Page::Home/>)/>
                <Route path=StaticSegment("*any") view=|| view!(<MainPage page=Page::NotFound/>)/>
            </ParentRoute>
            //<Route path=StaticSegment("/*any") view=NotFound/>
        </Routes>}}</Router>
    }
}

#[component(transparent)]
fn Dashboard() -> impl IntoView {
    use thaw::*;
    use crate::accounts::WithAccountClient;
    #[cfg(feature="server")]
    view!{
        <WithAccount>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/immt.css"/>
        <Outlet/>
    </WithAccount>
    }
    #[cfg(feature="client")]
    view!{<WithAccountClient user=LoginState::Loading>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/immt.css"/>
        <Outlet/>
    </WithAccountClient>}
}

#[component]
fn Header() -> impl IntoView {
    use thaw::*;
    view!{
        <div style="width:100%">
        <Grid cols=3>
            <GridItem>""</GridItem>
            <GridItem><h1 style="font-family:serif">"iMᴍᴛ"</h1></GridItem>
            <GridItem>
                <div style="width:calc(100% - 20px);text-align:right;padding:10px"><UserField/>
                </div>
            </GridItem>
        </Grid>
        <Divider/>
        </div>
    }
}

#[component]
fn UserField() -> impl IntoView {
    let login = crate::accounts::get_account();
    match login {
        LoginState::None =>view!(<LoginForm/>).into_any(),
        LoginState::Admin | LoginState::NoAccounts => view!(<span>"Admin"</span>).into_any(),
        LoginState::User(user) => view!(<span>{user.name}</span>).into_any(),
        LoginState::Loading => view!(<Spinner size=SpinnerSize::Tiny/>).into_any()
    }
}

#[island]
fn LoginForm() -> impl IntoView {
    let act = ServerAction::<crate::accounts::Login>::new();
    view!{
        <ActionForm action=act>
            "Username: "<input type="text" name="username"/>
            "Password: "<input type="password" name="password"/>
            <input type="submit" value="Log in"/>
        </ActionForm>
    }
}

#[component]
fn MainPage(page:Page) -> impl IntoView {
    let do_main = move || view!(<main>{match page {
        Page::Home => view!{<HomePage/>}.into_any(),
        Page::MathHub => view!{<ArchivesTop/>}.into_any(),
        //Page::Graphs => view!{<GraphTest/>},
        Page::Log => view!{<FullLog/>}.into_any(),
        Page::Queue => view!{<QueuesTop/>}.into_any(),
        Page::Query => view!{<Query/>}.into_any(),
        Page::Settings => view!{<Settings/>}.into_any(),
        _ => view!(<NotFound/>).into_any()
        //Page::Login => view!{<LoginPage/>}
    }}</main>);

    view!{<ConfigProvider theme=RwSignal::new(Theme::dark())>
        <Layout position=LayoutPosition::Absolute>
            <LayoutHeader class="immt-header"><Header/></LayoutHeader>
            <Layout position=LayoutPosition::Absolute class="immt-main" content_style="height:100%" has_sider=true>
                <LayoutSider class="immt-menu" content_style="width:100%;height:100%">
                    <SideMenu page/>
                </LayoutSider>
                <div style="width:100%;padding-left:5px;height:calc(100vh - 67px)">{do_main()}</div>
            </Layout>
        </Layout>
    </ConfigProvider>}
}

#[component]
fn SideMenu(page:Page) -> impl IntoView {
    use thaw::*;
    let login = crate::accounts::get_account();
    view!{
        <NavDrawer selected_value=page.to_string() class="immt-menu-inner">
            <NavItem value="home" href="/">"Home"</NavItem>
            <NavItem value="mathhub" href="/dashboard/mathhub">"MathHub"</NavItem>
            <NavItem value="query" href="/dashboard/query">"Queries"</NavItem>
            {match login {
                LoginState::Admin | LoginState::NoAccounts => view!{
                    //<a href="/dashboard/graphs"><MenuItem key="graphs" label="Graphs"/></a>
                    <NavItem value="log" href="/dashboard/log">"Logs"</NavItem>
                    <NavItem value="settings" href="/dashboard/settings">"Settings"</NavItem>
                    <NavItem value="queue" href="/dashboard/queue">"Queue"</NavItem>
                }.into_any(),
                LoginState::User(..) => "".into_any(),
                LoginState::None | LoginState::Loading => "".into_any()
            }}
        </NavDrawer>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {"Henlo"}
}

/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    #[cfg(feature = "server")]
    {
        let resp = expect_context::<leptos_axum::ResponseOptions>();
        resp.set_status(http::StatusCode::NOT_FOUND);
    }

    view! {
        <h3>"Not Found"</h3>
    }
}