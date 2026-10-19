//! Schema migrations, applied in order at open. The version is kept in SQLite's `user_version`
//! (`rusqlite_migration`). Migrations are append-only: never edit one that has shipped.

use rusqlite_migration::{Migrations, M};

/// Every migration, oldest first.
pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up(include_str!("001_initial.sql"))])
}

#[cfg(test)]
mod tests {
    #[test]
    fn migrations_are_valid() {
        super::migrations().validate().unwrap();
    }
}
