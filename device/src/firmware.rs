use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Slot {
    A,
    B,
}

impl Slot {
    pub fn other(self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::A,
        }
    }

    pub fn file_name(self) -> &'static str {
        match self {
            Self::A => "slot_a.bin",
            Self::B => "slot_b.bin",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct FirmwareState {
    pub active_slot: Slot,
    pub version: u32,
}

impl FirmwareState {
    pub fn load(dir: &Path) -> anyhow::Result<FirmwareState> {
        let state_file = dir.join("state.json");
        if !state_file.try_exists()? {
            return Ok(FirmwareState {
                active_slot: Slot::A,
                version: 0,
            });
        }

        let bytes = fs::read(&state_file)
            .with_context(|| format!("cannot read json file {}", state_file.display()))?;

        let state: FirmwareState = serde_json::from_slice(&bytes)
            .with_context(|| format!("cannot parse json file {}", state_file.display()))?;

        Ok(state)
    }

    pub fn save(&self, dir: &Path) -> anyhow::Result<()> {
        fs::create_dir_all(dir)?;
        let state_file = dir.join("state.json");
        let temp_file = dir.join("temp_state.json");

        let payload = serde_json::to_vec_pretty(self)?;

        let mut file = File::create(&temp_file)
            .with_context(|| format!("cannot create temp file {}", temp_file.display()))?;
        file.write_all(&payload)?;
        file.sync_all()?;

        fs::rename(&temp_file, &state_file)
            .with_context(|| format!("cannot rename file {}", state_file.display()))?;

        Ok(())
    }

    pub fn install(&mut self, dir: &Path, image: &[u8], version: u32) -> anyhow::Result<()> {
        fs::create_dir_all(dir)?;
        let target = self.active_slot.other();
        let image_file = dir.join(Slot::file_name(target));

        let mut file = File::create(&image_file)
            .with_context(|| format!("cannot create image file {}", image_file.display()))?;
        file.write_all(image)?;
        file.sync_all()?;

        let state = FirmwareState {
            active_slot: target,
            version,
        };
        state.save(dir)?;
        *self = state;

        Ok(())
    }
}

pub fn handle_update(
    payload: &[u8],
    public_key: &[u8; 32],
    insecure: bool,
    state: &mut FirmwareState,
    dir: &Path,
) -> anyhow::Result<()> {
    let update: proto::FirmwareUpdate = serde_json::from_slice(payload)?;

    let image = if insecure {
        eprintln!("WARNING: firmware verification disabled");
        hex::decode(&update.image)?
    } else {
        proto::verify_firmware(&update, public_key, state.version)?
    };

    state.install(dir, &image, update.manifest.version)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn clean_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("device_test_firmware_{name}"));
        let _ = fs::remove_dir_all(&dir);

        dir
    }

    #[test]
    fn other_returns_opposite_slot() {
        assert_eq!(Slot::A.other(), Slot::B);
        assert_eq!(Slot::B.other(), Slot::A);
    }

    #[test]
    fn slots_use_different_files() {
        assert_ne!(Slot::A.file_name(), Slot::B.file_name());
    }

    #[test]
    fn missing_state_is_factory_state() {
        let dir = clean_dir("missing");

        let state = FirmwareState::load(&dir).unwrap();

        assert_eq!(
            state,
            FirmwareState {
                active_slot: Slot::A,
                version: 0
            }
        );
    }

    #[test]
    fn saved_state_can_be_loaded() {
        let dir = clean_dir("roundtrip");
        let state = FirmwareState {
            active_slot: Slot::B,
            version: 2,
        };

        state.save(&dir).unwrap();
        let loaded = FirmwareState::load(&dir);
        fs::remove_dir_all(&dir).unwrap();

        assert_eq!(loaded.unwrap(), state);
    }

    #[test]
    fn save_replaces_previous_state() {
        let dir = clean_dir("replace");
        let old = FirmwareState {
            active_slot: Slot::B,
            version: 1,
        };
        let new = FirmwareState {
            active_slot: Slot::A,
            version: 2,
        };

        old.save(&dir).unwrap();
        new.save(&dir).unwrap();
        let loaded = FirmwareState::load(&dir);
        fs::remove_dir_all(&dir).unwrap();

        assert_eq!(loaded.unwrap(), new);
    }

    #[test]
    fn save_leaves_only_state_file() {
        let dir = clean_dir("leftovers");
        let state = FirmwareState {
            active_slot: Slot::B,
            version: 1,
        };

        state.save(&dir).unwrap();
        let files: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        fs::remove_dir_all(&dir).unwrap();

        assert_eq!(files, ["state.json"]);
    }

    #[test]
    fn corrupted_state_is_an_error() {
        let dir = clean_dir("corrupted");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("state.json"), "not a json").unwrap();

        let loaded = FirmwareState::load(&dir);
        fs::remove_dir_all(&dir).unwrap();

        assert!(loaded.is_err());
    }
}
