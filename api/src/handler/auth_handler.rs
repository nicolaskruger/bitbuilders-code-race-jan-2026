use actix_web::{
    HttpRequest, HttpResponse, Responder, get, post,
    web::{self, Json},
};
use sqlx::{Pool, Postgres};

use crate::{
    contract::repo::auth_repo_trait::IAuthService,
    dto::user_dto::UserDto,
    entity::{
        auth_entity::{AuthMe, AuthMsg, UserAuth},
        user_entity::UserFetched,
    },
    repo::user_repo::UserRepo,
    service::auth_service::AuthService,
};

fn new_auth_service<'a>(pool: &'a web::Data<Pool<Postgres>>) -> AuthService<UserRepo<'a>> {
    let user_repo = UserRepo::new(pool);
    AuthService::new(user_repo)
}

#[post("/auth")]
pub async fn auth(pool: web::Data<Pool<Postgres>>, body: Json<UserDto>) -> impl Responder {
    let auth_service = new_auth_service(&pool);

    let user_auth = UserAuth {
        name: body.name.clone(),
        password: body.password.clone(),
    };

    match auth_service.auth(&user_auth).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(err) => HttpResponse::Unauthorized().json(AuthMsg { msg: err }),
    }
}

fn user_fetched_to_auth_me(fetched: &UserFetched) -> AuthMe {
    AuthMe {
        id: fetched.id,
        name: fetched.name.clone(),
    }
}

#[get("/auth/me")]
pub async fn me(pool: web::Data<Pool<Postgres>>, req: HttpRequest) -> impl Responder {
    match new_auth_service(&pool).user(req).await {
        Ok(fetched) => HttpResponse::Ok().json(user_fetched_to_auth_me(&fetched)),
        Err(_) => HttpResponse::Unauthorized().finish(),
    }
}
