use clickhouse::Client;
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};

// ---------------------------------------------------------------------------
// Main loop
// ---------------------------------------------------------------------------

fn main() {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut clients: HashMap<String, Client> = HashMap::new();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading from stdin: {}", e);
                break;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        let req: JsonValue = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to parse request: {}", e);
                continue;
            }
        };

        let id = req["id"].clone();
        let method = match req["method"].as_str() {
            Some(m) => m.to_string(),
            None => {
                send_error(&mut stdout, id, -32600, "Method not specified");
                continue;
            }
        };

        let params = &req["params"];
        let conn_params = &params["params"];

        let client = get_or_create_client(&mut clients, conn_params);
        let db = conn_params
            .get("database")
            .and_then(|d| d.as_str())
            .unwrap_or("default")
            .to_string();

        match method.as_str() {
            "test_connection" => {
                match rt.block_on(test_connection(&client)) {
                    Ok(v) => send_success(&mut stdout, id, v),
                    Err(e) => send_error(&mut stdout, id, -32000, &e),
                }
            }
            "get_databases" => {
                match rt.block_on(get_databases(&client)) {
                    Ok(v) => send_success(&mut stdout, id, v),
                    Err(e) => send_error(&mut stdout, id, -32000, &e),
                }
            }
            "get_schemas" => {
                match rt.block_on(get_databases(&client)) {
                    Ok(v) => send_success(&mut stdout, id, v),
                    Err(e) => send_error(&mut stdout, id, -32000, &e),
                }
            }
            "get_tables" => {
                let schema = params
                    .get("schema")
                    .and_then(|s| s.as_str())
                    .unwrap_or(&db)
                    .to_string();
                match rt.block_on(get_tables(&client, &schema)) {
                    Ok(v) => send_success(&mut stdout, id, v),
                    Err(e) => send_error(&mut stdout, id, -32001, &e),
                }
            }
            "get_columns" => {
                let schema = params
                    .get("schema")
                    .and_then(|s| s.as_str())
                    .unwrap_or(&db)
                    .to_string();
                let table = params.get("table").and_then(|t| t.as_str()).unwrap_or("");
                match rt.block_on(get_columns(&client, &schema, table)) {
                    Ok(v) => send_success(&mut stdout, id, v),
                    Err(e) => send_error(&mut stdout, id, -32002, &e),
                }
            }
            "get_foreign_keys" => {
                send_success(&mut stdout, id, json!([]));
            }
            "get_indexes" => {
                let schema = params
                    .get("schema")
                    .and_then(|s| s.as_str())
                    .unwrap_or(&db)
                    .to_string();
                let table = params.get("table").and_then(|t| t.as_str()).unwrap_or("");
                match rt.block_on(get_indexes(&client, &schema, table)) {
                    Ok(v) => send_success(&mut stdout, id, v),
                    Err(e) => send_error(&mut stdout, id, -32004, &e),
                }
            }
            "get_views" => {
                let schema = params
                    .get("schema")
                    .and_then(|s| s.as_str())
                    .unwrap_or(&db)
                    .to_string();
                match rt.block_on(get_views(&client, &schema)) {
                    Ok(v) => send_success(&mut stdout, id, v),
                    Err(e) => send_error(&mut stdout, id, -32005, &e),
                }
            }
            "get_view_definition" => {
                let schema = params
                    .get("schema")
                    .and_then(|s| s.as_str())
                    .unwrap_or(&db)
                    .to_string();
                let view = params.get("view").and_then(|v| v.as_str()).unwrap_or("");
                match rt.block_on(get_view_definition(&client, &schema, view)) {
                    Ok(v) => send_success(&mut stdout, id, v),
                    Err(e) => send_error(&mut stdout, id, -32006, &e),
                }
            }
            "get_view_columns" => {
                let schema = params
                    .get("schema")
                    .and_then(|s| s.as_str())
                    .unwrap_or(&db)
                    .to_string();
                let view = params.get("view").and_then(|v| v.as_str()).unwrap_or("");
                match rt.block_on(get_columns(&client, &schema, view)) {
                    Ok(v) => send_success(&mut stdout, id, v),
                    Err(e) => send_error(&mut stdout, id, -32007, &e),
                }
            }
            "create_view" => {
                let schema = params
                    .get("schema")
                    .and_then(|s| s.as_str())
                    .unwrap_or(&db)
                    .to_string();
                let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let definition = params.get("definition").and_then(|d| d.as_str()).unwrap_or("");
                let sql = format!(
                    "CREATE VIEW `{}`.`{}` AS {}",
                    escape_identifier(&schema),
                    escape_identifier(name),
                    definition
                );
                match rt.block_on(execute_statement(&client, &sql)) {
                    Ok(_) => send_success(&mut stdout, id, json!(null)),
                    Err(e) => send_error(&mut stdout, id, -32000, &e),
                }
            }
            "alter_view" => {
                let schema = params
                    .get("schema")
                    .and_then(|s| s.as_str())
                    .unwrap_or(&db)
                    .to_string();
                let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let definition = params.get("definition").and_then(|d| d.as_str()).unwrap_or("");
                let sql = format!(
                    "CREATE OR REPLACE VIEW `{}`.`{}` AS {}",
                    escape_identifier(&schema),
                    escape_identifier(name),
                    definition
                );
                match rt.block_on(execute_statement(&client, &sql)) {
                    Ok(_) => send_success(&mut stdout, id, json!(null)),
                    Err(e) => send_error(&mut stdout, id, -32000, &e),
                }
            }
            "drop_view" => {
                let schema = params
                    .get("schema")
                    .and_then(|s| s.as_str())
                    .unwrap_or(&db)
                    .to_string();
                let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let sql = format!(
                    "DROP VIEW `{}`.`{}`",
                    escape_identifier(&schema),
                    escape_identifier(name)
                );
                match rt.block_on(execute_statement(&client, &sql)) {
                    Ok(_) => send_success(&mut stdout, id, json!(null)),
                    Err(e) => send_error(&mut stdout, id, -32000, &e),
                }
            }
            "get_routines" | "get_routine_parameters" => {
                send_success(&mut stdout, id, json!([]));
            }
            "get_routine_definition" => {
                send_error(
                    &mut stdout,
                    id,
                    -32601,
                    "ClickHouse does not support stored routines",
                );
            }
            "execute_query" => {
                let query = params.get("query").and_then(|q| q.as_str()).unwrap_or("");
                let limit = params
                    .get("page_size")
                    .and_then(|l| l.as_u64())
                    .map(|l| l as u32);
                let page = params
                    .get("page")
                    .and_then(|p| p.as_u64())
                    .map(|p| p as u32)
                    .unwrap_or(1);
                match rt.block_on(execute_query(&client, &db, query, limit, page)) {
                    Ok(v) => send_success(&mut stdout, id, v),
                    Err(e) => send_error(&mut stdout, id, -32012, &e),
                }
            }
            "insert_record" => {
                let schema = params
                    .get("schema")
                    .and_then(|s| s.as_str())
                    .unwrap_or(&db)
                    .to_string();
                let table = params.get("table").and_then(|t| t.as_str()).unwrap_or("");
                let data = params
                    .get("data")
                    .and_then(|d| d.as_object())
                    .cloned()
                    .unwrap_or_default();
                match rt.block_on(insert_record(&client, &schema, table, &data)) {
                    Ok(n) => send_success(&mut stdout, id, json!(n)),
                    Err(e) => send_error(&mut stdout, id, -32013, &e),
                }
            }
            "update_record" => {
                let schema = params
                    .get("schema")
                    .and_then(|s| s.as_str())
                    .unwrap_or(&db)
                    .to_string();
                let table = params.get("table").and_then(|t| t.as_str()).unwrap_or("");
                let pk_col = params
                    .get("primary_key_column")
                    .and_then(|p| p.as_str())
                    .unwrap_or("");
                let pk_val = params.get("primary_key_value").cloned().unwrap_or(JsonValue::Null);
                let col_name = params
                    .get("column")
                    .and_then(|c| c.as_str())
                    .unwrap_or("");
                let new_val = params.get("value").cloned().unwrap_or(JsonValue::Null);
                match rt.block_on(update_record(
                    &client, &schema, table, pk_col, &pk_val, col_name, &new_val,
                )) {
                    Ok(n) => send_success(&mut stdout, id, json!(n)),
                    Err(e) => send_error(&mut stdout, id, -32014, &e),
                }
            }
            "delete_record" => {
                let schema = params
                    .get("schema")
                    .and_then(|s| s.as_str())
                    .unwrap_or(&db)
                    .to_string();
                let table = params.get("table").and_then(|t| t.as_str()).unwrap_or("");
                let pk_col = params
                    .get("primary_key_column")
                    .and_then(|p| p.as_str())
                    .unwrap_or("");
                let pk_val = params.get("primary_key_value").cloned().unwrap_or(JsonValue::Null);
                match rt.block_on(delete_record(&client, &schema, table, pk_col, &pk_val)) {
                    Ok(n) => send_success(&mut stdout, id, json!(n)),
                    Err(e) => send_error(&mut stdout, id, -32015, &e),
                }
            }
            "get_schema_snapshot" => {
                let schema = params
                    .get("schema")
                    .and_then(|s| s.as_str())
                    .unwrap_or(&db)
                    .to_string();
                match rt.block_on(get_schema_snapshot(&client, &schema)) {
                    Ok(v) => send_success(&mut stdout, id, v),
                    Err(e) => send_error(&mut stdout, id, -32016, &e),
                }
            }
            "get_all_columns_batch" => {
                let schema = params
                    .get("schema")
                    .and_then(|s| s.as_str())
                    .unwrap_or(&db)
                    .to_string();
                match rt.block_on(get_all_columns_batch(&client, &schema)) {
                    Ok(v) => send_success(&mut stdout, id, v),
                    Err(e) => send_error(&mut stdout, id, -32017, &e),
                }
            }
            "get_all_foreign_keys_batch" => {
                send_success(&mut stdout, id, json!({}));
            }
            "get_create_table_sql" => {
                let schema = params
                    .get("schema")
                    .and_then(|s| s.as_str())
                    .unwrap_or(&db)
                    .to_string();
                let table_name = params
                    .get("table_name")
                    .and_then(|t| t.as_str())
                    .unwrap_or("");
                match rt.block_on(get_create_table_sql(&client, &schema, table_name)) {
                    Ok(v) => send_success(&mut stdout, id, v),
                    Err(e) => send_error(&mut stdout, id, -32018, &e),
                }
            }
            "get_add_column_sql" => {
                let schema = params
                    .get("schema")
                    .and_then(|s| s.as_str())
                    .unwrap_or(&db)
                    .to_string();
                let table = params.get("table").and_then(|t| t.as_str()).unwrap_or("");
                let column = params.get("column").cloned().unwrap_or(json!({}));
                let col_name = column.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let col_type = column.get("data_type").and_then(|t| t.as_str()).unwrap_or("String");
                let nullable = column.get("is_nullable").and_then(|n| n.as_bool()).unwrap_or(true);
                let ch_type = if nullable {
                    format!("Nullable({})", col_type)
                } else {
                    col_type.to_string()
                };
                send_success(
                    &mut stdout,
                    id,
                    json!([format!(
                        "ALTER TABLE `{}`.`{}` ADD COLUMN `{}` {}",
                        schema, table, col_name, ch_type
                    )]),
                );
            }
            "get_alter_column_sql" => {
                let schema = params
                    .get("schema")
                    .and_then(|s| s.as_str())
                    .unwrap_or(&db)
                    .to_string();
                let table = params.get("table").and_then(|t| t.as_str()).unwrap_or("");
                let old_name = params
                    .get("old_column")
                    .and_then(|c| c.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("");
                let new_col = params.get("new_column").cloned().unwrap_or(json!({}));
                let new_name = new_col.get("name").and_then(|n| n.as_str()).unwrap_or(old_name);
                let new_type = new_col.get("data_type").and_then(|t| t.as_str());

                let mut statements = Vec::new();

                if old_name != new_name && !old_name.is_empty() {
                    statements.push(format!(
                        "ALTER TABLE `{}`.`{}` RENAME COLUMN `{}` TO `{}`",
                        schema, table, old_name, new_name
                    ));
                }
                if let Some(t) = new_type {
                    statements.push(format!(
                        "ALTER TABLE `{}`.`{}` MODIFY COLUMN `{}` {}",
                        schema, table, new_name, t
                    ));
                }
                if statements.is_empty() {
                    statements.push("-- No changes needed".to_string());
                }
                send_success(&mut stdout, id, json!(statements));
            }
            "get_create_index_sql" => {
                let schema = params
                    .get("schema")
                    .and_then(|s| s.as_str())
                    .unwrap_or(&db)
                    .to_string();
                let table = params.get("table").and_then(|t| t.as_str()).unwrap_or("");
                let index_name = params
                    .get("index_name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("idx");
                let columns: Vec<String> = params
                    .get("columns")
                    .and_then(|c| c.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let cols_expr = columns.join(", ");
                send_success(
                    &mut stdout,
                    id,
                    json!([format!(
                        "ALTER TABLE `{}`.`{}` ADD INDEX `{}` ({}) TYPE minmax GRANULARITY 1",
                        schema, table, index_name, cols_expr
                    )]),
                );
            }
            "get_create_foreign_key_sql" => {
                send_success(
                    &mut stdout,
                    id,
                    json!(["-- ClickHouse does not support foreign key constraints"]),
                );
            }
            "drop_index" => {
                let schema = params
                    .get("schema")
                    .and_then(|s| s.as_str())
                    .unwrap_or(&db)
                    .to_string();
                let table = params.get("table").and_then(|t| t.as_str()).unwrap_or("");
                let index_name = params
                    .get("index_name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("");
                match rt.block_on(drop_index(&client, &schema, table, index_name)) {
                    Ok(_) => send_success(&mut stdout, id, json!(null)),
                    Err(e) => send_error(&mut stdout, id, -32024, &e),
                }
            }
            "drop_foreign_key" => {
                send_error(
                    &mut stdout,
                    id,
                    -32601,
                    "ClickHouse does not support foreign key constraints",
                );
            }
            _ => {
                send_error(
                    &mut stdout,
                    id,
                    -32601,
                    &format!("Method '{}' not implemented", method),
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Connection management
// ---------------------------------------------------------------------------

fn build_client(params: &JsonValue) -> Client {
    let host = params
        .get("host")
        .and_then(|h| h.as_str())
        .unwrap_or("localhost");
    let port = params
        .get("port")
        .and_then(|p| p.as_u64())
        .unwrap_or(8123);
    let database = params
        .get("database")
        .and_then(|d| d.as_str())
        .unwrap_or("default");
    let username = params
        .get("username")
        .and_then(|u| u.as_str())
        .unwrap_or("default");
    let password = params
        .get("password")
        .and_then(|p| p.as_str())
        .unwrap_or("");
    let use_ssl = params
        .get("ssl")
        .and_then(|s| s.as_bool())
        .unwrap_or(false);

    let protocol = if use_ssl { "https" } else { "http" };
    let url = format!("{}://{}:{}", protocol, host, port);

    Client::default()
        .with_url(&url)
        .with_user(username)
        .with_password(password)
        .with_database(database)
}

fn get_or_create_client<'a>(
    clients: &'a mut HashMap<String, Client>,
    params: &JsonValue,
) -> &'a Client {
    let key = format!(
        "{}:{}:{}:{}:{}",
        params.get("host").and_then(|h| h.as_str()).unwrap_or("localhost"),
        params.get("port").and_then(|p| p.as_u64()).unwrap_or(8123),
        params.get("database").and_then(|d| d.as_str()).unwrap_or("default"),
        params.get("username").and_then(|u| u.as_str()).unwrap_or("default"),
        params.get("password").and_then(|p| p.as_str()).unwrap_or(""),
    );

    if !clients.contains_key(&key) {
        let client = build_client(params);
        clients.insert(key.clone(), client);
    }

    clients.get(&key).unwrap()
}

// ---------------------------------------------------------------------------
// JSON-RPC helpers
// ---------------------------------------------------------------------------

fn send_success(stdout: &mut io::Stdout, id: JsonValue, result: JsonValue) {
    let response = json!({
        "jsonrpc": "2.0",
        "result": result,
        "id": id
    });
    let mut res_str = serde_json::to_string(&response).unwrap();
    res_str.push('\n');
    stdout.write_all(res_str.as_bytes()).unwrap();
    stdout.flush().unwrap();
}

fn send_error(stdout: &mut io::Stdout, id: JsonValue, code: i32, message: &str) {
    let response = json!({
        "jsonrpc": "2.0",
        "error": {
            "code": code,
            "message": message
        },
        "id": id
    });
    let mut res_str = serde_json::to_string(&response).unwrap();
    res_str.push('\n');
    stdout.write_all(res_str.as_bytes()).unwrap();
    stdout.flush().unwrap();
}

// ---------------------------------------------------------------------------
// Query helpers
// ---------------------------------------------------------------------------

/// Execute a query using JSONEachRow format and parse the rows.
/// Returns (column_names, rows_as_json_arrays).
async fn query_json_rows(
    client: &Client,
    sql: &str,
) -> Result<(Vec<String>, Vec<Vec<JsonValue>>), String> {
    let mut cursor = client
        .query(sql)
        .fetch_bytes("JSONEachRow")
        .map_err(|e| e.to_string())?;

    let mut all_bytes = Vec::new();
    while let Some(chunk) = cursor
        .next()
        .await
        .map_err(|e| e.to_string())?
    {
        all_bytes.extend_from_slice(&chunk);
    }

    let content = String::from_utf8_lossy(&all_bytes);
    let mut json_rows: Vec<serde_json::Map<String, JsonValue>> = Vec::new();

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let obj: serde_json::Map<String, JsonValue> = serde_json::from_str(line)
            .map_err(|e| format!("Failed to parse row: {}", e))?;
        json_rows.push(obj);
    }

    let column_names: Vec<String> = if let Some(first) = json_rows.first() {
        first.keys().cloned().collect()
    } else {
        vec![]
    };

    let rows: Vec<Vec<JsonValue>> = json_rows
        .iter()
        .map(|row| {
            column_names
                .iter()
                .map(|col| row.get(col).cloned().unwrap_or(JsonValue::Null))
                .collect()
        })
        .collect();

    Ok((column_names, rows))
}

/// Execute a non-SELECT statement.
async fn execute_statement(client: &Client, sql: &str) -> Result<(), String> {
    client
        .query(sql)
        .execute()
        .await
        .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Schema discovery
// ---------------------------------------------------------------------------

async fn test_connection(client: &Client) -> Result<JsonValue, String> {
    let (_, rows) = query_json_rows(client, "SELECT 1 AS ok").await?;
    let _ = rows;
    Ok(json!({ "success": true }))
}

async fn get_databases(client: &Client) -> Result<JsonValue, String> {
    let (_, rows) = query_json_rows(
        client,
        "SELECT name FROM system.databases ORDER BY name",
    )
    .await?;

    let names: Vec<JsonValue> = rows
        .iter()
        .filter_map(|row| row.first())
        .cloned()
        .collect();

    Ok(json!(names))
}

async fn get_tables(client: &Client, database: &str) -> Result<JsonValue, String> {
    let sql = format!(
        "SELECT name, comment \
         FROM system.tables \
         WHERE database = '{}' AND is_temporary = 0 AND engine NOT LIKE '%View%' \
         ORDER BY name",
        escape_string(database)
    );
    let (_, rows) = query_json_rows(client, &sql).await?;

    let result: Vec<JsonValue> = rows
        .iter()
        .map(|row| {
            let name = row.first().and_then(|v| v.as_str()).unwrap_or("").to_string();
            let comment = row.get(1).and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string());
            json!({ "name": name, "schema": database, "comment": comment })
        })
        .collect();

    Ok(json!(result))
}

async fn get_columns(client: &Client, database: &str, table: &str) -> Result<JsonValue, String> {
    let sql = format!(
        "SELECT name, type, default_kind, default_expression, comment, is_in_primary_key \
         FROM system.columns \
         WHERE database = '{}' AND table = '{}' \
         ORDER BY position",
        escape_string(database),
        escape_string(table)
    );
    let (_, rows) = query_json_rows(client, &sql).await?;

    let result: Vec<JsonValue> = rows
        .iter()
        .map(|row| {
            let name = row.first().and_then(|v| v.as_str()).unwrap_or("").to_string();
            let data_type = row.get(1).and_then(|v| v.as_str()).unwrap_or("String").to_string();
            let is_nullable = data_type.starts_with("Nullable");
            let default_kind = row.get(2).and_then(|v| v.as_str()).unwrap_or("");
            let default_expr = row.get(3).and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string());
            let is_pk = row.get(5).map(|v| v == &json!(1) || v.as_str() == Some("1")).unwrap_or(false);

            json!({
                "name": name,
                "data_type": data_type,
                "is_primary_key": is_pk,
                "is_nullable": is_nullable,
                "is_auto_increment": false,
                "column_default": if default_kind.is_empty() { JsonValue::Null } else { default_expr.into() },
                "comment": row.get(4).and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()),
            })
        })
        .collect();

    Ok(json!(result))
}

async fn get_indexes(client: &Client, database: &str, table: &str) -> Result<JsonValue, String> {
    // Data skipping indexes
    let sql = format!(
        "SELECT name, expr, type \
         FROM system.data_skipping_indices \
         WHERE database = '{}' AND table = '{}' \
         ORDER BY name",
        escape_string(database),
        escape_string(table)
    );
    let (_, rows) = query_json_rows(client, &sql).await?;

    let mut indexes: Vec<JsonValue> = rows
        .iter()
        .map(|row| {
            let name = row.first().and_then(|v| v.as_str()).unwrap_or("").to_string();
            let expr = row.get(1).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let idx_type = row.get(2).and_then(|v| v.as_str()).unwrap_or("").to_string();
            json!({
                "index_name": name,
                "columns": [expr],
                "is_unique": false,
                "is_primary": false,
                "index_type": idx_type,
            })
        })
        .collect();

    // Add primary key as a synthetic index entry
    let pk_sql = format!(
        "SELECT name FROM system.columns \
         WHERE database = '{}' AND table = '{}' AND is_in_primary_key = 1 \
         ORDER BY position",
        escape_string(database),
        escape_string(table)
    );
    if let Ok((_, pk_rows)) = query_json_rows(client, &pk_sql).await {
        if !pk_rows.is_empty() {
            let pk_cols: Vec<String> = pk_rows
                .iter()
                .filter_map(|row| row.first().and_then(|v| v.as_str()).map(|s| s.to_string()))
                .collect();
            indexes.insert(
                0,
                json!({
                    "index_name": "PRIMARY",
                    "columns": pk_cols,
                    "is_unique": true,
                    "is_primary": true,
                    "index_type": "primary",
                }),
            );
        }
    }

    Ok(json!(indexes))
}

async fn get_views(client: &Client, database: &str) -> Result<JsonValue, String> {
    let sql = format!(
        "SELECT name, engine, comment \
         FROM system.tables \
         WHERE database = '{}' AND engine LIKE '%View%' \
         ORDER BY name",
        escape_string(database)
    );
    let (_, rows) = query_json_rows(client, &sql).await?;

    let result: Vec<JsonValue> = rows
        .iter()
        .map(|row| {
            let name = row.first().and_then(|v| v.as_str()).unwrap_or("").to_string();
            let engine = row.get(1).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let comment = row.get(2).and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string());
            json!({
                "name": name,
                "schema": database,
                "is_materialized": engine.contains("Materialized"),
                "comment": comment,
            })
        })
        .collect();

    Ok(json!(result))
}

async fn get_view_definition(
    client: &Client,
    database: &str,
    view: &str,
) -> Result<JsonValue, String> {
    let sql = format!(
        "SELECT as_select FROM system.tables WHERE database = '{}' AND name = '{}'",
        escape_string(database),
        escape_string(view)
    );
    let (_, rows) = query_json_rows(client, &sql).await?;

    let definition = rows
        .first()
        .and_then(|row| row.first())
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(json!(definition))
}

// ---------------------------------------------------------------------------
// Query execution
// ---------------------------------------------------------------------------

fn is_select_like(query: &str) -> bool {
    let upper = query.trim().to_uppercase();
    upper.starts_with("SELECT")
        || upper.starts_with("SHOW")
        || upper.starts_with("DESCRIBE")
        || upper.starts_with("DESC ")
        || upper.starts_with("EXPLAIN")
        || upper.starts_with("WITH")
}

fn has_format_clause(query: &str) -> bool {
    query.to_uppercase().contains(" FORMAT ")
}

async fn execute_query(
    client: &Client,
    _database: &str,
    query: &str,
    limit: Option<u32>,
    page: u32,
) -> Result<JsonValue, String> {
    let q = query.trim();

    if !is_select_like(q) {
        // DDL / DML — execute and return an empty result set
        execute_statement(client, q).await?;
        return Ok(json!({
            "columns": [],
            "rows": [],
            "affected_rows": 0,
            "truncated": false,
            "has_more": false,
            "pagination": null,
        }));
    }

    if has_format_clause(q) {
        // User supplied their own FORMAT — strip it from the query and pass it
        // to fetch_bytes to avoid the double-FORMAT conflict.
        let upper = q.to_uppercase();
        let (query_body, fmt) = match upper.rfind(" FORMAT ") {
            Some(pos) => (&q[..pos], q[pos + 8..].trim()),
            None => (q, "TSV"),
        };
        let mut cursor = client
            .query(query_body)
            .fetch_bytes(fmt)
            .map_err(|e| e.to_string())?;
        let mut all_bytes = Vec::new();
        while let Some(chunk) = cursor.next().await.map_err(|e| e.to_string())? {
            all_bytes.extend_from_slice(&chunk);
        }
        let text = String::from_utf8_lossy(&all_bytes).to_string();
        return Ok(json!({
            "columns": ["result"],
            "rows": [[text]],
            "affected_rows": 0,
            "truncated": false,
            "has_more": false,
            "pagination": null,
        }));
    }

    // Wrap with LIMIT/OFFSET for pagination
    let page = if page == 0 { 1 } else { page };
    let fetch_limit = limit.map(|l| l + 1);
    let paged_sql = if let Some(l) = fetch_limit {
        let offset = (page - 1) * limit.unwrap();
        format!("SELECT * FROM ({}) LIMIT {} OFFSET {}", q, l, offset)
    } else {
        q.to_string()
    };

    let mut cursor = client
        .query(&paged_sql)
        .fetch_bytes("JSONEachRow")
        .map_err(|e| e.to_string())?;

    let mut all_bytes = Vec::new();
    while let Some(chunk) = cursor.next().await.map_err(|e| e.to_string())? {
        all_bytes.extend_from_slice(&chunk);
    }

    let content = String::from_utf8_lossy(&all_bytes);
    let mut json_rows: Vec<serde_json::Map<String, JsonValue>> = Vec::new();

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<serde_json::Map<String, JsonValue>>(line) {
            Ok(obj) => json_rows.push(obj),
            Err(e) => return Err(format!("Failed to parse result row: {}", e)),
        }
    }

    let has_more = fetch_limit.map(|l| json_rows.len() > l as usize - 1).unwrap_or(false);
    if has_more {
        json_rows.truncate(limit.unwrap() as usize);
    }

    let columns: Vec<String> = if let Some(first) = json_rows.first() {
        first.keys().cloned().collect()
    } else {
        vec![]
    };

    let rows: Vec<JsonValue> = json_rows
        .iter()
        .map(|row| {
            let vals: Vec<JsonValue> = columns
                .iter()
                .map(|col| row.get(col).cloned().unwrap_or(JsonValue::Null))
                .collect();
            JsonValue::Array(vals)
        })
        .collect();

    Ok(json!({
        "columns": columns,
        "rows": rows,
        "affected_rows": 0,
        "truncated": has_more,
        "has_more": has_more,
        "pagination": if limit.is_some() {
            json!({
                "page": page,
                "page_size": limit.unwrap(),
                "total_rows": null,
                "has_more": has_more,
            })
        } else {
            JsonValue::Null
        },
    }))
}

