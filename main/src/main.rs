use std::path::{Path, PathBuf};

use immt_controller::BaseController;

/*
fn main() {
    unsafe { backtrace_on_stack_overflow::enable() };
    let runtime = tokio::runtime::Builder::new_multi_thread()
        //.thread_stack_size(8 * 1024 * 1024)
        .thread_name("main")
        .enable_all().build().unwrap();
    runtime.block_on(main_async());
}
*/
#[tokio::main]
async fn main() {
    let _ce = color_eyre::install();
    let settings = immt::settings::get(immt::cli::Cli::get());

    let ctrl = BaseController::new(settings);
    let server = async {
        #[cfg(debug_assertions)]
        let basepath = {
            if std::env::var("LEPTOS_OUTPUT_NAME").is_err() {
                unsafe{std::env::set_var("LEPTOS_OUTPUT_NAME", "immt")};
                "target/web"
            } else {
                "target/web"
            }
        };
        #[cfg(not(debug_assertions))]
            let _basepath = std::env::current_exe().unwrap().parent().unwrap().join("web");
        #[cfg(not(debug_assertions))]
            let basepath = _basepath.to_str().unwrap();
        immt_web::server::run_server(basepath).await.unwrap()
    };
    tokio::select!{
        _ = server => (),
        _ = tokio::signal::ctrl_c() => std::process::exit(0)
    }

}



/*
fn copy_relational() {
    use std::path::{Path,PathBuf};
    use std::fs::File;
    use std::io::BufReader;
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).try_init();
    immt_api::utils::time(|| {
        let dir = Path::new("/home/jazzpirate/temp/dbtest/nquads");
        for e in walkdir::WalkDir::new(dir).min_depth(1).into_iter().filter_map(|e| e.ok()) {
            match e.path().extension() {
                Some(ext) if ext == "nq" => (),
                _ => continue
            }
            let path = e.path();
            let outpath = format!("/home/jazzpirate/work/MathHub{}/.immt",&path.parent().unwrap().to_str().unwrap()[dir.to_str().unwrap().len()..]);
            let outpath = PathBuf::from(outpath);
            if !outpath.exists() {continue}
            let ps = outpath.to_str().unwrap();
            let id = &ps["/home/jazzpirate/work/MathHub/".len()..ps.len()-"/.immt".len()];
            let outpath = outpath.join("rel.ttl");
            tracing::info!("Loading nquads from {}",path.display());
            let file = File::open(path).unwrap();
            let buf = BufReader::new(file);
            let reader = oxigraph::io::RdfParser::from_format(oxigraph::io::RdfFormat::NQuads);
            let mut triples = Vec::new();
            reader.parse_read(buf).for_each(|t| {
                if let Ok(oxigraph::model::Quad { subject, predicate, object,..}) = t {
                    triples.push(oxigraph::model::Triple { subject, predicate, object });
                }
            });
            tracing::info!("Loaded {} triples",triples.len());
            let out = File::create(outpath).unwrap();
            let mut writer = oxigraph::io::RdfSerializer::from_format(oxigraph::io::RdfFormat::Turtle)
                .with_prefix("ulo","http://mathhub.info/ulo").unwrap()
                .with_prefix("schema",format!("http://mathhub.info/{}",id)).unwrap()
                .serialize_to_write(out);
            for t in triples {
                writer.write_triple(&t).unwrap();
            }
            tracing::info!("Wrote triples");
        }
    },"parsing nquads");
}

 */