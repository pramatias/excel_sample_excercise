mod data_gen;
mod utils;
mod reader;

use anyhow::Result;
use clap::{ArgAction, Args, Parser, Subcommand};
use log::{LevelFilter, debug};
use std::path::PathBuf;

use init::initialize_logger;

use crate::data_gen::{WriteError, generate_records, write_records};
use crate::reader::ExcelReader;

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

    // Instantiate the reader; this opens the workbook and parses all sheets
    let reader = ExcelReader::new(&args.file)
        .map_err(|e| anyhow::anyhow!("Failed to initialize ExcelReader: {}", e))?;

    log::info!(
        "Successfully read {} records from {}",
        reader.records.len(),
        args.file
    );

    // Use the reader's built-in pretty‐print
    reader.pretty_print();

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
