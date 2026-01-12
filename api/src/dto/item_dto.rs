use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ItemDto {
    name: String,
    price: f64,
}
