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
        <a href=href><thaw::Icon icon=icondata_tb::TbBrandVscodeOutline/></a>
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
/*
#[cfg(feature = "hydrate")]
mod hydrate {
    use super::ServerFnExt;
    use leptos::{
        prelude::{FromServerFnError, ServerFnError},
        server_fn::{
            self,
            codec::{FromReq, FromRes, IntoReq, IntoRes},
            request::BrowserMockReq,
            response::BrowserMockRes,
        },
    };

    impl<
        In: server_fn::codec::Encoding,
        Out: server_fn::codec::Encoding,
        Err: server_fn::serde::Serialize
            + server_fn::serde::de::DeserializeOwned
            + std::fmt::Debug
            + Clone
            + Send
            + Sync
            + std::fmt::Display
            + std::str::FromStr
            + 'static,
        F: leptos::server_fn::ServerFn<
                Client = leptos::server_fn::client::browser::BrowserClient,
                Server = leptos::server_fn::mock::BrowserMockServer,
                Protocol = leptos::server_fn::Http<In, Out>,
                Error = ServerFnError<Err>,
                InputStreamError = ServerFnError<Err>,
                OutputStreamError = ServerFnError<Err>,
            > + FromReq<In, BrowserMockReq, F::Error>
            + IntoReq<In, server_fn::request::browser::BrowserRequest, F::Error>
            + server_fn::serde::Serialize
            + server_fn::serde::de::DeserializeOwned
            + std::fmt::Debug
            + Clone
            + Send
            + Sync
            + for<'de> server_fn::serde::Deserialize<'de>,
    > ServerFnExt for F
    where
        F::Output: IntoRes<Out, BrowserMockRes, F::Error>
            + FromRes<Out, server_fn::response::browser::BrowserResponse, F::Error>
            + IntoReq<In, server_fn::request::browser::BrowserRequest, F::Error>,
    {
        type Output = <Self as leptos::server_fn::ServerFn>::Output;
        type Error = F::Error;
        #[cfg(feature = "hydrate")]
        async fn call_remote(self, url: String) -> Result<Self::Output, Self::Error> {
            use server_fn::response::ClientRes;
            let input = self;
            let path = format!("{}{}", url, F::PATH);
            let path: &str = &path;
            // ---------------------------------------
            /*
            Ok(<F::Protocol as server_fn::Protocol<
                F,
                F::Output,
                F::Client,
                F::Server,
                F::Error,
                F::Error,
                F::Error,
            >>::run_client(path, input)
            .await?)
            */

            // create and send request on client
            let req = input.into_req(path, Out::CONTENT_TYPE)?;
            let req = TODO SOMETHING ELSE;
            let res =
                <leptos::server_fn::client::browser::BrowserClient as server_fn::client::Client<
                    ServerFnError<Err>,
                    ServerFnError<Err>,
                    ServerFnError<Err>,
                >>::send(req)
                .await?;

            let status = <server_fn::response::browser::BrowserResponse as ClientRes<
                ServerFnError<Err>,
            >>::status(&res);
            let location = <server_fn::response::browser::BrowserResponse as ClientRes<
                ServerFnError<Err>,
            >>::location(&res);
            let has_redirect_header = <server_fn::response::browser::BrowserResponse as ClientRes<
                ServerFnError<Err>,
            >>::has_redirect(&res);

            // if it returns an error status, deserialize the error using the error's decoder.
            let res = if (400..=599).contains(&status) {
                let bytes = <server_fn::response::browser::BrowserResponse as ClientRes<
                    ServerFnError<Err>,
                >>::try_into_bytes(res);
                Err(ServerFnError::de(bytes.await?))
            } else {
                // otherwise, deserialize the body as is
                let output =
                    <Self::Output as FromRes<Out, _, ServerFnError<Err>>>::from_res(res).await?;
                Ok(output)
            }?;

            // if redirected, call the redirect hook (if that's been set)
            if (300..=399).contains(&status) || has_redirect_header {
                server_fn::redirect::call_redirect_hook(&location);
            }
            Ok(res)
        }
    }
}
 */

