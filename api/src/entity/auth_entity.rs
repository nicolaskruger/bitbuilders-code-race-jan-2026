use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i32,
    pub exp: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthResponse {
    pub token: String,
    pub refresh: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserDto {
    pub name: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthMsg {
    pub msg: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserAuth {
    pub name: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggedUser {
    pub id: i32,
    pub name: String,
    pub password: String,
}
