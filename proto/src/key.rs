use std::{fs, path::Path};

use crate::ProtoError;

pub fn load_key(path: &Path) -> Result<[u8; 32], ProtoError> {
    let bytes = fs::read(path)?;
    let key: [u8; 32] = bytes
        .try_into()
        .map_err(|bytes: Vec<u8>| ProtoError::InvalidKeyLength(bytes.len()))?;

    Ok(key)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn valid_key() {
        let path = std::env::temp_dir().join("proto_test_valid_key.bin");
        fs::write(&path, [7u8; 32]).unwrap();

        let load = load_key(&path);
        fs::remove_file(&path).unwrap();

        assert_eq!(load.unwrap(), [7u8; 32]);
    }

    #[test]
    fn too_short_key() {
        let path = std::env::temp_dir().join("proto_test_short_key.bin");
        fs::write(&path, [7u8; 31]).unwrap();

        let load = load_key(&path);
        fs::remove_file(&path).unwrap();

        assert!(matches!(load, Err(ProtoError::InvalidKeyLength(31))));
    }

    #[test]
    fn file_does_not_exist() {
        let path = std::env::temp_dir().join("proto_test_missing_key.bin");

        let load = load_key(&path);

        assert!(matches!(load, Err(ProtoError::Io(_))));
    }
}
