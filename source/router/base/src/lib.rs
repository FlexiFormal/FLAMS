#![allow(clippy::must_use_candidate)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(any(
    all(feature = "ssr", feature = "hydrate", not(feature = "docs-only")),
    not(any(feature = "ssr", feature = "hydrate"))
))]
compile_error!("exactly one of the features \"ssr\" or \"hydrate\" must be enabled");

mod login_state;
pub use login_state::*;
pub mod uris;
pub mod ws;

use leptos::{either::EitherOf3, prelude::*};

pub fn vscode_link(
    archive: &flams_ontology::uris::ArchiveId,
    rel_path: &str,
) -> impl IntoView + use<> {
    let href = format!("vscode://kwarc.flams/open?a={archive}&rp={rel_path}");
    view! {
        <a href=href><thaw::Icon icon=icondata_tb::TbBrandVscode/></a>
    }
}

#[component]
pub fn RequireLogin<Ch: IntoView + 'static>(children: TypedChildren<Ch>) -> impl IntoView {
    require_login(children.into_inner())
}

pub fn require_login<Ch: IntoView + 'static>(
    children: impl FnOnce() -> Ch + Send + 'static,
) -> impl IntoView {
    use flams_web_utils::components::{Spinner, display_error};

    let children = std::sync::Arc::new(flams_utils::parking_lot::Mutex::new(Some(children)));
    move || match LoginState::get() {
        LoginState::Loading => EitherOf3::A(view!(<Spinner/>)),
        LoginState::Admin | LoginState::NoAccounts | LoginState::User { is_admin: true, .. } => {
            EitherOf3::B((children.clone().lock().take()).map(|f| f()))
        }
        _ => EitherOf3::C(view!(<div>{display_error("Not logged in".into())}</div>)),
    }
}

#[cfg(feature = "ssr")]
/// #### Errors
pub fn get_oauth() -> Result<(flams_git::gl::auth::GitLabOAuth, String), String> {
    use flams_git::gl::auth::GitLabOAuth;
    use leptos::prelude::*;
    let Some(session) = use_context::<axum_login::AuthSession<flams_database::DBBackend>>() else {
        return Err("Internal Error".to_string());
    };
    let Some(user) = session.user else {
        return Err("Not logged in".to_string());
    };
    let Some(oauth): Option<GitLabOAuth> = expect_context() else {
        return Err("Not Gitlab integration set up".to_string());
    };
    Ok((oauth, user.secret))
}

pub trait ServerFnExt {
    type Output;
    type Error;
    #[cfg(feature = "hydrate")]
    #[allow(async_fn_in_trait)]
    async fn call_remote(self, url: String) -> Result<Self::Output, Self::Error>;
}

#[cfg(feature = "hydrate")]
mod hydrate {
    use super::ServerFnExt;
    use bytes::Bytes;
    use futures::{Stream, StreamExt};
    use leptos::server_fn::codec::{FromReq, FromRes, IntoReq};
    use leptos::server_fn::request::browser::Request;
    use leptos::{
        prelude::*,
        server_fn::{request::ClientReq, response::ClientRes},
        wasm_bindgen::JsCast,
    };
    use send_wrapper::SendWrapper;
    use wasm_streams::ReadableStream;

    struct Paired<
        F: leptos::server_fn::ServerFn<Client = leptos::server_fn::client::browser::BrowserClient>,
    > {
        sfn: F,
        url: String,
    }

    struct BrowserRequest(SendWrapper<Request>);
    struct BrowserFormData(SendWrapper<leptos::web_sys::FormData>);

    impl<CustErr> ClientReq<CustErr> for BrowserRequest {
        type FormData = BrowserFormData;

        fn try_new_get(
            path: &str,
            accepts: &str,
            content_type: &str,
            query: &str,
        ) -> Result<Self, ServerFnError<CustErr>> {
            let mut url = String::with_capacity(path.len() + 1 + query.len());
            url.push_str(path);
            url.push('?');
            url.push_str(query);
            Ok(Self(SendWrapper::new(
                Request::get(&url)
                    .header("Content-Type", content_type)
                    .header("Accept", accepts)
                    .build()
                    .map_err(|e| ServerFnError::Request(e.to_string()))?,
            )))
        }

        fn try_new_post(
            path: &str,
            accepts: &str,
            content_type: &str,
            body: String,
        ) -> Result<Self, ServerFnError<CustErr>> {
            let url = path;
            Ok(Self(SendWrapper::new(
                Request::post(&url)
                    .header("Content-Type", content_type)
                    .header("Accept", accepts)
                    .body(body)
                    .map_err(|e| ServerFnError::Request(e.to_string()))?,
            )))
        }

