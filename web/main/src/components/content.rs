use leptos::prelude::*;
use immt_core::content::ArgSpec;
use immt_core::narration::{CSS, StatementKind};
use immt_core::uris::{ContentURI, DocumentURI, ModuleURI, NarrDeclURI, SymbolURI, Name};
#[cfg(feature="server")]
use immt_core::{uris::{URI,NarrativeURI},narration::DocumentElement};

#[cfg(feature="server")]
mod uris {
    use leptos_router::params::ParamsMap;
    use immt_core::narration::Language;
    use immt_core::uris::{ArchiveId, ContentURI, URI, ModuleURI, Name, DocumentURI, NarrativeURI, NarrDeclURI};

    pub(crate) trait MapLike {
        fn get(&self, key: &str) -> Option<&str>;
    }
    impl MapLike for ParamsMap {
        fn get(&self,key:&str) -> Option<&str> {
            self.get_str(key)
        }
    }
    impl MapLike for std::collections::HashMap<String,String> {
        fn get(&self,key:&str) -> Option<&str> {
            self.get(key).map(|s| s.as_str())
        }
    }

    pub(crate) fn get_uri(a:ArchiveId,p:Option<Name>,l:Option<Language>,d:Option<Name>,e:Option<Name>,m:Option<Name>,c:Option<Name>) -> Option<immt_core::uris::URI> {
        use immt_controller::{ControllerTrait,controller};
        use immt_api::backend::archives::Storage;

        if let Some(m) = m {
            if d.is_some() {return None}
            if e.is_some() {return None}
            let controller = controller();
            let a = controller.backend().get_archive(a,|a| a.map(|a| a.uri()))?;
            let m = ModuleURI::new(a,p,m, l.unwrap_or(Language::English));
            Some(URI::Content(match c {
                Some(n) => ContentURI::Symbol(immt_core::uris::symbols::SymbolURI::new(m,n)),
                None => ContentURI::Module(m)
            }))
        } else if let Some(d) = d {
            if m.is_some() {return None}
            if c.is_some() {return None}
            let controller = controller();
            let a = controller.backend().get_archive(a,|a| a.map(|a| a.uri()))?;
            let d = DocumentURI::new(a,p,d, l.unwrap_or(Language::English));
            Some(URI::Narrative(match e {
                Some(n) => NarrativeURI::Decl(NarrDeclURI::new(d,n)),
                None => NarrativeURI::Doc(d)
            }))
        } else {
            if e.is_some() {return None}
            if c.is_some() {return None}
            let controller = controller();
            let a = controller.backend().get_archive(a,|a| a.map(|a| a.uri()))?;
            Some(URI::Archive(a))
        }
    }


    pub(crate) fn get_doc_uri(a:ArchiveId,p:Option<Name>,l:Language,d:Name) -> Option<DocumentURI> {
        use immt_controller::{ControllerTrait,controller};
        use immt_api::backend::archives::Storage;
        let controller = controller();
        let a = controller.backend().get_archive(a,|a| a.map(|a| a.uri()))?;
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
    pub(crate) fn from_params(p:&impl MapLike) -> Option<URI> {
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
}

macro_rules! uri {
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
                    return uris::from_archive_relpath(a,&rp)
                }
                if let Some(d) = d {uris::get_doc_uri(a,p,l.unwrap_or(immt_core::narration::Language::English),d)} else {None}
            } else {None});
            $code
        }
    };
    ($(#[$($prefix:tt)*])? $({$($annot:tt)*})? fn $name:ident($($(#[$($attr:tt)*])? $arg:ident:$tp:ty),*) $(-> {$($ret:tt)*})? = $uri:ident $code:block) => {
        $(#[$($prefix)*])?
        $($($annot)*)? fn $name(
            uri:Option<immt_core::uris::URI>,
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
                    return uris::from_archive_relpath(a,&rp).map(|r| r.into());
                }
                uris::get_uri(a,p,l,d,e,m,c)
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
                    return uris::from_archive_relpath(a,&rp)
                }
                if let Some(d) = d {uris::get_doc_uri(a,p,l.unwrap_or(immt_core::narration::Language::English),d)} else {None}
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
            #[prop(optional)] uri:Option<immt_core::uris::URI>,
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
                    return uris::from_archive_relpath(a,&rp).map(|r| r.into());
                }
                server::get_uri(a,p,l,d,e,m,c)
            } else {None});
            $code
        }
    };
}

uri!{
#[server(
    prefix="",
    endpoint="fragment",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
)]
 {pub async} fn get_fragment() -> {Result<(Box<[CSS]>, Box<str>),ServerFnError<String>>} = uri {
    if let Some(d) = uri {
        match d {
            URI::Narrative(NarrativeURI::Doc(d)) =>
                fragments::document(d).await,
            URI::Narrative(NarrativeURI::Decl(d)) =>
                fragments::paragraph(d).await,
            URI::Content(ContentURI::Symbol(s)) =>
                fragments::symbol(s).await,
            _ => Ok((Default::default(),"TODO".to_string().into()))
        }
    } else {Err(ServerFnError::WrappedServerError("Invalid URI".to_string())) }
}}


mod fragments {
    use immt_core::narration::CSS;
    use immt_core::uris::{DocumentURI, NarrDeclURI, SymbolURI};
    use leptos::prelude::*;
    #[cfg(feature = "server")]
    use immt_core::narration::DocumentElement;
    #[server(
        prefix="/api/fragments",
        endpoint="document",
        input=server_fn::codec::GetUrl,
        output=server_fn::codec::Json
    )]
    pub async fn document(uri:DocumentURI) -> Result<(Box<[CSS]>, Box<str>),ServerFnError<String>> {
        use immt_api::controller::Controller;
        let backend = immt_controller::controller().backend();
        backend.get_html_async(uri).await.ok_or_else(|| ServerFnError::WrappedServerError("Document not found!".to_string()))
    }
    #[server(
        prefix="/api/fragments",
        endpoint="paragraph",
        input=server_fn::codec::GetUrl,
        output=server_fn::codec::Json
    )]
    pub async fn paragraph(uri:NarrDeclURI) -> Result<(Box<[CSS]>,Box<str>),ServerFnError<String>> {
        use immt_api::controller::Controller;
        let backend = immt_controller::controller().backend();
        let doc = backend.get_document_async(uri.doc()).await.ok_or_else(|| ServerFnError::WrappedServerError("Document not found!".to_string()))?;
        if let Some(DocumentElement::Paragraph(p)) = doc.as_ref().get(uri.name()) {
            doc.read_snippet_async(p.range).await.ok_or_else(|| ServerFnError::WrappedServerError("Paragraph not found!".to_string()))
        } else {
            Err(ServerFnError::WrappedServerError("Paragraph not found!".to_string()))
        }
    }
    #[server(
        prefix="/api/fragments",
        endpoint="document",
        input=server_fn::codec::GetUrl,
        output=server_fn::codec::Json
    )]
    pub async fn symbol(uri:SymbolURI) -> Result<(Box<[CSS]>, Box<str>),ServerFnError<String>> {
        use immt_api::controller::Controller;
        let backend = immt_controller::controller().backend();
        let (range,d) = backend.get_definitions(uri).map(|d| {
            let range = d.get().range;
            (range,d.take())
        }).next().ok_or_else(|| ServerFnError::WrappedServerError("Symbol not found!".to_string()))?;
        d.read_snippet_async(range).await.ok_or_else(|| ServerFnError::WrappedServerError("Symbol not found!".to_string()))
    }
}

