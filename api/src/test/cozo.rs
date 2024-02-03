use cozo::*;

pub async fn test() {
    let db = cozo::Db::new(MemStorage::default()).unwrap();
    db.initialize().unwrap();
    let script = "?[a] := a in [1, 2, 3]";
    let result = db.run_script(script, Default::default(), ScriptMutability::Immutable).unwrap();
    println!("{:?}", result);
}