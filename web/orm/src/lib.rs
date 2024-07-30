pub use sea_orm_migration::prelude::*;
pub mod entities;

mod m20220101_000001_create_table;
//mod m20240725_081500_logins;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20220101_000001_create_table::Migration)]
    }
}

use sea_orm::{EnumIter,DeriveActiveEnum};
#[derive(Iden, EnumIter,DeriveActiveEnum,Clone,Debug,PartialEq,Eq)]
#[sea_orm(rs_type="u8",db_type="Integer")]
pub enum Rights {
    Full = 0,
    None = 1
}