pub mod omdoc {
    use leptos::prelude::*;
    use immt_core::narration::StatementKind;
    use immt_core::uris::{ContentURI, DocumentURI, ModuleURI, NamedURI, NarrDeclURI, SymbolURI};
    use crate::components::content::{HTMLConstant, HTMLDocElem, HTMLDocument};

    #[server(
        prefix="/api/omdoc",
        endpoint="document",
        input=server_fn::codec::GetUrl,
        output=server_fn::codec::Json
    )]
    pub async fn omdoc_document(uri:DocumentURI) -> Result<HTMLDocument,ServerFnError<String>> {
        use immt_api::controller::Controller;
        let backend = immt_controller::controller().backend();
        let doc = backend.get_document_async(uri).await.ok_or_else(|| ServerFnError::WrappedServerError("Document not found!".to_string()))?;
        Ok(doc.into())
    }

    #[server(
        prefix="/api/omdoc",
        endpoint="constant",
        input=server_fn::codec::GetUrl,
        output=server_fn::codec::Json
    )]
    pub async fn omdoc_constant(uri:SymbolURI) -> Result<super::HTMLConstant,ServerFnError<String>> {
        use immt_api::controller::Controller;
        use immt_core::content::ContentElement;
        let backend = immt_controller::controller().backend();
        let m = backend.get_module_async(uri.module()).await.ok_or_else(|| ServerFnError::WrappedServerError("Module not found!".to_string()))?;
        let c = m.get(uri.name()).ok_or_else(|| ServerFnError::WrappedServerError("Constant not found!".to_string()))?;
        let ContentElement::Constant(c) = c else {return Err(ServerFnError::WrappedServerError("Not a constant".to_string()))};
        let tp_html = c.tp.as_ref().map(|t| backend.display_term(t).to_string());
        let df_html = c.df.as_ref().map(|t| backend.display_term(t).to_string());
        let notations = backend.get_notations(uri).map(|(m,not)| {
            let mut op = String::new();
            let op = if let Some(Ok(_)) = not.apply_op("OMID",uri,&mut op) {
                Some(op)
            } else {None};
            let mut nots = String::new();
            let nots = if let Ok(_) = not.display(uri,&mut nots) {
                Some(nots)
            } else {None};
            (m,not.id,nots,op)
        }).collect();
        let macro_name = c.macroname;
        Ok(super::HTMLConstant{uri:c.uri,tp_html,df_html,arity:c.arity.clone(),macro_name,notations})
    }

    #[island]
    pub fn OMDocDocumentURI(uri:DocumentURI) -> impl IntoView {
        let resource = Resource::new(|| (),move |_| omdoc_document(uri));
        view!{
            <Suspense fallback=|| view!(<thaw::Spinner/>)>{
                if let Some(Ok(doc)) = resource.get() {
                    Some(view!{<OMDocDocument doc/>}.into_any())
                } else {Some(view!(<span>"Document not found"</span>).into_any())}
            }</Suspense>
        }
    }

    #[component]
    pub fn OMDocDocument(doc:HTMLDocument) -> impl IntoView {
        let uses = do_uses("Uses",doc.uses);
        let title = doc.title;
        let children = doc.children;
        view!{
            <h2 inner_html=title/>
            <div style="text-align:left;"><thaw::Space vertical=true>{uses}{
                omdoc_doc_elems(children,false)
            }</thaw::Space></div>
        }
    }

    #[island]
    pub fn OMDocConstantURI(uri:SymbolURI) -> impl IntoView {
        let resource = Resource::new(|| (),move |_| omdoc_constant(uri));
        view!{
            <Suspense fallback=|| view!(<thaw::Spinner/>)>{move || {
                if let Some(Ok(c)) = resource.get() {
                    view!{<OMDocConstant c/>}.into_any()
                } else {view!(<span>"Symbol not found"</span>).into_any()}
            }}</Suspense>
        }
    }

    const ARGS : &'static str = "abcdefghijk";
    const VARS : &'static str = "xyzvwrstu";

    #[component]
    pub fn OMDocConstant(c:HTMLConstant) -> impl IntoView {
        use immt_core::content::ArgType;
        view!{<div style="position:relative">
            {super::sym_shtml(c.uri.into())}
            {c.tp_html.map(|s| view!(" : "<math><mrow inner_html=s/></math><br/>))}
            {c.df_html.map(|s| view!(" := "<math><mrow inner_html=s/></math><br/>))}
            {c.macro_name.map(|s| {
                view!(<div style="font-family:monospace;position:absolute;top:0;right:0;">"\\"{s.to_string()}{
                    c.arity.into_iter().enumerate().map(|(i,a)| match a {
                        ArgType::Normal => view!("{"{ARGS.chars().nth(i).unwrap()}"}").into_any(),
                        ArgType::Sequence => view!("{"
                            <math><mrow><msub>
                                <mi>{ARGS.chars().nth(i).unwrap()}</mi>
                                <mn>"1"</mn>
                            </msub></mrow></math>
                            ",...,"
                            <math><mrow><msub>
                                <mi>{ARGS.chars().nth(i).unwrap()}</mi>
                                <mi>"n"</mi>
                            </msub></mrow></math>
                            "}").into_any(),
                        ArgType::Binding => view!("{"{VARS.chars().nth(i).unwrap()}"}").into_any(),
                        ArgType::BindingSequence => view!("{"
                            <math><mrow><msub>
                                <mi>{VARS.chars().nth(i).unwrap()}</mi>
                                <mn>"1"</mn>
                            </msub></mrow></math>
                            ",...,"
                            <math><mrow><msub>
                                <mi>{VARS.chars().nth(i).unwrap()}</mi>
                                <mi>"n"</mi>
                            </msub></mrow></math>
                            "}").into_any()
                    }).collect::<Vec<_>>()
                }</div>)
            })}
            {if c.notations.is_empty() {None} else {Some(
                view!(<div><b>"Notations"</b></div><table>
                    <thead><tr><th>"application"</th><th>"operator"</th><th>"id"</th><th>"in module"</th></tr></thead>
                    <tbody>{
                    c.notations.into_iter().map(|(m,n,full,op)| {view!{
                        <tr>
                            <td>{full.map(|s| view!(<math><mrow inner_html=s/></math>))}</td>
                            <td>{op.map(|s| view!(<math><mrow inner_html=s/></math>))}</td>
                            <td>{n.to_string()}</td>
                            <td><LinkedName uri=NamedURI::Content(m.into())/></td>
                        </tr>
                    }}).collect::<Vec<_>>()
                }</tbody></table>)
            )}}
        </div>}
    }

    #[inline]
    fn omdoc_doc_elems(children:Vec<HTMLDocElem>,in_structure:bool) -> impl IntoView {
        children.into_iter().map(|e| match e {
            HTMLDocElem::InputRef(uri) => view!(<OMDocInputref uri/>).into_any(),
            HTMLDocElem::Section {uri,title_html,children,uses}
                => view!(<OMDocSection title_html elems=children uri uses in_structure/>).into_any(),
            HTMLDocElem::Module {uri,children,uses, imports }
                => view!(<OMDocDocumentModule elems=children uri uses imports/>).into_any(),
            HTMLDocElem::Structure {uri,children,uses,imports}
                => view!(<OMDocDocumentStructure elems=children uri uses imports/>).into_any(),
            HTMLDocElem::Morphism {uri,domain,children,total}
                => view!(<OMDocDocumentMorphism elems=children uri domain total/>).into_any(),
            HTMLDocElem::Paragraph {uri,kind,uses,children,fors,terms_html}
                => view!(<OMDocParagraph kind uri fors terms_html uses elems=children />).into_any(),
            HTMLDocElem::Constant(uri)
                => view!(<OMDocDocumentConstant uri in_structure/>).into_any(),
            HTMLDocElem::Var {uri,tp,df}
                => view!(<OMDocVariable uri tp df/>).into_any(),
            HTMLDocElem::Term(t)
                => view!(<div>"Term "<math><mrow inner_html=t/></math></div>).into_any()
        }).collect_view()
    }

    #[component]
    pub fn OMDocVariable(uri:NarrDeclURI,tp:Option<String>,df:Option<String>) -> impl IntoView {
        view!{
            <div>
                <b>"Variable "{super::var_shtml(uri)}</b>
                {tp.map(|s| view!(" : "<math><mrow inner_html=s/></math>))}
                {df.map(|s| view!(" := "<math><mrow inner_html=s/></math>))}
            </div>
        }
    }
    #[island]
    pub fn OMDocInputref(uri:DocumentURI) -> impl IntoView {
        use thaw::*;
        let name = uri.name();
        let (expanded, set_expanded) = signal(false);
        view!{<details>
            <summary on:click=move |_| set_expanded.update(|b| *b=!*b)>
                <b>"Document "<LinkedName uri=uri.into()/></b>
            </summary>
            <Card>{move || {
                if expanded.get() {Some(view!(<OMDocDocumentURI uri/>))} else {None}
            }}</Card>
        </details>}
    }
    #[island]
    pub fn OMDocDocumentConstant(uri:SymbolURI,in_structure:bool) -> impl IntoView {
        use thaw::*;
        let name = uri.name();
        let (expanded, set_expanded) = signal(false);
        view!{<details>
            <summary on:click=move |_| set_expanded.update(|b| *b=!*b)>
                <b>{if in_structure {"Field "} else {"Symbol "}}{super::sym_shtml(uri.into())}</b>
            </summary>
            <Card>{move || {
                if expanded.get() {Some(view!(<OMDocConstantURI uri/>))} else {None}
            }}</Card>
        </details>}
    }

    #[island]
    pub fn OMDocParagraph(kind:StatementKind,uri:NarrDeclURI,fors:Vec<ContentURI>,terms_html:Vec<(SymbolURI,String)>,uses:Vec<ModuleURI>,elems:Vec<HTMLDocElem>) -> impl IntoView {
        use thaw::*;
        let name = uri.name();
        let (expanded, set_expanded) = signal(false);
        fn do_name(uri:ContentURI,terms:&Vec<(SymbolURI,String)>) -> impl IntoView {
            view!{
                {super::sym_shtml(uri)}
                {match uri {
                    ContentURI::Symbol(s) => if let Some((_,t)) = terms.iter().find(|(s2,_)| *s2 == s) {
                        Some(view!(" as "<math><mrow inner_html=t.clone()/></math>))
                    } else {None},
                    _ => None
                }}
            }
        }
        view!{
            <Card>
                <CardHeader>
                    {kind.to_string()}" "<LinkedName uri=uri.into()/>
                    <CardHeaderDescription slot>
                        {do_uses("Uses",uses)}<br/>
                        {if fors.is_empty() {None} else {
                            let mut iter = fors.iter();
                            Some(view!{<div>
                                "introduces "{do_name(*iter.next().unwrap(),&terms_html)}
                                {iter.map(|u| view!(", "{do_name(*u,&terms_html)})).collect::<Vec<_>>()}
                            </div>})
                        }}
                    </CardHeaderDescription>
                </CardHeader>
                //<div><b>{kind.to_string()}" "<LinkedName uri=uri.into()/></b></div>
                { omdoc_doc_elems(elems,false) }
                <details>
                <summary on:click=move |_| set_expanded.update(|b| *b=!*b)>
                    "Text"
                </summary>
                <Card>{move || {
                    if expanded.get() {
                        let resource = Resource::new(|| (),move |_| super::fragments::paragraph(uri));
                        Some(view!(<Suspense fallback=|| view!(<thaw::Spinner/>)>{
                            if let Some(Ok(re)) = resource.get() {
                            // TODO: CSS
                                Some(view!{<div inner_html=re.1.to_string()/>}.into_any())
                            } else {Some("Paragraph not found".into_any())}
                        }</Suspense>))
                    } else {None}
                }}</Card>
            </details>
            </Card>
        }
    }

    #[component]
    pub fn OMDocSection(uri:NarrDeclURI,title_html:String,elems:Vec<HTMLDocElem>,uses:Vec<ModuleURI>,in_structure:bool) -> impl IntoView {
        use thaw::*;
        view!{
            <Card>
                <CardHeader><UriHover uri=uri.into()>
                    <b>" - "</b><span inner_html=title_html/>
                </UriHover>
                <CardHeaderDescription slot>{do_uses("Uses",uses)}</CardHeaderDescription>
                </CardHeader>
                { omdoc_doc_elems(elems,in_structure) }
            </Card>
        }
    }

    #[component]
    pub fn OMDocDocumentModule(uri:ModuleURI, elems:Vec<HTMLDocElem>, uses:Vec<ModuleURI>, imports:Vec<ModuleURI>) -> impl IntoView {
        use thaw::*;
        view!{
            <Card>
                <CardHeader>"Module "<LinkedName uri=NamedURI::Content(uri.into())/>
                <CardHeaderDescription slot>{do_uses("Uses",uses)}<br/>{do_uses("Imports",imports)}</CardHeaderDescription>
                </CardHeader>
                { omdoc_doc_elems(elems,false) }
            </Card>
        }
    }

    #[component]
    pub fn OMDocDocumentStructure(uri:ModuleURI, elems:Vec<HTMLDocElem>, uses:Vec<ModuleURI>, imports:Vec<ModuleURI>) -> impl IntoView {
        use thaw::*;
        view!{
            <Card>
                <CardHeader>"Structure "<LinkedName uri=NamedURI::Content(uri.into())/>
                <CardHeaderDescription slot>{do_uses("Uses",uses)}<br/>{do_uses("Imports",imports)}</CardHeaderDescription>
                </CardHeader>
                { omdoc_doc_elems(elems,true) }
            </Card>
        }
    }

    #[component]
    pub fn OMDocDocumentMorphism(uri:ModuleURI,domain:ModuleURI, elems:Vec<HTMLDocElem>, total:bool) -> impl IntoView {
        format!("TODO Morphism: {uri}")
    }

    fn do_uses(s:&'static str,v:Vec<ModuleURI>) -> impl IntoView {
        if v.is_empty() {None} else {Some(view!{
        <div>
            {s}" "
        {
            let mut iter = v.iter();
            view!{
                {
                    iter.next().map(|next| {
                        view!(<LinkedName uri=NamedURI::Content((*next).into())/>).into_view()
                    })
                }
                {
                    iter.map(|u|
                        view!(", "<LinkedName uri=NamedURI::Content((*u).into())/>)
                    ).collect::<Vec<_>>()
                }
            }
        }
        </div>
    })}
    }

    #[component]
    fn LinkedName(uri: NamedURI) -> impl IntoView {
        let name = uri.name();
        let name = name.as_ref();
        let name = name.rsplit_once('/').map(|(_,n)| n).unwrap_or(name);
        let name = name.to_string();
        view!(<UriHover uri><span style="font-family:monospace">{name}</span></UriHover>)
    }

    #[component]
    fn UriHover(uri: NamedURI,children:Children) -> impl IntoView {
        use thaw::*;
        view!(
            <Popover><PopoverTrigger slot>
              <a style="color:blue" href=format!("/?uri={}",urlencoding::encode(&uri.to_string()))>{
                children()
        }</a>
            </PopoverTrigger>{uri.to_string()}
            </Popover>
        )
    }

}


