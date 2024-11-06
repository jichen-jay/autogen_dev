use crate::agent::chat_agent::Agent;
use crate::msg_types::*;
use crate::msg_types::{chat_msg_types::ChatMessage, AgentId, SubscriptionId, TopicId};
use crate::tool_types::Tool;
use chat_msg_types::TextMessage;
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::{self, Receiver, Sender};

pub enum ChatMessageEnvelope {
    SendMessageEnvelope(SendMessage),
    ResponseMessageEnvelope(ResponseMessage),
    PublishMessageEnvelope(PublishMessage),
}

impl ChatMessageEnvelope {
    fn enclose(msg: MessageWrapper) -> Self {
        match msg {
            MessageWrapper::PublishMessage(pm) => ChatMessageEnvelope::PublishMessageEnvelope(pm),
            MessageWrapper::SendMessage(sm) => ChatMessageEnvelope::SendMessageEnvelope(sm),
            MessageWrapper::ResponseMessage(rm) => ChatMessageEnvelope::ResponseMessageEnvelope(rm),
        }
    }
}

pub enum MessageWrapper {
    SendMessage(SendMessage),
    ResponseMessage(ResponseMessage),
    PublishMessage(PublishMessage),
}

pub struct SendMessage {
    pub message: ChatMessage,
    pub sender: Option<AgentId>,
    pub recipient: AgentId,
    pub parent: Option<AgentId>,
}

impl SendMessage {
    fn from(
        message: ChatMessage,
        sender: Option<AgentId>,
        recipient: AgentId,
        parent: Option<AgentId>,
    ) -> Self {
        SendMessage {
            message,
            sender,
            recipient,
            parent,
        }
    }

    fn suit_up(self) -> MessageWrapper {
        MessageWrapper::SendMessage(self)
    }
}

#[derive(Debug, Clone)]
pub struct ResponseMessage {
    pub message: ChatMessage,
    pub sender: AgentId,
    pub recipient: Option<AgentId>,
}

impl ResponseMessage {
    fn from(message: ChatMessage, sender: AgentId, recipient: Option<AgentId>) -> Self {
        ResponseMessage {
            message,
            sender,
            recipient,
        }
    }
    fn suit_up(self) -> MessageWrapper {
        MessageWrapper::ResponseMessage(self)
    }
}

#[derive(Clone)]
pub struct PublishMessage {
    pub message: ChatMessage,
    pub sender: Option<AgentId>,
    pub topic_id: TopicId,
}

impl PublishMessage {
    fn from(message: ChatMessage, sender: Option<AgentId>, topic_id: TopicId) -> Self {
        PublishMessage {
            message,
            sender,
            topic_id,
        }
    }
    fn suit_up(self) -> MessageWrapper {
        MessageWrapper::PublishMessage(self)
    }
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
        self.subscriptions.retain(|sub| sub.subscription_id != id);
    }

    pub async fn get_subscribed_recipients(&mut self, topic_id: &TopicId) -> Vec<AgentId> {
        match self.seen_topics.contains(topic_id) {
            false => {
                self.build_for_new_topic(topic_id);
                Vec::new()
            }
            true => self.subscribed_recipients.get(topic_id).unwrap().clone(),
        }
    }

    pub fn build_for_new_topic(&mut self, topic_id: &TopicId) {
        self.seen_topics.insert(topic_id.clone());

        for subscription in &self.subscriptions {
            if subscription.is_match(&topic_id) {
                let agent_id = subscription.map_to_agent(&topic_id);
                self.subscribed_recipients
                    .entry(topic_id.clone())
                    .or_insert_with(Vec::new) // Create a new Vec if the topic doesn't exist
                    .push(agent_id);
            }
        }
    }
}

pub struct Counter {
    count: i8,
}

impl Counter {
    pub fn increment(&mut self) {
        self.count += 1;
    }
    pub fn decrement(&mut self) {
        self.count -= 1;
    }
}

