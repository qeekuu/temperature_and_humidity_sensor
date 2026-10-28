mod cli;

use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::thread;
use std::time::Duration;

use anyhow::Context;
use clap::Parser;
use proto::{Envelope, ReplayGuard, SensorReading, decrypt_reading};
use rumqttc::{Client, Event, Incoming, MqttOptions, Publish, QoS, TlsConfiguration, Transport};

use crate::cli::{Cli, Mode};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        cli::Commands::Listen {
            mode,
            key,
            ca_file,
            port,
            login,
            password,
        } => listen(mode, &key, &ca_file, port, login, password),
        cli::Commands::Keygen { out } => keygen(&out),
    }
}

fn listen(
    mode: Mode,
    key_path: &Path,
    ca_file: &Path,
    port: u16,
    login: String,
    password: Option<String>,
) -> anyhow::Result<()> {
    let key = match mode {
        Mode::Plain | Mode::Tls => None,
        Mode::E2e => Some(proto::load_key(key_path)?),
    };

    let mqttoptions = match mode {
        Mode::Plain | Mode::E2e => mqtt_plain_e2e(port),
        Mode::Tls => {
            let password = password.context("set MQTT_PASSWORD_SERVER for tls mode")?;
            mqtt_tls(ca_file, port, login, password)?
        }
    };

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

fn mqtt_tls(
    ca_file: &Path,
    port: u16,
    login: String,
    password: String,
) -> anyhow::Result<MqttOptions> {
    let mut mqttoptions = MqttOptions::new("server", "localhost", port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_credentials(login, password);

    let ca = fs::read(ca_file)?;

    let transport = Transport::Tls(TlsConfiguration::Simple {
        ca,
        alpn: None,
        client_auth: None,
    });
    mqttoptions.set_transport(transport);

    Ok(mqttoptions)
}

fn mqtt_plain_e2e(port: u16) -> MqttOptions {
    let mut mqttoptions = MqttOptions::new("server", "localhost", port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    mqttoptions
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
