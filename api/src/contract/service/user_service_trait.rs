use crate::entity::user_entity::UserRegister;

use async_trait::async_trait;

#[async_trait]
pub trait IUserService {
    async fn register(&self, dto: &UserRegister) -> Result<(), String>;
}
