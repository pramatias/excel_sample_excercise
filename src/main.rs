mod data_gen;
mod utils;

use anyhow::{Context, Result};
use clap::{ArgAction, Args, Parser, Subcommand};
use log::{LevelFilter, debug};
use std::path::{Path, PathBuf};
use calamine::{open_workbook_auto, Reader, DataType};
use uuid::Uuid;

use init::initialize_logger;

use crate::data_gen::{WriteError, generate_records, write_records};
use crate::utils::Record;

/// CLI for record generation and reading
#[derive(Parser, Debug)]
#[command(name = "record-gen", version = "0.1.0", author = "Your Name", about = "Generate or read records")]
struct Cli {
    #[command(flatten)]
    verbose: Verbosity,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate records
    Gen(GenArgs),
    /// Read records from a file
    Read(ReadArgs),
}

/// Arguments for `gen` subcommand
#[derive(Args, Debug)]
struct GenArgs {
    /// Output file path
    #[arg(short = 'o', long = "output", help = "Output file path")]
    output: Option<String>,

    /// Number of records to generate
    #[arg(short = 'n', long = "count", help = "Number of records", default_value_t = 10000)]
    count: u32,
}

/// Arguments for `read` subcommand
#[derive(Args, Debug)]
struct ReadArgs {
    /// Path to the file to read
    #[arg(help = "File to read")]
    file: String,
}

fn main() -> Result<()> {
    // Parse CLI args
    let cli = Cli::parse();

    // Initialize logger
    let level = cli.verbose.log_level_filter();
    initialize_logger(level)?;

    // Dispatch subcommands
    match cli.command {
        Commands::Gen(args) => run_gen(args)?,
        Commands::Read(args) => run_read(args)?,
    }

    Ok(())
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

/// Run the `read` command: read and display records
fn run_read(args: ReadArgs) -> Result<()> {
    log::debug!("Reading records from {}", args.file);

    let path = Path::new(&args.file);
    let mut workbook = open_workbook_auto(path)
        .with_context(|| format!("Failed to open workbook: {}", args.file))?;

    let mut all_records = Vec::new();

    // Iterate over every sheet
    for sheet_name in workbook.sheet_names().to_owned().into_iter() {
        log::debug!("Using sheet '{}'", sheet_name);

        let range = workbook
            .worksheet_range(&sheet_name)
            .with_context(|| format!("Error reading sheet '{}'", sheet_name))?;

        let (total_rows, total_cols) = range.get_size();
        log::debug!("Sheet '{}' dimensions: {} rows x {} columns", sheet_name, total_rows, total_cols);

        // Skip header row
        for (i, row) in range.rows().skip(1).enumerate() {
            let excel_row = i + 2; // 1-based index + header
            if excel_row <= 6 || excel_row % 100 == 0 {
                log::debug!("Sheet '{}': processing row {}", sheet_name, excel_row);
            }

            // UUID in column A
            let id_str = row
                .get(0)
                .and_then(DataType::get_string)
                .context("Invalid or missing UUID cell")?;
            let id = Uuid::parse_str(id_str)
                .with_context(|| format!("UUID parse error for '{}'", id_str))?;

            // String columns
            let region       = row.get(1).map(|c| c.to_string()).unwrap_or_default();
            let municipality = row.get(2).map(|c| c.to_string()).unwrap_or_default();
            let company      = row.get(3).map(|c| c.to_string()).unwrap_or_default();
            let phone        = row.get(4).map(|c| c.to_string()).unwrap_or_default();
            let contact      = row.get(5).map(|c| c.to_string()).unwrap_or_default();

            // Numeric columns
            let total_order = row.get(6)
                .and_then(DataType::get_float)
                .context("Invalid total_order")? as u32;
            let recent_order = row.get(7)
                .and_then(DataType::get_float)
                .context("Invalid recent_order")? as u32;

            let record = Record {
                id,
                region,
                municipality,
                company,
                phone,
                contact,
                total_order,
                recent_order,
            };

            if excel_row <= 6 {
                log::debug!("Sheet '{}', row {} record: {:?}", sheet_name, excel_row, record);
            }

            all_records.push(record);
        }
    }

    println!("Read {} total records from {}", all_records.len(), args.file);
    log::debug!("Successfully processed all sheets");

    for rec in all_records.iter().take(5) {
        println!("{:?}", rec);
    }

    Ok(())
}

/// Configure logging verbosity using -v/--verbose and -q/--quiet flags.
#[derive(Args, Debug)]
pub struct Verbosity {
    /// Increase the level of verbosity (repeatable).
    #[arg(short = 'v', long, action = ArgAction::Count, display_order = 99)]
    pub verbose: u8,

    /// Decrease the level of verbosity (repeatable).
    #[arg(short = 'q', long, action = ArgAction::Count, display_order = 100)]
    pub quiet: u8,
}

impl Verbosity {
    pub fn log_level_filter(&self) -> LevelFilter {
        if self.quiet > 0 {
            LevelFilter::Warn
        } else {
            match self.verbose {
                0 => LevelFilter::Info,
                1 => LevelFilter::Debug,
                _ => LevelFilter::Trace,
            }
        }
    }
}
