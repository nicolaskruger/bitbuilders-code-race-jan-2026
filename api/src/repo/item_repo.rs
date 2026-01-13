use async_trait::async_trait;
use sqlx::{Pool, Postgres};

use crate::{
    contract::repo::{item_repo_trait::IItemRepo, user_repo_trait::IUserRepo},
    entity::item_entity::ItemCreate,
};

pub struct ItemRepo<'a> {
    pool: &'a Pool<Postgres>,
}

impl<'a> ItemRepo<'a> {
    pub fn new(pool: &'a Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IItemRepo for ItemRepo<'_> {
    async fn create(&self, item: &ItemCreate) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenvy::dotenv;
    use sqlx::postgres::PgPoolOptions;
    use std::env;

    async fn load_pool() -> Pool<Postgres> {
        dotenv().ok();
        let db_str = env::var("db_str").unwrap();

        PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_str)
            .await
            .unwrap()
    }

    #[tokio::test]
    #[ignore = "db_test"]
    async fn user_repo_exists() {
        let pool = load_pool().await;

        let repo = ItemRepo::new(&pool);

        let item = ItemCreate {
            name: String::from("item - a"),
            price: 1.3,
        };

        repo.create(&item).await;

        sqlx::query("DELETE FROM item WHERE name = $1")
            .bind(item.name)
            .execute(&pool)
            .await
            .unwrap();
    }
}
