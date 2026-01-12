use actix_web::{HttpRequest, http::StatusCode};
use async_trait::async_trait;

use crate::entity::{
    auth_entity::{AuthResponse, UserAuth},
    user_entity::UserFetched,
};

#[async_trait(?Send)]
pub trait IAuthService {
    async fn auth(&self, user: &UserAuth) -> Result<AuthResponse, String>;
    async fn user(&self, req: HttpRequest) -> Result<UserFetched, StatusCode>;
}