// ---------------------------------------------------------------------------
// CRUD operations
// ---------------------------------------------------------------------------

async fn insert_record(
    client: &Client,
    database: &str,
    table: &str,
    data: &serde_json::Map<String, JsonValue>,
) -> Result<u64, String> {
    if data.is_empty() {
        return Err("No data provided for insert".to_string());
    }

    let columns: Vec<String> = data.keys().map(|k| format!("`{}`", k)).collect();
    let values: Vec<String> = data.values().map(format_value_for_sql).collect();

    let sql = format!(
        "INSERT INTO `{}`.`{}` ({}) VALUES ({})",
        escape_identifier(database),
        escape_identifier(table),
        columns.join(", "),
        values.join(", ")
    );

    execute_statement(client, &sql).await?;
    Ok(1)
}

async fn update_record(
    client: &Client,
    database: &str,
    table: &str,
    pk_col: &str,
    pk_val: &JsonValue,
    col_name: &str,
    new_val: &JsonValue,
) -> Result<u64, String> {
    let sql = format!(
        "ALTER TABLE `{}`.`{}` UPDATE `{}` = {} WHERE `{}` = {}",
        escape_identifier(database),
        escape_identifier(table),
        escape_identifier(col_name),
        format_value_for_sql(new_val),
        escape_identifier(pk_col),
        format_value_for_sql(pk_val),
    );

    execute_statement(client, &sql).await?;
    Ok(1)
}

