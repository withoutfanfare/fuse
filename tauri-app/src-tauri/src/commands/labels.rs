use tauri::State;

use crate::db::DbState;
use crate::models::LabelSummary;

use super::CommandError;

/// Return all distinct labels across open PRs with their occurrence counts and colours.
#[tauri::command]
pub fn get_all_labels(
    repo_id: Option<i64>,
    state: State<'_, DbState>,
) -> Result<Vec<LabelSummary>, CommandError> {
    let db = state.reader.lock().unwrap();

    // Build WHERE clause for optional repo filter
    let (where_clause, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match repo_id {
        Some(rid) => (
            "WHERE p.state = 'OPEN' AND p.repo_id = ?1",
            vec![Box::new(rid) as Box<dyn rusqlite::types::ToSql>],
        ),
        None => ("WHERE p.state = 'OPEN'", vec![]),
    };

    let sql = format!(
        r#"SELECT p.labels, p.label_colours FROM pull_requests p {}"#,
        where_clause
    );

    let mut stmt = db.prepare(&sql)?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        let labels_json: String = row.get(0)?;
        let colours_json: String = row.get(1)?;
        Ok((labels_json, colours_json))
    })?;

    // Aggregate label counts and pick the first colour seen for each label
    let mut counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    let mut colours: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    for row in rows {
        let (labels_json, colours_json) = row?;
        let labels: Vec<String> = serde_json::from_str(&labels_json).unwrap_or_default();
        let colour_map: std::collections::HashMap<String, String> =
            serde_json::from_str(&colours_json).unwrap_or_default();

        for label in labels {
            *counts.entry(label.clone()).or_insert(0) += 1;
            if let Some(c) = colour_map.get(&label) {
                colours.entry(label).or_insert_with(|| c.clone());
            }
        }
    }

    let mut result: Vec<LabelSummary> = counts
        .into_iter()
        .map(|(name, count)| LabelSummary {
            color: colours.get(&name).cloned(),
            name,
            count,
        })
        .collect();

    // Sort by count descending, then alphabetically
    result.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));

    Ok(result)
}
