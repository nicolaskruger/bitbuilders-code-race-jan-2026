use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ItemCreate {
    pub name: String,
    pub price: f64,
    pub user_id: i32,
}
