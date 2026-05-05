use crate::infer::ColumnSchema;

pub fn generate_create_table(table_name: &str, cols: &[ColumnSchema], dialect: &str) -> String {
    // Normalize the dialect name so the caller can pass uppercase or lowercase values.
    let dialect = dialect.to_lowercase();

    //check the inferred type of each column and map it to dialect
    let mut parts: Vec<String> = Vec::with_capacity(cols.len());
    for c in cols {
        let t = map_type(&c.data_type, &dialect);
        let null_str = if c.nullable { " NULL" } else { "" };
        parts.push(format!("{} {}{}", c.name, t, null_str));
    }

    format!("CREATE TABLE {} (\n    {}\n);", table_name, parts.join(",\n    "))
}

fn map_type(inferred: &str, dialect: &str) -> &'static str {
    //mappings
    match (inferred, dialect) {
        //Postgres
        //mySQL
        //SparkSQL
        //SQL Server

        ("BOOLEAN", "postgres") => "BOOLEAN",
        ("BOOLEAN", "mysql") => "TINYINT(1)",
        ("BOOLEAN", "sparksql") => "BOOLEAN",
        ("BOOLEAN", "sqlserver") => "BIT",

        ("INTEGER", "postgres") => "INTEGER",
        ("INTEGER", "mysql") => "INT",
        ("INTEGER", "sparksql") => "INT",
        ("INTEGER", "sqlserver") => "INT",

        ("BIGINT", "postgres") => "BIGINT",
        ("BIGINT", "mysql") => "BIGINT",
        ("BIGINT", "sparksql") => "BIGINT",
        ("BIGINT", "sqlserver") => "BIGINT",

        ("DOUBLE", "postgres") => "DOUBLE PRECISION",
        ("DOUBLE", "mysql") => "DOUBLE",
        ("DOUBLE", "sparksql") => "DOUBLE",
        ("DOUBLE", "sqlserver") => "FLOAT",

        ("TIMESTAMP", "postgres") => "TIMESTAMP",
        ("TIMESTAMP", "mysql") => "DATETIME",
        ("TIMESTAMP", "sparksql") => "TIMESTAMP",
        ("TIMESTAMP", "sqlserver") => "DATETIME2",

        ("TEXT", "postgres") => "TEXT",
        ("TEXT", "mysql") => "TEXT",
        ("TEXT", "sparksql") => "STRING",
        ("TEXT", "sqlserver") => "NVARCHAR(MAX)",

        //fallback for unknowns
        (_, _) => "TEXT",
    }
}
