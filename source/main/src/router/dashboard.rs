use leptos::{either::{Either, EitherOf4, EitherOf7, EitherOf9}, prelude::*};
use leptos_meta::Stylesheet;
use leptos_router::{components::{Outlet, Redirect}, hooks::use_navigate};
use crate::users::{Login,LoginState};

use super::Page;
use immt_web_utils::components::Themer;

use thaw::{Caption1, Divider, Grid, GridItem, Layout, LayoutHeader, LayoutPosition, LayoutSider, Menu, MenuItem, MenuTrigger, MenuTriggerType, NavDrawer, NavItem, ToasterInjection};


#[cfg(feature="hydrate")]
use std::borrow::Cow;
#[cfg(feature="hydrate")]
use immt_web_utils::components::display_error;

fn do_main(page:Page) -> impl IntoView {
  let inner =  || match page {
    Page::Home => EitherOf9::A(view!(<span>"TODO"</span>)),
    Page::MathHub => EitherOf9::B(view!{<super::backend::ArchivesTop/>}),
    //Page::Graphs => view!{<GraphTest/>},
    Page::Log => EitherOf9::C(view!{<super::logging::Logger/>}),
    Page::Queue => EitherOf9::D(view!{<super::buildqueue::QueuesTop/>}),
    Page::Query => EitherOf9::E(view!{<super::query::Query/>}),
    Page::Settings => EitherOf9::F(view!{<super::settings::Settings/>}),
    Page::MyArchives => EitherOf9::G(view!{<super::git::Archives/>}),
    Page::Users => EitherOf9::H(view!{<super::users::Users/>}),
    _ => EitherOf9::I(view!(<span>"TODO"</span>)),
    //Page::Login => view!{<LoginPage/>}
  };
  view!(<main style="height:100%">{inner()}</main>)
}

#[component(transparent)]
pub fn Dashboard() -> impl IntoView {
  view!{
    <Stylesheet id="leptos" href="/pkg/immt.css"/>
    <Outlet/>
  }
}

fn do_dashboard<V:IntoView + 'static>(f:impl FnOnce() -> V + Send + 'static) -> impl IntoView {
  use shtml_viewer_components::SHTMLGlobalSetup;
  view!{
    <Themer><SHTMLGlobalSetup>
      <Layout position=LayoutPosition::Absolute>
        //<Login>
          <LayoutHeader class="immt-header">
            <div style="width:100%">
              <Grid cols=3>
                <GridItem>""</GridItem>
                <GridItem>
                  <h1 style="font-family:serif;color:var(--colorBrandForeground1)">
                    "iMᴍᴛ"
                  </h1>
                </GridItem>
                <GridItem>
                  <div style="width:calc(100% - 20px);text-align:right;padding:10px">
                    {user_field()}
                  </div>
                </GridItem>
              </Grid>
              <Divider/>
            </div>
          </LayoutHeader>
          {f()}
        //</Login>
      </Layout>
    </SHTMLGlobalSetup></Themer>
  }
}

#[component]
pub fn MainPage(page:Page) -> impl IntoView {
  do_dashboard(move || view!(
    <Layout position=LayoutPosition::Absolute class="immt-main" content_style="height:100%" has_sider=true>
      <LayoutSider class="immt-menu" content_style="width:100%;height:100%">
        {side_menu(page)}
      </LayoutSider>
      <Layout>
        <div style="width:calc(100% - 10px);padding-left:5px;height:calc(100vh - 67px)">
          {do_main(page)}
          </div>
      </Layout>
    </Layout>
  ))
}

