use fake::faker::company::en::CompanyName;
use fake::faker::phone_number::en::PhoneNumber;
use fake::faker::name::en::Name;
use fake::Fake;
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;
use thiserror::Error;

// Import the xlsx writer and its error type
use umya_spreadsheet::{new_file, Worksheet};
use umya_spreadsheet::writer::xlsx::XlsxError;
use fake::rand;
use fake::Rng;
use fake::rand::prelude::SliceRandom;
use quick_xml::Error as XmlError;

use crate::utils::Record;

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("XML error: {0}")]
    Xml(#[from] XmlError),

    #[error("XLSX error: {0:?}")]
    Xlsx(XlsxError),

    #[error("Worksheet creation error: {0}")]
    Sheet(&'static str),
}

// 2) Provide the From<XlsxError> impl yourself:
impl From<XlsxError> for WriteError {
    fn from(e: XlsxError) -> Self {
        WriteError::Xlsx(e)
    }
}

pub fn write_records(path: &Path, records: &[Record]) -> Result<(), WriteError> {
    let mut wb = new_file();
    let ws: &mut Worksheet = wb
        .new_sheet("Eu_Data")
        .map_err(WriteError::Sheet)?;

    // Helper to write a row
    fn write_row(ws: &mut Worksheet, row: u32, vals: &[&str]) {
        for (col, v) in vals.iter().enumerate() {
            ws.get_cell_by_column_and_row_mut((col + 1) as u32, row)
              .set_value(*v);
        }
    }

    // Header (now includes TotalOrder and RecentOrder)
    write_row(
        ws,
        1,
        &["ID", "Region", "Municipality", "Company", "Phone", "Contact", "TotalOrder", "RecentOrder"],
    );

    // Data rows
    for (i, rec) in records.iter().enumerate() {
        write_row(
            ws,
            (i + 2) as u32,
            &[
                &rec.id.to_string(),
                &rec.region,
                &rec.municipality,
                &rec.company,
                &rec.phone,
                &rec.contact,
                &rec.total_order.to_string(),
                &rec.recent_order.to_string(),
            ],
        );
    }

    umya_spreadsheet::writer::xlsx::write(&wb, path)?;
    Ok(())
}

/// Generates `count` fake `Record`s in-memory
pub fn generate_records(count: usize) -> Vec<Record> {
    // region → municipalities map
    let mut eu_map: HashMap<&str, Vec<&str>> = HashMap::new();
    eu_map.insert("Bavaria (DE)", vec!["Munich", "Nuremberg", "Augsburg"]);
    eu_map.insert("Île-de-France (FR)", vec!["Paris", "Boulogne-Billancourt", "Saint-Denis"]);
    eu_map.insert("Lombardy (IT)", vec!["Milan", "Bergamo", "Brescia"]);
    eu_map.insert("Andalusia (ES)", vec!["Seville", "Málaga", "Granada"]);
    let regions: Vec<_> = eu_map.keys().copied().collect();

    let mut rng = rand::thread_rng();
    let mut out = Vec::with_capacity(count);

    for _ in 0..count {
        let region = *regions.choose(&mut rng).unwrap();
        let municipality = *eu_map.get(region).unwrap().choose(&mut rng).unwrap();

        // Generate total and recent orders, ensuring recent <= total
        let total_order: u32 = rng.gen_range(0..=1_000);            // or any upper bound you prefer
        let recent_order: u32 = rng.gen_range(0..=total_order);

        out.push(Record {
            id: Uuid::new_v4(),
            region: region.to_string(),
            municipality: municipality.to_string(),
            company: CompanyName().fake(),
            phone: PhoneNumber().fake(),
            contact: Name().fake(),
            total_order,
            recent_order,
        });
    }

    out
}
