use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ItemEntity {
    id: i32,
    name: String,
    price: f64,
}
