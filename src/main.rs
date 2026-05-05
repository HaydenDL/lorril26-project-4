use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

mod infer;
mod ddl;

//pull command line arguements
#[derive(Parser, Debug)]
#[command(author, version, about = "Infer SQL schema from CSV files", long_about = None)]
#[command(arg_required_else_help = true)]
struct Args {
    //path to input CSV file
    #[arg(short = 'f', long = "file", value_name = "CSV_FILE", value_hint = clap::ValueHint::FilePath)]
    file: PathBuf,

    //check which SQL dialect to use for output (postgres, mysql, sparksql, sqlserver)
    #[arg(short = 'd', long = "dialect", value_enum, default_value_t = Dialect::Postgres)]
    dialect: Dialect,

    //number of rows to sample for infering data types (default 10000)
    #[arg(short = 's', long = "sample-size", default_value = "10000", value_parser = parse_sample_size)]
    sample_size: usize,

    //output file, when not used it prints to terminal
    #[arg(short = 'o', long = "output", value_name = "OUTPUT_FILE", value_hint = clap::ValueHint::FilePath)]
    output: Option<PathBuf>,

    //name to use for table, if not used the csv file name is used
    #[arg(short = 't', long = "table-name", value_name = "TABLE_NAME")]
    table_name: Option<String>,

    //delimeter used, default ','
    #[arg(long = "delimiter", default_value = ",", value_parser = parse_delimiter)]
    delimiter: String,

    //print additional runtime details to help debug CLI behavior
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,
}
//different SQL dialects we support
#[derive(Copy, Clone, Debug, ValueEnum)]
enum Dialect {
    Postgres,
    Mysql,
    Sparksql,
    Sqlserver,
}

impl Dialect {
    fn as_str(self) -> &'static str {
        match self {
            Dialect::Postgres => "postgres",
            Dialect::Mysql => "mysql",
            Dialect::Sparksql => "sparksql",
            Dialect::Sqlserver => "sqlserver",
        }
    }
}

//check if deliminator is exactly one character, if not return error
fn parse_delimiter(raw: &str) -> Result<String, String> {
    let mut chars = raw.chars();
    match (chars.next(), chars.next()) {
        (Some(ch), None) => Ok(ch.to_string()),
        _ => Err(String::from("delimiter must be exactly one character")),
    }
}

//check if sample size is a positive whole numb, if not return error
fn parse_sample_size(raw: &str) -> Result<usize, String> {
    let value = raw
        .parse::<usize>()
        .map_err(|_| String::from("sample-size must be a positive whole number"))?;

    if value == 0 {
        Err(String::from("sample-size must be greater than zero"))
    } else {
        Ok(value)
    }
}

fn run(args: Args) -> Result<()> {
    //fail if no input file/doesnt exist
    if !args.file.exists() {
        anyhow::bail!("input file not found: {}", args.file.display());
    }

    //check if output file is valid if provided
    let delimiter = args.delimiter.chars().next().unwrap_or(',');
    let file_path = args.file.to_string_lossy().to_string();

    //prepare table with name or csv filename
    let table_name = args
        .table_name
        .clone()
        .unwrap_or_else(|| infer::default_table_name(&file_path));

    if args.verbose {
        eprintln!("[verbose] input file: {}", args.file.display());
        eprintln!("[verbose] dialect: {}", args.dialect.as_str());
        eprintln!("[verbose] sample size: {}", args.sample_size);
        eprintln!("[verbose] delimiter: {:?}", delimiter);
        eprintln!("[verbose] table name: {}", table_name);
    }

    //infer the schema
    let cols = infer::infer_from_path(&file_path, args.sample_size, delimiter)
        .with_context(|| format!("failed to read or infer schema from {}", args.file.display()))?;

    if args.verbose {
        eprintln!("[verbose] inferred columns: {}", cols.len());
    }

    //generate the DDL statement
    let ddl = ddl::generate_create_table(&table_name, &cols, args.dialect.as_str());

    //write to output file or print to terminal
    if let Some(out) = args.output {
        std::fs::write(&out, ddl)
            .with_context(|| format!("failed to write output file {}", out.display()))?;
        if args.verbose {
            eprintln!("[verbose] wrote output to: {}", out.display());
        }
    } else {
        println!("{}", ddl);
    }

    Ok(())
}

fn main() {
    //print error and cause
    if let Err(err) = run(Args::parse()) {
        eprintln!("error: {}", err);
        for cause in err.chain().skip(1) {
            eprintln!("  caused by: {}", cause);
        }
        std::process::exit(1);
    }
}
