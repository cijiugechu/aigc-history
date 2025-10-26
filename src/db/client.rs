use scylla::{Session, SessionBuilder};
use std::sync::Arc;
use thiserror::Error;

use crate::config::ScyllaConfig;

use super::migration;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database connection error: {0}")]
    ConnectionError(#[from] scylla::transport::errors::NewSessionError),

    #[error("Query error: {0}")]
    QueryError(#[from] scylla::transport::errors::QueryError),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Not found")]
    NotFound,

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Migration error: {0}")]
    MigrationError(String),
}

#[derive(Clone)]
pub struct DbClient {
    session: Arc<Session>,
    keyspace: String,
}

impl DbClient {
    pub async fn new(config: &ScyllaConfig) -> Result<Self, DbError> {
        tracing::info!("Initializing Scylla session with nodes {:?}", config.nodes);
        let session = SessionBuilder::new()
            .known_nodes(&config.nodes)
            .build()
            .await?;

        tracing::info!("Scylla session established, starting migrations");
        migration::run_migrations(&session, config).await?;

        tracing::info!(
            "Migrations complete, selecting keyspace '{}'",
            config.keyspace
        );
        // Use the keyspace for all connections
        session.use_keyspace(&config.keyspace, false).await?;
        tracing::info!("Keyspace '{}' selected", config.keyspace);

        Ok(DbClient {
            session: Arc::new(session),
            keyspace: config.keyspace.clone(),
        })
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn keyspace(&self) -> &str {
        &self.keyspace
    }
}
