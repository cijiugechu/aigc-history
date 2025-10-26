use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentType {
    Text(TextContent),
    Image(ImageContent),
    ToolCall(ToolCallContent),
    ToolResult(ToolResultContent),
    ImageBatch(ImageBatchContent),
    Metadata(MetadataContent),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextContent {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageContent {
    pub image_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCallContent {
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub tool_call_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolResultContent {
    pub tool_call_id: String,
    pub result: serde_json::Value,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageBatchContent {
    pub images: Vec<ImageBatchItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageBatchItem {
    pub image_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetadataContent {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub is_public: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fork_from_conversation_id: Option<uuid::Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fork_from_message_id: Option<uuid::Uuid>,
}

pub type ContentMetadata = HashMap<String, String>;

impl ContentType {
    pub fn to_type_string(&self) -> &str {
        match self {
            ContentType::Text(_) => "text",
            ContentType::Image(_) => "image",
            ContentType::ToolCall(_) => "tool_call",
            ContentType::ToolResult(_) => "tool_result",
            ContentType::ImageBatch(_) => "image_batch",
            ContentType::Metadata(_) => "metadata",
        }
    }

    pub fn from_parts(content_type: &str, content_data: &str) -> Result<Self, String> {
        match content_type {
            "text" => Ok(ContentType::Text(
                serde_json::from_str(content_data)
                    .map_err(|e| format!("Failed to parse text content: {}", e))?,
            )),
            "image" => Ok(ContentType::Image(
                serde_json::from_str(content_data)
                    .map_err(|e| format!("Failed to parse image content: {}", e))?,
            )),
            "tool_call" => Ok(ContentType::ToolCall(
                serde_json::from_str(content_data)
                    .map_err(|e| format!("Failed to parse tool_call content: {}", e))?,
            )),
            "tool_result" => Ok(ContentType::ToolResult(
                serde_json::from_str(content_data)
                    .map_err(|e| format!("Failed to parse tool_result content: {}", e))?,
            )),
            "image_batch" => Ok(ContentType::ImageBatch(
                serde_json::from_str(content_data)
                    .map_err(|e| format!("Failed to parse image_batch content: {}", e))?,
            )),
            "metadata" => Ok(ContentType::Metadata(
                serde_json::from_str(content_data)
                    .map_err(|e| format!("Failed to parse metadata content: {}", e))?,
            )),
            _ => Err(format!("Unknown content type: {}", content_type)),
        }
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        match self {
            ContentType::Text(c) => serde_json::to_string(c),
            ContentType::Image(c) => serde_json::to_string(c),
            ContentType::ToolCall(c) => serde_json::to_string(c),
            ContentType::ToolResult(c) => serde_json::to_string(c),
            ContentType::ImageBatch(c) => serde_json::to_string(c),
            ContentType::Metadata(c) => serde_json::to_string(c),
        }
    }
}
