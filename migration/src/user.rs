use sqlx::Pool;
use sqlx::postgres::Postgres;

pub async fn create(pool: &Pool<Postgres>) {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            password TEXT NOT NULL
        )"#,
    )
    .execute(pool)
    .await
    .unwrap();
}
