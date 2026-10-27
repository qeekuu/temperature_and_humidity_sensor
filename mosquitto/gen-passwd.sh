#!/usr/bin/env bash
# Creates the broker password file (mosquitto/config/passwd) with random
# passwords and writes them to mosquitto/credentials.env for the clients.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")"

USERS=(sensor-34 server)

if [[ -e config/passwd ]]; then
    echo "mosquitto/config/passwd already exists, remove it first to regenerate" >&2
    exit 1
fi

: > config/passwd
: > credentials.env
chmod 600 config/passwd credentials.env

for user in "${USERS[@]}"; do
    password="$(openssl rand -hex 16)"
    # Run as the current user so the file is not owned by root.
    docker run --rm --user "$(id -u):$(id -g)" \
        -v "$PWD/config:/work" eclipse-mosquitto:2 \
        mosquitto_passwd -b /work/passwd "$user" "$password"
    var="MQTT_PASSWORD_$(echo "$user" | tr 'a-z-' 'A-Z_')"
    echo "export $var=$password" >> credentials.env
done

echo "Created mosquitto/config/passwd and mosquitto/credentials.env"
