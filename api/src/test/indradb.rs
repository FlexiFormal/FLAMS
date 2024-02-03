use indradb;
pub async fn test() {
    let db: indradb::Database<indradb::MemoryDatastore> = indradb::MemoryDatastore::new_db();
    let in_v = indradb::Vertex::new("test");
}