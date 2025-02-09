pub(crate) mod ws;

use leptos::{either::{Either, EitherOf3}, prelude::*};

use std::{borrow::Cow, fmt::Display, future::Future};
use flams_web_utils::components::display_error;

use flams_web_utils::components::Spinner;

use crate::users::{LoginError, LoginState};


pub fn from_server_fnonce<E,Fut,F,T,V:IntoView+'static>(needs_login:bool,f: F, r:impl FnOnce(T) -> V + Send + 'static) -> impl IntoView 
  where Fut: Future<Output = Result<T,ServerFnError<E>>> + Send + 'static,
    F: Fn() -> Fut + Send + Sync + 'static, 
    T: Send + Sync + Clone + 'static + serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: Display + Clone + serde::Serialize + for<'de> serde::Deserialize<'de> + Send + Sync + 'static
{
  let wrapped_r = std::sync::Arc::new(std::sync::Mutex::new(Some(r)));
  let res = Resource::new(|| (),move |()| f());
  let go = move || view! {
    <Suspense fallback = || view!(<Spinner/>)>{move ||
      match res.get() {
        Some(Ok(t)) =>
          EitherOf3::A(wrapped_r.lock().ok().and_then(|mut lock| std::mem::take(&mut *lock).map(|r| r(t)))),
        Some(Err(e)) => EitherOf3::B(err(e.to_string())),
        None => EitherOf3::C(view!(<Spinner/>)),
      }
    }</Suspense>
  };
  if needs_login {
    Either::Left(move || {let go = go.clone(); match LoginState::get() {
      LoginState::Loading => EitherOf3::A(view!(<Spinner/>)),
      LoginState::Admin | LoginState::NoAccounts | LoginState::User{is_admin:true,..} => EitherOf3::B(go()),
      _ => EitherOf3::C(err(LoginError::NotLoggedIn.to_string()))
    }})
  } else { Either::Right(go()) }
}


pub fn from_server_clone<E,Fut,F,T,V:IntoView+'static>(needs_login:bool,f: F, r:impl FnOnce(T) -> V + Clone + Send + 'static) -> impl IntoView 
  where Fut: Future<Output = Result<T,ServerFnError<E>>> + Send + 'static,
    F: Fn() -> Fut + Send + Sync + 'static, 
    T: Send + Sync + Clone + 'static + serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: Display + Clone + serde::Serialize + for<'de> serde::Deserialize<'de> + Send + Sync + 'static
{
  let res = Resource::new(|| (),move |()| f());
  let go = move || view! {
      <Suspense fallback = || view!(<Spinner/>)>{move ||
        match res.get() {
          Some(Ok(t)) => EitherOf3::A((r.clone())(t)),
          Some(Err(e)) => EitherOf3::B(err(e.to_string())),
          None => EitherOf3::C(view!(<Spinner/>)),
        }
      }</Suspense>
  };
  if needs_login {
    Either::Left(move || {let go = go.clone(); match LoginState::get() {
      LoginState::Loading => EitherOf3::A(view!(<Spinner/>)),
      LoginState::Admin | LoginState::NoAccounts | LoginState::User{is_admin:true,..} => EitherOf3::B(go()),
      _ => EitherOf3::C(err(LoginError::NotLoggedIn.to_string()))
    }})
  } else { Either::Right(go()) }
}

pub fn from_server_copy<E,Fut,F,T,V:IntoView+'static>(needs_login:bool,f: F, r:impl FnOnce(T) -> V + Copy + Send + 'static) -> impl IntoView 
  where Fut: Future<Output = Result<T,ServerFnError<E>>> + Send + 'static,
    F: Fn() -> Fut + Send + Sync + 'static, 
    T: Send + Sync + Clone + 'static + serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: Display + Clone + serde::Serialize + for<'de> serde::Deserialize<'de> + Send + Sync + 'static
{
  let res = Resource::new(|| (),move |()| f());
  let go = move || view! {
      <Suspense fallback = || view!(<Spinner/>)>{move ||
        match res.get() {
          Some(Ok(t)) => EitherOf3::A(r(t)),
          Some(Err(e)) => EitherOf3::B(err(e.to_string())),
          None => EitherOf3::C(view!(<Spinner/>)),
        }
      }</Suspense>
  };
  if needs_login {
    Either::Left(move || match LoginState::get() {
      LoginState::Loading => EitherOf3::A(view!(<Spinner/>)),
      LoginState::Admin | LoginState::NoAccounts | LoginState::User{is_admin:true,..} => EitherOf3::B(go()),
      _ => EitherOf3::C(err(LoginError::NotLoggedIn.to_string()))
    })
  } else { Either::Right(go()) }
}

fn err(e:String) -> impl IntoView {
    display_error(Cow::Owned(format!("Error: {e}")));
}

pub fn needs_login<V:IntoView+'static>(mut f:impl FnMut() -> V + Send + 'static) -> impl IntoView {
  move || match LoginState::get() {
    LoginState::Admin | LoginState::NoAccounts | LoginState::User{is_admin:true,..} => EitherOf3::A(f()),
    LoginState::Loading => EitherOf3::B(view!(<Spinner/>)),
    o => {
      leptos::logging::log!("Wut? {o:?}");
      EitherOf3::C(err(LoginError::NotLoggedIn.to_string()))
    }
  }
}