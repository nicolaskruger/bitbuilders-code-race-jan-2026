use async_trait::async_trait;
use sqlx::{Pool, Postgres};

use crate::{
    contract::repo::{item_repo_trait::IItemRepo, user_repo_trait::IUserRepo},
    entity::item_entity::ItemEntity,
};

pub struct ItemRepo<'a> {
    pool: &'a Pool<Postgres>,
}

impl<'a> ItemRepo<'a> {
    pub fn new(pool: &'a Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IItemRepo for ItemRepo<'_> {
    async fn create(&self, name: &ItemEntity) -> bool {
        todo!()
    }
}