#[component]
pub(crate) fn URITop() -> impl IntoView {
    #[cfg(feature="server")]
    {
        use leptos_router::*;
        use immt_core::uris::*;
        let params = leptos_router::hooks::use_query_map();
        move || {
            let uri = params.with(|p| uris::from_params(p));
            uri.map(|uri| {
                match uri {
                    URI::Narrative(NarrativeURI::Doc(d)) => view!(<Document uri=d/>).into_any(),
                    _ => view!("TODO: "{uri.to_string()}).into_any()
                }
            })
        }
    }
    #[cfg(feature="client")]
    view!(<div/>)
}

#[component]
fn Document(uri:DocumentURI) -> impl IntoView {
    #[cfg(feature="server")]
    {
        let res = Resource::new_blocking(|| (), move |_| fragments::document(uri));
        view! {
            <Suspense fallback=|| view!(<thaw::Spinner/>)>
                <Show
                    when=move || res.with(|s| s.as_ref().map(|s| s.is_ok()) == Some(true))
                    fallback=|| view!(<span>"Document not found"</span>)
                >
                <OMDocDocumentDrawer uri/>
                {
                    let Some(Ok((css,html))) = res.get() else {unreachable!()};
                    view!(<CSSHTML css=css.into() html=html.into()/>)
                }</Show>
            </Suspense>
        }
    }
    #[cfg(feature="client")]
    view!(<div/>)
}

