use super::value_objects::{KeyId, Signature, Timestamp, UserId, X25519PublicKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreKeyType {
    Signed,
    OneTime,
}

#[derive(Debug, Clone)]
pub struct SignedPreKey {
    id: KeyId,
    user_id: UserId,
    public_key: X25519PublicKey,
    signature: Signature,
    created_at: Timestamp,
}

impl SignedPreKey {
    pub fn new(
        id: KeyId,
        user_id: UserId,
        public_key: X25519PublicKey,
        signature: Signature,
    ) -> Self {
        Self {
            id,
            user_id,
            public_key,
            signature,
            created_at: Timestamp::now(),
        }
    }

    pub fn id(&self) -> KeyId {
        self.id
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn public_key(&self) -> &X25519PublicKey {
        &self.public_key
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    pub fn created_at(&self) -> &Timestamp {
        &self.created_at
    }

    pub fn from_db(
        id: KeyId,
        user_id: UserId,
        public_key: X25519PublicKey,
        signature: Signature,
        created_at: Timestamp,
    ) -> Self {
        Self {
            id,
            user_id,
            public_key,
            signature,
            created_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OneTimePreKey {
    id: KeyId,
    user_id: UserId,
    public_key: X25519PublicKey,
    created_at: Timestamp,
}

impl OneTimePreKey {
    pub fn new(id: KeyId, user_id: UserId, public_key: X25519PublicKey) -> Self {
        Self {
            id,
            user_id,
            public_key,
            created_at: Timestamp::now(),
        }
    }

    pub fn id(&self) -> KeyId {
        self.id
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn public_key(&self) -> &X25519PublicKey {
        &self.public_key
    }

    pub fn created_at(&self) -> &Timestamp {
        &self.created_at
    }

    pub fn from_db(
        id: KeyId,
        user_id: UserId,
        public_key: X25519PublicKey,
        created_at: Timestamp,
    ) -> Self {
        Self {
            id,
            user_id,
            public_key,
            created_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PreKeyBundle {
    pub identity_key: Vec<u8>,
    pub signed_pre_key: SignedPreKey,
    pub one_time_pre_keys: Vec<OneTimePreKey>,
}
