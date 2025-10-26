use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Share {
    pub conversation_id: Uuid,
    pub shared_with: String,
    pub permission: Permission,
    pub shared_at: DateTime<Utc>,
    pub shared_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    Read,
    Branch,
    Fork,
}

impl Permission {
    pub fn as_str(&self) -> &str {
        match self {
            Permission::Read => "read",
            Permission::Branch => "branch",
            Permission::Fork => "fork",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "read" => Some(Permission::Read),
            "branch" => Some(Permission::Branch),
            "fork" => Some(Permission::Fork),
            _ => None,
        }
    }

    pub fn can_read(&self) -> bool {
        matches!(
            self,
            Permission::Read | Permission::Branch | Permission::Fork
        )
    }

    pub fn can_branch(&self) -> bool {
        matches!(self, Permission::Branch | Permission::Fork)
    }

    pub fn can_fork(&self) -> bool {
        matches!(self, Permission::Fork)
    }
}
