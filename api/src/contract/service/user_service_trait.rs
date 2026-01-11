use crate::entity::user_entity::UserRegister;

pub trait IUserService {
    async fn register(&self, dto: &UserRegister) -> Result<(), String>;
}