#[island]
fn OMDocDocumentDrawer(uri:DocumentURI) -> impl IntoView {
    use thaw::*;
    let open = RwSignal::new(false);
    view!(
        <div style="position:absolute;top:0;right:0;"><Button on_click=move |_| open.set(true)>OMDoc</Button></div>
        <OverlayDrawer open position=DrawerPosition::Right size=DrawerSize::Large>
            {move || if open.get() { Some(view!(<omdoc::OMDocDocumentURI uri/>)) } else {None}}
        </OverlayDrawer>
    )
}

#[component]
fn CSSHTML(css:Vec<CSS>,html:String) -> impl IntoView {
    use leptos_meta::{Stylesheet,Style,Script};
    view!(<For each=move || css.clone().into_iter().enumerate() key=|(u,_)| u.to_string()
        children = move |(u,css)| match css {
            CSS::Link(href) => view!(<Stylesheet href/>).into_any(),
            CSS::Inline(content) => view!(<Style>{content.to_string()}</Style>).into_any()
        }
        />
        <Script src="/shtml.js"/>
        <div style="text-align:left;" inner_html=html/>
    )
}

fn sym_shtml(uri:ContentURI) -> impl IntoView {
    let name = uri.name();
    let name = name.as_ref();
    let name = name.rsplit_once('/').map(|(_,n)| n).unwrap_or(name);
    let name = name.to_string();
    leptos::html::span()
        .attr("shtml:term","OMID")
        .attr("shtml:maincomp","")
        .attr("shtml:head",uri.to_string())
        .attr("style","font-family:monospace")
        .inner_html(name)
    //view!(<span (shtml:term)="OMID" (shtml:maincomp)="" (shtml:head)={uri.to_string()} style="font-family:monospace">{name.clone()}</span>)
    /*format!(
        "<span shtml:term=\"OMID\" shtml:maincomp shtml:head=\"{uri}\">{}</span>",uri.name()
    )*/
}
fn var_shtml(uri:NarrDeclURI) -> impl IntoView {
    let name = uri.name();
    let name = name.as_ref();
    let name = name.rsplit_once('/').map(|(_,n)| n).unwrap_or(name);
    let name = name.to_string();
    leptos::html::span()
        .attr("shtml:term","OMV")
        .attr("shtml:maincomp","")
        .attr("shtml:head",uri.to_string())
        .attr("style","font-family:monospace")
        .inner_html(name)
    //view!(<span shtml:term="OMV" shtml:varcomp="" shtml:head={uri.to_string()} style="font-family:monospace">{name.clone()}</span>)
    /*format!(
        "<span shtml:term=\"OMID\" shtml:maincomp shtml:head=\"{uri}\">{}</span>",uri.name()
    )*/
}

