use sqlx::Pool;
use sqlx::postgres::Postgres;

pub async fn create(pool: &Pool<Postgres>) {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS items (
            id SERIAL PRIMARY KEY,
            user_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            price DOUBLE PRECISION NOT NULL,

            CONSTRAINT fk_items_user
                    FOREIGN KEY (user_id)
                    REFERENCES users (id)
        )"#,
    )
    .execute(pool)
    .await
    .unwrap();
}
