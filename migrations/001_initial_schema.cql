-- AIGC History Service - Initial Schema
-- Create keyspace
CREATE KEYSPACE IF NOT EXISTS aigc_history 
WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};

USE aigc_history;

-- Main table: conversation_lineage (tree structure with lineage tracking)
CREATE TABLE IF NOT EXISTS conversation_lineage (
    conversation_id UUID,
    message_id UUID,
    parent_message_id UUID,
    role TEXT,
    content_type TEXT,
    content_data TEXT,
    content_metadata MAP<TEXT, TEXT>,
    lineage LIST<UUID>,
    created_at TIMESTAMP,
    created_by TEXT,
    PRIMARY KEY (conversation_id, message_id)
);

-- Branch tracking table
CREATE TABLE IF NOT EXISTS conversation_branches (
    conversation_id UUID,
    branch_id UUID,
    branch_name TEXT,
    leaf_message_id UUID,
    created_at TIMESTAMP,
    last_updated TIMESTAMP,
    created_by TEXT,
    is_active BOOLEAN,
    PRIMARY KEY (conversation_id, branch_id)
);

-- Conversation sharing/permissions table
CREATE TABLE IF NOT EXISTS conversation_shares (
    conversation_id UUID,
    shared_with TEXT,
    permission TEXT,
    shared_at TIMESTAMP,
    shared_by TEXT,
    PRIMARY KEY (conversation_id, shared_with)
);

-- User conversations index
CREATE TABLE IF NOT EXISTS user_conversations (
    user_id TEXT,
    last_activity TIMESTAMP,
    conversation_id UUID,
    active_branch_id UUID,
    PRIMARY KEY (user_id, last_activity, conversation_id)
) WITH CLUSTERING ORDER BY (last_activity DESC, conversation_id ASC);

-- Secondary index for branch lookups by leaf message
CREATE TABLE IF NOT EXISTS branch_by_leaf (
    leaf_message_id UUID,
    conversation_id UUID,
    branch_id UUID,
    PRIMARY KEY (leaf_message_id)
);

