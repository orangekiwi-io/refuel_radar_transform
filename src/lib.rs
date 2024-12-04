#![crate_type = "lib"]

use chrono::{DateTime, NaiveDateTime, ParseError, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub fn hello() {
    println!("hello from refuel_radar_transform")
}

/// Checks structure of input data, not the content
#[derive(Debug, Deserialize)]
struct PartialJsonStructure {
    last_updated: String,             // Ensure this field is a String
    stations: Vec<serde_json::Value>, // Ensure this field is an array
}

/// Represents the raw input data structure for petrol station information
#[derive(Debug, Serialize, Deserialize)]
pub struct RawStationData {
    pub last_updated: String,
    pub stations: Vec<RawStation>,
}

/// Represents a raw station entry
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawStation {
    pub site_id: String,
    pub brand: String,
    pub address: String,
    pub postcode: String,
    pub location: RawLocation,
    pub prices: HashMap<String, f64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum F64OrString {
    F64(f64),
    String(String),
}

impl F64OrString {
    fn as_f64(&self) -> Result<f64, &'static str> {
        match self {
            F64OrString::F64(val) => Ok(*val),
            F64OrString::String(s) => s.parse().map_err(|_| "Failed to parse string to f64"),
        }
    }
}

/// Represents the raw location coordinates
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RawLocation {
    pub latitude: F64OrString,
    pub longitude: F64OrString,
}

/// Represents the converted station data structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StationData {
    pub stations: Vec<Station>,
}

/// Represents a converted station
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Station {
    pub site_id: String,
    pub brand: String,
    pub address: String,
    pub postcode: String,
    pub location: Location,
    pub prices: Vec<PriceEntry>,
}

/// Represents the converted location
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
}

/// Represents a price entry with dynamic price data and timestamp
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PriceEntry {
    #[serde(flatten)]
    pub prices: HashMap<String, f64>,
    pub last_updated: String,
}

/// Errors that can occur during data conversion
#[derive(Debug, Error)]
pub enum ConversionError {
    /// Represents a failure to parse the datetime
    #[error("Failed to parse datetime: {0}")]
    DateTimeParseError(#[from] ParseError),

    /// Represents a JSON parsing error
    #[error("JSON parsing error: {0}")]
    JsonParseError(#[from] serde_json::Error),
}

/// Process raw station data to the target structure
///
/// # Arguments
///
/// * `raw_data` - The raw station data to be converted
///
/// # Returns
///
/// A `Result` containing the converted `StationData` or a `ConversionError`
pub fn check_json_data_structure(data: &str) -> bool {
    println!("*** check_json_data_structure");
    // let bob: PartialJsonStructure = serde_json::from_str(&data).unwrap();
    // println!("{:#?}", bob);

    let parsed: Result<PartialJsonStructure, _> = serde_json::from_str(data);
    match parsed {
        Ok(data_to_check) => {
            // Ensure the stations array has at least one element
            !data_to_check.stations.is_empty()
        }
        Err(_) => false, // Deserialization failed
    }
}

/// Converts raw station data to the target structure
///
/// # Arguments
///
/// * `raw_data` - The raw station data to be converted
///
/// # Returns
///
/// A `Result` containing the converted `StationData` or a `ConversionError`
pub fn convert_station_data(raw_data: RawStationData) -> Result<StationData, ConversionError> {
    let last_updated = parse_datetime(&raw_data.last_updated)?;

    let filtered_stations: Vec<RawStation> = raw_data
        .stations
        .iter()
        .filter(|entry| !entry.brand.is_empty())
        .cloned()
        .collect();

    println!("*** filtered_stations: {:#?}", filtered_stations);

    let stations = raw_data
        .stations
        .into_iter()
        .map(|raw_station| {
            Ok(Station {
                site_id: raw_station.site_id,
                brand: raw_station.brand,
                address: raw_station.address,
                postcode: raw_station.postcode,
                location: Location {
                    lat: raw_station.location.latitude.as_f64().unwrap(),
                    lon: raw_station.location.longitude.as_f64().unwrap(),
                },
                prices: vec![PriceEntry {
                    prices: raw_station.prices,
                    last_updated: last_updated.clone(),
                }],
            })
        })
        .collect::<Result<Vec<Station>, ConversionError>>()?;

    Ok(StationData { stations })
}

/// Parses a datetime string into ISO 8601 format
///
/// # Arguments
///
/// * `dt_str` - A datetime string in the format "dd/MM/yyyy HH:mm:ss"
///
/// # Returns
///
/// A `Result` containing the ISO 8601 formatted datetime string or a `ParseError`
pub fn parse_datetime(dt_str: &str) -> Result<String, ParseError> {
    // Parse the input date format "dd/MM/yyyy HH:mm:ss"
    let naive_dt = NaiveDateTime::parse_from_str(dt_str, "%d/%m/%Y %H:%M:%S")?;

    // Convert to UTC DateTime and then to ISO 8601 format
    let utc_dt: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive_dt, Utc);
    Ok(utc_dt.to_rfc3339())
}

/// Deserializes and converts JSON data
///
/// # Arguments
///
/// * `json_str` - A JSON string containing station data
///
/// # Returns
///
/// A `Result` containing the converted `StationData` or a `ConversionError`
// pub fn convert_json_data(json_str: &str) -> Result<StationData, ConversionError> {
pub fn convert_json_data(json_str: &str) {
    println!("*** lib.rs convert_json_data");
    println!("    json_str {:#?}", json_str);
    // let raw_data: RawStationData = serde_json::from_str(json_str)?;
    // TODO RL. Check data before processing any further.
    // - check last_updated is present
    // - loop through all station information. Exclude any entries that do not pass (brand: null etc)
    // - if price null remove entry
    let raw_data: RawStationData = serde_json::from_str(json_str).unwrap();
    println!("*** raw_data: {:#?}", raw_data);
    // convert_station_data(raw_data)
}

pub enum MathsError {
    DivisionByZero,
    NonPositiveLogarithm,
    NegativeSquareRoot,
}
// Example usage in tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_datetime_parsing() -> Result<(), ConversionError> {
        let input = "27/11/2024 11:45:32";
        let parsed = parse_datetime(input)?;
        assert!(parsed.starts_with("2024-11-27T11:45:32"));
        Ok(())
    }

    #[test]
    fn test_json_conversion() -> Result<(), ConversionError> {
        let json_str = r#"{
            "last_updated": "27/11/2024 11:45:32",
            "stations": [
                {
                    "site_id": "xxx",
                    "brand": "Bob",
                    "address": "The Petrol station",
                    "postcode": "AB1 2CD",
                    "location": {
                        "latitude": 51.5,
                        "longitude": 0
                    },
                    "prices": {
                        "E5": 138.9,
                        "E10": 129.9,
                        "B7": 138.9,
                        "SDV": 0
                    }
                }
            ]
        }"#;

        let converted = convert_json_data(json_str)?;
        assert_eq!(converted.stations.len(), 1);
        assert_eq!(converted.stations[0].site_id, "xxx");
        Ok(())
    }
}
