use uuid::Uuid;

/// Validates that a string is a valid UUID
pub fn validate_uuid(uuid_str: &str) -> Result<Uuid, String> {
    Uuid::parse_str(uuid_str).map_err(|e| format!("Invalid UUID: {}", e))
}

/// Validates a list of UUIDs
pub fn validate_uuid_list(uuids: &[Uuid]) -> Result<(), String> {
    if uuids.is_empty() {
        return Err("UUID list cannot be empty".to_string());
    }
    Ok(())
}
