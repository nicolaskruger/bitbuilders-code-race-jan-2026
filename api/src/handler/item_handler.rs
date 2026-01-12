use actix_web::{
    HttpRequest, HttpResponse, Responder, post,
    web::{self, Json},
};
use sqlx::{Pool, Postgres};

use crate::dto::item_dto::ItemDto;

#[post("/auth")]
pub async fn create(
    pool: web::Data<Pool<Postgres>>,
    body: Json<ItemDto>,
    req: HttpRequest,
) -> impl Responder {
    HttpResponse::Unauthorized().finish()
    // let auth_service = new_auth_service(&pool);
    //
    // let user_auth = UserAuth {
    //     name: body.name.clone(),
    //     password: body.password.clone(),
    // };
    //
    // match auth_service.auth(&user_auth).await {
    //     Ok(resp) => HttpResponse::Ok().json(resp),
    //     Err(err) => HttpResponse::Unauthorized().json(AuthMsg { msg: err }),
    // }
}