#[cfg(feature = "hydrate")]
mod hydrate {
    use super::ServerFnExt;
    use bytes::Bytes;
    use futures::{Stream, StreamExt};
    use leptos::server_fn::codec::{Encoding, FromReq, FromRes, IntoReq, IntoRes};
    use leptos::server_fn::request::BrowserMockReq;
    use leptos::server_fn::request::browser::{BrowserRequest as OrigBrowserRequest, Request};
    use leptos::server_fn::response::BrowserMockRes;
    use leptos::server_fn::response::browser::BrowserResponse as OrigBrowserResponse;
    use leptos::{
        prelude::*,
        server_fn::{request::ClientReq, response::ClientRes},
        wasm_bindgen::JsCast,
    };
    use send_wrapper::SendWrapper;
    use wasm_streams::ReadableStream;

    // -------------------------------------
    //
    struct Paired<
        In,
        F: leptos::server_fn::ServerFn<
                Client = leptos::server_fn::client::browser::BrowserClient,
                Server = leptos::server_fn::mock::BrowserMockServer,
                Protocol = leptos::server_fn::Http<In, server_fn::codec::Json>, //OutputEncoding = leptos::server_fn::codec::Json,
            >,
    > {
        sfn: F,
        url: String,
    }

    struct BrowserRequest(SendWrapper<Request>);
    struct BrowserFormData(SendWrapper<leptos::web_sys::FormData>);
    struct EncodingWrap<E: Encoding>(E);

    impl<E: Encoding> server_fn::ContentType for EncodingWrap<E> {
        const CONTENT_TYPE: &'static str = E::CONTENT_TYPE;
    }
    impl<E: Encoding> Encoding for EncodingWrap<E> {
        const METHOD: http::Method = E::METHOD;
    }

    impl<
        In: Encoding,
        Err: server_fn::serde::Serialize
            + server_fn::serde::de::DeserializeOwned
            + std::fmt::Debug
            + Clone
            + Send
            + Sync
            + std::fmt::Display
            + std::str::FromStr
            + 'static,
        F: leptos::server_fn::ServerFn<
                Client = leptos::server_fn::client::browser::BrowserClient,
                Server = leptos::server_fn::mock::BrowserMockServer,
                Protocol = leptos::server_fn::Http<In, server_fn::codec::Json>,
                Error = ServerFnError<Err>,
                InputStreamError = ServerFnError<Err>,
                OutputStreamError = ServerFnError<Err>,
            > + FromReq<In, BrowserMockReq, ServerFnError<Err>>,
    > FromReq<EncodingWrap<In>, BrowserMockReq, ServerFnError<Err>> for Paired<In, F>
    {
        async fn from_req(req: BrowserMockReq) -> Result<Self, ServerFnError<Err>> {
            Ok(Self {
                sfn: <F as FromReq<In, BrowserMockReq, ServerFnError<Err>>>::from_req(req).await?,
                url: String::new(),
            })
        }
    }

    impl<
        In: Encoding,
        Err: server_fn::serde::Serialize
            + server_fn::serde::de::DeserializeOwned
            + std::fmt::Debug
            + Clone
            + Send
            + Sync
            + std::fmt::Display
            + std::str::FromStr
            + 'static,
        F: leptos::server_fn::ServerFn<
                Client = leptos::server_fn::client::browser::BrowserClient,
                Server = leptos::server_fn::mock::BrowserMockServer,
                Protocol = leptos::server_fn::Http<In, server_fn::codec::Json>,
                Error = ServerFnError<Err>,
                InputStreamError = ServerFnError<Err>,
                OutputStreamError = ServerFnError<Err>,
            > + IntoReq<In, OrigBrowserRequest, ServerFnError<Err>>,
    > IntoReq<EncodingWrap<In>, BrowserRequest, ServerFnError<Err>> for Paired<In, F>
    {
        fn into_req(
            self,
            _path: &str,
            accepts: &str,
        ) -> Result<BrowserRequest, ServerFnError<Err>> {
            let Paired { sfn, url } = self;
            let path = format!("{}{}", url, F::PATH);
            let req = <F as IntoReq<In, OrigBrowserRequest, _>>::into_req(sfn, &path, accepts)?;
            let req: Request = req.into();
            Ok(BrowserRequest(SendWrapper::new(req)))
        }
    }

