use async_trait::async_trait;

use crate::entity::item_entity::ItemEntity;

#[async_trait]
pub trait IItemRepo {
    async fn create(&self, name: &ItemEntity) -> bool;
}
