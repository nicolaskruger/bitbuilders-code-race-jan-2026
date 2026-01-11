use async_trait::async_trait;

use crate::{
    contract::{repo::user_repo_trait::IUserRepo, service::user_service_trait::IUserService},
    entity::user_entity::UserRegister,
};

pub struct UserService<R: IUserRepo> {
    repo: R,
}

impl<R: IUserRepo> UserService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<R: IUserRepo + Sync> IUserService for UserService<R> {
    async fn register(&self, dto: &UserRegister) -> Result<(), String> {
        if self.repo.exists(&dto.name).await {
            Err(String::from("Not created because it already exists."))
        } else {
            let dto = UserRegister {
                name: dto.name.clone(),
                password: self.repo.password_hash(&dto.password).await,
            };
            self.repo.register(&dto).await;
            Ok(())
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::{entity::user_entity::UserFetched, repo::user_repo::UserRepo};

    use super::*;
    use async_trait::async_trait;
    use dotenvy::dotenv;
    use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
    use std::env;

    struct MockUserRepo {
        mock_exists: bool,
    }

    #[async_trait]
    impl IUserRepo for MockUserRepo {
        async fn exists(&self, _: &str) -> bool {
            self.mock_exists
        }

        async fn password_hash(&self, _: &str) -> String {
            String::from("pass")
        }

        async fn register(&self, dto: &UserRegister) {
            assert_eq!(dto.name, "nk");
            assert_eq!(dto.password, "pass");
        }

        async fn fetch_by_name(&self, _: &str) -> UserFetched {
            todo!()
        }

        async fn fetch_by_id(&self, _: i32) -> UserFetched {
            todo!()
        }
    }

    #[tokio::test]
    async fn error_on_create_existing_user() {
        let mock_repo = MockUserRepo { mock_exists: true };

        let service = UserService { repo: mock_repo };

        let dto = UserRegister {
            name: String::from("nk"),
            password: String::from("123"),
        };

        let result = service.register(&dto).await;

        assert!(result.is_err(), "Not created because it already exists.");
    }

    #[tokio::test]
    async fn create_user() {
        let mut _dto = UserRegister {
            name: String::from(""),
            password: String::from(""),
        };

        let mock_repo = MockUserRepo { mock_exists: false };

        let service = UserService { repo: mock_repo };

        let dto = UserRegister {
            name: String::from("nk"),
            password: String::from("123"),
        };

        let result = service.register(&dto).await;

        assert!(result.is_ok(), " Created.");
    }

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

    #[tokio::test]
    #[ignore = "db_test"]
    async fn user_service_register() {
        let pool = load_pool().await;

        let new_user = UserRegister {
            name: String::from("new_user"),
            password: String::from("new_password"),
        };

        let repo = UserRepo::new(&pool);
        let service = UserService::new(repo);

        let res = service.register(&new_user).await;
        assert!(res.is_ok());

        let res = service.register(&new_user).await;
        assert!(res.is_err());

        sqlx::query("DELETE FROM users WHERE name = $1")
            .bind(new_user.name)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore = "db_test"]
    async fn user_repo_fetch() {
        let pool = load_pool().await;

        let new_user = UserRegister {
            name: String::from("new_user"),
            password: String::from("new_password"),
        };

        sqlx::query("DELETE FROM users WHERE name = $1")
            .bind(new_user.name.clone())
            .execute(&pool)
            .await
            .unwrap();

        let repo = UserRepo::new(&pool);
        let service = UserService::new(repo);

        let res = service.register(&new_user).await;
        assert!(res.is_ok());

        let repo = UserRepo::new(&pool);
        let fetch_user = repo.fetch_by_name("new_user").await;

        assert_eq!(fetch_user.name, "new_user");

        sqlx::query("DELETE FROM users WHERE name = $1")
            .bind(new_user.name)
            .execute(&pool)
            .await
            .unwrap();
    }
}
