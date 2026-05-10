use std::str::FromStr;

use dbx_core::connection::{AppState, PoolKind};
use dbx_core::models::connection::{default_ssh_connect_timeout_secs, ConnectionConfig, DatabaseType};
use dbx_core::schema_snapshot::snapshot;
use dbx_core::storage::Storage;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

fn sqlite_config(path: &std::path::Path) -> ConnectionConfig {
    ConnectionConfig {
        id: "sqlite-fixture".to_string(),
        name: "SQLite Fixture".to_string(),
        db_type: DatabaseType::Sqlite,
        driver_profile: Some("builtin-sqlite".to_string()),
        driver_label: None,
        url_params: None,
        host: path.display().to_string(),
        port: 0,
        username: String::new(),
        password: String::new(),
        database: None,
        color: None,
        ssh_enabled: false,
        ssh_host: String::new(),
        ssh_port: 22,
        ssh_user: String::new(),
        ssh_password: String::new(),
        ssh_key_path: String::new(),
        ssh_key_passphrase: String::new(),
        ssh_expose_lan: false,
        ssh_connect_timeout_secs: default_ssh_connect_timeout_secs(),
        ssl: false,
        sysdba: false,
        connection_string: None,
        jdbc_driver_class: None,
        jdbc_driver_paths: Vec::new(),
    }
}

async fn create_sqlite_fixture(path: &std::path::Path) {
    let url = format!("sqlite:{}?mode=rwc", path.display());
    let options = SqliteConnectOptions::from_str(&url).unwrap().create_if_missing(true);
    let pool = SqlitePoolOptions::new().max_connections(1).connect_with(options).await.unwrap();

    sqlx::query("PRAGMA foreign_keys = ON").execute(&pool).await.unwrap();
    sqlx::query("CREATE TABLE teams (id INTEGER PRIMARY KEY, name TEXT NOT NULL)").execute(&pool).await.unwrap();
    sqlx::query(
        "CREATE TABLE users (id INTEGER PRIMARY KEY, team_id INTEGER NOT NULL, email TEXT NOT NULL UNIQUE, FOREIGN KEY(team_id) REFERENCES teams(id))",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("CREATE INDEX idx_users_team_id ON users(team_id)").execute(&pool).await.unwrap();
    sqlx::query("CREATE TRIGGER trg_users_ai AFTER INSERT ON users BEGIN SELECT 1; END").execute(&pool).await.unwrap();
    sqlx::query("CREATE VIEW active_users AS SELECT id, email FROM users").execute(&pool).await.unwrap();

    pool.close().await;
}

async fn open_state() -> AppState {
    let storage_path = std::env::temp_dir().join(format!("dbx-schema-snapshot-storage-{}.db", uuid::Uuid::new_v4()));
    AppState::new(Storage::open(&storage_path).await.unwrap())
}

#[tokio::test]
async fn snapshot_standardizes_sqlite_tables_views_and_metadata() {
    let data_path = std::env::temp_dir().join(format!("dbx-schema-snapshot-data-{}.db", uuid::Uuid::new_v4()));
    create_sqlite_fixture(&data_path).await;

    let state = open_state().await;
    let config = sqlite_config(&data_path);
    let pool = dbx_core::db::sqlite::connect_path(&data_path.display().to_string()).await.unwrap();
    state.configs.lock().await.insert(config.id.clone(), config.clone());
    state.connections.lock().await.insert(config.id.clone(), PoolKind::Sqlite(pool));

    let snapshot = snapshot(&state, &config.id, None, None).await.unwrap();

    assert_eq!(snapshot.connection_id, "sqlite-fixture");
    assert_eq!(snapshot.connection_name, "SQLite Fixture");
    assert_eq!(snapshot.database.as_deref(), Some("main"));
    assert_eq!(snapshot.database_type, DatabaseType::Sqlite);
    assert_eq!(snapshot.driver_profile.as_deref(), Some("builtin-sqlite"));
    let now = chrono::Utc::now();
    assert!(snapshot.captured_at <= now);
    assert!(snapshot.captured_at > now - chrono::Duration::seconds(5));
    assert_eq!(snapshot.databases.iter().map(|db| db.name.as_str()).collect::<Vec<_>>(), vec!["main"]);

    let table_names = snapshot.tables.iter().map(|table| table.name.as_str()).collect::<Vec<_>>();
    assert_eq!(table_names, vec!["teams", "users"]);
    let view_names = snapshot.views.iter().map(|view| view.name.as_str()).collect::<Vec<_>>();
    assert_eq!(view_names, vec!["active_users"]);

    let users = snapshot.tables.iter().find(|table| table.name == "users").unwrap();
    assert_eq!(users.table_type, "BASE TABLE");
    assert!(users.columns.iter().any(|column| column.name == "email" && !column.is_nullable));
    assert!(users.indexes.iter().any(|index| index.name == "idx_users_team_id" && index.columns == vec!["team_id"]));
    assert!(users.foreign_keys.iter().any(|fk| fk.column == "team_id" && fk.ref_table == "teams"));
    assert!(users.triggers.iter().any(|trigger| trigger.name == "trg_users_ai" && trigger.event == "INSERT"));
}
