use leptos::*;
use immt_core::uris::archives::ArchiveId;
use immt_core::narration::CSS;
use immt_core::uris::{MMTUri, NarrativeURI};

macro_rules! leptos_uri {
    (@ap $name:ident => $archive:expr, $path:expr $(,$e:expr)*) => {
        $name(None,Some($archive),Some($path),None,None,None,None,None,None $(,$e)*)
    };
    (@ap DOC $name:ident => $archive:expr, $path:expr $(,$e:expr)*) => {
        $name(None,Some($archive),Some($path),None,None,None $(,$e)*)
    };
    (@uri $name:ident => $uri:expr $(,$e:expr)*) => {
        $name(Some($uri),None,None,None,None,None,None,None,None $(,$e)*)
    };
    (@uri DOC $name:ident => $uri:expr $(,$e:expr)*) => {
        $name(Some($uri),None,None,None,None,None $(,$e)*)
    };
    (DOC $(#[$($prefix:tt)*])? $({$($annot:tt)*})? fn $name:ident($($(#[$($attr:tt)*])? $arg:ident:$tp:ty),*) $(-> {$($ret:tt)*})? = $uri:ident $code:block) => {
        $(#[$($prefix)*])?
        $($($annot)*)? fn $name(
            uri:Option<immt_core::uris::DocumentURI>,
            a:Option<immt_core::uris::ArchiveId>,
            rp:Option<String>,
            l:Option<immt_core::narration::Language>,
            d:Option<immt_core::uris::Name>,
            p:Option<immt_core::uris::Name>
            $(,$(#[$($attr)*])? $arg:$tp)*
        ) $(-> $($ret)*)? {
            let $uri = uri.or_else(|| if let Some(a) = a {
                if let Some(rp) = rp {
                    return server::from_archive_relpath(a,&rp)
                }
                if let Some(d) = d {server::get_doc_uri(a,p,l.unwrap_or(immt_core::narration::Language::English),d)} else {None}
            } else {None});
            $code
        }
    };
    ($(#[$($prefix:tt)*])? $({$($annot:tt)*})? fn $name:ident($($(#[$($attr:tt)*])? $arg:ident:$tp:ty),*) $(-> {$($ret:tt)*})? = $uri:ident $code:block) => {
        $(#[$($prefix)*])?
        $($($annot)*)? fn $name(
            uri:Option<immt_core::uris::MMTUri>,
            a:Option<immt_core::uris::ArchiveId>,
            rp:Option<String>,
            l:Option<immt_core::narration::Language>,
            d:Option<immt_core::uris::Name>,
            e:Option<immt_core::uris::Name>,
            m:Option<immt_core::uris::Name>,
            c:Option<immt_core::uris::Name>,
            p:Option<immt_core::uris::Name>
            $(,$(#[$($attr)*])? $arg:$tp)*
        ) $(-> $($ret)*)? {
            let $uri = uri.or_else(|| if let Some(a) = a {
                if let Some(rp) = rp {
                    return server::from_archive_relpath(a,&rp).map(|r| r.into())
                }
                server::get_uri(a,p,l,d,e,m,c)
            } else {None});
            $code
        }
    };
    (DOC @opt $(#[$($prefix:tt)*])? $({$($annot:tt)*})? fn $name:ident($($(#[$($attr:tt)*])? $arg:ident:$tp:ty),*) $(-> {$($ret:tt)*})? = $uri:ident $code:block) => {
        $(#[$($prefix)*])?
        $($($annot)*)? fn $name(
            #[prop(optional)] uri:Option<immt_core::uris::DocumentURI>,
            #[prop(optional)] a:Option<immt_core::uris::ArchiveId>,
            #[prop(optional)] rp:Option<String>,
            #[prop(optional)] l:Option<immt_core::narration::Language>,
            #[prop(optional)] d:Option<immt_core::uris::Name>,
            #[prop(optional)] p:Option<immt_core::uris::Name>
            $(,$(#[$($attr)*])? $arg:$tp)*
        ) $(-> $($ret)*)? {
            let $uri = uri.or_else(|| if let Some(a) = a {
                #[cfg(feature="server")]
                {if let Some(rp) = rp {
                    return server::from_archive_relpath(a,&rp)
                }
                if let Some(d) = d {server::get_doc_uri(a,p,l.unwrap_or(immt_core::narration::Language::English),d)} else {None}
                }
                #[cfg(feature="client")]
                {None}
            } else {None});
            $code
        }
    };
    (@opt $(#[$($prefix:tt)*])? $({$($annot:tt)*})? fn $name:ident($($(#[$($attr:tt)*])? $arg:ident:$tp:ty),*) $(-> {$($ret:tt)*})? = $uri:ident $code:block) => {
        $(#[$($prefix)*])?
        $($($annot)*)? fn $name(
            #[prop(optional)] uri:Option<immt_core::uris::MMTUri>,
            #[prop(optional)] a:Option<immt_core::uris::ArchiveId>,
            #[prop(optional)] rp:Option<String>,
            #[prop(optional)] l:Option<immt_core::narration::Language>,
            #[prop(optional)] d:Option<immt_core::uris::Name>,
            #[prop(optional)] e:Option<immt_core::uris::Name>,
            #[prop(optional)] m:Option<immt_core::uris::Name>,
            #[prop(optional)] c:Option<immt_core::uris::Name>,
            #[prop(optional)] p:Option<immt_core::uris::Name>
            $(,$(#[$($attr)*])? $arg:$tp)*
        ) $(-> $($ret)*)? {
            let $uri = uri.or_else(|| if let Some(a) = a {
                if let Some(rp) = rp {
                    return server::from_archive_relpath(a,&rp).map(|r| r.into())
                }
                server::get_uri(a,p,l,d,e,m,c)
            } else {None});
            $code
        }
    };
}

#[cfg(feature="server")]
pub(crate) mod server {
    use leptos::*;
    use leptos_router::ParamsMap;
    use immt_core::content::Term;
    use immt_core::uris::{MMTUri, Name, NarrativeURI, ModuleURI, DocumentURI, ArchiveURI};
    use immt_core::narration::{CSS, DocumentElement, Language};
    use immt_core::uris::archives::ArchiveId;
    trait MapLike {
        fn get(&self, key: &str) -> Option<&str>;
    }

    impl MapLike for ParamsMap {
        fn get(&self,key:&str) -> Option<&str> {
            self.get(key).map(|s| s.as_str())
        }
    }
    impl MapLike for std::collections::HashMap<String,String> {
        fn get(&self,key:&str) -> Option<&str> {
            self.get(key).map(|s| s.as_str())
        }
    }

    pub(crate) fn get_uri(a:ArchiveId,p:Option<Name>,l:Option<Language>,d:Option<Name>,e:Option<Name>,m:Option<Name>,c:Option<Name>) -> Option<immt_core::uris::MMTUri> {
        use immt_controller::{ControllerTrait,controller};
        use immt_api::backend::archives::Storage;

        if let Some(m) = m {
            if d.is_some() {return None}
            if e.is_some() {return None}
            let controller = controller();
            let a = controller.archives().find(a,|a| a.map(|a| a.uri()))?;
            let m = ModuleURI::new(a,p,m, l.unwrap_or(Language::English));
            Some(MMTUri::Content(match c {
                Some(n) => immt_core::uris::ContentURI::Symbol(immt_core::uris::symbols::SymbolURI::new(m,n)),
                None => immt_core::uris::ContentURI::Module(m)
            }))
        } else if let Some(d) = d {
            if m.is_some() {return None}
            if c.is_some() {return None}
            let controller = controller();
            let a = controller.archives().find(a,|a| a.map(|a| a.uri()))?;
            let d = DocumentURI::new(a,p,d, l.unwrap_or(Language::English));
            Some(MMTUri::Narrative(match e {
                Some(n) => NarrativeURI::Decl(immt_core::uris::NarrDeclURI::new(d,n)),
                None => NarrativeURI::Doc(d)
            }))
        } else {
            if e.is_some() {return None}
            if c.is_some() {return None}
            let controller = controller();
            let a = controller.archives().find(a,|a| a.map(|a| a.uri()))?;
            Some(MMTUri::Archive(a))
        }
    }


    pub(crate) fn get_doc_uri(a:ArchiveId,p:Option<Name>,l:Language,d:Name) -> Option<DocumentURI> {
        use immt_controller::{ControllerTrait,controller};
        use immt_api::backend::archives::Storage;
        let controller = controller();
        let a = controller.archives().find(a,|a| a.map(|a| a.uri()))?;
        let d = DocumentURI::new(a,p,d,l);
        Some(d)
    }

    pub(crate) fn from_archive_relpath(a:ArchiveId,rp:&str) -> Option<DocumentURI> {
        let (p,n) = if let Some((p,n)) = rp.rsplit_once('/') {
            (Some(Name::new(p)),n)
        } else {
            (None,rp)
        };
        let (n,l) = if let Some((n,l)) = n.rsplit_once('.') {
            if let Some(l) = Language::try_from(l).ok() {
                (Name::new(n),Some(l))
            } else if let Some((n,l)) = n.rsplit_once('.') {
                (Name::new(n),Language::try_from(l).ok())
            } else {
                (Name::new(n),None)
            }
        } else {
            (Name::new(n),None)
        };
        return get_doc_uri(a,p,l.unwrap_or(Language::English),n)
    }

    #[inline]
    pub(crate) fn from_params(p:&impl MapLike) -> Option<MMTUri> {
        if let Some(uri) = p.get("uri") {
            return uri.parse().ok()
        }
        let a = ArchiveId::new(p.get("a")?);
        if let Some(rp) = p.get("rp") {
            return from_archive_relpath(a,rp).map(|r| r.into())
        }
        let l = p.get("l").map(|s| Language::try_from(s.as_ref()).ok()).flatten();
        let d = p.get("d").map(Name::new);
        let e = p.get("e").map(Name::new);
        let m = p.get("m").map(Name::new);
        let c = p.get("c").map(Name::new);
        let p = p.get("p").map(Name::new);
        get_uri(a,p,l,d,e,m,c)
    }


    pub(crate) async fn get_html(axum::extract::Query(params) : axum::extract::Query<std::collections::HashMap<String,String>>) -> axum::response::Html<String> {
        use immt_api::controller::Controller;
        const FAIL: &str = "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"UTF-8\">\n  <title>SHTML</title>\n</head>\n<body>\nDocument not found\n</body>";
        struct CSSWrap(Vec<CSS>);
        let uri = if let Some(MMTUri::Narrative(NarrativeURI::Doc(d))) = from_params(&params) {d} else {return axum::response::Html(FAIL.to_string())};
        impl std::fmt::Display for CSSWrap{
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                for css in &self.0 {
                    match css {
                        CSS::Link(href) => write!(f,"\n  <link rel=\"stylesheet\" href=\"{}\"/>",href),
                        CSS::Inline(content) => write!(f,"\n  <style>\n{}\n  </style>",content)
                    }?;
                }
                Ok(())
            }
        }
        if let Some((css,s)) = immt_controller::controller().archives().get_html_async(uri).await {
            let s = format!("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"UTF-8\">\n  <title>SHTML</title>\n  <script src=\"/shtml.js\"></script>{}\n</head>\n<body>\n{}\n</body>",
                            CSSWrap(css),s
            );
            axum::response::Html(s)
        } else {
            axum::response::Html(FAIL.to_string())
        }
    }

    pub(super) fn do_doc_elem(e:&DocumentElement,in_structure:bool) -> impl IntoView {
        use thaw::*;
        match e {
            DocumentElement::SetSectionLevel(_) | DocumentElement::Definiendum{..} => None,
            DocumentElement::Module(m) => {
                let v = m.children.iter().map(|e| do_doc_elem(e,false)).collect::<Vec<_>>();
                Some(view!{
                    <Card title=format!("Module {}",m.name)>{v}</Card>
                }.into_view())
            },
            DocumentElement::MathStructure(m) => {
                let v = m.children.iter().map(|e| do_doc_elem(e,true)).collect::<Vec<_>>();
                Some(view!{
                    <Card title=format!("Structure {}",m.name)>{v}</Card>
                }.into_view())
            },
            DocumentElement::Paragraph(p) => {
                let v = p.children.iter().map(|e| do_doc_elem(e,in_structure)).collect::<Vec<_>>();
                let fors = p.fors.iter().map(|u|
                    view!(<pre>{u.to_string()}</pre><br/>)).collect::<Vec<_>>();
                let tms = p.terms.iter().map(|(_,u)|
                    view!(<pre>{do_term(u)}</pre><br/>)).collect::<Vec<_>>();
                Some(view!{
                    <Card title=format!("{}",p.kind)><span>{fors}{tms}</span>{v}</Card>
                }.into_view())
            }
            DocumentElement::ConstantDecl(c) if in_structure =>
                Some(view!{<Card title=format!("Field {}",c.name())><span/></Card>}),
            DocumentElement::ConstantDecl(c) =>
                Some(view!{<Card title=format!("Constant {}",c.name())><span/></Card>}),
            DocumentElement::TopTerm(t) => {
                let s = do_term(t);
                Some(view!{<Card title="Expression">{s}</Card>}.into_view())
            }
            _ => Some(view!(<div>{format!("TODO: {e:?}")}</div>).into_view())
        }
    }
    pub(super) fn do_term(t:&Term) -> impl IntoView {

        view!(<div>"TODO: Term"</div>)
    }
}

leptos_uri!{DOC
#[server(
    prefix="/content",
    endpoint="inner",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
)]
 {pub async} fn get_html_inner() -> {Result<(Vec<CSS>, String),ServerFnError<String>>} = uri {
    use immt_api::controller::Controller;
    let uri = if let Some(d) = uri {
        d
    } else {return Err(ServerFnError::WrappedServerError("Invalid URI".to_string())) };
    immt_controller::controller().archives().get_html_async(uri).await
        .ok_or_else(|| ServerFnError::WrappedServerError("Document not found!".to_string()))
}}

#[component]
pub(crate) fn SomeUri() -> impl IntoView {
    #[cfg(feature="server")]
    {
        use leptos_router::*;
        use server::*;
        use immt_core::uris::*;
        let params = use_query_map();
        move || {
            let uri = params.with(|p| from_params(p));
            uri.map(|uri| match uri {
                MMTUri::Narrative(NarrativeURI::Doc(d)) => view!(<Document uri=d/>),
                _ => todo!()
            })
        }
    }
    #[cfg(feature="client")]
    view!(<div/>)
}

#[component]
fn DocumentOMDoc(uri:immt_core::uris::DocumentURI) -> impl IntoView {
    use thaw::*;
    #[cfg(feature="server")]
    {
        use immt_api::controller::Controller;
        let res = create_resource(|| (),move |_| immt_controller::controller().archives().get_document_from_uri_async(uri));
        return view!{<Suspense>{move || {
            match res.get() {
                Some(Some(d)) => view!{
                    <h2>
                        "["{uri.archive().id().to_string()}"]"
                        {uri.path().map(|p| format!("/{p}/"))}
                        {uri.name().to_string()}
                    </h2>
                    <div style="text-align:left;"><Space vertical=true>{
                        d.elements.iter().map(|e| server::do_doc_elem(e,false)).collect::<Vec<_>>()
                    }</Space></div>
                }.into_view(),
                _ => view!(<span>"Document not found"</span>).into_view(),
            }
        }}</Suspense>}
    }
    #[cfg(feature="client")]
    view!(<div/>)
}


leptos_uri!{DOC @opt
#[component]
{pub(crate)} fn Document() -> {impl IntoView} = uri {
    use thaw::*;
    if let Some(uri) = uri {
    let res = create_blocking_resource(|| (),move |_| leptos_uri!(@uri DOC get_html_inner => uri.into()));
    Some(view!{
        <Suspense fallback=|| view!(<Spinner/>)>{
            if let Some(Ok((css,html))) = res.get() {
                Some(view!(<Layout style="height:100vh" content_style="height:100%">
                    <OMDocDrawer><DocumentOMDoc uri/></OMDocDrawer>
                    <CSSHTML css html/>
                    /*<Layout has_sider=true>
                        <LayoutSider></LayoutSider>
                        <Layout></Layout>
                    </Layout>*/
                </Layout>))
            } else {None}
        }</Suspense>
    })
            } else {None}
}}

#[island]
fn OMDocDrawer(children: Children) -> impl IntoView {
    use thaw::*;
    let show = create_rw_signal(false);
    view!(
        <Button on_click=move |_| show.set(true) style="position:absolute;top:0;right:0;">OMDoc</Button>
        <Drawer title="OMDoc".to_string() show placement=DrawerPlacement::Right mount=DrawerMount::None width="80%">
        {children()}
    </Drawer>)
}
/*
leptos_uri!{
    #[component]
    {pub(crate)} fn SHtmlIFrame(#[prop(optional)] ht:String) -> {impl IntoView} = uri {
        uri.map(|uri| {
            view!(<iframe src=format!("/content/html?uri={}",uri)
                style=if ht.is_empty() {
                    "width:100%;border: 0;".to_string()
                } else {
                    format!("width:100%;height:{ht};border: 0;")
                }
            ></iframe>).into_view()
        })
    }
}

 */

/*leptos_uri!{
    #[component]
    {pub(crate)} fn SHtmlIFrame(#[prop(optional)] ht:String) -> {impl IntoView} = uri {
        uri.map(|uri| {
            view!(<iframe src=format!("/{uri}")
                style=if ht.is_empty() {
                    "width:100%;border: 0;".to_string()
                } else {
                    format!("width:100%;height:{ht};border: 0;")
                }
            ></iframe>).into_view()
        })
    }
}*/

#[component]
fn CSSHTML(css:Vec<CSS>,html:String) -> impl IntoView {
    use leptos_meta::{Stylesheet,Style,Script};
    view!(<For each=move || css.clone().into_iter().enumerate() key=|(u,_)| u.to_string()
        children = move |(u,css)| match css {
            CSS::Link(href) => view!(<Stylesheet href/>),
            CSS::Inline(content) => view!(<Style>{content}</Style>)
        }
        />
        <Script src="/shtml.js"/>
        <div style="text-align:left;" inner_html=html/>
    )
}

#[component]
pub(crate) fn SHtmlComponent(archive:ArchiveId,path:String) -> impl IntoView {
    use thaw::*;
    let res = create_resource(|| (),move |_| leptos_uri!(@ap DOC get_html_inner => archive,path.clone()));
    view!{
        <Suspense fallback=|| view!(<Spinner/>)>{
            if let Some(Ok((css,html))) = res.get() {
                view!(<CSSHTML css html/>).into_view()
            } else {view!(<span/>).into_view()}
        }</Suspense>
    }
}