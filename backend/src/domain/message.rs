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
    sender_sequence_number: u64,
    #[expect(dead_code)]
    ciphertext: Ciphertext,
    #[expect(dead_code)]
    iv: Iv,
    #[expect(dead_code)]
    salt: Salt,
    #[expect(dead_code)]
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
        sender_sequence_number: u64,
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
            sender_sequence_number,
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

    pub fn sender_sequence_number(&self) -> u64 {
        self.sender_sequence_number
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::ConversationId;

    fn make_message(seq: u64) -> Message {
        Message::new(
            ConversationId::new(),
            UserId::new(),
            seq,
            Ciphertext::new(vec![0u8; 16]),
            Iv::new([0u8; 24]),
            Salt::new([0u8; 32]),
            Signature::new([0u8; 64]),
            None,
        )
    }

    #[test]
    fn test_message_sequence_number() {
        let msg = make_message(42);
        assert_eq!(msg.sender_sequence_number(), 42);
    }

    #[test]
    fn test_message_starts_sent() {
        let msg = make_message(1);
        assert_eq!(msg.status(), MessageStatus::Sent);
    }

    #[test]
    fn test_mark_delivered() {
        let mut msg = make_message(1);
        msg.mark_delivered();
        assert_eq!(msg.status(), MessageStatus::Delivered);
    }

    #[test]
    fn test_mark_read() {
        let mut msg = make_message(1);
        msg.mark_delivered();
        msg.mark_read();
        assert_eq!(msg.status(), MessageStatus::Read);
    }

    #[test]
    fn test_sequence_number_strictly_increasing() {
        let msg1 = make_message(1);
        let msg2 = make_message(2);
        let msg3 = make_message(3);
        assert!(msg2.sender_sequence_number() > msg1.sender_sequence_number());
        assert!(msg3.sender_sequence_number() > msg2.sender_sequence_number());
    }
}
