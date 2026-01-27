use tauri_plugin_sql::{Migration, MigrationKind};

pub fn migrations() -> Vec<Migration> {
  vec![
    Migration {
      version: 1,
      description: "create_tables",
      sql: include_str!("./migrations/v1.sql"),
      kind: MigrationKind::Up,
    }
  ]
}