use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::ProtoError;

#[derive(Serialize, Deserialize, Debug)]
pub struct Manifest {
    pub version: u32,
    pub image_digest: String,
    pub signature: String,
}

fn image_digest(image: &[u8]) -> [u8; 32] {
    Sha256::digest(image).into()
}

fn bytes_to_sign(version: u32, image_digest: [u8; 32]) -> Vec<u8> {
    let mut message = Vec::with_capacity(4 + 32);
    message.extend_from_slice(&version.to_be_bytes());
    message.extend_from_slice(&image_digest);

    message
}

pub fn to_public_key(key: &[u8; 32]) -> [u8; 32] {
    let signing_key = SigningKey::from_bytes(key);
    let public_key = signing_key.verifying_key();

    public_key.to_bytes()
}

pub fn sign_manifest(private_key: &[u8; 32], version: u32, image: &[u8]) -> Manifest {
    let digest = image_digest(image);
    let bytes = bytes_to_sign(version, digest);
    let signing_key = SigningKey::from_bytes(private_key);
    let signature = signing_key.sign(&bytes);

    Manifest {
        version,
        image_digest: hex::encode(digest),
        signature: hex::encode(signature.to_bytes()),
    }
}

pub fn verify_manifest(
    manifest: &Manifest,
    image: &[u8],
    public_key: &[u8; 32],
    current_version: u32,
) -> Result<(), ProtoError> {
    let public_key =
        VerifyingKey::from_bytes(public_key).map_err(|_| ProtoError::InvalidSignature)?;

    let mut signature_array = [0u8; 64];
    hex::decode_to_slice(&manifest.signature, &mut signature_array)
        .map_err(|_| ProtoError::InvalidSignature)?;
    let signature = Signature::from_bytes(&signature_array);

    let mut image_digest_array = [0u8; 32];
    hex::decode_to_slice(&manifest.image_digest, &mut image_digest_array)
        .map_err(|_| ProtoError::InvalidDigest)?;

    let message = bytes_to_sign(manifest.version, image_digest_array);
    public_key
        .verify_strict(&message, &signature)
        .map_err(|_| ProtoError::InvalidSignature)?;

    let digest = image_digest(image);

    if digest != image_digest_array {
        return Err(ProtoError::InvalidDigest);
    }

    if manifest.version <= current_version {
        return Err(ProtoError::VersionTooOld);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const RFC_PRIVATE_KEY: &str =
        "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";
    const RFC_PUBLIC_KEY: &str = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
    const MANUFACTURER_PRIVATE_KEY: [u8; 32] = [7u8; 32];
    const ATTACKER_PRIVATE_KEY: [u8; 32] = [9u8; 32];
    const IMAGE: &[u8] = b"firmware image";

    #[test]
    fn derives_public_key_from_rfc_vector() {
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(RFC_PRIVATE_KEY, &mut bytes).unwrap();
        let public_key = to_public_key(&bytes);

        assert_eq!(hex::encode(public_key), RFC_PUBLIC_KEY);
    }

    #[test]
    fn image_digest_matches_nist_vector_for_abc() {
        let digest = image_digest(b"abc");
        assert_eq!(
            hex::encode(digest),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    fn return_signed_manifest(version: u32) -> Manifest {
        sign_manifest(&MANUFACTURER_PRIVATE_KEY, version, IMAGE)
    }

    #[test]
    fn accepts_valid_firmware() {
        let manifest = return_signed_manifest(2);
        let public_key = to_public_key(&MANUFACTURER_PRIVATE_KEY);

        assert!(verify_manifest(&manifest, IMAGE, &public_key, 1).is_ok());
    }

    #[test]
    fn rejects_tampered_image() {
        let manifest = return_signed_manifest(2);
        let public_key = to_public_key(&MANUFACTURER_PRIVATE_KEY);
        let tempered_image: &[u8] = b"image";

        let result = verify_manifest(&manifest, tempered_image, &public_key, 1);

        assert!(matches!(result, Err(ProtoError::InvalidDigest)));
    }

    #[test]
    fn rejects_tampered_signature() {
        let mut manifest = return_signed_manifest(2);
        let public_key = to_public_key(&MANUFACTURER_PRIVATE_KEY);

        let mut signature = hex::decode(&manifest.signature).expect("signature is valid hex");
        signature[0] ^= 1;
        manifest.signature = hex::encode(signature);

        let result = verify_manifest(&manifest, IMAGE, &public_key, 1);

        assert!(matches!(result, Err(ProtoError::InvalidSignature)));
    }

    #[test]
    fn rejects_attacker_signature() {
        let manifest = sign_manifest(&ATTACKER_PRIVATE_KEY, 2, IMAGE);
        let public_key = to_public_key(&MANUFACTURER_PRIVATE_KEY);

        let result = verify_manifest(&manifest, IMAGE, &public_key, 1);

        assert!(matches!(result, Err(ProtoError::InvalidSignature)));
    }

    #[test]
    fn rejects_raised_version() {
        let mut manifest = return_signed_manifest(2);
        manifest.version = 99;
        let public_key = to_public_key(&MANUFACTURER_PRIVATE_KEY);

        let result = verify_manifest(&manifest, IMAGE, &public_key, 2);

        assert!(matches!(result, Err(ProtoError::InvalidSignature)));
    }

    #[test]
    fn rejects_same_version() {
        let manifest = return_signed_manifest(2);
        let public_key = to_public_key(&MANUFACTURER_PRIVATE_KEY);

        let result = verify_manifest(&manifest, IMAGE, &public_key, 2);

        assert!(matches!(result, Err(ProtoError::VersionTooOld)));
    }

    #[test]
    fn rejects_downgrade_version() {
        let manifest = return_signed_manifest(1);
        let public_key = to_public_key(&MANUFACTURER_PRIVATE_KEY);

        let result = verify_manifest(&manifest, IMAGE, &public_key, 2);

        assert!(matches!(result, Err(ProtoError::VersionTooOld)));
    }
}
