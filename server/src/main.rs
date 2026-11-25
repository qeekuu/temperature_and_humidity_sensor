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

const MAX_PACKET_SIZE: usize = 1024 * 1024;

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
        cli::Commands::FwKeygen { public, private } => firmware_keygen(&public, &private),
        cli::Commands::FwSign {
            firmware,
            version,
            private,
            out,
        } => fw_sign(&firmware, version, &private, out.as_deref()),
        cli::Commands::FwPublish {
            mode,
            ca_file,
            port,
            login,
            password,
            json_file,
            device_id,
        } => fw_publish(mode, &ca_file, port, login, password, &json_file, device_id),
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
        Mode::Plain | Mode::E2e => mqtt_plain_e2e(port, "server"),
        Mode::Tls => {
            let password = password.context("set MQTT_PASSWORD_SERVER for tls mode")?;
            mqtt_tls(ca_file, port, login, password, "server")?
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
    client_id: &str,
) -> anyhow::Result<MqttOptions> {
    let mut mqttoptions = MqttOptions::new(client_id, "localhost", port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_credentials(login, password);
    mqttoptions.set_max_packet_size(MAX_PACKET_SIZE, MAX_PACKET_SIZE);

    let ca = fs::read(ca_file)?;

    let transport = Transport::Tls(TlsConfiguration::Simple {
        ca,
        alpn: None,
        client_auth: None,
    });
    mqttoptions.set_transport(transport);

    Ok(mqttoptions)
}

fn mqtt_plain_e2e(port: u16, client_id: &str) -> MqttOptions {
    let mut mqttoptions = MqttOptions::new(client_id, "localhost", port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_max_packet_size(MAX_PACKET_SIZE, MAX_PACKET_SIZE);

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

    write_file(out, &key, 0o600)?;

    Ok(())
}

fn firmware_keygen(pub_out: &Path, priv_out: &Path) -> anyhow::Result<()> {
    let key: [u8; 32] = rand::random();

    let public_key = proto::to_public_key(&key);

    write_file(priv_out, &key, 0o600)?;
    write_file(pub_out, &public_key, 0o644)?;

    Ok(())
}

fn write_file(path: &Path, key: &[u8], mode: u32) -> anyhow::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(mode)
        .open(path)
        .with_context(|| format!("cannot create file {}", path.display()))?;
    file.write_all(key)?;

    Ok(())
}

fn fw_sign(
    firmware: &Path,
    version: u32,
    private_key_path: &Path,
    out: Option<&Path>,
) -> anyhow::Result<()> {
    let private_key = proto::load_key(private_key_path)?;
    let image = fs::read(firmware)
        .with_context(|| format!("cannot read firmware file {}", firmware.display()))?;

    let update = proto::sign_firmware(&private_key, version, &image);
    let update_json = serde_json::to_vec_pretty(&update)?;

    let out_path = out.map_or_else(
        || firmware.with_extension("signed.json"),
        |path| path.to_path_buf(),
    );

    write_file(&out_path, &update_json, 0o644)?;

    println!("Signed firmware version {}", update.manifest.version);
    println!("SHA-256: {}", update.manifest.image_digest);
    println!("Output: {}", out_path.display());

    Ok(())
}

fn fw_publish(
    mode: Mode,
    ca_file: &Path,
    port: u16,
    login: String,
    password: Option<String>,
    json_file: &Path,
    device_id: u64,
) -> anyhow::Result<()> {
    let payload = fs::read(json_file)
        .with_context(|| format!("cannot read json file {}", json_file.display()))?;
    let mqttoptions = match mode {
        Mode::Plain | Mode::E2e => mqtt_plain_e2e(port, "server-fw"),
        Mode::Tls => {
            let password = password.context("set MQTT_PASSWORD_SERVER for tls mode")?;
            mqtt_tls(ca_file, port, login, password, "server-fw")?
        }
    };

    let (client, mut connection) = Client::new(mqttoptions, 10);
    let topic = format!("firmware/{device_id}/update");

    client.publish(&topic, QoS::AtLeastOnce, true, payload)?;

    for notification in connection.iter() {
        match notification {
            Ok(Event::Incoming(Incoming::PubAck(_))) => {
                println!("Published firmware update on {topic}");
                return Ok(());
            }
            Ok(_) => {}
            Err(error) => {
                return Err(error.into());
            }
        }
    }
    anyhow::bail!("connection closed before broker confirmed the update");
}
