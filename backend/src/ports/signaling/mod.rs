use std::future::Future;

use crate::domain::value_objects::UserId;

pub enum SignalingMessage {
    Offer {
        sender_id: UserId,
        sdp: String,
        conversation_type: String,
    },
    Answer {
        sender_id: UserId,
        sdp: String,
    },
    IceCandidate {
        sender_id: UserId,
        candidate: String,
        sdp_mid: String,
        sdp_mline_index: u16,
    },
    Accept {
        sender_id: UserId,
    },
    Reject {
        sender_id: UserId,
        reason: String,
    },
}

pub trait SignalingPort: Send + Sync {
    fn send_to_user(
        &self,
        target_id: &UserId,
        message: &SignalingMessage,
    ) -> impl Future<Output = Result<(), ()>> + Send;

    fn broadcast_to_contacts(
        &self,
        user_id: &UserId,
        event: &str,
        payload: &str,
    ) -> impl Future<Output = Result<(), ()>> + Send;

    fn send_to_group_members(
        &self,
        member_ids: &[UserId],
        message: &SignalingMessage,
    ) -> impl Future<Output = Result<(), ()>> + Send;
}
