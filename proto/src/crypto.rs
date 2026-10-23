use chacha20poly1305::{
    ChaCha20Poly1305, Nonce,
    aead::{Aead, KeyInit, Payload},
};

use crate::{Envelope, ProtoError};

fn make_nonce(counter: u64) -> Nonce {
    let mut nonce_bytes = [0u8; 12];
    let counter_bytes = counter.to_be_bytes();
    nonce_bytes[..8].copy_from_slice(&counter_bytes);

    Nonce::from(nonce_bytes)
}

fn make_aad(device_id: u64, counter: u64) -> Vec<u8> {
    // [device_id.to_be_bytes(), counter.to_be_bytes()].concat()
    let mut aad_bytes = Vec::new();
    let device_id_bytes = device_id.to_be_bytes();
    let counter_bytes = counter.to_be_bytes();

    aad_bytes.extend_from_slice(&device_id_bytes);
    aad_bytes.extend_from_slice(&counter_bytes);

    aad_bytes
}

pub fn encrypt(
    key: &[u8; 32],
    device_id: u64,
    counter: u64,
    plaintext: &[u8],
) -> Result<Envelope, ProtoError> {
    let cipher = ChaCha20Poly1305::new(key.into());

    let nonce = make_nonce(counter);
    let aad = make_aad(device_id, counter);

    let ciphertext = match cipher.encrypt(
        &nonce,
        Payload {
            msg: plaintext,
            aad: &aad,
        },
    ) {
        Ok(ciphertext) => ciphertext,
        Err(_) => return Err(ProtoError::Crypto),
    };

    Ok(Envelope {
        device_id,
        counter,
        ciphertext,
    })
}

pub fn decrypt(key: &[u8; 32], envelope: &Envelope) -> Result<Vec<u8>, ProtoError> {
    let cipher = ChaCha20Poly1305::new(key.into());

    let nonce = make_nonce(envelope.counter);
    let aad = make_aad(envelope.device_id, envelope.counter);

    let plaintext = match cipher.decrypt(
        &nonce,
        Payload {
            msg: &envelope.ciphertext,
            aad: &aad,
        },
    ) {
        Ok(plaintext) => plaintext,
        Err(_) => return Err(ProtoError::Crypto),
    };

    Ok(plaintext)
}
