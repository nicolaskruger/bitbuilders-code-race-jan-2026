use actix_web::{
    HttpResponse, Responder, post,
    web::{self, Json},
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::user::IUserRepo;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthResponse {
    pub token: String,
    pub refresh: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct UserDto {
    name: String,
    password: String,
}

#[post("/user/auth")]
pub async fn auth(pool: web::Data<Pool<Postgres>>, body: Json<UserDto>) -> impl Responder {
    HttpResponse::Unauthorized()
}

trait IAuthService {
    async fn auth(&self, user: &UserDto) -> Result<AuthResponse, String>;
}

struct AuthService<UserRepo: IUserRepo> {
    user_repo: UserRepo,
}

impl<UserRepo: IUserRepo> AuthService<UserRepo> {
    fn new(user_repo: UserRepo) -> Self {
        Self { user_repo }
    }
}

impl<R: IUserRepo> IAuthService for AuthService<R> {
    async fn auth(&self, user: &UserDto) -> Result<AuthResponse, String> {
        if self.user_repo.exists(&user.name).await == false {
            Err(String::from("User de not exists"))
        } else {
            todo!()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::user;

    use super::*;
    use dotenvy::dotenv;
    use sqlx::postgres::PgPoolOptions;
    use std::env;

    struct MockUserRepo {
        mock_exists: bool,
    }

    impl IUserRepo for MockUserRepo {
        async fn exists(&self, _: &str) -> bool {
            self.mock_exists
        }

        async fn password_hash(&self, _: &str) -> String {
            String::from("pass")
        }

        async fn register(&self, _: &user::UserDto) {
            todo!()
        }

        fn fetch_by_name(&self, _: &str) -> impl Future<Output = user::UserDto> {
            async move { todo!() }
        }
    }

    #[tokio::test]
    async fn auth_error_when_user_do_not_exists() {
        let mock_repo = MockUserRepo { mock_exists: false };

        let service = AuthService::new(mock_repo);

        let dto = UserDto {
            name: String::from("nk"),
            password: String::from("123"),
        };

        let res: Result<_, _> = service.auth(&dto).await;

        assert!(res.is_err(), "Not created because it already exists.");
    }
}
