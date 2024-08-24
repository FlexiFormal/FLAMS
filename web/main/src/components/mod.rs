pub mod mathhub_tree;
pub mod graph_viewer;
pub mod logging;
pub mod queue;
pub mod settings;
pub mod content;
pub mod query;
//mod thaws;
//pub use thaws::*;

use std::future::Future;
pub use mathhub_tree::ArchiveOrGroups;
//pub use graph_viewer::GraphTest;
pub use logging::FullLog;
pub use queue::QueuesTop;
pub use settings::Settings;
pub use query::Query;

use leptos::prelude::*;
use thaw::{BadgeAppearance,BadgeSize,BadgeColor};

#[derive(Copy,Clone,serde::Serialize,serde::Deserialize,PartialEq,Debug,Default)]
pub(crate) enum ThemeType { #[default] Light,Dark }
impl<'a> From<&'a thaw::Theme> for ThemeType {
    fn from(theme: &'a thaw::Theme) -> Self {
        if theme.name == "dark" {
            ThemeType::Dark
        } else {
            ThemeType::Light
        }
    }
}
impl Into<thaw::Theme> for ThemeType {
    fn into(self) -> thaw::Theme {
        match self {
            ThemeType::Light => thaw::Theme::light(),
            ThemeType::Dark => thaw::Theme::dark()
        }
    }
}
#[component(transparent)]
pub fn Themer(children:Children) -> impl IntoView {
    use thaw::*;
    use leptos::reactive_graph::signal::*;
    #[cfg(feature = "client")]
    use gloo_storage::Storage;
    #[cfg(feature="server")]
    let signal = RwSignal::<thaw::Theme>::new(Theme::light());
    #[cfg(feature = "client")]
    let signal = {
        let sig = gloo_storage::LocalStorage::get("theme").map(|theme:ThemeType| {
            RwSignal::<thaw::Theme>::new(theme.into())
        }).unwrap_or_else(|_| RwSignal::<thaw::Theme>::new(Theme::light()));
        Effect::new(move || {
            sig.with(move |theme| {
                gloo_storage::LocalStorage::set("theme",ThemeType::from(theme));
            })
        });
        sig
    };
    /*
    use leptos_use::storage::use_local_storage;
    let (theme_store,theme_write,_) = use_local_storage::<ThemeType,codee::string::JsonSerdeCodec>("theme");
    let signal = RwSignal::<thaw::Theme>::new(theme_store.get().unwrap_or_default().into());
    Effect::new(|| {
        signal.with(move |theme| {
            theme_write.set(theme.into());
        })
    });
    // TODO use_session_storage
     */
    provide_context(signal);
    view!{
        <ConfigProvider theme=signal><ToasterProvider>{children()}</ToasterProvider></ConfigProvider>
    }
}

#[component]
pub fn Tree(children:Children) -> impl IntoView {
    crate::css!(mathhub in "trees.css");
    view!{
        <ul class="immt-treeview">{children()}</ul>
    }
}
#[component]
pub fn Leaf(children:Children) -> impl IntoView {
    view!{
        <li class="immt-treeview-li">{children()}</li>
    }
}
#[component]
pub fn Subtree(
    #[prop(optional)] lazy:bool,
    w_header:WHeader,
    mut children:ChildrenFnMut
) -> impl IntoView {
    let expanded = RwSignal::new(false);
    view!{
        <li class="immt-treeview-li"><details>
            <summary class="immt-treeview-summary" on:click=move |_| {expanded.update(|b| *b = !*b)}>
                {(w_header.children)()}
            </summary>
        <Tree>{if !lazy {children().into_any()} else {
            (move || if expanded.get() {
                let children = children();
                Some(children)
            } else {None}).into_any()
        }}</Tree>
        </details></li>
    }
}

#[component]
pub fn Collapsible(
    #[prop(optional)] lazy:bool,
    #[prop(optional)] w_header:Option<WHeader>,
    mut children:ChildrenFnMut
) -> impl IntoView {
    let expanded = RwSignal::new(false);
    view!{<details>
        <summary on:click=move |_| expanded.update(|b| *b=!*b)>{
            w_header.map(|c| (c.children)()).unwrap_or_else(|| view!(<span/>).into_any())
        }</summary>
        <div>{
            if lazy { (move || if expanded.get() {
                Some(children())
                } else { None }
            ).into_any()} else {children().into_any()}
        }</div>
    </details>}
}

#[component]
pub fn Block(
    //#[prop(optional)] collapse:Option<Collapse>,
    #[prop(optional)] w_header:Option<WHeader>,
    #[prop(optional)] header_aux:Option<HeaderAux>,
    #[prop(optional)] header_aux_2:Option<HeaderAux2>,
    #[prop(optional)] footer:Option<Footer>,
    #[prop(optional)] separator:Option<Separator>,
    children:Children
) -> impl IntoView {
    use thaw::*;
    crate::css!(block = ".immt-block-card { width:100%;margin-top:10px;margin-bottom:10px } .immt-block-card-inner {margin:0 !important;}");
    let has_header = w_header.is_some() || header_aux.is_some() || header_aux_2.is_some();
    let has_separator = has_header || separator.is_some();
    let expanded = RwSignal::new(false);
    view!{
        <Card class="immt-block-card">
            {if has_header {
                CardHeader(CardHeaderProps{
                    class:Option::<String>::None.into(),
                    card_header_action:header_aux.map(|c| CardHeaderAction{children:c.children}),
                    card_header_description:header_aux_2.map(|c| CardHeaderDescription{children:c.children}),
                    children:w_header.map(|c| c.children).unwrap_or_else(|| Box::new(|| view!(<span/>).into_any()))
                }).into_any()
            } else {"".into_any()}}
            {if has_separator {
                Some(separator.map(|c| view!(<div><Divider>{(c.children)()}</Divider></div>))
                    .unwrap_or_else(|| view!(<div><Divider/></div>)))
            } else {None}}
            <CardPreview class="immt-block-card-inner">{children()}/*{match collapse {
                Some(Collapse{children:c,lazy}) => {
                    let expanded = RwSignal::new(false);
                    view! {<details>
                        <summary on:click=move |_| expanded.update(|b| *b=!*b)>{c()}</summary>
                        {if lazy {
                            (move || if expanded.get() {Some((children)())} else {None}).into_any()
                        } else {children().into_any()}}
                    </details>}
                }.into_any(),
                _ => children().into_any()
            }}*/</CardPreview>
            {footer.map(|h| view!{
                <CardFooter>{(h.children)()}</CardFooter>
            })}
        </Card>
    }
}

#[component]
pub fn WideDrawer(
    lazy:bool,
    trigger:Trigger,
    #[prop(optional)] w_header:Option<WHeader>,
    mut children:ChildrenFnMut
) -> impl IntoView{
    crate::css!(widedrawer = ".immt-wide-drawer { z-index:5; .thaw-overlay-drawer { width: 80%; } }");
    use thaw::*;
    let open = RwSignal::new(false);
    view!{
        <span on:click=move |_| open.set(true)>{(trigger.children)()}</span>
        <OverlayDrawer class="immt-wide-drawer" open position=DrawerPosition::Right size=DrawerSize::Large>
            <DrawerHeader><DrawerHeaderTitle>
                <DrawerHeaderTitleAction slot>
                     <Button
                        appearance=ButtonAppearance::Subtle
                        on_click=move |_| open.set(false)>"x"</Button>
                </DrawerHeaderTitleAction>
                {w_header.map(|h| (h.children)())}
            </DrawerHeaderTitle></DrawerHeader>
            //<div style="padding:0 var(--spacingHorizontalXXL) var(--spacingVerticalS)">
            <DrawerBody>{move ||
                if lazy || open.get() { children().into_any()}
                else {"".into_any()}
            }</DrawerBody>//</div>
        </OverlayDrawer>
    }
}

#[slot]
pub struct Separator { children:Children }
#[slot]
pub struct Trigger { children:Children }
#[slot]
pub struct WHeader { children:Children }
#[slot]
pub struct Footer { children:Children }
#[slot]
pub struct HeaderAux { children:Children }
#[slot]
pub struct HeaderAux2 { children:Children }

#[macro_export]
macro_rules! css {
    ($id:ident in $file:literal) => {
        $crate::components::inject_css(const_format::concatcp!("immt-",stringify!($id)),include_str!($file))
    };
    ($id:ident = $content:literal) => {
        $crate::components::inject_css(const_format::concatcp!("immt-",stringify!($id)),$content)
    };
}

pub fn inject_css(id: &'static str, content: &'static str) {
    #[cfg(feature="server")]
    {
        use leptos::view;
        use leptos_meta::Style;

        let _ = view! {
            <Style id=id>
                {content}
            </Style>
        };
    }
    #[cfg(feature="client")]
    {
        use leptos::prelude::document;
        let head = document().head().expect("head does not exist");
        let style = head
            .query_selector(&format!("style#{id}"))
            .expect("query style element error");

        if style.is_some() {
            return;
        }

        let style = document()
            .create_element("style")
            .expect("create style element error");
        _ = style.set_attribute("id", &id);
        style.set_text_content(Some(content));
        _ = head.prepend_with_node_1(&style);
    }
}

pub fn inject_script(id: &'static str, path: &'static str) {
    #[cfg(feature="server")]
    {
        use leptos::view;
        use leptos_meta::Script;


        let _ = view! {
            <Script id src=path/>
        };
    }
    #[cfg(feature="client")]
    {
        use leptos::prelude::document;
        let head = document().head().expect("head does not exist");
        let style = head
            .query_selector(&format!("script#{id}"))
            .expect("query style element error");

        if style.is_some() {
            return;
        }

        let style = document()
            .create_element("script")
            .expect("create script element error");
        _ = style.set_attribute("id", &id);
        _ = style.set_attribute("src", path);
        _ = head.prepend_with_node_1(&style);
    }
}


pub fn inject_stylesheet(id:String,path:impl Into<String>) {
    let id=format!("ID{id}");
    #[cfg(feature="server")]
    {
        use leptos::view;
        use leptos_meta::Stylesheet;

        let _ = view! {
            <Stylesheet id href=path.into()/>
        };
    }
    #[cfg(feature="client")]
    {
        use leptos::prelude::document;
        let head = document().head().expect("head does not exist");
        let style = head
            .query_selector(&format!("style#{id}"))
            .expect("query style element error");

        if style.is_some() {
            return;
        }

        let style = document()
            .create_element("link")
            .expect("create link element error");
        _ = style.set_attribute("id", &id);
        _ = style.set_attribute("href", &path.into());
        _ = style.set_attribute("rel", "stylesheet");
        _ = head.prepend_with_node_1(&style);
    }
}

pub fn inject_css_string(id: impl Into<String>+std::fmt::Display, content: impl Into<String>) {
    let id=format!("ID{id}");
    #[cfg(feature="server")]
    {
        use leptos::view;
        use leptos_meta::Style;
        let content = content.into();

        let _ = view! {
            <Style id>
                {content}
            </Style>
        };
    }
    #[cfg(feature="client")]
    {
        use leptos::prelude::document;
        let head = document().head().expect("head does not exist");
        let style = head
            .query_selector(&format!("style#{id}"))
            .expect("query style element error");

        if style.is_some() {
            return;
        }

        let style = document()
            .create_element("style")
            .expect("create style element error");
        _ = style.set_attribute("id", &id);
        style.set_text_content(Some(&content.into()));
        _ = head.prepend_with_node_1(&style);
    }
}

/*
pub(crate) fn icon(icon:icondata_core::Icon) -> impl IntoView {
    icon_with_options(icon,Some("1em"),Some("1em"),None,None)
}

pub(crate) fn icon_with_options(icon:icondata_core::Icon,width:Option<&str>,height:Option<&str>,style:Option<&str>,class:Option<&str>) -> impl IntoView {
    let style = match (style,icon.style) {
        (Some(a),Some(b)) => format!("{b} {}",a),
        (Some(a),None) => a.to_string(),
        (None,Some(b)) => b.to_string(),
        (None,None) => "vertical-align:sub;".to_string(),
    };
    view! {
        <div style="display:inline-block;margin:auto">
        <svg
            x=icon.x y=icon.y style=style
            width=width.map(|w| w.to_string()) height=height.map(|w| w.to_string())
            viewBox=icon.view_box.map(|view_box| view_box.to_string())
            stroke-linecap=icon.stroke_linecap.map(|a| a.to_string())
            stroke-linejoin=icon.stroke_linejoin.map(|a| a.to_string())
            stroke-width=icon.stroke_width.map(|a| a.to_string())
            stroke=icon.stroke.map(|a| a.to_string())
            fill=icon.fill.unwrap_or("currentColor").to_string()
            inner_html=icon.data.to_string()
        ></svg></div>
    }
}

 */

#[inline(always)]
pub(crate) fn wait_blocking<T,Fut,V:IntoView + 'static>(
                        fetcher: impl Fn() -> Fut + Send + Sync + 'static,
                        f: impl (FnMut(T) -> V) + Clone + Send + Sync + 'static
) -> impl IntoView
    where
        T: Send + Sync + Clone + serde::Serialize + for<'de>serde::Deserialize<'de> + 'static,
        Fut: Future<Output = T> + Send + 'static {
    let resource = Resource::new_blocking(|| (), move |_| fetcher());
    view!{
        <Suspense fallback=|| view!(<thaw::Spinner/>)>
        <Show when=move || resource.get().is_some() fallback=|| view!(<thaw::Spinner/>)>
        {
            resource.get().map(f.clone())
        }</Show></Suspense>
    }
}

#[inline(always)]
pub(crate) fn wait<T,Fut,V:IntoView + 'static>(
    fetcher: impl Fn() -> Fut + Send + Sync + 'static,
    f: impl (FnMut(T) -> V) + Clone + Send + 'static
) -> impl IntoView
    where
        T: Send + Sync + Clone + serde::Serialize + for<'de>serde::Deserialize<'de> + 'static,
        Fut: Future<Output = T> + Send + 'static {
    let resource = Resource::new(|| (),move |_| fetcher());
    view!{
        <Suspense fallback= || view!(<thaw::Spinner/>)>{move || {
            resource.get().map(f.clone())
        }}</Suspense>
    }
}

#[component]
pub(crate) fn IFrame(src:String,#[prop(optional,into)] ht:String) -> impl IntoView {
    view!(<iframe src=format!("/{src}") style=if ht.is_empty() {
        "width:100%;border: 0;".to_string()
    } else {
        format!("width:100%;height:{ht};border: 0;")
    }></iframe>)
}