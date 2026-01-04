use std::env;

use actix_web::{
    HttpResponse, Responder, post,
    web::{self, Json},
};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use bcrypt;

use crate::user::IUserRepo;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    user_id: i32,
}

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

    async fn token_build(&self, user: &UserDto) -> AuthResponse {
        let fetch_user = self.user_repo.fetch_by_name(&user.name).await;

        let claim = Claims {
            user_id: fetch_user.id,
        };

        let token_sercret = env::var("token_sercret").unwrap_or(String::from("secret"));

        let token = encode(
            &Header::default(),
            &claim,
            &EncodingKey::from_secret(token_sercret.as_ref()),
        )
        .unwrap();

        AuthResponse {
            token,
            refresh: String::from(""),
        }
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
            Ok(self.token_build(user).await)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::user;

    use super::*;
    use bcrypt::{DEFAULT_COST, hash};
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

        fn fetch_by_name(&self, _: &str) -> impl Future<Output = user::User> {
            let user = self.fetch_user.unwrap();
            let dto = user::User {
                id: 123,
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

    #[tokio::test]
    async fn auth_api_fetch_token() {
        let hash_password = hash("123", DEFAULT_COST).unwrap();

        let fetch_user = user::UserDto {
            name: String::from("nk"),
            password: hash_password,
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

        assert!(res.is_ok(), "Match password");

        let auth_token = res.unwrap();

        assert!(!auth_token.token.is_empty());
    }
}
