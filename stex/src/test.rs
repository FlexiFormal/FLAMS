use std::path::Path;
use immt_system::utils::parse::SourceOffsetLineCol;

fn check_file(path:&Path) {
    let contents = std::fs::read_to_string(path).unwrap();
    let tokenizer = crate::quickparse::tokenizer::TeXTokenizer::<SourceOffsetLineCol>::new(&contents,Some(path),&());
    let v = tokenizer.collect::<Vec<_>>();
    println!("Done: {}",v.len())
}

#[test]
fn check_metathy() {
    check_file(Path::new("/home/jazzpirate/work/MathHub/sTeX/meta-inf/source/Metatheory.en.tex"))
}