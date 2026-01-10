use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::contract::repo::auth_repo_trait::IAuthService;
use crate::contract::repo::user_repo_trait::IUserRepo;
use crate::entity::auth_entity::*;
use crate::entity::user_entity::UserFetched;
use actix_web::{
    HttpRequest,
    http::{StatusCode, header},
};
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

    async fn token_build(&self, user: &UserDto) -> AuthResponse {
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
    async fn user(&self, req: HttpRequest) -> Result<UserFetched, StatusCode> {
        let token_sercret = env::var("token_sercret").unwrap_or(String::from("secret"));
        let claim: Option<_> = req
            .headers()
            .get(header::AUTHORIZATION)
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