    impl<
        In: Encoding,
        Err: server_fn::serde::Serialize
            + server_fn::serde::de::DeserializeOwned
            + std::fmt::Debug
            + Clone
            + Send
            + Sync
            + std::fmt::Display
            + std::str::FromStr
            + 'static,
        F: leptos::server_fn::ServerFn<
                Client = leptos::server_fn::client::browser::BrowserClient,
                Server = leptos::server_fn::mock::BrowserMockServer,
                Protocol = leptos::server_fn::Http<In, server_fn::codec::Json>,
                Error = ServerFnError<Err>,
                InputStreamError = ServerFnError<Err>,
                OutputStreamError = ServerFnError<Err>,
            >,
    > leptos::server_fn::ServerFn for Paired<In, F>
    where
        F: FromReq<In, BrowserMockReq, ServerFnError<Err>>
            + IntoReq<In, OrigBrowserRequest, ServerFnError<Err>>
            + server_fn::serde::Serialize
            + server_fn::serde::de::DeserializeOwned
            + std::fmt::Debug
            + Clone
            + Send
            + Sync
            + for<'de> server_fn::serde::Deserialize<'de>,
        /*Paired<In, F>: IntoReq<In, OrigBrowserRequest, ServerFnError<Err>>
        + FromReq<In, BrowserMockReq, ServerFnError<Err>>
        + Send,*/
        F::Output: IntoRes<server_fn::codec::Json, BrowserMockRes, ServerFnError<Err>>
            + FromRes<server_fn::codec::Json, OrigBrowserResponse, ServerFnError<Err>>
            + Send
            + for<'de> server_fn::serde::Deserialize<'de>,
    {
        const PATH: &'static str = F::PATH;
        type Client = ClientWrap;
        type Server = F::Server;
        type Protocol = leptos::server_fn::Http<EncodingWrap<In>, server_fn::codec::Json>;
        type Output = F::Output;
        type Error = ServerFnError<Err>;
        type InputStreamError = ServerFnError<Err>;
        type OutputStreamError = ServerFnError<Err>;
        fn middlewares() -> Vec<
            std::sync::Arc<
                dyn server_fn::middleware::Layer<
                        <Self::Server as server_fn::server::Server<Self::Error>>::Request,
                        <Self::Server as server_fn::server::Server<Self::Error>>::Response,
                    >,
            >,
        > {
            F::middlewares()
        }
        async fn run_body(self) -> Result<Self::Output, Self::Error> {
            unreachable!()
        }
    }

    impl<E: FromServerFnError> ClientReq<E> for BrowserRequest {
        type FormData = BrowserFormData;

        fn try_new_req_query(
            path: &str,
            content_type: &str,
            accepts: &str,
            query: &str,
            method: http::Method,
        ) -> Result<Self, E> {
            let mut url = String::with_capacity(path.len() + 1 + query.len());
            url.push_str(path);
            url.push('?');
            url.push_str(query);
            let inner = match method {
                http::Method::GET => Request::get(&url),
                http::Method::DELETE => Request::delete(&url),
                http::Method::POST => Request::post(&url),
                http::Method::PUT => Request::put(&url),
                http::Method::PATCH => Request::patch(&url),
                m => {
                    return Err(E::from_server_fn_error(
                        ServerFnErrorErr::UnsupportedRequestMethod(m.to_string()),
                    ));
                }
            };
            Ok(Self(SendWrapper::new(
                inner
                    .header("Content-Type", content_type)
                    .header("Accept", accepts)
                    .build()
                    .map_err(|e| {
                        E::from_server_fn_error(ServerFnErrorErr::Request(e.to_string()))
                    })?,
            )))
        }

        fn try_new_req_text(
            path: &str,
            content_type: &str,
            accepts: &str,
            body: String,
            method: http::Method,
        ) -> Result<Self, E> {
            let url = path;
            let inner = match method {
                http::Method::POST => Request::post(&url),
                http::Method::PATCH => Request::patch(&url),
                http::Method::PUT => Request::put(&url),
                m => {
                    return Err(E::from_server_fn_error(
                        ServerFnErrorErr::UnsupportedRequestMethod(m.to_string()),
                    ));
                }
            };
            Ok(Self(SendWrapper::new(
                inner
                    .header("Content-Type", content_type)
                    .header("Accept", accepts)
                    .body(body)
                    .map_err(|e| {
                        E::from_server_fn_error(ServerFnErrorErr::Request(e.to_string()))
                    })?,
            )))
        }

