/// Integration smoke-test for the ClickHouse plugin JSON-RPC interface.
/// Spawns the plugin binary and sends a sequence of JSON-RPC requests.
///
/// Usage: cargo run --bin test_plugin
use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn main() {
    let mut child = Command::new("cargo")
        .args(["run", "--bin", "tabularis-clickhouse-plugin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to spawn plugin process");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");

    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            println!("PLUGIN: {}", line.unwrap());
        }
    });

    let conn_params = json!({
        "driver": "clickhouse",
        "host": "localhost",
        "port": 8123,
        "database": "default",
        "username": "default",
        "password": "",
        "ssl": false
    });

    let requests = vec![
        json!({
            "jsonrpc": "2.0",
            "method": "test_connection",
            "params": { "params": conn_params },
            "id": 1
        }),
        json!({
            "jsonrpc": "2.0",
            "method": "get_databases",
            "params": { "params": conn_params },
            "id": 2
        }),
        json!({
            "jsonrpc": "2.0",
            "method": "get_tables",
            "params": { "params": conn_params, "schema": "default" },
            "id": 3
        }),
        json!({
            "jsonrpc": "2.0",
            "method": "execute_query",
            "params": {
                "params": conn_params,
                "query": "SELECT version()",
                "limit": 10,
                "page": 1
            },
            "id": 4
        }),
    ];

    for req in requests {
        let mut req_str = serde_json::to_string(&req).unwrap();
        req_str.push('\n');
        println!("SENDING: {}", req_str.trim());
        stdin.write_all(req_str.as_bytes()).unwrap();
        stdin.flush().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    drop(stdin);
    child.wait().unwrap();
}
