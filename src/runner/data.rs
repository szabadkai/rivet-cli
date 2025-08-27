use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

pub async fn load_csv_data(file_path: &Path) -> Result<Vec<HashMap<String, String>>> {
    let content = fs::read_to_string(file_path)
        .await
        .with_context(|| format!("Failed to read CSV file: {}", file_path.display()))?;

    let mut reader = csv::Reader::from_reader(content.as_bytes());
    let headers = reader
        .headers()
        .context("Failed to read CSV headers")?
        .clone();

    let mut data = Vec::new();
    for record in reader.records() {
        let record = record.context("Failed to parse CSV record")?;
        let mut row = HashMap::new();

        for (i, value) in record.iter().enumerate() {
            if let Some(header) = headers.get(i) {
                row.insert(header.to_string(), value.to_string());
            }
        }

        if !row.is_empty() {
            data.push(row);
        }
    }

    Ok(data)
}
