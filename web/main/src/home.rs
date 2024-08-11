use leptos::*;
//use leptos::server_fn::ServerFn;
use leptos_meta::*;
use leptos_router::*;
use crate::components::*;
use thaw::*;
use crate::accounts::{LoginState, WithAccount, WithAccountClient};
use crate::components::mathhub_tree::ArchivesTop;

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
            let has_a_param = create_memo(move |_| params.with(|p| p.get("a").is_some() || p.get("uri").is_some()));
            view!{<Routes>
            <Route path="/" view=move || if has_a_param.get() {
                    view! { <content::SomeUri/> }
                } else {
                    view! { <Redirect path="/dashboard"/> }
                }
            />
        /*
            <Route path="/:a" view=|| view!(<content::SomeUri/>)/>
            <Route path="/" view=move || view!{
                <Redirect path="/dashboard" /> // TODO
            }/>

         */
            <Route path="/dashboard" view=Dashboard>
                // id=leptos means cargo-leptos will hot-reload this stylesheet
                <Stylesheet id="leptos" href="/pkg/immt.css"/>
                <Route path="mathhub" view=|| view!(<MainPage page=Page::MathHub/>)/>
                //<Route path="graphs" view=|| view!(<MainPage page=Page::Graphs/>)/>
                <Route path="log" view=|| view!(<MainPage page=Page::Log/>)/>
                <Route path="queue" view=|| view!(<MainPage page=Page::Queue/>)/>
                <Route path="settings" view=|| view!(<MainPage page=Page::Settings/>)/>
                <Route path="query" view=|| view!(<MainPage page=Page::Query/>)/>
                <Route path="" view=|| view!(<MainPage page=Page::Home/>)/>
                <Route path="*any" view=|| view!(<MainPage page=Page::NotFound/>)/>
            </Route>
            //<Route path="/*any" view=NotFound/>
        </Routes>}}</Router>
    }
}

#[component(transparent)]
fn Dashboard() -> impl IntoView {
    use thaw::*;
    let theme = create_rw_signal(Theme::light());
    provide_context(theme);
    view!{
        <WithAccount><Show when=move || {expect_context::<ReadSignal<LoginState>>().get() != LoginState::Loading}>
            <ThemeProvider theme><GlobalStyle/><Layout position=LayoutPosition::Absolute>
                <LayoutHeader style="background-color: #0078ffaa;height:67px;text-align:center;">
                    <Grid cols=3>
                        <GridItem offset=1><h2>"iMᴍᴛ"</h2></GridItem>
                        <GridItem><Space justify=SpaceJustify::End><UserField/></Space></GridItem>
                    </Grid>
                </LayoutHeader>
                <Layout style="padding: 20px;top:67px" position=LayoutPosition::Absolute>
                    <Outlet/>
                </Layout>
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
        //Page::Graphs => view!{<GraphTest/>},
        Page::Log => view!{<FullLog/>},
        Page::Queue => view!{<QueuesTop/>},
        Page::Query => view!{<Query/>},
        Page::Settings => view!{<Settings/>},
        _ => view!(<NotFound/>).into_view()
        //Page::Login => view!{<LoginPage/>}
    }}</main>);
    view!{<Layout has_sider=true style="height:100%" content_style="height:100%">
        <LayoutSider class="immt-menu" content_class="immt-menu">
            <Menu value=page.to_string()>
                <a href="/"><MenuItem key="home" label="Home"/></a>
                <a href="/dashboard/mathhub"><MenuItem key="mathhub" label="MathHub"/></a>
                <a href="/dashboard/query"><MenuItem key="query" label="Queries"/></a>
                {match login.get() {
                    LoginState::Admin | LoginState::NoAccounts => view!{
                        //<a href="/dashboard/graphs"><MenuItem key="graphs" label="Graphs"/></a>
                        <a href="/dashboard/log"><MenuItem key="log" label="Logs"/></a>
                        <a href="/dashboard/settings"><MenuItem key="settings" label="Settings"/></a>
                        <a href="/dashboard/queue"><MenuItem key="queue" label="Queue"/></a>
                    },
                    LoginState::User(..) => view!{
                        <span/><span/>//<a href="/dashboard/graphs"><MenuItem key="graphs" label="Graphs"/></a><span/>
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
        <Layout><WithAccountClient user=login.get()>{mymain()}</WithAccountClient></Layout>
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