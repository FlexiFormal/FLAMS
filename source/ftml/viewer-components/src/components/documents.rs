use super::Gotto;
use super::TOCSource;
use crate::iterate;
use crate::FTMLDocumentSetup;
use flams_ontology::uris::{DocumentElementURI, DocumentURI, NameStep};
use flams_web_utils::components::wait;
use flams_web_utils::do_css;
use leptos::prelude::*;
use leptos_dyn_dom::DomStringCont;

#[cfg(feature = "omdoc")]
#[component]
pub fn DocumentFromURI(
    uri: DocumentURI,
    #[prop(optional, into)] toc: TOCSource,
    #[prop(optional, into)] gottos: Vec<Gotto>,
    #[prop(optional)] omdoc: crate::components::omdoc::OMDocSource,
) -> impl IntoView {
    wait(
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
pub fn FragmentFromURI(uri: DocumentElementURI) -> impl IntoView {
    let uricl = uri.clone();
    wait(
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
    uri: DocumentURI,
    #[prop(optional, into)] toc: TOCSource,
    #[prop(optional, into)] gottos: Vec<Gotto>,
) -> impl IntoView {
    wait(
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
    #[prop(optional)] uri: Option<DocumentElementURI>,
) -> impl IntoView {
    use leptos::context::Provider;
    let name = uri.as_ref().map(|uri| uri.name().last_name().clone());
    let uri = uri.map_or_else(DocumentURI::no_doc, |d| d.document().clone());
    if let Some(name) = name {
        leptos::either::Either::Left(view! {<FTMLDocumentSetup uri>
            <Provider value=ForcedName(name)>
            <DomStringCont html cont=iterate/>
            </Provider>
        </FTMLDocumentSetup>})
    } else {
        leptos::either::Either::Right(view! {<FTMLDocumentSetup uri>
            <DomStringCont html cont=iterate/>
        </FTMLDocumentSetup>})
    }
}

#[derive(Clone, Debug)]
pub struct ForcedName(NameStep);
impl ForcedName {
    pub fn update(&self, uri: &DocumentElementURI) -> DocumentElementURI {
        let name = uri.name().clone();
        let doc = uri.document().clone();
        doc & name.with_last_name(self.0.clone())
    }
}

#[cfg(feature = "omdoc")]
#[component]
pub fn DocumentString(
    html: String,
    #[prop(optional)] uri: Option<DocumentURI>,
    #[prop(optional, into)] toc: TOCSource,
    #[prop(optional, into)] gottos: Vec<Gotto>,
    #[prop(optional)] omdoc: crate::components::omdoc::OMDocSource,
) -> impl IntoView {
    use flams_web_utils::components::ClientOnly;
    let uri = uri.unwrap_or_else(DocumentURI::no_doc);
    let burger = !matches!(
        (&toc, &omdoc),
        (TOCSource::None, crate::components::omdoc::OMDocSource::None)
    );
    view! {<FTMLDocumentSetup uri>
        {
            if burger {Some(
                do_burger(toc,gottos,omdoc)
            )}
            else { None }
        }
        <DomStringCont html cont=iterate/>
    </FTMLDocumentSetup>}
}

#[cfg(not(feature = "omdoc"))]
#[component]
pub fn DocumentString(
    html: String,
    #[prop(optional)] uri: Option<DocumentURI>,
    #[prop(optional, into)] toc: TOCSource,
    #[prop(optional, into)] gottos: Vec<Gotto>,
) -> impl IntoView {
    let uri = uri.unwrap_or_else(DocumentURI::no_doc);
    let burger = !matches!(toc, TOCSource::None);
    view! {<FTMLDocumentSetup uri>
        {if burger {Some(
            do_burger(toc,gottos)
        )}
        else { None }}
        <DomStringCont html cont=iterate/>
    </FTMLDocumentSetup>}
}

#[cfg(feature = "omdoc")]
fn do_burger(
    toc: TOCSource,
    gottos: Vec<Gotto>,
    omdoc: crate::components::omdoc::OMDocSource,
) -> impl IntoView {
    use flams_web_utils::components::Burger;
    //use flams_web_utils::components::ClientOnly;
    crate::components::do_toc(toc, gottos, move |v| {
        view! {
            /*<ClientOnly>
                <div style="width:0;height:0;margin-left:auto;">
                    <div style="position:fixed">
                        {crate::components::omdoc::do_omdoc(omdoc)}
                        <div style="width:fit-content;height:fit-content;">{v}</div>
                    </div>
                </div>
            </ClientOnly>*/
            <Burger>{crate::components::omdoc::do_omdoc(omdoc)}{v}</Burger>
        }
    })
}

#[cfg(not(feature = "omdoc"))]
fn do_burger(toc: crate::components::TOCSource, gottos: Vec<Gotto>) -> impl IntoView {
    use flams_web_utils::components::Burger;
    //use flams_web_utils::components::ClientOnly;
    crate::components::do_toc(toc, gottos, move |v| {
        view! {
           //<ClientOnly> <div>{v}</div></ClientOnly>
            <Burger>{v}</Burger>
        }
    })
}
