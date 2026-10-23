# Temperature and Humidity Sensor — IoT Security

University project for the *Selected Aspects of Cybersecurity* course (topic 7: Cybersecurity in IoT).

A Rust simulation of a temperature and humidity sensor that sends readings to a server over MQTT.
The project shows how to secure an IoT device's communication and firmware updates, and what happens
when those protections are missing.

## Scope

- **Communication modes:**
  - `plain` — unencrypted MQTT (baseline),
  - `tls` — MQTT over TLS,
  - `e2e` — application-layer encryption (ChaCha20-Poly1305) with a counter for replay protection.
- **Secure firmware update** — Ed25519-signed manifest, SHA-256 hash and version checks, A/B slots.
- **Attack demonstrations** — eavesdropping (Wireshark), replay, message tampering, unauthorized client,
  forged firmware.
- **Threat analysis** — STRIDE, OWASP IoT Top 10, ETSI EN 303 645.

## Structure

- `proto` — library: message format, encryption, anti-replay, firmware signatures
- `device` — simulated sensor
- `server` — receives readings, signs firmware, simulates attacks

## License

MIT — see [LICENSE.md](LICENSE.md).
