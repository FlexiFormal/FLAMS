use flams_utils::settings::SettingsSpec;
use flams_web_utils::inject_css;
use leptos::prelude::*;

use crate::{users::LoginError, utils::from_server_copy};

#[server(
  prefix="/api",
  endpoint="settings",
  output=server_fn::codec::Json
)]
#[allow(clippy::unused_async)]
pub async fn get_settings() -> Result<(SettingsSpec,usize,bool),ServerFnError<LoginError>> {
  use flams_system::settings::Settings;
  use flams_system::backend::GlobalBackend;
  use crate::users::LoginState;
  match LoginState::get_server() {
      LoginState::Admin | LoginState::NoAccounts | LoginState::User{is_admin:true,..} => {
          let mut spec = Settings::get().as_spec();
          if let Some(pw) = spec.server.admin_pwd.as_mut() {
              *pw = "********".to_string();
          }
          if let Some(tk) = spec.gitlab.token.as_mut() {
              *tk = "********".to_string().into_boxed_str();
          }
          if let Some(secret) = spec.gitlab.app_secret.as_mut() {
              *secret = "********".to_string().into_boxed_str();
          }
          let rels = GlobalBackend::get().triple_store().num_relations();
          Ok((spec,rels,flams_system::GITLAB.has_loaded()))
      },
      _ => {
        Err(ServerFnError::WrappedServerError(LoginError::NotLoggedIn))
      }
  }
}
#[server(
  prefix="/api",
  endpoint="reload",
  output=server_fn::codec::Json
)]
pub async fn reload() -> Result<(),ServerFnError<LoginError>> {
  use flams_system::settings::Settings;
  use flams_system::backend::GlobalBackend;
  use crate::users::LoginState;
  match LoginState::get_server() {
      LoginState::Admin | LoginState::NoAccounts | LoginState::User{is_admin:true,..} => {
          let _ = tokio::task::spawn_blocking(move ||
            GlobalBackend::get().reset()
          ).await;
          Ok(())
      },
      _ => {
        Err(ServerFnError::WrappedServerError(LoginError::NotLoggedIn))
      }
  }
}


#[component]
pub(super) fn Settings() -> impl IntoView {
  use thaw::Table;

  inject_css("flams-settings", r"
.flams-settings-table {
    width:max-content !important;
}
.flams-settings-col {
    border:1px solid black;
    padding:3px 10px;
}
  ");
  from_server_copy(true,get_settings,|(settings,mem,gl)| {
    let loading = RwSignal::new(false);
    let reload_act = flams_web_utils::components::message_action(
      move |()| {loading.set(true);reload()},
      move |()| {loading.set(false);format!("success")}
    );
    view!(
      <Table class="flams-settings-table"><thead/><tbody>
        <tr><td><h2>"Status"</h2></td><td/></tr>
          <tr>
            <td class="flams-settings-col"><b>"Relations"</b></td>
            <td class="flams-settings-col">{mem.to_string()}</td>
          </tr>
          <tr>
            <td></td>
            <td>{move || if loading.get() {
              leptos::either::Either::Left(view!(<flams_web_utils::components::Spinner/>))
            } else {
              leptos::either::Either::Right(view!(<button on:click=move |_| {reload_act.dispatch(());}>"Reload"</button>))
            }
          }</td>
          </tr>
        <tr><td><h2>"Settings"</h2></td><td/></tr>
          <tr><td><h3>"General"</h3></td><td/></tr>
            <tr>
              <td class="flams-settings-col"><b>"MathHub"</b></td>
              <td class="flams-settings-col">{settings.mathhubs.into_iter().map(|m| m.display().to_string() + " ").collect::<Vec<_>>()}</td>
            </tr>
            <tr>
              <td class="flams-settings-col"><b>"Debug Mode"</b></td>
              <td class="flams-settings-col">{settings.debug}</td>
            </tr>
            <tr>
              <td class="flams-settings-col"><b>"Log Directory"</b></td>
              <td class="flams-settings-col">{settings.log_dir.unwrap_or_else(|| unreachable!()).display().to_string()}</td>
            </tr>
            <tr>
              <td class="flams-settings-col"><b>"Database Path"</b></td>
              <td class="flams-settings-col">{settings.database.unwrap_or_else(|| unreachable!()).display().to_string()}</td>
            </tr>
            <tr>
              <td class="flams-settings-col"><b>"Temp Directory"</b></td>
              <td class="flams-settings-col">{settings.temp_dir.unwrap_or_else(|| unreachable!()).display().to_string()}</td>
            </tr>
          <tr><td><h3>"Server"</h3></td><td/></tr>
            <tr>
              <td class="flams-settings-col"><b>"IP/Port"</b></td>
              <td class="flams-settings-col">{settings.server.ip.unwrap_or_else(|| unreachable!())}":"{settings.server.port}</td>
            </tr>
            <tr>
              <td class="flams-settings-col"><b>"Gitlab URL"</b></td>
              <td class="flams-settings-col">{settings.gitlab.url.map_or_else(|| leptos::either::Either::Left("(None)".to_string()),|s| 
                leptos::either::Either::Right(view!({s.to_string()}{
                  if gl {
                    leptos::either::Either::Left(view!(" "<div style="color:green;display:inline;"><thaw::Icon icon=icondata_ai::AiCheckOutlined/></div>))
                  } else {
                    leptos::either::Either::Right(view!(" "<div style="color:red;display:inline;"><thaw::Icon icon=icondata_ai::AiCloseOutlined/></div>))
                  }
                })
              ))}</td>
            </tr>
          <tr><td><h3>"Build Queue"</h3></td><td/></tr>
            <tr>
              <td class="flams-settings-col"><b>"Threads:"</b></td>
              <td class="flams-settings-col">{settings.buildqueue.num_threads}</td>
            </tr>
        </tbody></Table>
    )
  })
}