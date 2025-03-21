use crate::{LoginState, users::UserData};
use flams_web_utils::components::{Spinner, display_error};
use leptos::{either::EitherOf4, prelude::*};

#[component(transparent)]
pub fn LoginProvider<Ch: IntoView + 'static>(children: TypedChildren<Ch>) -> impl IntoView {
    let children = children.into_inner();
    let res = Resource::new_blocking(
        || (),
        |()| async {
            super::server_fns::login_state().await.unwrap_or_else(|e| {
                leptos::logging::error!("Error getting login state: {e}");
                LoginState::None
            })
        },
    );
    let sig = RwSignal::new(LoginState::Loading);
    let _ = view! {<Suspense>{move || {res.get();()}}</Suspense>};
    let _ = Effect::new(move |_| {
        if let Some(r) = res.get() {
            sig.set(r)
        }
    });
    provide_context(sig);
    children()
}

#[component]
pub fn Users() -> impl IntoView {
    let r = Resource::new(|| (), |()| super::server_fns::get_users());
    view! {<Suspense fallback = || view!(<Spinner/>)>{move ||
      match r.get() {
        Some(Ok(users)) if users.is_empty() => EitherOf4::A("(No users)"),
        Some(Err(e)) => EitherOf4::B(
          display_error(e.to_string().into())
        ),
        None => EitherOf4::C(view!(<Spinner/>)),
        Some(Ok(users)) => EitherOf4::D(user_table(users))
      }
    }</Suspense>}
}

fn user_table(v: Vec<UserData>) -> impl IntoView {
    use thaw::{
        Button, ButtonSize, Table, TableBody, TableCell, TableCellLayout, TableHeader,
        TableHeaderCell, TableRow,
    };
    view! {<Table>
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
          if let Ok(_) = super::server_fns::set_admin(id,nv).await {
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
}
