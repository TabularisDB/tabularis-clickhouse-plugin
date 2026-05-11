<div align="center">
  <img src="https://raw.githubusercontent.com/debba/tabularis/main/public/logo-sm.png" width="120" height="120" />
</div>

# tabularis-clickhouse-plugin
<p align="center">

![](https://img.shields.io/github/release/debba/tabularis-clickhouse-plugin.svg?style=flat)
![](https://img.shields.io/github/downloads/debba/tabularis-clickhouse-plugin/total.svg?style=flat)
![Build & Release](https://github.com/debba/tabularis-clickhouse-plugin/workflows/Release/badge.svg)
[![Discord](https://img.shields.io/discord/1470772941296894128?color=5865F2&logo=discord&logoColor=white)](https://discord.com/invite/K2hmhfHRSt)

</p>

A [ClickHouse](https://clickhouse.com/) plugin for [Tabularis](https://github.com/debba/tabularis), the lightweight database management tool.

This plugin enables Tabularis to connect to any ClickHouse instance and provides table and view browsing, inline CRUD, index management, and SQL query execution through a JSON-RPC 2.0 over stdio interface.

**Discord** - [Join our discord server](https://discord.com/invite/K2hmhfHRSt) and chat with the maintainers.

## Table of Contents

- [Features](#features)
- [Supported ClickHouse Data Types](#supported-clickhouse-data-types)
- [Installation](#installation)
  - [Automatic (via Tabularis)](#automatic-via-tabularis)
  - [Manual Installation](#manual-installation)
- [How It Works](#how-it-works)
- [Query Syntax](#query-syntax)
- [Supported Operations](#supported-operations)
- [Building from Source](#building-from-source)
- [Development](#development)
- [Changelog](#changelog)
- [License](#license)

## Features

- **Connection** — Connect to any ClickHouse instance via host/port with optional authentication and TLS.
- **Database & Table Browsing** — List databases (schemas), tables, and their columns with native ClickHouse types.
- **View Support** — List views, retrieve their definitions and column metadata.
- **Index Inspection** — List data-skipping indexes with name, type, columns, and granularity.
- **Query Execution** — Run arbitrary SQL queries with pagination support.
- **Inline Editing** — Insert, update, and delete rows directly from the Tabularis data grid.
- **DDL Generation** — Generates `CREATE TABLE`, `ALTER TABLE ADD/MODIFY/RENAME COLUMN`, and `ADD INDEX` statements.
- **Cross-platform** — Pre-built binaries for Linux (x86_64, aarch64), macOS (x86_64, aarch64), and Windows (x86_64).

## Supported ClickHouse Data Types

| Category | Types |
|---|---|
| **Integer** | Int8, Int16, Int32, Int64, Int128, Int256, UInt8, UInt16, UInt32, UInt64, UInt128, UInt256 |
| **Floating point** | Float32, Float64, Decimal |
| **String** | String, FixedString |
| **Date/Time** | Date, Date32, DateTime, DateTime64 |
| **Other** | Boolean, UUID, IPv4, IPv6, Array, Tuple, Map, Nullable, LowCardinality, Enum8, Enum16 |

## Installation

### Automatic (via Tabularis)

If your version of Tabularis supports plugin management, the ClickHouse plugin can be installed directly from the application.

### Manual Installation

1. Download the latest release for your platform from the [Releases page](https://github.com/debba/tabularis-clickhouse-plugin/releases).
2. Extract the archive.
3. Copy `tabularis-clickhouse-plugin` (or `tabularis-clickhouse-plugin.exe` on Windows) and `manifest.json` into the Tabularis plugins directory:

| OS | Plugins Directory |
|---|---|
| **Linux** | `~/.local/share/tabularis/plugins/clickhouse/` |
| **macOS** | `~/Library/Application Support/com.debba.tabularis/plugins/clickhouse/` |
| **Windows** | `%APPDATA%\com.debba.tabularis\plugins\clickhouse\` |

4. Restart Tabularis.

## How It Works

The plugin is a standalone Rust binary that communicates with Tabularis through **JSON-RPC 2.0 over stdio**:

1. Tabularis spawns the plugin as a child process.
2. Requests are sent as newline-delimited JSON-RPC messages to the plugin's `stdin`.
3. The plugin connects to ClickHouse via its HTTP interface and writes responses to `stdout`.

Connection state is pooled per URL — the same `Client` instance is reused for all operations within a session.

## Query Syntax

The `execute_query` method accepts standard ClickHouse SQL:

```sql
-- Select rows
SELECT * FROM system.tables WHERE database = 'default'

-- Aggregation
SELECT database, count() AS table_count
FROM system.tables
GROUP BY database
ORDER BY table_count DESC

-- DDL
CREATE TABLE my_db.events
(
    id       UInt64,
    event    String,
    ts       DateTime DEFAULT now()
)
ENGINE = MergeTree
ORDER BY (ts, id)

-- Add an index
ALTER TABLE my_db.events ADD INDEX idx_event (event) TYPE minmax GRANULARITY 1
```

## Supported Operations

| Method | Description |
|---|---|
| `test_connection` | Ping the ClickHouse server |
| `get_databases` | List all databases |
| `get_schemas` | List all databases (alias for `get_databases`) |
| `get_tables` | List tables in the given database |
| `get_columns` | Return column names and types for a table or view |
| `get_foreign_keys` | Returns `[]` (ClickHouse has no FK constraints) |
| `get_indexes` | List data-skipping indexes for a table |
| `get_views` | List views in the given database |
| `get_view_definition` | Return the `CREATE VIEW` statement for a view |
| `get_view_columns` | Return column metadata for a view |
| `get_routines` / `get_routine_parameters` | Returns `[]` (no stored routines in ClickHouse) |
| `execute_query` | Run a SQL query with optional pagination |
| `insert_record` | Insert a new row |
| `update_record` | Update a single column value by primary key |
| `delete_record` | Delete a row by primary key |
| `get_schema_snapshot` | Full schema dump (all tables + columns) |
| `get_all_columns_batch` | Column metadata for all tables in a database |
| `get_all_foreign_keys_batch` | Returns `{}` |
| `get_create_table_sql` | Fetches the `CREATE TABLE` statement from `SHOW CREATE TABLE` |
| `get_add_column_sql` | Generates `ALTER TABLE ... ADD COLUMN ...` |
| `get_alter_column_sql` | Generates `ALTER TABLE ... RENAME/MODIFY COLUMN ...` |
| `get_create_index_sql` | Generates `ALTER TABLE ... ADD INDEX ... TYPE minmax` |
| `get_create_foreign_key_sql` | Returns a not-supported note |
| `drop_index` | Drops a data-skipping index |
| `drop_foreign_key` | Returns a not-supported error |

## Building from Source

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (edition 2021)
- A running ClickHouse instance (for integration tests)

### Build

```bash
cargo build --release
```

The binary will be located at `target/release/tabularis-clickhouse-plugin`.

### Install Locally

A convenience script is provided to build and copy the plugin to the Tabularis plugins directory:

```bash
./sync.sh
```

## Development

### Testing the Plugin

A simulated Tabularis integration test is included (spawns the plugin and sends JSON-RPC requests via stdio):

```bash
cargo run --bin test_plugin
```

### Manual JSON-RPC test via shell

```bash
echo '{"jsonrpc":"2.0","method":"test_connection","params":{"params":{"host":"localhost","port":8123,"database":"default","username":"default","password":"","ssl":false}},"id":1}' \
  | ./target/release/tabularis-clickhouse-plugin
```

### Tech Stack

- **Language:** Rust (edition 2021)
- **Database driver:** [clickhouse](https://crates.io/crates/clickhouse) v0.13 (async HTTP client)
- **Serialization:** serde + serde_json
- **Async runtime:** tokio
- **Protocol:** JSON-RPC 2.0 over stdio

## [Changelog](./CHANGELOG.md)

## License

Apache License 2.0
