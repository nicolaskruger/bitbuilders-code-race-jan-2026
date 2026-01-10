use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserRespose {
    msg: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
pub struct UserFetched {
    pub id: i32,
    pub name: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserRegister {
    pub name: String,
    pub password: String,
}
