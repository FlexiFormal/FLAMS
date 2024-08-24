use leptos::prelude::*;
//use leptos::server_fn::ServerFn;
use leptos_meta::*;
use leptos_router::{*,components::*,SsrMode,hooks::use_query_map};
use crate::components::*;
use thaw::*;
use crate::accounts::LoginState;
use crate::components::mathhub_tree::ArchivesTop;
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
                <HydrationScripts options />//islands=true/>
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
            //<Route path=StaticSegment("/*any") view=NotFound/>
        </Routes>}}</Router>
    }
}

#[component(transparent)]
fn Dashboard() -> impl IntoView {
    use thaw::*;
    use crate::accounts::WithAccount;
    view!{
        <WithAccount>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/immt.css"/>
        <Outlet/>
    </WithAccount>
    }
}

#[component]
fn Header() -> impl IntoView {
    use thaw::*;
    view!{
        <div style="width:100%">
        <Grid cols=3>
            <GridItem>""</GridItem>
            <GridItem><h1 style="font-family:serif;color:var(--colorBrandForeground1)">"iMᴍᴛ"</h1></GridItem>
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
    use thaw::*;
    crate::css!(user_menu=".immt-user-menu-trigger{width:fit-content;margin-left:auto} .immt-user-menu {position:absolute;right:0}");
    let theme = expect_context::<RwSignal::<thaw::Theme>>();
    let on_select = move |key: String| match key.as_str() {
        "theme" => {
            theme.update(|v| {
                if v.name == "dark" {
                    *v = thaw::Theme::light()
                } else {
                    *v = thaw::Theme::dark()
                }
            });
        }
        _ => {}
    };


    view!{<Menu on_select trigger_type=MenuTriggerType::Hover class="immt-user-menu">
        <MenuTrigger slot class="immt-user-menu-trigger">
            <thaw::Avatar />
        </MenuTrigger>
        // AiGitlabFilled
        {move || {
            let dark = theme.with(|v| v.name == "dark");
            let icon = if dark {icondata_bi::BiSunRegular} else {icondata_bi::BiMoonSolid};
            let text = if dark {"Light Mode"} else {"Dark Mode"};
            view!(<MenuItem value="theme" icon=icon>{text}</MenuItem>)
        }}
        <Divider/>
        {move || {
            let login = crate::accounts::get_account();
            match login {
                LoginState::None =>view!(<LoginForm/>).into_any(),
                LoginState::Admin | LoginState::NoAccounts => view!(<span>"Admin"</span>).into_any(),
                LoginState::User(user) => view!(<span>{user.name}</span>).into_any(),
                LoginState::Loading => view!(<Spinner size=SpinnerSize::Tiny/>).into_any()
            }
        }}
    </Menu>}
}

#[island]
fn LoginForm() -> impl IntoView {
    use thaw::*;

    let toaster = ToasterInjection::expect_context();

    let _ = view!(<Input/><Button/>);
    let username: NodeRef<leptos::html::Input> = create_node_ref();
    let pw: NodeRef<leptos::html::Input> = create_node_ref();
    let login = expect_context::<RwSignal<LoginState>>();
    let action = Action::new(move |(name,pw):&(String,String)| {
        let name = name.clone();let pw=pw.clone();
        async move {
            if let Ok(u) = crate::accounts::login(name,pw).await {
                login.set(u);
            } else {
                toaster.dispatch_toast(view!{
                    /*<Toast><ToastBody>*/<MessageBar intent=MessageBarIntent::Error>
                        <MessageBarBody>
                            //<MessageBarTitle>"Intent error"</MessageBarTitle>
                            "User name or password incorrect"
                        </MessageBarBody>
                    </MessageBar>/*</ToastBody></Toast>*/
                }.into_any(),ToastOptions::default().with_position(ToastPosition::Top))
            }
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        // stop the page from reloading!
        ev.prevent_default();
        let un = username.get().expect("<input> should be mounted").value();
        let pw = pw.get().expect("<input> should be mounted").value();
        action.dispatch((un,pw));
    };
    view!{
        <form on:submit=on_submit>
            <Caption1>"Login:"</Caption1><br/>
            <span class="thaw-input">
                <input node_ref=username class="thaw-input__input" type="text" placeholder="user name" name="username"/>
            </span><br/>
            <span class="thaw-input">
                <input node_ref=pw class="thaw-input__input" type="password" placeholder="password" name="password"/>
            </span><br/>
            <input class="thaw-button--secondary thaw-button--small thaw-button thaw-button--rounded" type="submit" value="Log in"/>
        </form>
    }
}

#[component]
fn MainPage(page:Page) -> impl IntoView {
    use crate::components::Themer;
    let do_main = move || view!(<main style="height:100%">{match page {
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

    view!{<Themer>
        <span class="thaw-config-provider"><Layout position=LayoutPosition::Absolute>
            <LayoutHeader class="immt-header"><Header/></LayoutHeader>
            <Layout position=LayoutPosition::Absolute class="immt-main" content_style="height:100%" has_sider=true>
                <LayoutSider class="immt-menu" content_style="width:100%;height:100%">
                    <SideMenu page/>
                </LayoutSider>
                <Layout>
                    <div style="width:calc(100% - 10px);padding-left:5px;height:calc(100vh - 67px)">{do_main()}</div>
                </Layout>
            </Layout>
        </Layout></span>
    </Themer>}
}

#[component]
fn SideMenu(page:Page) -> impl IntoView {
    use thaw::*;
    let login = || crate::accounts::get_account();
    view!{
        <NavDrawer selected_value=page.to_string() class="immt-menu-inner">
            <NavItem value="home" href="/">"Home"</NavItem>
            <NavItem value="mathhub" href="/dashboard/mathhub">"MathHub"</NavItem>
            <NavItem value="query" href="/dashboard/query">"Queries"</NavItem>
            {move || match login() {
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