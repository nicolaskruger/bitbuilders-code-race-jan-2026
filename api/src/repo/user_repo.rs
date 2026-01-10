use crate::contract::repo::user_repo_trait::IUserRepo;
use crate::entity::user_entity::{UserFetched, UserRegister};
use async_trait::async_trait;
use bcrypt::{DEFAULT_COST, hash};
use sqlx::Pool;
use sqlx::postgres::Postgres;

pub struct UserRepo<'a> {
    pool: &'a Pool<Postgres>,
}

impl<'a> UserRepo<'a> {
    pub fn new(pool: &'a Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IUserRepo for UserRepo<'_> {
    async fn exists(&self, name: &str) -> bool {
        let exists: (bool,) = sqlx::query_as(
            r#"
        SELECT EXISTS (
            SELECT 1
            FROM users
            WHERE name = $1
        )
        "#,
        )
        .bind(name)
        .fetch_one(self.pool)
        .await
        .unwrap();

        exists.0
    }
    async fn password_hash(&self, password: &str) -> String {
        hash(password, DEFAULT_COST).unwrap()
    }
    async fn register(&self, dto: &UserRegister) {
        sqlx::query("INSERT INTO users (name, password) VALUES ($1, $2)")
            .bind(&dto.name)
            .bind(&dto.password)
            .execute(self.pool)
            .await
            .unwrap();
    }

    async fn fetch_by_name(&self, name: &str) -> UserFetched {
        sqlx::query_as::<_, UserFetched>(
            r#"
            SELECT id, name, password
            FROM users
            WHERE name = $1
        "#,
        )
        .bind(name)
        .fetch_one(self.pool)
        .await
        .unwrap()
    }

    async fn fetch_by_id(&self, id: i32) -> UserFetched {
        sqlx::query_as::<_, UserFetched>(
            r#"
            SELECT id, name, password
            FROM users
            WHERE id = $1
        "#,
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
        .unwrap()
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

        let repo = UserRepo::new(&pool);

        repo.exists(&String::from("test")).await;
    }

    #[tokio::test]
    #[ignore = "db_test"]
    async fn user_repo_creaate_user() {
        let pool = load_pool().await;

        let new_user = UserRegister {
            name: String::from("new_user"),
            password: String::from("new_password"),
        };

        let repo = UserRepo::new(&pool);

        repo.register(&new_user).await;

        let exists = repo.exists(&String::from("new_user")).await;

        assert!(exists, "exists user");

        sqlx::query("DELETE FROM users WHERE name = $1")
            .bind(new_user.name)
            .execute(&pool)
            .await
            .unwrap();
    }
}