pub struct AgentRuntime {
    pub message_queue: Vec<ChatMessageEnvelope>,
    pub tool_store: HashMap<String, Tool>,
    pub intervention_handlers: HashMap<String, Handler>,
    pub instantiated_agents:
        HashMap<AgentId, (Sender<ChatMessageEnvelope>, Receiver<ChatMessageEnvelope>)>,
    pub outstanding_tasks: Counter,
    pub background_tasks: HashSet<String>,
    pub subscription_manager: SubscriptionManager,
}

pub struct Handler;

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

    pub fn known_agent_names(self) -> HashSet<AgentId> {
        self.instantiated_agents.keys().cloned().collect()
    }

    pub async fn send_message(
        &mut self,
        message: ChatMessage,
        recipient: AgentId,
        sender: Option<AgentId>,
    ) {
        if let Some((tx, _)) = self.instantiated_agents.get(&recipient) {
            let (response_tx, mut response_rx) = mpsc::channel::<ChatMessageEnvelope>(1);

            let msg = ChatMessageEnvelope::enclose(
                SendMessage::from(message, sender, recipient.clone(), None).suit_up(),
            );

            if let Err(_) = tx.send(msg).await {
                println!("Error sending message to agent {:?}", recipient);
            }

            if let Some(response) = response_rx.recv().await {
                self.message_queue.push(response);
            } else {
                println!("Error receiving response");
            }
        } else {
            println!("Error: Agent not found: {:?}", recipient);
        }
    }

    pub async fn publish_message(
        &mut self,
        message: ChatMessage,
        topic_id: TopicId,
        sender: Option<AgentId>,
    ) {
        let recipients = self
            .subscription_manager
            .get_subscribed_recipients(&topic_id)
            .await;

        for recipient in recipients {
            self.send_message(message.clone(), recipient, sender.clone())
                .await; // Send to each subscribed agent
        }
    }

    pub async fn register_factory(&mut self, agent: AgentId, sub: Subscription) {
        self.subscription_manager.add_subscription(sub).await;
    }

    pub async fn process_next(&mut self) {
        // This is the main message processing loop
/*         while let Some(me) = self.message_queue.pop() {
            match me {
                ChatMessageEnvelope::PublishMessageEnvelope(pme) => {
                    self.outstanding_tasks.increment();
                    let task = tokio::spawn(self.process_publish(pme));
                    self.background_tasks
                        .insert(format!("publish-task-{:?}", pme.topic_id));
                    // Handle completion or errors
                    if let Err(err) = task.await {
                        println!("Error in publish task: {:?}", err);
                    }
                    self.background_tasks
                        .remove(&format!("publish-task-{:?}", pme.topic_id));
                    self.outstanding_tasks.decrement();
                }
                ChatMessageEnvelope::SendMessageEnvelope(sme) => {
                    self.outstanding_tasks.increment();
                    let task = tokio::spawn(self.process_send(sme));
                    self.background_tasks
                        .insert(format!("send-task-{:?}", sme.recipient));
                    // Handle completion or errors
                    if let Err(err) = task.await {
                        println!("Error in send task: {:?}", err);
                    }
                    self.background_tasks
                        .remove(&format!("send-task-{:?}", sme.recipient));
                    self.outstanding_tasks.decrement();
                }
                ChatMessageEnvelope::ResponseMessageEnvelope(rme) => {
                    self.outstanding_tasks.increment();
                    let task = tokio::spawn(
                        self.process_response(ChatMessageEnvelope::ResponseMessageEnvelope(rme)),
                    );
                    self.background_tasks
                        .insert(format!("response-task-{:?}", rme.sender));
                    // Handle completion or errors
                    if let Err(err) = task.await {
                        println!("Error in response task: {:?}", err);
                    }
                    self.background_tasks
                        .remove(&format!("response-task-{:?}", rme.sender));
                    self.outstanding_tasks.decrement();
                }
            }
        } */

       todo!()
    }
    pub async fn add_subscription(&mut self, sub: Subscription) {
        self.subscription_manager.add_subscription(sub).await;
    }

    pub async fn remove_subscription(&mut self, sub_id: SubscriptionId) {
        self.subscription_manager.remove_subscription(sub_id).await;
    }

    pub fn save_state(self) {}
    pub fn load_state(self) {}

    pub async fn process_send(&mut self, message_envelope: SendMessage) {
        let recipient = message_envelope.recipient.clone();
        let mut recipient_agent = self.get_agent(recipient.clone());

        let msg = recipient_agent.on_message(message_envelope.message).await;

        let response: ResponseMessage =
            ResponseMessage::from(msg, recipient.clone(), Some(recipient));
        self.message_queue
            .push(ChatMessageEnvelope::ResponseMessageEnvelope(
                ResponseMessage {
                    sender: response.sender,
                    recipient: response.recipient,
                    message: response.message,
                },
            ));

        self.outstanding_tasks.decrement();
    }

    pub async fn process_publish(&mut self, message_envelope: ChatMessageEnvelope) {
/*         if let ChatMessageEnvelope::PublishMessageEnvelope(pm) = message_envelope {
            let recipients: Vec<AgentId> = self
                .subscription_manager
                .get_subscribed_recipients(&pm.topic_id)
                .await;

            for agent_id in recipients {
                let mut recipient_agent = self.get_agent(agent_id);

                let msg = recipient_agent.on_message(pm.message.clone()).await;

                let response: ResponseMessage =
                    ResponseMessage::from(msg, agent_id.clone(), Some(agent_id));

                self.message_queue
                    .push(ChatMessageEnvelope::ResponseMessageEnvelope(
                        ResponseMessage {
                            sender: response.sender,
                            recipient: response.recipient,
                            message: response.message,
                        },
                    ));

                self.outstanding_tasks.decrement();
            }
        } else {
            panic!("Unexpected message envelope type");
        } */
       todo!()
    }

    pub async fn process_response(&mut self, message_evelope: ChatMessageEnvelope) {
        todo!()
    }

    pub fn get_agent(&self, agent_id: AgentId) -> Agent {
        // self.instantiated_agents.get(&agent_id).unwrap().clone()

        todo!()
    }
}