async fn delete_record(
    client: &Client,
    database: &str,
    table: &str,
    pk_col: &str,
    pk_val: &JsonValue,
) -> Result<u64, String> {
    let sql = format!(
        "DELETE FROM `{}`.`{}` WHERE `{}` = {}",
        escape_identifier(database),
        escape_identifier(table),
        escape_identifier(pk_col),
        format_value_for_sql(pk_val),
    );

    execute_statement(client, &sql).await?;
    Ok(1)
}

async fn drop_index(
    client: &Client,
    database: &str,
    table: &str,
    index_name: &str,
) -> Result<(), String> {
    let sql = format!(
        "ALTER TABLE `{}`.`{}` DROP INDEX `{}`",
        escape_identifier(database),
        escape_identifier(table),
        escape_identifier(index_name),
    );
    execute_statement(client, &sql).await
}

// ---------------------------------------------------------------------------
// Batch / snapshot
// ---------------------------------------------------------------------------

async fn get_schema_snapshot(client: &Client, database: &str) -> Result<JsonValue, String> {
    let tables_val = get_tables(client, database).await?;
    let tables = tables_val.as_array().cloned().unwrap_or_default();

    let mut columns_map = serde_json::Map::new();
    for table in &tables {
        let name = table["name"].as_str().unwrap_or("").to_string();
        let cols = get_columns(client, database, &name)
            .await
            .unwrap_or(json!([]));
        columns_map.insert(name, cols);
    }

    Ok(json!({
        "tables": tables,
        "columns": columns_map,
        "foreign_keys": {},
    }))
}

