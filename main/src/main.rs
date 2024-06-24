use std::path::{Path, PathBuf};

#[cfg(feature="async")]
use immt_controller::BaseControllerAsync;
#[cfg(not(feature="async"))]
use immt_controller::BaseController;
use clap::{Parser, Subcommand};

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
    //copy();
    dotenv::dotenv().ok();
/*
    let metrics = tokio::runtime::Handle::current().metrics();

    println!("Runtime is using {} workers, {} blocking threads, {} of which are idle",
             metrics.num_workers(),
        metrics.num_blocking_threads(),
        metrics.num_idle_blocking_threads()
    );*/
    #[cfg(feature="async")]
    println!("Async controller");
    #[cfg(not(feature="async"))]
    println!("Sync controller");

    let cli = Cli::parse();
    #[cfg(feature="async")]
    let ctrl = match cli.mathhubs {
        None =>  BaseControllerAsync::new(cli.debug,immt_api::MATHHUB_PATHS.iter().map(|s| s.as_path())).await,
        Some(s) => BaseControllerAsync::new(cli.debug,s.split(',').map(Path::new)).await
    };
    #[cfg(not(feature="async"))]
        let ctrl = match cli.mathhubs {
        None =>  BaseController::new(cli.debug,immt_api::MATHHUB_PATHS.iter().map(|s| s.as_path())),
        Some(s) => BaseController::new(cli.debug,s.split(',').map(Path::new))
    };
    let server = match (cli.ip,cli.port) {
        (Some(addr),Some(port)) => {
            Some((addr,port))
        },
        (_,Some(port)) => {
            Some(("127.0.0.1".to_string(),port))
        },
        (Some(ip),_) => {
            Some((ip,8080))
        },
        _ => None
    };
    let tui = async {
        /*if cli.tui {
            todo!()
        }*/
    };
    let server = async {
        if let Some((ip,port)) = server {
            #[cfg(debug_assertions)]
            let basepath = "target/web";
            #[cfg(not(debug_assertions))]
            let _basepath = std::env::current_exe().unwrap().parent().unwrap().join("web");
            #[cfg(not(debug_assertions))]
            let basepath = _basepath.to_str().unwrap();
            immt_web::server::run_server(ip,port,basepath).await.unwrap()
        } else {
            let ip = "127.0.0.1";
            let port = 3000;
            #[cfg(debug_assertions)]
                let basepath = "../target/web";
            #[cfg(not(debug_assertions))]
                let _basepath = std::env::current_exe().unwrap().parent().unwrap().join("web");
            #[cfg(not(debug_assertions))]
                let basepath = _basepath.to_str().unwrap();
            immt_web::server::run_server(ip,port,basepath).await.unwrap()
        }
    };
    let gui = async {
        /*if !cli.gui_off {
            todo!()
        }*/
    };
    tokio::select!{
        _ = async move {tokio::join!(tui,server,gui)} => (),
        _ = tokio::signal::ctrl_c() => std::process::exit(0)
    }
        /*,async move {loop {
            let metrics = tokio::runtime::Handle::current().metrics();
            println!("Runtime is using {} workers, {} blocking threads, {} of which are idle. Tasks: {}",
                     metrics.num_workers(),
                metrics.num_blocking_threads(),
                metrics.num_idle_blocking_threads(),
                metrics.active_tasks_count()
            );
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }}*/

}


#[derive(Parser)]
#[command(propagate_version = true, version, about, long_about = Some(
"iMᴍᴛ - Generic knowledge management system for flexiformal knowledge\n\
--------------------------------------------------------------------\n\
See the \u{1b}]8;;https://github.com/UniFormal/MMT\u{1b}\\documentation\u{1b}]8;;\u{1b}\\ for details"
))]
struct Cli {
    /// a comma-separated list of MathHub paths (if not given, the default paths are used
    /// as determined by the MATHHUB system variable or ~/.mathhub/mathhub.path)
    #[arg(short, long)]
    mathhubs: Option<String>,

    /// whether to enable debug logging
    #[arg(short, long)]
    debug:bool,

    /// turn off the GUI
    #[arg(short,long)]
    gui_off:bool,

    /// whether to start the TUI
    #[arg(short,long)]
    tui:bool,

    /// Network port to use for the server
    #[arg(long,value_parser = clap::value_parser!(u16).range(1..))]
    port: Option<u16>,

    /// Network address to use for the server
    #[arg(long)]
    ip: Option<String>,
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