#[tokio::main]
async fn main() {
    let mut runtime = AgentRuntime {
        message_queue: vec![],
        tool_store: HashMap::<String, Tool>::new(),
        intervention_handlers: HashMap::new(),
        instantiated_agents: HashMap::<
            AgentId,
            (Sender<ChatMessageEnvelope>, Receiver<ChatMessageEnvelope>),
        >::new(),
        outstanding_tasks: Counter { count: 0 },
        background_tasks: HashSet::<String>::new(),
        subscription_manager: SubscriptionManager {
            subscriptions: HashSet::<Subscription>::new(),
            seen_topics: HashSet::<TopicId>::new(),
            subscribed_recipients: HashMap::<TopicId, Vec<AgentId>>::new(),
        },
    };

    let topic_id = TopicId::new(Some("general_topic"));

    let chat_agent_id = AgentId::new(Some("BaseAgent"));
    let user_agent_id = AgentId::new(Some("UserAgent"));

    let chat_subscriptions = Subscription {
        id: topic_id.clone(),
        subscription_id: new_subscription_id(),
    };

    runtime
        .register_factory(chat_agent_id.clone(), chat_subscriptions)
        .await;

    let message = ChatMessage::TextMessage(TextMessage {
        content: TextContent {
            text: "Hello, BaseAgent!".to_string(),
        },
        source: AgentId::new(Some("place")),
    });

    let message2 = ChatMessage::TextMessage(TextMessage {
        content: TextContent {
            text: "Hello, subscribers!".to_string(),
        },
        source: AgentId::new(Some("place")),
    });

    runtime.send_message(message, chat_agent_id, None).await;

    runtime.publish_message(message2, topic_id, None).await;

    loop {
        runtime.process_next().await;
    }
}
