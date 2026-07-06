use std::future::Future;

use crate::domain::value_objects::UserId;

pub struct OnlineUser {
    pub user_id: UserId,
    pub username: String,
}

pub trait PresencePort: Send + Sync {
    fn user_online(
        &self,
        user_id: &UserId,
        session_id: &crate::domain::value_objects::SessionId,
    ) -> impl Future<Output = ()> + Send;

    fn user_offline(&self, user_id: &UserId) -> impl Future<Output = ()> + Send;

    fn is_online(&self, user_id: &UserId) -> impl Future<Output = bool> + Send;

    fn get_online_users(&self) -> impl Future<Output = Vec<OnlineUser>> + Send;
}