        fn try_new_post_bytes(
            path: &str,
            accepts: &str,
            content_type: &str,
            body: Bytes,
        ) -> Result<Self, ServerFnError<CustErr>> {
            let url = path;
            let body: &[u8] = &body;
            let body = leptos::web_sys::js_sys::Uint8Array::from(body).buffer();
            Ok(Self(SendWrapper::new(
                Request::post(&url)
                    .header("Content-Type", content_type)
                    .header("Accept", accepts)
                    .body(body)
                    .map_err(|e| ServerFnError::Request(e.to_string()))?,
            )))
        }

        fn try_new_multipart(
            path: &str,
            accepts: &str,
            body: Self::FormData,
        ) -> Result<Self, ServerFnError<CustErr>> {
            let url = path;
            Ok(Self(SendWrapper::new(
                Request::post(&url)
                    .header("Accept", accepts)
                    .body(body.0.take())
                    .map_err(|e| ServerFnError::Request(e.to_string()))?,
            )))
        }

        fn try_new_post_form_data(
            path: &str,
            accepts: &str,
            content_type: &str,
            body: Self::FormData,
        ) -> Result<Self, ServerFnError<CustErr>> {
            let form_data = body.0.take();
            let url_params =
                leptos::web_sys::UrlSearchParams::new_with_str_sequence_sequence(&form_data)
                    .map_err(|e| {
                        ServerFnError::Serialization(e.as_string().unwrap_or_else(|| {
                            "Could not serialize FormData to URLSearchParams".to_string()
                        }))
                    })?;
            Ok(Self(SendWrapper::new(
                Request::post(path)
                    .header("Content-Type", content_type)
                    .header("Accept", accepts)
                    .body(url_params)
                    .map_err(|e| ServerFnError::Request(e.to_string()))?,
            )))
        }

        fn try_new_streaming(
            path: &str,
            accepts: &str,
            content_type: &str,
            body: impl Stream<Item = Bytes> + 'static,
        ) -> Result<Self, ServerFnError<CustErr>> {
            fn streaming_request(
                path: &str,
                accepts: &str,
                content_type: &str,
                body: impl Stream<Item = Bytes> + 'static,
            ) -> Result<Request, leptos::wasm_bindgen::JsValue> {
                use leptos::wasm_bindgen::JsValue;
                let stream = ReadableStream::from_stream(body.map(|bytes| {
                    let data = leptos::web_sys::js_sys::Uint8Array::from(bytes.as_ref());
                    let data = JsValue::from(data);
                    Ok(data) as Result<JsValue, JsValue>
                }))
                .into_raw();

                let headers = leptos::web_sys::Headers::new()?;
                headers.append("Content-Type", content_type)?;
                headers.append("Accept", accepts)?;

                let init = leptos::web_sys::RequestInit::new();
                init.set_headers(&headers);
                init.set_method("POST");
                init.set_body(&stream);

                // Chrome requires setting `duplex: "half"` on streaming requests
                leptos::web_sys::js_sys::Reflect::set(
                    &init,
                    &JsValue::from_str("duplex"),
                    &JsValue::from_str("half"),
                )?;
                let req = leptos::web_sys::Request::new_with_str_and_init(path, &init)?;
                Ok(Request::from(req))
            }

            // TODO abort signal
            let request = streaming_request(path, accepts, content_type, body)
                .map_err(|e| ServerFnError::Request(format!("{e:?}")))?;
            Ok(Self(SendWrapper::new(request)))
        }
    }

    struct ClientWrap;
    impl<E> leptos::server_fn::client::Client<E> for ClientWrap {
        type Request = BrowserRequest;
        type Response = BrowserResponse;

        fn send(
            req: BrowserRequest,
        ) -> impl Future<Output = Result<Self::Response, ServerFnError<E>>> + Send {
            SendWrapper::new(async move {
                let request = req.0.take();
                let res = request
                    .send()
                    .await
                    .map(|res| BrowserResponse(SendWrapper::new(res)))
                    .map_err(|e| ServerFnError::Request(e.to_string()));
                res
            })
        }
    }
    impl<F: leptos::server_fn::ServerFn<Client = leptos::server_fn::client::browser::BrowserClient>>
        FromReq<F::InputEncoding, F::ServerRequest, F::Error> for Paired<F>
    {
        fn from_req(
            req: F::ServerRequest,
        ) -> impl Future<Output = Result<Self, ServerFnError<F::Error>>> + Send {
            async move {
                Ok(Self {
                    sfn: F::from_req(req).await?,
                    url: String::new(),
                })
            }
        }
    }
    impl<
        'a,
        F: leptos::server_fn::ServerFn<Client = leptos::server_fn::client::browser::BrowserClient>,
    > IntoReq<F::InputEncoding, BrowserRequest, F::Error> for Paired<F>
    where
        F: IntoReq<F::InputEncoding, BrowserRequest, F::Error>,
    {
        fn into_req(
            self,
            path: &str,
            accepts: &str,
        ) -> std::result::Result<BrowserRequest, ServerFnError<F::Error>> {
            let url = format!("{}{path}", self.url);
            let req = IntoReq::<F::InputEncoding, BrowserRequest, F::Error>::into_req(
                self.sfn, &url, accepts,
            )?;
            Ok(req)
        }
    }

