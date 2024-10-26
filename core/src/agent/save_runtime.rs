use crate::agent::chat_agent::Agent;
use crate::msg_types::*;
use crate::msg_types::{chat_msg_types::ChatMessage, AgentId, SubscriptionId, TopicId};
use crate::tool_types::{AgentType, Tool};
use std::collections::{HashMap, HashSet};

pub enum ChatMessageEnvelope {
    SendMessageEnvelope(SendMessage),
    ResponseMessageEnvelope(ResponseMessage),
    PublishMessageEnvelope(PublishMessage),
}

pub struct SendMessage {
    pub message: ChatMessage,
    pub sender: Option<AgentId>,
    pub recepient: AgentId,
    pub parent: Option<AgentId>,
}
pub struct ResponseMessage {
    pub message: ChatMessage,
    pub sender: AgentId,
    pub recepient: Option<AgentId>,
}

#[derive(Clone)]
pub struct PublishMessage {
    pub message: ChatMessage,
    pub sender: Option<AgentId>,
    pub topic_id: TopicId,
}
pub struct InterventionHanlder;

impl InterventionHanlder {
    pub fn on_send(self, message: ChatMessage, sender: Option<AgentId>, recepient: AgentId) {
        todo!()
    }
    pub fn on_publish(self, message: ChatMessage, sender: Option<AgentId>) {
        todo!()
    }
    pub fn on_response(self, message: ChatMessage, sender: AgentId, recepient: AgentId) {
        todo!()
    }
}

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

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct Subscription {
    pub id: TopicId,
    pub subscription_id: SubscriptionId,
}

impl Subscription {
    pub fn is_match(&self, topic_id: &TopicId) -> bool {
        &self.id == topic_id
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
    pub async fn add_subscription(&mut self, subscription: Subscription) {
        self.subscriptions.insert(subscription);
    }

    pub async fn remove_subscription(&mut self, id: SubscriptionId) {
        self.subscriptions.retain(|sub| sub.id != id);
    }

    pub async fn get_subscribed_recipients(&mut self, topic_id: TopicId) -> Vec<AgentId> {
        if !self.seen_topics.contains(&topic_id) {
            self.build_for_new_topic(topic_id);
        }

        self.subscribed_recipients.get(&topic_id).unwrap().clone()
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
    pub instantiated_agents: HashMap<AgentId, Agent>,
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

    pub fn send_message(
        &mut self,
        message: ChatMessage,
        recepient: AgentId,
        sender: Option<AgentId>,
    ) {
        self.message_queue
            .push(ChatMessageEnvelope::SendMessageEnvelope(SendMessage {
                sender: sender,
                recepient: recepient,
                message: message,
                parent: None,
            }));
    }

    pub fn publish_message(
        &mut self,
        message: ChatMessage,
        topic_id: TopicId,
        sender: Option<AgentId>,
    ) {
        self.message_queue
            .push(ChatMessageEnvelope::PublishMessageEnvelope(
                PublishMessage {
                    sender: sender,
                    message: message,
                    topic_id: topic_id,
                },
            ));
        todo!()
    }

    pub async fn register(
        &self,
        typ: &str,
        agent_factory: HashMap<String, Tool>,
        subscriptions: Vec<String>,
    ) {
        todo!()
    }

    pub async fn register_factory(&self, typ: AgentType, agent_factory: HashMap<String, Tool>) {
        todo!()
    }

    pub async fn add_subscription(&self, subscription: SubscriptionId) {
        todo!()
    }

    pub async fn remove_subscription(&mut self, id: SubscriptionId) {
        todo!()
    }
    pub fn save_state(self) {}
    pub fn load_state(self) {}
    pub async fn process_send(&mut self, message_envelope: SendMessage) {
        let recepient = message_envelope.recepient;
        let recepient_agent = self.get_agent(recepient).clone();
        let message_context = ChatMessageContext {
            sender: message_envelope.sender,
            topic_id: None,
            is_rpc: true,
        };

        let response: ResponseMessage = recepient_agent
            .on_messages(message_envelope.message, message_context)
            .await;

        self.message_queue
            .push(ChatMessageEnvelope::ResponseMessageEnvelope(
                ResponseMessage {
                    sender: response.sender,
                    recepient: response.recepient,
                    message: response.message,
                },
            ));

        self.outstanding_tasks -= 1;
    }

    pub async fn process_publish(&mut self, message_envelope: PublishMessage) {
        let recepients: Vec<AgentId> = self
            .subscription_manager
            .get_subscribed_recipients(message_envelope.topic_id)
            .await;

        for agent_id in recepients {
            let message_context = ChatMessageContext {
                sender: message_envelope.sender.clone(),
                topic_id: None,
                is_rpc: true,
            };
            let recepient_agent = self.get_agent(agent_id).clone();

            let response: ResponseMessage = recepient_agent
                .on_messages(message_envelope.message.clone(), message_context)
                .await;

            self.message_queue
                .push(ChatMessageEnvelope::ResponseMessageEnvelope(
                    ResponseMessage {
                        sender: response.sender,
                        recepient: response.recepient,
                        message: response.message,
                    },
                ));
            self.outstanding_tasks -= 1;
        }
    }

    pub async fn process_response(&mut self, message_evelope: ChatMessageEnvelope) {
        todo!()
    }

    pub async fn process_next(&mut self) {
        if self.message_queue.len() == 0 {
            return;
        }

        while let Some(me) = self.message_queue.pop() {
            todo!()
            // match me {
            //     ChatMessageEnvelope::PublishMessageEnvelope(ref pme) => {
            //         if let Some(handlers) = &self.intervention_handlers {
            //             for handler in handlers {
            //                 let _ = handler.on_publish(pme.message, pme.sender);
            //             }
            //         }
            //     }
            //     ChatMessageEnvelope::SendMessageEnvelope(sme) => {
            //         if let Some(handlers) = self.intervention_handlers {
            //             for handler in handlers {
            //                 let _ = handler.on_send(sme.message, sme.sender, sme.recepient);
            //             }
            //         }
            //     }
            //     ChatMessageEnvelope::ResponseMessageEnvelope(rme) => {
            //         if let Some(handlers) = self.intervention_handlers {
            //             for handler in handlers {
            //                 let _ = handler.on_response(rme.message, rme.sender, rme.recepient.unwrap());
            //             }
            //         }
            //     }
            // }
        }
    }
    pub fn get_agent(&self, agent_id: AgentId) -> Agent {
        self.instantiated_agents.get(&agent_id).unwrap().clone()
    }
}
