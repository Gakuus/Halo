use super::value_objects::{ConversationId, Timestamp, UserId};

#[derive(Debug, Clone)]
pub struct Conversation {
    id: ConversationId,
    participant_a: UserId,
    participant_b: UserId,
    created_at: Timestamp,
    last_message_at: Option<Timestamp>,
    is_active: bool,
}

impl Conversation {
    pub fn new(participant_a: UserId, participant_b: UserId) -> Self {
        let (a, b) = order_participants(participant_a, participant_b);
        Self {
            id: ConversationId::new(),
            participant_a: a,
            participant_b: b,
            created_at: Timestamp::now(),
            last_message_at: None,
            is_active: true,
        }
    }

    pub fn id(&self) -> &ConversationId {
        &self.id
    }

    pub fn participant_a(&self) -> &UserId {
        &self.participant_a
    }

    pub fn participant_b(&self) -> &UserId {
        &self.participant_b
    }

    pub fn participants(&self) -> (&UserId, &UserId) {
        (&self.participant_a, &self.participant_b)
    }

    pub fn includes_user(&self, user_id: &UserId) -> bool {
        self.participant_a == *user_id || self.participant_b == *user_id
    }

    pub fn other_participant(&self, user_id: &UserId) -> Option<&UserId> {
        if self.participant_a == *user_id {
            Some(&self.participant_b)
        } else if self.participant_b == *user_id {
            Some(&self.participant_a)
        } else {
            None
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn created_at(&self) -> &Timestamp {
        &self.created_at
    }

    pub fn last_message_at(&self) -> Option<&Timestamp> {
        self.last_message_at.as_ref()
    }
}

fn order_participants(a: UserId, b: UserId) -> (UserId, UserId) {
    if a.as_uuid() < b.as_uuid() {
        (a, b)
    } else {
        (b, a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_orders_participants() {
        let a = UserId::new();
        let b = UserId::new();
        let conv = Conversation::new(a, b);
        let (p1, p2) = conv.participants();

        assert!(p1.as_uuid() < p2.as_uuid());
    }

    #[test]
    fn test_includes_user() {
        let a = UserId::new();
        let b = UserId::new();
        let conv = Conversation::new(a, b);

        assert!(conv.includes_user(&a));
        assert!(conv.includes_user(&b));

        let c = UserId::new();
        assert!(!conv.includes_user(&c));
    }

    #[test]
    fn test_other_participant() {
        let a = UserId::new();
        let b = UserId::new();
        let conv = Conversation::new(a, b);

        assert_eq!(conv.other_participant(&a), Some(&b));
        assert_eq!(conv.other_participant(&b), Some(&a));
    }
}
