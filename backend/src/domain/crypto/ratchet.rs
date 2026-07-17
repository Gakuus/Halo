use chacha20poly1305::{
    ChaCha20Poly1305, Nonce,
    aead::{Aead, KeyInit},
};
use rand::RngCore;
use rand::rngs::OsRng;
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::ZeroizeOnDrop;

use super::{ChainKey, KEY_LENGTH, SharedSecret, kdf_ck, kdf_rk};

#[derive(Debug, Clone, ZeroizeOnDrop)]
pub struct MessageKey([u8; KEY_LENGTH]);

impl MessageKey {
    fn new(data: [u8; KEY_LENGTH]) -> Self {
        Self(data)
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    pub dh_public_key: Vec<u8>,
    pub previous_chain_length: u32,
}

pub struct CipherMessage {
    pub header: Header,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

#[derive(Clone, ZeroizeOnDrop)]
pub struct RatchetState {
    root_key: SharedSecret,
    send_chain_key: Option<ChainKeyState>,
    recv_chain_key: Option<ChainKeyState>,
    dh_send_keypair: Option<StaticSecret>,
    dh_recv_public_key: Option<PublicKey>,
    skipped_message_keys: Vec<SkippedMessageKey>,
}

impl std::fmt::Debug for RatchetState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RatchetState")
            .field("send_chain_key", &self.send_chain_key)
            .field("recv_chain_key", &self.recv_chain_key)
            .field("dh_send_keypair", &"[redacted]")
            .field("dh_recv_public_key", &self.dh_recv_public_key)
            .field("skipped_message_keys", &self.skipped_message_keys.len())
            .finish()
    }
}

#[derive(Debug, Clone, ZeroizeOnDrop)]
struct ChainKeyState {
    chain_key: ChainKey,
    message_index: u32,
}

#[derive(Debug, Clone, ZeroizeOnDrop)]
struct SkippedMessageKey {
    dh_public_key: Vec<u8>,
    message_index: u32,
    message_key: MessageKey,
}

impl RatchetState {
    /// Creates a new ratchet state from an X3DH shared secret.
    ///
    /// `is_initiator` — if true, derives an initial sending chain from the
    /// root key and generates a DH key pair for the first message header.
    /// The responder waits for the first message to set up chains.
    pub fn new(shared_secret: SharedSecret, is_initiator: bool) -> Self {
        let mut state = RatchetState {
            root_key: shared_secret,
            send_chain_key: None,
            recv_chain_key: None,
            dh_send_keypair: None,
            dh_recv_public_key: None,
            skipped_message_keys: Vec::new(),
        };

        if is_initiator {
            let dh_sk = StaticSecret::random_from_rng(&mut OsRng);
            state.dh_send_keypair = Some(dh_sk);
            let ck = ChainKey::new(
                super::hkdf(KEY_LENGTH, &[0u8; KEY_LENGTH], state.root_key.as_bytes())
                    .try_into()
                    .unwrap(),
            );
            state.send_chain_key = Some(ChainKeyState {
                chain_key: ck,
                message_index: 0,
            });
        }

        state
    }

    /// Encrypts a message using the sending chain.
    pub fn encrypt(
        &mut self,
        plaintext: &[u8],
        _associated_data: &[u8],
    ) -> Result<CipherMessage, &'static str> {
        if self.dh_send_keypair.is_none() {
            self.dh_send_keypair = Some(StaticSecret::random_from_rng(&mut OsRng));
        }

        if self.send_chain_key.is_none() {
            if let Some(dh_recv_pk) = self.dh_recv_public_key.as_ref() {
                let dh_sk = self.dh_send_keypair.as_ref().unwrap();
                let shared = dh_sk.diffie_hellman(dh_recv_pk);
                let (new_rk, new_ck) = kdf_rk(&self.root_key, shared.as_bytes());
                self.root_key = new_rk;
                self.send_chain_key = Some(ChainKeyState {
                    chain_key: new_ck,
                    message_index: 0,
                });
            } else {
                let ck_bytes: [u8; KEY_LENGTH] =
                    super::hkdf(KEY_LENGTH, &[0u8; KEY_LENGTH], self.root_key.as_bytes())
                        .try_into()
                        .unwrap();
                self.send_chain_key = Some(ChainKeyState {
                    chain_key: ChainKey::new(ck_bytes),
                    message_index: 0,
                });
            }
        }

