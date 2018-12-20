use crate::transport;
use failure::{err_msg, Error};
use log::{error, info, warn};
use rumqtt::{MqttClient, MqttOptions, QoS};

mod config;
mod janus;

#[derive(Debug)]
pub(crate) struct AgentBuilder {
    agent_id: transport::AgentId,
    backend_account_id: transport::AccountId,
}

impl AgentBuilder {
    fn new(agent_id: transport::AgentId, backend_account_id: transport::AccountId) -> Self {
        Self {
            agent_id,
            backend_account_id,
        }
    }

    fn start(
        self,
        config: &config::Mqtt,
    ) -> Result<(Agent, crossbeam_channel::Receiver<rumqtt::Notification>), Error> {
        let client_id = Self::mqtt_client_id(&self.agent_id);
        let options = Self::mqtt_options(&client_id, &config)?;
        let (tx, rx) = MqttClient::start(options)?;

        let group = transport::SharedGroup::new("loadbalancer", self.agent_id.account_id().clone());
        let mut agent = Agent::new(self.agent_id, self.backend_account_id, tx);
        agent.tx.subscribe(
            agent.backend_responses_subscription(&group),
            QoS::AtLeastOnce,
        )?;

        Ok((agent, rx))
    }

    fn mqtt_client_id(agent_id: &transport::AgentId) -> String {
        format!("v1.mqtt3/agents/{agent_id}", agent_id = agent_id)
    }

    fn mqtt_options(client_id: &str, config: &config::Mqtt) -> Result<MqttOptions, Error> {
        let uri = config.uri.parse::<http::Uri>()?;
        let host = uri.host().ok_or_else(|| err_msg("missing MQTT host"))?;
        let port = uri
            .port_part()
            .ok_or_else(|| err_msg("missing MQTT port"))?;

        Ok(MqttOptions::new(client_id, host, port.as_u16()).set_keep_alive(30))
    }
}

pub(crate) struct Agent {
    id: transport::AgentId,
    backend_account_id: transport::AccountId,
    tx: rumqtt::MqttClient,
}

impl Agent {
    fn new(
        id: transport::AgentId,
        backend_account_id: transport::AccountId,
        tx: MqttClient,
    ) -> Self {
        Self {
            id,
            backend_account_id,
            tx,
        }
    }

    fn publish<T>(&mut self, topic: &str, payload: &T) -> Result<(), Error>
    where
        T: serde::Serialize,
    {
        use crate::transport::compat::to_envelope;

        let message = to_envelope(payload, None)?;
        let bytes = serde_json::to_string(&message)?;

        self.tx
            .publish(topic, QoS::AtLeastOnce, bytes)
            .map_err(|_| err_msg(format!("Failed to publish an MQTT message: {:?}", message)))
    }

    fn backend_input_topic(&self, backend_agent_id: &transport::AgentId) -> String {
        format!(
            "agents/{backend_agent_id}/api/v1/in/{app_name}",
            backend_agent_id = backend_agent_id,
            app_name = &self.id.account_id()
        )
    }

    fn backend_responses_subscription(&self, group: &transport::SharedGroup) -> String {
        format!(
            "$share/{group}/apps/{backend_name}/api/v1/responses",
            group = group,
            backend_name = &self.backend_account_id,
        )
    }
}

pub(crate) fn run() {
    // Config
    let config = config::load().expect("Failed to load config");
    info!("App config: {:?}", config);

    // Agent
    let agent_id = transport::AgentId::new("a", config.id);
    let (mut tx, rx) = AgentBuilder::new(agent_id, config.backend_id.clone())
        .start(&config.mqtt)
        .expect("Failed to create an agent");

    // TODO: derive a backend agent id from a status message
    let backend_agent_id = transport::AgentId::new("a", config.backend_id.clone());

    // TODO: Replace with Real-Time Connection data
    use uuid::Uuid;
    let room_id = Uuid::new_v4();
    let rtc_id = Uuid::new_v4();

    // Creating a Janus Gateway session
    let req = janus::create_session_request(room_id, rtc_id).expect("Failed to build a request");
    tx.publish(&tx.backend_input_topic(&backend_agent_id), &req)
        .expect("Failed to publish a message");

    for message in rx {
        match message {
            rumqtt::client::Notification::Publish(ref message) => {
                let topic = &message.topic_name;
                let data = &message.payload.as_slice();

                let result = janus::handle_message(&mut tx, data);
                match result {
                    Err(err) => handle_error(topic, data, err),
                    Ok(_) => info!("Message has been processed"),
                }
            }
            _ => error!("An unsupported type of message = {:?}", message),
        }
    }
}

fn handle_error(topic: &str, data: &[u8], error: Error) {
    let message = std::str::from_utf8(data).unwrap_or("[non-utf8 characters]");
    warn!(
        "Processing of a message = {} from a topic = {} failed because of an error = {}.",
        message, topic, error
    );
}
