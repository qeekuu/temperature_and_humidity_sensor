use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    /// Specifies the communication mode you're running the application with
    #[arg(short, long, value_enum, default_value_t = Mode::Plain)]
    pub mode: Mode,
    /// Specifies device id (u64) you're running the application with
    #[arg(short, long, default_value_t = 34)]
    pub device_id: u64,
    /// Specifies the port you're running the application with (MQTT broker)
    #[arg(short, long, default_value_t = 1883)]
    pub port: u16,
    /// Specifies the input path to key
    #[arg(short, long, default_value = "keys/key.bin")]
    pub key: PathBuf,
    /// Specifies the input path to counter file
    #[arg(short, long, default_value = "state/counter.txt")]
    pub counter_file: PathBuf,
    /// CA certificate used to verify the broker (tls mode)
    #[arg(short = 'C', long, default_value = "certs/ca.crt")]
    pub ca_file: PathBuf,
    /// Login credentials for TLS connection
    #[arg(long, default_value = "sensor-34")]
    pub login: String,
    /// Password credentials for TLS connection
    #[arg(long, env = "MQTT_PASSWORD_SENSOR_34")]
    pub password: Option<String>,
}

#[derive(ValueEnum, Copy, Clone, Debug)]
pub enum Mode {
    /// Unencrypted MQTT
    Plain,
    /// MQTT over TLS
    Tls,
    /// Encrypted MQTT by Application-layer encryption (ChaCha20-Poly1305)
    E2e,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Args::command().debug_assert();
    }
}
