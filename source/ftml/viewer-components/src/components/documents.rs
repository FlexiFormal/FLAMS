use leptos::prelude::*;
use flams_ontology::uris::{DocumentElementURI, DocumentURI, Name, NameStep};
use super::TOCSource;
use flams_web_utils::components::wait;
use flams_web_utils::do_css;
use crate::FTMLDocumentSetup;
use leptos_dyn_dom::DomStringCont;
use crate::iterate;


#[cfg(feature="omdoc")]
#[component]
pub fn DocumentFromURI(
    uri:DocumentURI,
    #[prop(optional,into)] toc:TOCSource,
    #[prop(optional)] omdoc:crate::components::omdoc::OMDocSource
) -> impl IntoView {
    wait(
        move || {
            tracing::info!("fetching {uri}");
            let fut = crate::remote::server_config.full_doc(uri.clone());
            async move {fut.await.ok()}
        }, 
        move |(uri,css,html)| {
            for c in css { do_css(c); }
            view!(<DocumentString html uri toc=toc.clone() omdoc=omdoc.clone()/>)
        }, 
        "Error loading document reference".to_string(),
    )
}

#[component]
pub fn FragmentFromURI(
    uri:DocumentElementURI,
) -> impl IntoView {
    let uricl = uri.clone();
    wait(
        move || {
            tracing::info!("fetching {uri}");
            let fut = crate::remote::server_config.paragraph(uri.clone());
            async move {fut.await.ok()}
        }, 
        move |(_,css,html)| {
            for c in css { do_css(c); }
            view!(<FragmentString html uri=uricl.clone()/>)
        }, 
        "Error loading document fragment".to_string(),
    )
}

#[cfg(not(feature="omdoc"))]
#[component]
pub fn DocumentFromURI(
    uri:DocumentURI,
    #[prop(optional,into)] toc:TOCSource
) -> impl IntoView {

    wait(
        move || {
            tracing::info!("fetching {uri}");
            let fut = crate::remote::server_config.full_doc(uri.clone());
            async move {fut.await.ok()}
        }, 
        move |(uri,css,html)| {
            for c in css { do_css(c); }
            view!(<DocumentString html uri toc=toc.clone()/>)
        }, 
        "Error loading document reference".to_string(),
    )
}

#[cfg(feature="omdoc")]
#[component]
pub fn DocumentString(
    html:String,
    #[prop(optional)] uri:Option<DocumentURI>,
    #[prop(optional,into)] toc:TOCSource,
    #[prop(optional)] omdoc:crate::components::omdoc::OMDocSource
) -> impl IntoView {

    let uri = uri.unwrap_or_else(DocumentURI::no_doc);
    let burger = !matches!((&toc,&omdoc),(TOCSource::None,crate::components::omdoc::OMDocSource::None));
    view! {<FTMLDocumentSetup uri>
        {if burger {Some(
            do_burger(toc,omdoc)
        )} 
        else { None }}
        <DomStringCont html cont=iterate/>
    </FTMLDocumentSetup>}
}

#[component]
pub fn FragmentString(
    html:String,
    #[prop(optional)] uri:Option<DocumentElementURI>,
) -> impl IntoView {
    use leptos::context::Provider;
    let name = uri.as_ref().map(|uri| uri.name().last_name().clone());
    let uri = uri.map_or_else(DocumentURI::no_doc,|d| d.document().clone());
    if let Some(name) = name { leptos::either::Either::Left(
        view!{<FTMLDocumentSetup uri>
            <Provider value=ForcedName(name)>
            <DomStringCont html cont=iterate/>
            </Provider>
        </FTMLDocumentSetup>}
    )} else {
        leptos::either::Either::Right(
            view!{<FTMLDocumentSetup uri>
                <DomStringCont html cont=iterate/>
            </FTMLDocumentSetup>}
        )
    }
}

#[derive(Clone,Debug)]
pub struct ForcedName(NameStep);
impl ForcedName {
    pub fn update(&self,uri:&DocumentElementURI) -> DocumentElementURI {
        let name = uri.name().clone();
        let doc = uri.document().clone();
        doc & name.with_last_name(self.0.clone())
    }
}


#[cfg(not(feature="omdoc"))]
#[component]
pub fn DocumentString(
    html:String,
    #[prop(optional)] uri:Option<DocumentURI>,
    #[prop(optional,into)] toc:TOCSource
) -> impl IntoView {
    let uri = uri.unwrap_or_else(DocumentURI::no_doc);
    let burger = !matches!(toc,TOCSource::None);
    view! {<FTMLDocumentSetup uri>
        {if burger {Some(
            do_burger(toc)
        )} 
        else { None }}
        <DomStringCont html cont=iterate/>
    </FTMLDocumentSetup>}
}


#[cfg(feature="omdoc")]
fn do_burger(
    toc:TOCSource,
    omdoc:crate::components::omdoc::OMDocSource
) -> impl IntoView {
    use flams_web_utils::components::Burger;
    crate::components::do_toc(toc,move |v| view!{
        <Burger>{crate::components::omdoc::do_omdoc(omdoc)}{v}</Burger>
    })
}

#[cfg(not(feature="omdoc"))]
fn do_burger(
    toc:crate::components::TOCSource
) -> impl IntoView {
    use flams_web_utils::components::Burger;
    crate::components::do_toc(toc,move |v| view!{
        <Burger>{v}</Burger>
    })
}