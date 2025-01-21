use leptos::prelude::*;
use immt_ontology::uris::DocumentURI;
use super::TOCSource;
use immt_web_utils::components::wait;
use immt_web_utils::do_css;
use crate::SHTMLDocumentSetup;
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
    view! {<SHTMLDocumentSetup uri>
        {if burger {Some(
            do_burger(toc,omdoc)
        )} 
        else { None }}
        <DomStringCont html cont=iterate/>
    </SHTMLDocumentSetup>}
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
    view! {<SHTMLDocumentSetup uri>
        {if burger {Some(
            do_burger(toc)
        )} 
        else { None }}
        <DomStringCont html cont=iterate/>
    </SHTMLDocumentSetup>}
}


#[cfg(feature="omdoc")]
fn do_burger(
    toc:TOCSource,
    omdoc:crate::components::omdoc::OMDocSource
) -> impl IntoView {
    use immt_web_utils::components::Burger;
    crate::components::do_toc(toc,move |v| view!{
        <Burger>{crate::components::omdoc::do_omdoc(omdoc)}{v}</Burger>
    })
}

#[cfg(not(feature="omdoc"))]
fn do_burger(
    toc:crate::components::TOCSource
) -> impl IntoView {
    use immt_web_utils::components::Burger;
    crate::components::do_toc(toc,move |v| view!{
        <Burger>{v}</Burger>
    })
}