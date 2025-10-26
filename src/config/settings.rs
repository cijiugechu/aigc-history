use std::env;

#[derive(Debug, Clone)]
pub struct Settings {
    pub server: ServerConfig,
    pub scylla: ScyllaConfig,
    pub s3: S3Config,
    pub app: AppConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct ScyllaConfig {
    pub nodes: Vec<String>,
    pub keyspace: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone)]
pub struct S3Config {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub region: String,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub max_lineage_depth: usize,
    pub max_batch_size: usize,
}

impl Settings {
    pub fn from_env() -> Result<Self, String> {
        Ok(Settings {
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .map_err(|e| format!("Invalid SERVER_PORT: {}", e))?,
            },
            scylla: ScyllaConfig {
                nodes: env::var("SCYLLA_NODES")
                    .unwrap_or_else(|_| "localhost:9042".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
                keyspace: env::var("SCYLLA_KEYSPACE")
                    .unwrap_or_else(|_| "aigc_history".to_string()),
                username: env::var("SCYLLA_USERNAME").ok(),
                password: env::var("SCYLLA_PASSWORD").ok(),
            },
            s3: S3Config {
                endpoint: env::var("S3_ENDPOINT")
                    .unwrap_or_else(|_| "http://localhost:9000".to_string()),
                access_key: env::var("S3_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
                secret_key: env::var("S3_SECRET_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
                bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "aigc-images".to_string()),
                region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            },
            app: AppConfig {
                max_lineage_depth: env::var("MAX_LINEAGE_DEPTH")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()
                    .unwrap_or(1000),
                max_batch_size: env::var("MAX_BATCH_SIZE")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .unwrap_or(100),
            },
        })
    }
}
