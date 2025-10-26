use scylla::Session;
use std::path::PathBuf;
use tokio::{
    fs,
    time::{Duration, sleep},
};
use tracing::{debug, error, info, warn};

use crate::config::ScyllaConfig;

use super::DbError;

/// Run all `.cql` migrations using the provided session.
/// Migrations are executed sequentially in lexicographic order.
pub async fn run_migrations(session: &Session, config: &ScyllaConfig) -> Result<(), DbError> {
    let migrations_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations");
    let migrations_dir_display = migrations_dir.to_str().unwrap_or("migrations directory");

    let mut entries = fs::read_dir(&migrations_dir).await.map_err(|e| {
        DbError::MigrationError(format!(
            "Failed to read migrations directory {}: {}",
            migrations_dir_display, e
        ))
    })?;

    let mut files = Vec::new();
    while let Some(entry) = entries.next_entry().await.map_err(|e| {
        DbError::MigrationError(format!(
            "Failed to iterate migrations in {}: {}",
            migrations_dir_display, e
        ))
    })? {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("cql") {
            files.push(path);
        }
    }

    files.sort();

    if files.is_empty() {
        warn!(
            "No migrations found in '{}'; skipping migration step",
            migrations_dir_display
        );
        return Ok(());
    }

    info!("Applying {} migration file(s)", files.len());

    let mut keyspace_ready = false;

    for path in files {
        let display_path = path
            .to_str()
            .map(|s| s.to_string())
            .or_else(|| {
                path.file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());
        info!("Running migration file: {}", display_path);

        let migration_sql = fs::read_to_string(&path).await.map_err(|e| {
            DbError::MigrationError(format!("Failed to read {}: {}", display_path, e))
        })?;

        // Replace default keyspace name with configured keyspace, if present.
        let migration_sql = migration_sql.replace("aigc_history", &config.keyspace);

        let statements: Vec<String> = migration_sql
            .split(';')
            .filter_map(|chunk| {
                let cleaned = chunk
                    .lines()
                    .filter_map(|line| {
                        let trimmed = line.trim();
                        if trimmed.is_empty() || trimmed.starts_with("--") {
                            None
                        } else {
                            Some(trimmed)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                let statement = cleaned.trim().to_string();
                if statement.is_empty() {
                    None
                } else {
                    Some(statement)
                }
            })
            .collect();

        info!("Executing {} statement(s)", statements.len());

        for (index, statement) in statements.iter().enumerate() {
            let upper = statement.to_uppercase();
            if upper.starts_with("USE ") {
                info!(
                    "Statement {}: switching keyspace via session.use_keyspace(\"{}\")",
                    index + 1,
                    config.keyspace
                );
                ensure_keyspace_selected(session, &config.keyspace, index + 1, &display_path)
                    .await?;
                keyspace_ready = true;
                continue;
            } else if upper.contains("CREATE KEYSPACE") {
                info!("Statement {}: creating keyspace if absent", index + 1);
            } else if upper.contains("CREATE TABLE") {
                let table_name = statement
                    .split_whitespace()
                    .skip_while(|&s| s.to_uppercase() != "TABLE")
                    .nth(1)
                    .unwrap_or("unknown");
                info!("Statement {}: creating table {}", index + 1, table_name);
            } else if upper.contains("CREATE INDEX") {
                info!("Statement {}: creating index", index + 1);
            } else {
                info!("Statement {}: executing migration statement", index + 1);
            }

            if !keyspace_ready && !upper.contains("CREATE KEYSPACE") {
                ensure_keyspace_selected(session, &config.keyspace, index + 1, &display_path)
                    .await?;
                keyspace_ready = true;
            }

            match session.query(statement.as_str(), &[]).await {
                Ok(_) => {
                    info!(
                        "Statement {} applied successfully from {}",
                        index + 1,
                        display_path
                    );
                }
                Err(err) => {
                    // Allow idempotent migrations.
                    let error_msg = err.to_string();
                    if error_msg.contains("already exists") {
                        warn!(
                            "Statement {} skipped: object already exists ({}).",
                            index + 1,
                            error_msg
                        );
                        continue;
                    }

                    error!(
                        "Failed to execute statement {} from {}: {}",
                        index + 1,
                        display_path,
                        error_msg
                    );
                    debug!("Statement {} content: {}", index + 1, statement);

                    return Err(DbError::MigrationError(format!(
                        "Failed to execute statement {} from {}: {}",
                        index + 1,
                        display_path,
                        err
                    )));
                }
            }

            if upper.contains("CREATE KEYSPACE") {
                if let Err(err) = session.await_schema_agreement().await {
                    warn!(
                        "Schema agreement wait after creating keyspace '{}' failed: {}",
                        config.keyspace, err
                    );
                }
                if let Err(err) = session.refresh_metadata().await {
                    warn!(
                        "Metadata refresh after creating keyspace '{}' failed: {}",
                        config.keyspace, err
                    );
                }
                ensure_keyspace_selected(session, &config.keyspace, index + 1, &display_path)
                    .await?;
                keyspace_ready = true;
            }
        }
    }

    info!("Database migrations applied successfully");
    Ok(())
}

async fn ensure_keyspace_selected(
    session: &Session,
    keyspace: &str,
    statement_index: usize,
    display_path: &str,
) -> Result<(), DbError> {
    const MAX_ATTEMPTS: usize = 6;
    for attempt in 0..MAX_ATTEMPTS {
        match session.use_keyspace(keyspace, false).await {
            Ok(_) => {
                if attempt > 0 {
                    info!(
                        "Successfully selected keyspace '{}' after {} retry(s)",
                        keyspace, attempt
                    );
                } else {
                    info!("Selected keyspace '{}'", keyspace);
                }
                return Ok(());
            }
            Err(err) => {
                let err_msg = err.to_string();
                warn!(
                    "Attempt {} to select keyspace '{}' before statement {} in {} failed: {}",
                    attempt + 1,
                    keyspace,
                    statement_index,
                    display_path,
                    err_msg
                );

                if attempt + 1 == MAX_ATTEMPTS {
                    return Err(DbError::MigrationError(format!(
                        "Failed to select keyspace '{}' prior to statement {} in {}: {}",
                        keyspace, statement_index, display_path, err
                    )));
                }

                if let Err(refresh_err) = session.refresh_metadata().await {
                    warn!(
                        "Refreshing metadata after failed keyspace selection attempt {} for '{}' failed: {}",
                        attempt + 1,
                        keyspace,
                        refresh_err
                    );
                }
                let backoff = Duration::from_millis(250 * (attempt as u64 + 1));
                sleep(backoff).await;
            }
        }
    }

    Err(DbError::MigrationError(format!(
        "Unable to select keyspace '{}' after {} attempts when processing {}",
        keyspace, MAX_ATTEMPTS, display_path
    )))
}
