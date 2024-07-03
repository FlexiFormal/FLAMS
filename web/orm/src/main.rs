use sea_orm_migration::prelude::*;
use immt_web_orm::Migrator;

#[async_std::main]
async fn main() {
    cli::run_cli(Migrator).await;
}