        fn try_new_req_bytes(
            path: &str,
            content_type: &str,
            accepts: &str,
            body: Bytes,
            method: http::Method,
        ) -> Result<Self, E> {
            let url = path;
            let body: &[u8] = &body;
            let body = leptos::web_sys::js_sys::Uint8Array::from(body).buffer();
            let inner = match method {
                http::Method::POST => Request::post(&url),
                http::Method::PATCH => Request::patch(&url),
                http::Method::PUT => Request::put(&url),
                m => {
                    return Err(E::from_server_fn_error(
                        ServerFnErrorErr::UnsupportedRequestMethod(m.to_string()),
                    ));
                }
            };
            Ok(Self(SendWrapper::new(
                inner
                    .header("Content-Type", content_type)
                    .header("Accept", accepts)
                    .body(body)
                    .map_err(|e| {
                        E::from_server_fn_error(ServerFnErrorErr::Request(e.to_string()))
                    })?,
            )))
        }

        fn try_new_req_multipart(
            path: &str,
            accepts: &str,
            body: Self::FormData,
            method: http::Method,
        ) -> Result<Self, E> {
            let url = path;
            let inner = match method {
                http::Method::POST => Request::post(&url),
                http::Method::PATCH => Request::patch(&url),
                http::Method::PUT => Request::put(&url),
                m => {
                    return Err(E::from_server_fn_error(
                        ServerFnErrorErr::UnsupportedRequestMethod(m.to_string()),
                    ));
                }
            };
            Ok(Self(SendWrapper::new(
                inner
                    .header("Accept", accepts)
                    .body(body.0.take())
                    .map_err(|e| {
                        E::from_server_fn_error(ServerFnErrorErr::Request(e.to_string()))
                    })?,
            )))
        }

        fn try_new_req_form_data(
            path: &str,
            accepts: &str,
            content_type: &str,
            body: Self::FormData,
            method: http::Method,
        ) -> Result<Self, E> {
            let form_data = body.0.take();
            let url_params =
                leptos::web_sys::UrlSearchParams::new_with_str_sequence_sequence(&form_data)
                    .map_err(|e| {
                        E::from_server_fn_error(ServerFnErrorErr::Serialization(
                            e.as_string().unwrap_or_else(|| {
                                "Could not serialize FormData to URLSearchParams".to_string()
                            }),
                        ))
                    })?;
            let inner = match method {
                http::Method::POST => Request::post(path),
                http::Method::PUT => Request::put(path),
                http::Method::PATCH => Request::patch(path),
                m => {
                    return Err(E::from_server_fn_error(
                        ServerFnErrorErr::UnsupportedRequestMethod(m.to_string()),
                    ));
                }
            };
            Ok(Self(SendWrapper::new(
                inner
                    .header("Content-Type", content_type)
                    .header("Accept", accepts)
                    .body(url_params)
                    .map_err(|e| {
                        E::from_server_fn_error(ServerFnErrorErr::Request(e.to_string()))
                    })?,
            )))
        }

        fn try_new_req_streaming(
            path: &str,
            accepts: &str,
            content_type: &str,
            body: impl Stream<Item = Bytes> + Send + 'static,
            method: http::Method,
        ) -> Result<Self, E> {
            fn streaming_request(
                path: &str,
                accepts: &str,
                content_type: &str,
                method: http::Method,
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
                init.set_method(method.as_str());
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
            let request =
                streaming_request(path, accepts, content_type, method, body).map_err(|e| {
                    E::from_server_fn_error(ServerFnErrorErr::Request(format!("{e:?}")))
                })?;
            Ok(Self(SendWrapper::new(request)))
        }
    }

