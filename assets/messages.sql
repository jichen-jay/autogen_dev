-- Table for Agents
CREATE TABLE agents (
    agent_id VARCHAR(255) PRIMARY KEY,
    agent_name VARCHAR(255) NOT NULL,
    agent_description TEXT
);

-- Table for Topics
CREATE TABLE topics (
    topic_id VARCHAR(255) PRIMARY KEY,
    topic_name VARCHAR(255) NOT NULL
);

-- Table for Subscriptions (Modified to allow multiple subscriptions)
CREATE TABLE subscriptions (
    subscription_id VARCHAR(255) PRIMARY KEY,
    agent_id VARCHAR(255) NOT NULL,
    topic_id VARCHAR(255) NOT NULL,
    FOREIGN KEY (agent_id) REFERENCES agents(agent_id),
    FOREIGN KEY (topic_id) REFERENCES topics(topic_id)
);

-- Table for Messages (No changes needed, but this is where the subscription information is used)
CREATE TABLE messages (
    message_id VARCHAR(255) PRIMARY KEY,
    sender_id VARCHAR(255) NOT NULL,
    recipient_id VARCHAR(255) NOT NULL,
    topic_id VARCHAR(255) NOT NULL,
    message_type VARCHAR(255) NOT NULL, -- UserMessage, SystemMessage, AssistantMessage, etc.
    message_content TEXT NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (sender_id) REFERENCES agents(agent_id),
    FOREIGN KEY (recipient_id) REFERENCES agents(agent_id),
    FOREIGN KEY (topic_id) REFERENCES topics(topic_id)
);
