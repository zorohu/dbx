use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::extract::{Multipart, Path as AxumPath, State};
use axum::response::sse::{Event, Sse};
use axum::Json;
use dbx_core::query;
use dbx_core::sql;
use dbx_core::sql::SqlFileStatementAction;
use futures::stream::Stream;
use serde::Deserialize;

use crate::error::AppError;
use crate::state::WebState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlFileExecuteRequest {
    pub execution_id: String,
    pub connection_id: String,
    pub database: String,
    pub file_path: String,
    pub continue_on_error: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlFileExecuteWrapper {
    pub request: SqlFileExecuteRequest,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelSqlFileRequest {
    pub execution_id: String,
}

pub async fn preview_sql_file(
    State(state): State<Arc<WebState>>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let tmp_dir = state.data_dir.join("tmp");
    std::fs::create_dir_all(&tmp_dir).map_err(|e| AppError(e.to_string()))?;

    if let Some(field) = multipart.next_field().await.map_err(|e| AppError(e.to_string()))? {
        let file_name = field.file_name().unwrap_or("upload.sql").to_string();
        let data = field.bytes().await.map_err(|e| AppError(e.to_string()))?;

        let file_path = safe_uploaded_sql_path(&tmp_dir, &file_name)?;
        std::fs::write(&file_path, &data).map_err(|e| AppError(e.to_string()))?;

        let size_bytes = data.len() as u64;
        let content = String::from_utf8_lossy(&data);
        let preview: String = content.chars().take(5000).collect();

        return Ok(Json(serde_json::json!({
            "fileName": file_name,
            "filePath": file_path.to_string_lossy(),
            "sizeBytes": size_bytes,
            "preview": preview,
        })));
    }

    Err(AppError("No file uploaded".to_string()))
}

pub async fn execute_sql_file(
    State(state): State<Arc<WebState>>,
    Json(body): Json<SqlFileExecuteWrapper>,
) -> Result<Json<serde_json::Value>, AppError> {
    let req = body.request;
    let execution_id = req.execution_id.clone();

    let (tx, _) = tokio::sync::broadcast::channel::<String>(256);
    state.sse_channels.write().await.insert(execution_id.clone(), tx.clone());

    let app = state.app.clone();
    let state_clone = state.clone();

    let file_path = validated_uploaded_sql_path(&state.data_dir, &req.file_path)?;

    tokio::spawn(async move {
        match std::fs::metadata(&file_path) {
            Ok(meta) if meta.len() > 200 * 1024 * 1024 => {
                let progress = dbx_core::sql::SqlFileProgress {
                    execution_id: req.execution_id.clone(),
                    status: dbx_core::sql::SqlFileStatus::Error,
                    statement_index: 0,
                    success_count: 0,
                    failure_count: 0,
                    affected_rows: 0,
                    elapsed_ms: 0,
                    statement_summary: String::new(),
                    error: Some(format!("File too large: {} bytes (max {} bytes)", meta.len(), 200 * 1024 * 1024)),
                };
                if let Ok(json) = serde_json::to_string(&progress) {
                    let _ = tx.send(json);
                }
                return;
            }
            Err(e) => {
                let progress = dbx_core::sql::SqlFileProgress {
                    execution_id: req.execution_id.clone(),
                    status: dbx_core::sql::SqlFileStatus::Error,
                    statement_index: 0,
                    success_count: 0,
                    failure_count: 0,
                    affected_rows: 0,
                    elapsed_ms: 0,
                    statement_summary: String::new(),
                    error: Some(e.to_string()),
                };
                if let Ok(json) = serde_json::to_string(&progress) {
                    let _ = tx.send(json);
                }
                return;
            }
            _ => {}
        }

        let file_content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => {
                let progress = dbx_core::sql::SqlFileProgress {
                    execution_id: req.execution_id.clone(),
                    status: dbx_core::sql::SqlFileStatus::Error,
                    statement_index: 0,
                    success_count: 0,
                    failure_count: 0,
                    affected_rows: 0,
                    elapsed_ms: 0,
                    statement_summary: String::new(),
                    error: Some(e.to_string()),
                };
                if let Ok(json) = serde_json::to_string(&progress) {
                    let _ = tx.send(json);
                }
                return;
            }
        };

        // Send started
        let started = dbx_core::sql::SqlFileProgress {
            execution_id: req.execution_id.clone(),
            status: dbx_core::sql::SqlFileStatus::Started,
            statement_index: 0,
            success_count: 0,
            failure_count: 0,
            affected_rows: 0,
            elapsed_ms: 0,
            statement_summary: String::new(),
            error: None,
        };
        if let Ok(json) = serde_json::to_string(&started) {
            let _ = tx.send(json);
        }

        let statements = sql::split_sql_statements(&file_content);
        let import_target = {
            let configs = app.configs.read().await;
            configs.get(&req.connection_id).map(|config| (config.db_type, config.driver_profile.clone()))
        };
        let start = std::time::Instant::now();
        let mut success_count = 0usize;
        let mut failure_count = 0usize;
        let mut total_affected: u64 = 0;

        for (i, stmt) in statements.iter().enumerate() {
            let statement_action = import_target
                .as_ref()
                .map(|(db_type, driver_profile)| {
                    sql::prepare_sql_file_statement(stmt, db_type, driver_profile.as_deref())
                })
                .unwrap_or_else(|| SqlFileStatementAction::Execute(stmt.to_string()));
            let (stmt_to_execute, summary, should_execute) = match statement_action {
                SqlFileStatementAction::Execute(statement) => {
                    let summary = sql::statement_summary(&statement);
                    (statement, summary, true)
                }
                SqlFileStatementAction::Skip => (String::new(), sql::statement_summary(stmt), false),
            };

            // Send running
            let running = dbx_core::sql::SqlFileProgress {
                execution_id: req.execution_id.clone(),
                status: dbx_core::sql::SqlFileStatus::Running,
                statement_index: i,
                success_count,
                failure_count,
                affected_rows: total_affected,
                elapsed_ms: start.elapsed().as_millis(),
                statement_summary: summary.clone(),
                error: None,
            };
            if let Ok(json) = serde_json::to_string(&running) {
                let _ = tx.send(json);
            }

            if !should_execute {
                success_count += 1;
                let done = dbx_core::sql::SqlFileProgress {
                    execution_id: req.execution_id.clone(),
                    status: dbx_core::sql::SqlFileStatus::StatementDone,
                    statement_index: i,
                    success_count,
                    failure_count,
                    affected_rows: total_affected,
                    elapsed_ms: start.elapsed().as_millis(),
                    statement_summary: summary,
                    error: None,
                };
                if let Ok(json) = serde_json::to_string(&done) {
                    let _ = tx.send(json);
                }
                continue;
            }

            match query::execute_sql_statement(&app, &req.connection_id, &req.database, &stmt_to_execute, None, None)
                .await
            {
                Ok(result) => {
                    success_count += 1;
                    total_affected += result.affected_rows;
                    let done = dbx_core::sql::SqlFileProgress {
                        execution_id: req.execution_id.clone(),
                        status: dbx_core::sql::SqlFileStatus::StatementDone,
                        statement_index: i,
                        success_count,
                        failure_count,
                        affected_rows: total_affected,
                        elapsed_ms: start.elapsed().as_millis(),
                        statement_summary: summary,
                        error: None,
                    };
                    if let Ok(json) = serde_json::to_string(&done) {
                        let _ = tx.send(json);
                    }
                }
                Err(e) => {
                    failure_count += 1;
                    let failed = dbx_core::sql::SqlFileProgress {
                        execution_id: req.execution_id.clone(),
                        status: dbx_core::sql::SqlFileStatus::StatementFailed,
                        statement_index: i,
                        success_count,
                        failure_count,
                        affected_rows: total_affected,
                        elapsed_ms: start.elapsed().as_millis(),
                        statement_summary: summary,
                        error: Some(e),
                    };
                    if let Ok(json) = serde_json::to_string(&failed) {
                        let _ = tx.send(json);
                    }
                    if !req.continue_on_error {
                        break;
                    }
                }
            }
        }

        // Send final done
        let final_done = dbx_core::sql::SqlFileProgress {
            execution_id: req.execution_id.clone(),
            status: dbx_core::sql::SqlFileStatus::Done,
            statement_index: statements.len(),
            success_count,
            failure_count,
            affected_rows: total_affected,
            elapsed_ms: start.elapsed().as_millis(),
            statement_summary: String::new(),
            error: None,
        };
        if let Ok(json) = serde_json::to_string(&final_done) {
            let _ = tx.send(json);
        }

        state_clone.remove_sse_channel(&req.execution_id).await;
    });

    Ok(Json(serde_json::json!({ "executionId": execution_id })))
}

fn safe_uploaded_sql_path(tmp_dir: &Path, file_name: &str) -> Result<PathBuf, AppError> {
    let base_name =
        file_name.rsplit(|ch| ch == '/' || ch == '\\').find(|part| !part.is_empty()).unwrap_or("upload.sql").trim();
    if base_name.is_empty() || base_name == "." || base_name == ".." {
        return Err(AppError("Invalid SQL file name".to_string()));
    }
    Ok(tmp_dir.join(base_name))
}

fn validated_uploaded_sql_path(data_dir: &Path, file_path: &str) -> Result<PathBuf, AppError> {
    let path = PathBuf::from(file_path);
    if !path.is_absolute() {
        return Err(AppError("File path must be absolute".to_string()));
    }

    let tmp_dir = data_dir.join("tmp").canonicalize().map_err(|e| AppError(e.to_string()))?;
    let canonical_path = path.canonicalize().map_err(|e| AppError(e.to_string()))?;
    if !canonical_path.starts_with(&tmp_dir) {
        return Err(AppError("File path must be inside the uploaded SQL directory".to_string()));
    }
    Ok(canonical_path)
}

pub async fn sql_file_progress(
    State(state): State<Arc<WebState>>,
    AxumPath(execution_id): AxumPath<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>>, AppError> {
    let channels = state.sse_channels.read().await;
    let tx = channels.get(&execution_id).ok_or_else(|| AppError("Execution not found".to_string()))?;
    let rx = tx.subscribe();
    drop(channels);
    Ok(crate::sse::sse_from_channel(rx))
}

pub async fn cancel_sql_file(
    State(state): State<Arc<WebState>>,
    Json(req): Json<CancelSqlFileRequest>,
) -> Json<serde_json::Value> {
    // Remove the channel to stop the execution loop
    state.sse_channels.write().await.remove(&req.execution_id);
    Json(serde_json::json!({ "cancelled": true }))
}

#[cfg(test)]
mod tests {
    use super::{safe_uploaded_sql_path, validated_uploaded_sql_path};

    #[test]
    fn uploaded_sql_path_uses_only_the_file_name() {
        let data_dir = std::env::temp_dir().join(format!("dbx-web-sql-file-test-{}", uuid::Uuid::new_v4()));
        let tmp_dir = data_dir.join("tmp");

        let path = match safe_uploaded_sql_path(&tmp_dir, "../outside.sql") {
            Ok(path) => path,
            Err(error) => panic!("{}", error.0),
        };

        assert_eq!(path, tmp_dir.join("outside.sql"));
        let _ = std::fs::remove_dir_all(data_dir);
    }

    #[test]
    fn execution_path_must_stay_inside_uploaded_tmp_dir() {
        let data_dir = std::env::temp_dir().join(format!("dbx-web-sql-file-test-{}", uuid::Uuid::new_v4()));
        let tmp_dir = data_dir.join("tmp");
        std::fs::create_dir_all(&tmp_dir).unwrap();
        let outside = data_dir.join("outside.sql");
        std::fs::write(&outside, "select 1;").unwrap();

        let result = validated_uploaded_sql_path(&data_dir, &outside.to_string_lossy());

        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(data_dir);
    }
}
