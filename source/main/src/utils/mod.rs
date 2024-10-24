pub(crate) mod ws;

use leptos::prelude::*;

#[cfg(feature="hydrate")]
use thaw::ToasterInjection;
#[cfg(feature="hydrate")]
use std::borrow::Cow;
use std::{fmt::Display, future::Future};
#[cfg(feature="hydrate")]
use immt_web_utils::components::error_toast;

use immt_web_utils::components::Spinner;

use crate::users::{LoginError, LoginState};


pub fn from_server_fnonce<E,Fut,F,T,V:IntoView+'static>(needs_login:bool,f: F, r:impl FnOnce(T) -> V + Send + 'static) -> impl IntoView 
  where Fut: Future<Output = Result<T,ServerFnError<E>>> + Send + 'static,
    F: Fn() -> Fut + Send + Sync + 'static, 
    T: Send + Sync + Clone + 'static + serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: Display + Clone + serde::Serialize + for<'de> serde::Deserialize<'de> + Send + Sync + 'static
{
  let wrapped_r = std::sync::Arc::new(std::sync::Mutex::new(Some(r)));
  let res = Resource::new(|| (),move |()| f());
  let go = move || {
    view!(
      <Suspense fallback = || view!(<Spinner/>)>{move ||
        match res.get() {
          Some(Ok(t)) =>
            wrapped_r.lock().ok().and_then(|mut lock| std::mem::take(&mut *lock).map(|r| r(t))).into_any(),
          Some(Err(e)) => err(e.to_string()).into_any(),
          None => view!(<Spinner/>).into_any(),
        }
      }</Suspense>
    ).into_any()
  };
  if needs_login {
    let login = expect_context::<RwSignal<LoginState>>();
    (move || {let go = go.clone(); match login.get() {
      LoginState::Loading => view!(<Spinner/>).into_any(),
      LoginState::Admin | LoginState::NoAccounts => go(),
      _ => err(LoginError::NotLoggedIn.to_string()).into_any()
    }}).into_any()
  } else { go() }
}


pub fn from_server_clone<E,Fut,F,T,V:IntoView+'static>(needs_login:bool,f: F, r:impl FnOnce(T) -> V + Clone + Send + 'static) -> impl IntoView 
  where Fut: Future<Output = Result<T,ServerFnError<E>>> + Send + 'static,
    F: Fn() -> Fut + Send + Sync + 'static, 
    T: Send + Sync + Clone + 'static + serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: Display + Clone + serde::Serialize + for<'de> serde::Deserialize<'de> + Send + Sync + 'static
{
  let res = Resource::new(|| (),move |()| f());
  let go = move || {
    view!(
      <Suspense fallback = || view!(<Spinner/>)>{move ||
        match res.get() {
          Some(Ok(t)) => (r.clone())(t).into_any(),
          Some(Err(e)) => err(e.to_string()).into_any(),
          None => view!(<Spinner/>).into_any(),
        }
      }</Suspense>
    ).into_any()
  };
  if needs_login {
    let login = expect_context::<RwSignal<LoginState>>();
    (move || {let go = go.clone(); match login.get() {
      LoginState::Loading => view!(<Spinner/>).into_any(),
      LoginState::Admin | LoginState::NoAccounts => go(),
      _ => err(LoginError::NotLoggedIn.to_string()).into_any()
    }}).into_any()
  } else { go() }
}

pub fn from_server_copy<E,Fut,F,T,V:IntoView+'static>(needs_login:bool,f: F, r:impl FnOnce(T) -> V + Copy + Send + 'static) -> impl IntoView 
  where Fut: Future<Output = Result<T,ServerFnError<E>>> + Send + 'static,
    F: Fn() -> Fut + Send + Sync + 'static, 
    T: Send + Sync + Clone + 'static + serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: Display + Clone + serde::Serialize + for<'de> serde::Deserialize<'de> + Send + Sync + 'static
{
  let res = Resource::new(|| (),move |()| f());
  let go = move || {
    view!(
      <Suspense fallback = || view!(<Spinner/>)>{move ||
        match res.get() {
          Some(Ok(t)) => r(t).into_any(),
          Some(Err(e)) => err(e.to_string()).into_any(),
          None => view!(<Spinner/>).into_any(),
        }
      }</Suspense>
    ).into_any()
  };
  if needs_login {
    let login = expect_context::<RwSignal<LoginState>>();
    (move || match login.get() {
      LoginState::Loading => view!(<Spinner/>).into_any(),
      LoginState::Admin | LoginState::NoAccounts => go(),
      _ => err(LoginError::NotLoggedIn.to_string()).into_any()
    }).into_any()
  } else { go() }
}

fn err(e:String) -> impl IntoView {
  #[cfg(feature="hydrate")]
  {
    let toaster = expect_context::<ToasterInjection>();
    error_toast(Cow::Owned(format!("Error: {e}")), toaster);
  }
  view!(<h3 style="color:red">"Error: "{e}</h3>)
}

pub fn needs_login<V:IntoView+'static>(mut f:impl FnMut() -> V + Send + 'static) -> impl IntoView {
  let login = expect_context::<RwSignal<LoginState>>();
  move || match login.get() {
    LoginState::Admin | LoginState::NoAccounts => f().into_any(),
    LoginState::Loading => view!(<Spinner/>).into_any(),
    o => {
      leptos::logging::log!("Wut? {o:?}");
      err(LoginError::NotLoggedIn.to_string()).into_any()
    }
  }
}