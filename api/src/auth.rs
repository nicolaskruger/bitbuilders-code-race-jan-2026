use std::env;

use actix_web::{
    HttpRequest, HttpResponse, Responder,
    http::{StatusCode, header},
    post,
    web::{self, Json},
};
use api::user::{self, User};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

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

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AuthMsg {
    msg: String,
}

pub async fn auth_user(req: HttpRequest) -> Result<user::User, StatusCode> {
    let token = req.headers().get(header::AUTHORIZATION);

    if token == None {
        Err(StatusCode::UNAUTHORIZED)
    } else if let Some(token) = token {
        let token_sercret = env::var("token_sercret").unwrap_or(String::from("secret"));
        let token = token.to_str().ok().and_then(|h| h.strip_prefix("Bearer "));

        if token == None {
            Err(StatusCode::UNAUTHORIZED)
        } else if let Some(token) = token {
            print!("{}", token);
            let res = decode::<Claims>(
                token,
                &DecodingKey::from_secret(token_sercret.as_ref()),
                &Validation::default(),
            );
            if let Err(err) = res {
                print!("{}", err);
                Err(StatusCode::UNAUTHORIZED)
            } else if let Ok(td) = res {
                Ok(User {
                    id: td.claims.user_id,
                    name: String::from("tmp"),
                    password: String::from("tmp"),
                })
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    } else {
        todo!()
    }
}

#[post("/auth")]
pub async fn auth(pool: web::Data<Pool<Postgres>>, body: Json<UserDto>) -> impl Responder {
    let user_repo = user::UserRepo::new(&pool);
    let auth_service = AuthService::new(user_repo);

    match auth_service.auth(&body).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(err) => HttpResponse::Unauthorized().json(AuthMsg { msg: err }),
    }
}

trait IAuthService {
    async fn auth(&self, user: &UserDto) -> Result<AuthResponse, String>;
}

struct AuthService<UserRepo: user::IUserRepo> {
    user_repo: UserRepo,
}

impl<UserRepo: user::IUserRepo> AuthService<UserRepo> {
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

impl<R: user::IUserRepo> IAuthService for AuthService<R> {
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
    use actix_web::http::header;
    use api::user;

    use super::*;
    use async_trait::async_trait;
    use bcrypt::{DEFAULT_COST, hash};

    struct MockUserRepo<'a> {
        mock_exists: bool,
        fetch_user: Option<&'a user::UserDto>,
    }

    #[async_trait]
    impl user::IUserRepo for MockUserRepo<'_> {
        async fn exists(&self, _: &str) -> bool {
            self.mock_exists
        }

        async fn password_hash(&self, _: &str) -> String {
            String::from("pass")
        }

        async fn register(&self, _: &user::UserDto) {
            todo!()
        }

        async fn fetch_by_name(&self, _: &str) -> user::User {
            let user = self.fetch_user.unwrap();
            user::User {
                id: 123,
                name: user.name.clone(),
                password: user.password.clone(),
            }
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
    #[tokio::test]
    async fn auth_middeware_no_token() {
        let req = actix_web::test::TestRequest::default().to_http_request();
        let res = auth_user(req).await;

        assert!(res.is_err());

        assert_eq!(res.err().unwrap(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn auth_middeware_not_valid_jwt_token() {
        let req = actix_web::test::TestRequest::default()
            .insert_header((header::AUTHORIZATION, "invalid token"))
            .to_http_request();
        let res = auth_user(req).await;

        assert!(res.is_err());

        assert_eq!(res.err().unwrap(), StatusCode::UNAUTHORIZED);
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct WrongClaim {}

    #[tokio::test]
    async fn auth_middeware_not_rigth_clames() {
        let claim = WrongClaim {};

        let token_sercret = env::var("token_sercret").unwrap_or(String::from("secret"));

        let token = encode(
            &Header::default(),
            &claim,
            &EncodingKey::from_secret(token_sercret.as_ref()),
        )
        .unwrap();
        let req = actix_web::test::TestRequest::default()
            .insert_header((header::AUTHORIZATION, token))
            .to_http_request();
        let res = auth_user(req).await;

        assert!(res.is_err());

        assert_eq!(res.err().unwrap(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn auth_middeware_rigth_clames() {
        let claim = Claims { user_id: 123 };

        let token_sercret = env::var("token_sercret").unwrap_or(String::from("secret"));

        let token = encode(
            &Header::default(),
            &claim,
            &EncodingKey::from_secret(token_sercret.as_ref()),
        )
        .unwrap();

        let token = format!("Bearer {}", token);

        let req = actix_web::test::TestRequest::default()
            .insert_header((header::AUTHORIZATION, token))
            .to_http_request();
        let res = auth_user(req).await;

        assert!(res.is_ok());

        if let Ok(res) = res {
            assert_eq!(res.id, 123)
        }
    }
}
