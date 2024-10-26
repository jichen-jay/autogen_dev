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
    pub instantiated_agents: HashMap<AgentId, Agent>,
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
        todo!()
    }

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
    }
    pub async fn register(
        &mut self,
        typ: &str,
        agent_factory: HashMap<String, Tool>,
        subscriptions: Vec<String>,
    ) {
        // if self.instantiated_agents.contains_key(typ) {
        //     println!("Error: Agent type {} already registered", typ);
        //     return;
        // }

        let agent_id = new_agent_id();
        let agent = Agent {
            id: agent_id,
            meta_data: "some data".to_string(),
        };

        self.instantiated_agents.insert(agent_id, agent);
    }

    pub async fn register_factory(&self, typ: AgentType, agent_factory: HashMap<String, Tool>) {
        todo!()
    }

    pub async fn add_subscription(&mut self, subscription: SubscriptionId) {
        let new_subscription = Subscription {
            id: new_topic_id(),
            subscription_id: subscription,
        };

        self.subscription_manager
            .add_subscription(new_subscription)
            .await;
    }

    pub async fn remove_subscription(&mut self, id: SubscriptionId) {
        self.subscription_manager.remove_subscription(id).await;
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

        self.outstanding_tasks.decrement();
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
            self.outstanding_tasks.decrement();
        }
    }

    pub async fn process_response(&mut self, message_evelope: ChatMessageEnvelope) {
        todo!()
    }

    pub async fn process_next(&mut self) {
        if self.message_queue.len() == 0 {
            return;
        }

        // while let Some(me) = self.message_queue.pop() {
        //     self.outstanding_tasks.increment();
        //     match me {
        //         ChatMessageEnvelope::PublishMessageEnvelope(ref pme) => {
        //             let task = tokio::spawn(self.process_publish(pme.clone()));
        //             self.background_tasks
        //                 .insert(format!("publish-task-{:?}", pme.topic_id));
        //             task.await.unwrap();
        //         }
        //         ChatMessageEnvelope::SendMessageEnvelope(sme) => {
        //             let task = tokio::spawn(self.process_send(sme));
        //             self.background_tasks
        //                 .insert(format!("send-task-{:?}", sme.recepient));
        //             task.await.unwrap();
        //             self.background_tasks
        //                 .remove(&format!("send-task-{:?}", sme.recepient));
        //         }
        //         ChatMessageEnvelope::ResponseMessageEnvelope(rme) => {
        //             let task = tokio::spawn(
        //                 self.process_response(ChatMessageEnvelope::ResponseMessageEnvelope(rme)),
        //             );
        //             self.background_tasks
        //                 .insert(format!("response-task-{:?}", rme.sender));
        //             task.await.unwrap();
        //             self.background_tasks
        //                 .remove(&format!("response-task-{:?}", rme.sender));
        //         }
        //     }
        // }
        todo!()
    }
    pub fn get_agent(&self, agent_id: AgentId) -> Agent {
        self.instantiated_agents.get(&agent_id).unwrap().clone()
    }
}

