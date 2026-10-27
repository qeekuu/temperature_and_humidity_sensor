#!/usr/bin/env bash
# Generates a local CA and a TLS certificate for the Mosquitto broker.
# All files are written next to this script (certs/).
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")"

DAYS=365

if [[ -e ca.crt ]]; then
    echo "certs/ca.crt already exists, remove generated files first to regenerate" >&2
    exit 1
fi

# 1. Certificate authority (self-signed). Clients trust only this certificate.
openssl req -new -x509 -days "$DAYS" -noenc \
    -keyout ca.key -out ca.crt \
    -subj "/CN=IoT Demo CA" \
    -addext "basicConstraints=critical,CA:TRUE" \
    -addext "keyUsage=critical,keyCertSign,cRLSign"

# 2. Broker private key (unencrypted, Mosquitto cannot prompt for a passphrase).
openssl genrsa -out server.key 2048

# 3. Certificate signing request for the broker.
openssl req -new -key server.key -out server.csr -subj "/CN=localhost"

# 4. Broker certificate signed by the CA. rustls only checks the SAN, not the CN.
openssl x509 -req -in server.csr -CA ca.crt -CAkey ca.key -CAcreateserial \
    -out server.crt -days "$DAYS" \
    -extfile <(printf '%s\n' \
        "basicConstraints=CA:FALSE" \
        "keyUsage=critical,digitalSignature,keyEncipherment" \
        "extendedKeyUsage=serverAuth" \
        "subjectAltName=DNS:localhost,IP:127.0.0.1")

# The CA key never leaves this machine. The broker key must be readable by the
# mosquitto user inside the container (UID 1883), hence 644 (demo simplification).
chmod 600 ca.key
chmod 644 server.key

echo "Generated: ca.crt ca.key server.crt server.key"
