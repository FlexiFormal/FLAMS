use leptos::*;
use immt_core::utils::settings::SettingsSpec;
use crate::accounts::{if_logged_in, login_status, LoginState};
use crate::components::mathhub_tree::ArchiveOrGroup;
use crate::console_log;
use crate::utils::{target};
use crate::utils::errors::IMMTError;

#[server(
    prefix="/api/sys",
    endpoint="get_settings",
    output=server_fn::codec::Json
)]
pub async fn get_settings() -> Result<SettingsSpec,ServerFnError> {
    use immt_controller::{controller,ControllerTrait};
    match login_status().await? {
        LoginState::Admin => {
            let mut spec = controller().settings().as_spec();
            if let Some(pw) = spec.server.admin_pwd.as_mut() {
                *pw = "********".to_string();
            }
            Ok(spec)
        },
        _ => Err(ServerFnError::Registration("Not logged in".to_string()))
    }
}

#[component]
pub fn Settings() -> impl IntoView {
    use thaw::*;
    view!(
        <Await future = || get_settings() let:settings blocking=true>{
            let settings = settings.clone().unwrap();
            view!{
            <h1>"Settings"</h1>
            <h3>"General"</h3>
            <Table><thead/><tbody>
                <tr><td><b>"MathHub"</b></td><td>{settings.mathhubs.into_iter().map(|m| m.display().to_string() + " ").collect::<Vec<_>>()}</td></tr>
                <tr><td><b>"Debug Mode"</b></td><td>{settings.debug}</td></tr>
                <tr><td><b>"Log Directory"</b></td><td>{settings.log_dir.unwrap().display().to_string()}</td></tr>
            </tbody></Table>
            <h3>"Server"</h3>
            <Table><thead/><tbody>
                <tr><td><b>"IP/Port"</b></td><td>{settings.server.ip.unwrap()}":"{settings.server.port}</td></tr>
                <tr><td><b>"Database Path"</b></td><td>{settings.server.database.unwrap().display().to_string()}</td></tr>
            </tbody></Table>
            <h3>"Build Queue"</h3>
            <Table><thead/><tbody>
                <tr><td><b>"Threads:"</b></td><td>{settings.buildqueue.num_threads}</td></tr>
            </tbody></Table>
            }
        }</Await>
    )
}

#[island]
fn Test(children:Children) -> impl IntoView {
    if_logged_in(|| children(),|| view!{<div>"Please log in to view this content"</div><span/>})
}