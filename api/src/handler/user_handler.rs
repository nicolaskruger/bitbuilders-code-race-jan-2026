use actix_web::{HttpResponse, Responder, post, web};
use sqlx::{Pool, Postgres};

use crate::{
    contract::service::user_service_trait::IUserService,
    dto::user_dto::{UserDto, UserRespose},
    entity::user_entity::UserRegister,
    repo::user_repo::UserRepo,
    service::user_service::UserService,
};

#[post("/user")]
pub async fn create(pool: web::Data<Pool<Postgres>>, body: web::Json<UserDto>) -> impl Responder {
    let repo = UserRepo::new(&pool);
    let service = UserService::new(repo);

    let data_in = UserRegister {
        password: body.password.clone(),
        name: body.name.clone(),
    };

    let res = service.register(&data_in).await;

    match res {
        Ok(_) => HttpResponse::Created().json(UserRespose {
            msg: String::from("user created"),
        }),
        Err(msg) => HttpResponse::BadRequest().json(UserRespose { msg }),
    }
}
