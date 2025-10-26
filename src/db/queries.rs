// Prepared query statements for ScyllaDB operations

// conversation_lineage queries
pub const INSERT_MESSAGE: &str = r#"
    INSERT INTO conversation_lineage (
        conversation_id, message_id, parent_message_id, role,
        content_type, content_data, content_metadata, lineage,
        created_at, created_by
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
"#;

pub const SELECT_MESSAGE: &str = r#"
    SELECT conversation_id, message_id, parent_message_id, role,
           content_type, content_data, content_metadata, lineage,
           created_at, created_by
    FROM conversation_lineage
    WHERE conversation_id = ? AND message_id = ?
"#;

pub const SELECT_MESSAGE_CHILDREN: &str = r#"
    SELECT conversation_id, message_id, parent_message_id, role,
           content_type, content_data, content_metadata, lineage,
           created_at, created_by
    FROM conversation_lineage
    WHERE conversation_id = ? AND parent_message_id = ?
    ALLOW FILTERING
"#;

pub const SELECT_MESSAGES_BY_IDS: &str = r#"
    SELECT conversation_id, message_id, parent_message_id, role,
           content_type, content_data, content_metadata, lineage,
           created_at, created_by
    FROM conversation_lineage
    WHERE conversation_id = ? AND message_id IN ?
"#;

pub const SELECT_ALL_MESSAGES: &str = r#"
    SELECT conversation_id, message_id, parent_message_id, role,
           content_type, content_data, content_metadata, lineage,
           created_at, created_by
    FROM conversation_lineage
    WHERE conversation_id = ?
"#;

pub const DELETE_CONVERSATION: &str = r#"
    DELETE FROM conversation_lineage WHERE conversation_id = ?
"#;

// conversation_branches queries
pub const INSERT_BRANCH: &str = r#"
    INSERT INTO conversation_branches (
        conversation_id, branch_id, branch_name, leaf_message_id,
        created_at, last_updated, created_by, is_active
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
"#;

pub const SELECT_BRANCH: &str = r#"
    SELECT conversation_id, branch_id, branch_name, leaf_message_id,
           created_at, last_updated, created_by, is_active
    FROM conversation_branches
    WHERE conversation_id = ? AND branch_id = ?
"#;

pub const SELECT_BRANCHES_BY_CONVERSATION: &str = r#"
    SELECT conversation_id, branch_id, branch_name, leaf_message_id,
           created_at, last_updated, created_by, is_active
    FROM conversation_branches
    WHERE conversation_id = ?
"#;

pub const UPDATE_BRANCH_LEAF: &str = r#"
    UPDATE conversation_branches
    SET leaf_message_id = ?, last_updated = ?
    WHERE conversation_id = ? AND branch_id = ?
"#;

pub const UPDATE_BRANCH_NAME: &str = r#"
    UPDATE conversation_branches
    SET branch_name = ?, last_updated = ?
    WHERE conversation_id = ? AND branch_id = ?
"#;

pub const DELETE_BRANCH: &str = r#"
    DELETE FROM conversation_branches
    WHERE conversation_id = ? AND branch_id = ?
"#;

// conversation_shares queries
pub const INSERT_SHARE: &str = r#"
    INSERT INTO conversation_shares (
        conversation_id, shared_with, permission, shared_at, shared_by
    ) VALUES (?, ?, ?, ?, ?)
"#;

pub const SELECT_SHARES_BY_CONVERSATION: &str = r#"
    SELECT conversation_id, shared_with, permission, shared_at, shared_by
    FROM conversation_shares
    WHERE conversation_id = ?
"#;

pub const SELECT_SHARE: &str = r#"
    SELECT conversation_id, shared_with, permission, shared_at, shared_by
    FROM conversation_shares
    WHERE conversation_id = ? AND shared_with = ?
"#;

pub const DELETE_SHARE: &str = r#"
    DELETE FROM conversation_shares
    WHERE conversation_id = ? AND shared_with = ?
"#;

// user_conversations queries
pub const INSERT_USER_CONVERSATION: &str = r#"
    INSERT INTO user_conversations (
        user_id, last_activity, conversation_id, active_branch_id
    ) VALUES (?, ?, ?, ?)
"#;

pub const SELECT_USER_CONVERSATIONS: &str = r#"
    SELECT user_id, last_activity, conversation_id, active_branch_id
    FROM user_conversations
    WHERE user_id = ?
    LIMIT ?
"#;

pub const UPDATE_USER_CONVERSATION_ACTIVITY: &str = r#"
    INSERT INTO user_conversations (
        user_id, last_activity, conversation_id, active_branch_id
    ) VALUES (?, ?, ?, ?)
"#;

// branch_by_leaf queries
pub const INSERT_BRANCH_BY_LEAF: &str = r#"
    INSERT INTO branch_by_leaf (leaf_message_id, conversation_id, branch_id)
    VALUES (?, ?, ?)
"#;

pub const SELECT_BRANCH_BY_LEAF: &str = r#"
    SELECT leaf_message_id, conversation_id, branch_id
    FROM branch_by_leaf
    WHERE leaf_message_id = ?
"#;

pub const DELETE_BRANCH_BY_LEAF: &str = r#"
    DELETE FROM branch_by_leaf WHERE leaf_message_id = ?
"#;