    struct ClientWrap;
    impl<
        Error: FromServerFnError + Send,
        InputStreamError: FromServerFnError,
        OutputStreamError: FromServerFnError,
    > leptos::server_fn::client::Client<Error, InputStreamError, OutputStreamError> for ClientWrap
    {
        type Request = BrowserRequest;
        type Response = BrowserResponse;

        fn send(req: BrowserRequest) -> impl Future<Output = Result<Self::Response, Error>> + Send {
            SendWrapper::new(async move {
                let request = req.0.take();
                let res = request
                    .send()
                    .await
                    .map(|res| BrowserResponse(SendWrapper::new(res)))
                    .map_err(|e| {
                        Error::from_server_fn_error(ServerFnErrorErr::Request(e.to_string()))
                    });
                res
            })
        }
        async fn open_websocket(
            _: &str,
        ) -> Result<
            (
                impl Stream<Item = Result<Bytes, Bytes>> + Send + 'static,
                impl futures::Sink<Bytes> + Send + 'static,
            ),
            Error,
        > {
            Err::<(futures::stream::BoxStream<Result<Bytes, Bytes>>, Vec<Bytes>), _>(
                Error::from_server_fn_error(ServerFnErrorErr::ServerError(
                    "Not implemented".to_string(),
                )),
            )
        }

        fn spawn(future: impl Future<Output = ()> + Send + 'static) {
            wasm_bindgen_futures::spawn_local(future);
        }
    }

    struct BrowserResponse(SendWrapper<leptos::server_fn::response::browser::Response>);

    impl<E: FromServerFnError> ClientRes<E> for BrowserResponse {
        fn try_into_string(self) -> impl Future<Output = Result<String, E>> + Send {
            SendWrapper::new(async move {
                self.0.text().await.map_err(|e| {
                    E::from_server_fn_error(ServerFnErrorErr::Deserialization(e.to_string()))
                })
            })
        }

        fn try_into_bytes(self) -> impl Future<Output = Result<Bytes, E>> + Send {
            SendWrapper::new(async move {
                self.0.binary().await.map(Bytes::from).map_err(|e| {
                    E::from_server_fn_error(ServerFnErrorErr::Deserialization(e.to_string()))
                })
            })
        }

        fn try_into_stream(
            self,
        ) -> Result<impl Stream<Item = Result<Bytes, Bytes>> + Send + Sync + 'static, E> {
            let stream = ReadableStream::from_raw(self.0.body().unwrap())
                .into_stream()
                .map(|data| match data {
                    Err(e) => {
                        leptos::web_sys::console::error_1(&e);
                        Err(format!("{e:?}").into())
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

    // -------------------------------------

    impl<
        In: Encoding,
        Err: server_fn::serde::Serialize
            + server_fn::serde::de::DeserializeOwned
            + std::fmt::Debug
            + Clone
            + Send
            + Sync
            + std::fmt::Display
            + std::str::FromStr
            + 'static,
        F: leptos::server_fn::ServerFn<
                Client = leptos::server_fn::client::browser::BrowserClient,
                Server = leptos::server_fn::mock::BrowserMockServer,
                Protocol = leptos::server_fn::Http<In, server_fn::codec::Json>,
                Error = ServerFnError<Err>,
                InputStreamError = ServerFnError<Err>,
                OutputStreamError = ServerFnError<Err>,
            >,
    > ServerFnExt for F
    where
        F: FromReq<In, BrowserMockReq, ServerFnError<Err>>
            + IntoReq<In, OrigBrowserRequest, ServerFnError<Err>>
            + server_fn::serde::Serialize
            + server_fn::serde::de::DeserializeOwned
            + std::fmt::Debug
            + Clone
            + Send
            + Sync
            + for<'de> server_fn::serde::Deserialize<'de>,
        F::Output: IntoRes<server_fn::codec::Json, BrowserMockRes, ServerFnError<Err>>
            + FromRes<server_fn::codec::Json, OrigBrowserResponse, ServerFnError<Err>>
            + Send
            + for<'de> server_fn::serde::Deserialize<'de>,
    {
        type Output = <Self as leptos::server_fn::ServerFn>::Output;
        type Error = <Self as leptos::server_fn::ServerFn>::Error;
        #[cfg(feature = "hydrate")]
        async fn call_remote(self, url: String) -> Result<Self::Output, Self::Error> {
            use leptos::server_fn::ServerFn;
            Paired { sfn: self, url }.run_on_client().await
        }
    }
}
