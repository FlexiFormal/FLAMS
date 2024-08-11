fn main() {
    /*
    if let Some(s) = std::env::var("IMMT_BUILD_GRAPHS").ok() {
        if s.eq_ignore_ascii_case("true") {} else { return }
    } else { return }
    let p = if let Ok(p) = std::env::current_dir() {
        if let Some(s) = p.as_os_str().to_str() {
            if s.ends_with("main") {
                p
            } else {
                panic!("Wrong build directory")
            }
        } else {
            panic!("Wrong build directory")
        }
    } else {
        panic!("Wrong build directory")
    };
    let main = p.parent().expect("Wrong build directory").parent().expect("Wrong build directory");
    //println!("cargo::rerun-if-changed=../graphs");
    std::process::Command::new("trunk")
        .env("CARGO_TARGET_DIR",main.join("target"))
        .env("RUSTFLAGS","--cfg=web_sys_unstable_apis")
        .current_dir(main.join("web").join("graphs"))
        .args(vec!["build","--features=client","--release"]).stdout(std::io::stderr()).stderr(std::io::stderr()).status().expect("trunk build failed!");
    let target = main.join("web").join("main").join("assets").join("graphs");
    let _ = std::fs::create_dir_all(&target);
    let mut replaces = vec![
        ("manifest.json".to_string(),"graph_viewer/manifest.json"),
        ("sw.js".to_string(),"graph_viewer/sw.js"),
        ("%%QUERY_URL%%".to_string(),"/api/graph")
    ];
    let mut js = String::new();
    let mut html = String::new();
    for entry in std::fs::read_dir(main.join("web/graphs/dist")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let mut file_name = path.file_name().unwrap().to_str().unwrap();
        if file_name.starts_with("favicon") {
            replaces.push((file_name.to_string(),"graph_viewer/favicon.ico"));
            file_name = "favicon.ico";
        } else if file_name.starts_with("immt-graphs") && file_name.ends_with(".wasm") {
            replaces.push((file_name.to_string(),"graph_viewer/immt-graphs_bg.wasm"));
            file_name = "immt-graphs_bg.wasm";
        } else if file_name.starts_with("immt-graphs") && file_name.ends_with(".js") {
            replaces.push((file_name.to_string(),"graph_viewer/immt-graphs.js"));
            js = std::fs::read_to_string(&path).unwrap();
            continue
        } else if file_name == "index.html" {
            html = std::fs::read_to_string(&path).unwrap();
            continue
        }
        std::fs::copy(&path,&target.join(file_name)).unwrap();
    }
    js = replaces.iter().fold(js,|js,(from,to)| js.replace(from,to));
    html = replaces.iter().fold(html,|html,(from,to)| html.replace(from,to));
    std::fs::write(target.join("immt-graphs.js"),js).unwrap();
    std::fs::write(target.join("index.html"),html).unwrap();

     */
}