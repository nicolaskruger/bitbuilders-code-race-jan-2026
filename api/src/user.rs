use actix_web::{HttpResponse, Responder, post, web};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct UserDto {
    pub name: String,
    pub password: String,
}

#[post("/user")]
pub async fn create(body: web::Json<UserDto>) -> impl Responder {
    HttpResponse::Created().body(body.name.clone())
}
