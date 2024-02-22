use immt_system::controller::Controller;
use immt_system::utils::measure;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::level_filters::LevelFilter;
use tracing::{info, instrument, Level};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::layer::SubscriberExt;
#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mh = get_mathhub();
    let (mut ui, _guard) = tracing();
    let controller = {
        let mut builder = Controller::builder(mh);
        immt_stex::register(&mut builder);
        builder.build()
    };
    //test_progress();
    ui.run(controller).unwrap();
}

use clap::{Parser, Subcommand};
use immt_system::utils::progress::{ProgressBar, ProgressLayer};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Whether to start the server. Note that the server can also be started from the UI.
    #[clap(long)]
    #[arg(default_value_t = false)]
    server: bool,
    /// The host to bind the server to.
    #[clap(long)]
    #[arg(default_value_t = {"http://localhost".to_string()})]
    server_host: String,
    /// The port to bind the server to.
    #[clap(long)]
    #[arg(default_value_t = 7070)]
    server_port: u16,
    #[clap(long)]
    #[arg(default_value_t = false)]
    lsp: bool,
}

fn tracing() -> (immt_ui::ui::Ui, Option<WorkerGuard>) {
    let now = chrono::Local::now();
    let file_appender = get_logdir().map(|f| {
        tracing_appender::rolling::RollingFileAppender::builder()
            .rotation(tracing_appender::rolling::Rotation::DAILY)
            .filename_prefix(now.format("%Y-%m-%d-%H.%M.%S").to_string())
            .build(f.display().to_string())
            .expect("failed to initialize file logging")
    });
    let layer_guard = file_appender.map(tracing_appender::non_blocking);
    let (ui, uilayer) = immt_ui::ui::Ui::new();
    let guard = match layer_guard {
        None => {
            tracing::subscriber::set_global_default(
                tracing_subscriber::registry()
                    .with(uilayer)
                    .with(ProgressLayer::default()),
            )
            .unwrap();
            None
        }
        Some((file_layer, guard)) => {
            let subscriber = tracing_subscriber::registry()
                .with(
                    tracing_subscriber::fmt::Layer::default()
                        .with_writer(file_layer.with_max_level(Level::DEBUG))
                        .with_ansi(false)
                        .with_file(false)
                        .with_line_number(false)
                        .json(),
                )
                .with(uilayer)
                .with(ProgressLayer::default());
            tracing::subscriber::set_global_default(subscriber).unwrap();
            Some(guard)
        }
    };
    // TODO
    (ui, guard)
}

fn get_logdir() -> Option<PathBuf> {
    if let Ok(f) = std::env::var("IMMT_LOGDIR") {
        let path = PathBuf::from(f);
        if std::fs::create_dir_all(&path).is_ok() {
            return Some(path);
        }
    }
    if let Ok(p) = std::env::current_exe() {
        if let Some(path) = p.parent().map(|p| p.join("logs")) {
            if std::fs::create_dir_all(&path).is_ok() {
                return Some(path);
            }
        }
    }
    let path = std::env::temp_dir().join("immt").join("logs");
    if let Ok(_) = std::fs::create_dir_all(&path) {
        return Some(path);
    }
    None
}

fn get_mathhub() -> PathBuf {
    let mh = immt_api::mathhub();
    if !mh.exists() {
        std::fs::create_dir_all(&mh).expect("Could not create non-existent MathHub directory")
    }
    // TODO:initialize?
    mh
}

#[instrument(level = "info", name = "test one", skip_all)]
fn test_progress() {
    let pb = immt_system::utils::progress::in_progress("Test progress", 10, false, "Loading...");
    let span = tracing::Span::current();
    std::thread::spawn(move || {
        let _span = span.enter();
        for i in 0..10 {
            std::thread::sleep(Duration::from_millis(100));
            test_progress2((i + 1) * 100, pb);
        }
    });
}

#[instrument(level = "info", name = "test two", skip_all)]
fn test_progress2(millis: u64, outer: Option<ProgressBar>) {
    let pb = immt_system::utils::progress::in_progress("Test progress 2", 100, true, "Loading...");
    let span = tracing::Span::current();
    std::thread::spawn(move || {
        let _span = span.enter();
        for _ in 0..100 {
            std::thread::sleep(Duration::from_millis(millis));
            if let Some(pb) = pb {
                pb.tick()
            }
        }
        if let Some(pb) = outer {
            pb.tick()
        }
    });
}

