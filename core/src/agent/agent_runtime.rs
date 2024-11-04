use crate::agent::chat_agent::Agent;
use crate::msg_types::*;
use crate::msg_types::{chat_msg_types::ChatMessage, AgentId, SubscriptionId, TopicId};
use crate::tool_types::{AgentType, Tool};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{self, Receiver, Sender};

pub enum ChatMessageEnvelope {
    SendMessageEnvelope(SendMessage),
    ResponseMessageEnvelope(ResponseMessage),
    PublishMessageEnvelope(PublishMessage),
}

pub struct SendMessage {
    pub message: ChatMessage,
    pub sender: Option<AgentId>,
    pub recipient: AgentId,
    pub parent: Option<AgentId>,
}

#[derive(Debug, Clone)]
pub struct ResponseMessage {
    pub message: ChatMessage,
    pub sender: AgentId,
    pub recipient: Option<AgentId>,
}

#[derive(Clone)]
pub struct PublishMessage {
    pub message: ChatMessage,
    pub sender: Option<AgentId>,
    pub topic_id: TopicId,
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
    pub instantiated_agents: HashMap<AgentId, (Sender<AgentMessage>, Receiver<AgentMessage>)>,
    pub outstanding_tasks: Counter,
    pub background_tasks: HashSet<String>,
    pub subscription_manager: SubscriptionManager,
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

    pub fn known_agent_names(self) -> HashSet<AgentId> {
        self.instantiated_agents.keys().cloned().collect()
    }

    pub async  fn send_message(
        &mut self,
        message: ChatMessage,
        recipient: AgentId,
        sender: Option<AgentId>,
    ) {
        if let Some((tx, _)) = self.instantiated_agents.get(&recipient) {
            let (response_tx, mut response_rx) = mpsc::channel(1);

            let msg = AgentMessage::Send {
                sender,
                recipient: recipient.clone(),
                message,
                response_tx,
            };

            if let Err(_) = tx.send(msg).await {
                println!("Error sending message to agent {:?}", recipient);
            }

            tokio::spawn(async move {
                //should we save the reponse to the message_queue?
                if let Some(Ok(response)) = response_rx.recv().await {
                    println!("Received Response: {:?}", response);

                    self.message_queue.push(response.clone());
                } else {
                    println!("Error receiving response");
                }
            });
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
            self.send_message(message.clone(), recipient, sender.clone()).await; // Send to each subscribed agent
        }
    }


    pub async fn register_factory(&mut self, agent: AgentId, tool_set: Vec<Tool>) {
        // let agent_id = AgentId::new(typ.to_string(), "agent_id");
        // typ: AgentType,

        // let caps = self.get_agent(agent_id).
        // // Convert the tools into a HashMap<String, Tool>
        // let tools: HashMap<String, Tool> =
        //     tool_set.into_iter().map(|t| (t.name.clone(), t)).collect();

        // let agent = Agent::new(agent_id.clone(), tools);

        // // Create a communication channel for the agent
        // let (tx, rx) = mpsc::channel(32);

        // // Store the agent's communication channel
        // self.instantiated_agents.insert(agent_id, (tx, rx));

        // ... [Logic to register subscriptions] ...
        todo!()
    }

    pub async fn process_next(&mut self) {
        // This is the main message processing loop
        while let Some(me) = self.message_queue.pop() {
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
        }
    }
    pub async fn add_subscription(&mut self, subscription: Subscription) {

        self.subscription_manager
            .add_subscription(subscription)
            .await;
    }

    pub async fn remove_subscription(&mut self, id: SubscriptionId) {
        self.subscription_manager.remove_subscription(id).await;
    }

    pub fn save_state(self) {}
    pub fn load_state(self) {}

    pub async fn process_send(&mut self, message_envelope: SendMessage) {
        let recipient = message_envelope.recipient;
        let recipient_agent = self.get_agent(recipient).clone();
        let message_context = ChatMessageContext {
            sender: message_envelope.sender,
            topic_id: None,
            is_rpc: true,
        };

        let response: ResponseMessage = recipient_agent
            .on_messages(message_envelope.message, message_context)
            .await;

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

    pub async fn process_publish(&mut self, message_envelope: PublishMessage) {
        let recipients: Vec<AgentId> = self
            .subscription_manager
            .get_subscribed_recipients(message_envelope.topic_id)
            .await;

        for agent_id in recipients {
            let message_context = ChatMessageContext {
                sender: message_envelope.sender.clone(),
                topic_id: None,
                is_rpc: true,
            };
            let recipient_agent = self.get_agent(agent_id).clone();

            let response: ResponseMessage = recipient_agent
                .on_messages(message_envelope.message.clone(), message_context)
                .await;

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
    }

    pub async fn process_response(&mut self, message_evelope: ChatMessageEnvelope) {
        todo!()
    }


    pub fn get_agent(&self, agent_id: AgentId) -> Agent {
        self.instantiated_agents.get(&agent_id).unwrap().clone()
    }
}

#[tokio::main]
async fn main() {
    let mut runtime = AgentRuntime {
        message_queue: vec![],
        tool_store: HashMap::<String, Tool>::new(), // Global tool store (if you need one)
        instantiated_agents:
            HashMap::<AgentId, (Sender<AgentMessage>, Receiver<AgentMessage>)>::new(),
        outstanding_tasks: Counter { count: 0 },
        background_tasks: HashSet::<String>::new(),
        subscription_manager: SubscriptionManager {
            subscriptions: HashSet::<Subscription>::new(),
            seen_topics: HashSet::<TopicId>::new(),
            subscribed_recipients: HashMap::<TopicId, Vec<AgentId>>::new(),
        },
    };

    let chat_tools = HashMap::new(); //  Tools associated with the BaseAgent
    let chat_subscriptions = vec![new_topic_id("general_topic")];
    runtime
        .register("BaseAgent", chat_tools, chat_subscriptions)
        .await;

    let message = ChatMessage::Text {
        content: "Hello, BaseAgent!".to_string(),
    };
    let chat_agent_id = AgentId::new("BaseAgent", "agent_BaseAgent".to_string());
    runtime.send_message(message, chat_agent_id, None);

    let topic = TopicId::new("general_topic");
    let message2 = ChatMessage::Code {
        content: "print('Hello, subscribers!')".to_string(),
    };
    runtime.publish_message(message2, topic, None);

    loop {
        runtime.process_next().await;
    }
}

#[derive(Debug)]
enum AgentMessage {
    Send {
        sender: Option<AgentId>,
        recipient: AgentId,
        message: ChatMessage, // Boxed for dynamic dispatch
        response_tx: Sender<Result<ResponseMessage, String>>, // For sending back responses
    },
    Publish {
        sender: Option<AgentId>,
        topic: TopicId,
        message: ChatMessage,
    },
}
