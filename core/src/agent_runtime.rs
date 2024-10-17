use crate::agent::*;
use crate::common_types::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub type AgentId = Uuid;
pub type TopicId = Uuid;
pub type SubscriptionId = Uuid;

pub enum ChatMessageEnvelope {
    SendMessageEnvelope(WrappedMessage),
    ResponseMessageEnvelope(WrappedMessage),
    PublishMessageEnvelope(WrappedMessage, String),
}

pub struct WrappedMessage {
    pub message: ChatMessage,
    pub sender: String,
    pub recepient: String,
    pub parent: Option<String>,
}
pub struct InterventionHanlder;
pub struct TraceProvider;

pub enum RunState {
    Running = 0,
    Cancelled = 1,
    UntilIdle = 2,
}
pub struct RunContext {
    pub run_state: RunState,
}

impl RunContext {
    pub fn run(self) {}
    pub fn stop(self) {}
    pub fn stop_when_idle(self) {}
    pub fn stop_when_cancel(self) {}
}

pub struct Subscription {
    pub id: TopicId,
    pub subscription_id: SubscriptionId,
}

impl Subscription {
    pub fn is_match(&self, topic_id: &TopicId) -> bool {
        todo!()
    }

    pub fn map_to_agent(&self, topic_id: &TopicId) -> AgentId {
        todo!()
    }
}
pub struct SubscriptionManager {
    pub subscriptions: HashSet<Subscription>,
    pub seen_topics: HashSet<TopicId>,
    pub subscribed_recipients: HashMap<TopicId, Vec<AgentId>>,
}

impl SubscriptionManager {
    pub async fn add_subscription(self, subscription: Subscription) {
    //     let agent_id = subscription.map_to_agent(&self.id);
    //     self.subscribed_recipients.insert(subscription);
    
    
    }

    pub async fn remove_subscription(self) {}

    pub fn is_not_sub(self) -> bool {
        todo!()
    }

    pub async fn get_subscribed_recipients(self, topic: TopicId) -> Vec<AgentId> {
        let mut res = self.subscribed_recipients.clone();
        if !self.seen_topics.contains(&topic) {
            res.remove(&topic);
        }

        res.into_iter()
            .map(|(_, v)| v.into_iter())
            .flatten()
            .collect::<HashSet<AgentId>>()
            .into_iter()
            .collect::<Vec<AgentId>>()
    }

    pub fn build_for_new_topic(&mut self, topic_id: TopicId) {
        self.seen_topics.insert(topic_id);

        for subscription in &self.subscriptions {
            if subscription.is_match(&topic_id) {
                let agent_id = subscription.map_to_agent(&topic_id);
                self.subscribed_recipients
                    .entry(topic_id)
                    .or_insert_with(Vec::new) // Create a new Vec if the topic doesn't exist
                    .push(agent_id);
            }
        }
    }
}

pub struct AgentRuntime {
    pub intervention_handlers: Option<Vec<InterventionHanlder>>,
    pub trace_provider: Option<TraceProvider>,
    pub message_queue: Vec<ChatMessageEnvelope>,
    pub tool_store: HashMap<String, Func>,
    // pub agent_factories: HashMap<String, Agent>,
    pub instantiated_agents: HashMap<String, Agent>,
    pub outstanding_tasks: i8,
    pub background_tasks: HashSet<String>,
    pub subscription_manager: SubscriptionManager,
    pub run_context: RunContext,
}

impl AgentRuntime {
    pub fn unprocessed_messages(self) -> Vec<ChatMessageEnvelope> {
        todo!()
    }

    pub fn known_tools(self) -> HashSet<String> {
        self.tool_store
            .keys()
            .map(String::from)
            .collect::<HashSet<String>>()
    }

    // pub fn known_agent_names(self) -> HashSet<String> {
    //     self.agent_factories
    //         .keys()
    //         .map(String::from)
    //         .collect::<HashSet<String>>()
    // }

    pub fn send_message(&mut self, message: ChatMessage, recepient: String, sender: String) {
        self.message_queue
            .push(ChatMessageEnvelope::SendMessageEnvelope(WrappedMessage {
                sender: sender,
                recepient: recepient,
                message: message,
                parent: None,
            }));
    }

    pub fn publish_message(
        &mut self,
        message: ChatMessage,
        topic_id: String,
        recepient: String,
        sender: String,
    ) {
        self.message_queue
            .push(ChatMessageEnvelope::PublishMessageEnvelope(
                WrappedMessage {
                    sender: sender,
                    recepient: recepient,
                    message: message,
                    parent: None,
                },
                topic_id,
            ));
    }

    pub fn save_state(self) {}
    pub fn load_state(self) {}
    pub fn process_send(self, message_envelope: ChatMessageEnvelope) {}
}