/*
mod test {
    //mod oxigraph;
    //mod surrealdb;
    //mod indradb;
    //mod cozo;

    use std::io::BufReader;
    use std::path::Path;
    use tracing::event;

    pub fn test() {
        //surrealdb::test().await;
        //oxigraph::test();
        //indradb::test().await;
        //cozo::test().await;
    }
/*
    pub fn rdfreadtest() {
        let f = Path::new("/home/jazzpirate/work/MathHub/sTeX/MathTutorial/relational/orders/MeetJoinSemilattice.en.brf");
        let file = std::fs::File::open(f).unwrap();
        let mut reader = oxbinaryrdf::BinaryRDFParser::default().parse_read(BufReader::new(file));
        for t in reader {
            println!("{}",t.unwrap());
        }
        let mut triples = Vec::new();
        measure("parsing nquads",|| {
            let dir = Path::new("/home/jazzpirate/temp/dbtest/nquads");
            event!(tracing::Level::INFO, "Loading nquads from {}",dir.display());
            let mut fs = 0;
            for e in walkdir::WalkDir::new(dir).min_depth(1).into_iter().filter_map(|e| e.ok()) {
                match e.path().extension() {
                    Some(ext) if ext == "nq" => (),
                    _ => continue
                }
                let path = e.path();
                let file = std::fs::File::open(path).unwrap();
                let buf = BufReader::new(file);
                let reader = oxigraph::io::RdfParser::from_format(oxigraph::io::RdfFormat::NQuads);
                fs += 1;
                reader.parse_read(buf).for_each(|t| {
                    if let Ok(t) = t {
                        triples.push(t);
                    }
                })
            }
            event!(tracing::Level::INFO, "Loaded {} triples from {} files",triples.len(),fs);
        });
        let mut triples = Vec::new();
        measure("parsing brf",|| {
            let dir = Path::new("/home/jazzpirate/temp/dbtest/brf");
            event!(tracing::Level::INFO, "Loading brf from {}",dir.display());
            let mut fs = 0;
            for e in walkdir::WalkDir::new(dir).min_depth(1).into_iter().filter_map(|e| e.ok()) {
                match e.path().extension() {
                    Some(ext) if ext == "brf" => (),
                    _ => continue
                }
                let path = e.path();
                let file = std::fs::File::open(path).unwrap();
                let buf = BufReader::new(file);
                let reader = oxigraph::io::RdfParser::from_format(oxigraph::io::RdfFormat::BinaryRDF);
                fs += 1;
                reader.parse_read(buf).for_each(|t| {
                    if let Ok(t) = t {
                        triples.push(t);
                    }
                })
            }
            event!(tracing::Level::INFO, "Loaded {} triples from {} files",triples.len(),fs);
        });
    }
    pub fn rdfstoretest() {
        use oxigraph::store::Store;
        measure("oxigraph loading nquads",|| {
            let store = Store::new().unwrap();
            let reader = store.bulk_loader().on_progress(|u| println!("{}%",u));
            let dir = Path::new("/home/jazzpirate/temp/dbtest/nquads");
            for e in walkdir::WalkDir::new(dir).min_depth(1).into_iter().filter_map(|e| e.ok()) {
                match e.path().extension() {
                    Some(ext) if ext == "nq" => (),
                    _ => continue
                }
                let path = e.path();
                let file = std::fs::File::open(path).unwrap();
                let buf = BufReader::new(file);
                reader.load_from_read(oxigraph::io::RdfParser::from_format(oxigraph::io::RdfFormat::NQuads),buf).unwrap();
            }
            event!(tracing::Level::INFO, "Loaded {} triples",store.len().unwrap());
        });
        measure("oxigraph loading brf",|| {
            let store = Store::new().unwrap();
            let reader = store.bulk_loader().on_progress(|u| println!("{}%",u));
            let dir = Path::new("/home/jazzpirate/temp/dbtest/brf");
            for e in walkdir::WalkDir::new(dir).min_depth(1).into_iter().filter_map(|e| e.ok()) {
                match e.path().extension() {
                    Some(ext) if ext == "brf" => (),
                    _ => continue
                }
                let path = e.path();
                let file = std::fs::File::open(path).unwrap();
                let buf = BufReader::new(file);
                reader.load_from_read(oxigraph::io::RdfParser::from_format(oxigraph::io::RdfFormat::BinaryRDF),buf).unwrap();
            }
            event!(tracing::Level::INFO, "Loaded {} triples",store.len().unwrap());
        });
    }

    pub fn ulo_roundtrip() {
        use immt_api::ontology::rdf::*;
        let mut triples = Vec::new();
        triples.extend_from_slice(dc::TRIPLES);
        triples.extend_from_slice(owl::TRIPLES);
        triples.extend_from_slice(ulo2::TRIPLES);
        let mut triples = triples.into_iter().map(|t| t.into_owned()).collect::<Vec<_>>();
/*
        measure("parsing nquads",|| {
            let dir = Path::new("/home/jazzpirate/temp/dbtest/nquads");
            event!(tracing::Level::INFO, "Loading nquads from {}",dir.display());
            let mut fs = 0;
            for e in walkdir::WalkDir::new(dir).min_depth(1).into_iter().filter_map(|e| e.ok()) {
                match e.path().extension() {
                    Some(ext) if ext == "nq" => (),
                    _ => continue
                }
                let path = e.path();
                let file = std::fs::File::open(path).unwrap();
                let buf = BufReader::new(file);
                let reader = oxigraph::io::RdfParser::from_format(oxigraph::io::RdfFormat::NQuads);
                fs += 1;
                reader.parse_read(buf).for_each(|t| {
                    if let Ok(Quad { subject, predicate, object,..}) = t {
                        triples.push(Triple { subject, predicate, object });
                    }
                })
            }
            event!(tracing::Level::INFO, "Loaded {} triples from {} files",triples.len(),fs);
        });
*/
        let mut out = Vec::new();
        let mut out_quads = Vec::new();

        measure("roundtrip triples -> turtle",|| {
            let ser = oxigraph::io::RdfSerializer::from_format(oxigraph::io::RdfFormat::Turtle)
               ;// .with_prefix("schema",ulo2::NS.as_str().to_string()).unwrap();
            let mut ser = ser.serialize_to_write(&mut out);
            for t in &triples {
                ser.write_triple(t).unwrap()
            };
            ser.finish().unwrap();
        });
        println!("{}",std::str::from_utf8(out.as_slice()).unwrap());
        event!(tracing::Level::INFO, "Serialized to {} bytes",out.len());
        measure("roundtrip turtle -> triples",|| {
            let read = oxigraph::io::RdfParser::from_format(oxigraph::io::RdfFormat::Turtle)
                .with_base_iri(ulo2::NS.as_str().to_string()).unwrap();
            let mut read = read.parse_read(out.as_slice());
            while let Some(Ok(t)) = read.next() {
                out_quads.push(t);
            };
        });
        event!(tracing::Level::INFO, "retrieved {} quads",out_quads.len());
        out.clear();
        measure("roundtrip triples -> nquads",|| {
            let ser = oxigraph::io::RdfSerializer::from_format(oxigraph::io::RdfFormat::NQuads)
                .with_prefix("schema",ulo2::NS.as_str().to_string()).unwrap();
            let mut ser = ser.serialize_to_write(&mut out);
            for t in &triples {
                ser.write_triple(t).unwrap()
            };
            ser.finish().unwrap();
        });
        event!(tracing::Level::INFO, "Serialized to {} bytes",out.len());
        out_quads.clear();
        measure("roundtrip nquads -> triples",|| {
            let read = oxigraph::io::RdfParser::from_format(oxigraph::io::RdfFormat::NQuads)
                .with_base_iri(ulo2::NS.as_str().to_string()).unwrap();
            let mut read = read.parse_read(out.as_slice());
            while let Some(Ok(t)) = read.next() {
                out_quads.push(t);
            };
        });
        event!(tracing::Level::INFO, "retrieved {} quads",out_quads.len());
        out.clear();
        measure("roundtrip triples -> binary",|| {
            let ser = oxigraph::io::RdfSerializer::from_format(oxigraph::io::RdfFormat::BinaryRDF)
                .with_prefix("schema",ulo2::NS.as_str().to_string()).unwrap();
            let mut ser = ser.serialize_to_write(&mut out);
            for t in &triples {
                ser.write_triple(t).unwrap()
            };
            ser.finish().unwrap();
        });
        event!(tracing::Level::INFO, "Serialized to {} bytes",out.len());
        out_quads.clear();
        measure("roundtrip binary -> triples",|| {
            let read = oxigraph::io::RdfParser::from_format(oxigraph::io::RdfFormat::BinaryRDF)
                .with_base_iri(ulo2::NS.as_str().to_string()).unwrap();
            let mut read = read.parse_read(out.as_slice());
            while let Some(Ok(t)) = read.next() {
                out_quads.push(t);
            };
        });
        event!(tracing::Level::INFO, "retrieved {} quads",out_quads.len());
    }

 */
}

