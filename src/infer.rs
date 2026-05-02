// Determines what data type each entry in the CSV
// Each data type is narrower than the last for determining the best fit for the column.
// Narrowest:
// 1. BOOLEAN: true/false, 1/0
// 2. INTEGER: whole numbers within i32 range
// 3. BIGINT: whole numbers within i64 range
// 4. DOUBLE: any valid floating point number
// 5. TIMESTAMP: any value that can be parsed as a date or datetime
// 6. TEXT: anything else (widest) - Strings
// These data types are mapped to the equivalent type in the SQL dialect the user choose
use anyhow::Result;
use chrono::DateTime;
use csv::ReaderBuilder;
use regex::Regex;
use serde::Serialize;
use std::fs::File;

const I32_MIN_I64: i64 = i32::MIN as i64;
const I32_MAX_I64: i64 = i32::MAX as i64;

#[derive(Debug, Serialize, Clone)]
pub struct ColumnSchema {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

//determine if is a boolean
fn is_bool(s: &str) -> bool {
    matches!(s.to_lowercase().as_str(), "true" | "false" | "1" | "0")
}

pub fn default_table_name(path: &str) -> String {
    //when making table name, use the csv file name
    std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("inferred_table")
        .to_string()
}
//parse the CSV headers and sample the data to determine what data type each column is
pub fn infer_from_path(path: &str, sample_size: usize, delimiter: char) -> Result<Vec<ColumnSchema>> {
    let f = File::open(path)?;
    let mut rdr = ReaderBuilder::new()
        .delimiter(delimiter as u8)
        .has_headers(true)
        .flexible(true)  // Allow rows with different field counts; treat missing as empty
        .from_reader(f);

    let headers = rdr.headers()?.clone();
    let ncols = headers.len();

    //track what types are still possible for a column
    let mut nulls = vec![0usize; ncols];
    let mut can_int = vec![true; ncols];
    let mut can_float = vec![true; ncols];
    let mut can_bool = vec![true; ncols];
    let mut can_ts = vec![true; ncols];
    //used for determining bigint or integer
    let mut int_min: Vec<Option<i64>> = vec![None; ncols];
    let mut int_max: Vec<Option<i64>> = vec![None; ncols];

    //timestamp format
    let iso_re = Regex::new(r"^\d{4}-\d{2}-\d{2}([T ]\d{2}:\d{2}:\d{2})?").unwrap();

    // Read up to `sample_size` data rows and narrow down the possible types.
    for (i, result) in rdr.records().enumerate() {
        if i >= sample_size {
            break;
        }
        let record = result?;
        for col in 0..ncols {
            let val = record.get(col).unwrap_or("").trim();
            if val.is_empty() {
                //empty cell: nullable and skip for type checks
                nulls[col] += 1;
                continue;
            }
            //check if can be integer/bigint
            if can_int[col] {
                match val.parse::<i64>() {
                    Ok(parsed) => { //check if fits into i32
                        int_min[col] = Some(match int_min[col] {
                            Some(current_min) => current_min.min(parsed),
                            None => parsed, 
                        });
                        int_max[col] = Some(match int_max[col] {
                            Some(current_max) => current_max.max(parsed),
                            None => parsed, 
                        });
                    }
                    Err(_) => { //not an integer
                        can_int[col] = false;
                    }
                }
            }
            //check if can be float
            if can_float[col] {
                if val.parse::<f64>().is_err() { //not a float
                    can_float[col] = false;
                }
            }
            //check if can be boolean
            if can_bool[col] {
                if !is_bool(val) { //not a boolean
                    can_bool[col] = false;
                }
            }
            //check if can be timestamp
            if can_ts[col] {
                let try_ts = DateTime::parse_from_rfc3339(val).is_ok() || iso_re.is_match(val);
                if !try_ts { //not a timestamp
                    can_ts[col] = false;
                }
            }
        }
    }

    let mut out = Vec::with_capacity(ncols);
    for col in 0..ncols {
        //use header, if no header use col1, col2, etc
        let name = headers.get(col).unwrap_or(&format!("col{}", col)).to_string();
        //empty cell, nullable
        let nullable = nulls[col] > 0;


        //choose the best fit
        let data_type = if can_bool[col] {
            "BOOLEAN"
        } else if can_int[col] {
            let min_val = int_min[col].unwrap_or(0);
            let max_val = int_max[col].unwrap_or(0);
            if min_val < I32_MIN_I64 || max_val > I32_MAX_I64 { //determine if big int or integer
                "BIGINT"
            } else {
                "INTEGER"
            }
        } else if can_float[col] {
            "DOUBLE"
        } else if can_ts[col] {
            "TIMESTAMP"
        } else {
            "TEXT"
        }
        .to_string();

        out.push(ColumnSchema { name, data_type, nullable });
    }

    Ok(out)
}