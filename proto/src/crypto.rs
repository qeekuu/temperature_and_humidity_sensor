use chacha20poly1305::{
    ChaCha20Poly1305, Nonce,
    aead::{Aead, KeyInit, Payload},
};

use crate::{Envelope, ProtoError, SensorReading};

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

fn encrypt(
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

fn decrypt(key: &[u8; 32], envelope: &Envelope) -> Result<Vec<u8>, ProtoError> {
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

pub fn encrypt_reading(
    key: &[u8; 32],
    counter: u64,
    reading: &SensorReading,
) -> Result<Envelope, ProtoError> {
    let bytes = serde_json::to_vec(reading)?;
    encrypt(key, reading.device_id, counter, &bytes)
}

pub fn decrypt_reading(key: &[u8; 32], envelope: &Envelope) -> Result<SensorReading, ProtoError> {
    let bytes = decrypt(key, envelope)?;
    Ok(serde_json::from_slice(&bytes)?)
}

#[cfg(test)]
mod tests {

    use super::*;
    use chrono::Utc;

    const KEY: [u8; 32] = [7u8; 32];
    const DEVICE_ID: u64 = 34;
    const COUNTER: u64 = 1;
    const PLAINTEXT: &[u8] = b"hello";

    fn sample_envelope() -> Envelope {
        encrypt(&KEY, DEVICE_ID, COUNTER, PLAINTEXT).unwrap()
    }

    #[test]
    fn decrypt_returns_original_plaintext() {
        let envelope = sample_envelope();
        let decrypted = decrypt(&KEY, &envelope).unwrap();
        assert_eq!(decrypted, PLAINTEXT);
    }

    #[test]
    fn modified_ciphertext_returns_crypto() {
        let mut envelope = sample_envelope();
        envelope.ciphertext[0] ^= 1;
        let result = decrypt(&KEY, &envelope);

        assert!(matches!(result, Err(ProtoError::Crypto)));
    }

    #[test]
    fn modified_counter_returns_crypto() {
        let mut envelope = sample_envelope();
        envelope.counter += 1;
        let result = decrypt(&KEY, &envelope);

        assert!(matches!(result, Err(ProtoError::Crypto)));
    }

    #[test]
    fn modified_device_id_returns_crypto() {
        let mut envelope = sample_envelope();
        envelope.device_id = 57;
        let result = decrypt(&KEY, &envelope);

        assert!(matches!(result, Err(ProtoError::Crypto)));
    }

    #[test]
    fn wrong_key_returns_crypto() {
        let envelope = sample_envelope();
        let result = decrypt(&[8u8; 32], &envelope);

        assert!(matches!(result, Err(ProtoError::Crypto)));
    }

    #[test]
    fn encrypt_with_different_counters_results_in_diffrent_ciphertext() {
        let envelope_one = encrypt(&KEY, DEVICE_ID, COUNTER, PLAINTEXT).unwrap();
        let envelope_two = encrypt(&KEY, DEVICE_ID, COUNTER + 1, PLAINTEXT).unwrap();

        assert_ne!(envelope_one.ciphertext, envelope_two.ciphertext);
    }

    fn sample_reading() -> SensorReading {
        SensorReading {
            device_id: DEVICE_ID,
            temp: 22.5,
            humidity: 45.0,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn decrypt_reading_returns_original_reading() {
        let reading = sample_reading();
        let envelope = encrypt_reading(&KEY, COUNTER, &reading).unwrap();
        let decrypted = decrypt_reading(&KEY, &envelope).unwrap();

        assert_eq!(decrypted, reading);
    }

    #[test]
    fn encrypt_reading_sets_envelope_header() {
        let reading = sample_reading();
        let envelope = encrypt_reading(&KEY, COUNTER, &reading).unwrap();

        assert_eq!(envelope.device_id, reading.device_id);
        assert_eq!(envelope.counter, COUNTER);
    }

    #[test]
    fn decrypt_reading_with_invalid_json_returns_serde() {
        let envelope = sample_envelope();
        let result = decrypt_reading(&KEY, &envelope);

        assert!(matches!(result, Err(ProtoError::Serde(_))));
    }
}
