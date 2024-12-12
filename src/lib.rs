use chrono::{DateTime, NaiveDateTime, ParseError, Utc};
use station_struts::{FuelStationData, PriceLastUpdated, StationPriceLastUpdated, StationPrices};

pub mod station_struts;
/// Processes fuel station data from a JSON string, transforming it into a structured format.
///
/// This function performs a multi-step transformation of fuel station data:
/// 1. Deserializes the input JSON into a `FuelStationData` struct
/// 2. Extracts the last updated timestamp and stations
/// 3. Processes station data and adds last updated information
///
/// # Parameters
///
/// - `json_data`: A JSON-formatted string containing fuel station information
///
/// # Returns
///
/// A vector of `StationPriceLastUpdated` structs, each containing:
/// - Station details (site ID, brand, address, etc.)
/// - Prices
/// - Last updated timestamp
///
/// # Process Flow
///
/// - Deserialize JSON using `serde_json`
/// - Parse the last updated timestamp
/// - Process individual stations
/// - Add last updated timestamp to each station's price information
///
/// # Behavior
///
/// - Returns an empty vector if no stations are present
/// - Propagates parsing errors via `expect()`
///
/// # Examples
///
/// ```rust
/// let json = r#"{"last_updated": "2023-01-01T12:00:00Z", "stations": [...]}"#;
/// let processed_stations = process_data(json);
/// ```
///
/// # Potential Panics
///
/// - Panics if JSON is invalid
/// - Panics if timestamp parsing fails
/// - Panics if station serialization fails
pub fn process_data(json_data: &str) -> Vec<StationPriceLastUpdated> {
    let data: FuelStationData = serde_json::from_str(json_data).expect("Invalid JSON");
    // println!("=== data:\n{:#?}", data);
    let FuelStationData {
        last_updated,
        stations,
    } = data;

    if !stations.is_empty() {
        let last_updated_parsed = parse_datetime(&last_updated).unwrap();
        let stations_json = serde_json::to_string(&stations).unwrap();
        let processed_stations = process_stations(&stations_json);

        let stations_with_last_updated: Vec<StationPriceLastUpdated> = processed_stations
            .into_iter()
            .map(|station| StationPriceLastUpdated {
                site_id: station.site_id,
                brand: station.brand,
                address: station.address,
                postcode: station.postcode,
                location: station.location,
                prices: vec![PriceLastUpdated {
                    prices: station.prices,
                    lu: last_updated_parsed.to_string(),
                }],
            })
            .collect();

        stations_with_last_updated
    } else {
        let nothing: Vec<StationPriceLastUpdated> = vec![];
        nothing
    }
}

/// Processes JSON station data and extracts valid `StationPrices` entries.
///
/// This function performs the following operations:
/// 1. Parses the input JSON string into a vector of JSON values
/// 2. Attempts to convert each JSON value into a `StationPrices` struct
/// 3. Filters out any conversion failures, returning only successfully parsed entries
///
/// # Parameters
///
/// - `json_data`: A string slice containing JSON-formatted station data
///
/// # Returns
///
/// A vector of `StationPrices` structs successfully parsed from the input JSON
///
/// # Parsing Strategy
///
/// - Uses `serde_json::from_str` to parse the JSON string
/// - Falls back to an empty vector if initial parsing fails
/// - Converts individual JSON values to `StationPrices` using `serde_json::from_value`
/// - Filters out any entries that fail to convert
///
/// # Examples
///
/// ```rust
/// let json = r#"[{"id": 1, "name": "Station A"}, {"id": 2, "name": "Station B"}]"#;
/// let stations = process_stations(json);
/// assert!(!stations.is_empty());
/// ```
///
/// # Performance
///
/// - Uses iterator-based processing for efficiency
/// - Minimal memory overhead with `filter_map`
///
/// # Errors
///
/// - Silently handles JSON parsing and conversion errors
/// - Returns only successfully parsed `StationPrices` entries
fn process_stations(json_data: &str) -> Vec<StationPrices> {
    let result: Vec<Result<StationPrices, serde_json::Error>> =
        serde_json::from_str::<Vec<serde_json::Value>>(json_data)
            .unwrap_or_default()
            .into_iter()
            .map(serde_json::from_value::<StationPrices>)
            .collect();

    result.into_iter().filter_map(Result::ok).collect()
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