        let dh_sk = self.dh_send_keypair.as_ref().unwrap();
        let dh_pk = PublicKey::from(dh_sk);

        let chain = self.send_chain_key.as_mut().ok_or("no send chain")?;
        let (new_ck, mk) = kdf_ck(&chain.chain_key);
        chain.chain_key = new_ck;
        chain.message_index += 1;

        let header = Header {
            dh_public_key: dh_pk.as_bytes().to_vec(),
            previous_chain_length: self
                .recv_chain_key
                .as_ref()
                .map(|c| c.message_index)
                .unwrap_or(0),
        };

        let msg_key = MessageKey::new(mk);

        let cipher = ChaCha20Poly1305::new_from_slice(msg_key.as_bytes())
            .map_err(|_| "invalid key length")?;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| "encryption failed")?;

        Ok(CipherMessage {
            header,
            nonce: nonce_bytes.to_vec(),
            ciphertext,
        })
    }

    /// Decrypts a message, handling DH ratchet if the header contains a new DH key.
    pub fn decrypt(
        &mut self,
        message: &CipherMessage,
        _associated_data: &[u8],
    ) -> Result<Vec<u8>, &'static str> {
        let header = &message.header;

        if let Some(skipped) = self.try_skipped_message_key(header) {
            let cipher = ChaCha20Poly1305::new_from_slice(skipped.as_bytes())
                .map_err(|_| "invalid key length")?;
            let nonce = Nonce::from_slice(&message.nonce);
            return cipher
                .decrypt(nonce, message.ciphertext.as_ref())
                .map_err(|_| "decryption failed");
        }

        let is_new_dh_key = !header.dh_public_key.is_empty()
            && self.dh_recv_public_key.as_ref().map_or(true, |current| {
                current.as_bytes() != header.dh_public_key.as_slice()
            });

        if is_new_dh_key {
            let sender_dh_pk = {
                let mut pk_arr = [0u8; 32];
                pk_arr.copy_from_slice(&header.dh_public_key);
                PublicKey::from(pk_arr)
            };

            if self.recv_chain_key.is_none()
                && self.send_chain_key.is_none()
                && self.dh_recv_public_key.is_none()
            {
                let ck_bytes: [u8; KEY_LENGTH] =
                    super::hkdf(KEY_LENGTH, &[0u8; KEY_LENGTH], self.root_key.as_bytes())
                        .try_into()
                        .unwrap();
                self.recv_chain_key = Some(ChainKeyState {
                    chain_key: ChainKey::new(ck_bytes),
                    message_index: 0,
                });
            } else {
                let dh_sk = self
                    .dh_send_keypair
                    .get_or_insert_with(|| StaticSecret::random_from_rng(&mut OsRng));
                let shared = dh_sk.diffie_hellman(&sender_dh_pk);
                let (new_rk, new_ck) = kdf_rk(&self.root_key, shared.as_bytes());
                self.root_key = new_rk;

                if self.recv_chain_key.is_some() {
                    self.send_chain_key = Some(ChainKeyState {
                        chain_key: new_ck,
                        message_index: 0,
                    });
                } else {
                    self.recv_chain_key = Some(ChainKeyState {
                        chain_key: new_ck,
                        message_index: 0,
                    });
                }
            }

            self.dh_recv_public_key = Some(sender_dh_pk);
        }

        let chain = self.recv_chain_key.as_mut().ok_or("no receive chain")?;
        let (new_ck, mk) = kdf_ck(&chain.chain_key);
        chain.chain_key = new_ck;
        chain.message_index += 1;

        let msg_key = MessageKey::new(mk);

        let cipher = ChaCha20Poly1305::new_from_slice(msg_key.as_bytes())
            .map_err(|_| "invalid key length")?;
        let nonce = Nonce::from_slice(&message.nonce);

        cipher
            .decrypt(nonce, message.ciphertext.as_ref())
            .map_err(|_| "decryption failed")
    }

    fn try_skipped_message_key(&self, header: &Header) -> Option<MessageKey> {
        self.skipped_message_keys.iter().find_map(|sk| {
            if sk.dh_public_key == header.dh_public_key
                && sk.message_index == header.previous_chain_length
            {
                Some(sk.message_key.clone())
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    use crate::domain::crypto::x3dh::{x3dh_initiate, x3dh_receive};

    fn perform_x3dh() -> (SharedSecret, SharedSecret) {
        let mut rng = OsRng;

        let alice_ik = StaticSecret::random_from_rng(&mut rng);
        let alice_ek = StaticSecret::random_from_rng(&mut rng);

        let bob_ik = StaticSecret::random_from_rng(&mut rng);
        let bob_ik_pk = PublicKey::from(&bob_ik);

        let bob_spk = StaticSecret::random_from_rng(&mut rng);
        let bob_spk_pk = PublicKey::from(&bob_spk);

        let alice_result = x3dh_initiate(&alice_ik, &alice_ek, &bob_ik_pk, &bob_spk_pk, None);

        let alice_ek_pk = PublicKey::from(&alice_ek);

        let bob_result = x3dh_receive(
            &PublicKey::from(&alice_ik),
            &alice_ek_pk,
            &bob_ik,
            &bob_spk,
            None,
        );

        (alice_result.shared_secret, bob_result.shared_secret)
    }

    #[test]
    fn test_ratchet_roundtrip() {
        let (alice_sk, bob_sk) = perform_x3dh();

        let mut alice = RatchetState::new(alice_sk, true);
        let mut bob = RatchetState::new(bob_sk, false);

        let msg = b"Hello, Double Ratchet!";
        let ad = b"associated data";

        let cipher = alice.encrypt(msg, ad).unwrap();
        let decrypted = bob.decrypt(&cipher, ad).unwrap();

        assert_eq!(decrypted, msg);
    }

    #[test]
    fn test_multiple_messages() {
        let (alice_sk, bob_sk) = perform_x3dh();

        let mut alice = RatchetState::new(alice_sk, true);
        let mut bob = RatchetState::new(bob_sk, false);

        let ad = b"ad";

        for i in 0..5 {
            let msg = format!("message {i}");
            let cipher = alice.encrypt(msg.as_bytes(), ad).unwrap();
            let decrypted = bob.decrypt(&cipher, ad).unwrap();
            assert_eq!(decrypted, msg.as_bytes(), "roundtrip failed for msg {i}");
        }
    }

    #[test]
    fn test_bidirectional() {
        let (alice_sk, bob_sk) = perform_x3dh();

        let mut alice = RatchetState::new(alice_sk, true);
        let mut bob = RatchetState::new(bob_sk, false);

        // Alice sends first message
        let cipher1 = alice.encrypt(b"hi from alice", b"ad").unwrap();
        let dec1 = bob.decrypt(&cipher1, b"ad").unwrap();
        assert_eq!(dec1, b"hi from alice");

        // Bob replies
        let cipher2 = bob.encrypt(b"hi from bob", b"ad").unwrap();
        let dec2 = alice.decrypt(&cipher2, b"ad").unwrap();
        assert_eq!(dec2, b"hi from bob");

        // Another round
        let cipher3 = alice.encrypt(b"how are you?", b"ad").unwrap();
        let dec3 = bob.decrypt(&cipher3, b"ad").unwrap();
        assert_eq!(dec3, b"how are you?");
    }
}
