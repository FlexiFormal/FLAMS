#[cfg(feature = "ssr")]
fn main() {
    use immt::server::settings;
    use immt_system::settings::SettingsSpec;
    #[allow(unused_imports)]
    use immt_stex::STEX;
    fn exit() {
      immt_system::building::queue_manager::QueueManager::clear();
      let _ = immt_system::settings::Settings::get().close();
      std::process::exit(0)
    }

    #[allow(clippy::redundant_pub_crate)]
    #[allow(clippy::future_not_send)]
    async fn run(settings: SettingsSpec) {
      let lsp = settings.lsp;
        let _ce = color_eyre::install();
        immt_system::initialize(settings);
        if lsp {
            let (sender,recv) = tokio::sync::watch::channel(None);
            tokio::select! {
              () = immt::server::run(Some(sender)) => {},
              () = immt::server::lsp::lsp(recv) => {},
              _ = tokio::signal::ctrl_c() => exit()
            }
        } else {
            tokio::select! {
              () = immt::server::run(None) => {},
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