/*
uri!{DOC @opt
#[component]
{pub(crate)} fn Document() -> {impl IntoView} = uri {
    use thaw::*;
    if let Some(uri) = uri {
    let res = create_blocking_resource(|| (),move |_| uri!(@uri DOC get_document_inner => uri.into()));
    Some(view!{
        <Suspense fallback=|| view!(<Spinner/>)>{
            if let Some(Ok((css,html))) = res.get() {
                Some(view!(<Layout style="height:100vh" content_style="height:100%">
                    //<OMDocDrawer><DocumentOMDoc uri/></OMDocDrawer>
                    //<OMDocDrawerURI uri=uri.into()/>
                    //<OMDocDrawerLink link=format!("./omdoc?uri={}",urlencoding::encode(&uri.to_string()))/>
                    <CSSHTML css html/>
                </Layout>))
            } else {None}
        }</Suspense>
    })} else {None}
}}
*/

/*

#[component]
pub(crate) fn OMDocTop() -> impl IntoView {
    #[cfg(feature="server")]
    {
        use leptos_router::*;
        use server::*;
        use immt_core::uris::*;
        let params = use_query_map();
        move || {
            let uri = params.with(|p| from_params(p));
            uri.map(|uri| {
                match uri {
                    MMTUri::Narrative(NarrativeURI::Doc(d)) => view!(<DocumentOMDoc uri=d/>).into_view(),
                    _ => view!("TODO: "{uri.to_string()}).into_view()
                }
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
        use immt_controller::Controller;
        let res = create_blocking_resource(|| (),move |_| async move {
            immt_controller::controller().backend().get_document_async(uri).await.map(server::DDWrap)
            //doc.map(|doc| leptos::ssr::render_to_string(|| server::omdoc_document(doc)))
            //doc.map(|d| server::omdoc_document(d))
        });
        return view!{<Suspense>{
            match res.get() {
                Some(Some(d)) => {
                    server::omdoc_document(d.0).into_view()
                    //view!(<span inner_html=d/>).into_view()//d.into_view(),
                }
                _ => view!(<span>"Document not found"</span>).into_view(),
            }
        }</Suspense>}
    }
    #[cfg(feature="client")]
    view!(<div/>)
}


#[island]
fn OMDocDrawerLink(link:String) -> impl IntoView {
    use thaw::*;
    let show = create_rw_signal(false);
    view!(
        <Button on_click=move |_| show.set(true) style="position:absolute;top:0;right:0;">OMDoc</Button>
        <Drawer show placement=DrawerPlacement::Right mount=DrawerMount::None width="80%" height="100%">
        {move || {
            if show.get() {
                view!(<crate::components::IFrame src=link.clone() ht="100%"/>).into_view()
            } else {view!(<div/>).into_view()}
        }}
    </Drawer>)
}

#[island]
fn OMDocDrawer(children: Children) -> impl IntoView {
    use thaw::*;
    let show = create_rw_signal(false);
    view!(
        <Button on_click=move |_| show.set(true) style="position:absolute;top:0;right:0;">OMDoc</Button>
        <Drawer show placement=DrawerPlacement::Right mount=DrawerMount::None width="80%">
        {children()}
    </Drawer>)
}
/*
#[island]
fn OMDocDrawerURI(uri:MMTUri) -> impl IntoView {
    use thaw::*;
    let show = create_rw_signal(false);
    view!(
        <Button on_click=move |_| show.set(true) style="position:absolute;top:0;right:0;">OMDoc</Button>
        <Drawer show placement=DrawerPlacement::Right mount=DrawerMount::None width="80%">
        {move || {
            if show.get() {
                let resource = create_resource(|| (),move |_| leptos_uri!(@uri get_omdoc_inner => uri));
                view!{
                    <Suspense fallback=|| view!(<Spinner/>)>{
                        if let Some(Ok(s)) = resource.get() {
                            Some(view!(<div inner_html=s/>))
                        } else {None}
                    }</Suspense>
                }.into_view()
            } else {view!(<div/>).into_view()}
        }}
    </Drawer>)
}

 */

#[cfg(feature="server")]
pub(crate) mod server {
    use leptos::*;
    use leptos_router::ParamsMap;
    use serde::Serializer;
    use immt_api::controller::Controller;
    use immt_core::content::Term;
    use immt_core::uris::{MMTUri, Name, NarrativeURI, ModuleURI, DocumentURI, ArchiveURI, ContentURI, NarrDeclURI, SymbolURI, NamedURI};
    use immt_core::narration::{CSS, DocData, Document, DocumentElement, DocumentModule, DocumentReference, Language, Section};
    use immt_core::uris::archives::ArchiveId;
/*
    pub(crate) async fn document_html(axum::extract::Query(params) : axum::extract::Query<std::collections::HashMap<String,String>>) -> axum::response::Html<String> {
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
        if let Some((css,s)) = immt_controller::controller().backend().get_html_async(uri).await {
            let s = format!("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"UTF-8\">\n  <title>SHTML</title>\n  <script src=\"/shtml.js\"></script>{}\n</head>\n<body>\n{}\n</body>",
                            CSSWrap(css),s
            );
            axum::response::Html(s)
        } else {
            axum::response::Html(FAIL.to_string())
        }
    }

 */

    pub(super) fn omdoc_document(dd:DocData) -> impl IntoView {
        use thaw::*;
        let d = dd.as_ref();
        let (uses,_,children) = doc_elems(&d.elements,&dd);
        let ttl = d.title.clone().unwrap_or_else(||
            if let Some(p) = d.uri.path() {
                format!("[{}/{p}/{}]",d.uri.archive().id(),d.uri.name())
            } else {
                format!("[{}]{}", d.uri.archive().id(),d.uri.name())
            }.into()
        );
        view!{
            <h2 inner_html=ttl.to_string()/>
            <div style="text-align:left;"><Space vertical=true>
            {uses}{children}</Space></div>
        }
    }

