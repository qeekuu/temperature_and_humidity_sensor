use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Receive and display sensor readings
    Listen {
        /// Specifies the communication mode you're running the application with
        #[arg(short, long, value_enum, default_value_t = Mode::Plain)]
        mode: Mode,
        /// Specifies the key path for e2e
        #[arg(short, long, default_value = "keys/key.bin")]
        key: PathBuf,
        /// CA certificate used to verify the broker (tls mode)
        #[arg(short = 'C', long, default_value = "certs/ca.crt")]
        ca_file: PathBuf,
        /// Specifies the port
        #[arg(short, long, default_value_t = 1883)]
        port: u16,
        /// Login credentials for TLS connection
        #[arg(long, default_value = "server")]
        login: String,
        /// Password credentials for TLS connection
        #[arg(long, env = "MQTT_PASSWORD_SERVER")]
        password: Option<String>,
    },
    /// Generate a new pre-shared key for e2e mode
    Keygen {
        /// Output path for the key
        #[arg(short, long, default_value = "keys/key.bin")]
        out: PathBuf,
    },
    /// Generate a new public and private keys for firmware updates
    FwKeygen {
        /// Specifies the path to public key
        #[arg(long, default_value = "keys/fw_public.bin")]
        public: PathBuf,
        /// Specifies the path to private key
        #[arg(long, default_value = "keys/fw_private.bin")]
        private: PathBuf,
    },
    /// Sign a firmware image and write the update file
    FwSign {
        /// Specifies the path to firmware image
        #[arg(long)]
        firmware: PathBuf,
        /// Specifies the version of the firmware
        #[arg(short, long)]
        version: u32,
        /// Specifies the path to private key
        #[arg(long, default_value = "keys/fw_private.bin")]
        private: PathBuf,
        /// Output path for the signed update [default: <FIRMWARE> with .signed.json extension]
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
    /// Publish a signed firmware update
    FwPublish {
        /// Specifies the communication mode you're running the application with
        #[arg(short, long, value_enum, default_value_t = Mode::Plain)]
        mode: Mode,
        /// CA certificate used to verify the broker (tls mode)
        #[arg(short = 'C', long, default_value = "certs/ca.crt")]
        ca_file: PathBuf,
        /// Specifies the port
        #[arg(short, long, default_value_t = 1883)]
        port: u16,
        /// Login credentials for TLS connection
        #[arg(long, default_value = "server")]
        login: String,
        /// Password credentials for TLS connection
        #[arg(long, env = "MQTT_PASSWORD_SERVER")]
        password: Option<String>,
        /// Specifies the path to signed update file
        #[arg(long)]
        json_file: PathBuf,
        /// Specifies device id
        #[arg(short, long, default_value_t = 34)]
        device_id: u64,
    },
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
        Cli::command().debug_assert();
    }
}
