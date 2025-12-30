use actix_web::{HttpResponse, Responder, post, web};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct User {
    name: String,
    password: String,
}

#[post("/user")]
pub async fn create(body: web::Json<User>) -> impl Responder {
    HttpResponse::Ok().body(body.name.clone())
}
