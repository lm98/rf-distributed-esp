use std::sync::mpsc::{Sender, Receiver, self};
use esp_idf_svc::mqtt::client::{EspMqttClient, MqttClientConfiguration, EspMqttMessage, Details::Complete};
use esp_idf_svc::mqtt::client::QoS;
use rf_distributed::message::Message;
use rf_distributed::network::{NetworkResult, NetworkUpdate, sync::Network};
use esp_idf_svc::mqtt::client::Event;

pub struct EspMqttNetwork<'a> {
    client: EspMqttClient<'a>,
    rx: Receiver<NetworkUpdate>,
}

impl<'a> EspMqttNetwork<'a> {
    pub fn new(broker_url: &str, config: &MqttClientConfiguration, topics: Vec<i32>) -> Self {
        let (tx, rx): (Sender<NetworkUpdate>, Receiver<NetworkUpdate>) = mpsc::channel();
        let mut client = EspMqttClient::new(
            broker_url, 
            config, 
            move |message_event| match message_event {
                Ok(Event::Received(msg)) => {
                    let update = EspMqttNetwork::process_message(msg);
                    tx.send(update).unwrap();
                },
                _ => { 
                    tx.send(NetworkUpdate::None).unwrap();
                 },
            }).unwrap();

        EspMqttNetwork::subscribe_to_topics(&mut client, topics).unwrap();

        Self {
            client, 
            rx,
        }
    }

    pub fn subscribe_to_topics(client: &mut EspMqttClient, topics: Vec<i32>) -> NetworkResult<()> {
        for topic in topics {
            client.subscribe(&format!("hello-rufi/{topic}/subscriptions"), QoS::AtLeastOnce)?;
        }
        Ok(())
    }

    fn process_message(message: &EspMqttMessage) -> NetworkUpdate {
        match message.details() {
            Complete => {
                let message_data: &[u8] = message.data();
                if let Ok(mex) = serde_json::from_slice::<Message>(message_data) {
                    NetworkUpdate::Update { msg: serde_json::to_string(&mex).unwrap() }
                } else {
                    NetworkUpdate::None
                }
            },
            _ => NetworkUpdate::None,
        }
    }
}

impl<'a> Network for EspMqttNetwork<'a> {
    fn send(&mut self, source: i32, msg: String) -> NetworkResult<()> {
        self.client.publish(
            format!("hello-rufi/{source}/subscriptions").as_str(), 
            QoS::AtLeastOnce, 
            false, 
            msg.as_bytes()
        )?;
        Ok(())
    }

    fn receive(&mut self) -> NetworkResult<NetworkUpdate> {
        Ok(self.rx.recv()?)
    }
}