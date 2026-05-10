use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::connection::AppState;
use crate::models::connection::{ConnectionConfig, DatabaseType};
use crate::{schema, types};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableSnapshot {
    pub name: String,
    pub table_type: String,
    pub comment: Option<String>,
    pub columns: Vec<types::ColumnInfo>,
    pub indexes: Vec<types::IndexInfo>,
    pub foreign_keys: Vec<types::ForeignKeyInfo>,
    pub triggers: Vec<types::TriggerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaSnapshot {
    pub connection_id: String,
    pub connection_name: String,
    pub database: Option<String>,
    pub database_type: DatabaseType,
    pub driver_profile: Option<String>,
    pub captured_at: DateTime<Utc>,
    pub databases: Vec<types::DatabaseInfo>,
    pub schemas: Vec<String>,
    pub tables: Vec<TableSnapshot>,
    pub views: Vec<TableSnapshot>,
}

pub async fn snapshot(
    state: &AppState,
    connection_id: &str,
    database: Option<&str>,
    schema_name: Option<&str>,
) -> Result<SchemaSnapshot, String> {
    let config = {
        let configs = state.configs.lock().await;
        configs.get(connection_id).cloned().ok_or("Connection config not found")?
    };

    let db = snapshot_database(&config, database);
    let databases = schema::list_databases_core(state, connection_id).await.unwrap_or_default();
    let schemas = if db.is_empty() {
        Vec::new()
    } else {
        schema::list_schemas_core(state, connection_id, &db).await.unwrap_or_default()
    };
    let effective_schema = schema_name.or_else(|| schemas.first().map(String::as_str)).unwrap_or("");
    let table_infos = if db.is_empty() {
        Vec::new()
    } else {
        schema::list_tables_core(state, connection_id, &db, effective_schema).await.unwrap_or_default()
    };

    let mut tables = Vec::new();
    let mut views = Vec::new();
    for table in table_infos {
        let table_snapshot = TableSnapshot {
            columns: schema::get_columns_core(state, connection_id, &db, effective_schema, &table.name)
                .await
                .unwrap_or_default(),
            indexes: schema::list_indexes_core(state, connection_id, &db, effective_schema, &table.name)
                .await
                .unwrap_or_default(),
            foreign_keys: schema::list_foreign_keys_core(state, connection_id, &db, effective_schema, &table.name)
                .await
                .unwrap_or_default(),
            triggers: schema::list_triggers_core(state, connection_id, &db, effective_schema, &table.name)
                .await
                .unwrap_or_default(),
            name: table.name,
            table_type: table.table_type,
            comment: table.comment,
        };

        if is_view(&table_snapshot.table_type) {
            views.push(table_snapshot);
        } else {
            tables.push(table_snapshot);
        }
    }

    Ok(SchemaSnapshot {
        connection_id: config.id,
        connection_name: config.name,
        database: (!db.is_empty()).then_some(db),
        database_type: config.db_type,
        driver_profile: config.driver_profile,
        captured_at: Utc::now(),
        databases,
        schemas,
        tables,
        views,
    })
}

fn snapshot_database(config: &ConnectionConfig, requested_database: Option<&str>) -> String {
    requested_database
        .or_else(|| config.effective_database())
        .or_else(|| embedded_default_database(&config.db_type))
        .unwrap_or_default()
        .to_string()
}

fn embedded_default_database(db_type: &DatabaseType) -> Option<&'static str> {
    match db_type {
        DatabaseType::Sqlite | DatabaseType::DuckDb => Some("main"),
        _ => None,
    }
}

fn is_view(table_type: &str) -> bool {
    table_type.eq_ignore_ascii_case("view")
}
