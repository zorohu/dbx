use serde::{Deserialize, Serialize};

use crate::models::connection::DatabaseType;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditableStructureColumn {
    pub id: String,
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    #[serde(default)]
    pub default_value: String,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub is_primary_key: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original: Option<ColumnInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_position: Option<usize>,
    #[serde(default)]
    pub marked_for_drop: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub column_default: Option<String>,
    #[serde(default)]
    pub is_primary_key: bool,
    #[serde(default)]
    pub extra: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditableStructureIndex {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub columns: Vec<String>,
    #[serde(default)]
    pub is_unique: bool,
    #[serde(default)]
    pub is_primary: bool,
    #[serde(default)]
    pub filter: String,
    #[serde(default)]
    pub index_type: String,
    #[serde(default)]
    pub included_columns: Vec<String>,
    #[serde(default)]
    pub comment: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original: Option<IndexInfo>,
    #[serde(default)]
    pub marked_for_drop: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexInfo {
    pub name: String,
    #[serde(default)]
    pub columns: Vec<String>,
    #[serde(default)]
    pub is_unique: bool,
    #[serde(default)]
    pub is_primary: bool,
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub index_type: Option<String>,
    #[serde(default)]
    pub included_columns: Option<Vec<String>>,
    #[serde(default)]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableStructureSqlOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub database_type: Option<DatabaseType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    pub table_name: String,
    #[serde(default)]
    pub columns: Vec<EditableStructureColumn>,
    #[serde(default)]
    pub indexes: Vec<EditableStructureIndex>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableStructureSqlResult {
    pub statements: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StructureDialect {
    Mysql,
    Postgres,
    Sqlite,
    DuckDb,
    SqlServer,
    Oracle,
    H2,
    ClickHouse,
    Unsupported,
}

#[derive(Debug, Clone, Copy)]
struct TableStructureCapabilities {
    dialect: StructureDialect,
    add_column: bool,
    drop_column: bool,
    rename_column: bool,
    alter_existing_column: bool,
    reorder_column: bool,
    comment: bool,
    create_index: bool,
    drop_index: bool,
    rebuild_index: bool,
    index_type: bool,
    index_include: bool,
    index_filter: bool,
    index_comment: bool,
    alter_primary_key: bool,
}

impl Default for TableStructureCapabilities {
    fn default() -> Self {
        Self {
            dialect: StructureDialect::Unsupported,
            add_column: false,
            drop_column: false,
            rename_column: false,
            alter_existing_column: false,
            reorder_column: false,
            comment: false,
            create_index: false,
            drop_index: false,
            rebuild_index: false,
            index_type: false,
            index_include: false,
            index_filter: false,
            index_comment: false,
            alter_primary_key: false,
        }
    }
}

fn capabilities_for(database_type: Option<DatabaseType>) -> TableStructureCapabilities {
    let base = TableStructureCapabilities::default();
    match database_type {
        Some(
            DatabaseType::Mysql
            | DatabaseType::Doris
            | DatabaseType::StarRocks
            | DatabaseType::Goldendb
            | DatabaseType::Sundb,
        ) => TableStructureCapabilities {
            dialect: StructureDialect::Mysql,
            add_column: true,
            drop_column: true,
            rename_column: true,
            alter_existing_column: true,
            reorder_column: true,
            comment: true,
            create_index: true,
            drop_index: true,
            rebuild_index: true,
            index_type: true,
            alter_primary_key: true,
            ..base
        },
        Some(
            DatabaseType::Postgres
            | DatabaseType::Gaussdb
            | DatabaseType::OpenGauss
            | DatabaseType::Highgo
            | DatabaseType::Vastbase
            | DatabaseType::Kingbase,
        ) => TableStructureCapabilities {
            dialect: StructureDialect::Postgres,
            add_column: true,
            drop_column: true,
            rename_column: true,
            alter_existing_column: true,
            comment: true,
            create_index: true,
            drop_index: true,
            rebuild_index: true,
            index_type: true,
            index_include: true,
            index_filter: true,
            index_comment: true,
            alter_primary_key: true,
            ..base
        },
        Some(DatabaseType::Redshift) => TableStructureCapabilities {
            dialect: StructureDialect::Postgres,
            add_column: true,
            drop_column: true,
            rename_column: true,
            alter_existing_column: true,
            comment: true,
            ..base
        },
        Some(DatabaseType::Sqlite) => TableStructureCapabilities {
            dialect: StructureDialect::Sqlite,
            add_column: true,
            drop_column: true,
            rename_column: true,
            create_index: true,
            drop_index: true,
            rebuild_index: true,
            index_filter: true,
            ..base
        },
        Some(DatabaseType::DuckDb) => TableStructureCapabilities {
            dialect: StructureDialect::DuckDb,
            add_column: true,
            drop_column: true,
            rename_column: true,
            create_index: true,
            drop_index: true,
            rebuild_index: true,
            ..base
        },
        Some(DatabaseType::SqlServer) => TableStructureCapabilities {
            dialect: StructureDialect::SqlServer,
            add_column: true,
            drop_column: true,
            create_index: true,
            drop_index: true,
            rebuild_index: true,
            index_type: true,
            index_include: true,
            index_filter: true,
            ..base
        },
        Some(DatabaseType::Oracle | DatabaseType::Dameng | DatabaseType::OceanbaseOracle) => {
            TableStructureCapabilities {
                dialect: StructureDialect::Oracle,
                add_column: true,
                drop_column: true,
                rename_column: true,
                alter_existing_column: true,
                comment: true,
                create_index: true,
                drop_index: true,
                rebuild_index: true,
                index_type: true,
                ..base
            }
        }
        Some(DatabaseType::H2) => TableStructureCapabilities {
            dialect: StructureDialect::H2,
            add_column: true,
            drop_column: true,
            rename_column: true,
            alter_existing_column: true,
            comment: true,
            create_index: true,
            drop_index: true,
            rebuild_index: true,
            ..base
        },
        Some(DatabaseType::ClickHouse) => TableStructureCapabilities {
            dialect: StructureDialect::ClickHouse,
            add_column: true,
            drop_column: true,
            rename_column: true,
            alter_existing_column: true,
            reorder_column: true,
            comment: true,
            ..base
        },
        _ => base,
    }
}

pub fn build_table_structure_change_sql(options: TableStructureSqlOptions) -> TableStructureSqlResult {
    let mut warnings = validate_draft(&options);
    let mut statements = build_column_sql(&options, &mut warnings);
    statements.extend(build_index_sql(&options, &mut warnings));
    TableStructureSqlResult { statements, warnings }
}

pub fn build_create_table_sql(options: TableStructureSqlOptions) -> TableStructureSqlResult {
    let mut warnings = Vec::new();
    if clean(&options.table_name).is_empty() {
        warnings.push("Table name is required.".to_string());
    }
    let active_columns: Vec<_> = options.columns.iter().filter(|column| !column.marked_for_drop).collect();
    if active_columns.is_empty() {
        warnings.push("At least one column is required.".to_string());
    }
    validate_columns(&active_columns, &mut warnings);
    if !warnings.is_empty() {
        return TableStructureSqlResult { statements: Vec::new(), warnings };
    }

    let capabilities = capabilities_for(options.database_type);
    let dialect = capabilities.dialect;
    let table = qualified_table(dialect, options.schema.as_deref(), &options.table_name);
    let mut statements = Vec::new();
    let mut column_definitions = Vec::new();

    for column in &active_columns {
        let data_type = if dialect == StructureDialect::ClickHouse {
            clickhouse_column_type(column)
        } else {
            column.data_type.trim().to_string()
        };
        let mut parts = vec![quote_ident(dialect, &column.name), data_type];
        if !column.is_nullable && !column.is_primary_key && dialect != StructureDialect::ClickHouse {
            parts.push("NOT NULL".to_string());
        }
        let default_value = normalize_default(Some(&column.default_value));
        if !default_value.is_empty() {
            parts.push(format!("DEFAULT {default_value}"));
        }
        if dialect == StructureDialect::Mysql && capabilities.comment && !clean(&column.comment).is_empty() {
            parts.push(format!("COMMENT {}", quote_string(&clean(&column.comment))));
        }
        column_definitions.push(parts.join(" "));
    }

    let pk_columns: Vec<_> = active_columns.iter().filter(|column| column.is_primary_key).collect();
    if !pk_columns.is_empty() {
        let pk_list = pk_columns.iter().map(|column| quote_ident(dialect, &column.name)).collect::<Vec<_>>().join(", ");
        column_definitions.push(format!("PRIMARY KEY ({pk_list})"));
    }

    statements.push(format!("CREATE TABLE {table} (\n  {}\n);", column_definitions.join(",\n  ")));

    if capabilities.comment
        && matches!(dialect, StructureDialect::Postgres | StructureDialect::Oracle | StructureDialect::H2)
    {
        for column in &active_columns {
            if !clean(&column.comment).is_empty() {
                statements.push(format!(
                    "COMMENT ON COLUMN {table}.{} IS {};",
                    quote_ident(dialect, &column.name),
                    quote_string(&clean(&column.comment))
                ));
            }
        }
    }
    if capabilities.comment && dialect == StructureDialect::ClickHouse {
        for column in &active_columns {
            if !clean(&column.comment).is_empty() {
                statements.push(format!(
                    "ALTER TABLE {table} COMMENT COLUMN {} {};",
                    quote_ident(dialect, &column.name),
                    quote_string(&clean(&column.comment))
                ));
            }
        }
    }

    for index in options.indexes.iter().filter(|index| !index.marked_for_drop && !index.is_primary) {
        if !capabilities.create_index {
            warnings.push(format!(
                "Creating indexes is not supported for {} from this editor.",
                database_label(options.database_type)
            ));
            continue;
        }
        statements.extend(build_create_index_statements(dialect, &table, index, &mut warnings));
    }

    TableStructureSqlResult { statements, warnings }
}

fn build_column_sql(options: &TableStructureSqlOptions, warnings: &mut Vec<String>) -> Vec<String> {
    let capabilities = capabilities_for(options.database_type);
    let dialect = capabilities.dialect;
    let table = qualified_table(dialect, options.schema.as_deref(), &options.table_name);
    let database_label = database_label(options.database_type);
    let active_columns: Vec<_> = options.columns.iter().filter(|column| !column.marked_for_drop).collect();
    let has_original_column_positions = active_columns.iter().any(|column| column.original_position.is_some());
    let mut statements = Vec::new();

    for column in &options.columns {
        if column.marked_for_drop {
            let Some(original) = &column.original else {
                continue;
            };
            if !capabilities.drop_column {
                warnings.push(format!("Dropping columns is not supported for {database_label} from this editor."));
                continue;
            }
            if original.is_primary_key {
                warnings.push(format!("Primary key column \"{}\" cannot be dropped from this editor.", original.name));
                continue;
            }
            statements.push(format!("ALTER TABLE {table} DROP COLUMN {};", quote_ident(dialect, &original.name)));
            continue;
        }

        let active_index = active_columns.iter().position(|active| active.id == column.id).unwrap_or(0);
        let position_clause = if has_original_column_positions {
            column_position_clause(dialect, &active_columns, active_index)
        } else {
            String::new()
        };
        let has_position_change = has_original_column_positions
            && matches!(dialect, StructureDialect::Mysql | StructureDialect::ClickHouse)
            && column.original.is_some()
            && mysql_column_position_changed(&active_columns, active_index);

        if column.original.is_none() {
            if !capabilities.add_column {
                warnings.push(format!("Adding columns is not supported for {database_label} from this editor."));
                continue;
            }
            statements.extend(build_add_column_sql(dialect, &table, column, &position_clause));
            continue;
        }

        if !has_existing_column_attribute_change(column) && !has_position_change {
            continue;
        }
        let original = column.original.as_ref().unwrap();
        let has_rename = column.name != original.name;
        let has_attribute_change = column.data_type.trim() != original.data_type.trim()
            || column.is_nullable != original.is_nullable
            || normalize_default(Some(&column.default_value)) != original_default(column)
            || clean(&column.comment) != original_comment(column);
        if has_position_change && !capabilities.reorder_column {
            warnings.push(format!("Reordering columns is not supported for {database_label} from this editor."));
        }
        if has_rename && !capabilities.rename_column {
            warnings.push(format!("Renaming columns is not supported for {database_label} from this editor."));
        }
        if has_attribute_change && !capabilities.alter_existing_column && dialect != StructureDialect::Sqlite {
            warnings.push(format!("Editing existing columns is not supported for {database_label} yet."));
        }
        if (has_position_change && !capabilities.reorder_column)
            || (has_rename && !capabilities.rename_column)
            || (has_attribute_change && !capabilities.alter_existing_column && dialect != StructureDialect::Sqlite)
        {
            continue;
        }

        match dialect {
            StructureDialect::Mysql => statements.extend(build_mysql_existing_column_sql(
                &table,
                column,
                if has_position_change { &position_clause } else { "" },
            )),
            StructureDialect::Postgres => statements.extend(build_postgres_existing_column_sql(&table, column)),
            StructureDialect::Oracle => {
                statements.extend(build_oracle_like_existing_column_sql(dialect, &table, column))
            }
            StructureDialect::H2 => statements.extend(build_h2_existing_column_sql(&table, column)),
            StructureDialect::ClickHouse => statements.extend(build_clickhouse_existing_column_sql(
                &table,
                column,
                if has_position_change { &position_clause } else { "" },
            )),
            StructureDialect::Sqlite => statements.extend(build_sqlite_existing_column_sql(&table, column, warnings)),
            _ => warnings.push(format!("Editing existing columns is not supported for {database_label} yet.")),
        }
    }

    // Emit primary key constraint changes after individual column changes
    statements.extend(build_primary_key_sql(options, dialect, &table, warnings));

    statements
}

fn build_primary_key_sql(
    options: &TableStructureSqlOptions,
    dialect: StructureDialect,
    table: &str,
    warnings: &mut Vec<String>,
) -> Vec<String> {
    let capabilities = capabilities_for(options.database_type);

    let old_pk_names: Vec<&str> = options
        .columns
        .iter()
        .filter(|c| c.original.as_ref().is_some_and(|o| o.is_primary_key))
        .map(|c| c.name.as_str())
        .collect();

    let new_pk_names: Vec<&str> =
        options.columns.iter().filter(|c| !c.marked_for_drop && c.is_primary_key).map(|c| c.name.as_str()).collect();

    if old_pk_names == new_pk_names {
        return Vec::new();
    }

    if !capabilities.alter_primary_key {
        warnings.push(format!(
            "Changing primary keys is not supported for {} from this editor.",
            database_label(options.database_type)
        ));
        return Vec::new();
    }

    let mut statements = Vec::new();

    if !old_pk_names.is_empty() {
        match dialect {
            StructureDialect::Postgres => {
                let raw_table = options.table_name.split('.').last().unwrap_or(&options.table_name);
                let pk_name = format!("{}_pkey", clean(raw_table));
                statements.push(format!("ALTER TABLE {table} DROP CONSTRAINT {};", quote_ident(dialect, &pk_name)));
            }
            StructureDialect::Mysql => {
                statements.push(format!("ALTER TABLE {table} DROP PRIMARY KEY;"));
            }
            _ => {}
        }
    }

    if !new_pk_names.is_empty() {
        let pk_list = new_pk_names.iter().map(|n| quote_ident(dialect, n)).collect::<Vec<_>>().join(", ");
        statements.push(format!("ALTER TABLE {table} ADD PRIMARY KEY ({pk_list});"));
    }

    statements
}

fn build_index_sql(options: &TableStructureSqlOptions, warnings: &mut Vec<String>) -> Vec<String> {
    let capabilities = capabilities_for(options.database_type);
    let dialect = capabilities.dialect;
    let table = qualified_table(dialect, options.schema.as_deref(), &options.table_name);
    let database_label = database_label(options.database_type);
    let mut statements = Vec::new();

    for index in &options.indexes {
        if index.marked_for_drop {
            let Some(original) = &index.original else {
                continue;
            };
            if !capabilities.drop_index {
                warnings.push(format!("Dropping indexes is not supported for {database_label} from this editor."));
                continue;
            }
            if original.is_primary {
                warnings.push(format!("Primary index \"{}\" cannot be dropped from this editor.", original.name));
                continue;
            }
            statements.push(build_drop_index_sql(dialect, &table, options.schema.as_deref(), &original.name));
            continue;
        }

        if let Some(original) = &index.original {
            if !has_existing_index_change(index) {
                continue;
            }
            if !capabilities.rebuild_index || !capabilities.drop_index || !capabilities.create_index {
                warnings
                    .push(format!("Editing existing indexes is not supported for {database_label} from this editor."));
                continue;
            }
            if original.is_primary {
                warnings.push(format!("Primary index \"{}\" cannot be edited from this editor.", original.name));
                continue;
            }
            statements.push(build_drop_index_sql(dialect, &table, options.schema.as_deref(), &original.name));
            statements.extend(build_create_index_statements(dialect, &table, index, warnings));
            continue;
        }

        if !capabilities.create_index {
            warnings.push(format!("Creating indexes is not supported for {database_label} from this editor."));
            continue;
        }
        statements.extend(build_create_index_statements(dialect, &table, index, warnings));
    }

    statements
}

fn build_add_column_sql(
    dialect: StructureDialect,
    table: &str,
    column: &EditableStructureColumn,
    position_clause: &str,
) -> Vec<String> {
    let add_keyword = if dialect == StructureDialect::SqlServer { "ADD" } else { "ADD COLUMN" };
    let definition = column_definition(dialect, column);
    let mut statements = if dialect == StructureDialect::Oracle {
        vec![format!("ALTER TABLE {table} ADD ({definition});")]
    } else {
        vec![format!("ALTER TABLE {table} {add_keyword} {definition}{position_clause};")]
    };
    if matches!(dialect, StructureDialect::Postgres | StructureDialect::Oracle) && !clean(&column.comment).is_empty() {
        statements.push(format!(
            "COMMENT ON COLUMN {table}.{} IS {};",
            quote_ident(dialect, &column.name),
            quote_string(&clean(&column.comment))
        ));
    }
    if dialect == StructureDialect::ClickHouse && !clean(&column.comment).is_empty() {
        statements.push(format!(
            "ALTER TABLE {table} COMMENT COLUMN {} {};",
            quote_ident(dialect, &column.name),
            quote_string(&clean(&column.comment))
        ));
    }
    statements
}

fn build_mysql_existing_column_sql(
    table: &str,
    column: &EditableStructureColumn,
    position_clause: &str,
) -> Vec<String> {
    let original_name = column.original.as_ref().map(|original| original.name.as_str()).unwrap_or(&column.name);
    let operation = if column.name == original_name {
        format!("MODIFY COLUMN {}", column_definition(StructureDialect::Mysql, column))
    } else {
        format!(
            "CHANGE COLUMN {} {}",
            quote_ident(StructureDialect::Mysql, original_name),
            column_definition(StructureDialect::Mysql, column)
        )
    };
    vec![format!("ALTER TABLE {table} {operation}{position_clause};")]
}

fn build_postgres_existing_column_sql(table: &str, column: &EditableStructureColumn) -> Vec<String> {
    let Some(original) = &column.original else {
        return Vec::new();
    };
    let mut statements = Vec::new();
    let current_name = &column.name;
    if column.name != original.name {
        statements.push(format!(
            "ALTER TABLE {table} RENAME COLUMN {} TO {};",
            quote_ident(StructureDialect::Postgres, &original.name),
            quote_ident(StructureDialect::Postgres, &column.name)
        ));
    }
    if column.data_type.trim() != original.data_type.trim() {
        statements.push(format!(
            "ALTER TABLE {table} ALTER COLUMN {} TYPE {};",
            quote_ident(StructureDialect::Postgres, current_name),
            column.data_type.trim()
        ));
    }
    if column.is_nullable != original.is_nullable {
        let action = if column.is_nullable { "DROP NOT NULL" } else { "SET NOT NULL" };
        statements.push(format!(
            "ALTER TABLE {table} ALTER COLUMN {} {action};",
            quote_ident(StructureDialect::Postgres, current_name)
        ));
    }
    if normalize_default(Some(&column.default_value)) != original_default(column) {
        let default_value = normalize_default(Some(&column.default_value));
        let action =
            if default_value.is_empty() { "DROP DEFAULT".to_string() } else { format!("SET DEFAULT {default_value}") };
        statements.push(format!(
            "ALTER TABLE {table} ALTER COLUMN {} {action};",
            quote_ident(StructureDialect::Postgres, current_name)
        ));
    }
    if clean(&column.comment) != original_comment(column) {
        let comment_value =
            if clean(&column.comment).is_empty() { "NULL".to_string() } else { quote_string(&clean(&column.comment)) };
        statements.push(format!(
            "COMMENT ON COLUMN {table}.{} IS {comment_value};",
            quote_ident(StructureDialect::Postgres, current_name)
        ));
    }
    statements
}

fn build_oracle_like_existing_column_sql(
    dialect: StructureDialect,
    table: &str,
    column: &EditableStructureColumn,
) -> Vec<String> {
    let Some(original) = &column.original else {
        return Vec::new();
    };
    let mut statements = Vec::new();
    let mut current_name = original.name.clone();
    if column.name != original.name {
        statements.push(format!(
            "ALTER TABLE {table} RENAME COLUMN {} TO {};",
            quote_ident(dialect, &original.name),
            quote_ident(dialect, &column.name)
        ));
        current_name = column.name.clone();
    }
    if column.data_type.trim() != original.data_type.trim() {
        statements.push(format!(
            "ALTER TABLE {table} MODIFY ({} {});",
            quote_ident(dialect, &current_name),
            column.data_type.trim()
        ));
    }
    if column.is_nullable != original.is_nullable {
        let nullability = if column.is_nullable { "NULL" } else { "NOT NULL" };
        statements.push(format!("ALTER TABLE {table} MODIFY ({} {nullability});", quote_ident(dialect, &current_name)));
    }
    if normalize_default(Some(&column.default_value)) != original_default(column) {
        let default_value = normalize_default(Some(&column.default_value));
        let default_value = if default_value.is_empty() { "NULL".to_string() } else { default_value };
        statements.push(format!(
            "ALTER TABLE {table} MODIFY ({} DEFAULT {default_value});",
            quote_ident(dialect, &current_name)
        ));
    }
    if clean(&column.comment) != original_comment(column) {
        let comment_value =
            if clean(&column.comment).is_empty() { "NULL".to_string() } else { quote_string(&clean(&column.comment)) };
        statements
            .push(format!("COMMENT ON COLUMN {table}.{} IS {comment_value};", quote_ident(dialect, &current_name)));
    }
    statements
}

fn build_h2_existing_column_sql(table: &str, column: &EditableStructureColumn) -> Vec<String> {
    let Some(original) = &column.original else {
        return Vec::new();
    };
    let mut statements = Vec::new();
    let mut current_name = original.name.clone();
    if column.name != original.name {
        statements.push(format!(
            "ALTER TABLE {table} ALTER COLUMN {} RENAME TO {};",
            quote_ident(StructureDialect::H2, &original.name),
            quote_ident(StructureDialect::H2, &column.name)
        ));
        current_name = column.name.clone();
    }
    if column.data_type.trim() != original.data_type.trim() {
        statements.push(format!(
            "ALTER TABLE {table} ALTER COLUMN {} SET DATA TYPE {};",
            quote_ident(StructureDialect::H2, &current_name),
            column.data_type.trim()
        ));
    }
    if column.is_nullable != original.is_nullable {
        let action = if column.is_nullable { "DROP NOT NULL" } else { "SET NOT NULL" };
        statements.push(format!(
            "ALTER TABLE {table} ALTER COLUMN {} {action};",
            quote_ident(StructureDialect::H2, &current_name)
        ));
    }
    if normalize_default(Some(&column.default_value)) != original_default(column) {
        let default_value = normalize_default(Some(&column.default_value));
        let action =
            if default_value.is_empty() { "DROP DEFAULT".to_string() } else { format!("SET DEFAULT {default_value}") };
        statements.push(format!(
            "ALTER TABLE {table} ALTER COLUMN {} {action};",
            quote_ident(StructureDialect::H2, &current_name)
        ));
    }
    if clean(&column.comment) != original_comment(column) {
        let comment_value =
            if clean(&column.comment).is_empty() { "NULL".to_string() } else { quote_string(&clean(&column.comment)) };
        statements.push(format!(
            "COMMENT ON COLUMN {table}.{} IS {comment_value};",
            quote_ident(StructureDialect::H2, &current_name)
        ));
    }
    statements
}

fn build_clickhouse_existing_column_sql(
    table: &str,
    column: &EditableStructureColumn,
    position_clause: &str,
) -> Vec<String> {
    let Some(original) = &column.original else {
        return Vec::new();
    };
    let mut statements = Vec::new();
    let mut current_name = original.name.clone();
    if column.name != original.name {
        statements.push(format!(
            "ALTER TABLE {table} RENAME COLUMN {} TO {};",
            quote_ident(StructureDialect::ClickHouse, &original.name),
            quote_ident(StructureDialect::ClickHouse, &column.name)
        ));
        current_name = column.name.clone();
    }
    let next_type = clickhouse_column_type(column);
    if next_type != original.data_type.trim()
        || normalize_default(Some(&column.default_value)) != original_default(column)
    {
        let default_value = normalize_default(Some(&column.default_value));
        if !default_value.is_empty() {
            statements.push(format!(
                "ALTER TABLE {table} MODIFY COLUMN {} {next_type} DEFAULT {default_value}{position_clause};",
                quote_ident(StructureDialect::ClickHouse, &current_name)
            ));
        } else if !original_default(column).is_empty() {
            statements.push(format!(
                "ALTER TABLE {table} MODIFY COLUMN {} REMOVE DEFAULT;",
                quote_ident(StructureDialect::ClickHouse, &current_name)
            ));
            if next_type != original.data_type.trim() || !position_clause.is_empty() {
                statements.push(format!(
                    "ALTER TABLE {table} MODIFY COLUMN {} {next_type}{position_clause};",
                    quote_ident(StructureDialect::ClickHouse, &current_name)
                ));
            }
        } else {
            statements.push(format!(
                "ALTER TABLE {table} MODIFY COLUMN {} {next_type}{position_clause};",
                quote_ident(StructureDialect::ClickHouse, &current_name)
            ));
        }
    } else if !position_clause.is_empty() {
        statements.push(format!(
            "ALTER TABLE {table} MODIFY COLUMN {} {next_type}{position_clause};",
            quote_ident(StructureDialect::ClickHouse, &current_name)
        ));
    }
    if clean(&column.comment) != original_comment(column) {
        statements.push(format!(
            "ALTER TABLE {table} COMMENT COLUMN {} {};",
            quote_ident(StructureDialect::ClickHouse, &current_name),
            quote_string(&clean(&column.comment))
        ));
    }
    statements
}

fn build_sqlite_existing_column_sql(
    table: &str,
    column: &EditableStructureColumn,
    warnings: &mut Vec<String>,
) -> Vec<String> {
    let Some(original) = &column.original else {
        return Vec::new();
    };
    let mut statements = Vec::new();
    let unsupported_change = column.data_type.trim() != original.data_type.trim()
        || column.is_nullable != original.is_nullable
        || normalize_default(Some(&column.default_value)) != original_default(column)
        || clean(&column.comment) != original_comment(column);
    if column.name != original.name {
        statements.push(format!(
            "ALTER TABLE {table} RENAME COLUMN {} TO {};",
            quote_ident(StructureDialect::Sqlite, &original.name),
            quote_ident(StructureDialect::Sqlite, &column.name)
        ));
    }
    if unsupported_change {
        warnings.push(format!(
            "SQLite cannot safely alter existing column \"{}\" without rebuilding the table.",
            original.name
        ));
    }
    statements
}

fn build_drop_index_sql(dialect: StructureDialect, table: &str, schema: Option<&str>, index_name: &str) -> String {
    if matches!(dialect, StructureDialect::Mysql | StructureDialect::SqlServer) {
        return format!("DROP INDEX {} ON {table};", quote_ident(dialect, index_name));
    }
    if matches!(dialect, StructureDialect::Postgres | StructureDialect::Oracle)
        && schema.is_some_and(|schema| !schema.trim().is_empty())
    {
        return format!("DROP INDEX {}.{};", quote_ident(dialect, schema.unwrap()), quote_ident(dialect, index_name));
    }
    format!("DROP INDEX {};", quote_ident(dialect, index_name))
}

fn build_create_index_statements(
    dialect: StructureDialect,
    table: &str,
    index: &EditableStructureIndex,
    warnings: &mut Vec<String>,
) -> Vec<String> {
    let capabilities = capabilities_for(database_type_for_dialect(dialect));
    let name = clean(&index.name);
    let columns: Vec<String> =
        index.columns.iter().map(|column| clean(column)).filter(|column| !column.is_empty()).collect();
    if name.is_empty() || columns.is_empty() {
        return Vec::new();
    }

    let unique = if index.is_unique { "UNIQUE " } else { "" };
    let cols = columns.iter().map(|column| quote_ident(dialect, column)).collect::<Vec<_>>().join(", ");
    let idx_type = normalized_index_type(index);
    let mut type_prefix = String::new();
    let mut using_clause = String::new();

    if !idx_type.is_empty() && capabilities.index_type {
        match dialect {
            StructureDialect::Postgres => using_clause = format!(" USING {idx_type}"),
            StructureDialect::SqlServer => type_prefix = format!("{idx_type} "),
            StructureDialect::Mysql => {
                let (prefix, using) = mysql_index_parts(&idx_type);
                type_prefix = prefix;
                using_clause = using;
            }
            StructureDialect::Oracle if idx_type == "BITMAP" => type_prefix = "BITMAP ".to_string(),
            _ => {}
        }
    }

    let included_columns: Vec<String> =
        index.included_columns.iter().map(|column| clean(column)).filter(|column| !column.is_empty()).collect();
    let include_clause = if !included_columns.is_empty()
        && capabilities.index_include
        && matches!(dialect, StructureDialect::Postgres | StructureDialect::SqlServer)
    {
        format!(
            " INCLUDE ({})",
            included_columns.iter().map(|column| quote_ident(dialect, column)).collect::<Vec<_>>().join(", ")
        )
    } else {
        String::new()
    };
    let filter = clean(&index.filter);
    let supports_where = capabilities.index_filter
        && matches!(dialect, StructureDialect::Postgres | StructureDialect::SqlServer | StructureDialect::Sqlite);
    let where_clause = if !filter.is_empty() && supports_where { format!(" WHERE {filter}") } else { String::new() };
    let create_sql = if dialect == StructureDialect::Postgres {
        format!(
            "CREATE {unique}{type_prefix}INDEX {} ON {table}{using_clause} ({cols}){include_clause}{where_clause};",
            quote_ident(dialect, &name)
        )
    } else {
        format!(
            "CREATE {unique}{type_prefix}INDEX {}{using_clause} ON {table} ({cols}){include_clause}{where_clause};",
            quote_ident(dialect, &name)
        )
    };
    let mut statements = vec![create_sql];

    let comment = clean(&index.comment);
    if !comment.is_empty() && capabilities.index_comment && dialect == StructureDialect::Postgres {
        statements.push(format!("COMMENT ON INDEX {} IS {};", quote_ident(dialect, &name), quote_string(&comment)));
    } else if !comment.is_empty() && capabilities.index_comment {
        warnings.push(format!("Index comments are not supported for {} from this editor.", dialect_label(dialect)));
    }
    statements
}

fn validate_draft(options: &TableStructureSqlOptions) -> Vec<String> {
    let mut warnings = Vec::new();
    let active_columns: Vec<_> = options.columns.iter().filter(|column| !column.marked_for_drop).collect();
    validate_columns(&active_columns, &mut warnings);
    for index in options
        .indexes
        .iter()
        .filter(|index| !index.marked_for_drop && (index.original.is_none() || has_existing_index_change(index)))
    {
        if clean(&index.name).is_empty() {
            warnings.push("Index name cannot be empty.".to_string());
        }
        if index.columns.iter().map(|column| clean(column)).filter(|column| !column.is_empty()).count() == 0 {
            warnings.push(format!(
                "Index \"{}\" needs at least one column.",
                if index.name.is_empty() { "(new)" } else { &index.name }
            ));
        }
    }
    warnings
}

fn validate_columns(columns: &[&EditableStructureColumn], warnings: &mut Vec<String>) {
    let mut names = std::collections::HashSet::new();
    for column in columns {
        if clean(&column.name).is_empty() {
            warnings.push("Column name cannot be empty.".to_string());
        }
        if clean(&column.data_type).is_empty() {
            warnings.push(format!(
                "Column \"{}\" type cannot be empty.",
                if column.name.is_empty() { "(new)" } else { &column.name }
            ));
        }
        let key = clean(&column.name).to_lowercase();
        if !key.is_empty() && !names.insert(key) {
            warnings.push(format!("Column \"{}\" is duplicated.", column.name));
        }
    }
}

fn column_definition(dialect: StructureDialect, column: &EditableStructureColumn) -> String {
    let data_type = if dialect == StructureDialect::ClickHouse {
        clickhouse_column_type(column)
    } else {
        column.data_type.trim().to_string()
    };
    let mut parts = vec![quote_ident(dialect, &column.name), data_type];
    if !column.is_nullable && !is_oracle_like(dialect) && dialect != StructureDialect::ClickHouse {
        parts.push("NOT NULL".to_string());
    }
    let default_value = normalize_default(Some(&column.default_value));
    if !default_value.is_empty() {
        parts.push(format!("DEFAULT {default_value}"));
    }
    if dialect == StructureDialect::Mysql && !clean(&column.comment).is_empty() {
        parts.push(format!("COMMENT {}", quote_string(&clean(&column.comment))));
    }
    parts.join(" ")
}

fn clickhouse_column_type(column: &EditableStructureColumn) -> String {
    let data_type = column.data_type.trim();
    if column.is_nullable {
        if data_type.to_ascii_lowercase().starts_with("nullable") {
            data_type.to_string()
        } else {
            format!("Nullable({data_type})")
        }
    } else {
        unwrap_clickhouse_nullable_type(data_type)
    }
}

fn unwrap_clickhouse_nullable_type(data_type: &str) -> String {
    let trimmed = data_type.trim();
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("nullable(") && trimmed.ends_with(')') {
        trimmed[trimmed.find('(').unwrap_or(0) + 1..trimmed.len() - 1].trim().to_string()
    } else {
        trimmed.to_string()
    }
}

fn column_position_clause(dialect: StructureDialect, columns: &[&EditableStructureColumn], index: usize) -> String {
    if !matches!(dialect, StructureDialect::Mysql | StructureDialect::ClickHouse) {
        return String::new();
    }
    if index == 0 {
        return " FIRST".to_string();
    }
    format!(" AFTER {}", quote_ident(dialect, &columns.get(index - 1).map(|column| column.name.as_str()).unwrap_or("")))
}

fn mysql_column_position_changed(columns: &[&EditableStructureColumn], index: usize) -> bool {
    let Some(column) = columns.get(index) else {
        return false;
    };
    if column.original.is_none() || column.original_position.is_none() {
        return false;
    }
    current_previous_original_column_name(columns, index) != original_previous_column_name(columns, column)
}

fn original_previous_column_name(
    columns: &[&EditableStructureColumn],
    column: &EditableStructureColumn,
) -> Option<String> {
    let mut original_columns: Vec<_> =
        columns.iter().filter(|item| item.original.is_some() && item.original_position.is_some()).copied().collect();
    original_columns.sort_by_key(|item| item.original_position.unwrap_or(0));
    let index = original_columns.iter().position(|item| item.id == column.id)?;
    if index == 0 {
        None
    } else {
        original_columns[index - 1].original.as_ref().map(|original| original.name.clone())
    }
}

fn current_previous_original_column_name(columns: &[&EditableStructureColumn], index: usize) -> Option<String> {
    if index == 0 {
        None
    } else {
        columns.get(index - 1).map(|column| {
            column.original.as_ref().map(|original| original.name.clone()).unwrap_or_else(|| column.name.clone())
        })
    }
}

fn has_existing_column_attribute_change(column: &EditableStructureColumn) -> bool {
    let Some(original) = &column.original else {
        return false;
    };
    column.name != original.name
        || column.data_type.trim() != original.data_type.trim()
        || column.is_nullable != original.is_nullable
        || normalize_default(Some(&column.default_value)) != original_default(column)
        || clean(&column.comment) != original_comment(column)
}

fn has_existing_index_change(index: &EditableStructureIndex) -> bool {
    let Some(original) = &index.original else {
        return false;
    };
    clean(&index.name) != clean(&original.name)
        || index_list_changed(&index.columns, Some(&original.columns))
        || index.is_unique != original.is_unique
        || normalized_index_type(index) != clean(original.index_type.as_deref().unwrap_or("")).to_ascii_uppercase()
        || index_list_changed(&index.included_columns, original.included_columns.as_ref())
        || clean(&index.filter) != clean(original.filter.as_deref().unwrap_or(""))
        || clean(&index.comment) != clean(original.comment.as_deref().unwrap_or(""))
}

fn index_list_changed(next: &[String], previous: Option<&Vec<String>>) -> bool {
    let next_clean: Vec<_> = next.iter().map(|value| clean(value)).filter(|value| !value.is_empty()).collect();
    let previous_clean: Vec<_> =
        previous.unwrap_or(&Vec::new()).iter().map(|value| clean(value)).filter(|value| !value.is_empty()).collect();
    next_clean.len() != previous_clean.len()
        || next_clean.iter().enumerate().any(|(index, value)| previous_clean.get(index) != Some(value))
}

fn normalized_index_type(index: &EditableStructureIndex) -> String {
    clean(&index.index_type).to_ascii_uppercase()
}

fn mysql_index_parts(index_type: &str) -> (String, String) {
    match index_type.to_ascii_uppercase().as_str() {
        "FULLTEXT" | "SPATIAL" => (format!("{} ", index_type.to_ascii_uppercase()), String::new()),
        "RTREE" => ("SPATIAL ".to_string(), String::new()),
        "BTREE" | "HASH" => (String::new(), format!(" USING {}", index_type.to_ascii_uppercase())),
        _ => (String::new(), String::new()),
    }
}

fn qualified_table(dialect: StructureDialect, schema: Option<&str>, table_name: &str) -> String {
    if matches!(
        dialect,
        StructureDialect::Postgres | StructureDialect::Oracle | StructureDialect::SqlServer | StructureDialect::H2
    ) && schema.is_some_and(|schema| !schema.trim().is_empty())
    {
        return format!("{}.{}", quote_ident(dialect, schema.unwrap()), quote_ident(dialect, table_name));
    }
    quote_ident(dialect, table_name)
}

fn quote_ident(dialect: StructureDialect, name: &str) -> String {
    match dialect {
        StructureDialect::Mysql => format!("`{}`", name.replace('`', "``")),
        StructureDialect::SqlServer => format!("[{}]", name.replace(']', "]]")),
        _ => format!("\"{}\"", name.replace('"', "\"\"")),
    }
}

fn quote_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn clean(value: &str) -> String {
    value.trim().to_string()
}

fn normalize_default(value: Option<&String>) -> String {
    let trimmed = value.map(|value| value.trim()).unwrap_or("");
    if trimmed.eq_ignore_ascii_case("null") {
        String::new()
    } else {
        trimmed.to_string()
    }
}

fn original_default(column: &EditableStructureColumn) -> String {
    normalize_default(column.original.as_ref().and_then(|original| original.column_default.as_ref()))
}

fn original_comment(column: &EditableStructureColumn) -> String {
    clean(column.original.as_ref().and_then(|original| original.comment.as_deref()).unwrap_or(""))
}

fn is_oracle_like(dialect: StructureDialect) -> bool {
    dialect == StructureDialect::Oracle
}

fn database_label(database_type: Option<DatabaseType>) -> String {
    database_type
        .map(|database_type| {
            serde_json::to_value(database_type)
                .ok()
                .and_then(|value| value.as_str().map(str::to_string))
                .unwrap_or_else(|| "this database".to_string())
        })
        .unwrap_or_else(|| "this database".to_string())
}

fn dialect_label(dialect: StructureDialect) -> String {
    match dialect {
        StructureDialect::Mysql => "mysql",
        StructureDialect::Postgres => "postgres",
        StructureDialect::Sqlite => "sqlite",
        StructureDialect::DuckDb => "duckdb",
        StructureDialect::SqlServer => "sqlserver",
        StructureDialect::Oracle => "oracle",
        StructureDialect::H2 => "h2",
        StructureDialect::ClickHouse => "clickhouse",
        StructureDialect::Unsupported => "this database",
    }
    .to_string()
}

fn database_type_for_dialect(dialect: StructureDialect) -> Option<DatabaseType> {
    match dialect {
        StructureDialect::Mysql => Some(DatabaseType::Mysql),
        StructureDialect::Postgres => Some(DatabaseType::Postgres),
        StructureDialect::Sqlite => Some(DatabaseType::Sqlite),
        StructureDialect::DuckDb => Some(DatabaseType::DuckDb),
        StructureDialect::SqlServer => Some(DatabaseType::SqlServer),
        StructureDialect::Oracle => Some(DatabaseType::Oracle),
        StructureDialect::H2 => Some(DatabaseType::H2),
        StructureDialect::ClickHouse => Some(DatabaseType::ClickHouse),
        StructureDialect::Unsupported => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn column(name: &str) -> EditableStructureColumn {
        EditableStructureColumn {
            id: name.to_string(),
            name: name.to_string(),
            data_type: "varchar(255)".to_string(),
            is_nullable: true,
            default_value: String::new(),
            comment: String::new(),
            is_primary_key: false,
            original: None,
            original_position: None,
            marked_for_drop: false,
        }
    }

    fn index(name: &str, columns: &[&str]) -> EditableStructureIndex {
        EditableStructureIndex {
            id: name.to_string(),
            name: name.to_string(),
            columns: columns.iter().map(|column| column.to_string()).collect(),
            is_unique: false,
            is_primary: false,
            filter: String::new(),
            index_type: String::new(),
            included_columns: Vec::new(),
            comment: String::new(),
            original: None,
            marked_for_drop: false,
        }
    }

    #[test]
    fn builds_mysql_column_and_index_changes() {
        let mut renamed = column("display_name");
        renamed.data_type = "varchar(120)".to_string();
        renamed.is_nullable = false;
        renamed.default_value = "'guest'".to_string();
        renamed.comment = "Shown name".to_string();
        renamed.original = Some(ColumnInfo {
            name: "name".to_string(),
            data_type: "varchar(80)".to_string(),
            is_nullable: true,
            column_default: None,
            is_primary_key: false,
            extra: None,
            comment: Some(String::new()),
        });
        let mut email = column("email");
        email.is_nullable = false;
        let mut old_index = index("idx_old", &["name"]);
        old_index.marked_for_drop = true;
        old_index.original = Some(IndexInfo {
            name: "idx_old".to_string(),
            columns: vec!["name".to_string()],
            is_unique: false,
            is_primary: false,
            filter: None,
            index_type: None,
            included_columns: None,
            comment: None,
        });
        let mut email_index = index("uniq_users_email", &["email"]);
        email_index.is_unique = true;

        let result = build_table_structure_change_sql(TableStructureSqlOptions {
            database_type: Some(DatabaseType::Mysql),
            schema: None,
            table_name: "users".to_string(),
            columns: vec![renamed, email],
            indexes: vec![old_index, email_index],
        });

        assert_eq!(result.warnings, Vec::<String>::new());
        assert_eq!(
            result.statements,
            vec![
                "ALTER TABLE `users` CHANGE COLUMN `name` `display_name` varchar(120) NOT NULL DEFAULT 'guest' COMMENT 'Shown name';",
                "ALTER TABLE `users` ADD COLUMN `email` varchar(255) NOT NULL;",
                "DROP INDEX `idx_old` ON `users`;",
                "CREATE UNIQUE INDEX `uniq_users_email` ON `users` (`email`);",
            ]
        );
    }

    #[test]
    fn builds_postgres_create_table_with_comments_and_index() {
        let mut id = column("id");
        id.data_type = "integer".to_string();
        id.is_nullable = false;
        id.is_primary_key = true;
        let mut name = column("name");
        name.data_type = "text".to_string();
        name.comment = "Display name".to_string();
        let mut idx = index("idx_users_name", &["name"]);
        idx.index_type = "gin".to_string();
        idx.comment = "search".to_string();

        let result = build_create_table_sql(TableStructureSqlOptions {
            database_type: Some(DatabaseType::Postgres),
            schema: Some("public".to_string()),
            table_name: "users".to_string(),
            columns: vec![id, name],
            indexes: vec![idx],
        });

        assert_eq!(result.warnings, Vec::<String>::new());
        assert_eq!(
            result.statements,
            vec![
                "CREATE TABLE \"public\".\"users\" (\n  \"id\" integer,\n  \"name\" text,\n  PRIMARY KEY (\"id\")\n);",
                "COMMENT ON COLUMN \"public\".\"users\".\"name\" IS 'Display name';",
                "CREATE INDEX \"idx_users_name\" ON \"public\".\"users\" USING GIN (\"name\");",
                "COMMENT ON INDEX \"idx_users_name\" IS 'search';",
            ]
        );
    }

    #[test]
    fn warns_for_sqlite_unsafe_column_changes() {
        let mut col = column("name");
        col.data_type = "text".to_string();
        col.original = Some(ColumnInfo {
            name: "name".to_string(),
            data_type: "varchar(80)".to_string(),
            is_nullable: true,
            column_default: None,
            is_primary_key: false,
            extra: None,
            comment: None,
        });

        let result = build_table_structure_change_sql(TableStructureSqlOptions {
            database_type: Some(DatabaseType::Sqlite),
            schema: None,
            table_name: "users".to_string(),
            columns: vec![col],
            indexes: Vec::new(),
        });

        assert_eq!(result.statements, Vec::<String>::new());
        assert_eq!(
            result.warnings,
            vec!["SQLite cannot safely alter existing column \"name\" without rebuilding the table."]
        );
    }

    #[test]
    fn builds_mysql_column_reorder_statements() {
        let mut id = column("id");
        id.data_type = "int".to_string();
        id.is_nullable = false;
        id.is_primary_key = true;
        id.original_position = Some(0);
        id.original = Some(ColumnInfo {
            name: "id".to_string(),
            data_type: "int".to_string(),
            is_nullable: false,
            column_default: None,
            is_primary_key: true,
            extra: None,
            comment: None,
        });

        let mut email = column("email");
        email.original_position = Some(2);
        email.original = Some(ColumnInfo {
            name: "email".to_string(),
            data_type: "varchar(255)".to_string(),
            is_nullable: true,
            column_default: None,
            is_primary_key: false,
            extra: None,
            comment: None,
        });

        let mut name = column("display_name");
        name.id = "name".to_string();
        name.data_type = "varchar(120)".to_string();
        name.original_position = Some(1);
        name.original = Some(ColumnInfo {
            name: "name".to_string(),
            data_type: "varchar(80)".to_string(),
            is_nullable: true,
            column_default: None,
            is_primary_key: false,
            extra: None,
            comment: None,
        });

        let result = build_table_structure_change_sql(TableStructureSqlOptions {
            database_type: Some(DatabaseType::Mysql),
            schema: None,
            table_name: "users".to_string(),
            columns: vec![id, email, name],
            indexes: Vec::new(),
        });

        assert_eq!(result.warnings, Vec::<String>::new());
        assert_eq!(
            result.statements,
            vec![
                "ALTER TABLE `users` MODIFY COLUMN `email` varchar(255) AFTER `id`;",
                "ALTER TABLE `users` CHANGE COLUMN `name` `display_name` varchar(120) AFTER `email`;",
            ]
        );
    }

    #[test]
    fn builds_sql_server_quoted_column_and_index_statements() {
        let mut email = column("email");
        email.data_type = "nvarchar(255)".to_string();
        email.is_nullable = false;

        let result = build_table_structure_change_sql(TableStructureSqlOptions {
            database_type: Some(DatabaseType::SqlServer),
            schema: Some("dbo".to_string()),
            table_name: "users".to_string(),
            columns: vec![email],
            indexes: vec![index("idx_users_email", &["email"])],
        });

        assert_eq!(result.warnings, Vec::<String>::new());
        assert_eq!(
            result.statements,
            vec![
                "ALTER TABLE [dbo].[users] ADD [email] nvarchar(255) NOT NULL;",
                "CREATE INDEX [idx_users_email] ON [dbo].[users] ([email]);",
            ]
        );
    }

    #[test]
    fn builds_duckdb_create_table_statements() {
        let mut name = column("name");
        name.data_type = "VARCHAR".to_string();
        name.is_nullable = false;
        let mut created_at = column("created_at");
        created_at.data_type = "TIMESTAMP".to_string();
        created_at.default_value = "current_timestamp".to_string();

        let result = build_create_table_sql(TableStructureSqlOptions {
            database_type: Some(DatabaseType::DuckDb),
            schema: None,
            table_name: "events".to_string(),
            columns: vec![name, created_at],
            indexes: vec![index("idx_events_name", &["name"])],
        });

        assert_eq!(result.warnings, Vec::<String>::new());
        assert_eq!(
            result.statements,
            vec![
                "CREATE TABLE \"events\" (\n  \"name\" VARCHAR NOT NULL,\n  \"created_at\" TIMESTAMP DEFAULT current_timestamp\n);",
                "CREATE INDEX \"idx_events_name\" ON \"events\" (\"name\");",
            ]
        );
    }

    #[test]
    fn builds_clickhouse_nullable_comment_and_reorder_statements() {
        let mut source = column("source");
        source.data_type = "String".to_string();
        source.is_nullable = true;
        source.comment = "traffic source".to_string();
        let mut status = column("status");
        status.data_type = "Nullable(String)".to_string();
        status.is_nullable = false;
        status.comment = "current status".to_string();
        status.original = Some(ColumnInfo {
            name: "status".to_string(),
            data_type: "Nullable(String)".to_string(),
            is_nullable: true,
            column_default: Some("'pending'".to_string()),
            is_primary_key: false,
            extra: None,
            comment: Some("old status".to_string()),
        });

        let result = build_table_structure_change_sql(TableStructureSqlOptions {
            database_type: Some(DatabaseType::ClickHouse),
            schema: None,
            table_name: "events".to_string(),
            columns: vec![source, status],
            indexes: Vec::new(),
        });

        assert_eq!(result.warnings, Vec::<String>::new());
        assert_eq!(
            result.statements,
            vec![
                "ALTER TABLE \"events\" ADD COLUMN \"source\" Nullable(String);",
                "ALTER TABLE \"events\" COMMENT COLUMN \"source\" 'traffic source';",
                "ALTER TABLE \"events\" MODIFY COLUMN \"status\" REMOVE DEFAULT;",
                "ALTER TABLE \"events\" MODIFY COLUMN \"status\" String;",
                "ALTER TABLE \"events\" COMMENT COLUMN \"status\" 'current status';",
            ]
        );
    }

    #[test]
    fn builds_h2_schema_qualified_existing_column_statements() {
        let mut name = column("DISPLAY_NAME");
        name.id = "name".to_string();
        name.data_type = "VARCHAR(120)".to_string();
        name.is_nullable = false;
        name.default_value = "'guest'".to_string();
        name.comment = "Display name".to_string();
        name.original = Some(ColumnInfo {
            name: "NAME".to_string(),
            data_type: "VARCHAR(80)".to_string(),
            is_nullable: true,
            column_default: None,
            is_primary_key: false,
            extra: None,
            comment: Some(String::new()),
        });

        let result = build_table_structure_change_sql(TableStructureSqlOptions {
            database_type: Some(DatabaseType::H2),
            schema: Some("PUBLIC".to_string()),
            table_name: "USERS".to_string(),
            columns: vec![name],
            indexes: vec![index("IDX_USERS_DISPLAY_NAME", &["DISPLAY_NAME"])],
        });

        assert_eq!(result.warnings, Vec::<String>::new());
        assert_eq!(
            result.statements,
            vec![
                "ALTER TABLE \"PUBLIC\".\"USERS\" ALTER COLUMN \"NAME\" RENAME TO \"DISPLAY_NAME\";",
                "ALTER TABLE \"PUBLIC\".\"USERS\" ALTER COLUMN \"DISPLAY_NAME\" SET DATA TYPE VARCHAR(120);",
                "ALTER TABLE \"PUBLIC\".\"USERS\" ALTER COLUMN \"DISPLAY_NAME\" SET NOT NULL;",
                "ALTER TABLE \"PUBLIC\".\"USERS\" ALTER COLUMN \"DISPLAY_NAME\" SET DEFAULT 'guest';",
                "COMMENT ON COLUMN \"PUBLIC\".\"USERS\".\"DISPLAY_NAME\" IS 'Display name';",
                "CREATE INDEX \"IDX_USERS_DISPLAY_NAME\" ON \"PUBLIC\".\"USERS\" (\"DISPLAY_NAME\");",
            ]
        );
    }

    #[test]
    fn builds_postgres_alter_table_add_primary_key() {
        let mut id = column("id");
        id.data_type = "integer".to_string();
        id.is_nullable = false;
        id.is_primary_key = true;
        id.original = Some(ColumnInfo {
            name: "id".to_string(),
            data_type: "integer".to_string(),
            is_nullable: false,
            column_default: None,
            is_primary_key: false,
            extra: None,
            comment: None,
        });

        let result = build_table_structure_change_sql(TableStructureSqlOptions {
            database_type: Some(DatabaseType::Postgres),
            schema: Some("public".to_string()),
            table_name: "users".to_string(),
            columns: vec![id],
            indexes: Vec::new(),
        });

        assert_eq!(result.warnings, Vec::<String>::new());
        assert_eq!(result.statements, vec!["ALTER TABLE \"public\".\"users\" ADD PRIMARY KEY (\"id\");"]);
    }

    #[test]
    fn builds_postgres_alter_table_drop_primary_key() {
        let mut id = column("id");
        id.data_type = "integer".to_string();
        id.is_nullable = false;
        id.is_primary_key = false;
        id.original = Some(ColumnInfo {
            name: "id".to_string(),
            data_type: "integer".to_string(),
            is_nullable: false,
            column_default: None,
            is_primary_key: true,
            extra: None,
            comment: None,
        });

        let result = build_table_structure_change_sql(TableStructureSqlOptions {
            database_type: Some(DatabaseType::Postgres),
            schema: Some("public".to_string()),
            table_name: "users".to_string(),
            columns: vec![id],
            indexes: Vec::new(),
        });

        assert_eq!(result.warnings, Vec::<String>::new());
        assert_eq!(result.statements, vec!["ALTER TABLE \"public\".\"users\" DROP CONSTRAINT \"users_pkey\";"]);
    }

    #[test]
    fn builds_mysql_alter_table_change_primary_key() {
        let mut old_pk = column("id");
        old_pk.id = "old_id".to_string();
        old_pk.data_type = "int".to_string();
        old_pk.is_nullable = false;
        old_pk.is_primary_key = false;
        old_pk.original = Some(ColumnInfo {
            name: "id".to_string(),
            data_type: "int".to_string(),
            is_nullable: false,
            column_default: None,
            is_primary_key: true,
            extra: None,
            comment: None,
        });

        let mut new_pk = column("uuid");
        new_pk.id = "new_uuid".to_string();
        new_pk.data_type = "varchar(36)".to_string();
        new_pk.is_nullable = false;
        new_pk.is_primary_key = true;
        new_pk.original = Some(ColumnInfo {
            name: "uuid".to_string(),
            data_type: "varchar(36)".to_string(),
            is_nullable: false,
            column_default: None,
            is_primary_key: false,
            extra: None,
            comment: None,
        });

        let result = build_table_structure_change_sql(TableStructureSqlOptions {
            database_type: Some(DatabaseType::Mysql),
            schema: None,
            table_name: "users".to_string(),
            columns: vec![old_pk, new_pk],
            indexes: Vec::new(),
        });

        assert_eq!(result.warnings, Vec::<String>::new());
        assert_eq!(
            result.statements,
            vec!["ALTER TABLE `users` DROP PRIMARY KEY;", "ALTER TABLE `users` ADD PRIMARY KEY (`uuid`);",]
        );
    }

    #[test]
    fn builds_no_statements_when_primary_key_unchanged() {
        let mut id = column("id");
        id.data_type = "integer".to_string();
        id.is_nullable = false;
        id.is_primary_key = true;
        id.original = Some(ColumnInfo {
            name: "id".to_string(),
            data_type: "integer".to_string(),
            is_nullable: false,
            column_default: None,
            is_primary_key: true,
            extra: None,
            comment: None,
        });

        let result = build_table_structure_change_sql(TableStructureSqlOptions {
            database_type: Some(DatabaseType::Postgres),
            schema: None,
            table_name: "users".to_string(),
            columns: vec![id],
            indexes: Vec::new(),
        });

        assert_eq!(result.warnings, Vec::<String>::new());
        assert!(result.statements.is_empty());
    }

    #[test]
    fn warns_sqlite_cannot_alter_primary_key() {
        let mut id = column("id");
        id.data_type = "integer".to_string();
        id.is_nullable = false;
        id.is_primary_key = true;
        id.original = Some(ColumnInfo {
            name: "id".to_string(),
            data_type: "integer".to_string(),
            is_nullable: false,
            column_default: None,
            is_primary_key: false,
            extra: None,
            comment: None,
        });

        let result = build_table_structure_change_sql(TableStructureSqlOptions {
            database_type: Some(DatabaseType::Sqlite),
            schema: None,
            table_name: "users".to_string(),
            columns: vec![id],
            indexes: Vec::new(),
        });

        assert_eq!(result.statements, Vec::<String>::new());
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("primary key"));
    }
}
