//! Schema migrations, applied in order at open. The version is kept in SQLite's `user_version`
//! (`rusqlite_migration`). Migrations are append-only: never edit one that has shipped.

use rusqlite_migration::{M, Migrations};

/// Every migration, oldest first.
pub(crate) fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(include_str!("001_initial.sql")),
        M::up(include_str!("002_issues.sql")),
        M::up(include_str!("003_auto_export.sql")),
        M::up(include_str!("004_attachment.sql")),
        M::up(include_str!("005_pdf_attempt.sql")),
        M::up(include_str!("006_tags_notes_relations.sql")),
        M::up(include_str!("007_duplicate_dismissal.sql")),
        M::up(include_str!("008_highlight.sql")),
        M::up(include_str!("009_search.sql")),
        M::up(include_str!("010_sync.sql")),
        M::up(include_str!("011_imported.sql")),
        M::up(include_str!("012_reading_saved_search.sql")),
        M::up(include_str!("013_change_log_triggers.sql")),
        M::up(include_str!("014_alert.sql")),
        M::up(include_str!("015_field_clocks.sql")),
        M::up(include_str!("016_sharing.sql")),
        M::up(include_str!("017_local_files.sql")),
        M::up(include_str!("018_auto_export_options.sql")),
    ])
}

#[cfg(test)]
mod tests {
    #[test]
    fn migrations_are_valid() {
        super::migrations().validate().unwrap();
    }
}
