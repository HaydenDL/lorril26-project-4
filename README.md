# lorril26-project-4
Hayden Lorrilliere
Data Engineering with Rust Capstone Project 4 - Smart Schema Generator

# Smart Schema Generator
A CLI that samples the first 10,000 rows of a CSV file and suggests a SQL CREATE TABLE statement.
The tool should support multiple database dialects such as:
PostgreSQL
MySQL
SparkSQL
SQL Server

### Build

```powershell
cargo build
```

### Show help

```powershell
cargo run -- --help
```

### Basic run

```powershell
cargo run -- --file <path-to-csv>
```

### Run with dialect

```powershell
cargo run -- --file <path-to-csv> --dialect postgres
```

Supported dialect values:
- postgres
- mysql
- sparksql
- sqlserver

### Flags and Format

Command format:

```powershell
cargo run -- --file <csv_file_path> [--dialect <dialect>] [--sample-size <rows>] [--output <output_sql_file>] [--table-name <table_name>] [--delimiter <delimiter>] [--verbose]
```

Required flag:
- `--file`, `-f` <csv_file_path>: Path to the input CSV file.

Optional flags:
- `--dialect`, `-d` <dialect>: SQL dialect. Valid values: postgres, mysql, sparksql, sqlserver. Default: postgres.
- `--sample-size`, `-s` <rows>: Number of rows to sample for inference. Default: 10000.
- `--output`, `-o` <output_sql_file>: Write generated SQL to a file instead of printing only to stdout.
- `--table-name`, `-t` <table_name>: Name to use in CREATE TABLE.
- `--delimiter` <delimiter>: CSV delimiter character. Default: comma (,).
- `--verbose`, `-v`: Print additional runtime details (input path, dialect, sample size, delimiter, table name, and output path when used).

Short-form format:

```powershell
cargo run -- -f <csv_file_path> -d <dialect> -s <rows> -o <output_sql_file> -t <table_name> -v
```

Notes:
- `--file` must be followed by exactly one file path value.
- If the file path contains spaces, wrap the path in quotes.
- Use either long flags (`--file`) or short flags (`-f`). Both are supported.


