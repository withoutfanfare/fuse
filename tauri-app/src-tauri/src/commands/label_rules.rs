use tauri::State;

use crate::db::DbState;
use crate::models::{LabelRule, LabelRuleMatch};

use super::CommandError;

/// Parse a label_automation_rules row into a LabelRule.
fn parse_label_rule(row: &rusqlite::Row) -> Result<LabelRule, rusqlite::Error> {
    let config_str: String = row.get(3)?;
    let config: serde_json::Value =
        serde_json::from_str(&config_str).unwrap_or(serde_json::Value::Object(Default::default()));
    let enabled_int: i64 = row.get(4)?;

    Ok(LabelRule {
        id: row.get(0)?,
        label_pattern: row.get(1)?,
        action_type: row.get(2)?,
        action_config: config,
        enabled: enabled_int != 0,
        created_at: row.get(5)?,
    })
}

/// List all label automation rules.
#[tauri::command]
pub fn list_label_rules(state: State<'_, DbState>) -> Result<Vec<LabelRule>, CommandError> {
    let db = state.reader.lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT id, label_pattern, action_type, action_config, enabled, created_at \
         FROM label_automation_rules \
         ORDER BY created_at ASC",
    )?;
    let rules = stmt
        .query_map([], parse_label_rule)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rules)
}

/// Create a new label automation rule.
#[tauri::command]
pub fn create_label_rule(
    label_pattern: String,
    action_type: String,
    action_config: serde_json::Value,
    state: State<'_, DbState>,
) -> Result<LabelRule, CommandError> {
    // Validate action_type
    let valid_types = ["set_priority", "add_checklist", "assign_group"];
    if !valid_types.contains(&action_type.as_str()) {
        return Err(CommandError::Gh(format!(
            "Invalid action_type '{}'. Must be one of: {}",
            action_type,
            valid_types.join(", ")
        )));
    }

    let config_str = serde_json::to_string(&action_config)?;
    let db = state.writer.lock().unwrap();

    db.execute(
        "INSERT INTO label_automation_rules (label_pattern, action_type, action_config) \
         VALUES (?1, ?2, ?3)",
        rusqlite::params![label_pattern, action_type, config_str],
    )?;

    let id = db.last_insert_rowid();
    let created_at: String = db.query_row(
        "SELECT created_at FROM label_automation_rules WHERE id = ?1",
        [id],
        |row| row.get(0),
    )?;

    Ok(LabelRule {
        id,
        label_pattern,
        action_type,
        action_config,
        enabled: true,
        created_at,
    })
}

/// Delete a label automation rule by ID.
#[tauri::command]
pub fn delete_label_rule(id: i64, state: State<'_, DbState>) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();
    let affected = db.execute("DELETE FROM label_automation_rules WHERE id = ?1", [id])?;
    if affected == 0 {
        return Err(CommandError::NotFound(format!("Label rule with id {}", id)));
    }
    Ok(())
}

/// Toggle the enabled state of a label rule.
#[tauri::command]
pub fn toggle_label_rule(
    id: i64,
    enabled: bool,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();
    let affected = db.execute(
        "UPDATE label_automation_rules SET enabled = ?1 WHERE id = ?2",
        rusqlite::params![enabled as i64, id],
    )?;
    if affected == 0 {
        return Err(CommandError::NotFound(format!("Label rule with id {}", id)));
    }
    Ok(())
}

/// Check whether a glob-style label pattern matches a given label.
/// Supports `*` as a wildcard for any sequence of characters.
fn pattern_matches(pattern: &str, label: &str) -> bool {
    let pattern_lower = pattern.to_lowercase();
    let label_lower = label.to_lowercase();

    // Simple glob matching: split on '*' and check that parts appear in order
    let parts: Vec<&str> = pattern_lower.split('*').collect();
    if parts.len() == 1 {
        // No wildcard — exact match
        return pattern_lower == label_lower;
    }

    let mut pos = 0usize;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        match label_lower[pos..].find(part) {
            Some(found) => {
                // First segment must match at the start if there's no leading '*'
                if i == 0 && found != 0 {
                    return false;
                }
                pos += found + part.len();
            }
            None => return false,
        }
    }

    // Last segment must match at the end if there's no trailing '*'
    if !parts.last().unwrap_or(&"").is_empty() {
        return pos == label_lower.len();
    }

    true
}

/// Evaluate all enabled label rules against a PR's labels and return matches.
#[tauri::command]
pub fn evaluate_label_rules(
    pr_id: i64,
    state: State<'_, DbState>,
) -> Result<Vec<LabelRuleMatch>, CommandError> {
    let db = state.reader.lock().unwrap();

    // Fetch the PR's labels
    let labels_json: String = db
        .query_row(
            "SELECT labels FROM pull_requests WHERE id = ?1",
            [pr_id],
            |row| row.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                CommandError::NotFound(format!("Pull request with id {}", pr_id))
            }
            other => CommandError::Db(other),
        })?;

    let labels: Vec<String> = serde_json::from_str(&labels_json).unwrap_or_default();

    // Fetch all enabled rules
    let mut stmt = db.prepare(
        "SELECT id, label_pattern, action_type, action_config, enabled, created_at \
         FROM label_automation_rules \
         WHERE enabled = 1 \
         ORDER BY created_at ASC",
    )?;
    let rules: Vec<LabelRule> = stmt
        .query_map([], parse_label_rule)?
        .collect::<Result<Vec<_>, _>>()?;

    let mut matches = Vec::new();
    for rule in rules {
        for label in &labels {
            if pattern_matches(&rule.label_pattern, label) {
                matches.push(LabelRuleMatch {
                    rule: rule.clone(),
                    matched_label: label.clone(),
                });
                break; // One match per rule is sufficient
            }
        }
    }

    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        assert!(pattern_matches("bug", "bug"));
        assert!(pattern_matches("Bug", "bug"));
        assert!(!pattern_matches("bug", "bugfix"));
    }

    #[test]
    fn test_wildcard_prefix() {
        assert!(pattern_matches("*fix", "bugfix"));
        assert!(pattern_matches("*fix", "hotfix"));
        assert!(!pattern_matches("*fix", "fixing"));
    }

    #[test]
    fn test_wildcard_suffix() {
        assert!(pattern_matches("bug*", "bugfix"));
        assert!(pattern_matches("bug*", "bug"));
    }

    #[test]
    fn test_wildcard_middle() {
        assert!(pattern_matches("priority:*", "priority:high"));
        assert!(pattern_matches("priority:*", "priority:low"));
    }

    #[test]
    fn test_star_only() {
        assert!(pattern_matches("*", "anything"));
    }
}
