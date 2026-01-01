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

trait IUserRepo {
    async fn exists(&self, name: &String) -> bool;
}

trait IUserService {
    async fn register(&self, dto: &UserDto) -> Result<(), String>;
}

struct UserService<R: IUserRepo> {
    repo: R,
}

impl<R: IUserRepo> UserService<R> {
    fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R: IUserRepo> IUserService for UserService<R> {
    async fn register(&self, dto: &UserDto) -> Result<(), String> {
        if self.repo.exists(&dto.name).await {
            Err(String::from("Not created because it already exists."))
        } else {
            Ok(())
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    struct MockUserRepo {
        mock_exists: bool,
    }

    impl IUserRepo for MockUserRepo {
        async fn exists(&self, name: &String) -> bool {
            self.mock_exists
        }
    }

    #[tokio::test]
    async fn error_on_create_existing_user() {
        let mock_repo = MockUserRepo { mock_exists: true };

        let service = UserService { repo: mock_repo };

        let dto = UserDto {
            name: String::from("nk"),
            password: String::from("123"),
        };

        let result = service.register(&dto).await;

        assert!(result.is_err(), "Not created because it already exists.");
    }
}