fn side_menu(page:Page) -> impl IntoView {
    view!{
        <NavDrawer selected_value=page.to_string() class="immt-menu-inner">
            <NavItem value="home" href="/">"Home"</NavItem>
            <NavItem value="mathhub" href="/dashboard/mathhub">"MathHub"</NavItem>
            <NavItem value="query" href="/dashboard/query">"Queries"</NavItem>
            {move || match LoginState::get() {
                LoginState::NoAccounts => leptos::either::EitherOf5::A(view!{
                    <NavItem value="log" href="/dashboard/log">"Logs"</NavItem>
                    <NavItem value="settings" href="/dashboard/settings">"Settings"</NavItem>
                    <NavItem value="queue" href="/dashboard/queue">"Queue"</NavItem>
                }),
                LoginState::Admin  => leptos::either::EitherOf5::B(view!{
                  <NavItem value="log" href="/dashboard/log">"Logs"</NavItem>
                  <NavItem value="settings" href="/dashboard/settings">"Settings"</NavItem>
                  <NavItem value="queue" href="/dashboard/queue">"Queue"</NavItem>
                  <NavItem value="users" href="/dashboard/users">"Manage Users"</NavItem>
                }),
                LoginState::User{is_admin:true,..} => leptos::either::EitherOf5::C(view!{
                  <NavItem value="log" href="/dashboard/log">"Logs"</NavItem>
                  <NavItem value="settings" href="/dashboard/settings">"Settings"</NavItem>
                  <NavItem value="queue" href="/dashboard/queue">"Queue"</NavItem>
                  <NavItem value="archives" href="/dashboard/archives">"My Archives"</NavItem>
                }),
                LoginState::User{..} => leptos::either::EitherOf5::D(view!{
                    <NavItem value="archives" href="/dashboard/archives">"My Archives"</NavItem>
                }),
                LoginState::None | LoginState::Loading => leptos::either::EitherOf5::E(())
            }}
        </NavDrawer>
    }
}

fn user_field() -> impl IntoView {
    use immt_web_utils::components::{Spinner,SpinnerSize};
    let theme = expect_context::<RwSignal::<thaw::Theme>>();
    let on_select = move |key: String| match key.as_str() {
        "theme" => {
            theme.update(|v| {
                if v.name == "dark" {
                    *v = thaw::Theme::light();
                } else {
                    *v = thaw::Theme::dark();
                }
            });
        }
        _ => unreachable!()
    };
    let src = Memo::new(|_| match LoginState::get() {
      LoginState::User{avatar,..} => Some(avatar),
      LoginState::Admin => Some("/admin.png".to_string()),
      _ => None
    });

    view!{<div class="immt-user-menu-trigger"><Menu on_select trigger_type=MenuTriggerType::Hover class="immt-user-menu">
        <MenuTrigger slot>
            <thaw::Avatar src />
        </MenuTrigger>
        // AiGitlabFilled
        {move || {
            let dark = theme.with(|v| v.name == "dark");
            let icon = if dark {icondata_bi::BiSunRegular} else {icondata_bi::BiMoonSolid};
            let text = if dark {"Light Mode"} else {"Dark Mode"};
            view!(<MenuItem value="theme" icon=icon>{text}</MenuItem>)
        }}
        <Divider/>
        {move || match LoginState::get() {
            LoginState::None => EitherOf4::A(login_form()),
            LoginState::NoAccounts => EitherOf4::B(view!(<span>"Admin"</span>)),
            LoginState::Admin => EitherOf4::C(logout_form("admin".to_string())),
            LoginState::User{name,..} => EitherOf4::C(logout_form(name)),
            LoginState::Loading => EitherOf4::D(view!(<Spinner size=SpinnerSize::Tiny/>))
        }}
    </Menu></div>}
}

fn logout_form(user:String) -> impl IntoView {
  use thaw::{Button,Input,InputType};
  let login  = expect_context::<RwSignal<LoginState>>();
  let action = Action::new(move |_| {
    login.set(LoginState::None);
    crate::users::logout()
  });
  view!(<span>{user}" "<Button on_click=move |_| {action.dispatch(());}>Logout</Button></span>)
}

fn login_form() -> impl IntoView {
  use thaw::{Button,Input,InputType};
  let pw = NodeRef::<leptos::html::Input>::new();
  let login  = expect_context();
  let action = Action::new(move |pwd:&String| {
    do_login(pwd.clone(),login)
  });
  let value = RwSignal::<String>::new(String::new());
  view!{
    <Button on_click=move |_| {action.dispatch(value.get_untracked());}>Login</Button>
    <Input placeholder="admin pwd" value input_type=InputType::Password/>
  }
}



#[allow(unused_variables)]
async fn do_login(pw:String,login:RwSignal<LoginState>) {
  let pwd = if pw.is_empty() {None} else {Some(pw)};
  match crate::users::login(pwd).await {
    Ok(Some(u@ (LoginState::Admin | LoginState::User{..}))) => login.set(u),
    Ok(_) => (),
    Err(e) => {
      #[cfg(feature="hydrate")]
      display_error(Cow::Owned(format!("Error: {e}")));
    }
  }
  let _ = view!(<Redirect path="/"/>);
}