    impl<F: leptos::server_fn::ServerFn<Client = leptos::server_fn::client::browser::BrowserClient>>
        leptos::server_fn::ServerFn for Paired<F>
    where
        F::Output: FromRes<F::OutputEncoding, BrowserResponse, F::Error>,
        F: IntoReq<F::InputEncoding, BrowserRequest, F::Error>,
    {
        const PATH: &'static str = F::PATH;
        type Client = ClientWrap;
        type ServerRequest = F::ServerRequest;
        type ServerResponse = F::ServerResponse;
        type Output = F::Output;
        type OutputEncoding = F::OutputEncoding;
        type InputEncoding = F::InputEncoding;
        type Error = F::Error;
        fn middlewares() -> Vec<
            std::sync::Arc<
                dyn server_fn::middleware::Layer<Self::ServerRequest, Self::ServerResponse>,
            >,
        > {
            F::middlewares()
        }
        fn run_body(
            self,
        ) -> impl Future<Output = std::result::Result<Self::Output, ServerFnError<Self::Error>>> + Send
        {
            self.sfn.run_body()
        }
    }

    impl<
        F: leptos::server_fn::ServerFn<
                Client = leptos::server_fn::client::browser::BrowserClient,
                OutputEncoding = leptos::server_fn::codec::Json,
            >,
    > ServerFnExt for F
    where
        F::Output: leptos::server_fn::serde::Serialize
            + for<'de> leptos::server_fn::serde::Deserialize<'de>,
        F: IntoReq<F::InputEncoding, BrowserRequest, F::Error>,
    {
        type Output = <Self as leptos::server_fn::ServerFn>::Output;
        type Error = ServerFnError<<Self as leptos::server_fn::ServerFn>::Error>;
        #[cfg(feature = "hydrate")]
        async fn call_remote(self, url: String) -> Result<Self::Output, Self::Error> {
            use leptos::server_fn::ServerFn;
            Paired { sfn: self, url }.run_on_client().await
        }
    }

    struct BrowserResponse(SendWrapper<leptos::server_fn::response::browser::Response>);

    impl<CustErr> ClientRes<CustErr> for BrowserResponse {
        fn try_into_string(
            self,
        ) -> impl Future<Output = Result<String, ServerFnError<CustErr>>> + Send {
            // the browser won't send this async work between threads (because it's single-threaded)
            // so we can safely wrap this
            SendWrapper::new(async move {
                self.0
                    .text()
                    .await
                    .map_err(|e| ServerFnError::Deserialization(e.to_string()))
            })
        }

        fn try_into_bytes(
            self,
        ) -> impl Future<Output = Result<bytes::Bytes, ServerFnError<CustErr>>> + Send {
            // the browser won't send this async work between threads (because it's single-threaded)
            // so we can safely wrap this
            SendWrapper::new(async move {
                self.0
                    .binary()
                    .await
                    .map(Bytes::from)
                    .map_err(|e| ServerFnError::Deserialization(e.to_string()))
            })
        }

        fn try_into_stream(
            self,
        ) -> Result<
            impl Stream<Item = Result<Bytes, ServerFnError>> + Send + 'static,
            ServerFnError<CustErr>,
        > {
            let stream = ReadableStream::from_raw(self.0.body().unwrap())
                .into_stream()
                .map(|data| match data {
                    Err(e) => {
                        leptos::web_sys::console::error_1(&e);
                        Err(ServerFnError::Request(format!("{e:?}")))
                    }
                    Ok(data) => {
                        let data = data.unchecked_into::<leptos::web_sys::js_sys::Uint8Array>();
                        let mut buf = Vec::new();
                        let length = data.length();
                        buf.resize(length as usize, 0);
                        data.copy_to(&mut buf);
                        Ok(Bytes::from(buf))
                    }
                });
            Ok(SendWrapper::new(stream))
        }

        fn status(&self) -> u16 {
            self.0.status()
        }

        fn status_text(&self) -> String {
            self.0.status_text()
        }

        fn location(&self) -> String {
            self.0
                .headers()
                .get("Location")
                .unwrap_or_else(|| self.0.url())
        }

        fn has_redirect(&self) -> bool {
            self.0
                .headers()
                .get(leptos::server_fn::redirect::REDIRECT_HEADER)
                .is_some()
        }
    }
}
