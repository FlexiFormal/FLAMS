use super::Gotto;
use super::TOCSource;
use crate::iterate;
use crate::FTMLDocumentSetup;
use flams_ontology::uris::IsNarrativeUri;
use flams_ontology::uris::NarrativeUri;
use flams_ontology::uris::SimpleUriName;
use flams_ontology::uris::{DocumentElementUri, DocumentUri};
use flams_web_utils::components::wait_local;
use flams_web_utils::{do_css, inject_css};
use leptos::prelude::*;
use leptos_posthoc::DomStringCont;

#[cfg(feature = "omdoc")]
#[component]
pub fn DocumentFromURI(
    uri: DocumentUri,
    #[prop(optional, into)] toc: TOCSource,
    #[prop(optional, into)] gottos: Vec<Gotto>,
    #[prop(optional)] omdoc: crate::components::omdoc::OMDocSource,
) -> impl IntoView {
    wait_local(
        move || {
            tracing::info!("fetching {uri}");
            let fut = crate::remote::server_config.full_doc(uri.clone());
            async move { fut.await.ok() }
        },
        move |(uri, css, html)| {
            for c in css {
                do_css(c);
            }
            view!(<DocumentString html uri toc=toc.clone() gottos=gottos.clone() omdoc=omdoc.clone()/>)
        },
        "Error loading document reference".to_string(),
    )
}

#[component]
pub fn FragmentFromURI(uri: DocumentElementUri) -> impl IntoView {
    let uricl = uri.clone();
    wait_local(
        move || {
            tracing::info!("fetching {uri}");
            let fut = crate::remote::server_config.paragraph(uri.clone());
            async move { fut.await.ok() }
        },
        move |(_, css, html)| {
            for c in css {
                do_css(c);
            }
            view!(<FragmentString html uri=uricl.clone()/>)
        },
        "Error loading document fragment".to_string(),
    )
}

#[cfg(not(feature = "omdoc"))]
#[component]
pub fn DocumentFromURI(
    uri: DocumentUri,
    #[prop(optional, into)] toc: TOCSource,
    #[prop(optional, into)] gottos: Vec<Gotto>,
) -> impl IntoView {
    wait_local(
        move || {
            tracing::info!("fetching {uri}");
            let fut = crate::remote::server_config.full_doc(uri.clone());
            async move { fut.await.ok() }
        },
        move |(uri, css, html)| {
            for c in css {
                do_css(c);
            }
            view!(<DocumentString html uri gottos=gottos.clone() toc=toc.clone()/>)
        },
        "Error loading document reference".to_string(),
    )
}

#[component]
pub fn FragmentString(
    html: String,
    #[prop(optional)] uri: Option<DocumentElementUri>,
) -> impl IntoView {
    use leptos::context::Provider;
    use leptos::either::EitherOf3;
    let name: Option<SimpleUriName> = uri
        .as_ref()
        .map(|uri| flams_utils::unwrap!(uri.name().last().parse().ok()));
    let needs_suffix = uri
        .as_ref()
        .map(|uri| !uri.name().is_simple())
        .unwrap_or_default();
    let doc = uri.as_ref().map_or_else(
        || DocumentUri::no_doc().clone(),
        |d| d.document_uri().clone(),
    );
    view! {<FTMLDocumentSetup uri=doc>{
        match name {
            Some(name) if needs_suffix => {
                let uri = flams_utils::unwrap!(uri);
                let nuri: NarrativeUri = uri.parent().map_or(uri.document_uri().clone().into(), |e| e.clone().into());
                EitherOf3::A(view!{
                    <Provider value=ForcedName(Some(name))>
                    <Provider value=nuri>
                        <DomStringCont html cont=iterate/>
                    </Provider>
                    </Provider>
                })
            },
            Some(name) => EitherOf3::B(view!{
                <Provider value=ForcedName(Some(name))>
                    <DomStringCont html cont=iterate/>
                </Provider>
            }),
            _ => EitherOf3::C(view!{
                <DomStringCont html cont=iterate/>
            })
        }
    }</FTMLDocumentSetup>}
}

#[derive(Clone, Debug, Default)]
pub struct ForcedName(Option<SimpleUriName>);
impl ForcedName {
    pub fn update(&self, uri: &DocumentElementUri) -> DocumentElementUri {
        match self.0.as_ref() {
            Some(n) => {
                let doc = uri.document_uri().clone();
                doc & uri.name().with_last_name(n)
            }
            _ => uri.clone(),
        }
    }
}

#[cfg(feature = "omdoc")]
#[component]
pub fn DocumentString(
    html: String,
    #[prop(optional)] uri: Option<DocumentUri>,
    #[prop(optional, into)] toc: TOCSource,
    #[prop(optional, into)] gottos: Vec<Gotto>,
    #[prop(optional)] omdoc: crate::components::omdoc::OMDocSource,
) -> impl IntoView {
    use thaw::Flex;
    let uri = uri.unwrap_or_else(|| DocumentUri::no_doc().clone());
    let burger = !matches!(
        (&toc, &omdoc),
        (TOCSource::None, crate::components::omdoc::OMDocSource::None)
    );
    view! {<FTMLDocumentSetup uri><Flex>
        <div><DomStringCont html cont=iterate/></div>
        {if burger {
            Some(do_toc_sidebar(toc,gottos,omdoc))
        } else {None}}
    </Flex></FTMLDocumentSetup>
    }
}

