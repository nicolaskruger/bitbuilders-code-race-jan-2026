use actix_web::{
    HttpResponse, Responder, post,
    web::{self, Json},
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use bcrypt;

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

    async fn match_password(&self, user: &UserDto) -> bool {
        let fetch_user = self.user_repo.fetch_by_name(&user.name).await;
        bcrypt::verify(&user.password, &fetch_user.password).unwrap_or(false)
    }
}

impl<R: IUserRepo> IAuthService for AuthService<R> {
    async fn auth(&self, user: &UserDto) -> Result<AuthResponse, String> {
        if !self.user_repo.exists(&user.name).await {
            Err(String::from("User de not exists."))
        } else if !self.match_password(user).await {
            Err(String::from("Miss match password."))
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

    struct MockUserRepo<'a> {
        mock_exists: bool,
        fetch_user: Option<&'a user::UserDto>,
    }

    impl IUserRepo for MockUserRepo<'_> {
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
            let user = self.fetch_user.unwrap();
            let dto = user::UserDto {
                name: user.name.clone(),
                password: user.password.clone(),
            };
            async move { dto }
        }
    }

    #[tokio::test]
    async fn auth_error_when_user_do_not_exists() {
        let mock_repo = MockUserRepo {
            mock_exists: false,
            fetch_user: None,
        };

        let service = AuthService::new(mock_repo);

        let dto = UserDto {
            name: String::from("nk"),
            password: String::from("123"),
        };

        let res: Result<_, _> = service.auth(&dto).await;

        assert!(res.is_err(), "Not created because it already exists.");
    }

    #[tokio::test]
    async fn wrong_password() {
        let fetch_user = user::UserDto {
            name: String::from("nk"),
            password: String::from("456"),
        };

        let mock_repo = MockUserRepo {
            mock_exists: true,
            fetch_user: Some(&fetch_user),
        };

        let service = AuthService::new(mock_repo);

        let dto = UserDto {
            name: String::from("nk"),
            password: String::from("123"),
        };

        let res: Result<_, _> = service.auth(&dto).await;

        assert!(res.is_err(), "Miss match password");
    }
}
