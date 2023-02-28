use std::{net::IpAddr, process, time::Duration};

use log::{debug, error, info};
use mqtt::{Client, Message, Receiver};
use paho_mqtt as mqtt;
use serde::Deserialize;

const INPUT_TOPICS: &[&str] = &["+/+/lightTriggers", "+/+/lightReset"];
const INPUT_QOS: &[i32; INPUT_TOPICS.len()] = &[2, 2];

pub struct TetherAgent {
    client: Client,
    receiver: Receiver<Option<Message>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LightTriggerMessage {
    pub id: usize,
    pub target_brightness: f32,
    pub attack_duration: Option<usize>,
    pub release_duration: Option<usize>,
    pub final_brightness: Option<f32>,
    pub transmission_range: Option<f32>,
    pub transmission_delay: Option<i64>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LightResetMessage {
    #[serde(default)]
    pub target_brightness: Option<f32>,
    #[serde(default)]
    pub fade_duration: Option<usize>,
}

pub enum LightMessages {
    Trigger(LightTriggerMessage),
    Reset(LightResetMessage),
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
                match self.client.subscribe_many(INPUT_TOPICS, INPUT_QOS) {
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

    pub fn check_messages(&self) -> Option<LightMessages> {
        if let Some(m) = self.receiver.try_iter().find_map(|m| m) {
            let payload = m.payload().to_vec();

            let plug_name = parse_plug_name(m.topic());

            match plug_name {
                "lightTriggers" => {
                    let light_message: Result<LightTriggerMessage, rmp_serde::decode::Error> =
                        rmp_serde::from_slice(&payload);

                    match light_message {
                        Ok(parsed) => {
                            info!("Parsed LightTriggerMessage: {parsed:?}");
                            if parsed.target_brightness.is_nan() {
                                panic!("target_brightness should be a valid number");
                            }
                            Some(LightMessages::Trigger(parsed))
                        }
                        Err(e) => {
                            error!("Failed to parse Light Reset message: {}", e);
                            None
                        }
                    }
                }
                "lightReset" => {
                    let light_message: Result<LightResetMessage, rmp_serde::decode::Error> =
                        rmp_serde::from_slice(&payload);

                    match light_message {
                        Ok(parsed) => {
                            info!("Parsed LightResetMessage: {parsed:?}");
                            Some(LightMessages::Reset(parsed))
                        }
                        Err(e) => {
                            error!("Failed to parse Light Reset message: {}", e);
                            None
                        }
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }
}

fn parse_plug_name(topic: &str) -> &str {
    let parts: Vec<&str> = topic.split('/').collect();
    parts[2]
}
