// Integration tests for conversation operations
// Run with: cargo test --test integration

#[cfg(test)]
mod tests {
    use aigc_history::{
        config::{AppConfig, ScyllaConfig},
        db::DbClient,
        domain::{ContentType, MessageRole, TextContent},
        repositories::LineageRepository,
        services::ConversationService,
    };

    async fn setup_test_service() -> ConversationService {
        let scylla_config = ScyllaConfig {
            nodes: vec!["localhost:9042".to_string()],
            keyspace: "aigc_history_test".to_string(),
            username: None,
            password: None,
        };

        let app_config = AppConfig {
            max_lineage_depth: 1000,
            max_batch_size: 100,
        };

        let db_client = DbClient::new(&scylla_config)
            .await
            .expect("Failed to connect to test database");

        let lineage_repo = LineageRepository::new(db_client);

        ConversationService::new(lineage_repo, app_config)
    }

    #[tokio::test]
    #[ignore] // Requires running ScyllaDB
    async fn test_create_conversation() {
        let service = setup_test_service().await;

        let result = service
            .create_conversation("Test Conversation".to_string(), "user_test".to_string())
            .await;

        assert!(result.is_ok());
        let conversation = result.unwrap();
        assert_eq!(conversation.title().unwrap(), "Test Conversation");
    }

    #[tokio::test]
    #[ignore] // Requires running ScyllaDB
    async fn test_append_message() {
        let service = setup_test_service().await;

        // Create conversation
        let conversation = service
            .create_conversation("Test Conversation".to_string(), "user_test".to_string())
            .await
            .unwrap();

        // Append message
        let content = ContentType::Text(TextContent {
            text: "Hello, world!".to_string(),
        });

        let result = service
            .append_message(
                conversation.conversation_id,
                conversation.root_message.message_id,
                MessageRole::Human,
                content,
                std::collections::HashMap::new(),
                "user_test".to_string(),
            )
            .await;

        assert!(result.is_ok());
        let message = result.unwrap();
        assert_eq!(message.lineage.len(), 2); // Root + new message
    }

    #[tokio::test]
    #[ignore] // Requires running ScyllaDB
    async fn test_get_lineage_path() {
        let service = setup_test_service().await;

        // Create conversation
        let conversation = service
            .create_conversation("Test Conversation".to_string(), "user_test".to_string())
            .await
            .unwrap();

        // Append first message
        let content1 = ContentType::Text(TextContent {
            text: "Message 1".to_string(),
        });

        let message1 = service
            .append_message(
                conversation.conversation_id,
                conversation.root_message.message_id,
                MessageRole::Human,
                content1,
                std::collections::HashMap::new(),
                "user_test".to_string(),
            )
            .await
            .unwrap();

        // Append second message
        let content2 = ContentType::Text(TextContent {
            text: "Message 2".to_string(),
        });

        let message2 = service
            .append_message(
                conversation.conversation_id,
                message1.message_id,
                MessageRole::Assistant,
                content2,
                std::collections::HashMap::new(),
                "assistant".to_string(),
            )
            .await
            .unwrap();

        // Get lineage
        let lineage = service
            .get_lineage_path(conversation.conversation_id, message2.message_id)
            .await
            .unwrap();

        assert_eq!(lineage.len(), 3); // Root + message1 + message2
    }
}

