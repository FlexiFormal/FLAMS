use leptos::*;
use leptos::server_fn::ServerFn;
use leptos_meta::*;
use leptos_router::*;
use crate::components::*;
use thaw::*;
use crate::accounts::{LoginState, WithAccount, WithAccountClient};
use crate::components::mathhub_tree::ArchivesTop;
use crate::console_log;

#[derive(Copy,Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
pub enum Page {
    Home,
    MathHub,
    Graphs,
    Log,
    NotFound,
    Queue,
    Settings,
    Login
}
impl Page {
    pub fn key(self) -> &'static str {
        match self {
            Page::Home => "home",
            Page::MathHub => "mathhub",
            Page::Graphs => "graphs",
            Page::Log => "log",
            Page::Login => "login",
            Page::Queue => "queue",
            Page::Settings => "settings",
            Page::NotFound => "notfound"
        }
    }
}
impl std::fmt::Display for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.key())
    }

}

/*
#[derive(Copy,Clone)]
pub struct PathSignal{pub read:ReadSignal<Page>,pub write:WriteSignal<Page>}
#[cfg(feature = "server")]
#[component]
pub(crate) fn Main() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    //let foo = create_resource(|| (),|_| async {test().await});
    //let (path, home_path) = create_signal("home");
    //let mh_path = home_path.clone();
    //let graph_path = home_path.clone();
    //let log_path = home_path.clone();
    let theme = create_rw_signal(Theme::light());
    let themeclone = theme.clone();
    let callback = move |t:Theme| themeclone.set(t);
    //let (path,path_set) = create_signal(Page::Home);
    //provide_context(PathSignal{read:path.clone(),write:path_set});
    view! {
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/immt.css"/>

        // sets the document title
        <Title text="iMᴍᴛ"/>
        /*{
            #[cfg(feature = "accounts")]
            {view!{<LoginCheck/>} }
        }*/
        <ThemeProvider theme><GlobalStyle /><Router>
            <LoginCheck><Layout>
                <LayoutHeader style="background-color: #0078ffaa;">
                    <Grid cols=3>
                        <GridItem offset=1><h2>"iMᴍᴛ"</h2></GridItem>
                        <GridItem><Space justify=SpaceJustify::End><User/></Space></GridItem>
                    </Grid>
                </LayoutHeader>
                <Layout style="padding: 20px;">
                    <Routes>
                        <Route path="/" view=|| view!(<TopMenu page=Page::Home/>)/>
                        <Route path="/mathhub" view=|| view!(<TopMenu page=Page::MathHub/>)/>
                        <Route path="/graphs" view=|| view!(<TopMenu page=Page::Graphs/>)/>
                        <Route path="/log" view=|| view!(<TopMenu page=Page::Log/>)/>
                        <Route path="/queue" view=|| view!(<TopMenu page=Page::Queue/>)/>
                        <Route path="/settings" view=|| view!(<TopMenu page=Page::Settings/>)/>
                        <Route path="/*any" view=|| view!(<TopMenu page=Page::NotFound/>)/>
                    </Routes>
                </Layout>
            </Layout></LoginCheck>
        </Router></ThemeProvider>
    }
}

#[component]
fn TopMenu(page:Page) -> impl IntoView {
    view!{<Layout has_sider=true>
        <LayoutSider class="immt-menu" content_class="immt-menu">
            <LeftMenu page/>
           /* <Card>
                <Space>
                    <Button on_click=move |_| callback.call(Theme::light())>"Light"</Button>
                    <Button on_click=move |_| callback.call(Theme::dark())>"Dark"</Button>
                </Space>
            </Card> */
        </LayoutSider>
        <main class="immt-main">{match page {
            Page::Home => view!{<HomePage/>},
            Page::MathHub => view!{<ArchiveOrGroups/>},
            Page::Graphs => view!{<GraphTest/>},
            Page::Log => view!{<FullLog/>},
            Page::Queue => view!{<Queue/>},
            Page::Settings => view!{<Settings/>},
            _ => view!(<NotFound/>).into_view()
            //Page::Login => view!{<LoginPage/>}
        }}</main>
    </Layout>}
}

#[island]
fn User() -> impl IntoView {
    async fn status() -> Result<LoginState,ServerFnError> {
        #[cfg(feature="server")]
        {Ok(LoginState::Loading)}
        #[cfg(feature="client")]
        {crate::accounts::login_status().await}
    }

    let status = create_local_resource(|| (),|_| status());
    let login = expect_context::<RwSignal<LoginState>>();
    view!{
        <Suspense fallback=|| view! {<Spinner size=SpinnerSize::Tiny/>}>{move || {
            let logstate = if let Some(Ok(ret)) = status.get() {
                ret
            } else { LoginState::None };
            login.set(logstate.clone());
            match logstate {
                LoginState::None =>view!(<LoginForm/>).into_view(),
                LoginState::Admin | LoginState::NoAccounts => view!(<span>"Admin"</span>).into_view(),
                LoginState::User(user) => view!(<span>{user.name}</span>).into_view(),
                LoginState::Loading => view!(<Spinner size=SpinnerSize::Tiny/>).into_view()
            }
        }}</Suspense>

    }
}