    pub(super) fn omdoc_doc_elem(e:&DocumentElement,in_structure:bool,top:&DocData) -> impl IntoView + Clone {
        use immt_api::controller::Controller;
        use thaw::*;
        match e {
            DocumentElement::InputRef(dr) => Some(omdoc_inputref(dr)).into_view(),
            DocumentElement::Section(s) => Some(omdoc_section(s,top)).into_view(),
            DocumentElement::Module(m) => Some(omdoc_doc_module(m,top)).into_view(),
            DocumentElement::MathStructure(m) => {
                let v = m.children.iter().map(|e| omdoc_doc_elem(e,true,top)).collect::<Vec<_>>();
                let name = m.uri.name().as_ref().rsplit_once('/').map(|(_,n)| n).unwrap_or(m.uri.name().as_ref()).to_string();
                let uri = m.module_uri;
                Some(move || view!{
                    <Card clone:v>
                        <CardHeader slot>"Structure "<super::LinkedName uri=NamedURI::Content(uri.into())/></CardHeader>""
                    {v}
                    </Card>
                }).into_view()
            },
            DocumentElement::Morphism(m) => {
                let v = m.children.iter().map(|e| omdoc_doc_elem(e,true,top)).collect::<Vec<_>>();
                let name = m.uri.name().as_ref().rsplit_once('/').map(|(_,n)| n).unwrap_or(m.uri.name().as_ref()).to_string();
                let domain = m.domain;
                Some(move || view!{
                    <Card title=format!("Morphism {name}") clone:v clone:name>
                        <CardHeader slot clone:name>"Morphism "{name}": "<super::LinkedName uri=NamedURI::Content(domain.into())/></CardHeader>{v}
                    </Card>
                }).into_view()
            },
            DocumentElement::Paragraph(p) => {
                let ctrl = immt_controller::controller();
                let b = ctrl.backend();
                let v = p.children.iter().map(|e| omdoc_doc_elem(e,in_structure,top)).collect::<Vec<_>>();
                let fors = p.fors.iter().map(|u|
                    view!({sym_shtml(*u)}", ")).collect::<Vec<_>>();
                let tms = p.terms.iter().map(|(_,u)|
                    view!(<math><mrow inner_html=u.display(|s| b.get_notations(s)).to_string()/></math><br/>)).collect::<Vec<_>>();
                let kind = p.kind;
                Some(move || view!{<Card title=kind.to_string() clone:fors clone:tms clone:v>
                    <CardHeaderExtra slot clone:fors><span style="font-weight:normal;">
                        {if !fors.is_empty() {"Introduces "} else {""}}
                        <span style="font-family:monospace;">{fors}</span>
                    </span></CardHeaderExtra>
                    <span>{tms}</span>
                    {v}
                </Card>}).into_view()
            }
            DocumentElement::ConstantDecl(c) if in_structure => {
                let c = *c;
                Some(move || view! {<Card class="immt-small-card" title=format!("Field {}",c.name())><span/></Card>}).into_view()
            }
            DocumentElement::ConstantDecl(c) => {
                let c = *c;
                Some(move || view! {<Card class="immt-small-card">
                    <CardHeader slot>"Symbol "{sym_shtml(c.into())}</CardHeader>
                        ""
                    </Card>}).into_view()
            }DocumentElement::VarDef{uri,..} => {
                let c = *uri;
                Some(move || view! {<Card class="immt-small-card">
                    <CardHeader slot>"Variable "{var_shtml(c)}</CardHeader>
                        ""
                    </Card>}).into_view()
            }
            DocumentElement::TopTerm(t) => {
                let ctrl = immt_controller::controller();
                let b = ctrl.backend();
                let s = t.display(|s| b.get_notations(s)).to_string();
                Some(move || view!{<Card class="immt-small-card" title="Expression" clone:s>
                    <CardHeader slot clone:s>"Expression "<math><mrow inner_html=s/></math></CardHeader>
                    ""
                </Card>}).into_view()
            }
            DocumentElement::UseModule(_) | DocumentElement::ImportModule(_) | DocumentElement::VarNotation {..} |
            DocumentElement::Symref {..} | DocumentElement::Varref {..} |
            DocumentElement::SetSectionLevel(_) | DocumentElement::Definiendum{..} => None::<String>.into_view(),
            _ => {
                let e = e.to_string();
                Some(move || view!(<div>{format!("TODO: {e}")}</div>)).into_view()
            }
        }
    }

    pub(super) fn omdoc_inputref(dr:&DocumentReference) -> impl IntoView {
        let uri = dr.target;
        move || view!{<super::OmDocInputref uri/>}
    }

    pub(super) fn omdoc_section(s:&Section,top:&DocData) -> impl IntoView {
        use thaw::*;
        use super::UriHover;

        let ttl = s.title.as_ref().map(|t| top.get_title(t)).flatten().unwrap_or_else(|| {
            let name = s.uri.name();
            let name = name.as_ref().rsplit_once('/').map(|(_,n)| n).unwrap_or(name.as_ref());
            format!("<span style=\"font-family:monospace\">{name}</span>").into()
        });

        let (uses,_,v) = doc_elems(&s.children,top);
        let uri = s.uri;
        let ttl = ttl.to_string();
        move || view!{
            <Card clone:ttl clone:uses clone:v>
                <CardHeader slot clone:ttl><UriHover uri=uri.into()>
                    <span inner_html=ttl/>
                </UriHover></CardHeader>
                <CardHeaderExtra slot clone:uses>{uses}</CardHeaderExtra>
                {v}
            </Card>
        }
    }
    pub(super) fn omdoc_doc_module(m:&DocumentModule,top:&DocData) -> impl IntoView {
        use thaw::*;
        use super::UriHover;
        let (uses,imports,v) = doc_elems(&m.children,top);
        let name = m.uri.name();
        let name = name.as_ref().rsplit_once('/').map(|(_,n)| n).unwrap_or(name.as_ref());
        let name = name.to_string();
        let uri = m.uri;
        move || view!{
            <Card title=format!("Module {}",name) clone:name clone:uses clone:imports clone:v>
                <CardHeader slot clone:name><UriHover uri=uri.into()>
                    "Module "<span style="font-family:monospace">{name}</span>
                </UriHover></CardHeader>
                <CardHeaderExtra slot clone:uses clone:imports>{uses}<br/>{imports}</CardHeaderExtra>
                {v}
            </Card>
        }
    }

