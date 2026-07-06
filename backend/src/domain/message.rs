use super::value_objects::{
    Ciphertext, ConversationId, Iv, MessageId, Salt, Signature, Timestamp, UserId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageStatus {
    Sent,
    Delivered,
    Read,
    Failed,
}

#[derive(Debug, Clone)]
pub struct Message {
    id: MessageId,
    conversation_id: ConversationId,
    sender_id: UserId,
    ciphertext: Ciphertext,
    iv: Iv,
    salt: Salt,
    signature: Signature,
    status: MessageStatus,
    timestamp: Timestamp,
    reply_to: Option<MessageId>,
}

impl Message {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        conversation_id: ConversationId,
        sender_id: UserId,
        ciphertext: Ciphertext,
        iv: Iv,
        salt: Salt,
        signature: Signature,
        reply_to: Option<MessageId>,
    ) -> Self {
        Self {
            id: MessageId::new(),
            conversation_id,
            sender_id,
            ciphertext,
            iv,
            salt,
            signature,
            status: MessageStatus::Sent,
            timestamp: Timestamp::now(),
            reply_to,
        }
    }

    pub fn id(&self) -> &MessageId {
        &self.id
    }

    pub fn conversation_id(&self) -> &ConversationId {
        &self.conversation_id
    }

    pub fn sender_id(&self) -> &UserId {
        &self.sender_id
    }

    pub fn status(&self) -> MessageStatus {
        self.status
    }

    pub fn mark_delivered(&mut self) {
        self.status = MessageStatus::Delivered;
    }

    pub fn mark_read(&mut self) {
        self.status = MessageStatus::Read;
    }

    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }

    pub fn reply_to(&self) -> Option<&MessageId> {
        self.reply_to.as_ref()
    }
}
