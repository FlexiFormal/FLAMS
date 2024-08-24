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

    pub(crate) trait MapLike:std::fmt::Debug {
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
    use immt_core::content::ArgSpec;
    use immt_core::narration::StatementKind;
    use immt_core::uris::{ContentURI, DocumentURI, ModuleURI, Name, NamedURI, NarrDeclURI, SymbolURI};
    use crate::components::content::{HTMLConstant, HTMLDocElem, HTMLDocument};
    use crate::css;

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
        let c = backend.get_constant_async(uri).await.ok_or_else(|| ServerFnError::WrappedServerError("Constant not found!".to_string()))?;
        let c = c.as_ref();
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

    #[component]
    pub fn OMDocDocumentURI(uri:DocumentURI) -> impl IntoView {
        crate::components::wait(move || omdoc_document(uri),|doc| {
            if let Ok(doc) = doc { view!(<OMDocDocument doc/>).into_any() } else {
                view!(<span>"Document not found"</span>).into_any()
            }
        })
    }

    #[component]
    pub fn OMDocDocument(doc:HTMLDocument) -> impl IntoView {
        //use thaw::*;
        use crate::components::*;
        let uses = do_uses("Uses",doc.uses);
        let title = doc.title;
        let children = doc.children;
        view!{
            <Block>
                <WHeader slot><h2 inner_html=title/></WHeader>
                <HeaderAux slot>{uses}</HeaderAux>
                {omdoc_doc_elems(children,false)}
            </Block>
        }
    }

    #[component]
    pub fn OMDocConstantURI(uri:SymbolURI) -> impl IntoView {
        crate::components::wait(move || omdoc_constant(uri),|c| {
            if let Ok(c) = c {
                view!{
                    "TODOOOO" // TODO
                    <OMDocConstantInner c/>
                }.into_any()
            } else {
                view!(<span>"Symbol not found"</span>).into_any()
            }
        })
    }

    #[component]
    pub fn OMDocConstantInnerURI(uri:SymbolURI) -> impl IntoView {
        crate::components::wait(move || omdoc_constant(uri),|c|
            if let Ok(c) = c { view!(<OMDocConstantInner c/>).into_any() } else {
                view!(<span>"Symbol not found"</span>).into_any()
            }
        )
    }

    const ARGS : &'static str = "abcdefghijk";
    const VARS : &'static str = "xyzvwrstu";

    fn macroname(macro_name:Name,arity:ArgSpec) -> impl IntoView {
        use thaw::{Text,TextTag,Caption1};
        use immt_core::content::ArgType;
        view!{<div><Caption1>"Macro: "</Caption1> <Text tag=TextTag::Code>"\\"{macro_name.to_string()}{
            arity.into_iter().enumerate().map(|(i,a)| match a {
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
        }</Text></div>}
    }

    #[component]
    pub fn OMDocConstantInner(c:HTMLConstant) -> impl IntoView {
        use immt_core::content::ArgType;
        use crate::components::Block;
        use thaw::*;
        css!(notation_table = ".immt-notation-table { width:max-content; td,th {text-align:center;padding-left:20px;padding-right:20px} }");
        view!{<Block>
            {c.df_html.map(|s| view!(" := "{super::do_term(s)}))}
            {c.macro_name.map(|s| macroname(s,c.arity))}
            {if c.notations.is_empty() {None} else {Some(
                view!(<Caption1>"Notations:"</Caption1>
                    <div style="margin-left:30px"><Table class="immt-notation-table">
                    <TableHeader><TableRow>
                        <TableCell>"id"</TableCell>
                        <TableCell>"application"</TableCell>
                        <TableCell>"operator"</TableCell>
                        <TableCell>"in module"</TableCell>
                    </TableRow></TableHeader>
                    <TableBody>{
                        c.notations.into_iter().map(|(m,n,full,op)| {view!{
                            <TableRow>
                                <TableCell>{n.to_string()}</TableCell>
                                <TableCell>{full.map(|s| view!(<math><mrow inner_html=s/></math>))}</TableCell>
                                <TableCell>{op.map(|s| view!(<math><mrow inner_html=s/></math>))}</TableCell>
                                <TableCell><LinkedName uri=NamedURI::Content(m.into())/></TableCell>
                            </TableRow>
                        }}).collect_view()
                    }</TableBody>
                </Table></div>)
            )}}
        </Block>}
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
            HTMLDocElem::Constant{uri,tp}
                => view!(<OMDocDocumentConstant uri tp in_structure/>).into_any(),
            HTMLDocElem::Var {uri,tp,df}
                => view!(<OMDocVariable uri tp df/>).into_any(),
            HTMLDocElem::Term(t)
                => view!(<div>"Term "{super::do_term(t)}</div>).into_any()
        }).collect_view()
    }

    #[component]
    pub fn OMDocVariable(uri:NarrDeclURI,tp:Option<String>,df:Option<String>) -> impl IntoView {
        view!{
            <div>
                <b>"Variable "{super::var_shtml(uri)}</b>
                {tp.map(|s| view!(" : "{super::do_term(s)}))}
                {df.map(|s| view!(" := "{super::do_term(s)}))}
            </div>
        }
    }
    #[component]
    pub fn OMDocInputref(uri:DocumentURI) -> impl IntoView {
        use thaw::*;
        use crate::components::*;
        let name = uri.name();
        view!{<Collapsible lazy=true>
            <WHeader slot>
                <Caption1Strong>"Document\u{00a0}"<LinkedName uri=uri.into()/></Caption1Strong>
            </WHeader>
            <OMDocDocumentURI uri/>
        </Collapsible>}
    }
    #[component]
    pub fn OMDocDocumentConstant(uri:SymbolURI,tp:Option<String>,in_structure:bool) -> impl IntoView {
        use thaw::*;
        use crate::components::*;
        let name = uri.name();
        view! {<div style="margin-left:15px">
            <h4 style="margin:0;display:inline-block">
                {if in_structure {"Field "} else {"Symbol "}}
                {super::sym_shtml(uri.into())}
            </h4>
            {tp.map(|s| view!(" : "{super::do_term(s)}))}
            <div style="margin-left:20px"><Collapsible lazy=true>
                <WHeader slot><Caption1>"More Info"</Caption1></WHeader>
                <OMDocConstantInnerURI uri/>
            </Collapsible></div>
        </div>}
    }

    #[component]
    pub fn OMDocParagraph(kind:StatementKind,uri:NarrDeclURI,fors:Vec<ContentURI>,terms_html:Vec<(SymbolURI,String)>,uses:Vec<ModuleURI>,elems:Vec<HTMLDocElem>) -> impl IntoView {
        use thaw::*;
        use crate::components::*;
        let name = uri.name();
        let (expanded, set_expanded) = signal(false);
        fn do_name(uri:ContentURI,terms:&Vec<(SymbolURI,String)>) -> impl IntoView {
            view!{
                {super::sym_shtml(uri)}
                {match uri {
                    ContentURI::Symbol(s) => if let Some((_,t)) = terms.iter().find(|(s2,_)| *s2 == s) {
                        Some(view!(" as "{super::do_term(t.clone())}))
                    } else {None},
                    _ => None
                }}
            }
        }
        view!{
            <Block>
                <WHeader slot><h4 style="margin:0">
                    {kind.to_string() + "\u{00a0}"}
                    <LinkedName uri=uri.into()/>
                </h4></WHeader>
                <HeaderAux slot>{do_uses("Uses",uses)}</HeaderAux>
                <HeaderAux2 slot>{if fors.is_empty() {None} else {
                    let mut iter = fors.into_iter();
                    Some(view!{<Text>
                        "introduces\u{00a0}"{do_name(iter.next().unwrap(),&terms_html)}
                        {iter.map(|u| view!(", "{do_name(u,&terms_html)})).collect_view()}
                    </Text>})
                }}</HeaderAux2>
                <Separator slot>"Elements"</Separator>
                { omdoc_doc_elems(elems,false) }
                <Footer slot><Collapsible lazy=true>
                    <WHeader slot><Caption1>"Show"</Caption1></WHeader>
                    {
                        crate::components::wait(
                            move || super::fragments::paragraph(uri),
                            move |re| if let Ok(re) = re {
                                view!{<Block><div inner_html=re.1.to_string()/></Block>}.into_any()
                            } else {
                                view!{"Paragraph not found"}.into_any()
                            }
                        )
                    }
                </Collapsible></Footer>
            </Block>
        }
    }

    #[component]
    pub fn OMDocSection(uri:NarrDeclURI,title_html:String,elems:Vec<HTMLDocElem>,uses:Vec<ModuleURI>,in_structure:bool) -> impl IntoView {
        use thaw::*;
        use crate::components::*;
        view!{
            <Block>
                <WHeader slot><h3 style="margin:0">
                    <UriHover uri=uri.into()><span inner_html=title_html/></UriHover>
                </h3></WHeader>
                <HeaderAux slot>{do_uses("Uses",uses)}</HeaderAux>
                { omdoc_doc_elems(elems,in_structure) }
            </Block>
        }
    }

    #[component]
    pub fn OMDocDocumentModule(uri:ModuleURI, elems:Vec<HTMLDocElem>, uses:Vec<ModuleURI>, imports:Vec<ModuleURI>) -> impl IntoView {
        use thaw::*;
        use crate::components::*;
        view!{
            <Block>
                <WHeader slot>
                    <h3 style="margin:0;">
                        "Module\u{00a0}"<LinkedName uri=NamedURI::Content(uri.into())/>
                    </h3>
                </WHeader>
                <HeaderAux slot>{do_uses("Uses",uses)}</HeaderAux>
                <HeaderAux2 slot>{do_uses("Imports",imports)}</HeaderAux2>
                { omdoc_doc_elems(elems,false) }
            </Block>
        }
    }

    #[component]
    pub fn OMDocDocumentStructure(uri:ModuleURI, elems:Vec<HTMLDocElem>, uses:Vec<ModuleURI>, imports:Vec<ModuleURI>) -> impl IntoView {
        use thaw::*;
        use crate::components::*;
        view!{
            <Block>
                <WHeader slot>
                    <h4 style="margin:0">"Structure "<LinkedName uri=NamedURI::Content(uri.into())/></h4>
                </WHeader>
                <HeaderAux slot>{do_uses("Uses",uses)}</HeaderAux>
                <HeaderAux2 slot>{do_uses("Imports",imports)}</HeaderAux2>
                { omdoc_doc_elems(elems,true) }
            </Block>
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
        view!(<UriHover uri>{name}</UriHover>)
    }

    #[component]
    fn UriHover(uri: NamedURI,children:Children) -> impl IntoView {
        use thaw::*;
        crate::components::inject_css("immt-omdoc-link",r#"
            .immt-omdoc-link { color: var(--colorCompoundBrandForeground1);}
            .immt-omdoc-link:hover {color: var(--colorCompoundBrandForeground1Hover);}
            .immt-omdoc-link-popover {z-index:10;}
        "#);
        view!(
            <Popover class="immt-omdoc-link-popover" appearance=PopoverAppearance::Brand><PopoverTrigger slot class="immt-omdoc-link">
               <a href=format!("/?uri={}",urlencoding::encode(&uri.to_string()))>{
                children()
               }</a>
            </PopoverTrigger>{uri.to_string()}
            </Popover>
        )
    }

}


#[component]
pub(crate) fn URITop() -> impl IntoView {
    use leptos_router::*;
    use immt_core::uris::*;
    use thaw::*;
    use leptos_meta::Stylesheet;
    use crate::components::Themer;
    view!{
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/immt.css"/>
        <Themer>
        {
            // Ugly hack to get client/server isomorphism
            // TODO try a signal
            crate::components::wait_blocking(move || async move {
                #[cfg(feature="server")]
                {leptos_router::hooks::use_query_map().with(|p| uris::from_params(p))}
                #[cfg(feature="client")]
                {Option::<URI>::None}
            },|uri| view!{
            <Show when=move || uri.is_some() fallback=|| view!(<span>"No Document"</span>)>
            <Layout position=LayoutPosition::Absolute>{
                match uri.unwrap() {
                    URI::Narrative(NarrativeURI::Doc(d)) => view!(<Document uri=d/>).into_any(),
                    uri => view!(<span>"TODO: "{uri.to_string()}</span>).into_any()
                }
            }</Layout></Show>
        })
        }
        </Themer>
    }
}

#[component]
fn Document(uri:DocumentURI) -> impl IntoView {
    crate::components::wait(
        move || fragments::document(uri),
        move |doc| {let ok=doc.is_ok(); view!{
            <Show when=move || ok fallback=|| view!(<span>"Document not found"</span>)>{
                let Ok((css,html)) = doc.clone() else {unreachable!()};
                view!(
                    <OMDocDocumentDrawer uri/>
                    <CSSHTML css=css.into() html=html.into()/>
                )
            }</Show>
        }}
    )
}

#[component]
fn OMDocDocumentDrawer(uri:DocumentURI) -> impl IntoView {
    use thaw::*;
    use crate::components::{WideDrawer,Trigger,WHeader};
    use omdoc::OMDocDocumentURI;
    view!{<WideDrawer lazy=true>
        <Trigger slot><div style="position:fixed;top:5px;right:5px;">
            <Button appearance=ButtonAppearance::Primary>OMDoc</Button>
        </div></Trigger>
        <omdoc::OMDocDocumentURI uri/>
    </WideDrawer>}
}

#[component]
fn CSSHTML(css:Vec<CSS>,html:String) -> impl IntoView {
    use leptos_meta::{Stylesheet,Style,Script};
    for c in css {
        match c {
            CSS::Link(href) => {
                //let _ = view!(<Stylesheet href=href/>);
                let id = immt_core::utils::hashstr(&href);
                crate::components::inject_stylesheet(id,href);

            },
            CSS::Inline(content) => {
                //let _ = view!(<Style>{content.to_string()}</Style>);
                let id = immt_core::utils::hashstr(&content);
                crate::components::inject_css_string(id,content);
            }
        }
    }
    //let _ = view!(<Script src="/shtml.js"/>);
    crate::components::inject_script("shtml","/shtml.js");
    view!(
        <div style="text-align:left;" inner_html=html/>
    )
}

fn do_term(s:String) -> impl IntoView {
    use thaw::*;
    view!(<Text tag=TextTag::Code><math><mrow inner_html=s/></math></Text>)
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
        .inner_html(name)
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
        .inner_html(name)
}


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
    Constant{
        uri:SymbolURI,
        tp:Option<String>
    },
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
                    {name.to_string()}
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
            DocumentElement::ConstantDecl(s) => ret.push(HTMLDocElem::Constant {
                uri:*s,
                tp:backend.get_constant(*s).map(|c| c.as_ref().tp.as_ref().map(|t| backend.display_term(t).to_string())).flatten()
            }),
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