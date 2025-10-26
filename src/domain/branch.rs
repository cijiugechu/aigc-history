use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub conversation_id: Uuid,
    pub branch_id: Uuid,
    pub branch_name: String,
    pub leaf_message_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub created_by: String,
    pub is_active: bool,
}

impl Branch {
    pub fn new(
        conversation_id: Uuid,
        branch_name: String,
        leaf_message_id: Uuid,
        created_by: String,
    ) -> Self {
        let now = Utc::now();
        Branch {
            conversation_id,
            branch_id: Uuid::new_v4(),
            branch_name,
            leaf_message_id,
            created_at: now,
            last_updated: now,
            created_by,
            is_active: true,
        }
    }

    pub fn update_leaf(&mut self, new_leaf_id: Uuid) {
        self.leaf_message_id = new_leaf_id;
        self.last_updated = Utc::now();
    }
}
