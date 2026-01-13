mod item;
mod user;

use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();

    let db_str = env::var("db_str").unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_str)
        .await?;

    user::create(&pool).await;
    item::create(&pool).await;

    Ok(())
}
