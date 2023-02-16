use std::{net::IpAddr, process, time::Duration};

use mqtt::{Client, Message, Receiver};
use paho_mqtt as mqtt;
use serde::Deserialize;

const INPUT_TOPIC: &str = "+/+/lightTriggers";

pub struct TetherAgent {
    client: Client,
    receiver: Receiver<Option<Message>>,
}

#[derive(Deserialize, Debug)]
struct LightTriggerMessage {
    id: usize,
}

impl TetherAgent {
    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }

    pub fn new(tether_host: IpAddr) -> Self {
        let broker_uri = format!("tcp://{}:1883", tether_host);

        let create_opts = mqtt::CreateOptionsBuilder::new()
            .server_uri(broker_uri)
            .client_id("")
            .finalize();

        // Create the client connection
        let client = mqtt::Client::new(create_opts).unwrap();

        // Initialize the consumer before connecting
        let receiver = client.start_consuming();

        TetherAgent { client, receiver }
    }

    pub fn connect(&mut self) {
        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .user_name("tether")
            .password("sp_ceB0ss!")
            .keep_alive_interval(Duration::from_secs(30))
            .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
            .clean_session(true)
            .finalize();

        // Make the connection to the broker
        println!("Connecting to the MQTT server...");
        match self.client.connect(conn_opts) {
            Ok(res) => {
                println!("Connected OK: {:?}", res);
                match self.client.subscribe(INPUT_TOPIC, 2) {
                    Ok(res) => {
                        println!("Subscribe OK: {:?}", res);
                    }
                    Err(e) => {
                        println!("Error subscribing: {:?}", e);
                    }
                }
            }
            Err(e) => {
                println!("Error connecting to the broker: {:?}", e);
                process::exit(1);
            }
        }
    }

    pub fn check_messages(&self) -> Option<usize> {
        if let Some(m) = self.receiver.try_iter().find_map(|m| m) {
            let payload = m.payload().to_vec();
            let light_message: Result<LightTriggerMessage, rmp_serde::decode::Error> =
                rmp_serde::from_slice(&payload);
            match light_message {
                Ok(parsed) => {
                    println!("Parsed LightTriggerMessage: {:?}", parsed);
                    Some(parsed.id)
                }
                Err(e) => {
                    println!("Error parsing LightTriggerMessage: {:?}", e);
                    None
                }
            }
        } else {
            None
        }
    }
}
