use std::collections::HashMap;

use crate::ProtoError;

#[derive(Default)]
pub struct ReplayGuard {
    last_counters: HashMap<u64, u64>,
}

impl ReplayGuard {
    pub fn new() -> Self {
        ReplayGuard {
            last_counters: HashMap::new(),
        }
    }

    pub fn check(&mut self, device_id: u64, counter: u64) -> Result<(), ProtoError> {
        if let Some(last) = self.last_counters.get(&device_id)
            && counter <= *last
        {
            return Err(ProtoError::Replay);
        }

        self.last_counters.insert(device_id, counter);
        Ok(())
    }
}
