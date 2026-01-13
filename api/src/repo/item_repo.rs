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
    async fn register(&self, item: &ItemCreate) {
        sqlx::query("INSERT INTO items (name, price, user_id) VALUES ($1, $2, $3)")
            .bind(&item.name)
            .bind(item.price)
            .bind(item.user_id)
            .execute(self.pool)
            .await
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::{entity::user_entity::UserRegister, repo::user_repo::UserRepo};

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
    async fn item_repo_exists() {
        let pool = load_pool().await;

        let item_repo = ItemRepo::new(&pool);

        let user_respo = UserRepo::new(&pool);

        let user = UserRegister {
            name: String::from("my_name"),
            password: String::from("my_password"),
        };

        user_respo.register(&user).await;
        let fetched_user = user_respo.fetch_by_name("my_name").await;

        let item = ItemCreate {
            name: String::from("item - a"),
            price: 1.3,
            user_id: fetched_user.id,
        };

        item_repo.register(&item).await;

        sqlx::query("DELETE FROM items WHERE name = $1")
            .bind(item.name)
            .execute(&pool)
            .await
            .unwrap();
    }
}
