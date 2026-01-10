use super::user_entity::{UserFetched, UserRegister};

use async_trait::async_trait;

#[async_trait]
pub trait IUserRepo {
    async fn exists(&self, name: &str) -> bool;
    async fn password_hash(&self, password: &str) -> String;
    async fn register(&self, dto: &UserRegister);
    async fn fetch_by_name(&self, name: &str) -> UserFetched;
    async fn fetch_by_id(&self, id: i32) -> UserFetched;
}
