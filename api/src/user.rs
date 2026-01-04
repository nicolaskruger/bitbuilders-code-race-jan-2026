use actix_web::{HttpResponse, Responder, post, web};
use async_trait::async_trait;
use bcrypt::{DEFAULT_COST, hash};
use serde::{Deserialize, Serialize};
use sqlx::Pool;
use sqlx::postgres::Postgres;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct UserRespose {
    msg: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserDto {
    pub name: String,
    pub password: String,
}

#[post("/user")]
pub async fn create(pool: web::Data<Pool<Postgres>>, body: web::Json<UserDto>) -> impl Responder {
    let repo = UserRepo::new(&pool);
    let service = UserService::new(repo);

    let res = service.register(&body.into_inner()).await;

    match res {
        Ok(_) => HttpResponse::Created().json(UserRespose {
            msg: String::from("user created"),
        }),
        Err(msg) => HttpResponse::BadRequest().json(UserRespose { msg }),
    }
}

pub struct UserRepo<'a> {
    pool: &'a Pool<Postgres>,
}

#[async_trait]
pub trait IUserRepo {
    async fn exists(&self, name: &str) -> bool;
    async fn password_hash(&self, password: &str) -> String;
    async fn register(&self, dto: &UserDto);
    async fn fetch_by_name(&self, name: &str) -> User;
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
    async fn register(&self, dto: &UserDto) {
        sqlx::query("INSERT INTO users (name, password) VALUES ($1, $2)")
            .bind(&dto.name)
            .bind(&dto.password)
            .execute(self.pool)
            .await
            .unwrap();
    }

    async fn fetch_by_name(&self, name: &str) -> User {
        sqlx::query_as::<_, User>(
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
}

trait IUserService {
    async fn register(&self, dto: &UserDto) -> Result<(), String>;
}

struct UserService<R: IUserRepo> {
    repo: R,
}

impl<R: IUserRepo> UserService<R> {
    fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R: IUserRepo> IUserService for UserService<R> {
    async fn register(&self, dto: &UserDto) -> Result<(), String> {
        if self.repo.exists(&dto.name).await {
            Err(String::from("Not created because it already exists."))
        } else {
            let dto = UserDto {
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
    use super::*;
    use dotenvy::dotenv;
    use sqlx::postgres::PgPoolOptions;
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

        async fn register(&self, dto: &UserDto) {
            assert_eq!(dto.name, "nk");
            assert_eq!(dto.password, "pass");
        }

        async fn fetch_by_name(&self, name: &str) -> User {
            todo!()
        }
    }

    #[tokio::test]
    async fn error_on_create_existing_user() {
        let mock_repo = MockUserRepo { mock_exists: true };

        let service = UserService { repo: mock_repo };

        let dto = UserDto {
            name: String::from("nk"),
            password: String::from("123"),
        };

        let result = service.register(&dto).await;

        assert!(result.is_err(), "Not created because it already exists.");
    }

    #[tokio::test]
    async fn create_user() {
        let mut _dto = UserDto {
            name: String::from(""),
            password: String::from(""),
        };

        let mock_repo = MockUserRepo { mock_exists: false };

        let service = UserService { repo: mock_repo };

        let dto = UserDto {
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

        let new_user = UserDto {
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

        let new_user = UserDto {
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

        let new_user = UserDto {
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
