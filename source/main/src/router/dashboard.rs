use leptos::prelude::*;
use leptos_meta::Stylesheet;
use leptos_router::components::Outlet;
use crate::users::{Login,LoginState};

use super::Page;
use immt_web_utils::components::Themer;

use thaw::{Caption1, Divider, Grid, GridItem, Layout, LayoutHeader, LayoutPosition, LayoutSider, Menu, MenuItem, MenuTrigger, MenuTriggerType, NavDrawer, NavItem, ToasterInjection};


#[cfg(feature="hydrate")]
use std::borrow::Cow;
#[cfg(feature="hydrate")]
use immt_web_utils::components::error_toast;

fn do_main(page:Page) -> impl IntoView {
  let inner =  || match page {
    Page::Home => view!(<span>"TODO"</span>).into_any(),//view!{<HomePage/>}.into_any(),
    Page::MathHub => view!{<super::backend::ArchivesTop/>}.into_any(),
    //Page::Graphs => view!{<GraphTest/>},
    Page::Log => view!{<super::logging::Logger/>}.into_any(),
    Page::Queue => view!{<super::buildqueue::QueuesTop/>}.into_any(),
    Page::Query => view!{<super::query::Query/>}.into_any(),
    Page::Settings => view!{<super::settings::Settings/>}.into_any(),
    _ => view!(<span>"TODO"</span>).into_any(),//view!(<super::NotFound/>).into_any()
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
  view!{
    <Themer>
      <Layout position=LayoutPosition::Absolute>
        <Login>
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
        </Login>
      </Layout>
    </Themer>
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
    let login = expect_context::<RwSignal<LoginState>>();
    view!{
        <NavDrawer selected_value=page.to_string() class="immt-menu-inner">
            <NavItem value="home" href="/">"Home"</NavItem>
            <NavItem value="mathhub" href="/dashboard/mathhub">"MathHub"</NavItem>
            <NavItem value="query" href="/dashboard/query">"Queries"</NavItem>
            {move || match login.get() {
                LoginState::Admin | LoginState::NoAccounts => view!{
                    //<a href="/dashboard/graphs"><MenuItem key="graphs" label="Graphs"/></a>
                    <NavItem value="log" href="/dashboard/log">"Logs"</NavItem>
                    <NavItem value="settings" href="/dashboard/settings">"Settings"</NavItem>
                    <NavItem value="queue" href="/dashboard/queue">"Queue"</NavItem>
                }.into_any(),
                LoginState::User(..) | LoginState::None | LoginState::Loading => view!(<span/>).into_any()
            }}
        </NavDrawer>
    }
}

fn user_field() -> impl IntoView {
    let theme = expect_context::<RwSignal::<thaw::Theme>>();
    let login : RwSignal<LoginState> = expect_context();
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
        {move || match login.get() {
            LoginState::None => login_form().into_any(),
            LoginState::Admin | LoginState::NoAccounts => view!(<span>"Admin"</span>).into_any(),
            LoginState::User(user) => view!(<span>{user}</span>).into_any(),
            LoginState::Loading => view!(<thaw::Spinner size=thaw::SpinnerSize::Tiny/>).into_any()
        }}
    </Menu>}
}

fn login_form() -> impl IntoView {
    let toaster = ToasterInjection::expect_context();
    let login = expect_context::<RwSignal<LoginState>>();
    let _ = view!(<thaw::Input/><thaw::Button/>);
    let username = NodeRef::<leptos::html::Input>::new();
    let pw = NodeRef::<leptos::html::Input>::new();
    let action = Action::new(move |(name,pw):&(String,String)| {
      do_login(name.clone(),pw.clone(),login,toaster)
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

#[allow(unused_variables)]
async fn do_login(name:String,pw:String,login:RwSignal<LoginState>,toaster:thaw::ToasterInjection) {
  match crate::users::login(name, pw).await {
    Ok(u@(LoginState::Admin | LoginState::User(_))) => login.set(u),
    Ok(_) => {
      #[cfg(feature="hydrate")]
      error_toast(Cow::Borrowed("User does not exist or password incorrect"), toaster);
    }
    Err(e) => {
      #[cfg(feature="hydrate")]
      error_toast(Cow::Owned(format!("Error: {e}")),toaster);
    }
  }
}