#[island]
fn LoginCheck(children:Children) -> impl IntoView {
    let login = create_rw_signal(LoginState::Loading);
    view!{
        <Provider value=login>{children()}</Provider>
    }
}
#[island]
fn LeftMenu(page:Page) -> impl IntoView {
    let state = expect_context::<RwSignal<LoginState>>();
    view! {
    <Menu value=page.to_string()>
        <a href="/"><MenuItem key="home" label="Home"/></a>
        <a href="/mathhub"><MenuItem key="mathhub" label="MathHub"/></a>
        {move || match state.get() {
            LoginState::Admin | LoginState::NoAccounts => view!{
                        <a href="/graphs"><MenuItem key="graphs" label="Graphs"/></a>
                        <a href="/log"><MenuItem key="log" label="Logs"/></a>
                        <a href="/settings"><MenuItem key="settings" label="Settings"/></a>
                        <a href="/queue"><MenuItem key="queue" label="Queue"/></a>
                    },
            LoginState::User(..) => view!{
                    <a href="/graphs"><MenuItem key="graphs" label="Graphs"/></a><span/>
                },
            LoginState::None | LoginState::Loading => view!(<span/><span/>)
        }}
    </Menu>
    }
}



*/*/

// --------------------------------------------------

#[component]
pub fn MainNew() -> impl IntoView {
    provide_meta_context();
    view! {
        <Title text="iMᴍᴛ"/>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/immt.css"/>
        <Router><Routes>
            <Route path="/" view=move || view!{
                <Redirect path="/dashboard" /> // TODO
            }/>
            <Route path="/dashboard" view=Dashboard>
                <Route path="mathhub" view=|| view!(<MainPage page=Page::MathHub/>)/>
                <Route path="graphs" view=|| view!(<MainPage page=Page::Graphs/>)/>
                <Route path="log" view=|| view!(<MainPage page=Page::Log/>)/>
                <Route path="queue" view=|| view!(<MainPage page=Page::Queue/>)/>
                <Route path="settings" view=|| view!(<MainPage page=Page::Settings/>)/>
                <Route path="" view=|| view!(<MainPage page=Page::Home/>)/>
                <Route path="*any" view=|| view!(<MainPage page=Page::NotFound/>)/>
            </Route>
            //<Route path="/*any" view=NotFound/>
        </Routes></Router>
    }
}

#[component(transparent)]
fn Dashboard() -> impl IntoView {
    use thaw::*;
    let theme = create_rw_signal(Theme::light());
    provide_context(theme);
    view!{
        <WithAccount><Show when=move || {expect_context::<ReadSignal<LoginState>>().get() != LoginState::Loading}>
            <ThemeProvider theme><GlobalStyle/><Layout content_style="width:100%">
                <LayoutHeader style="background-color: #0078ffaa;">
                    <Grid cols=3>
                        <GridItem offset=1><h2>"iMᴍᴛ"</h2></GridItem>
                        <GridItem><Space justify=SpaceJustify::End><UserField/></Space></GridItem>
                    </Grid>
                </LayoutHeader>
                <Layout style="padding: 20px;"><Outlet/></Layout>
            </Layout></ThemeProvider>
        </Show></WithAccount>
    }
}

#[component]
fn UserField() -> impl IntoView {
    let login = expect_context::<ReadSignal<LoginState>>();
    move || match login.get() {
        LoginState::None =>view!(<LoginForm/>).into_view(),
        LoginState::Admin | LoginState::NoAccounts => view!(<span>"Admin"</span>).into_view(),
        LoginState::User(user) => view!(<span>{user.name}</span>).into_view(),
        LoginState::Loading => view!(<Spinner size=SpinnerSize::Tiny/>).into_view()
    }
}

#[island]
fn LoginForm() -> impl IntoView {
    let act = create_server_action::<crate::accounts::Login>();
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
    let login = expect_context::<ReadSignal<LoginState>>();
    let mymain = move || view!(<main class="immt-main">{match page {
        Page::Home => view!{<HomePage/>},
        Page::MathHub => view!{<ArchivesTop/>},
        Page::Graphs => view!{<GraphTest/>},
        Page::Log => view!{<FullLog/>},
        Page::Queue => view!{<QueuesTop/>},
        Page::Settings => view!{<Settings/>},
        _ => view!(<NotFound/>).into_view()
        //Page::Login => view!{<LoginPage/>}
    }}</main>);
    view!{<Layout has_sider=true content_style="width:100%">
        <LayoutSider class="immt-menu" content_class="immt-menu">
            <Menu value=page.to_string()>
                <a href="/"><MenuItem key="home" label="Home"/></a>
                <a href="/dashboard/mathhub"><MenuItem key="mathhub" label="MathHub"/></a>
                {match login.get() {
                    LoginState::Admin | LoginState::NoAccounts => view!{
                        <a href="/dashboard/graphs"><MenuItem key="graphs" label="Graphs"/></a>
                        <a href="/dashboard/log"><MenuItem key="log" label="Logs"/></a>
                        <a href="/dashboard/settings"><MenuItem key="settings" label="Settings"/></a>
                        <a href="/dashboard/queue"><MenuItem key="queue" label="Queue"/></a>
                    },
                    LoginState::User(..) => view!{
                        <a href="/dashboard/graphs"><MenuItem key="graphs" label="Graphs"/></a><span/>
                    },
                    LoginState::None | LoginState::Loading => view!(<span/><span/>)
                }}
            </Menu>
           /* <Card>
                <Space>
                    <Button on_click=move |_| callback.call(Theme::light())>"Light"</Button>
                    <Button on_click=move |_| callback.call(Theme::dark())>"Dark"</Button>
                </Space>
            </Card> */
        </LayoutSider>
        <WithAccountClient user=login.get()>{mymain()}</WithAccountClient>
    </Layout>}
}

#[component]
//fn HomePage<Ctrl:immt_api::controller::Controller+Clone+Send+'static>(#[prop(optional)] _ty: PhantomData<Ctrl>) -> impl IntoView {
fn HomePage() -> impl IntoView {
    //let ctrl = crate::get_controller::<Ctrl>();
    //println!("Wuuhuu! \\o/");
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