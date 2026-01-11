use actix_web::{
    HttpResponse, Responder, post,
    web::{self, Json},
};
use sqlx::{Pool, Postgres};

use crate::{
    contract::repo::auth_repo_trait::IAuthService,
    dto::user_dto::UserDto,
    entity::auth_entity::{AuthMsg, UserAuth},
    repo::user_repo::UserRepo,
    service::auth_service::AuthService,
};

#[post("/auth")]
pub async fn auth(pool: web::Data<Pool<Postgres>>, body: Json<UserDto>) -> impl Responder {
    let user_repo = UserRepo::new(&pool);
    let auth_service = AuthService::new(user_repo);

    let user_auth = UserAuth {
        name: body.name.clone(),
        password: body.password.clone(),
    };

    match auth_service.auth(&user_auth).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(err) => HttpResponse::Unauthorized().json(AuthMsg { msg: err }),
    }
}
