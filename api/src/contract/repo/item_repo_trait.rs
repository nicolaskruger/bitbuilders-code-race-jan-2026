use async_trait::async_trait;

use crate::entity::item_entity::ItemCreate;

#[async_trait]
pub trait IItemRepo {
    async fn create(&self, item: &ItemCreate);
}
