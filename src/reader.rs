use anyhow::Result;
// use log::debug;
use std::path::PathBuf;
use calamine::{open_workbook_auto, Reader, DataType, Error as CalamineError};
use uuid::Uuid;
use prettytable::{Table, Row, Cell};
use thiserror::Error;

use crate::utils::Record;

/// Errors that can occur when reading or processing the workbook
#[derive(Error, Debug)]
pub enum ExcelReaderError {
    #[error("I/O or format error: {0}")]
    Calamine(#[from] CalamineError),

    #[error("UUID parse error for '{0}'")]
    UuidParse(String),
}

/// Reader for Excel files containing Record data
pub struct ExcelReader {
    pub records: Vec<Record>,
}

impl ExcelReader {
    /// Create a new reader and load all records from the given file path
    pub fn new<P: Into<PathBuf>>(file: P) -> Result<Self, ExcelReaderError> {
        let path = file.into();
        let mut workbook = open_workbook_auto(&path)?;
        let mut all_records = Vec::new();

        for sheet_name in workbook.sheet_names().to_owned() {
            let range = workbook
                .worksheet_range(&sheet_name)
                .map_err(ExcelReaderError::Calamine)?;

            // Skip header row
            for (_i, row) in range.rows().skip(1).enumerate() {
                let id_str = row
                    .get(0)
                    .and_then(DataType::get_string)
                    .ok_or_else(|| ExcelReaderError::UuidParse("<missing>".to_string()))?;
                let id = Uuid::parse_str(id_str)
                    .map_err(|_| ExcelReaderError::UuidParse(id_str.to_string()))?;

                let region       = row.get(1).map(|c| c.to_string()).unwrap_or_default();
                let municipality = row.get(2).map(|c| c.to_string()).unwrap_or_default();
                let company      = row.get(3).map(|c| c.to_string()).unwrap_or_default();
                let phone        = row.get(4).map(|c| c.to_string()).unwrap_or_default();
                let contact      = row.get(5).map(|c| c.to_string()).unwrap_or_default();

                let total_order = row.get(6)
                    .and_then(DataType::get_float)
                    .unwrap_or(0.0) as u32;
                let recent_order = row.get(7)
                    .and_then(DataType::get_float)
                    .unwrap_or(0.0) as u32;

                all_records.push(Record { id, region, municipality, company, phone, contact, total_order, recent_order });
            }
        }

        Ok(ExcelReader {records: all_records })
    }

    /// Pretty-print the loaded records as a table
    pub fn pretty_print(&self) {
        let mut table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new("ID"),
            Cell::new("Region"),
            Cell::new("Municipality"),
            Cell::new("Company"),
            Cell::new("Phone"),
            Cell::new("Contact"),
            Cell::new("Total Order"),
            Cell::new("Recent Order"),
        ]));

        for rec in &self.records {
            table.add_row(Row::new(vec![
                Cell::new(&rec.id.to_string()),
                Cell::new(&rec.region),
                Cell::new(&rec.municipality),
                Cell::new(&rec.company),
                Cell::new(&rec.phone),
                Cell::new(&rec.contact),
                Cell::new(&rec.total_order.to_string()),
                Cell::new(&rec.recent_order.to_string()),
            ]));
        }

        table.printstd();
    }
}