    fn doc_elems(elems:&Vec<DocumentElement>,top:&DocData) -> (Option<impl IntoView + Clone>,Option<impl IntoView + Clone>,impl IntoView + Clone) {
        let mut uses = Vec::new();
        let mut imports = Vec::new();
        let v = elems.iter().filter_map(|e|
            if let DocumentElement::UseModule(uri) = e {
                uses.push(*uri);
                None
            } else if let DocumentElement::ImportModule(uri) = e {
                imports.push(*uri);
                None
            } else {
                Some(omdoc_doc_elem(e, false,top))
            }
        ).collect::<Vec<_>>();
        (if uses.is_empty() {None} else {
            Some(move || view!{
                <div>"Uses "
                <span>{uses.iter().map(|u| view!(<super::LinkedName uri=NamedURI::Content((*u).into())/>", ")).collect::<Vec<_>>()}</span>
                </div>
            })
        },if imports.is_empty() {None} else {
            Some(move || view!{
                <div>"Imports "
                <span>{imports.iter().map(|u| view!(<super::LinkedName uri=NamedURI::Content((*u).into())/>", ")).collect::<Vec<_>>()}</span>
                </div>
            })
        },move || v.clone())
    }

    fn var_shtml(uri:NarrDeclURI) -> impl IntoView {
        let name = uri.name();
        let name = name.as_ref();
        let name = name.rsplit_once('/').map(|(_,n)| n).unwrap_or(name);
        let name = name.to_string();
        move || view!(<span shtml:term="OMV" shtml:varcomp="" shtml:head={uri.to_string()} style="font-family:monospace">{name.clone()}</span>)
        /*format!(
            "<span shtml:term=\"OMV\" shtml:varcomp shtml:head=\"{uri}\">{}</span>",uri.name().as_ref().split('/').last().unwrap_or(uri.name().as_ref())
        )*/
    }

    // URI parsing --------------------------------------------------------
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
            let a = controller.backend().get_archive(a,|a| a.map(|a| a.uri()))?;
            let m = ModuleURI::new(a,p,m, l.unwrap_or(Language::English));
            Some(MMTUri::Content(match c {
                Some(n) => ContentURI::Symbol(immt_core::uris::symbols::SymbolURI::new(m,n)),
                None => ContentURI::Module(m)
            }))
        } else if let Some(d) = d {
            if m.is_some() {return None}
            if c.is_some() {return None}
            let controller = controller();
            let a = controller.backend().get_archive(a,|a| a.map(|a| a.uri()))?;
            let d = DocumentURI::new(a,p,d, l.unwrap_or(Language::English));
            Some(MMTUri::Narrative(match e {
                Some(n) => NarrativeURI::Decl(NarrDeclURI::new(d,n)),
                None => NarrativeURI::Doc(d)
            }))
        } else {
            if e.is_some() {return None}
            if c.is_some() {return None}
            let controller = controller();
            let a = controller.backend().get_archive(a,|a| a.map(|a| a.uri()))?;
            Some(MMTUri::Archive(a))
        }
    }


    pub(crate) fn get_doc_uri(a:ArchiveId,p:Option<Name>,l:Language,d:Name) -> Option<DocumentURI> {
        use immt_controller::{ControllerTrait,controller};
        use immt_api::backend::archives::Storage;
        let controller = controller();
        let a = controller.backend().get_archive(a,|a| a.map(|a| a.uri()))?;
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

    #[derive(Clone)]
    pub(crate) struct DDWrap(pub DocData);
    impl serde::Serialize for DDWrap {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
            serializer.serialize_str(&self.0.as_ref().uri.to_string())
        }
    }
    impl<'de> serde::Deserialize<'de> for DDWrap {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
            let s = DocumentURI::deserialize(deserializer)?;
            immt_controller::controller().backend().get_document(s).map(|d| DDWrap(d)).ok_or_else(|| serde::de::Error::custom("Failed"))
        }
    }
}

#[island]
fn LinkedName(uri: NamedURI) -> impl IntoView {
    use thaw::*;
    let name = uri.name();
    let name = name.as_ref();
    let name = name.rsplit_once('/').map(|(_,n)| n).unwrap_or(name);
    let name = name.to_string();
    view!(<UriHover uri><span style="font-family:monospace">{name}</span></UriHover>)
}

#[island]
fn UriHover(uri: NamedURI,children:Children) -> impl IntoView {
    use thaw::*;
    view!(
            <Popover><PopoverTrigger slot>
              <a style="color:blue" href=format!("/?uri={}",urlencoding::encode(&uri.to_string()))>{
                children()
        }</a>
            </PopoverTrigger>{uri.to_string()}
            </Popover>
        )
}

/*
leptos_uri! {
#[server(
    prefix="/html",
    endpoint="omdoc",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
)]
{pub async} fn get_omdoc_inner() -> {Result<String,ServerFnError<String>>} = uri {
    use immt_api::controller::Controller;
    crate::console_log!("Here!");
        provide_meta_context();
    let uri = if let Some(d) = uri {
        d
    } else {return Err(ServerFnError::WrappedServerError("Invalid URI".to_string())) };
    match uri {
        MMTUri::Narrative(NarrativeURI::Doc(d)) => {
            let doc = immt_controller::controller().backend().get_document_async(d).await;
            if let Some(doc) = doc {
                Ok(ssr::render_to_string(move || server::omdoc_document(doc)).to_string())
            } else {
                return Err(ServerFnError::WrappedServerError("Document not found!".to_string()))
            }
        }
        MMTUri::Narrative(NarrativeURI::Decl(d)) => todo!(),
        _ => return Err(ServerFnError::WrappedServerError("TODO".to_string()))
    }
}}
*/




