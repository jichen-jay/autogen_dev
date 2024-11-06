use crate::agent::agent_runtime::AgentRuntime;
use crate::msg_types::AgentId;

pub struct TerminationCondition;

pub struct GroupChat {
    pub runtime: AgentRuntime,
    pub participants: Vec<AgentId>,
    pub termination_condition: Option<TerminationCondition>, // Option to store termination condition
    pub group_chat_manager: AgentId,                         // Store the manager's AgentId
    pub parent_topic_type: String,
    pub group_topic_type: String,
    pub participant_topic_types: Vec<String>, // Store a list of participant topic types
    pub participant_descriptions: Vec<String>, // Store descriptions if needed
}

impl GroupChat {
    pub fn new(
        participants: Vec<AgentId>,
        runtime: AgentRuntime,
        group_chat_manager: AgentId,
        group_topic_type: String,
        termination_condition: Option<TerminationCondition>,
        parent_topic_type: String,
        participant_topic_types: Vec<String>,
        participant_descriptions: Vec<String>,
    ) -> Self {
        GroupChat {
            runtime,
            participants,
            termination_condition,
            group_chat_manager,
            group_topic_type,
            parent_topic_type,
            participant_topic_types,
            participant_descriptions,
        }
    }
}

impl GroupChat {
    // ... (existing methods) ...
    pub async fn run_stream(&mut self, task: &str) -> Result<(), String> {
        // ... (Initialize channels and start task handler) ...

        // 1. Register participants
        // for (i, participant_id) in self.participants.iter().enumerate() {
        //     let agent_type = self.participant_topic_types[i].clone(); // Get from list
        //     let topic = TopicId::new(agent_type.clone());
        //     let subscription = Subscription {
        //         id: topic.clone(),
        //         subscription_id: SubscriptionId::new(participant_id.clone(), topic.clone()),
        //     };
        //     self.runtime.add_subscription(subscription).await;

        //     // ... (Optionally register tools and other subscriptions) ...
        // }

        // // 2. Register the group chat manager
        // // ... (Similar logic for registering the manager with the correct topic types and subscriptions, ensuring it can handle parent topic, round-robin, and possibly individual messages) ...

        // // 3. Publish the task
        // self.runtime
        //     .publish_message(
        //         ChatMessage::Text {
        //             content: task.to_string(),
        //         },
        //         TopicId::new(self.group_topic_type.clone()),
        //         None,
        //     )
        //     .await;

        // ... (Process messages, ensure the process_messages task correctly routes messages based on topic types) ...
        todo!()
    }

    // async fn process_messages(
    //     &mut self,
    //     message_rx: &mut UnboundedReceiver<ChatMessage>,
    //     mut stop_rx: oneshot::Receiver<()>,
    // ) {
    //     let mut messages: Vec<ChatMessage> = vec![];
    //     loop {
    //         tokio::select! {
    //             Some(message) = message_rx.recv() => {
    //                 // ... (Logic to determine message type based on context or message properties - ensure you handle the three levels of topic types) ...

    //                 messages.push(message);
    //                 // ... (Handle the message based on the topic) ...
    //             }
    //             _ = &mut stop_rx => {
    //                 break;
    //             }
    //         }
    //     }
    // ... (Process messages) ...
    // }
}

#[tokio::main]
async fn main() {
    let parent_topic_type = "team_topic";
    let group_topic_type = "round_robin_group_topic";
    let participant_topic_types = vec!["participant1".to_string(), "participant2".to_string()];
    let participant_descriptions = vec![
        "Description for participant 1".to_string(),
        "Description for participant 2".to_string(),
    ];

    // ... (Create agents, register them) ...

    // Register the group chat manager
    // ... (Register the manager with the parent and the round-robin group topic) ...

    // Create the GroupChat
    // let mut group_chat = GroupChat::new(
    //     vec![chat_agent_id, another_agent_id],
    //     runtime,
    //     group_chat_manager_id,
    //     group_topic_type.to_string(),
    //     Some(TerminationCondition::MaxMessages(5)),
    //     parent_topic_type.to_string(),
    //     participant_topic_types,
    //     participant_descriptions,
    // );

    // // Start the conversation stream
    // group_chat.run_stream("Hello everyone!").await;
}
