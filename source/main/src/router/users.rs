use leptos::prelude::*;

use crate::users::LoginError;

#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub struct UserData {
  id:i64,
  name:String,
  username:String,
  email:String,
  avatar_url:String,
  is_admin:bool
}

#[server(
  prefix="/api",
  endpoint="get_users",
)]
pub async fn get_users() -> Result<Vec<UserData>,ServerFnError<LoginError>> {
  use crate::users::LoginState;
  match LoginState::get_server() {
      LoginState::Admin | LoginState::NoAccounts => (),
      _ => return Err(ServerFnError::WrappedServerError(LoginError::NotLoggedIn))
  }
  let Some(session) = use_context::<axum_login::AuthSession<crate::server::db::DBBackend>>() else {
    return Ok(Vec::new());
  };
  let mut users = session.backend.all_users().await
    .map_err(|_| ServerFnError::WrappedServerError(LoginError::NotLoggedIn))?;
  users.sort_by_key(|e| e.id);
  Ok(users.into_iter().map(|u| UserData {
    id:u.id,
    name:u.name,
    username:u.username,
    email:u.email,
    avatar_url:u.avatar_url,
    is_admin:u.is_admin
  }).collect())
}

#[server(
  prefix="/api",
  endpoint="set_admin",
)]
pub async fn set_admin(user_id:i64, is_admin:bool) -> Result<(),ServerFnError<LoginError>> {
  use crate::users::LoginState;
  match LoginState::get_server() {
      LoginState::Admin | LoginState::NoAccounts => (),
      _ => return Err(ServerFnError::WrappedServerError(LoginError::NotLoggedIn))
  }
  let Some(session) = use_context::<axum_login::AuthSession<crate::server::db::DBBackend>>() else {
    return Ok(());
  };
  session.backend.set_admin(user_id, is_admin).await
    .map_err(|_| ServerFnError::WrappedServerError(LoginError::NotLoggedIn))?;
  Ok(())
  
}


#[component]
pub(crate) fn Users() -> impl IntoView {
  let r = Resource::new(|| (),|()| get_users());
  view!{<Suspense fallback = || view!(<immt_web_utils::components::Spinner/>)>{move ||
    match r.get() {
      Some(Ok(users)) if users.is_empty() => leptos::either::EitherOf4::A("(No users)"),
      Some(Err(e)) => leptos::either::EitherOf4::B(
        immt_web_utils::components::display_error(e.to_string().into())
      ),
      None => leptos::either::EitherOf4::C(view!(<immt_web_utils::components::Spinner/>)),
      Some(Ok(users)) => leptos::either::EitherOf4::D(user_table(users))
    }
  }</Suspense>}
}

fn user_table(v:Vec<UserData>) -> impl IntoView {
  use thaw::{Table,TableHeader,Button,ButtonSize,TableHeaderCell,TableBody,TableRow,TableCell,TableCellLayout};
  view!{<Table>
    <TableHeader><TableRow>
      <TableHeaderCell>""</TableHeaderCell>
      <TableHeaderCell>"Id"</TableHeaderCell>
      <TableHeaderCell>"Username"</TableHeaderCell>
      <TableHeaderCell>"Name"</TableHeaderCell>
      <TableHeaderCell>"Email"</TableHeaderCell>
      <TableHeaderCell>"Admin Access"</TableHeaderCell>
    </TableRow></TableHeader>
    <TableBody>{v.into_iter().map(|UserData {id,name,username,email,avatar_url,is_admin}| {
      let is_admin = RwSignal::new(is_admin);
      let a = ArcAction::new(move |()| async move {
        let nv = !is_admin.get_untracked();
        if let Ok(_) = set_admin(id,nv).await {
          is_admin.set(nv);
        }
      });
      let f = move || {
        let on_click = a.clone();
        if is_admin.get() {
          leptos::either::Either::Left(
            view!{
            "Yes "
            <Button size=ButtonSize::Small on_click=move |_| {on_click.dispatch(());}>"Demote"</Button>
            }
        )
        } else {
          leptos::either::Either::Right(
            view!{
            "No "
            <Button size=ButtonSize::Small on_click=move |_| {on_click.dispatch(());}>"Promote"</Button>
            }
        )
        }
      };
      view! {<TableRow>
        <TableCell><TableCellLayout><thaw::Avatar src=avatar_url /></TableCellLayout></TableCell>
        <TableCell><TableCellLayout>{id}</TableCellLayout></TableCell>
        <TableCell><TableCellLayout>{username}</TableCellLayout></TableCell>
        <TableCell><TableCellLayout>{name}</TableCellLayout></TableCell>
        <TableCell><TableCellLayout>{email}</TableCellLayout></TableCell>
        <TableCell><TableCellLayout>{f}</TableCellLayout></TableCell>
      </TableRow>}
    }).collect_view()}</TableBody>
  </Table>}
  /*
  pub struct UserData {
    id:i64,
    name:String,
    username:String,
    email:String,
    avatar_url:String,
    is_admin:bool
  } */
}