#[cfg(not(feature = "omdoc"))]
#[component]
pub fn DocumentString(
    html: String,
    #[prop(optional)] uri: Option<DocumentUri>,
    #[prop(optional, into)] toc: TOCSource,
    #[prop(optional, into)] gottos: Vec<Gotto>,
) -> impl IntoView {
    use thaw::Flex;
    let uri = uri.unwrap_or_else(DocumentUri::no_doc);
    let burger = !matches!(toc, TOCSource::None);
    view! {<FTMLDocumentSetup uri><Flex>
        <div><DomStringCont html cont=iterate/></div>
        {if burger {
            Some(do_toc_sidebar(toc,gottos))
        } else {None}}
    </Flex></FTMLDocumentSetup>
    }
}

#[cfg(feature = "omdoc")]
fn do_toc_sidebar(
    toc: TOCSource,
    gottos: Vec<Gotto>,
    omdoc: crate::components::omdoc::OMDocSource,
) -> impl IntoView {
    inject_css("ftml-toc", include_str!("./toc.css"));
    //use flams_web_utils::components::Burger;
    use flams_web_utils::components::ClientOnly;
    use thaw::{Button, ButtonShape, ButtonSize, Scrollbar};
    let visible = RwSignal::new(true);
    let display = Memo::new(move |_| {
        if visible.get() {
            "ftml-toc-visible"
        } else {
            "ftml-toc-invisible"
        }
    });

    let hl_option: RwSignal<crate::HighlightOption> = expect_context();
    let value = RwSignal::new(hl_option.get_untracked().as_str().to_string());
    Effect::new(move || {
        if let Some(v) = crate::HighlightOption::from_str(&value.get()) {
            if hl_option.get_untracked() != v {
                hl_option.set(v);
            }
        }
    });
    use thaw::Select;
    let select = move || {
        if hl_option.get() == crate::HighlightOption::None {
            None
        } else {
            Some(
                view!(<Select value default_value=value.get_untracked() size=thaw::SelectSize::Small>
            <option class="ftml-comp">{crate::HighlightOption::Colored.as_str()}</option>
            <option class="ftml-comp-subtle">{crate::HighlightOption::Subtle.as_str()}</option>
            <option>{crate::HighlightOption::Off.as_str()}</option>
        </Select>),
            )
        }
    };

    crate::components::do_toc(toc, gottos, move |v| {
        view! {<div class="ftml-toc-sidebar">
        <ClientOnly>
            //<div style="width:0;height:0;margin-left:auto;">
            //    <div style="position:fixed">
            //<div style="max-height:600px">
            //        <InlineDrawer open=visible position=DrawerPosition::Right>
            //        <DrawerBody>
                        <Button
                            //appearance=ButtonAppearance::Subtle
                            shape=ButtonShape::Circular
                            size=ButtonSize::Small
                            on_click=move |_| visible.set(!visible.get_untracked())
                        >{move || if visible.get() {"⌃"} else {"⌄"}}</Button>
                        <div class=display>
                        {select}
                        {crate::components::omdoc::do_omdoc(omdoc)}
                        <Scrollbar style="width:fit-content;max-height:575px;">{v}</Scrollbar>
                        </div>
            //        </DrawerBody>
            //        </InlineDrawer>
            //</div>
            //    </div>
            //</div>
        </ClientOnly>
        //<Burger>{crate::components::omdoc::do_omdoc(omdoc)}{v}</Burger>
        </div>}
    })
}

#[cfg(not(feature = "omdoc"))]
fn do_toc_sidebar(toc: crate::components::TOCSource, gottos: Vec<Gotto>) -> impl IntoView {
    //use flams_web_utils::components::Burger;
    use flams_web_utils::components::ClientOnly;
    use thaw::{
        Button, ButtonAppearance, ButtonShape, ButtonSize, DrawerBody, DrawerPosition,
        InlineDrawer, Scrollbar,
    };
    inject_css("ftml-toc", include_str!("./toc.css"));
    let visible = RwSignal::new(true);
    let display = Memo::new(move |_| {
        if visible.get() {
            "ftml-toc-visible"
        } else {
            "ftml-toc-invisible"
        }
    });

    let hl_option: RwSignal<crate::HighlightOption> = expect_context();
    let value = RwSignal::new(hl_option.get_untracked().as_str().to_string());
    Effect::new(move || {
        if let Some(v) = crate::HighlightOption::from_str(&value.get()) {
            if hl_option.get_untracked() != v {
                hl_option.set(v);
            }
        }
    });
    use thaw::Select;
    let select = move || {
        if hl_option.get() == crate::HighlightOption::None {
            None
        } else {
            Some(view!(<Select value size=thaw::SelectSize::Small>
            <option>{crate::HighlightOption::Colored.as_str()}</option>
            <option>{crate::HighlightOption::Subtle.as_str()}</option>
            <option>{crate::HighlightOption::Off.as_str()}</option>
        </Select>))
        }
    };

    crate::components::do_toc(toc, gottos, move |v| {
        view! {<div class="ftml-toc-sidebar">
            <ClientOnly>
                <Button
                    shape=ButtonShape::Circular
                    size=ButtonSize::Small
                    on_click=move |_| visible.set(!visible.get_untracked())
                >{move || if visible.get() {"⌃"} else {"⌄"}}</Button>
                <div class=display>{select}
                <Scrollbar style="width:fit-content;max-height:575px;">{v}</Scrollbar>
                </div>
            </ClientOnly>
        </div>}
    })
}
