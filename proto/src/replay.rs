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

#[cfg(test)]
mod tests {
    use super::*;

    const DEVICE_ID: u64 = 34;
    const OTHER_DEVICE_ID: u64 = 50;

    #[track_caller]
    fn accepts(guard: &mut ReplayGuard, device_id: u64, counter: u64) {
        let result = guard.check(device_id, counter);
        assert!(
            result.is_ok(),
            "device {device_id}, counter {counter}: expected accept, got {result:?}"
        );
    }

    #[track_caller]
    fn rejects(guard: &mut ReplayGuard, device_id: u64, counter: u64) {
        let result = guard.check(device_id, counter);
        assert!(
            matches!(result, Err(ProtoError::Replay)),
            "device {device_id}, counter {counter}: expected Replay, got {result:?}"
        );
    }

    #[test]
    fn first_message_from_device_is_accepted() {
        let mut guard = ReplayGuard::new();
        accepts(&mut guard, DEVICE_ID, 1);
    }

    #[test]
    fn increasing_counters_are_accepted() {
        let mut guard = ReplayGuard::new();
        for counter in 1..=3 {
            accepts(&mut guard, DEVICE_ID, counter);
        }
    }

    #[test]
    fn same_counter_twice_returns_replay() {
        let mut guard = ReplayGuard::new();
        accepts(&mut guard, DEVICE_ID, 1);
        rejects(&mut guard, DEVICE_ID, 1);
    }

    #[test]
    fn counter_lower_than_last_returns_replay() {
        let mut guard = ReplayGuard::new();
        accepts(&mut guard, DEVICE_ID, 3);
        rejects(&mut guard, DEVICE_ID, 1);
    }

    #[test]
    fn same_counter_from_different_devices_is_accepted() {
        let mut guard = ReplayGuard::new();
        accepts(&mut guard, DEVICE_ID, 1);
        accepts(&mut guard, OTHER_DEVICE_ID, 1);
    }

    #[test]
    fn rejected_message_does_not_change_counter_state() {
        let mut guard = ReplayGuard::new();
        accepts(&mut guard, DEVICE_ID, 3);
        rejects(&mut guard, DEVICE_ID, 1);
        // Would be accepted if the rejected counter had overwritten the stored one.
        rejects(&mut guard, DEVICE_ID, 2);
        accepts(&mut guard, DEVICE_ID, 4);
    }
}
