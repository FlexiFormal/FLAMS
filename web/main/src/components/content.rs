use leptos::*;
use immt_core::uris::archives::ArchiveId;
use immt_core::narration::CSS;

#[cfg(feature="server")]
pub(crate) mod server {
    use leptos::*;
    use leptos_router::ParamsMap;
    use immt_core::content::Term;
    use immt_core::uris::{MMTUri, Name, NarrativeURI, ModuleURI, DocumentURI, ArchiveURI};
    use immt_core::narration::{CSS, DocumentElement, Language};
    use immt_core::uris::archives::ArchiveId;

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
    #[inline]
    pub(crate) fn from_params(params:&Memo<ParamsMap>) -> Option<MMTUri> {
        params.with(|p| {
            let a = ArchiveId::new(p.get("a")?);
            let l = p.get("l").map(|s| Language::try_from(s.as_ref()).ok()).flatten();
            let d = p.get("d").map(Name::new);
            let e = p.get("e").map(Name::new);
            let m = p.get("m").map(Name::new);
            let c = p.get("c").map(Name::new);
            let p = p.get("p").map(Name::new);
            get_uri(a,p,l,d,e,m,c)
        })
    }

    #[derive(serde::Deserialize)]
    pub(crate) struct GetHTMLFullParams {a:ArchiveId, p:String}

    pub(crate) async fn get_html(axum::extract::Query(GetHTMLFullParams{a,p}) : axum::extract::Query<GetHTMLFullParams>) -> axum::response::Html<String> {
        use immt_api::controller::Controller;
        struct CSSWrap(Vec<CSS>);
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
        if let Some((css,s)) = immt_controller::controller().archives().get_html_async(a,p).await {
            let s = format!("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"UTF-8\">\n  <title>SHTML</title>\n  <script src=\"/shtml.js\"></script>{}\n</head>\n<body>\n{}\n</body>",
                            CSSWrap(css),s
            );
            axum::response::Html(s)
        } else {
            axum::response::Html(format!("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"UTF-8\">\n  <title>SHTML</title>\n</head>\n<body>\nDocument not found\n</body>",
            ))
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


#[server(
    prefix="/content",
    endpoint="body",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
)]
pub async fn get_html_body(archive:ArchiveId,rel_path:String) -> Result<(Vec<CSS>,String),ServerFnError<String>> {
    use immt_api::controller::Controller;
    immt_controller::controller().archives().get_html_async(archive,rel_path).await
        .ok_or_else(|| ServerFnError::WrappedServerError("Document not found!".to_string()))
}

#[component]
pub(crate) fn SomeUri() -> impl IntoView {
    #[cfg(feature="server")]
    {
        use leptos_router::*;
        use server::*;
        use immt_core::uris::*;
        let params = use_query_map();
        move || {
            let uri = match params.with(|p| p.get("uri").map(|s| s.parse())) {
                Some(Ok(u)) => Some(u),
                _ => from_params(&params)
            };
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
fn Document(uri:immt_core::uris::DocumentURI) -> impl IntoView {
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

#[component]
pub(crate) fn SHtmlIFrame(archive:ArchiveId,path:String,#[prop(optional)] ht:String) -> impl IntoView {
    view!(<iframe src=format!("/content/html?a={}&p={}",archive,path)
        style=if ht.is_empty() {
            "width:100%;border: 0;".to_string()
        } else {
            format!("width:100%;height:{ht};border: 0;")
        }
        ></iframe>)
}
#[island]
pub(crate) fn SHtmlIsland(archive:ArchiveId,path:String) -> impl IntoView {
    use thaw::*;
    use leptos_meta::{Stylesheet,Style,Script};
    leptos_meta::provide_meta_context();
    let res = create_resource(|| (),move |_| get_html_body(archive,path.clone()));
    view!{
        <Suspense fallback=|| view!(<Spinner/>)>{
            if let Some(Ok((css,html))) = res.get() {
                view!(<For each=move || css.clone().into_iter().enumerate() key=|(u,_)| u.to_string()
                    children = move |(_,css)| match css {
                        CSS::Link(href) => view!(<Stylesheet href/>),
                        CSS::Inline(content) => view!(<Style>{content}</Style>)
                    }
                    />
                    <Script src="/shtml.js"/>
                    <div inner_html=html />
                ).into_view()
            } else {view!(<span/>).into_view()}
        }</Suspense>
    }
}