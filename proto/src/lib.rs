use serde::{ Serialize, Deserialize };
use chrono::{ DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct SensorReading {
    device_id: u64,
    temp: f64,
    humidity: f64,
    timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Envelope {
    device_id: u64,
    counter: u64,
    nonce: u64,
    ciphertext: u64,
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn serialize_and_deserialize_readings() {
        let read_data = SensorReading {
            device_id: 1,
            temp: 36.6,
            humidity: 67.5,
            timestamp: Utc::now(),
        };

        let serialized = serde_json::to_string(&read_data).unwrap();
        let deserialize: SensorReading = serde_json::from_str(&serialized).unwrap();

        assert_eq!(read_data.device_id, deserialize.device_id);
        assert_eq!(read_data.timestamp, deserialize.timestamp);
    }
}
