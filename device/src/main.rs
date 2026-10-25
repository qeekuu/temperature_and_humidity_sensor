use std::{thread, time::Duration};

use chrono::Utc;
use proto::SensorReading;
use rumqttc::{Client, LastWill, MqttOptions, QoS};

const DEVICE_ID: u64 = 34;

fn main() -> anyhow::Result<()> {
    let mut mqttoptions = MqttOptions::new(format!("sensor-{DEVICE_ID}"), "localhost", 1883);
    let will = LastWill::new(
        format!("sensor/{DEVICE_ID}/status"),
        "offline",
        QoS::AtMostOnce,
        false,
    );
    mqttoptions
        .set_keep_alive(Duration::from_secs(5))
        .set_last_will(will);

    let (client, mut connection) = Client::new(mqttoptions, 10);
    thread::spawn(move || {
        if let Err(e) = run_sensor(client) {
            eprintln!("sensor error: {e}");
        }
    });

    for notification in connection.iter() {
        match notification {
            Ok(event) => println!("{event:?}"),
            Err(error) => {
                eprintln!("connection error: {error}");
                thread::sleep(Duration::from_secs(2));
            }
        }
    }

    Ok(())
}

fn run_sensor(client: Client) -> anyhow::Result<()> {
    let topic = format!("sensor/{DEVICE_ID}/telemetry");
    let qos = QoS::AtLeastOnce;

    loop {
        let data = SensorReading {
            device_id: DEVICE_ID,
            temp: rand::random_range(0.0..45.0),
            humidity: rand::random_range(0.0..50.0),
            timestamp: Utc::now(),
        };
        let payload = serde_json::to_vec(&data)?;

        client.publish(&topic, qos, false, payload)?;

        thread::sleep(Duration::from_secs(3));
    }
}
