use std::path::Path;
use immt_api::backend::Controller;
use immt_api::utils::measure;

mod test {
    //mod oxigraph;
    //mod surrealdb;
    //mod indradb;
    //mod cozo;

    pub fn test() {
        //surrealdb::test().await;
        //oxigraph::test();
        //indradb::test().await;
        //cozo::test().await;
    }
}

//#[tokio::main]
/*async*/ fn main() {
    // tracing_subscriber::FmtSubscriber::default()
    //tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
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
            .init();
    //).unwrap();
    archives();
    test::test()//.await;
}


fn archives() {
    use rayon::prelude::*;
    //env_logger::builder().filter_level(log::LevelFilter::Info).try_init();//.unwrap();
    let controller = measure("archive manager",|| {
        Controller::new(Path::new("/home/jazzpirate/work/MathHub")).build()
        //tracing::info!("Found {} archives",mgr.into_iter().count());
    });
    let f = |_| {std::thread::sleep(std::time::Duration::from_secs_f32(0.2))};
    measure("iterating single threaded",|| {
        for a in controller.archives().into_iter() {
            f(a);
        }
    });
    measure("iterating parallel",|| {
        controller.archives().into_par_iter().for_each(|a| f(a));
    });
}