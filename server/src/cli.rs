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