#[island]
pub fn OmDocInputref(uri:DocumentURI) -> impl IntoView {
    use thaw::*;
    let name = uri.name();
    let (expanded, set_expanded) = create_signal(false);
    move || view!{<details>
        <summary on:click=move |_| {crate::console_log!("Updating"); set_expanded.update(|b| *b=!*b)}>
            "Document "<LinkedName uri=uri.into()/>
        </summary>
        <div>{move || {
            if expanded.get() {crate::console_log!("Wuuuh!");Some("TODO")} else {None}
        }}</div>
    </details>}
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

/*
#[component]
pub(crate) fn SHtmlComponent(archive:ArchiveId,path:String) -> impl IntoView {
    use thaw::*;
    let res = create_resource(|| (),move |_| leptos_uri!(@ap DOC get_document_inner => archive,path.clone()));
    view!{
        <Suspense fallback=|| view!(<Spinner/>)>{
            if let Some(Ok((css,html))) = res.get() {
                view!(<CSSHTML css html/>).into_view()
            } else {view!(<span/>).into_view()}
        }</Suspense>
    }
}

 */
*/

#[derive(Clone,serde::Serialize,serde::Deserialize)]
pub struct HTMLConstant {
    uri:SymbolURI,
    df_html:Option<String>,
    tp_html:Option<String>,
    arity:ArgSpec,
    macro_name:Option<Name>,
    notations:Vec<(ModuleURI,Name,Option<String>,Option<String>)>
}

#[derive(Clone,serde::Serialize,serde::Deserialize)]
pub struct HTMLDocument {
    uri:DocumentURI,
    title:String,
    uses:Vec<ModuleURI>,
    children:Vec<HTMLDocElem>
}

#[cfg(feature="server")]
impl From<immt_core::narration::DocData> for HTMLDocument {
    fn from(dd:immt_core::narration::DocData) -> HTMLDocument {
        let doc = dd.as_ref();
        let uri = doc.uri;
        use immt_api::controller::Controller;
        let ctrl = immt_controller::controller();
        let b = ctrl.backend();
        let title = doc.title.as_ref().map(|s| s.to_string()).unwrap_or_else(||
            if let Some(p) = uri.path() {
                format!("[{}/{p}/{}]",uri.archive().id(),uri.name())
            } else {
                format!("[{}]{}", uri.archive().id(),uri.name())
            }
        );
        let (uses,_,children) = HTMLDocElem::do_elems(&doc.elements,&dd,b);
        HTMLDocument{uri,title,children,uses}
    }
}

#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
pub enum HTMLDocElem {
    InputRef(DocumentURI),
    Section{
        uri:NarrDeclURI,
        title_html:String,
        children:Vec<HTMLDocElem>,
        uses:Vec<ModuleURI>
    },
    Module{
        uri:ModuleURI,
        children:Vec<HTMLDocElem>,
        uses:Vec<ModuleURI>,
        imports:Vec<ModuleURI>
    },
    Structure{
        uri:ModuleURI,
        children:Vec<HTMLDocElem>,
        uses:Vec<ModuleURI>,
        imports:Vec<ModuleURI>
    },
    Morphism{
        uri:ModuleURI,
        domain:ModuleURI,
        children:Vec<HTMLDocElem>,
        total:bool
    },
    Paragraph{
        uri:NarrDeclURI,
        kind:StatementKind,
        fors:Vec<ContentURI>,
        terms_html:Vec<(SymbolURI,String)>,
        children:Vec<HTMLDocElem>,
        uses:Vec<ModuleURI>
    },
    Constant(SymbolURI),
    Var {
        uri:NarrDeclURI,
        tp:Option<String>,
        df:Option<String>
    },
    Term(String)
}

#[cfg(feature="server")]
impl HTMLDocElem {
    fn do_elems(elems:&Vec<immt_core::narration::DocumentElement>,top:&immt_core::narration::DocData,backend:&immt_api::backend::Backend) -> (Vec<ModuleURI>,Vec<ModuleURI>,Vec<HTMLDocElem>) {
        use immt_core::narration::DocumentElement;
        let mut uses = Vec::new();
        let mut imports = Vec::new();
        let mut ret = Vec::new();
        for e in elems { match e {
            DocumentElement::InputRef(ir) => ret.push(HTMLDocElem::InputRef(ir.target)),
            DocumentElement::UseModule(uri) => uses.push(*uri),
            DocumentElement::ImportModule(uri)  => imports.push(*uri),
            DocumentElement::Section(s) => {
                let (uses,_,children) = HTMLDocElem::do_elems(&s.children,top,backend);
                // TODO css
                let title_html = s.title.as_ref().map(|t| top.read_snippet(t.range).map(|s| s.1.to_string())).flatten().unwrap_or_else(|| {
                    let name = s.uri.name();
                    let name = name.as_ref().rsplit_once('/').map(|(_,n)| n).unwrap_or(name.as_ref());
                    format!("<span style=\"font-family:monospace\">{name}</span>")
                });
                ret.push(HTMLDocElem::Section{uri:s.uri,title_html,children,uses})
            }
            DocumentElement::Module(m) => {
                let (uses,imports,children) = HTMLDocElem::do_elems(&m.children,top,backend);
                let uri = m.module_uri;
                ret.push(HTMLDocElem::Module{uri,children,uses,imports})
            }
            DocumentElement::MathStructure(s) => {
                let (uses,imports,children) = HTMLDocElem::do_elems(&s.children,top,backend);
                let uri = s.module_uri;
                ret.push(HTMLDocElem::Structure{uri,children,uses,imports})
            }
            DocumentElement::Morphism(s) => {
                let (_,_,children) = HTMLDocElem::do_elems(&s.children,top,backend);
                let uri = s.content_uri;
                let domain = s.domain;
                ret.push(HTMLDocElem::Morphism{uri,children,domain,total:s.total})
            }
            DocumentElement::Paragraph(p) => {
                let (uses,_,children) = HTMLDocElem::do_elems(&p.children,top,backend);
                let kind = p.kind;
                let fors = p.fors.clone();
                let terms_html = p.terms.iter().map(|(s,t)|
                    (*s,backend.display_term(t).to_string())
                ).collect::<Vec<_>>();
                let uri = p.uri;
                ret.push(HTMLDocElem::Paragraph{uri,kind,fors,terms_html,children,uses})
            }
            DocumentElement::ConstantDecl(s) => ret.push(HTMLDocElem::Constant(*s)),
            DocumentElement::VarDef{uri,tp,df,..} => {
                let uri = *uri;
                let tp = tp.as_ref().map(|t| backend.display_term(t).to_string());
                let df = df.as_ref().map(|t| backend.display_term(t).to_string());
                ret.push(HTMLDocElem::Var{uri,tp,df})
            }
            DocumentElement::TopTerm(t) => {
                let s = backend.display_term(t).to_string();
                ret.push(HTMLDocElem::Term(s))
            }


            DocumentElement::VarNotation {..} |
            DocumentElement::Symref {..} | DocumentElement::Varref {..} |
            DocumentElement::SetSectionLevel(_) | DocumentElement::Definiendum{..} => (),
            DocumentElement::Problem(_) => {}
        }};
        (uses,imports,ret)
    }
}