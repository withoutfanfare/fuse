use rusqlite::Connection;

/// Schema SQL for the Fuse database.
const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS repositories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    owner TEXT NOT NULL,
    name TEXT NOT NULL,
    default_branch TEXT NOT NULL DEFAULT 'main',
    added_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(owner, name)
);

CREATE TABLE IF NOT EXISTS pull_requests (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_id INTEGER NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    number INTEGER NOT NULL,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    state TEXT NOT NULL DEFAULT 'open',
    head_branch TEXT NOT NULL,
    base_branch TEXT NOT NULL,
    additions INTEGER NOT NULL DEFAULT 0,
    deletions INTEGER NOT NULL DEFAULT 0,
    changed_files INTEGER NOT NULL DEFAULT 0,
    review_decision TEXT,
    is_draft INTEGER NOT NULL DEFAULT 0,
    url TEXT NOT NULL,
    labels TEXT NOT NULL DEFAULT '[]',
    mergeable TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    merged_at TEXT,
    closed_at TEXT,
    last_synced_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(repo_id, number)
);

CREATE INDEX IF NOT EXISTS idx_pr_repo_state ON pull_requests(repo_id, state);
CREATE INDEX IF NOT EXISTS idx_pr_updated ON pull_requests(updated_at DESC);

CREATE TABLE IF NOT EXISTS pr_reviews (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pr_id INTEGER NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending','in_progress','reviewed','approved','changes_requested')),
    review_notes TEXT,
    review_file_path TEXT,
    reviewed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(pr_id)
);

CREATE TABLE IF NOT EXISTS review_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_id INTEGER NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    rule_text TEXT NOT NULL,
    position INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_rules_repo ON review_rules(repo_id, position);

CREATE TABLE IF NOT EXISTS sync_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_id INTEGER NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    synced_at TEXT NOT NULL DEFAULT (datetime('now')),
    pr_count INTEGER NOT NULL DEFAULT 0,
    error TEXT
);

CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT OR IGNORE INTO app_settings(key, value) VALUES ('poll_interval_seconds', '300'), ('theme', 'dark');
INSERT OR IGNORE INTO app_settings(key, value) VALUES ('editor_command', 'code');
INSERT OR IGNORE INTO app_settings(key, value) VALUES ('stale_threshold_days', '14');

CREATE TABLE IF NOT EXISTS repo_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    colour TEXT DEFAULT '#ff6b35',
    position INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS repo_group_members (
    group_id INTEGER NOT NULL REFERENCES repo_groups(id) ON DELETE CASCADE,
    repo_id INTEGER NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    PRIMARY KEY (group_id, repo_id)
);

CREATE TABLE IF NOT EXISTS review_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    body TEXT NOT NULL,
    position INTEGER NOT NULL DEFAULT 0
);

INSERT OR IGNORE INTO review_templates (id, name, body, position) VALUES
    (1, 'LGTM', 'Looks good to me!', 0),
    (2, 'Minor Comments', 'A few minor suggestions, but overall looks good. See inline comments.', 1),
    (3, 'Needs Discussion', 'I have some questions about the approach. Let''s discuss before merging.', 2);