//#[tokio::main]
fn main_old() {
    use std::io;
    //use tui::{backend::CrosstermBackend, Terminal};

    let stdout = io::stdout();
    //let backend = CrosstermBackend::new(stdout);
    //let mut terminal = Terminal::new(backend).unwrap();


    // simple:

    /*tracing_subscriber::FmtSubscriber::builder()
        .compact()
        //.pretty()
        .with_ansi(true)
        .with_file(false)
        .with_line_number(false)
        .with_level(true)
        .with_thread_names(false)
        .with_thread_ids(false)
        .with_max_level(tracing::Level::INFO)
        .with_target(true)
        .init(); */

    use tracing_subscriber::layer::Layer;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    let indicatif_layer = tracing_indicatif::IndicatifLayer::new();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
            //.compact()
            //.pretty()
            //.with_ansi(true)
            .with_file(false)
            .with_line_number(false)
            .with_level(true)
            .with_thread_names(false)
            .with_thread_ids(false)
            .with_target(true)
            .with_writer(indicatif_layer.get_stdout_writer())
            .with_filter(LevelFilter::INFO)
        )
        .with(indicatif_layer)
        //.with_max_level(tracing::Level::INFO)
        .init();

    //test::rdfreadtest();
    //test::rdfstoretest();

    //ulo_roundtrip();
    //copy_shit()
    archives();
    //test::test()//.await;
}



