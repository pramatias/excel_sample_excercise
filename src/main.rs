mod data_gen;
mod utils;

use std::path::PathBuf;

use crate::data_gen::{WriteError, generate_records, write_records};

/// CLI for record generation
#[derive(Parser, Debug)]
#[command(name = "record-gen")]
struct GenArgs {
    /// Output file path
    #[arg(short = 'o', long = "output", help = "Output file path")]
    output: Option<String>,

    /// Number of records to generate
    #[arg(short = 'n', long = "count", help = "Number of records", default_value_t = 10000)]
    count: u32,
}

fn run_gen(args: GenArgs) -> Result<(), WriteError> {
    let count = args.count as usize;
    debug!("Generating {} records", count);
    let records = generate_records(count);

    let out_path = args
        .output
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("records.xlsx"));
    debug!("Writing records to {:?}", out_path);
    write_records(&out_path, &records)?;

    println!("Generated {} records to {:?}", records.len(), out_path);
    Ok(())
}

fn main() -> Result<(), WriteError> {
}
