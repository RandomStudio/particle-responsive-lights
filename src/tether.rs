use std::{net::IpAddr, process, time::Duration};

use log::{debug, error, info};
use mqtt::{Client, Message, Receiver};
use paho_mqtt as mqtt;
use serde::Deserialize;

const INPUT_TOPIC: &str = "+/+/lightTriggers";

pub struct TetherAgent {
    client: Client,
    receiver: Receiver<Option<Message>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LightTriggerMessage {
    pub id: usize,
    pub target_brightness: f32,
    #[serde(default)]
    pub attack_duration: usize,
    #[serde(default)]
    pub release_duration: usize,
    #[serde(default)]
    pub final_brightness: f32,
}

impl TetherAgent {
    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }

    pub fn new(tether_host: IpAddr) -> Self {
        let broker_uri = format!("tcp://{tether_host}:1883");

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
        info!("Connecting to the MQTT server...");
        match self.client.connect(conn_opts) {
            Ok(res) => {
                info!("Connected OK: {res:?}");
                match self.client.subscribe(INPUT_TOPIC, 2) {
                    Ok(res) => {
                        debug!("Subscribe OK: {res:?}");
                    }
                    Err(e) => {
                        error!("Error subscribing: {e:?}");
                    }
                }
            }
            Err(e) => {
                error!("Error connecting to the broker: {e:?}");
                process::exit(1);
            }
        }
    }

    pub fn check_messages(&self) -> Option<LightTriggerMessage> {
        if let Some(m) = self.receiver.try_iter().find_map(|m| m) {
            let payload = m.payload().to_vec();
            let light_message: Result<LightTriggerMessage, rmp_serde::decode::Error> =
                rmp_serde::from_slice(&payload);
            match light_message {
                Ok(parsed) => {
                    info!("Parsed LightTriggerMessage: {parsed:?}");
                    Some(parsed)
                }
                Err(e) => {
                    error!("Error parsing LightTriggerMessage: {e:?}");
                    None
                }
            }
        } else {
            None
        }
    }
}