//have made updates in previous code, update code below to make logic work
// do you need the AgentMessage struct to pass message between agents?
#[tokio::main]
async fn main() {
    let (runtime_tx, mut runtime_rx) = mpsc::channel(32);

    // let (tx, rx) = mpsc::channel::<AgentMessage>(32);
    let runtime = AgentRuntime {
        message_queue: vec![],
        tool_store: HashMap::<String, Tool>::new(),
        instantiated_agents: HashMap::<AgentId, Agent>::new(),
        outstanding_tasks: Counter { count: 0 },
        background_tasks: HashSet::<String>::new(),
        subscription_manager: SubscriptionManager {
            subscriptions: HashSet::<Subscription>::new(),
            seen_topics: HashSet::<TopicId>::new(),
            subscribed_recipients: HashMap::<TopicId, Vec<AgentId>>::new(),
        },
    };

    tokio::spawn(async move {
        while let Some(message) = runtime_rx.recv().await {
            match message {
                AgentMessage::Send {
                    sender,
                    recipient,
                    message,
                    response_tx,
                } => {
                    if let Err(_) = runtime_tx
                        .send(AgentMessage::Send {
                            sender,
                            recipient,
                            message,
                            response_tx,
                        })
                        .await
                    {
                        // Handle send error, e.g., agent might have stopped
                        if let Err(_) = response_tx.send(Err("Agent not found".to_string())).await {
                            // ...
                        }
                    }
                }
                AgentMessage::Publish {
                    sender,
                    topic,
                    message,
                } => {
                    // Broadcast the message to all subscribed agents (not implemented here)
                    println!("Publish not implemented:  {:?}", message);
                }
            }
        }
    });

    // let runtime_task = tokio::spawn(runtime.run());

    // Agent defined already:
    //     #[derive(Clone)]
    // pub struct Agent {
    //     pub id: AgentId,
    //     pub meta_data: String,
    // }

    // impl Agent {
    //     pub async fn on_messages(
    //         self,
    //         message: ChatMessage,
    //         ctx: ChatMessageContext,
    //     ) -> ResponseMessage {
    //         // ResponseMessage {
    //         //     message: ChatMessage::Text {
    //         //         content: format!("Response from Agent {}", self.id.0),
    //         //     },
    //         //     sender: self.id.clone(),
    //         //     recepient: None,
    //         // }
    //         todo!()
    //     }

    //     pub fn save_state(self) {}
    //     pub fn load_state(self) {}
    // }
    //need to properly construct Agents
    let (tx, rx) = runtime.spawn_agent(Agent::new(AgentId::new("ChatAgent", "agent_id_1")));
    let (tx, rx) = runtime.spawn_agent(Agent::new(AgentId::new("ChatAgent", "agent_id_2")));

    // let (resp_tx, mut resp_rx) = mpsc::channel(1);
    // tx.send(AgentMessage::Send {
    //     sender: AgentId::new("sender_agent", "sender_agent_1"),
    //     recipient: AgentId::new("ChatAgent", "agent_id_2"),
    //     message: Box::new(ChatMessage::Text {
    //         content: "Hello from Agent 1!".to_string(),
    //     }),
    //     response_tx: resp_tx,
    // }).await.unwrap();

    let topic_id = TopicId::new("topic_id_1");
    // tx.send(AgentMessage::Publish {
    //     sender: AgentId::new("sender_agent", "sender_agent_1"),
    //     topic: topic_id.clone(),
    //     message: Box::new(ChatMessage::Text {
    //         content: "hello world".to_string(),
    //     }),
    // }).await.unwrap();

    // if let Some(response) = resp_rx.recv().await {
    //     match response {
    //         Ok(msg) => println!("Received response: {:?}", msg),
    //         Err(err) => println!("Error sending message: {}", err),
    //     }
    // }

    // runtime_task.await.unwrap();
    loop {
        // Poll for background tasks
        // let background_tasks = runtime.background_tasks.clone();
        // if background_tasks.is_empty() {
        //     break;
        // }

        // println!(
        //     "{:?} pending bg tasks: {:?}/{:?}",
        //     runtime.run_context.run_state,
        //     runtime.outstanding_tasks,
        //     runtime.background_tasks
        // );

        runtime.process_next().await
    }
}

impl AgentRuntime {
    pub fn spawn_agent(&self, agent: Agent) -> (Sender<AgentMessage>, Receiver<AgentMessage>) {
        let (tx, rx) = mpsc::channel(32);
        let id = agent.get_id();
        let mut agents = self.instantiated_agents.clone();
        agents.insert(id.clone(), agent);
        self.instantiated_agents = agents;

        tokio::spawn(async move {
            let mut rx = rx;
            while let Some(message) = rx.recv().await {
                match message {
                    AgentMessage::Send {
                        sender,
                        message,
                        response_tx,
                        ..
                    } => {
                        let response = agent
                            .on_messages(
                                message,
                                ChatMessageContext {
                                    sender: sender,
                                    topic_id: None,
                                    is_rpc: true,
                                },
                            )
                            .await;
                        if let Err(_) = response_tx.send(response).await {
                            // Handle the error (e.g., the sender might no longer be waiting for a response)
                        }
                    }
                    _ => {
                        // Handle other message types if needed
                    }
                };
            }
        });

        (tx, rx)
    }
}

#[derive(Debug)]
enum AgentMessage {
    Send {
        sender: AgentId,
        recipient: AgentId,
        message: ChatMessage, // Boxed for dynamic dispatch
        response_tx: Sender<Result<ResponseMessage, String>>, // For sending back responses
    },
    Publish {
        sender: AgentId,
        topic: String,
        message: ChatMessage,
    },
}
