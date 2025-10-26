use uuid::Uuid;

/// Compute lineage for a new message given parent's lineage
pub fn compute_lineage(parent_lineage: &[Uuid], new_message_id: Uuid) -> Vec<Uuid> {
    let mut lineage = parent_lineage.to_vec();
    lineage.push(new_message_id);
    lineage
}

/// Validate lineage depth
pub fn validate_lineage_depth(lineage: &[Uuid], max_depth: usize) -> Result<(), String> {
    if lineage.len() > max_depth {
        return Err(format!(
            "Lineage depth {} exceeds maximum allowed depth {}",
            lineage.len(),
            max_depth
        ));
    }
    Ok(())
}

/// Check if message A is an ancestor of message B by comparing lineages
pub fn is_ancestor(ancestor_id: Uuid, descendant_lineage: &[Uuid]) -> bool {
    descendant_lineage.contains(&ancestor_id)
}

/// Get the common ancestor path between two lineages
pub fn common_ancestor_path(lineage_a: &[Uuid], lineage_b: &[Uuid]) -> Vec<Uuid> {
    lineage_a
        .iter()
        .zip(lineage_b.iter())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| *a)
        .collect()
}

/// Calculate the depth difference between two messages in the same tree
pub fn depth_difference(lineage_a: &[Uuid], lineage_b: &[Uuid]) -> usize {
    let common = common_ancestor_path(lineage_a, lineage_b).len();
    (lineage_a.len() - common) + (lineage_b.len() - common)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_lineage() {
        let parent_lineage = vec![Uuid::new_v4(), Uuid::new_v4()];
        let new_id = Uuid::new_v4();

        let lineage = compute_lineage(&parent_lineage, new_id);

        assert_eq!(lineage.len(), 3);
        assert_eq!(lineage[2], new_id);
    }

    #[test]
    fn test_is_ancestor() {
        let ancestor = Uuid::new_v4();
        let middle = Uuid::new_v4();
        let leaf = Uuid::new_v4();
        let lineage = vec![ancestor, middle, leaf];

        assert!(is_ancestor(ancestor, &lineage));
        assert!(is_ancestor(middle, &lineage));
        assert!(!is_ancestor(Uuid::new_v4(), &lineage));
    }

    #[test]
    fn test_common_ancestor_path() {
        let root = Uuid::new_v4();
        let common = Uuid::new_v4();
        let branch_a = Uuid::new_v4();
        let branch_b = Uuid::new_v4();

        let lineage_a = vec![root, common, branch_a];
        let lineage_b = vec![root, common, branch_b];

        let common_path = common_ancestor_path(&lineage_a, &lineage_b);

        assert_eq!(common_path, vec![root, common]);
    }
}
