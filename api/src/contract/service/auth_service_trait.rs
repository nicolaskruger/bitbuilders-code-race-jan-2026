use actix_web::{HttpRequest, http::StatusCode};

use crate::entity::auth_entity::{AuthResponse, LoggedUser, UserAuth};

pub trait IAuthService {
    async fn auth(&self, user: &UserAuth) -> Result<AuthResponse, String>;
    async fn user(&self, req: HttpRequest) -> Result<LoggedUser, StatusCode>;
}