CREATE TABLE IF NOT EXISTS pr_ai_reviews (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pr_id INTEGER NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
    review_text TEXT NOT NULL,
    worktree_branch TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_ai_reviews_pr ON pr_ai_reviews(pr_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_pr_author ON pull_requests(author);
CREATE INDEX IF NOT EXISTS idx_pr_state_author ON pull_requests(state, author);
CREATE INDEX IF NOT EXISTS idx_pr_state_updated ON pull_requests(state, updated_at);
CREATE INDEX IF NOT EXISTS idx_pr_created_at ON pull_requests(created_at);
CREATE INDEX IF NOT EXISTS idx_pr_merged_at ON pull_requests(merged_at);
CREATE INDEX IF NOT EXISTS idx_reviews_updated ON pr_reviews(updated_at);
CREATE INDEX IF NOT EXISTS idx_reviews_reviewed ON pr_reviews(reviewed_at);
CREATE INDEX IF NOT EXISTS idx_synclog_synced_at ON sync_log(synced_at DESC);
"#;

/// Run all database migrations. Currently applies the full schema idempotently.
pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(SCHEMA)?;

    // Add body column to pull_requests if it doesn't already exist.
    // SQLite has no IF NOT EXISTS for ALTER TABLE, so we ignore duplicate column errors.
    match conn.execute_batch("ALTER TABLE pull_requests ADD COLUMN body TEXT") {
        Ok(()) => {}
        Err(e) => {
            let msg = e.to_string();
            if !msg.contains("duplicate column") {
                return Err(e);
            }
        }
    }

    // Add review_duration_seconds column to pr_reviews if it doesn't already exist.
    match conn.execute_batch(
        "ALTER TABLE pr_reviews ADD COLUMN review_duration_seconds INTEGER DEFAULT 0",
    ) {
        Ok(()) => {}
        Err(e) => {
            let msg = e.to_string();
            if !msg.contains("duplicate column") {
                return Err(e);
            }
        }
    }

    // Create checklist_state table for persisting per-PR review checklist ticks.
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS checklist_state (
            pr_id INTEGER PRIMARY KEY REFERENCES pull_requests(id) ON DELETE CASCADE,
            state_json TEXT NOT NULL DEFAULT '{}',
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )?;

    // Create label_automation_rules table for label-based automation (Feature 10).
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS label_automation_rules (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            label_pattern TEXT NOT NULL,
            action_type TEXT NOT NULL CHECK(action_type IN ('set_priority', 'add_checklist', 'assign_group')),
            action_config TEXT NOT NULL DEFAULT '{}',
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        "#,
    )?;

    // Create pr_requested_reviewers junction table for reviewer workload tracking (Feature 9).
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS pr_requested_reviewers (
            pr_id INTEGER NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
            reviewer TEXT NOT NULL,
            assigned_at TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (pr_id, reviewer)
        );
        CREATE INDEX IF NOT EXISTS idx_prr_reviewer ON pr_requested_reviewers(reviewer);
        "#,
    )?;

    // Create review_bookmarks table for file-level annotation bookmarks (Feature 8).
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS review_bookmarks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            pr_id INTEGER NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
            file_path TEXT NOT NULL,
            line_start INTEGER,
            line_end INTEGER,
            note TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_bookmarks_pr ON review_bookmarks(pr_id);
        "#,
    )?;

    // Create review_handoffs table for review handoff notes (Feature 11).
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS review_handoffs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            pr_id INTEGER NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
            reviewer_name TEXT NOT NULL,
            files_checked TEXT NOT NULL DEFAULT '[]',
            concerns TEXT NOT NULL DEFAULT '',
            remaining_work TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_handoffs_pr ON review_handoffs(pr_id);
        "#,
    )?;

    // Create pr_dependencies table for cross-PR dependency graph (Feature 2).
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS pr_dependencies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            pr_id INTEGER NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
            depends_on_pr_id INTEGER NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
            dependency_type TEXT NOT NULL CHECK(dependency_type IN ('body_reference', 'branch_ancestry')),
            UNIQUE(pr_id, depends_on_pr_id)
        );
        CREATE INDEX IF NOT EXISTS idx_deps_pr ON pr_dependencies(pr_id);
        CREATE INDEX IF NOT EXISTS idx_deps_target ON pr_dependencies(depends_on_pr_id);
        "#,
    )?;

    // Create review_sessions table for focused review session mode (Feature 4).
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS review_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            pr_id INTEGER NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
            started_at TEXT NOT NULL DEFAULT (datetime('now')),
            files_reviewed TEXT NOT NULL DEFAULT '[]',
            session_notes TEXT,
            status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'paused', 'completed'))
        );
        CREATE INDEX IF NOT EXISTS idx_sessions_pr ON review_sessions(pr_id, started_at DESC);
        "#,
    )?;

    // Add category column to review_bookmarks (Phase 5.5).
    match conn.execute_batch(
        "ALTER TABLE review_bookmarks ADD COLUMN category TEXT NOT NULL DEFAULT 'note'",
    ) {
        Ok(()) => {}
        Err(e) => {
            let msg = e.to_string();
            if !msg.contains("duplicate column") {
                return Err(e);
            }
        }
    }

    // Add resolved column to review_bookmarks (Phase 5.5).
    match conn.execute_batch(
        "ALTER TABLE review_bookmarks ADD COLUMN resolved INTEGER NOT NULL DEFAULT 0",
    ) {
        Ok(()) => {}
        Err(e) => {
            let msg = e.to_string();
            if !msg.contains("duplicate column") {
                return Err(e);
            }
        }
    }

    // Create notification_rules table for configurable PR alert triggers.
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS notification_rules (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            rule_type TEXT NOT NULL CHECK(rule_type IN ('risk_threshold', 'author', 'label', 'review_requested', 'status_change')),
            rule_config TEXT NOT NULL DEFAULT '{}',
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        "#,
    )?;

    // Notification settings defaults
    conn.execute_batch(
        r#"
        INSERT OR IGNORE INTO app_settings(key, value) VALUES ('notifications_enabled', 'true');
        INSERT OR IGNORE INTO app_settings(key, value) VALUES ('quiet_hours_start', '');
        INSERT OR IGNORE INTO app_settings(key, value) VALUES ('quiet_hours_end', '');
        INSERT OR IGNORE INTO app_settings(key, value) VALUES ('notification_risk_threshold', '7');
        "#,
    )?;

    // Create review_time_log table for granular time tracking per session.
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS review_time_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            pr_id INTEGER NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
            started_at TEXT NOT NULL DEFAULT (datetime('now')),
            duration_seconds INTEGER NOT NULL DEFAULT 0,
            UNIQUE(pr_id, started_at)
        );
        CREATE INDEX IF NOT EXISTS idx_review_time_pr ON review_time_log(pr_id);
        "#,
    )?;

    // Add last_delta_sync_at column to repositories for incremental sync.
    match conn.execute_batch(
        "ALTER TABLE repositories ADD COLUMN last_delta_sync_at TEXT",
    ) {
        Ok(()) => {}
        Err(e) => {
            let msg = e.to_string();
            if !msg.contains("duplicate column") {
                return Err(e);
            }
        }
    }

    // Create checklist_templates table for per-repository review checklist templates.
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS checklist_templates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            repo_id INTEGER REFERENCES repositories(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS checklist_template_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            template_id INTEGER NOT NULL REFERENCES checklist_templates(id) ON DELETE CASCADE,
            text TEXT NOT NULL,
            description TEXT,
            position INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_ct_repo ON checklist_templates(repo_id);
        CREATE INDEX IF NOT EXISTS idx_cti_template ON checklist_template_items(template_id, position);
        "#,
    )?;

    // Create filter_presets table for saveable filter combinations.
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS filter_presets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            is_builtin INTEGER NOT NULL DEFAULT 0,
            filter_config TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        "#,
    )?;

    // Insert built-in filter presets
    conn.execute_batch(
        r#"
        INSERT OR IGNORE INTO filter_presets (id, name, is_builtin, filter_config) VALUES
            (1, 'My Reviews', 1, '{"reviewRequested": true}'),
            (2, 'High Risk', 1, '{"minRiskScore": 7}'),
            (3, 'Stale', 1, '{"staleOnly": true, "staleDays": 3}');
        "#,
    )?;

    Ok(())
}
