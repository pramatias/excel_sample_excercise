mod data_gen;

use std::path::Path;

use crate::data_gen::{WriteError, generate_records, write_records};

fn main() -> Result<(), WriteError> {
    // 1. generate 1_000_000 records in memory
    let records = generate_records(10_000);

    // 2. write them out
    let out_path = Path::new("./records.xlsx");
    write_records(out_path, &records)?;
    println!("Generated {} records to {:?}", records.len(), out_path);

    Ok(())
}
