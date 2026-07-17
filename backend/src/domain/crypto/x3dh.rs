use x25519_dalek::{PublicKey, StaticSecret};

use super::{KEY_LENGTH, SharedSecret, hkdf};

pub struct X3dhResult {
    pub shared_secret: SharedSecret,
    pub associated_data: Vec<u8>,
}

/// X3DH initiation (Alice).
/// All identity/pre-keys are X25519 key pairs.
/// The ephemeral key is a StaticSecret (not EphemeralSecret) so it can
/// be used for multiple DH operations (DH2, DH3, optional DH4).
pub fn x3dh_initiate(
    identity_sk: &StaticSecret,
    ephemeral_sk: &StaticSecret,
    bob_identity_pk: &PublicKey,
    bob_signed_pre_key_pk: &PublicKey,
    bob_one_time_pre_key_pk: Option<&PublicKey>,
) -> X3dhResult {
    let ephemeral_pk = PublicKey::from(ephemeral_sk);

    let dh1 = identity_sk.diffie_hellman(bob_signed_pre_key_pk);
    let dh2 = ephemeral_sk.diffie_hellman(bob_identity_pk);
    let dh3 = ephemeral_sk.diffie_hellman(bob_signed_pre_key_pk);
    let dh4 = bob_one_time_pre_key_pk.map(|opk| ephemeral_sk.diffie_hellman(opk));

    let mut dh_input = Vec::with_capacity(128);
    dh_input.extend_from_slice(dh1.as_bytes());
    dh_input.extend_from_slice(dh2.as_bytes());
    dh_input.extend_from_slice(dh3.as_bytes());
    if let Some(d) = &dh4 {
        dh_input.extend_from_slice(d.as_bytes());
    }

    let raw = hkdf(KEY_LENGTH, &[0u8; KEY_LENGTH], &dh_input);
    let mut sk = [0u8; KEY_LENGTH];
    sk.copy_from_slice(&raw);

    let mut ad = Vec::with_capacity(64);
    ad.extend_from_slice(bob_identity_pk.as_bytes());
    ad.extend_from_slice(ephemeral_pk.as_bytes());

    X3dhResult {
        shared_secret: SharedSecret::new(sk),
        associated_data: ad,
    }
}

/// X3DH reception (Bob).
pub fn x3dh_receive(
    alice_identity_pk: &PublicKey,
    alice_ephemeral_pk: &PublicKey,
    identity_sk: &StaticSecret,
    signed_pre_key_sk: &StaticSecret,
    consumed_one_time_key_sk: Option<&StaticSecret>,
) -> X3dhResult {
    let dh1 = signed_pre_key_sk.diffie_hellman(alice_identity_pk);
    let dh2 = identity_sk.diffie_hellman(alice_ephemeral_pk);
    let dh3 = signed_pre_key_sk.diffie_hellman(alice_ephemeral_pk);
    let dh4 = consumed_one_time_key_sk.map(|opk| opk.diffie_hellman(alice_ephemeral_pk));

    let mut dh_input = Vec::with_capacity(128);
    dh_input.extend_from_slice(dh1.as_bytes());
    dh_input.extend_from_slice(dh2.as_bytes());
    dh_input.extend_from_slice(dh3.as_bytes());
    if let Some(d) = &dh4 {
        dh_input.extend_from_slice(d.as_bytes());
    }

    let raw = hkdf(KEY_LENGTH, &[0u8; KEY_LENGTH], &dh_input);
    let mut sk = [0u8; KEY_LENGTH];
    sk.copy_from_slice(&raw);

    let mut ad = Vec::with_capacity(64);
    let identity_pk = PublicKey::from(identity_sk);
    ad.extend_from_slice(identity_pk.as_bytes());
    ad.extend_from_slice(alice_ephemeral_pk.as_bytes());

    X3dhResult {
        shared_secret: SharedSecret::new(sk),
        associated_data: ad,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    #[test]
    fn test_x3dh_agreement() {
        let mut rng = OsRng;

        let alice_ik = StaticSecret::random_from_rng(&mut rng);
        let alice_ek = StaticSecret::random_from_rng(&mut rng);
        let alice_ek_pk = PublicKey::from(&alice_ek);

        let bob_ik = StaticSecret::random_from_rng(&mut rng);
        let bob_ik_pk = PublicKey::from(&bob_ik);

        let bob_spk = StaticSecret::random_from_rng(&mut rng);
        let bob_spk_pk = PublicKey::from(&bob_spk);

        let bob_opk = StaticSecret::random_from_rng(&mut rng);
        let bob_opk_pk = PublicKey::from(&bob_opk);

        let alice_result = x3dh_initiate(
            &alice_ik,
            &alice_ek,
            &bob_ik_pk,
            &bob_spk_pk,
            Some(&bob_opk_pk),
        );

        let bob_result = x3dh_receive(
            &PublicKey::from(&alice_ik),
            &alice_ek_pk,
            &bob_ik,
            &bob_spk,
            Some(&bob_opk),
        );

        assert_eq!(
            alice_result.shared_secret.as_bytes(),
            bob_result.shared_secret.as_bytes(),
        );
        assert_eq!(alice_result.associated_data, bob_result.associated_data,);
    }

    #[test]
    fn test_x3dh_no_one_time() {
        let mut rng = OsRng;

        let alice_ik = StaticSecret::random_from_rng(&mut rng);
        let alice_ek = StaticSecret::random_from_rng(&mut rng);
        let alice_ek_pk = PublicKey::from(&alice_ek);

        let bob_ik = StaticSecret::random_from_rng(&mut rng);
        let bob_ik_pk = PublicKey::from(&bob_ik);

        let bob_spk = StaticSecret::random_from_rng(&mut rng);
        let bob_spk_pk = PublicKey::from(&bob_spk);

        let alice_result = x3dh_initiate(&alice_ik, &alice_ek, &bob_ik_pk, &bob_spk_pk, None);

        let bob_result = x3dh_receive(
            &PublicKey::from(&alice_ik),
            &alice_ek_pk,
            &bob_ik,
            &bob_spk,
            None,
        );

        assert_eq!(
            alice_result.shared_secret.as_bytes(),
            bob_result.shared_secret.as_bytes(),
        );
    }
}
