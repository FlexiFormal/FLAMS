use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;


pub async fn test() {
    let db = surrealdb::engine::any::connect("mem://").await.unwrap();
    //let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("http://example.com#a").use_db("test").await.unwrap();
    let id = surrealdb::sql::Id::rand();
    println!("id: {:?}",id);
    let created: Vec<Record> = db
        .create("person")
        .content(Person {
            title: "Founder & CEO".to_string(),
            name: Name {
                first: "Tobie".to_string(),
                last: "Morgan Hitchcock".to_string(),
            },
            friends_with: vec![id.clone()],
            marketing: true,
        })
        .await.unwrap();
    println!("Created: {:?}", created);
    db.use_ns("http://example.com#b").use_db("test").await.unwrap();

    let created: Option<Record> = db
        .create(("person",id))
        .content(Person {
            title: "Master".to_string(),
            name: Name {
                first: "Hans".to_string(),
                last: "Dampf".to_string(),
            },
            friends_with: vec![],
            marketing: false,
        })
        .await.unwrap();
    println!("Created: {:?}", created);
    let people: Vec<Person> = db.select("person").await.unwrap();
    println!("People: {:?}", people);
    println!("HERE");
}

#[derive(Debug, Serialize,Deserialize)]
struct Name {
    first: String,
    last: String,
}

#[derive(Debug, Serialize,Deserialize)]
struct Person {
    title: String,
    name: Name,
    marketing: bool,
    friends_with:Vec<Id>
}

#[derive(Debug, Serialize)]
struct Responsibility {
    marketing: bool,
}

use surrealdb::sql::{Id, Thing};

#[derive(Debug, Deserialize)]
struct Record {
    #[allow(dead_code)]
    id: Thing,
}