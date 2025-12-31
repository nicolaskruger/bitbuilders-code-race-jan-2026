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

    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool)
        .await?;

    print!("{}", row.0);

    assert_eq!(row.0, 150);

    Ok(())
}
