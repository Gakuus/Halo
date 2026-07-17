use std::future::Future;

use crate::domain::{
    conversation::Conversation,
    error::DomainError,
    group::{Group, GroupMember},
    pre_key::{OneTimePreKey, PreKeyBundle, SignedPreKey},
    session::Session,
    user::User,
    value_objects::{
        ConversationId, Email, GroupId, KeyId, SessionId, UserId, Username, X25519PublicKey,
    },
};

pub trait UserRepository: Send + Sync {
    fn find_by_id(&self, id: &UserId) -> impl Future<Output = Result<User, DomainError>> + Send;
    fn find_by_username(
        &self,
        username: &Username,
    ) -> impl Future<Output = Result<User, DomainError>> + Send;
    fn find_by_email(
        &self,
        email: &Email,
    ) -> impl Future<Output = Result<User, DomainError>> + Send;
    fn save(&self, user: &User) -> impl Future<Output = Result<(), DomainError>> + Send;
    fn search(&self, query: &str) -> impl Future<Output = Result<Vec<User>, DomainError>> + Send;
    fn username_exists(
        &self,
        username: &Username,
    ) -> impl Future<Output = Result<bool, DomainError>> + Send;
    fn email_exists(&self, email: &Email)
    -> impl Future<Output = Result<bool, DomainError>> + Send;
}

pub trait SessionRepository: Send + Sync {
    fn find_by_id(
        &self,
        id: &SessionId,
    ) -> impl Future<Output = Result<Session, DomainError>> + Send;
    fn find_by_jwt_id(
        &self,
        jwt_id: &uuid::Uuid,
    ) -> impl Future<Output = Result<Session, DomainError>> + Send;
    fn find_active_by_user(
        &self,
        user_id: &UserId,
    ) -> impl Future<Output = Result<Option<Session>, DomainError>> + Send;
    fn save(&self, session: &Session) -> impl Future<Output = Result<(), DomainError>> + Send;
    fn update_status(
        &self,
        id: &SessionId,
        status: &crate::domain::session::SessionStatus,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;
}

pub trait ConversationRepository: Send + Sync {
    fn find_by_id(
        &self,
        id: &ConversationId,
    ) -> impl Future<Output = Result<Conversation, DomainError>> + Send;
    fn find_by_participants(
        &self,
        a: &UserId,
        b: &UserId,
    ) -> impl Future<Output = Result<Option<Conversation>, DomainError>> + Send;
    fn save(
        &self,
        conversation: &Conversation,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;
    fn find_by_user(
        &self,
        user_id: &UserId,
    ) -> impl Future<Output = Result<Vec<Conversation>, DomainError>> + Send;
}

pub trait GroupRepository: Send + Sync {
    fn find_by_id(&self, id: &GroupId) -> impl Future<Output = Result<Group, DomainError>> + Send;
    fn find_by_member(
        &self,
        user_id: &UserId,
    ) -> impl Future<Output = Result<Vec<Group>, DomainError>> + Send;
    fn save(&self, group: &Group) -> impl Future<Output = Result<(), DomainError>> + Send;
    fn add_member(
        &self,
        group_id: &GroupId,
        member: &GroupMember,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;
    fn remove_member(
        &self,
        group_id: &GroupId,
        user_id: &UserId,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;
    fn update(&self, group: &Group) -> impl Future<Output = Result<(), DomainError>> + Send;
}

pub trait PreKeyRepository: Send + Sync {
    fn find_bundle(
        &self,
        user_id: &UserId,
    ) -> impl Future<Output = Result<PreKeyBundle, DomainError>> + Send;

    fn save_signed_pre_key(
        &self,
        pre_key: &SignedPreKey,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    fn save_one_time_pre_keys(
        &self,
        keys: &[OneTimePreKey],
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    fn consume_one_time_pre_key(
        &self,
        user_id: &UserId,
        key_id: KeyId,
    ) -> impl Future<Output = Result<X25519PublicKey, DomainError>> + Send;

    fn count_one_time_pre_keys(
        &self,
        user_id: &UserId,
    ) -> impl Future<Output = Result<u32, DomainError>> + Send;
}