async fn get_all_columns_batch(client: &Client, database: &str) -> Result<JsonValue, String> {
    let sql = format!(
        "SELECT table, name, type, default_kind, default_expression, comment, is_in_primary_key \
         FROM system.columns \
         WHERE database = '{}' \
         ORDER BY table, position",
        escape_string(database)
    );
    let (_, rows) = query_json_rows(client, &sql).await?;

    let mut result: serde_json::Map<String, JsonValue> = serde_json::Map::new();

    for row in &rows {
        let table = row.first().and_then(|v| v.as_str()).unwrap_or("").to_string();
        let name = row.get(1).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let data_type = row.get(2).and_then(|v| v.as_str()).unwrap_or("String").to_string();
        let is_nullable = data_type.starts_with("Nullable");
        let default_kind = row.get(3).and_then(|v| v.as_str()).unwrap_or("");
        let default_expr = row.get(4).and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string());
        let is_pk = row.get(6).map(|v| v == &json!(1) || v.as_str() == Some("1")).unwrap_or(false);

        let col = json!({
            "name": name,
            "data_type": data_type,
            "is_primary_key": is_pk,
            "is_nullable": is_nullable,
            "is_auto_increment": false,
            "column_default": if default_kind.is_empty() { JsonValue::Null } else { default_expr.into() },
            "comment": row.get(5).and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()),
        });

        result
            .entry(table)
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .unwrap()
            .push(col);
    }

    Ok(JsonValue::Object(result))
}

async fn get_create_table_sql(
    client: &Client,
    database: &str,
    table_name: &str,
) -> Result<JsonValue, String> {
    let sql = format!(
        "SHOW CREATE TABLE `{}`.`{}`",
        escape_identifier(database),
        escape_identifier(table_name)
    );
    let (_, rows) = query_json_rows(client, &sql).await?;

    let ddl = rows
        .first()
        .and_then(|row| row.first())
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(json!(ddl))
}

// ---------------------------------------------------------------------------
// Value helpers
// ---------------------------------------------------------------------------

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

fn escape_identifier(s: &str) -> String {
    s.replace('`', "``")
}

fn format_value_for_sql(val: &JsonValue) -> String {
    match val {
        JsonValue::Null => "NULL".to_string(),
        JsonValue::Bool(b) => if *b { "1".to_string() } else { "0".to_string() },
        JsonValue::Number(n) => n.to_string(),
        JsonValue::String(s) => format!("'{}'", escape_string(s)),
        JsonValue::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_value_for_sql).collect();
            format!("[{}]", items.join(", "))
        }
        JsonValue::Object(_) => format!("'{}'", escape_string(&val.to_string())),
    }
}
