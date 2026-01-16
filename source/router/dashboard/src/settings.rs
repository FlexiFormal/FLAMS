use flams_backend_types::ManagerCacheSize;
use flams_router_base::require_login;
use flams_utils::settings::SettingsSpec;
use flams_web_utils::components::wait_and_then_fn;
use ftml_dom::utils::css::inject_css;
use leptos::prelude::*;

#[server(
  prefix="/api",
  endpoint="settings",
  output=server_fn::codec::Json
)]
#[allow(clippy::unused_async)]
pub async fn get_settings() -> Result<(SettingsSpec, bool), ServerFnError<String>> {
    use flams_router_base::LoginState;
    use flams_system::settings::Settings;
    match LoginState::get_server() {
        LoginState::Admin | LoginState::NoAccounts | LoginState::User { is_admin: true, .. } => {
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
            Ok((spec, flams_git::gl::GLInstance::global().has_loaded()))
        }
        _ => Err("Not logged in".to_string().into()),
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Memory {
    uris: ftml_uris::MemoryState,
    terms: ftml_ontology::terms::TermCacheSize,
    backend: ManagerCacheSize,
    search: (usize, usize),
}

#[server(
  prefix="/api",
  endpoint="memory",
  output=server_fn::codec::Json
)]
#[allow(clippy::unused_async)]
pub async fn get_memory() -> Result<Memory, ServerFnError<String>> {
    use flams_math_archives::backend::GlobalBackend;
    use flams_router_base::LoginState;
    match LoginState::get_server() {
        LoginState::Admin | LoginState::NoAccounts | LoginState::User { is_admin: true, .. } => {
            tokio::task::spawn_blocking(|| {
                let backend = GlobalBackend.memory();
                let uris = ftml_uris::get_memory_state();
                let terms = ftml_ontology::terms::get_cache_size();
                Ok(Memory {
                    backend,
                    uris,
                    terms,
                    search: flams_search::Searcher::get().size(),
                })
            })
            .await
            .map_err(|e| e.to_string())?
        }
        _ => Err("Not logged in".to_string().into()),
    }
}

#[server(
  prefix="/api",
  endpoint="reload",
  output=server_fn::codec::Json
)]
pub async fn reload() -> Result<(), ServerFnError<String>> {
    use flams_math_archives::backend::GlobalBackend;
    use flams_router_base::LoginState;
    match LoginState::get_server() {
        LoginState::Admin | LoginState::NoAccounts | LoginState::User { is_admin: true, .. } => {
            let _ = tokio::task::spawn_blocking(move || {
                GlobalBackend.reset::<flams_system::TokioEngine>();
                ftml_uris::clear_memory();
                ftml_ontology::terms::clear_term_cache();
            })
            .await;
            let _ = tokio::task::spawn_blocking(|| {
                for e in flams_system::iter::<flams_system::FlamsExtension>() {
                    (e.on_reload)();
                }
            });
            Ok(())
        }
        _ => Err("Not logged in".to_string().into()),
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Settings;
#[leptos_router::lazy_route]
impl leptos_router::LazyRoute for Settings {
    fn data() -> Self {
        Self
    }
    fn view(Settings: Self) -> AnyView {
        settings()
    }
}

//#[component]
fn settings() -> AnyView {
    use thaw::Table;
    inject_css("flams-settings", include_str!("settings.css"));
    require_login(Box::new(|| {
        wait_and_then_fn(
            || async {
                Ok::<_, ServerFnError<String>>((get_settings().await?, get_memory().await?))
            },
            |((settings, gl), mem)| {
                let loading = RwSignal::new(false);
                let reload_act = flams_web_utils::components::message_action(
                    move |()| {
                        loading.set(true);
                        reload()
                    },
                    move |()| {
                        loading.set(false);
                        "success".to_string()
                    },
                );
                view!(
                  <Table class="flams-settings-table"><thead/><tbody>
                    <tr><td><h2>"Status"</h2></td><td/></tr>
                    {do_memory(mem)}
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
                        <tr>
                          <td class="flams-settings-col"><b>"Stack Size"</b></td>
                          <td class="flams-settings-col">{(settings.stack_size)}{if settings.stack_size.is_some() {"MB"} else {"(System default)"}}</td>
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
                ).into_any()
            },
        )
    }))
}

fn do_memory(mem: Memory) -> impl IntoView {
    let total = mem.terms.total_bytes() + mem.uris.total_bytes() + mem.backend.total_bytes();

    let total = total + mem.search.1;
    macro_rules! disp {
        ($name:literal = $num:expr;$bytes:expr) => {
            view!(<tr>
                <td class="flams-settings-col">{$name}</td>
                <td class="flams-settings-col">{$num}" ("{disp!($bytes)}")"</td>
            </tr>)
        };
        ($val:expr) => {
            bytesize::ByteSize::b($val as u64)
                .display()
                .iec_short()
                .to_string()
        };
    }

    let search = disp!("Search Index" = mem.search.0;mem.search.1);
    view! {
        <tr>
            <td class="flams-settings-col"><b>"Relations"</b></td>
            <td class="flams-settings-col">{mem.backend.relations}</td>
        </tr>
        {search}
        <tr><td/><td/></tr>
        <tr><td><b>"Backend"</b></td></tr>
            {disp!("Modules" = mem.backend.num_modules;mem.backend.modules_bytes)}
            {disp!("Documents" = mem.backend.num_documents;mem.backend.documents_bytes)}
            <tr>
                <td class="flams-settings-col">"Total"</td>
                <td class="flams-settings-col">{disp!(mem.backend.total_bytes())}</td>
            </tr>
        <tr><td><b>"URIs"</b></td></tr>
            {disp!("Base URIs" = mem.uris.num_base_uris;mem.uris.base_uris_bytes)}
            {disp!("IDs" = mem.uris.num_ids;mem.uris.ids_bytes)}
            {disp!("Archive IDs" = mem.uris.num_archives;mem.uris.archives_bytes)}
            {disp!("URI names" = mem.uris.num_uri_names;mem.uris.uri_names_bytes)}
            {disp!("URI paths" = mem.uris.num_uri_paths;mem.uris.uri_paths_bytes)}
            <tr>
                <td class="flams-settings-col">"Total"</td>
                <td class="flams-settings-col">{disp!(mem.uris.total_bytes())}</td>
            </tr>
        <tr><td><b>"Terms"</b></td></tr>
            {disp!("Applications" = mem.terms.num_applications;mem.terms.applications_bytes)}
            {disp!("Bindings" = mem.terms.num_bindings;mem.terms.bindings_bytes)}
            {disp!("Records" = mem.terms.num_records;mem.terms.records_bytes)}
            {disp!("Opaques" = mem.terms.num_opaques;mem.terms.opaques_bytes)}
            <tr>
                <td class="flams-settings-col">"Total"</td>
                <td class="flams-settings-col">{disp!(mem.terms.total_bytes())}</td>
            </tr>


        <tr><td/><td/></tr>
        <tr>
            <td class="flams-settings-col"><b>"Total"</b></td>
            <td class="flams-settings-col">{disp!(total)}</td>
        </tr>
    }
}
