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
    unimplemented!();
}

pub fn decrypt(key: &[u8; 32], envelope: &Envelope) -> Result<Vec<u8>, ProtoError> {
    unimplemented!();
}
