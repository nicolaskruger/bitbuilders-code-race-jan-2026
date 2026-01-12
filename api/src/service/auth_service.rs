use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    contract::repo::user_repo_trait::IUserRepo,
    contract::service::auth_service_trait::IAuthService,
    entity::{
        auth_entity::{AuthResponse, Claims, UserAuth},
        user_entity::UserFetched,
    },
};
use actix_web::{
    HttpRequest,
    http::{StatusCode, header::AUTHORIZATION},
};
use async_trait::async_trait;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};

pub struct AuthService<UserRepo: IUserRepo> {
    user_repo: UserRepo,
}
fn one_year_exp() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    now + 60 * 60 * 24 * 365
}
impl<UserRepo: IUserRepo> AuthService<UserRepo> {
    pub fn new(user_repo: UserRepo) -> Self {
        Self { user_repo }
    }

    async fn token_build(&self, user: &UserAuth) -> AuthResponse {
        let fetch_user = self.user_repo.fetch_by_name(&user.name).await;

        let claim = Claims {
            user_id: fetch_user.id,
            exp: one_year_exp(),
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

    async fn match_password(&self, user: &UserAuth) -> bool {
        let fetch_user = self.user_repo.fetch_by_name(&user.name).await;
        bcrypt::verify(&user.password, &fetch_user.password).unwrap_or(false)
    }
}

#[async_trait(?Send)]
impl<R: IUserRepo + Sync> IAuthService for AuthService<R> {
    async fn auth(&self, user: &UserAuth) -> Result<AuthResponse, String> {
        if !self.user_repo.exists(&user.name).await {
            Err(String::from("User de not exists."))
        } else if !self.match_password(user).await {
            Err(String::from("Miss match password."))
        } else {
            Ok(self.token_build(user).await)
        }
    }

    async fn user(&self, req: HttpRequest) -> Result<UserFetched, StatusCode> {
        let token_sercret = env::var("token_sercret").unwrap_or(String::from("secret"));
        let claim: Option<_> = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .map(|token| {
                decode::<Claims>(
                    token,
                    &DecodingKey::from_secret(token_sercret.as_ref()),
                    &Validation::default(),
                )
            });

        if let Some(Ok(cl)) = claim {
            Ok(self.user_repo.fetch_by_id(cl.claims.user_id).await)
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

#[cfg(test)]
mod tests {
    use actix_web::http::header;
    use jsonwebtoken::{EncodingKey, Header, encode};
    use serde::{Deserialize, Serialize};

    use crate::entity::user_entity::UserRegister;

    use super::*;
    use async_trait::async_trait;
    use bcrypt::{DEFAULT_COST, hash};

    struct MockUserRepo<'a> {
        mock_exists: bool,
        fetch_user: Option<&'a UserFetched>,
    }

    #[async_trait]
    impl IUserRepo for MockUserRepo<'_> {
        async fn exists(&self, _: &str) -> bool {
            self.mock_exists
        }

        async fn password_hash(&self, _: &str) -> String {
            String::from("pass")
        }

        async fn register(&self, _: &UserRegister) {
            todo!()
        }

        async fn fetch_by_name(&self, _: &str) -> UserFetched {
            let user = self.fetch_user.unwrap();
            UserFetched {
                id: 123,
                name: user.name.clone(),
                password: user.password.clone(),
            }
        }

        async fn fetch_by_id(&self, _: i32) -> UserFetched {
            UserFetched {
                id: 1,
                name: String::from("name"),
                password: String::from("password"),
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

        let dto = UserAuth {
            name: String::from("nk"),
            password: String::from("123"),
        };

        let res: Result<_, _> = service.auth(&dto).await;

        assert!(res.is_err(), "Not created because it already exists.");
    }

    fn template_service() -> AuthService<MockUserRepo<'static>> {
        let mock_repo = MockUserRepo {
            mock_exists: true,
            fetch_user: None,
        };
        AuthService::new(mock_repo)
    }

    #[tokio::test]
    async fn wrong_password() {
        let fetch_user = UserFetched {
            id: 1,
            name: String::from("nk"),
            password: String::from("456"),
        };

        let mock_repo = MockUserRepo {
            mock_exists: true,
            fetch_user: Some(&fetch_user),
        };

        let service = AuthService::new(mock_repo);

        let dto = UserAuth {
            name: String::from("nk"),
            password: String::from("123"),
        };

        let res: Result<_, _> = service.auth(&dto).await;

        assert!(res.is_err(), "Miss match password");
    }

    #[tokio::test]
    async fn auth_api_fetch_token() {
        let hash_password = hash("123", DEFAULT_COST).unwrap();

        let fetch_user = UserFetched {
            id: 1,
            name: String::from("nk"),
            password: hash_password,
        };

        let mock_repo = MockUserRepo {
            mock_exists: true,
            fetch_user: Some(&fetch_user),
        };

        let service = AuthService::new(mock_repo);

        let dto = UserAuth {
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
        let service = template_service();
        let res = service.user(req).await;

        assert!(res.is_err());

        assert_eq!(res.err().unwrap(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn auth_middeware_not_valid_jwt_token() {
        let req = actix_web::test::TestRequest::default()
            .insert_header((header::AUTHORIZATION, "invalid token"))
            .to_http_request();

        let service = template_service();
        let res = service.user(req).await;

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

        let service = template_service();
        let res = service.user(req).await;

        assert!(res.is_err());

        assert_eq!(res.err().unwrap(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn auth_middeware_rigth_clames() {
        let claim = Claims {
            user_id: 123,
            exp: one_year_exp(),
        };

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

        let service = template_service();
        let res = service.user(req).await;

        assert!(res.is_ok());

        if let Ok(res) = res {
            assert_eq!(res.id, 1)
        }
    }
}
