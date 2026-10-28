mod cli;

use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use anyhow::Context;
use chrono::Utc;
use clap::Parser;
use proto::{SensorReading, encrypt_reading};
use rumqttc::{Client, LastWill, MqttOptions, QoS, TlsConfiguration, Transport};

use crate::cli::{Args, Mode};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let key = match args.mode {
        Mode::Plain | Mode::Tls => None,
        Mode::E2e => Some(proto::load_key(&args.key)?),
    };

    let device_id = args.device_id;
    let counter_file = args.counter_file;

    let will = LastWill::new(
        format!("sensor/{device_id}/status"),
        "offline",
        QoS::AtMostOnce,
        false,
    );

    let mqttoptions = mqtt(
        args.mode,
        &args.ca_file,
        device_id,
        will,
        args.port,
        args.login,
        args.password,
    )?;
    let (client, mut connection) = Client::new(mqttoptions, 10);
    thread::spawn(move || {
        if let Err(e) = run_sensor(client, device_id, key, counter_file) {
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

fn mqtt(
    mode: Mode,
    ca_file: &Path,
    device_id: u64,
    will: LastWill,
    port: u16,
    login: String,
    password: Option<String>,
) -> anyhow::Result<MqttOptions> {
    let mut mqttoptions = MqttOptions::new(format!("sensor-{device_id}"), "localhost", port);
    mqttoptions
        .set_keep_alive(Duration::from_secs(5))
        .set_last_will(will);

    match mode {
        Mode::Plain | Mode::E2e => {}
        Mode::Tls => {
            let ca = fs::read(ca_file)?;
            let transport = Transport::Tls(TlsConfiguration::Simple {
                ca,
                alpn: None,
                client_auth: None,
            });
            let password = password.context("set MQTT_PASSWORD_SERVER for tls mode")?;

            mqttoptions
                .set_transport(transport)
                .set_credentials(login, password);
        }
    }

    Ok(mqttoptions)
}

fn run_sensor(
    client: Client,
    device_id: u64,
    key: Option<[u8; 32]>,
    counter_file: PathBuf,
) -> anyhow::Result<()> {
    let topic = format!("sensor/{device_id}/telemetry");
    let qos = QoS::AtLeastOnce;
    let mut counter = load_counter(&counter_file)?;

    loop {
        let data = SensorReading {
            device_id,
            temp: rand::random_range(0.0..45.0),
            humidity: rand::random_range(0.0..50.0),
            timestamp: Utc::now(),
        };

        let payload = match key {
            Some(key) => {
                counter += 1;
                save_counter(&counter_file, counter)?;
                let envelope = encrypt_reading(&key, counter, &data)?;
                serde_json::to_vec(&envelope)?
            }
            None => serde_json::to_vec(&data)?,
        };

        client.publish(&topic, qos, false, payload)?;
        thread::sleep(Duration::from_secs(3));
    }
}

fn load_counter(path: &Path) -> anyhow::Result<u64> {
    let exists = path.try_exists()?;
    if !exists {
        return Ok(0);
    }

    let content = fs::read_to_string(path)?;
    let counter = content.trim().parse::<u64>()?;

    Ok(counter)
}

fn save_counter(path: &Path, counter: u64) -> anyhow::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }

    let counter_text = counter.to_string();

    fs::write(path, counter_text)?;

    Ok(())
}
