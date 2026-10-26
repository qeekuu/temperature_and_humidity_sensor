mod cli;

use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{fs, thread};

use anyhow::Context;
use clap::Parser;
use proto::{Envelope, ReplayGuard, SensorReading, decrypt_reading};
use rumqttc::{Client, Event, Incoming, MqttOptions, Publish, QoS};

use crate::cli::{Cli, Mode};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        cli::Commands::Listen { mode, key } => listen(mode, key),
        cli::Commands::Keygen { out } => keygen(&out),
    }
}

fn listen(mode: Mode, key_path: PathBuf) -> anyhow::Result<()> {
    let key = match mode {
        Mode::Plain => None,
        Mode::Tls => anyhow::bail!("TLS not implemented yet"),
        Mode::E2e => Some(proto::load_key(&key_path)?),
    };

    let mut mqttoptions = MqttOptions::new("server", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut connection) = Client::new(mqttoptions, 10);

    client.subscribe("sensor/+/telemetry", QoS::AtLeastOnce)?;

    let mut guard = ReplayGuard::new();

    for notification in connection.iter() {
        match notification {
            Ok(Event::Incoming(Incoming::Publish(packet))) => {
                match handle_message(&packet, key, &mut guard) {
                    Ok(reading) => println!("Received data from device: {reading:?}"),
                    Err(error) => eprintln!("Invalid message on {}: {error}", packet.topic),
                }
            }
            Ok(_) => {}
            Err(error) => {
                eprintln!("Error: {error}");
                thread::sleep(Duration::from_secs(2));
            }
        }
    }

    Ok(())
}

fn handle_message(
    packet: &Publish,
    key: Option<[u8; 32]>,
    guard: &mut ReplayGuard,
) -> anyhow::Result<SensorReading> {
    let reading = match key {
        Some(key) => {
            let envelope = serde_json::from_slice::<Envelope>(&packet.payload)?;

            let expected_topic = format!("sensor/{}/telemetry", envelope.device_id);
            if packet.topic != expected_topic {
                anyhow::bail!(
                    "topic mismatch: expected {expected_topic}, got {}",
                    packet.topic
                );
            }

            let reading = decrypt_reading(&key, &envelope)?;

            guard.check(envelope.device_id, envelope.counter)?;

            reading
        }
        None => serde_json::from_slice::<SensorReading>(&packet.payload)?,
    };

    Ok(reading)
}

fn keygen(out: &Path) -> anyhow::Result<()> {
    let key: [u8; 32] = rand::random();

    if let Some(dir) = out.parent() {
        fs::create_dir_all(dir)?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(out)
        .with_context(|| format!("cannot create key file {}", out.display()))?;
    file.write_all(&key)?;

    Ok(())
}
