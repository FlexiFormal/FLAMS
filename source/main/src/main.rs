#[cfg(feature = "ssr")]
fn main() {
    use flams::server::settings;
    use flams_system::settings::SettingsSpec;
    #[allow(unused_imports)]
    use flams_stex::STEX;
    fn exit() {
      flams_system::building::queue_manager::QueueManager::clear();
      let _ = flams_system::settings::Settings::get().close();
      std::process::exit(0)
    }

    #[allow(clippy::redundant_pub_crate)]
    #[allow(clippy::future_not_send)]
    async fn run(settings: SettingsSpec) {
      let lsp = settings.lsp;
        //let _ce = color_eyre::install();
        flams_system::initialize(settings);
        if lsp {
            let (sender,recv) = tokio::sync::watch::channel(None);
            tokio::select! {
              () = flams::server::run(Some(sender)) => {},
              () = flams::server::lsp::lsp(recv) => {},
              _ = tokio::signal::ctrl_c() => exit()
            }
        } else {
            tokio::select! {
              () = flams::server::run(None) => {},
              _ = tokio::signal::ctrl_c() => exit()
            }
        }
    }

    let settings = settings::get_settings();
    let mut rt = tokio::runtime::Builder::new_multi_thread();
    rt.enable_all();
    #[cfg(debug_assertions)]
    { rt.thread_stack_size(4 * 1024 * 1024); }

    rt.build()
      .expect("Failed to initialize Tokio runtime")
      .block_on(run(settings));
}


#[cfg(feature = "hydrate")]
const fn main() {}