fn archives() {
    use rayon::prelude::*;
    //env_logger::builder().filter_level(log::LevelFilter::Info).try_init();//.unwrap();
    let controller = /*measure("archive manager",||*/ {
        let mut builder = Controller::builder(Path::new("/home/jazzpirate/work/MathHub"));
        immt_stex::register(&mut builder);
        builder.build()
        //tracing::info!("Found {} archives",mgr.into_iter().count());
    };//);
    std::thread::sleep(Duration::from_secs(60))
    //controller.run_repl();
    /*
    let f = |_| {std::thread::sleep(std::time::Duration::from_secs_f32(0.2))};
    measure("iterating single threaded",|| {
        for a in controller.archives().iter() {
            f(a);
        }
    });
    measure("iterating parallel",|| {
        controller.archives().par_iter().for_each(f);
    });

     */
}


use reedline_repl_rs::{Repl,Result as Res,clap::{Command,Arg,ArgMatches}};

#[instrument]
fn copy_shit() {
    measure("parsing nquads",|| {
        let dir = Path::new("/home/jazzpirate/temp/dbtest/nquads");
        for e in walkdir::WalkDir::new(dir).min_depth(1).into_iter().filter_map(|e| e.ok()) {
            match e.path().extension() {
                Some(ext) if ext == "nq" => (),
                _ => continue
            }
            let path = e.path();
            let outpath = format!("/home/jazzpirate/work/MathHub{}/.out",&path.parent().unwrap().to_str().unwrap()[dir.to_str().unwrap().len()..]);
            let outpath = PathBuf::from(outpath);
            if !outpath.exists() {continue}
            let ps = outpath.to_str().unwrap();
            let id = &ps["/home/jazzpirate/work/MathHub/".len()..ps.len()-"/.out".len()];
            let outpath = outpath.join("rel.ttl");
            info!("Loading nquads from {}",path.display());
            let file = File::open(path).unwrap();
            let buf = BufReader::new(file);
            let reader = oxigraph::io::RdfParser::from_format(oxigraph::io::RdfFormat::NQuads);
            let mut triples = Vec::new();
            reader.parse_read(buf).for_each(|t| {
                if let Ok(oxigraph::model::Quad { subject, predicate, object,..}) = t {
                    triples.push(oxigraph::model::Triple { subject, predicate, object });
                }
            });
            info!("Loaded {} triples",triples.len());
            let out = File::create(outpath).unwrap();
            let mut writer = oxigraph::io::RdfSerializer::from_format(oxigraph::io::RdfFormat::Turtle)
                .with_prefix("ulo","http://mathhub.info/ulo").unwrap()
                .with_prefix("schema",format!("http://mathhub.info/{}",id)).unwrap()
                .serialize_to_write(out);
            for t in triples {
                writer.write_triple(&t).unwrap();
            }
            info!("Wrote triples");
        }
    });
}

 */
