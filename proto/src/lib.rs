use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SensorReading {
    pub device_id: u64,
    pub temp: f64,
    pub humidity: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Envelope {
    pub device_id: u64,
    pub counter: u64,
    pub ciphertext: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_and_deserialize_readings() {
        let read_data = SensorReading {
            device_id: 1,
            temp: 36.6,
            humidity: 67.5,
            timestamp: Utc::now(),
        };

        let serialized = serde_json::to_string(&read_data).unwrap();
        let deserialized: SensorReading = serde_json::from_str(&serialized).unwrap();

        assert_eq!(read_data, deserialized);
    }

    #[test]
    fn serialize_and_deserialize_envelope() {
        let envelope = Envelope {
            device_id: 1,
            counter: 1,
            ciphertext: vec![1, 2, 3, 4, 5],
        };

        let serialized = serde_json::to_string(&envelope).unwrap();
        let deserialized: Envelope = serde_json::from_str(&serialized).unwrap();

        assert_eq!(envelope, deserialized);
    }
}
