mod cli;

use std::{thread, time::Duration};

use chrono::Utc;
use clap::Parser;
use proto::SensorReading;
use rumqttc::{Client, LastWill, MqttOptions, QoS};

use crate::cli::{Args, Mode};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.mode {
        Mode::Plain => {}
        Mode::Tls => anyhow::bail!("TLS not implemented yet"),
        Mode::E2e => anyhow::bail!("E2E not implemented yet"),
    };

    let device_id = args.device_id;

    let mut mqttoptions = MqttOptions::new(format!("sensor-{device_id}"), "localhost", args.port);
    let will = LastWill::new(
        format!("sensor/{device_id}/status"),
        "offline",
        QoS::AtMostOnce,
        false,
    );
    mqttoptions
        .set_keep_alive(Duration::from_secs(5))
        .set_last_will(will);

    let (client, mut connection) = Client::new(mqttoptions, 10);
    thread::spawn(move || {
        if let Err(e) = run_sensor(client, device_id) {
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

fn run_sensor(client: Client, device_id: u64) -> anyhow::Result<()> {
    let topic = format!("sensor/{device_id}/telemetry");
    let qos = QoS::AtLeastOnce;

    loop {
        let data = SensorReading {
            device_id,
            temp: rand::random_range(0.0..45.0),
            humidity: rand::random_range(0.0..50.0),
            timestamp: Utc::now(),
        };
        let payload = serde_json::to_vec(&data)?;

        client.publish(&topic, qos, false, payload)?;

        thread::sleep(Duration::from_secs(3));
    }
}
