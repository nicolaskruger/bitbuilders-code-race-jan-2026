use crate::entity::auth_entity::*;
use crate::entity::user_entity::UserFetched;
use actix_web::{HttpRequest, http::StatusCode};
use async_trait::async_trait;

#[async_trait]
pub trait IAuthService {
    async fn auth(&self, user: &UserDto) -> Result<AuthResponse, String>;
    async fn user(&self, req: HttpRequest) -> Result<UserFetched, StatusCode>;
}
