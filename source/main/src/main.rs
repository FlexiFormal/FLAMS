#[cfg(feature = "ssr")]
fn main() {
    use immt::server::settings;
    use immt_system::settings::SettingsSpec;
    #[allow(unused_imports)]
    use immt_stex::STEX;

    #[allow(clippy::redundant_pub_crate)]
    async fn run(settings: SettingsSpec) {
        let lsp = settings.lsp == Some(true);
        let _ce = color_eyre::install();
        immt_system::initialize(settings);
        if lsp {
          
        } else {
            tokio::select! {
              () = immt::server::run() => {},
              _ = tokio::signal::ctrl_c() => std::process::exit(0)
            }
        }
    }

    let settings = settings::get_settings();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to initialize Tokio runtime")
        .block_on(run(settings));
}

#[cfg(feature = "hydrate")]
const fn main() {}
