use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum DomainError {
    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("user not found")]
    UserNotFound,

    #[error("session not found")]
    SessionNotFound,

    #[error("session expired")]
    SessionExpired,

    #[error("session revoked")]
    SessionRevoked,

    #[error("invalid token")]
    InvalidToken,

    #[error("invalid username: {0}")]
    InvalidUsername(String),

    #[error("invalid email: {0}")]
    InvalidEmail(String),

    #[error("weak password")]
    WeakPassword,

    #[error("username already taken")]
    DuplicateUsername,

    #[error("email already registered")]
    DuplicateEmail,

    #[error("conversation not found")]
    ConversationNotFound,

    #[error("not a participant in this conversation")]
    NotParticipant,

    #[error("message too large")]
    MessageTooLarge,

    #[error("invalid signature")]
    InvalidSignature,

    #[error("group not found")]
    GroupNotFound,

    #[error("not a member of this group")]
    NotMember,

    #[error("not the group owner")]
    NotOwner,

    #[error("member already exists in group")]
    MemberAlreadyExists,

    #[error("group is full")]
    GroupFull,

    #[error("insufficient permissions")]
    InsufficientPermissions,

    #[error("user is offline")]
    UserOffline,

    #[error("account is locked")]
    AccountLocked,

    #[error("unauthorized")]
    Unauthorized,

    #[error("not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}
