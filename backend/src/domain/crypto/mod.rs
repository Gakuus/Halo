pub mod ratchet;
pub mod x3dh;

use sha2::{Digest, Sha256};
use zeroize::ZeroizeOnDrop;

pub const KEY_LENGTH: usize = 32;
pub const RATCHET_KEY_LENGTH: usize = 32;
pub const CHAIN_KEY_LENGTH: usize = 32;
pub const MAC_LENGTH: usize = 16;

#[derive(Debug, Clone, ZeroizeOnDrop)]
pub struct SharedSecret([u8; KEY_LENGTH]);

impl SharedSecret {
    pub fn new(data: [u8; KEY_LENGTH]) -> Self {
        Self(data)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Debug, Clone, ZeroizeOnDrop)]
pub struct ChainKey(pub [u8; CHAIN_KEY_LENGTH]);

impl ChainKey {
    pub fn new(data: [u8; CHAIN_KEY_LENGTH]) -> Self {
        Self(data)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

pub fn kdf_rk(rk: &SharedSecret, dh_output: &[u8]) -> (SharedSecret, ChainKey) {
    let mut input = Vec::with_capacity(KEY_LENGTH + dh_output.len());
    input.extend_from_slice(rk.as_bytes());
    input.extend_from_slice(dh_output);
    let raw = hkdf(KEY_LENGTH * 2, &[0u8; KEY_LENGTH], &input);
    let (rk_bytes, ck_bytes) = raw.split_at(KEY_LENGTH);

    let mut rk_arr = [0u8; KEY_LENGTH];
    rk_arr.copy_from_slice(rk_bytes);

    let mut ck_arr = [0u8; CHAIN_KEY_LENGTH];
    ck_arr.copy_from_slice(ck_bytes);

    (SharedSecret::new(rk_arr), ChainKey::new(ck_arr))
}

pub fn kdf_ck(ck: &ChainKey) -> (ChainKey, [u8; KEY_LENGTH]) {
    let mk = {
        let mut hk = Sha256::new();
        hk.update(ck.as_bytes());
        hk.update(&[0x01]);
        hk.finalize()
    };

    let ck_next = {
        let mut hk = Sha256::new();
        hk.update(ck.as_bytes());
        hk.update(&[0x02]);
        hk.finalize()
    };

    let mut ck_arr = [0u8; CHAIN_KEY_LENGTH];
    let mut mk_arr = [0u8; KEY_LENGTH];
    ck_arr.copy_from_slice(&ck_next);
    mk_arr.copy_from_slice(&mk);
    (ChainKey::new(ck_arr), mk_arr)
}

pub fn hkdf(length: usize, salt: &[u8], ikm: &[u8]) -> Vec<u8> {
    use hkdf::Hkdf;
    let (_, hk) = Hkdf::<Sha256>::extract(Some(salt), ikm);
    let mut okm = vec![0u8; length];
    hk.expand(&[], &mut okm).expect("hkdf expand");
    okm
}
