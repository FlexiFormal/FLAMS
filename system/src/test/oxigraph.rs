use std::collections::BTreeSet;
use std::path::Path;
use oxigraph::io::{DatasetFormat, GraphFormat};
use oxigraph::store::Store;
use oxigraph::sparql::{Query, QueryResults, QuerySolution};
use oxigraph::model::*;
use immt_api::utils::measure;

pub fn test() {
    use std::path::Path;
    //std::thread::sleep(std::time::Duration::from_secs(10));
    let db = store_from_dir();
    //std::thread::sleep(std::time::Duration::from_secs(10));
    match db.len() {
        Ok(i) if i > 0 => (),
        _ => load_nquads(&db)
    }
    //std::thread::sleep(std::time::Duration::from_secs(10));
    let len = measure("oxigraph number of triples",|| db.len().unwrap());
    println!("Done. {} triples",len);

    let ainotes = NamedNode::new("http://mathhub.info/MiKoMH/AI/course/notes/notes.omdoc#").unwrap();
    let inds = db.quads_for_pattern(Some(ainotes.as_ref().into()),None,None,None).into_iter().filter_map(|r| r.ok()).collect::<Vec<_>>();
    println!("{} inds",inds.len());

    let res = measure("full query + resolution",|| query(&db));
    println!("{} results",res.len());
}

fn store_from_dir() -> Store {
    measure("oxigraph from dir",|| Store::open("/home/jazzpirate/temp/dbtest/oxigraph").unwrap())
}

fn store_from_mem() -> Store {
    measure("oxigraph from mem",|| Store::new().unwrap())
}

fn load_nquads(store:&Store) {
    let reader = store.bulk_loader().on_progress(|u| println!("{}%",u));
    let dir = Path::new("/home/jazzpirate/temp/dbtest/nquads");
    measure("oxigraph loading nquads",|| {
        for e in walkdir::WalkDir::new(dir).min_depth(1).into_iter().filter_map(|e| e.ok()) {
            match e.path().extension() {
                Some(ext) if ext == "nq" => (),
                _ => continue
            }
            let path = e.path();
            println!("{}",path.display());
            let mut file = std::fs::File::open(path).unwrap();
            let mut buf = std::io::BufReader::new(file);
            reader.load_dataset(buf, DatasetFormat::NQuads,None).unwrap();
        }
    });
}

pub fn query(db:&Store) -> BTreeSet<NamedNode> {
    const QUERY: &str = "SELECT ?x WHERE {
  <http://mathhub.info/MiKoMH/AI/course/notes/notes.omdoc#> (<http://mathhub.info/ulo#crossrefs>|<http://mathhub.info/ulo#specifies>|<http://mathhub.info/ulo#contains>|<http://mathhub.info/ulo#has-language-module>)+ ?x .
  ?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://mathhub.info/ulo#constant> .
}";
    let mut query: Query = QUERY.try_into().unwrap();
    query.dataset_mut().set_default_graph_as_union();
    let res = measure("query",|| {
        db.query(query).unwrap()
    });
    if let QueryResults::Solutions(sol) = res {
        sol.into_iter().filter_map(|r|
            r.ok().map(|r| match r.get(0) {
                Some(Term::NamedNode(n)) => Some(n.clone()),
                _ => None
            }).flatten()
        ).collect::<BTreeSet<_>>()
    } else {
        BTreeSet::new()
    }
}

pub fn db_test() {
    use std::io::Write;
    let db = measure("db_test store from dir",|| Store::open("/home/jazzpirate/dbtest.nt").unwrap());
    let node1 = NamedNode::new("http://example.com#foo").unwrap();
    let node2 = NamedNode::new("http://example.com#bar").unwrap();
    let node3 = NamedNode::new("http://example.com#baz").unwrap();
    let quad = Quad::new(node1.clone(), node2, node3, GraphName::NamedNode(node1.clone()));
    println!("Res: {}", measure("db_test inserting quad",|| db.insert(&quad).unwrap()));
    let mut s = Vec::new();
    db.dump_dataset(&mut s, DatasetFormat::TriG).unwrap();
    match std::str::from_utf8(s.as_slice()) {
        Ok(s) => println!("TriG:\n{}", s),
        Err(e) => println!("Not a string")
    }
    println!("TriG Length: {}", s.len());
    s.clear();
    db.dump_dataset(&mut s, DatasetFormat::NQuads).unwrap();
    match std::str::from_utf8(s.as_slice()) {
        Ok(s) => println!("NQuads:\n{}", s),
        Err(e) => println!("Not a string")
    }
    println!("NQuads Length: {}", s.len());
    s.clear();
    db.dump_graph(&mut s, GraphFormat::NTriples, &node1).unwrap();
    match std::str::from_utf8(s.as_slice()) {
        Ok(s) => println!("NTriples:\n{}", s),
        Err(e) => println!("Not a string")
    }
    println!("Length: {}", s.len());
    s.clear();
    db.dump_graph(&mut s, GraphFormat::Turtle, &node1).unwrap();
    match std::str::from_utf8(s.as_slice()) {
        Ok(s) => println!("Turtle:\n{}", s),
        Err(e) => println!("Not a string")
    }
    println!("Length: {}", s.len());
    s.clear();
    db.dump_graph(&mut s, GraphFormat::RdfXml, &node1).unwrap();
    match std::str::from_utf8(s.as_slice()) {
        Ok(s) => println!("XML:\n{}", s),
        Err(e) => println!("Not a string")
    }
    println!("Length: {}", s.len());
}