use flams_router_base::LoginState;
use leptos::{
    either::{Either, EitherOf4, EitherOf7},
    prelude::*,
};
use leptos_meta::Stylesheet;
use leptos_router::{
    components::{Outlet, Redirect},
    hooks::use_navigate,
};

use super::Page;
use flams_web_utils::components::Themer;

use thaw::{
    Caption1, Divider, Grid, GridItem, Layout, LayoutHeader, LayoutPosition, LayoutSider, Menu,
    MenuItem, MenuTrigger, MenuTriggerType, NavDrawer, NavItem, ToasterInjection,
};

#[cfg(feature = "hydrate")]
use flams_web_utils::components::display_error;
#[cfg(feature = "hydrate")]
use std::borrow::Cow;

fn do_main(page: Page) -> impl IntoView {
    use leptos::either::EitherOf10::*;
    let inner = || match page {
        Page::Home => A(view!(<super::index::Index/>)),
        Page::MathHub => B(view! {<flams_router_backend::components::ArchivesTop/>}),
        //Page::Graphs => view!{<GraphTest/>},
        Page::Log => C(view! {<super::logging::Logger/>}),
        Page::Queue => D(view! {<flams_router_buildqueue_components::QueuesTop/>}),
        Page::Query => E(view! {<super::query::Query/>}),
        Page::Settings => F(view! {<super::settings::Settings/>}),
        Page::MyArchives => G(view! {<flams_router_git_components::Archives/>}),
        Page::Search => H(view! {<super::search::SearchTop/>}),
        Page::Users => I(view! {<flams_router_login::components::Users/>}),
        _ => J(view!(<span>"TODO"</span>)),
        //Page::Login => view!{<LoginPage/>}
    };
    view!(<main style="height:100%">{inner()}</main>)
}

#[component(transparent)]
pub fn Dashboard() -> impl IntoView {
    view! {
      <Stylesheet id="leptos" href="/pkg/flams.css"/>
      <Outlet/>
    }
}

fn do_dashboard<V: IntoView + 'static>(f: impl FnOnce() -> V + Send + 'static) -> impl IntoView {
    use ftml_viewer_components::FTMLGlobalSetup;
    view! {
      <Themer><FTMLGlobalSetup>
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
      </FTMLGlobalSetup></Themer>
    }
}

#[component]
pub fn MainPage(page: Page) -> impl IntoView {
    do_dashboard(move || {
        view!(
          <Layout position=LayoutPosition::Absolute class="flams-main" content_style="height:100%" has_sider=true>
            <LayoutSider class="flams-menu" content_style="width:100%;height:100%">
              {side_menu(page)}
            </LayoutSider>
            <Layout>
              <div style="width:calc(100% - 10px);padding-left:5px;height:calc(100vh - 67px)">
                {do_main(page)}
                </div>
            </Layout>
          </Layout>
        )
    })
}

fn side_menu(page: Page) -> impl IntoView {
    view! {
        <NavDrawer selected_value=page.to_string() class="flams-menu-inner">
            <NavItem value="home" href="/">"Home"</NavItem>
            <NavItem value="mathhub" href="/dashboard/mathhub">"MathHub"</NavItem>
            <NavItem value="query" href="/dashboard/query">"Queries"</NavItem>
            <NavItem value="search" href="/dashboard/search">"Search Content"</NavItem>
            {move || {let s = LoginState::get(); match s {
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
            }}}
        </NavDrawer>
    }
}

fn user_field() -> impl IntoView {
    use flams_web_utils::components::ClientOnly;
    use flams_web_utils::components::{Spinner, SpinnerSize};
    use thaw::MenuPosition;
    let theme = expect_context::<RwSignal<thaw::Theme>>();
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
        _ => unreachable!(),
    };
    let src = Memo::new(|_| match LoginState::get() {
        LoginState::User { avatar, .. } => Some(avatar),
        LoginState::Admin => Some("/admin.png".to_string()),
        _ => None,
    });

    view! {<ClientOnly><div class="flams-user-menu-trigger"><Menu on_select trigger_type=MenuTriggerType::Hover position=MenuPosition::FlexibleBottom>
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
    </Menu></div></ClientOnly>}
}

fn logout_form(user: String) -> impl IntoView {
    use thaw::{Button, Input, InputType};
    let login = expect_context::<RwSignal<LoginState>>();
    let action = Action::new(move |_| {
        login.set(LoginState::None);
        flams_router_login::server_fns::logout()
    });
    view!(<span>{user}" "<Button on_click=move |_| {action.dispatch(());}>Logout</Button></span>)
}

fn login_form() -> impl IntoView {
    use thaw::{Button, Input, InputType};
    let pw = NodeRef::<leptos::html::Input>::new();
    let login = expect_context();
    let action = Action::new(move |pwd: &String| do_login(pwd.clone(), login));
    let value = RwSignal::<String>::new(String::new());
    view! {
      <Button on_click=move |_| {action.dispatch(value.get_untracked());}>Login</Button>
      <Input placeholder="admin pwd" value input_type=InputType::Password/>
    }
}

#[allow(unused_variables)]
async fn do_login(pw: String, login: RwSignal<LoginState>) {
    let pwd = if pw.is_empty() { None } else { Some(pw) };
    match flams_router_login::server_fns::login(pwd).await {
        Ok(Some(u @ (LoginState::Admin | LoginState::User { .. }))) => login.set(u),
        Ok(_) => (),
        Err(e) => {
            #[cfg(feature = "hydrate")]
            display_error(Cow::Owned(format!("Error: {e}")));
        }
    }
    let _ = view!(<Redirect path="/dashboard"/>);
}
