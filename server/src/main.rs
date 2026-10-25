use std::thread;
use std::time::Duration;

use proto::SensorReading;
use rumqttc::{Client, Event, Incoming, MqttOptions, Publish, QoS};

fn main() -> anyhow::Result<()> {
    let mut mqttoptions = MqttOptions::new("server", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut connection) = Client::new(mqttoptions, 10);

    client.subscribe("sensor/+/telemetry", QoS::AtLeastOnce)?;

    for notification in connection.iter() {
        match notification {
            Ok(Event::Incoming(Incoming::Publish(packet))) => {
                match handle_message(&packet){
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

fn handle_message(packet: &Publish) -> anyhow::Result<SensorReading> {
    let reading = serde_json::from_slice::<SensorReading>(&packet.payload)?;
    
    Ok(reading)
}
